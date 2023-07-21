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
//! - _k0, _k1, _k2, _k3: indexed keys. Might increase n in the future.
//! - _geo: geospacial data.
//!
//! _ns is a history lesson in engineering.
//! _kn can have various types. Their meaning is defined by _ns.
//! Range query is supported for _k*.
//! _geo is managed by gene geo.

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;
use tokio::time::Instant;

use crate::message::{Costs, Id, Int, Uint};

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    _type: String,
    _id: Option<String>,

    #[serde(default)]
    _pub: bool,
    _eol: Option<String>,
    _tips: Option<Int>,
    _size: usize,
    _ns: Option<String>,

    _k0: Option<Value>,
    _k1: Option<Value>,
    _k2: Option<Value>,
    _k3: Option<Value>,

    _k0_: Option<Value>,
    _k1_: Option<Value>,
    _k2_: Option<Value>,
    _k3_: Option<Value>,

    _geo: String,

    _v: Vec<String>,

    #[serde(flatten)]
    v: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DocHeader {
    _id: String,
    _pub: bool,
    #[serde(with = "ts_seconds")]
    _eol: DateTime<Utc>,
    _tips: i64,
    _size: u64,
    _ns: String,
    _k0: Value,
    _k1: Value,
    _k2: Value,
    _k3: Value,
    _geo: Vec<f64>,
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
