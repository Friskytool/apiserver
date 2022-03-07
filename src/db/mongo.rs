pub use bson::{doc, Document};
use mongodb::{options::ClientOptions, Client};
pub use rocket::{figment::Figment, http::Status, serde::json::Json, State};
pub use rocket_db_pools::{Config, Connection, Database, Error as PoolsError, Pool};
use std::env::var;
use std::ops::Deref;
use std::time::Duration;

lazy_static! {
    pub static ref MONGO_DB: String = var("MONGO_DB").unwrap_or("Test".to_string());
}

pub struct ClientUnit(Client);

// Optional just to not drag ".0." syntax everywhere
impl Deref for ClientUnit {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Database)]
#[database("db_name")]
pub struct Db(ClientUnit);

// Boilerplate code which will not be needed when PR #1887 is merged
#[rocket::async_trait]
impl Pool for ClientUnit {
    type Error = PoolsError<mongodb::error::Error, std::convert::Infallible>;

    type Connection = Client;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config = figment.extract::<Config>()?;
        let mut opts = ClientOptions::parse(&config.url)
            .await
            .map_err(PoolsError::Init)?;
        opts.min_pool_size = config.min_connections;
        opts.max_pool_size = Some(config.max_connections as u32);
        opts.max_idle_time = config.idle_timeout.map(Duration::from_secs);
        opts.connect_timeout = Some(Duration::from_secs(config.connect_timeout));
        Ok(ClientUnit(
            Client::with_options(opts).map_err(PoolsError::Init)?,
        ))
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.0.clone())
    }
}
