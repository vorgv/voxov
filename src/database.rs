mod credit;
pub mod ripperd;

use crate::auth::nspm;
use crate::config::Config;
use crate::Result;
use bson::doc;
use mongodb::IndexModel;
use mongodb::{self, bson::Document, options::ClientOptions};
use redis::{aio::ConnectionManager, RedisError};
use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;
use std::result::Result as StdResult;
use std::str::FromStr;
use std::time::Duration;
use sysinfo::{DiskExt, System, SystemExt};

pub struct Database {
    /// Redis connection manager with auto retry
    cm: ConnectionManager,

    credit_limit: i64,
    access_ttl: i64,

    /// Map_v1 collection
    pub map1: mongodb::Collection<Document>,

    /// Meme metadata collection
    pub mm: mongodb::Collection<Document>,

    /// Credit log
    pub cl: mongodb::Collection<Document>,

    /// MongoDB
    mdb: mongodb::Database,

    /// Meme data bucket
    pub mr: Bucket,
}

async fn connect_redis(addr: &str) -> StdResult<ConnectionManager, RedisError> {
    let client = redis::Client::open(addr).unwrap();
    client.get_tokio_connection_manager().await
}

async fn connect_mongo(addr: &str) -> StdResult<mongodb::Database, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(addr).await?;
    client_options.app_name = Some("voxov".to_string());
    let client = mongodb::Client::with_options(client_options)?;
    Ok(client.database("voxov"))
}

use redis::{cmd, FromRedisValue, ToRedisArgs};

impl Database {
    /// Connect to databases, panic on failure.
    pub async fn new(config: &Config, create_index: bool) -> Database {
        let mdb = connect_mongo(&config.mongo_addr)
            .await
            .expect("MongoDB offline?");

        let db = Database {
            cm: connect_redis(&config.redis_addr)
                .await
                .expect("Redis offline?"),

            credit_limit: config.credit_limit,
            access_ttl: config.access_ttl,

            mm: mdb.collection::<Document>("mm"),

            map1: mdb.collection::<Document>("map"),

            cl: mdb.collection::<Document>("cl"),

            mdb,

            mr: Bucket::new(
                "voxov",
                Region::Custom {
                    region: config.s3_region.clone(),
                    endpoint: config.s3_addr.clone(),
                },
                Credentials::new(
                    Some(&config.s3_access_key),
                    Some(&config.s3_secret_key),
                    None,
                    None,
                    None,
                )
                .unwrap(),
            )
            .expect("S3 offline?")
            .with_path_style(),
        };

        if create_index {
            db.create_index()
                .await
                .expect("Database index creation failed.");
        }

        if config.samsara {
            db.samsara().await;
        }

        db
    }

