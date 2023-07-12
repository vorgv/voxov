use bytes::Bytes;
use chrono::{DateTime, Days, Utc};
use hyper::body::Body;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::IndexModel;
use s3::error::S3Error;
use s3::serde_types::Part;
use s3::Bucket;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
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
    space_cost_obj: Uint,
    space_cost_doc: Uint,
    traffic_cost: Uint,
}

impl Meme {
    pub fn new(config: &Config, db: &'static Database) -> Meme {
        Meme {
            db,
            ripperd_disabled: config.ripperd_disabled,
            ripperd_interval: config.ripperd_interval,
            space_cost_obj: config.space_cost_obj,
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
        changes: Costs,
        deadline: Instant,
        days: u64,
        raw: QueryBody,
    ) -> Result<Reply, Error> {
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
            .await
            .map_err(Error::S3)?;
        let (path, upload_id) = (msg.key, msg.upload_id);
        // Put chunks.
        let putter = Putter {
            space_cost_obj: self.space_cost_obj,
            content_type: Arc::from(content_type),
            mr,
            changes,
            deadline,
            days,
            raw,
            path: Arc::from(path),
            upload_id: Arc::from(upload_id),
            size: 0,
            hasher: blake3::Hasher::new(),
            part_number: 0,
            chunk_future: None,
        };
        let (mut changes, hash, size, maybe_error) = putter.await;
        if let Some(error) = maybe_error {
            return Err(error);
        }
        // Create metadata
        let now: DateTime<Utc> = Utc::now();
        let eol = now.checked_add_days(Days::new(days));
        let doc = doc! {
            "uid": uid.to_string(),
            "oid": oid.to_string(),
            "hash": hex::encode(hash),
            "size": size as i64,
            "public": false,
            "tips": 0,
            "eol": eol,
        };
        let cost = self.space_cost_doc * days;
        if cost > changes.space {
            changes.space = 0;
            return Err(Error::CostSpace);
        } else {
            changes.space -= cost;
        }
        let mm = &self.db.mm;
        mm.insert_one(doc, None).await.map_err(|_| Error::MongoDB)?;
        Ok(Reply::MemePut { changes, hash })
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

type ChunkFuture = Option<Pin<Box<dyn Future<Output = Result<Part, S3Error>> + Send>>>;

struct Putter {
    space_cost_obj: Uint,
    content_type: Arc<str>,
    mr: &'static Bucket,
    changes: Costs,
    deadline: Instant,
    days: u64,
    raw: QueryBody,
    path: Arc<str>,
    upload_id: Arc<str>,
    size: usize,
    hasher: blake3::Hasher,
    part_number: u32,
    chunk_future: ChunkFuture,
}

type PutterOutput = (Costs, Hash, usize, Option<Error>);

impl Putter {
    fn get_output(&self, maybe_error: Option<Error>) -> PutterOutput {
        (
            self.changes,
            *self.hasher.finalize().as_bytes(),
            self.size,
            maybe_error,
        )
    }
}

impl Future for Putter {
    type Output = PutterOutput;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Putting a chunk now?
        if self.chunk_future.is_some() {
            let poll = self.chunk_future.as_mut().unwrap().as_mut().poll(cx);
            if poll.is_pending() {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            let mut result = Err(S3Error::HttpFail);
            let _ = poll.map(|r| result = r);
            self.chunk_future = None;
            return match result {
                Ok(_) => Poll::Pending,
                Err(error) => Poll::Ready(self.get_output(Some(Error::S3(error)))),
            };
        }
        // Flags.
        let mut is_trailer = false;
        let mut bad_frame = false;
        let mut body_eof = false;
        let mut overflow = false;
        let mut space_out = false;
        let mut time_out = false;
        // If buffer is at its end, get a new frame.
        let poll = self.raw.as_mut().poll_frame(cx).map(|option| {
            match option {
                // Body reached the end?
                Some(result) => match result {
                    // Frame
                    Ok(frame) => {
                        if frame.is_data() {
                            let data = frame.into_data().unwrap_or_default();
                            self.hasher.update(&data);
                            // Space check
                            let cost = match (data.len() as u64 * self.space_cost_obj)
                                .checked_mul(self.days)
                            {
                                Some(i) => i / 1000, // per day per KB
                                None => return overflow = true,
                            };
                            if self.changes.space < cost {
                                self.changes.space = 0;
                                return space_out = true;
                            } else {
                                self.changes.space -= cost;
                            }
                            // Time check
                            if Instant::now() > self.deadline {
                                return time_out = true;
                            }
                            self.size += data.len();
                            self.part_number += 1;
                            //TODO wrap this future to pass argument by move.
                            let future = Box::pin(ChunkPutter {
                                mr: self.mr,
                                chunck: data,
                                path: self.path.clone(),
                                part_number: self.part_number,
                                upload_id: self.upload_id.clone(),
                                content_type: self.content_type.clone(),
                            });
                            self.chunk_future = Some(future);
                        } else {
                            is_trailer = true;
                        }
                    }
                    Err(_) => bad_frame = true,
                },
                None => body_eof = true,
            }
        });
        if poll.is_pending() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        // Handle flags.
        if body_eof {
            return Poll::Ready(self.get_output(None));
        }
        if bad_frame {
            return Poll::Ready(self.get_output(Some(Error::MemePut)));
        }
        if is_trailer {
            return Poll::Pending;
        }
        if overflow {
            return Poll::Ready(self.get_output(Some(Error::CostSpaceTooLarge)));
        }
        if space_out {
            return Poll::Ready(self.get_output(Some(Error::CostSpace)));
        }
        if time_out {
            return Poll::Ready(self.get_output(Some(Error::CostTime)));
        }
        Poll::Pending
    }
}

/// A wrapper to extend the lifetime of arguments.
struct ChunkPutter {
    mr: &'static Bucket,
    chunck: Bytes,
    path: Arc<str>,
    part_number: u32,
    upload_id: Arc<str>,
    content_type: Arc<str>,
}

impl Future for ChunkPutter {
    type Output = Result<Part, S3Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut future = Box::pin(self.mr.put_multipart_chunk(
            self.chunck.to_vec(),
            &self.path,
            self.part_number,
            &self.upload_id,
            &self.content_type,
        ));
        future.as_mut().poll(cx)
    }
}
