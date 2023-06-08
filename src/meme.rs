use mongodb::bson::spec::BinarySubtype;
use mongodb::bson::{doc, Binary};
use std::time::Duration;
use tokio::time::{sleep, Instant};

use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use crate::message::{Hash, Id};

pub struct Meme {
    database: &'static Database,
    ripperd_disabled: bool,
    ripperd_interval: u64,
}

impl Meme {
    pub fn new(config: &Config, database: &'static Database) -> Meme {
        Meme {
            database,
            ripperd_disabled: config.ripperd_disabled,
            ripperd_interval: config.ripperd_interval,
        }
    }

    /// Ripper Daemon periodically deletes memes by the EOL field.
    /// Enable this on one and only one instance in the cluster.
    pub async fn ripperd(&self) {
        if self.ripperd_disabled {
            return;
        }
        loop {
            sleep(Duration::from_secs(self.ripperd_interval)).await;
            //TODO
            // Get all memes with EOL < now
            // Remove them on S3 first to prevent leakage.
            // Remove them on MongoDB
        }
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
        let mm = &self.database.mm;
        let filter = doc! { "hash": Binary {subtype: BinarySubtype::Generic, bytes: hash.to_vec()}};
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
            let m_uid = meta.get_binary_generic("uid").map_err(|_| Error::Logical)?;
            if m_uid.as_slice() == uid.0 {
                return Ok(meta.to_string());
            }
        }
        Err(Error::MemeNotFound)
    }
}
