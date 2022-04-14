use crate::{db::Redis, models::User};
use deadpool_redis::redis::AsyncCommands;
use rocket::routes;
use rocket_db_pools::Connection;
use serde_json::Value;
#[get("/", rank = 1)]
async fn user(user: User) -> Option<Value> {
    user.get_data().await
}

#[get("/", rank = 2)]
async fn no_user() -> Option<Value> {
    Some(Value::Null)
}

#[get("/guilds")]
async fn guilds(mut conn: Connection<Redis>, user: User) -> Option<Value> {
    let guilds: Option<String> = conn
        .get(format!("user:{}:guilds", user.access_token.secret()))
        .await
        .ok();

    if let Some(data) = guilds {
        Some(serde_json::from_str(&data).unwrap())
    } else {
        let data = user.get_guilds().await;
        let _: Option<()> = conn
            .set_ex(
                format!("user:{}:guilds", user.access_token.secret()),
                serde_json::to_string(&data).unwrap(),
                3600,
            )
            .await
            .ok();
        data
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![user, no_user, guilds]
}
