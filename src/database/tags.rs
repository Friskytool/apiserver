use crate::{
    db::{
        mongo::{Json, Status, MONGO_DB},
        MongoDB,
    },
    models::{Tag, User},
};
use rocket::futures::TryStreamExt;

use rocket_db_pools::Connection;

#[get("/guilds/<guild_id>/tags")]
pub async fn get_tags(
    _user: User,
    conn: Connection<MongoDB>,
    guild_id: u64,
) -> Result<Json<Vec<Tag>>, Status> {
    match conn
        .database(&MONGO_DB)
        .collection::<Tag>("tags")
        .find(doc! {"guild_id":guild_id.to_string()}, None)
        .await
    {
        Ok(cursor) => Ok(Json(cursor.try_collect().await.unwrap_or(vec![]))),
        Err(e) => {
            dbg!(e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/guilds/<_guild_id>/tags", format = "json", data = "<tag>")]
pub async fn create_tag(
    _user: User,
    conn: Connection<MongoDB>,
    _guild_id: u64,
    tag: Json<Tag>,
) -> Status {
    match conn
        .database(&MONGO_DB)
        .collection::<Tag>("tags")
        .insert_one(tag.into_inner(), None)
        .await
    {
        Ok(_) => Status::Accepted,
        Err(_) => Status::InternalServerError,
    }
}

#[patch("/guilds/<_guild_id>/tags/<tag_id>", format = "json", data = "<tag>")]
pub async fn update_tag(
    _user: User,
    conn: Connection<MongoDB>,
    _guild_id: u64,
    tag_id: u64,
    tag: Json<Tag>,
) -> Status {
    match conn
        .database(&MONGO_DB)
        .collection::<Tag>("tags")
        .update_one(
            doc! {"_id":tag_id.to_string()},
            doc! {"$set":bson::to_bson(&tag.into_inner()).unwrap()},
            None,
        )
        .await
    {
        Ok(_) => Status::Accepted,
        Err(_) => Status::InternalServerError,
    }
}
