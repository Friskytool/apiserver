use std::collections::HashMap;

use crate::db::{
    mongo::{Json, Status, MONGO_DB},
    Db,
};
// use crate::models::User;
use bson::Document;
use mongodb::options::FindOneAndUpdateOptions;
use rocket_db_pools::Connection;
use serde_json::Value;

#[get("/guilds/<guild_id>/settings")]
pub async fn settings(conn: Connection<Db>, guild_id: u64) -> Result<Json<Document>, Status> {
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
    // _user: User,
    conn: Connection<Db>,
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

#[get("/guilds/<guild_id>")]
pub async fn guild(
    // _user: User,
    conn: Connection<Db>,
    guild_id: u64,
) -> Result<Json<Document>, Status> {
    match conn
        .database(&MONGO_DB)
        .collection::<Document>("servers")
        .find_one(doc! {"_id":guild_id.to_string()}, None)
        .await
    {
        Ok(Some(cursor)) => Ok(Json(cursor)),
        Ok(None) => Err(Status::NotFound),
        Err(e) => {
            dbg!(e);
            Err(Status::InternalServerError)
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![guild, settings, edit_settings]
}
