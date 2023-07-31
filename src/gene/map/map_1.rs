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
//! - _tips: price.
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

use bson::doc;
use bson::oid::ObjectId;
use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;
use std::io::Write;
use tokio::time::Instant;

use crate::error::Error;
use crate::message::{Costs, Id, Int, Uint};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Query {
    _type: String,
    _id: Option<ObjectId>,
    _uid: Option<String>,

    /// Default: public and self.
    _pub: Option<bool>,

    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    _eol_: Option<DateTime<Utc>>,

    _tip: Option<Int>,
    _tip_: Option<Int>,

    _size: usize,
    _size_: usize,

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
    _n: i32,

    _geo: Option<Vec<f64>>,

    /// Selected fields.
    _v: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Update {
    _type: String,
    _id: Option<ObjectId>,

    // Pub is managed by censor.
    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    _tip: Option<Int>,
    // Size is counted by backend.
    _ns: Option<String>,

    _0: Option<Value>,
    _1: Option<Value>,
    _2: Option<Value>,
    _3: Option<Value>,

    _geo: Option<Vec<f64>>,

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Delete {
    _type: String,
    _id: Option<ObjectId>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "_type")]
enum Request {
    Insert(Insert),
    Query(Query),
    Update(Update),
    Delete(Delete),
}

pub async fn v1(
    uid: &Id,
    arg: &str,
    _changes: &mut Costs,
    _space: Uint,
    _deadline: Instant,
) -> Result<String, Error> {
    let request: Request = serde_json::from_str(arg)?;
    match request {
        Request::Insert(insert) => {
            let ttl = insert._eol - Utc::now();
            if ttl <= Duration::zero() {
                return Err(Error::CostTime);
            }

            let tip = insert._tip.unwrap_or_default();

            let ns = insert._ns.unwrap_or_default();
            if !ns.is_empty() && ns.starts_with('_') {
                return Err(Error::Namespace);
            }

            if insert._geo.is_some() && insert._geo.as_ref().unwrap().len() != 2 {
                return Err(Error::GeoDim);
            }

            for (k, _) in insert.v {
                if !k.is_empty() && k.starts_with('_') {
                    return Err(Error::ReservedKey);
                }
            }

            let _0 = bson::to_bson(&insert._0).unwrap();
            let _1 = bson::to_bson(&insert._1).unwrap();
            let _2 = bson::to_bson(&insert._2).unwrap();
            let _3 = bson::to_bson(&insert._3).unwrap();

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

            let mut c = Counter { n: 0 };
            let _ = d.to_writer(&mut c);
            let s = d.get_i64_mut("_size").unwrap();
            *s = c.n as i64;
            // Insert document with deadline.
            // Reply.
            todo!()
            //TODO Index these fields in db init.
        }
        Request::Query(_query) => {
            // Set id.
            // Set uid.
            // Set pub.
            // Set eol.
            // Set tip.
            // Set size.
            // Set ns.
            // Set keys.
            // Set max doc count.
            // Set geo.
            // Select fields.
            // Query document with deadline.
            // Reply.
            todo!()
        }
        Request::Update(_update) => {
            // Set id.
            // Set uid.
            // Get original doc eol & size.
            // Set eol.
            // Set tip.
            // Set ns.
            // Set keys.
            // Set geo.
            // Set user fields.
            // Count new size.
            // Calculate diff between eol and size, then update space.
            // Update document with deadline.
            // Reply.
            todo!()
        }
        Request::Delete(_delete) => {
            // Set id.
            // Set uid.
            // Calculate eol & size -> space.
            // Delete document with deadline.
            // Add space to changes & reply.
            todo!()
        }
    }
}

struct Counter {
    pub n: usize,
}

impl Write for Counter {
    fn write (&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.n = buf.len();
        Ok(self.n)
    }

    fn flush (&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
