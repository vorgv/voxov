use std::time::Duration;
use tokio::time::sleep;

use crate::config::Config;
use crate::database::Database;

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
}
