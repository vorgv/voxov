use crate::config::Config;
use crate::database::Database;
use crate::ir::query::QueryBody;
use crate::ir::{Costs, Hash, Id, Reply};
use crate::{Error, Result};
use chrono::{DateTime, Days, Utc};
use http_body_util::BodyExt;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use s3::bucket::CHUNK_SIZE;
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
    /// The driver of MongoDB breaks if internal futures are dropped.
    /// This limitation hinders tokio::select style timeout.
    pub async fn get_meta(&self, uid: &Id, deadline: Instant, hash: &Hash) -> Result<String> {
        let mm = &self.db.mm;
        let filter = doc! { "hash": hex::encode(hash) };
        let handle = tokio::task::spawn(async move { mm.find_one(filter, None).await });
        let option_meta = tokio::time::timeout_at(deadline, handle)
            .await
            .map_err(|_| Error::CostTime)?
            .map_err(|_| Error::CostTime)??;
        if let Some(meta) = option_meta {
            if meta.get_bool("pub").map_err(|_| Error::Logical)? {
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
        changes: &mut Costs,
        deadline: Instant,
        days: u64,
        mut raw: QueryBody,
    ) -> Result<Reply> {
        // Create object with a random name.
        let oid = {
            let mut rng = rand::thread_rng();
            Id::rand(&mut rng)?
        };
        // Init chunk upload.
        let mr = &self.db.mr;
        let content_type = "".to_string();
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
        let eol = now.checked_add_days(Days::new(days));
        let doc = doc! {
            "uid": uid.to_string(),
            "oid": oid.to_string(),
            "hash": hex::encode(hash.as_bytes()),
            "size": size as i64,
            "pub": false,
            "tip": 0,
            "eol": eol,
        };
        let cost = self.space_cost_doc * days as i64;
        if cost > changes.space {
            changes.space = 0;
            return Err(Error::CostSpace);
        } else {
            changes.space -= cost;
        }
        let mm = &self.db.mm;
        mm.insert_one(doc, None).await?;
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
        let hash = hex::encode(hash);
        // Filter
        let filter = match public {
            true => doc! {
                "pub": true,
                "hash": hash.clone(),
            },
            false => doc! {
                "uid": uid.to_string(),
                "hash": hash.clone(),
            },
        };
        // Sort by tip
        let options = FindOneOptions::builder()
            .projection(doc! { "oid": 1, "uid": 1, "hash": 1, "size": 1, "tip": 1, "_id": 0 })
            .sort(doc! { "tip": 1 })
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
        let cost = self.traffic_cost * meta.get_i64("size").map_err(|_| Error::Logical)?;
        if cost > changes.traffic {
            return Err(Error::CostTraffic);
        }
        changes.traffic -= cost;
        // Pay tip
        if public {
            let tip = meta.get_i64("tip").map_err(|_| Error::Logical)?;
            if tip > changes.tip {
                return Err(Error::CostTip);
            }
            changes.tip -= tip;
            let meme_uid = meta.get_str("uid").map_err(|_| Error::Logical)?;
            use std::str::FromStr;
            let meme_uid = Id::from_str(meme_uid)?;
            self.db
                .incr_credit(&meme_uid, Some(uid), tip, "MemeTip")
                .await?;
        }
        // Stream object
        let oid = meta.get_str("oid").map_err(|_| Error::Logical)?;
        let mr = &self.db.mr;
        let stream = Box::pin(mr.get_object_stream(oid).await?);
        let now = Instant::now();
        let remaining: Duration = deadline - now;
        changes.time = remaining.as_millis() as i64 * self.time_cost;

        Ok(Reply::MemeGet {
            changes: *changes,
            raw: stream,
        })
    }
}
