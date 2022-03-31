use crate::models::User;
use rocket::routes;
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
async fn guilds(user: User) -> Option<Value> {
    user.get_guilds().await
}

pub fn routes() -> Vec<rocket::Route> {
    routes![user, no_user, guilds]
}
