use serde_json::Value;

use crate::models::User;

#[get("/user", rank = 1)]
async fn user(user: User) -> Option<Value> {
    user.get_data().await
}

#[get("/user", rank = 2)]
async fn no_user() -> Option<Value> {
    Some(Value::Null)
}

#[get("/guilds")]
async fn guilds(user: User) -> Option<Value> {
    user.get_guilds().await
}

#[get("/guilds/<_guild_id>")]
async fn get_guild(_user: User, _guild_id: u64) -> Option<Value> {
    Some(Value::Null)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![user, no_user, guilds, get_guild]
}
