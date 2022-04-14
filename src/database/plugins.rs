use std::collections::HashMap;

use crate::{
    db::{
        mongo::{Json, Status, MONGO_DB},
        MongoDB, Redis,
    },
    models::User,
};
// use crate::models::User;
use bson::Document;
use deadpool_redis::redis::{pipe, AsyncCommands};
use mongodb::options::FindOneAndUpdateOptions;

use rocket_db_pools::Connection;
use serde_json::Value;

#[get("/guilds/<guild_id>/settings")]
pub async fn get_settings(
    conn: Connection<MongoDB>,
    guild_id: u64,
) -> Result<Json<Document>, Status> {
    match conn
        .database(&MONGO_DB)
        .collection::<Document>("settings")
        .find_one(doc! {"guild_id":guild_id.to_string()}, None)
        .await
    {
        Ok(Some(document)) => Ok(Json(document)),
        Ok(None) => Ok(Json(doc! {})),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/guilds/<guild_id>/settings", format = "json", data = "<data>")]
pub async fn edit_settings(
    _user: User,
    conn: Connection<MongoDB>,
    guild_id: u64,
    data: Json<Value>,
) -> Status {
    let doc: HashMap<String, Document> = match serde_json::from_value(data.0) {
        Ok(doc) => doc,
        Err(_) => return Status::BadRequest,
    };

    for (key, value) in doc {
        match conn
            .database(&MONGO_DB)
            .collection::<Document>("settings")
            .find_one_and_update(
                doc! {"guild_id":guild_id.to_string()},
                doc! {"$set":{key:value}},
                FindOneAndUpdateOptions::builder().upsert(true).build(),
            )
            .await
        {
            Ok(_) => (),
            Err(_) => return Status::InternalServerError,
        }
    }
    Status::Accepted
}

#[get("/guilds/<guild_id>/plugins")]
pub async fn get_plugins(
    mut conn: Connection<Redis>,
    _user: User,
    guild_id: u64,
) -> Result<Json<Vec<Value>>, Status> {
    conn.smembers(format!("plugins:{}", guild_id))
        .await
        .map_err(|_| Status::ServiceUnavailable)
        .map(|o: Vec<String>| o.into_iter().map(Into::into).collect::<Vec<Value>>())
        .map(Json)
}

#[post("/guilds/<guild_id>/plugins", format = "json", data = "<data>")]
pub async fn edit_plugins(
    mut conn: Connection<Redis>,
    _user: User,
    guild_id: u64,
    data: Json<Vec<String>>,
) -> Status {
    let key = format!("plugins:{}", guild_id);

    let data: Vec<String> = data.iter().map(Into::into).collect();

    let cdata: Vec<String> = conn.smembers(&key).await.unwrap_or_default();
    let add = data
        .iter()
        .filter(|x| !cdata.contains(x))
        .collect::<Vec<&String>>();
    let rem = cdata
        .iter()
        .filter(|x| !data.contains(x))
        .collect::<Vec<&String>>();

    if add.len() > 0 || rem.len() > 0 {
        let mut pipe = pipe();

        for item in add {
            pipe.sadd(&key, item);
        }
        for item in rem {
            pipe.srem(&key, item);
        }

        match pipe.query_async::<_, ()>(&mut *conn).await {
            Ok(_) => Status::Accepted,
            Err(_) => Status::InternalServerError,
        }
    } else {
        Status::Accepted
    }
}
