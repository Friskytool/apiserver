use crate::{
    db::Redis,
    models::{user::Guild, User},
};
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

    let mut data: Vec<Guild>;

    if let Some(d) = guilds {
        data = serde_json::from_str(&d).unwrap();
    } else {
        data = user.get_guilds().await.unwrap_or_default();
        let _: Option<()> = conn
            .set_ex(
                format!("user:{}:guilds", user.access_token.secret()),
                serde_json::to_string(&data).unwrap(),
                360,
            )
            .await
            .ok();
    }
    for mut guild in data.iter_mut() {
        guild.bot_in_guild = Some(
            conn.sismember("guild", guild.id.clone())
                .await
                .unwrap_or_else(|e| unreachable!("{}", e)),
        )
    }
    dbg!(&data);

    serde_json::to_value(data).ok()
}

pub fn routes() -> Vec<rocket::Route> {
    routes![user, no_user, guilds]
}
