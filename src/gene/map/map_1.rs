//! Map
//!
//! A MongoDB wrapper provides the mapping abstaction for other genes.
//!
//! # VOxOV managed fields
//!
//! - _id: unique identifier.
//! - _uid: user identifier.
//! - _pub: visibility.
//! - _eol: end of life.
//! - _tip: price for get.
//! - _size: the size of doc.
//!
//! _id and _uid are immutable.
//! _pub is managed by the censor gene.
//! _eol is set in request, and it can be extended.
//!
//! # Indexed fields
//!
//! - _ns: namespace.
//! - _i: indexed keys. They are _0, _1, _2, ... , _7.
//! - _n: max doc count.
//! - _geo: geospacial information.
//!
//! _ns is a history lesson in engineering.
//! _i can have various types. Their meaning is defined under _ns.
//! Range query is supported as [_i, _i_].
//! _geo is managed by gene geo.

#![allow(clippy::just_underscores_and_digits)]

use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::ir::{Costs, Id};
use crate::Result;
use bson::oid::ObjectId;
use bson::{doc, to_bson, Document};
use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, Utc};
use mongodb::options::{FindOneAndDeleteOptions, FindOptions};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap as Map;
use std::io::Write;
use std::str::FromStr;
use tokio::time::Instant;
use tokio_stream::StreamExt;

#[derive(Deserialize, Debug)]
struct Put {
    _type: String,
    _id: Option<ObjectId>,
    // Uid is managed by auth.

    // Pub is managed by censor.
    #[serde(with = "ts_seconds")]
    _eol: DateTime<Utc>,
    _tip: Option<i64>,
    // Size is counted by backend.
    _ns: Option<String>,

    _0: Value,
    _1: Value,
    _2: Value,
    _3: Value,
    _4: Value,
    _5: Value,
    _6: Value,
    _7: Value,

    _geo: Option<Vec<f64>>,

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
struct Get {
    _type: String,
    _id: Option<ObjectId>,
    _uid: Option<String>,
    _pub: Option<bool>,

    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    _eol_: Option<DateTime<Utc>>,

    _tip: Option<i64>,
    _tip_: Option<i64>,

    _size: Option<i64>,
    _size_: Option<i64>,

    _ns: Option<String>,
    _ns_: Option<String>,

    _0: Option<Value>,
    _1: Option<Value>,
    _2: Option<Value>,
    _3: Option<Value>,
    _4: Option<Value>,
    _5: Option<Value>,
    _6: Option<Value>,
    _7: Option<Value>,

    _0_: Option<Value>,
    _1_: Option<Value>,
    _2_: Option<Value>,
    _3_: Option<Value>,
    _4_: Option<Value>,
    _5_: Option<Value>,
    _6_: Option<Value>,
    _7_: Option<Value>,

    /// Max doc count.
    _n: Option<u64>,

    _geo: Option<Vec<f64>>,

