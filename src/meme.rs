use crate::config::Config;
use crate::database::Database;
use crate::ir::query::QueryBody;
use crate::ir::{Costs, Hash, Id, Reply};
use crate::{Error, Result};
use chrono::{DateTime, Days, Utc};
use http_body_util::BodyExt;
use s3::bucket::CHUNK_SIZE;
use serde_json::json;
use sqlx::Row;
use std::time::Duration;
use tokio::time::Instant;

pub struct Meme {
    db: &'static Database,
    time_cost: i64,
    space_cost_obj: i64,
    space_cost_doc: i64,
    traffic_cost: i64,
}

impl Meme {
    pub fn new(config: &Config, db: &'static Database) -> Meme {
        Meme {
            db,
            time_cost: config.time_cost,
            space_cost_obj: config.space_cost_obj,
            space_cost_doc: config.space_cost_doc,
            traffic_cost: config.traffic_cost,
        }
    }

    /// Return meme metadata if meme is public or belongs to uid.
    pub async fn get_meta(&self, uid: &Id, _deadline: Instant, hash: &Hash) -> Result<String> {
        let hash_bytes = hash.as_slice();
        let uid_bytes = &uid.0[..];

        let row = sqlx::query(
            "SELECT id, uid, oid, hash, size, pub, tip, eol FROM meme_meta 
             WHERE hash = $1 AND (pub = true OR uid = $2)
             LIMIT 1",
        )
        .bind(hash_bytes)
        .bind(uid_bytes)
        .fetch_optional(&self.db.crdb)
        .await
        .map_err(|_| Error::MemeGet)?;

        if let Some(row) = row {
            return Self::format_meta_row(&row);
        }

        Err(Error::MemeNotFound)
    }

    fn format_meta_row(row: &sqlx::postgres::PgRow) -> Result<String> {
        let id: uuid::Uuid = row.get("id");
        let uid: Vec<u8> = row.get("uid");
        let oid: Vec<u8> = row.get("oid");
        let hash: Vec<u8> = row.get("hash");
        let size: i64 = row.get("size");
        let is_pub: bool = row.get("pub");
        let tip: i64 = row.get("tip");
        let eol: DateTime<Utc> = row.get("eol");

        let json = json!({
            "_id": id.to_string(),
            "uid": hex::encode(&uid),
            "oid": hex::encode(&oid),
            "hash": hex::encode(&hash),
            "size": size,
            "pub": is_pub,
            "tip": tip,
            "eol": eol.to_rfc3339(),
        });
        Ok(json.to_string())
    }

