use crate::db::Redis;
use deadpool_redis::redis::AsyncCommands;
use oauth2::{basic::BasicTokenType, AccessToken, RefreshToken, Scope};
use rocket::{
    http::Status,
    request::{self, FromRequest},
    serde::{Deserialize, Serialize},
    Request,
};
use rocket_db_pools::Connection;
use serde_json::Value;
use std::time::Duration;
use twilight_http::Client;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Guild {
    pub name: String,
    pub id: String,
    pub icon: Option<String>,
    pub owner: bool,
    pub permissions: String,
    pub features: Vec<String>,
    pub bot_in_guild: Option<bool>,
}

impl Guild {
    pub async fn setup(&mut self, conn: &mut Connection<Redis>) {
        if let Ok(value) = conn
            .sismember::<_, _, bool>("guilds", self.id.clone())
            .await
        {
            self.owner = value;
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub access_token: AccessToken,
    pub token_type: BasicTokenType,
    pub expires_in: Duration,
    pub refresh_token: RefreshToken,
    pub scopes: Vec<Scope>,
}

impl User {
    pub async fn get_data(&self) -> Option<Value> {
        let client = Client::new(format!("Bearer {}", self.access_token.secret()));

        serde_json::from_str(
            &client
                .current_user()
                .exec()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .ok()
    }

    pub async fn get_cache_data(&self, pool: &mut Connection<Redis>) -> Option<Value> {
        let data: Option<String> = pool
            .get(format!("user:{}:data", self.access_token.secret()))
            .await
            .ok();

        if let Some(data) = data {
            Some(serde_json::from_str(&data).unwrap())
        } else {
            let data = self.get_data().await;
            let _: Option<()> = pool
                .set_ex(
                    format!("user:{}:data", self.access_token.secret()),
                    serde_json::to_string(&data).unwrap(),
                    3600,
                )
                .await
                .ok();
            data
        }
    }

    pub async fn get_guilds(&self) -> Option<Vec<Guild>> {
        let client = Client::new(format!("Bearer {}", self.access_token.secret()));
        match client.current_user_guilds().exec().await {
            Ok(response) => {
                let guilds: Vec<Guild> =
                    serde_json::from_str(&response.text().await.unwrap()).unwrap();

                Some(guilds)
            }
            Err(e) => {
                dbg!(e);
                None
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MissingAuthorization {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = MissingAuthorization;

    // use serde_json to create a User from the cookie
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        let segments: Vec<&str> = request.uri().path().segments().collect();
        let guild_id: Option<u64> = if segments.contains(&"guilds") {
            let mut i: usize = 0;
            for x in 0..segments.len() {
                if segments[x] == "guilds" {
                    i = x;
                    break;
                }
            }
            segments
                .get(i + 1)
                .and_then(|s: &&str| s.parse::<u64>().ok())
        } else {
            None
        };
        let cookie = request.cookies().get_private("user");
        match cookie {
            Some(cookie) => {
                let user: User = serde_json::from_str(&cookie.value()).unwrap();

                if guild_id.is_some() {
                    let mut pool = request.guard::<Connection<Redis>>().await.unwrap();

                    let data = user.get_cache_data(&mut pool).await.unwrap();
                    if !pool
                        .exists::<_, bool>(format!(
                            "member.{}.{}",
                            guild_id.unwrap(),
                            data.get("id").unwrap().as_str().unwrap()
                        ))
                        .await
                        .unwrap_or(false)
                    {
                        return request::Outcome::Failure((
                            Status::Unauthorized,
                            MissingAuthorization::Missing,
                        ));
                    }
                }
                request::Outcome::Success(user)
            }
            None => {
                request::Outcome::Failure((Status::Unauthorized, MissingAuthorization::Missing))
            }
        }
    }
}
