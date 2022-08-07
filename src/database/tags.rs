use crate::{
    db::{
        mongo::{Json, Status, MONGO_DB},
        MongoDB, Redis,
    },
    models::{Tag, TagUpdate, User},
};
use deadpool_redis::redis::AsyncCommands;
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

#[post("/guilds/<guild_id>/tags", format = "json", data = "<tag>")]
pub async fn create_tag(
    _user: User,
    conn: Connection<MongoDB>,
    guild_id: u64,
    tag: Json<Tag>,
) -> Status {
    let mut tag = tag.into_inner();

    if tag.guild_id != guild_id.to_string() {
        return Status::BadRequest;
    }
    if !tag.validate() {
        return Status::BadRequest;
    }

    if let Err(err) = tag.create().await {
        dbg!(&err);
        return Status::InternalServerError;
    }
    match conn
        .database(&MONGO_DB)
        .collection::<Tag>("tags")
        .insert_one(&tag, None)
        .await
    {
        Ok(resp) => {
            if resp.inserted_id == bson::Bson::Null {
                Status::InternalServerError
            } else {
                Status::Created
            }
        }
        Err(e) => {
            dbg!(e);
            Status::InternalServerError
        }
    }
}

#[patch(
    "/guilds/<guild_id>/tags/<tag_name>",
    format = "json",
    data = "<payload>"
)]
pub async fn update_tag(
    _user: User,
    conn: Connection<MongoDB>,
    guild_id: u64,
    tag_name: String,
    payload: Json<TagUpdate>,
) -> Status {
    let TagUpdate { new, old } = payload.into_inner();

    if tag_name != old.name {
        return Status::BadRequest;
    }

    if let Err(why) = old.update(&new).await {
        eprintln!("{:#?}", &why);
        return Status::InternalServerError;
    }

    match conn
        .database(&MONGO_DB)
        .collection::<Tag>("tags")
        .update_one(
            doc! {"name":tag_name.to_string(), "guild_id": guild_id.to_string()},
            doc! {"$set":bson::to_bson(&new).unwrap()},
            None,
        )
        .await
    {
        Ok(r) => {
            dbg!(r);
            Status::Accepted
        }
        Err(_) => Status::InternalServerError,
    }
}

#[delete("/guilds/<guild_id>/tags/<tag_name>")]
pub async fn delete_tag(
    _user: User,
    conn: Connection<MongoDB>,
    mut redis: Connection<Redis>,
    guild_id: u64,
    tag_name: String,
) -> Status {
    match conn
        .database(&MONGO_DB)
        .collection::<Tag>("tags")
        .find_one_and_delete(
            doc! {"name":tag_name.to_string(), "guild_id": guild_id.to_string()},
            None,
        )
        .await
    {
        Ok(Some(tag)) => match tag.delete().await {
            Ok(_) => {
                // Cleaning up tag data
                redis
                    .del::<_, ()>(format!("tags:{}:{}", guild_id, tag_name))
                    .await
                    .ok();
                Status::Ok
            }
            Err(_) => Status::InternalServerError,
        },
        _ => Status::InternalServerError,
    }
}
