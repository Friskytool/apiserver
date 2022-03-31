use rocket_db_pools::{deadpool_redis::Pool, Database};

#[derive(Database)]
#[database("redis")]
pub struct Db(Pool);
