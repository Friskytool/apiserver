use crate::db::{
    mongo::{Json, Status, MONGO_DB},
    MongoDB,
};
use crate::models::User;
use bson::Document;
use rocket_db_pools::Connection;

#[get("/guilds/<guild_id>")]
pub async fn get_guild(
    _user: User,
    conn: Connection<MongoDB>,
    guild_id: u64,
) -> Result<Json<Document>, Status> {
    match conn
        .database(&MONGO_DB)
        .collection::<Document>("servers")
        .find_one(doc! {"_id":guild_id.to_string()}, None)
        .await
    {
        Ok(Some(tag)) => Ok(Json(tag)),
        Ok(None) => Err(Status::NotFound),
        Err(e) => {
            dbg!(e);
            Err(Status::InternalServerError)
        }
    }
}
