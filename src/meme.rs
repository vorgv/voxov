use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::IndexModel;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tokio_stream::StreamExt;

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::message::query::QueryBody;
use crate::message::{Costs, Hash, Id, Reply, Uint};

pub struct Meme {
    db: &'static Database,
    ripperd_disabled: bool,
    ripperd_interval: u64,
    space_cost_doc: Uint,
    traffic_cost: Uint,
}

impl Meme {
    pub fn new(config: &Config, db: &'static Database) -> Meme {
        Meme {
            db,
            ripperd_disabled: config.ripperd_disabled,
            ripperd_interval: config.ripperd_interval,
            space_cost_doc: config.space_cost_doc,
            traffic_cost: config.traffic_cost,
        }
    }

    /// Ripper Daemon periodically deletes memes by the EOL field.
    /// Enable this on one and only one instance in the cluster.
    pub async fn ripperd(&self) {
        if self.ripperd_disabled {
            return;
        }
        let mm = &self.db.mm;
        if let Err(error) = mm
            .create_index(IndexModel::builder().keys(doc! { "eol": 1 }).build(), None)
            .await
        {
            println!("Ripperd error: {}", error);
        }
        loop {
            sleep(Duration::from_secs(self.ripperd_interval)).await;
            if let Err(error) = self.rip().await {
                println!("Ripperd error: {}", error);
            }
        }
    }

    /// Fallible wrapper for a rip operation.
    async fn rip(&self) -> Result<(), Error> {
        // Get all memes with EOL < now
        let options = FindOptions::builder()
            .projection(doc! { "_id": 1, "eol": 1, "oid": 1 })
            .sort(doc! { "eol": 1 })
            .build();
        let mm = &self.db.mm;
        let mut cursor = mm
            .find(
                doc! {
                    "eol": { "$lt": Utc::now() }
                },
                options,
            )
            .await
            .map_err(|e| {
                println!("{}", e);
                Error::MongoDB
            })?;
        let mr = &self.db.mr;
        while let Some(meta) = cursor.try_next().await.map_err(|_| Error::MongoDB)? {
            // Remove them on S3 first to prevent leakage.
            let oid = meta.get_str("oid").map_err(Error::BsonValueAccess)?;
            mr.delete_object(oid).await.map_err(Error::S3)?;
            // Remove them on MongoDB
            let id = meta.get_object_id("_id").map_err(|_| Error::MongoDB)?;
            mm.find_one_and_delete(doc! { "_id": id }, None)
                .await
                .map_err(|_| Error::MongoDB)?;
        }
        Ok(())
    }

    /// Return meme metadata if meme is public or belongs to uid.
    /// The driver of MongoDB breaks if internal futures are dropped.
    /// This limitation hinders tokio::select! style timeout.
    pub async fn get_meta(
        &self,
        uid: &Id,
        deadline: Instant,
        hash: &Hash,
    ) -> Result<String, Error> {
        let mm = &self.db.mm;
        let filter = doc! { "hash": hex::encode(hash) };
        let handle = tokio::task::spawn(async move { mm.find_one(filter, None).await });
        let option_meta = tokio::time::timeout_at(deadline, handle)
            .await
            .map_err(|_| Error::CostTime)?
            .map_err(|_| Error::CostTime)?
            .map_err(|_| Error::MongoDB)?;
        if let Some(meta) = option_meta {
            if meta.get_bool("public").map_err(|_| Error::Logical)? {
                return Ok(meta.to_string());
            }
            let m_uid = meta.get_str("uid").map_err(|_| Error::Logical)?;
            if m_uid == uid.to_string() {
                return Ok(meta.to_string());
            }
        }
        Err(Error::MemeNotFound)
    }

    /// Stream version didn't work.
    /// Try using chunk.
    pub async fn put_meme(
        &self,
        uid: &Id,
        mut changes: Costs,
        deadline: Instant,
        days: u64,
        raw: QueryBody,
    ) -> Result<Reply, Error> {
        todo!()
    }

    /// Current implementation uses high-level stream.
    /// Further investigation on performance is required.
    pub async fn get_meme(
        &self,
        uid: &Id,
        mut changes: Costs,
        _deadline: Instant,
        hash: Hash,
        public: bool,
    ) -> Result<Reply, Error> {
        let hash = hex::encode(hash);
        // Filter
        let filter = match public {
            true => doc! {
                "public": true,
                "hash": hash.clone(),
            },
            false => doc! {
                "uid": uid.to_string(),
                "hash": hash.clone(),
            },
        };
        // Sort by tips
        let options = FindOneOptions::builder()
            .projection(doc! { "oid": 1, "uid": 1, "hash": 1, "size": 1, "tips": 1, "_id": 0 })
            .sort(doc! { "tips": 1 })
            .build();
        let mm = &self.db.mm;
        let meta = mm
            .find_one(filter, options)
            .await
            .map_err(|_| Error::MemeGet)?;
        if meta.is_none() {
            return Err(Error::MemeNotFound);
        }
        let meta = meta.unwrap();
        // Is fund enough for the file size
        let cost = self.traffic_cost * meta.get_i64("size").map_err(|_| Error::Logical)? as u64;
        if cost > changes.traffic {
            return Err(Error::CostTraffic);
        }
        changes.traffic -= cost;
        // Pay tips
        if public {
            let tips = meta.get_i64("tips").map_err(|_| Error::Logical)? as u64;
            if tips > changes.tips {
                return Err(Error::CostTips);
            }
            changes.tips -= tips;
            let uid = meta.get_str("uid").map_err(|_| Error::Logical)?;
            use std::str::FromStr;
            let uid = Id::from_str(uid)?;
            let u2c = ns(UID2CREDIT, &uid);
            self.db.incrby(&u2c[..], tips).await?;
        }
        // Stream object
        let oid = meta.get_str("oid").map_err(|_| Error::Logical)?;
        let mr = &self.db.mr;
        let stream = Box::pin(mr.get_object_stream(oid).await.map_err(Error::S3)?);
        // Check costs
        Ok(Reply::MemeGet {
            changes,
            raw: stream,
        })
    }
}
