pub mod mongo;
pub mod redis;

pub use mongo::Db as MongoDB;
pub use redis::Db as Redis;
