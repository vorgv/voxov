extern crate redis;
use redis::{aio::Connection, RedisError};

use crate::config::Config;
use crate::message::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    rc: Arc<Mutex<Connection>>,
}

async fn connect_redis(addr: &str) -> Result<Connection, RedisError> {
    let client = redis::Client::open(addr).unwrap();
    client.get_async_connection().await
}

use redis::{cmd, FromRedisValue, ToRedisArgs};
impl Database {
    pub async fn new(config: &Config) -> Database {
        Database {
            rc: match connect_redis(&config.redis_addr).await {
                Ok(con) => Arc::new(Mutex::new(con)),
                Err(_) => panic!("connection failed, todo trace"),
            },
        }
    }
    /// Set key-value pair with TTL by seconds
    pub async fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: usize,
    ) -> Result<(), Error> {
        let clone = Arc::clone(&self.rc);
        let mut lock = clone.lock().await;
        match cmd("SETEX")
            .arg(key)
            .arg(seconds)
            .arg(value)
            .query_async::<Connection, ()>(&mut lock)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Redis),
        }
    }
    /// Get value by key
    pub async fn get<K: ToRedisArgs, V: FromRedisValue>(&self, key: K) -> Result<V, Error> {
        let clone = Arc::clone(&self.rc);
        let mut lock = clone.lock().await;
        match cmd("GET")
            .arg(key)
            .query_async::<Connection, V>(&mut lock)
            .await
        {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::Redis),
        }
    }
    /// Get value by key and set TTL
    pub async fn getex<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        seconds: usize,
    ) -> Result<V, Error> {
        let clone = Arc::clone(&self.rc);
        let mut lock = clone.lock().await;
        match cmd("GETEX")
            .arg(key)
            .arg("EX")
            .arg(seconds)
            .query_async::<Connection, V>(&mut lock)
            .await
        {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::Redis),
        }
    }
    /// Delete key
    pub async fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), Error> {
        let clone = Arc::clone(&self.rc);
        let mut lock = clone.lock().await;
        match cmd("DEL")
            .arg(key)
            .query_async::<Connection, ()>(&mut lock)
            .await
        {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::Redis),
        }
    }
}

/// Namespace for keys
pub mod namespace {
    pub const _HIDDEN: u8 = 0;
    pub const ACCESS: u8 = 1;
    pub const REFRESH: u8 = 2;
}
