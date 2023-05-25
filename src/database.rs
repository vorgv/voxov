use crate::config::Config;
use crate::message::Error;
use mongodb::{self, bson::Document, options::ClientOptions};
use redis::{aio::ConnectionManager, RedisError};
use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;

pub struct Database {
    /// Redis connection manager with auto retry
    cm: ConnectionManager,

    /// Meme metadata collection
    mm: mongodb::Collection<Document>, //TODO: custom struct

    /// Meme raw data collection
    mr: Bucket, //TODO: s3
}

async fn connect_redis(addr: &str) -> Result<ConnectionManager, RedisError> {
    let client = redis::Client::open(addr).unwrap();
    client.get_tokio_connection_manager().await
}

async fn connect_mongo(addr: &str) -> Result<mongodb::Database, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(addr).await?;
    client_options.app_name = Some("voxov".to_string());
    let client = mongodb::Client::with_options(client_options)?;
    Ok(client.database("voxov"))
}

use redis::{cmd, FromRedisValue, ToRedisArgs};

impl Database {
    /// Connect to Redis, panic on failure
    pub async fn new(config: &Config) -> Database {
        let mdb = connect_mongo(&config.mongo_addr)
            .await
            .expect("MongoDB offline?");
        Database {
            cm: connect_redis(&config.redis_addr)
                .await
                .expect("Redis offline?"),
            mm: mdb.collection::<Document>("mm"),
            mr: Bucket::new(
                "voxov",
                Region::Custom {
                    region: config.s3_region.clone(),
                    endpoint: config.s3_addr.clone(),
                },
                Credentials::default().unwrap(),
            )
            .expect("S3 offline?"),
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
