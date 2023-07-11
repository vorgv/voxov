use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::IndexModel;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tokio_stream::StreamExt;

use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use crate::message::{Hash, Id, Uint};

pub struct Meme {
    db: &'static Database,
    ripperd_disabled: bool,
    ripperd_interval: u64,
    space_cost_doc: Uint,
}

impl Meme {
    pub fn new(config: &Config, db: &'static Database) -> Meme {
        Meme {
            db,
            ripperd_disabled: config.ripperd_disabled,
            ripperd_interval: config.ripperd_interval,
            space_cost_doc: config.space_cost_doc,
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
        hash: &Hash,
        deadline: Instant,
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
}