    /// Set key-value pair with TTL by seconds.
    pub async fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: i64,
    ) -> Result<()> {
        Ok(cmd("SETEX")
            .arg(key)
            .arg(seconds)
            .arg(value)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await?)
    }

    /// Get value by key.
    pub async fn get<K: ToRedisArgs, V: FromRedisValue>(&self, key: K) -> Result<V> {
        Ok(cmd("GET")
            .arg(key)
            .query_async::<ConnectionManager, V>(&mut self.cm.clone())
            .await?)
    }

    /// Get value by key and set TTL.
    pub async fn getex<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        seconds: i64,
    ) -> Result<V> {
        Ok(cmd("GETEX")
            .arg(key)
            .arg("EX")
            .arg(seconds)
            .query_async::<ConnectionManager, V>(&mut self.cm.clone())
            .await?)
    }

    /// Set expiry.
    pub async fn expire<K: ToRedisArgs>(&self, key: K, seconds: i64) -> Result<()> {
        Ok(cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await?)
    }

    /// Set expiry if no expiry.
    pub async fn expire_xx<K: ToRedisArgs>(&self, key: K, seconds: i64) -> Result<()> {
        Ok(cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .arg("XX")
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await?)
    }

    /// Increment the number.
    pub async fn incrby<K: ToRedisArgs>(&self, key: K, number: i64) -> Result<()> {
        Ok(cmd("INCRBY")
            .arg(key)
            .arg(number)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await?)
    }

    /// Increment the number by 1.
    pub async fn incr<K: ToRedisArgs>(&self, key: K) -> Result<i64> {
        Ok(cmd("INCR")
            .arg(key)
            .query_async::<ConnectionManager, i64>(&mut self.cm.clone())
            .await?)
    }

    /// Decrement the number.
    pub async fn decrby<K: ToRedisArgs>(&self, key: K, number: i64) -> Result<()> {
        Ok(cmd("DECRBY")
            .arg(key)
            .arg(number)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await?)
    }

    /// Returns if key exists.
    pub async fn exits<K: ToRedisArgs>(&self, key: K) -> Result<i64> {
        Ok(cmd("EXISTS")
            .arg(key)
            .query_async::<ConnectionManager, i64>(&mut self.cm.clone())
            .await?)
    }

    /// Delete key.
    pub async fn del<K: ToRedisArgs>(&self, key: K) -> Result<()> {
        Ok(cmd("DEL")
            .arg(key)
            .query_async::<ConnectionManager, ()>(&mut self.cm.clone())
            .await?)
    }

    /// Sent SMS.
    pub async fn sms_sent(&self, from: &str, to: &str, message: &str) -> Result<()> {
        let message = Id::from_str(format!("{:0>32}", message).as_str())?;
        let s = nspm(SMSSENT, to, &message);
        self.set(&s[..], from, self.access_ttl).await
    }

    /// Index MongoDB.
    async fn create_index(&self) -> Result<()> {
        self.mm
            .create_index(IndexModel::builder().keys(doc! { "eol": 1 }).build(), None)
            .await?;

        self.map1
            .create_indexes(
                vec![
                    IndexModel::builder()
                        .keys(doc! {
                            "_uid": "hashed",
                            "_pub": -1,
                            "_eol": 1,
                            "_tip": 1,
                            "_ns": 1,
                            "_0": 1,
                            "_1": 1,
                            "_2": 1,
                            "_3": 1,
                            "_4": 1,
                            "_5": 1,
                            "_6": 1,
                            "_7": 1,
                        })
                        .build(),
                    IndexModel::builder()
                        .keys(doc! {
                            "_geo": "2dsphere",
                        })
                        .build(),
                ],
                None,
            )
            .await?;

        Ok(())
    }

    /// Reset databases on resource draining.
    async fn samsara(&self) {
        let mut cm = self.cm.clone();
        let mdb = self.mdb.clone();
        let mr = self.mr.clone();

        tokio::spawn(async move {
            let mut sys = System::new_all();
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;

                let mut draining = false;
                sys.refresh_all();

                let maybe_disk = sys
                    .disks()
                    .iter()
                    .map(|d| (d.total_space(), d.available_space()))
                    .max();

                if let Some(disk) = maybe_disk {
                    if (disk.1 as f32 / disk.0 as f32) < 0.1 {
                        draining = true;
                    }
                } else {
                    println!("Samsara error: disk not found");
                    break;
                }

                if (sys.used_memory() as f32 / sys.total_memory() as f32) > 0.9 {
                    draining = true;
                }

                if !draining {
                    continue;
                }

                println!("Samsara");

                if let Err(error) = cmd("FLUSHALL")
                    .query_async::<ConnectionManager, ()>(&mut cm)
                    .await
                {
                    println!("Samsara Redis error: {}", error);
                }

                if let Err(error) = mdb.drop(None).await {
                    println!("Samsara MongoDB error: {}", error);
                }

                if let Err(error) = mr.delete().await {
                    println!("Samsara S3 error: {}", error);
                }
            }
        });
    }
}

/// Namespace for keys.
pub mod namespace {
    /// Never use the _HIDDEN namespace.
    pub const _HIDDEN: u8 = 0;

    pub const ACCESS: u8 = 1;
    pub const REFRESH: u8 = 2;
    pub const SMSSENDTO: u8 = 3;
    pub const SMSSENT: u8 = 4;
    pub const PHONE2UID: u8 = 5;
    pub const UID2PHONE: u8 = 6;
    pub const UID2CREDIT: u8 = 7;
    pub const UID2CHECKIN: u8 = 8;
}

use crate::ir::id::{Id, IDL};
use bytes::{Buf, Bytes};

use self::namespace::SMSSENT;

/// Prepend namespace tag before Id.
pub fn ns(n: u8, id: &Id) -> Bytes {
    ([n][..]).chain(&id.0[..]).copy_to_bytes(1 + IDL)
}
