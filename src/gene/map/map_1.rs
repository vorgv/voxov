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
//! - _u: indexed keys. Current range is [0, 3].
//! - _n: max doc count.
//! - _geo: geospacial information.
//!
//! _ns is a history lesson in engineering.
//! _u can have various types. Their meaning is defined under _ns.
//! Range query is supported as [_u, _u_].
//! _geo is managed by gene geo.

#![allow(clippy::just_underscores_and_digits)]

use crate::error::Error;
use crate::message::{Costs, Id, Int, Uint};
use bson::oid::ObjectId;
use bson::{doc, to_bson, Document};
use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, Utc};
use mongodb::options::{FindOneAndDeleteOptions, FindOptions};
use mongodb::Collection;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap as Map;
use std::io::Write;
use tokio::time::Instant;
use tokio_stream::StreamExt;

#[derive(Deserialize, Debug)]
struct Insert {
    _type: String,
    // Id is managed by database.
    // Uid is managed by auth.

    // Pub is managed by censor.
    #[serde(with = "ts_seconds")]
    _eol: DateTime<Utc>,
    _tip: Option<Int>,
    // Size is counted by backend.
    _ns: Option<String>,

    _0: Value,
    _1: Value,
    _2: Value,
    _3: Value,

    _geo: Option<Vec<f64>>,

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
struct GetTip {}

#[derive(Deserialize, Debug)]
struct Query {
    _type: String,
    _id: Option<ObjectId>,
    _uid: Option<String>,
    _pub: Option<bool>,

    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    _eol_: Option<DateTime<Utc>>,

    _tip: Option<Int>,
    _tip_: Option<Int>,

    _size: Option<i64>,
    _size_: Option<i64>,

    _ns: Option<String>,
    _ns_: Option<String>,

    _0: Option<Value>,
    _1: Option<Value>,
    _2: Option<Value>,
    _3: Option<Value>,

    _0_: Option<Value>,
    _1_: Option<Value>,
    _2_: Option<Value>,
    _3_: Option<Value>,

    /// Max doc count.
    _n: Option<u64>,

    _geo: Option<Vec<f64>>,

    /// Selected fields.
    _v: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Delete {
    _type: String,
    _id: Option<ObjectId>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_type")]
enum Request {
    Insert(Insert),
    GetTip(GetTip),
    Query(Query),
    Delete(Delete),
}

pub async fn v1(
    uid: &Id,
    arg: &str,
    changes: &mut Costs,
    deadline: Instant,
    space_cost: Uint,
    traffic_cost: Uint,
    map: &'static Collection<Document>,
) -> Result<String, Error> {
    let request: Request = serde_json::from_str(arg)?;
    match request {
        Request::Insert(insert) => {
            let ttl = insert._eol - Utc::now();
            if ttl < Duration::days(1) {
                return Err(Error::CostTime);
            }

            let tip = insert._tip.unwrap_or_default();
            if tip < 0 || tip > changes.tip as i64 {
                return Err(Error::CostTip);
            }

            let ns = insert._ns.unwrap_or_default();
            if !ns.is_empty() && ns.starts_with('_') {
                return Err(Error::Namespace);
            }

            if insert._geo.is_some() && insert._geo.as_ref().unwrap().len() != 2 {
                return Err(Error::GeoDim);
            }

            for k in insert.v.keys() {
                if !k.is_empty() && k.starts_with('_') {
                    return Err(Error::ReservedKey);
                }
            }

            let _0 = to_bson(&insert._0)?;
            let _1 = to_bson(&insert._1)?;
            let _2 = to_bson(&insert._2)?;
            let _3 = to_bson(&insert._3)?;

            let mut d = doc! {
                "_uid": uid.to_string(),
                "_pub": false,
                "_eol": insert._eol,
                "_tip": tip,
                "_ns": ns,
                "_0": _0,
                "_1": _1,
                "_2": _2,
                "_3": _3,
                "_geo": insert._geo,
                "_size": 0_i64,
            };

            for (k, v) in insert.v {
                let v_bson = to_bson(&v)?;
                d.insert(k, v_bson);
            }

            let d_size = doc_size(&d) as i64;
            let s = d.get_i64_mut("_size")?;
            *s = d_size;

            let kb = (d_size as u64 + 1023) / 1024;
            let days = ttl.num_days() as u64;
            let mut space: u64 = kb.checked_mul(days).ok_or(Error::NumCheck)?;
            space = space.checked_mul(space_cost).ok_or(Error::NumCheck)?;
            if changes.space < space {
                return Err(Error::CostSpace);
            }
            changes.space -= space;

            map.insert_one(d, None).await?;

            Ok("{}".into())
        }

        Request::GetTip(_get_tip) => {
            todo!()
        }

        Request::Query(query) => {
            let mut filter = Document::new();

            query._id.and_then(|id| filter.insert("_id", id));

            if let Some(doc_uid) = query._uid {
                if uid.to_string() == doc_uid {
                    query._pub.and_then(|p| filter.insert("_pub", p));
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

            filter_range!("_eol", query._eol, query._eol_);
            filter_range!("_tip", query._tip, query._tip_);
            filter_range!("_size", query._size, query._size_);
            filter_range!("_ns", query._ns, query._ns_);

            filter_key!("_0", query._0, query._0_);
            filter_key!("_1", query._1, query._1_);
            filter_key!("_2", query._2, query._2_);
            filter_key!("_3", query._3, query._3_);

            if let Some(geo) = query._geo {
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

            if let Some(values) = query._v {
                let mut proj = Document::new();
                for value in values {
                    proj.insert(value, 1);
                }
                options.projection = Some(proj);
            }

            options.max_time = Some(deadline - Instant::now());

            let mut i = 0;
            let mut b = Document::new();
            let mut s = changes.traffic / traffic_cost;
            let mut cursor = map.find(filter, options).await?;
            while let Some(d) = cursor.try_next().await? {
                if let Some(n) = query._n {
                    if n == i {
                        break;
                    }
                }

                let d_size = doc_size(&d) as u64;
                if d_size > s {
                    return Err(Error::CostTraffic);
                }
                s -= d_size;

                b.insert(i.to_string(), d);
                i += 1;
            }

            Ok(b.to_string())
        }

        Request::Delete(delete) => {
            let filter = doc! {
                "id": delete._id,
                "uid": uid.to_string(),
            };

            let options = FindOneAndDeleteOptions::builder()
                .projection(doc! {
                    "_id": 0,
                    "eol": 1,
                    "size": 1,
                })
                .build();

            let d = map
                .find_one_and_delete(filter.clone(), options)
                .await?
                .ok_or(Error::GeneMapNotFound)?;

            let eol: chrono::DateTime<Utc> = (*d.get_datetime("_eol")?).into();
            let now = Utc::now();
            if now > eol {
                return Err(Error::GeneMapExpired);
            }
            let ttl = eol - now;
            let size = d.get_i64("_eol")?;
            let space = ((size / 1024) * ttl.num_days()) as u64 * space_cost;
            changes.space += space;

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
