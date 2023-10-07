use super::Database;
use crate::{config::Config, Result};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::StreamExt;

pub struct Ripperd {
    db: &'static Database,
    ripperd_disabled: bool,
    ripperd_interval: u64,
}

impl Ripperd {
    pub fn new(config: &Config, db: &'static Database) -> Ripperd {
        Ripperd {
            db,
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
            if let Err(error) = self.rip_meme().await {
                println!("Rip meme error: {}", error);
            }
            if let Err(error) = self.rip_map1().await {
                println!("Rip map1 error: {}", error);
            }
            if let Err(error) = self.rip_cl().await {
                println!("Rip cl error: {}", error);
            }
        }
    }

    /// Rip meme_meta and meme_raw.
    async fn rip_meme(&self) -> Result<()> {
        // Get all memes with EOL < now.
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
            .await?;
        let mr = &self.db.mr;
        while let Some(meta) = cursor.try_next().await? {
            // Remove them on S3 first to prevent leakage.
            let oid = meta.get_str("oid")?;
            mr.delete_object(oid).await?;
            // Remove them on MongoDB
            let id = meta.get_object_id("_id")?;
            mm.find_one_and_delete(doc! { "_id": id }, None).await?;
        }
        Ok(())
    }

    /// Rip map database.
    async fn rip_map1(&self) -> Result<()> {
        // Get all maps with EOL < now
        todo!()
    }

    /// Rip credit log.
    async fn rip_cl(&self) -> Result<()> {
        // Get all cl with time + retention < now
        todo!()
    }
}
