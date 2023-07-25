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

use bson::oid::ObjectId;
use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;
use tokio::time::Instant;

use crate::message::{Costs, Id, Int, Uint};

#[derive(Serialize, Deserialize, Debug)]
struct RequestInsert {
    _type: String,
    // Id is managed by database.

    // Pub is managed by censor.
    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    _tips: Option<Int>,
    // Size is counted by backend.
    _ns: Option<String>,

    _0: Option<Value>,
    _1: Option<Value>,
    _2: Option<Value>,
    _3: Option<Value>,

    _0_: Option<Value>,
    _1_: Option<Value>,
    _2_: Option<Value>,
    _3_: Option<Value>,

    _geo: Option<Vec<f64>>,

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestQuery {
    _type: String,
    _id: Option<ObjectId>,
    _uid: String,

    #[serde(default)]
    _pub: bool,
    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    _tips: Option<Int>,
    _size: usize,
    _ns: Option<String>,

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

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestUpdate {
    _type: String,
    _id: Option<ObjectId>,

    // Pub is managed by censor.
    #[serde(with = "ts_seconds_option")]
    _eol: Option<DateTime<Utc>>,
    _tips: Option<Int>,
    // Size is counted by backend.
    _ns: Option<String>,

    _0: Option<Value>,
    _1: Option<Value>,
    _2: Option<Value>,
    _3: Option<Value>,

    _0_: Option<Value>,
    _1_: Option<Value>,
    _2_: Option<Value>,
    _3_: Option<Value>,

    _geo: Option<Vec<f64>>,

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestDelete {
    _type: String,
    _id: Option<ObjectId>,
}

pub async fn v1(
    _uid: &Id,
    _arg: &str,
    _changes: &mut Costs,
    _space: Uint,
    _deadline: Instant,
) -> String {
    "".to_string()
}