use chrono::{DateTime, Days, Utc};
use hyper::body::Body;
use mongodb::bson::spec::BinarySubtype;
use mongodb::bson::{doc, Binary};
use std::task::Poll;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::time::{sleep, Instant};

use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use crate::message::query::QueryBody;
use crate::message::{Costs, Hash, Id, Uint};

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
        let mm = &self.db.mm;
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

    pub async fn put_meme(
        &self,
        uid: &Id,
        mut changes: Costs,
        days: Uint,
        mut putter: &mut Putter,
    ) -> Result<(), Error> {
        // Create object with a random name.
        let oid = {
            let mut rng = rand::thread_rng();
            Id::rand(&mut rng)?
        };
        // On error, remove object.
        let mr = &self.db.mr;
        if (mr.put_object_stream(&mut putter, oid.to_string()).await).is_err() {
            mr.delete_object(oid.to_string())
                .await
                .map_err(|_| Error::S3)?;
            return Err(Error::MemeRawPut);
        }
        // Create meta-data.
        let hash: [u8; 32] = putter.get_hash().into();
        let now: DateTime<Utc> = Utc::now();
        let eol = now.checked_add_days(Days::new(days));
        let doc = doc! {
            "uid": uid.to_string(),
            "oid": oid.to_string(),
            "hash": hex::encode(hash),
            "public": false,
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
        Ok(())
    }
}

pub struct Putter {
    haser: blake3::Hasher,
    days: Uint,
    body: QueryBody,
    changes: Costs,
    deadline: Instant,
    space_cost_obj: Uint,
}

impl Putter {
    pub fn new(
        days: Uint,
        body: QueryBody,
        changes: Costs,
        deadline: Instant,
        space_cost_obj: Uint,
    ) -> Self {
        Putter {
            haser: blake3::Hasher::new(),
            days,
            body,
            changes,
            deadline,
            space_cost_obj,
        }
    }

    pub fn get_hash(&self) -> blake3::Hash {
        self.haser.finalize()
    }

    pub fn into_changes(self) -> Costs {
        self.changes
    }
}

impl AsyncRead for Putter {
    // Poll data frames from body while skip trailer frames.
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        use std::io;
        loop {
            let mut is_data = true;
            let poll =
                Body::poll_frame(self.body.as_mut(), cx).map(|option| -> std::io::Result<()> {
                    match option {
                        Some(result) => match result {
                            Ok(frame) => {
                                if frame.is_data() {
                                    let data = frame.into_data().unwrap();
                                    buf.put_slice(&data);
                                    // Space check
                                    let cost = match (data.len() as u64 * self.space_cost_obj)
                                        .checked_mul(self.days)
                                    {
                                        Some(i) => i / 1000, // per day per KB
                                        None => {
                                            return Err(io::Error::from(io::ErrorKind::InvalidData))
                                        }
                                    };
                                    if self.changes.space < cost {
                                        self.changes.space = 0;
                                        return Err(io::Error::from(io::ErrorKind::FileTooLarge));
                                    } else {
                                        self.changes.space -= cost;
                                    }
                                    // Time check
                                    if Instant::now() > self.deadline {
                                        return Err(io::Error::from(io::ErrorKind::TimedOut));
                                    }
                                } else {
                                    is_data = false;
                                }
                                Ok(())
                            }
                            Err(_) => Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                        },
                        None => Ok(()),
                    }
                });
            if is_data {
                return poll;
            }
        }
    }
}