    /// Selected fields.
    _v: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Drop {
    _type: String,
    _id: Option<ObjectId>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_type")]
enum Request {
    Put(Put),
    Get(Get),
    Drop(Drop),
}

pub struct V1Context<'a> {
    pub uid: &'a Id,
    pub arg: &'a String,
    pub changes: &'a mut Costs,
    pub deadline: Instant,
    pub space_cost: i64,
    pub traffic_cost: i64,
    pub db: &'static Database,
}

pub async fn v1(cx: V1Context<'_>, internal: bool) -> Result<String> {
    macro_rules! refund_space {
        ($d: expr) => {
            let eol: chrono::DateTime<Utc> = (*$d.get_datetime("_eol")?).into();
            let size = $d.get_i64("_size")?;
            let now = Utc::now();
            if now > eol {
                return Err(Error::GeneMapExpired);
            }
            let ttl = eol - now;
            let space = (size / 1024) * ttl.num_days() * cx.space_cost;
            cx.changes.space += space;
        };
    }

    let map = &cx.db.map;
    let request: Request = serde_json::from_str(cx.arg)?;
    match request {
        Request::Put(request) => {
            let ttl = request._eol - Utc::now();
            if ttl < Duration::days(1) {
                return Err(Error::CostTime);
            }

            let tip = request._tip.unwrap_or_default();
            if tip < 0 || tip > cx.changes.tip {
                return Err(Error::CostTip);
            }

            let ns = request._ns.unwrap_or_default();
            if !internal && !ns.is_empty() && ns.starts_with('_') {
                return Err(Error::Namespace);
            }

            if request._geo.is_some() && request._geo.as_ref().unwrap().len() != 2 {
                return Err(Error::GeoDim);
            }

            for k in request.v.keys() {
                if !k.is_empty() && k.starts_with('_') {
                    return Err(Error::ReservedKey);
                }
            }

            let _0 = to_bson(&request._0)?;
            let _1 = to_bson(&request._1)?;
            let _2 = to_bson(&request._2)?;
            let _3 = to_bson(&request._3)?;
            let _4 = to_bson(&request._4)?;
            let _5 = to_bson(&request._5)?;
            let _6 = to_bson(&request._6)?;
            let _7 = to_bson(&request._7)?;

            let mut d = doc! {
                "_uid": cx.uid.to_string(),
                "_pub": false,
                "_eol": request._eol,
                "_tip": tip,
                "_ns": ns,
                "_0": _0,
                "_1": _1,
                "_2": _2,
                "_3": _3,
                "_4": _4,
                "_5": _5,
                "_6": _6,
                "_7": _7,
                "_geo": request._geo,
                "_size": 0_i64,
            };

            for (k, v) in request.v {
                let v_bson = to_bson(&v)?;
                d.insert(k, v_bson);
            }

            let d_size = doc_size(&d) as i64;
            let s = d.get_i64_mut("_size")?;
            *s = d_size;

            let kb = (d_size + 1023) / 1024;
            let days = ttl.num_days();
            let mut space = kb.checked_mul(days).ok_or(Error::NumCheck)?;
            space = space.checked_mul(cx.space_cost).ok_or(Error::NumCheck)?;
            if cx.changes.space < space {
                return Err(Error::CostSpace);
            }
            cx.changes.space -= space;

            if let Some(id) = request._id {
                let mut filter = Document::new();
                filter.insert("_id", id);
                filter.insert("_uid", cx.uid.to_string());
                let found = map.find_one_and_replace(filter, d, None).await?;
                if let Some(old) = found {
                    refund_space!(old);
                }
            } else {
                map.insert_one(d, None).await?;
            }

            Ok("{}".into())
        }

        Request::Get(request) => {
            let mut filter = Document::new();

            request._id.and_then(|id| filter.insert("_id", id));

            if let Some(doc_uid) = request._uid {
                if cx.uid.to_string() == doc_uid {
                    request._pub.and_then(|p| filter.insert("_pub", p));
                } else {
                    filter.insert("_pub", true);
                }
                filter.insert("_uid", doc_uid);
            }

            macro_rules! filter_range {
                ($k:expr, $b:expr, $e:expr) => {
                    if let Some(begin) = $b {
                        if let Some(end) = $e {
                            filter.insert($k, doc! { "$gt": begin, "$lt": end });
                        } else {
                            filter.insert($k, begin);
                        }
                    }
                };
            }

            macro_rules! filter_key {
                ($k:expr, $b:expr, $e:expr) => {
                    if let Some(begin) = $b {
                        let begin = to_bson(&begin)?;
                        if let Some(end) = $e {
                            let end = to_bson(&end)?;
                            filter.insert($k, doc! { "$gt": begin, "$lt": end });
                        } else {
                            filter.insert($k, begin);
                        }
                    }
                };
            }

            filter_range!("_eol", request._eol, request._eol_);
            filter_range!("_tip", request._tip, request._tip_);
            filter_range!("_size", request._size, request._size_);
            filter_range!("_ns", request._ns, request._ns_);

            filter_key!("_0", request._0, request._0_);
            filter_key!("_1", request._1, request._1_);
            filter_key!("_2", request._2, request._2_);
            filter_key!("_3", request._3, request._3_);
            filter_key!("_4", request._4, request._4_);
            filter_key!("_5", request._5, request._5_);
            filter_key!("_6", request._6, request._6_);
            filter_key!("_7", request._7, request._7_);

            if let Some(geo) = request._geo {
                if geo.len() != 3 {
                    return Err(Error::GeoDim);
                }
                filter.insert(
                    "_geo",
                    doc! { "$geoWithin": {
                        "$centerSphere": [[geo[0], geo[1]], geo[2]],
                    }},
                );
            }

            let mut options = FindOptions::default();

            if let Some(values) = request._v {
                let mut proj = Document::new();
                for value in values {
                    proj.insert(value, 1);
                }
                options.projection = Some(proj);
            }

            options.max_time = Some(cx.deadline - Instant::now());

            let mut i = 0;
            let mut b = Document::new();
            let mut s = cx.changes.traffic / cx.traffic_cost;
            let mut cursor = map.find(filter, options).await?;
            while let Some(d) = cursor.try_next().await? {
                if let Some(n) = request._n {
                    if n == i {
                        b.insert("_error", "n");
                        break;
                    }
                }

                let d_size = doc_size(&d) as i64;
                if d_size > s {
                    return Err(Error::CostTraffic);
                }
                s -= d_size;

                let doc_uid = d.get_str("_uid")?;
                if doc_uid == cx.uid.to_string() {
                    continue;
                }

                let tip = d.get_i64("_tip")?;
                if tip > cx.changes.tip {
                    b.insert("_error", "tip");
                    b.insert("_error_id", d.get_object_id("_id")?);
                    b.insert("_error_tip", tip);
                    break;
                }
                cx.changes.tip -= tip;
                let u2c = ns(UID2CREDIT, &Id::from_str(doc_uid)?);
                cx.db.incrby(&u2c[..], tip).await?;

                b.insert(i.to_string(), d);
                i += 1;
            }

            Ok(b.to_string())
        }

        Request::Drop(request) => {
            let filter = doc! {
                "id": request._id,
                "uid": cx.uid.to_string(),
            };

            let options = FindOneAndDeleteOptions::builder()
                .projection(doc! {
                    "_id": 0,
                    "eol": 1,
                    "size": 1,
                })
                .build();

            let dropped = map
                .find_one_and_delete(filter.clone(), options)
                .await?
                .ok_or(Error::GeneMapNotFound)?;
            refund_space!(dropped);

            Ok("{}".into())
        }
    }
}

fn doc_size(d: &Document) -> usize {
    let mut c = Counter { n: 0 };
    let _ = d.to_writer(&mut c);
    c.n
}

struct Counter {
    pub n: usize,
}

impl Write for Counter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.n = buf.len();
        Ok(self.n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