    /// Stream version didn't work.
    /// Try using chunk.
    pub async fn put_meme(
        &self,
        uid: &Id,
        changes: &mut Costs,
        deadline: Instant,
        days: u64,
        mut raw: QueryBody,
    ) -> Result<Reply> {
        // Create object with a random name.
        let oid = {
            let mut rng = rand::rng();
            Id::rand(&mut rng)
        };
        // Init chunk upload.
        let mr = &self.db.mr;
        let content_type = String::new();
        let msg = mr
            .initiate_multipart_upload(&oid.to_string(), &content_type)
            .await?;
        let (path, upload_id) = (msg.key, msg.upload_id);
        // Upload chunks.
        let mut hasher = blake3::Hasher::new();
        let mut size = 0;
        let mut part_number = 0;
        let mut parts = vec![];
        let mut stack = vec![];
        let mut chunk_size = 0;
        while let Some(result) = raw.frame().await {
            let frame = result?;
            if let Ok(data) = frame.into_data() {
                // Space check
                let cost = match (data.len() as i64 * self.space_cost_obj).checked_mul(days as i64)
                {
                    Some(i) => i / 1000, // per day per KB
                    None => return Err(Error::CostSpaceTooLarge),
                };
                if changes.space < cost {
                    changes.space = 0;
                    return Err(Error::CostSpace);
                } else {
                    changes.space -= cost;
                }
                // Time check
                if Instant::now() > deadline {
                    return Err(Error::CostTime);
                }
                // Update metadata
                hasher.update(&data);
                size += data.len();
                // Append to stack;
                chunk_size += data.len();
                stack.push(data);
                if chunk_size >= CHUNK_SIZE {
                    part_number += 1;
                    let part = mr
                        .put_multipart_chunk(
                            stack.concat(),
                            &path,
                            part_number,
                            &upload_id,
                            &content_type,
                        )
                        .await?;
                    parts.push(part);
                    // Reset stack
                    chunk_size = 0;
                    stack.clear();
                }
            }
        }
        // Upload the last chunk.
        if chunk_size != 0 {
            part_number += 1;
            let part = mr
                .put_multipart_chunk(
                    stack.concat(),
                    &path,
                    part_number,
                    &upload_id,
                    &content_type,
                )
                .await?;
            parts.push(part);
        }
        // Complete chunk upload.
        mr.complete_multipart_upload(&path, &upload_id, parts)
            .await?;

        // Create metadata
        let hash = hasher.finalize();
        let now: DateTime<Utc> = Utc::now();
        let eol = now
            .checked_add_days(Days::new(days))
            .ok_or(Error::Logical)?;

        let cost = self.space_cost_doc * days as i64;
        if cost > changes.space {
            changes.space = 0;
            return Err(Error::CostSpace);
        } else {
            changes.space -= cost;
        }

        // Insert into CockroachDB
        sqlx::query(
            "INSERT INTO meme_meta (uid, oid, hash, size, pub, tip, eol) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&uid.0[..])
        .bind(&oid.0[..])
        .bind(hash.as_bytes().as_slice())
        .bind(size as i64)
        .bind(false)
        .bind(0_i64)
        .bind(eol)
        .execute(&self.db.crdb)
        .await?;

        let now = Instant::now();
        let remaining: Duration = deadline - now;
        changes.time = remaining.as_millis() as i64 * self.time_cost;

        Ok(Reply::MemePut {
            changes: *changes,
            hash: hash.into(),
        })
    }

    /// Current implementation uses high-level stream.
    /// Further investigation on performance is required.
    pub async fn get_meme(
        &self,
        uid: &Id,
        changes: &mut Costs,
        deadline: Instant,
        hash: Hash,
        public: bool,
    ) -> Result<Reply> {
        let hash_bytes = hash.as_slice();
        let uid_bytes = &uid.0[..];

        // Query with appropriate filter
        let row = if public {
            sqlx::query(
                "SELECT oid, uid, size, tip FROM meme_meta 
                 WHERE pub = true AND hash = $1 
                 ORDER BY tip ASC 
                 LIMIT 1",
            )
            .bind(hash_bytes)
            .fetch_optional(&self.db.crdb)
            .await
            .map_err(|_| Error::MemeGet)?
        } else {
            sqlx::query(
                "SELECT oid, uid, size, tip FROM meme_meta 
                 WHERE uid = $1 AND hash = $2 
                 ORDER BY tip ASC 
                 LIMIT 1",
            )
            .bind(uid_bytes)
            .bind(hash_bytes)
            .fetch_optional(&self.db.crdb)
            .await
            .map_err(|_| Error::MemeGet)?
        };

        let row = row.ok_or(Error::MemeNotFound)?;

        let oid: Vec<u8> = row.get("oid");
        let meme_uid_bytes: Vec<u8> = row.get("uid");
        let size: i64 = row.get("size");
        let tip: i64 = row.get("tip");

        // Is fund enough for the file size
        let cost = self.traffic_cost * size;
        if cost > changes.traffic {
            return Err(Error::CostTraffic);
        }
        changes.traffic -= cost;

        // Pay tip
        if public {
            if tip > changes.tip {
                return Err(Error::CostTip);
            }
            changes.tip -= tip;

            let mut meme_uid = Id::zero();
            meme_uid.0.copy_from_slice(&meme_uid_bytes);

            self.db
                .incr_credit(&meme_uid, Some(uid), tip, "MemeTip")
                .await?;
        }

        // Stream object
        let oid_hex = hex::encode(&oid);
        let mr = &self.db.mr;
        let stream = Box::pin(mr.get_object_stream(&oid_hex).await?);
        let now = Instant::now();
        let remaining: Duration = deadline - now;
        changes.time = remaining.as_millis() as i64 * self.time_cost;

        Ok(Reply::MemeGet {
            changes: *changes,
            raw: stream,
        })
    }
}
