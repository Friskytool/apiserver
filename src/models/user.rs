use oauth2::{basic::BasicTokenType, AccessToken, RefreshToken, Scope};
use rocket::{
    http::Status,
    request::{self, FromRequest},
    serde::{Deserialize, Serialize},
    Request,
};
use serde_json::Value;
use std::time::Duration;
use twilight_http::Client;

#[derive(Debug, Serialize, Deserialize)]
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

    pub async fn get_guilds(&self) -> Option<Value> {
        let client = Client::new(format!("Bearer {}", self.access_token.secret()));
        match client.current_user_guilds().exec().await {
            Ok(response) => serde_json::from_str(&response.text().await.unwrap()).ok(),
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
        let cookie = request.cookies().get_private("user");
        match cookie {
            Some(cookie) => {
                let user = serde_json::from_str(&cookie.value()).unwrap();

                request::Outcome::Success(user)
            }
            None => {
                request::Outcome::Failure((Status::Unauthorized, MissingAuthorization::Missing))
            }
        }
    }
}
