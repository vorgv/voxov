use super::Database;
use crate::{Result, config::Config};
use sqlx::Row;
use std::time::Duration;
use tokio::time::sleep;

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

    /// Ripper Daemon periodically deletes expired data.
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
            // Note: Credit log cleanup is no longer needed.
            // TigerBeetle's transfer log is the audit trail.
        }
    }

    /// Rip expired meme metadata and blob data.
    async fn rip_meme(&self) -> Result<()> {
        // Get all memes with EOL < now
        let rows = sqlx::query("SELECT id, oid FROM meme_meta WHERE eol < now()")
            .fetch_all(&self.db.crdb)
            .await?;

        let mr = &self.db.mr;
        for row in rows {
            let id: uuid::Uuid = row.get("id");
            let oid: Vec<u8> = row.get("oid");

            // Remove from S3 first to prevent data leakage
            let oid_hex = hex::encode(&oid);
            if let Err(e) = mr.delete_object(&oid_hex).await {
                println!("Rip meme S3 error for {}: {}", oid_hex, e);
                continue;
            }

            // Remove from CockroachDB
            if let Err(e) = sqlx::query("DELETE FROM meme_meta WHERE id = $1")
                .bind(id)
                .execute(&self.db.crdb)
                .await
            {
                println!("Rip meme DB error for {}: {}", id, e);
            }
        }

        Ok(())
    }

    /// Rip expired map documents.
    async fn rip_map1(&self) -> Result<()> {
        // Delete all map documents with EOL < now in a single query
        let result = sqlx::query("DELETE FROM map_docs WHERE eol < now()")
            .execute(&self.db.crdb)
            .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            println!("Ripped {} expired map documents", deleted);
        }

        Ok(())
    }
}
