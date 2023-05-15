use crate::config::Config;
use crate::message::Error;
use redis::{aio::ConnectionManager, RedisError};

pub struct Database {
    cm: ConnectionManager,
}

async fn connect_redis(addr: &str) -> Result<ConnectionManager, RedisError> {
    let client = redis::Client::open(addr).unwrap();
    client.get_tokio_connection_manager().await
}

use redis::{cmd, FromRedisValue, ToRedisArgs};

impl Database {
    /// Connect to Redis, panic on failure
    pub async fn new(config: &Config) -> Database {
        Database {
            cm: connect_redis(&config.redis_addr)
                .await
                .expect("Redis offline?"),
        }
    }
    /// Set key-value pair with TTL by seconds
    pub async fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: usize,
    ) -> Result<(), Error> {
        match cmd("SETEX")
            .arg(key)
            .arg(seconds)
            .arg(value)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Redis),
        }
    }
    /// Get value by key
    pub async fn get<K: ToRedisArgs, V: FromRedisValue>(&self, key: K) -> Result<V, Error> {
        match cmd("GET")
            .arg(key)
            .query_async::<ConnectionManager, V>(&mut self.cm.clone())
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
        match cmd("GETEX")
            .arg(key)
            .arg("EX")
            .arg(seconds)
            .query_async::<ConnectionManager, V>(&mut self.cm.clone())
            .await
        {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::Redis),
        }
    }
    /// Delete key
    pub async fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), Error> {
        match cmd("DEL")
            .arg(key)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
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
    pub const SMSSENDTO: u8 = 3;
    pub const SMSSENT: u8 = 4;
    pub const PHONE2UID: u8 = 5;
    pub const UID2PHONE: u8 = 6;
}

use crate::message::id::{Id, IDL};
use bytes::{Buf, Bytes};

/// Prepend namespace tag before Id
pub fn ns(n: u8, id: &Id) -> Bytes {
    ([n][..]).chain(&id.0[..]).copy_to_bytes(1 + IDL)
}
