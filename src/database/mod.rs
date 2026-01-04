//! Database module
//!
//! Handles database connections and provides access to MongoDB and Redis.

pub mod mongodb;
pub mod redis;

pub use mongodb::MongoDB;
pub use redis::RedisDB;
