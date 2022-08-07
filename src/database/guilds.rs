use crate::db::Redis;
use crate::models::User;
use deadpool_redis::redis::AsyncCommands;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde_json::Value;

#[get("/guilds/<guild_id>")]
pub async fn get_guild(
    _user: User,
    mut conn: Connection<Redis>,
    guild_id: u64,
) -> Result<Json<Value>, Status> {
    let data = conn.get::<_, String>(format!("guild.{}", guild_id)).await;

    match data {
        Ok(data) => {
            let guild = match serde_json::from_str(&data) {
                Ok(guild) => guild,
                Err(e) => {
                    dbg!(e);
                    return Err(Status::InternalServerError);
                }
            };
            Ok(Json(guild))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/guilds/<guild_id>/roles")]
pub async fn get_roles(
    _user: User,
    mut conn: Connection<Redis>,
    guild_id: u64,
) -> Result<Json<Vec<Value>>, Status> {
    let mut roles = vec![];

    for role_id in conn
        .smembers::<_, Vec<String>>(format!("role.{}", guild_id))
        .await
        .unwrap_or_else(|e| unreachable!("{}", e))
    {
        let role = conn
            .get::<_, String>(format!("role.{}.{}", guild_id, role_id))
            .await
            .unwrap_or_else(|e| unreachable!("{}", e));

        roles.push(serde_json::from_str(&role).unwrap_or_else(|e| unreachable!("{}", e)));
    }

    Ok(Json(roles))
}

#[get("/guilds/<guild_id>/channels")]
pub async fn get_channels(
    _user: User,
    mut conn: Connection<Redis>,
    guild_id: u64,
) -> Result<Json<Vec<Value>>, Status> {
    let mut channels = vec![];

    for channel_id in conn
        .smembers::<_, Vec<String>>(format!("channelmap.guild.{}", guild_id))
        .await
        .unwrap_or_else(|e| unreachable!("{}", e))
    {
        let channel = conn
            .get::<_, String>(format!("channel.{}", channel_id))
            .await
            .unwrap_or_else(|e| unreachable!("{}", e));
        channels.push(serde_json::from_str(&channel).unwrap_or_else(|e| unreachable!("{}", e)));
    }

    Ok(Json(channels))
}
