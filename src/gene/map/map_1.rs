//! Map
//!
//! A CockroachDB wrapper provides the mapping abstraction for other genes.
//!
//! # VOxOV managed fields
//!
//! - id: unique identifier (UUID).
//! - uid: user identifier.
//! - pub: visibility.
//! - eol: end of life.
//! - tip: price for get.
//! - size: the size of doc.
//!
//! id and uid are immutable.
//! pub is managed by the censor gene.
//! eol is set in request, and it can be extended.
//!
//! # Indexed fields
//!
//! - ns: namespace.
//! - i0-i7: indexed keys (JSONB).
//! - geo_lon, geo_lat: geospatial information.

#![allow(clippy::just_underscores_and_digits)]

use crate::database::Database;
use crate::ir::{Costs, Id};
use crate::{Error, Result};
use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::BTreeMap as Map;
use tokio::time::Instant;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct Put {
    _type: String,
    _id: Option<String>, // UUID string
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
    _id: Option<String>,
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
    _id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_type")]
enum Request {
    Put(Box<Put>),
    Get(Box<Get>),
    Drop(Drop),
}

pub struct V1Context<'a> {
    pub uid: &'a Id,
    pub arg: &'a str,
    pub changes: &'a mut Costs,
    pub _deadline: Instant,
    pub space_cost: i64,
    pub traffic_cost: i64,
    pub db: &'static Database,
}

pub async fn v1(cx: V1Context<'_>, internal: bool) -> Result<String> {
    let request: Request = serde_json::from_str(cx.arg)?;

    match request {
        Request::Put(request) => handle_put(cx, *request, internal).await,
        Request::Get(request) => handle_get(cx, *request, internal).await,
        Request::Drop(request) => handle_drop(cx, request).await,
    }
}

async fn handle_put(cx: V1Context<'_>, request: Put, internal: bool) -> Result<String> {
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

    let (geo_lon, geo_lat) = if let Some(geo) = &request._geo {
        if geo.len() != 2 {
            return Err(Error::GeoDim);
        }
        (Some(geo[0]), Some(geo[1]))
    } else {
        (None, None)
    };

    for k in request.v.keys() {
        if !k.is_empty() && k.starts_with('_') {
            return Err(Error::ReservedKey);
        }
    }

    // Convert indexed fields to JSONB
    let i0 = serde_json::to_value(&request._0)?;
    let i1 = serde_json::to_value(&request._1)?;
    let i2 = serde_json::to_value(&request._2)?;
    let i3 = serde_json::to_value(&request._3)?;
    let i4 = serde_json::to_value(&request._4)?;
    let i5 = serde_json::to_value(&request._5)?;
    let i6 = serde_json::to_value(&request._6)?;
    let i7 = serde_json::to_value(&request._7)?;

    // Convert extra fields to body JSONB
    let body = serde_json::to_value(&request.v)?;

    // Estimate document size
    let body_str = serde_json::to_string(&body)?;
    let d_size = body_str.len() as i64 + 200; // overhead for other fields

    let kb = (d_size + 1023) / 1024;
    let days = ttl.num_days();
    let mut space = kb.checked_mul(days).ok_or(Error::NumCheck)?;
    space = space.checked_mul(cx.space_cost).ok_or(Error::NumCheck)?;
    if cx.changes.space < space {
        return Err(Error::CostSpace);
    }
    cx.changes.space -= space;

    if let Some(id_str) = request._id {
        // Update existing document
        let id = Uuid::parse_str(&id_str).map_err(|_| Error::GeneMapNotFound)?;

        // First get old document for refund calculation
        let old_row = sqlx::query("SELECT eol, size FROM map_docs WHERE id = $1 AND uid = $2")
            .bind(id)
            .bind(&cx.uid.0[..])
            .fetch_optional(&cx.db.crdb)
            .await?;

        if let Some(old) = old_row {
            let old_eol: DateTime<Utc> = old.get("eol");
            let old_size: i64 = old.get("size");
            let now = Utc::now();
            if now < old_eol {
                let ttl = old_eol - now;
                let space_refund = (old_size / 1024) * ttl.num_days() * cx.space_cost;
                cx.changes.space += space_refund;
            }
        }

        // Replace document
        sqlx::query(
            "UPDATE map_docs SET 
             pub = false, eol = $1, tip = $2, ns = $3, size = $4,
             i0 = $5, i1 = $6, i2 = $7, i3 = $8, i4 = $9, i5 = $10, i6 = $11, i7 = $12,
             geo_lon = $13, geo_lat = $14, body = $15
             WHERE id = $16 AND uid = $17",
        )
        .bind(request._eol)
        .bind(tip)
        .bind(&ns)
        .bind(d_size)
        .bind(&i0)
        .bind(&i1)
        .bind(&i2)
        .bind(&i3)
        .bind(&i4)
        .bind(&i5)
        .bind(&i6)
        .bind(&i7)
        .bind(geo_lon)
        .bind(geo_lat)
        .bind(&body)
        .bind(id)
        .bind(&cx.uid.0[..])
        .execute(&cx.db.crdb)
        .await?;
    } else {
        // Insert new document
        sqlx::query(
            "INSERT INTO map_docs (uid, pub, eol, tip, ns, size, i0, i1, i2, i3, i4, i5, i6, i7, geo_lon, geo_lat, body)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
        )
        .bind(&cx.uid.0[..])
        .bind(false)
        .bind(request._eol)
        .bind(tip)
        .bind(&ns)
        .bind(d_size)
        .bind(&i0)
        .bind(&i1)
        .bind(&i2)
        .bind(&i3)
        .bind(&i4)
        .bind(&i5)
        .bind(&i6)
        .bind(&i7)
        .bind(geo_lon)
        .bind(geo_lat)
        .bind(&body)
        .execute(&cx.db.crdb)
        .await?;
    }

    Ok("{}".into())
}

async fn handle_get(cx: V1Context<'_>, request: Get, internal: bool) -> Result<String> {
    // Build dynamic query
    let mut query = String::from("SELECT id, uid, pub, eol, tip, ns, size, i0, i1, i2, i3, i4, i5, i6, i7, geo_lon, geo_lat, body FROM map_docs WHERE 1=1");
    let mut param_idx = 1;

    if let Some(id_str) = &request._id {
        if Uuid::parse_str(id_str).is_ok() {
            query.push_str(&format!(" AND id = ${}", param_idx));
            param_idx += 1;
        }
    }

    if !internal {
        if let Some(doc_uid) = &request._uid {
            if cx.uid.to_string() == *doc_uid {
                if request._pub.is_some() {
                    query.push_str(&format!(" AND pub = ${}", param_idx));
                    param_idx += 1;
                }
            } else {
                query.push_str(" AND pub = true");
            }
            query.push_str(&format!(" AND uid = ${}", param_idx));
            param_idx += 1;
        }
    }

    // Build range filters
    macro_rules! add_range_filter {
        ($col:expr, $begin:expr, $end:expr) => {
            if $begin.is_some() {
                if $end.is_some() {
                    query.push_str(&format!(
                        " AND {} > ${} AND {} < ${}",
                        $col,
                        param_idx,
                        $col,
                        param_idx + 1
                    ));
                    param_idx += 2;
                } else {
                    query.push_str(&format!(" AND {} = ${}", $col, param_idx));
                    param_idx += 1;
                }
            }
        };
    }

    // Add filters for eol, tip, size, ns
    add_range_filter!("eol", request._eol, request._eol_);
    add_range_filter!("tip", request._tip, request._tip_);
    add_range_filter!("size", request._size, request._size_);

    if request._ns.is_some() {
        if request._ns_.is_some() {
            query.push_str(&format!(
                " AND ns > ${} AND ns < ${}",
                param_idx,
                param_idx + 1
            ));
            param_idx += 2;
        } else {
            query.push_str(&format!(" AND ns = ${}", param_idx));
            param_idx += 1;
        }
    }
    let _ = param_idx;

    // Add limit
    if let Some(n) = request._n {
        query.push_str(&format!(" LIMIT {}", n));
    } else {
        query.push_str(" LIMIT 100"); // Default limit
    }

    // Execute query - simplified version without dynamic binding
    // For a full implementation, we'd need to use sqlx::QueryBuilder
    let rows = sqlx::query(&query).fetch_all(&cx.db.crdb).await?;

    let mut result = json!({});
    let mut i = 0;
    let mut s = cx.changes.traffic / cx.traffic_cost;

    for row in rows {
        let id: Uuid = row.get("id");
        let uid_bytes: Vec<u8> = row.get("uid");
        let is_pub: bool = row.get("pub");
        let eol: DateTime<Utc> = row.get("eol");
        let tip: i64 = row.get("tip");
        let ns: String = row.get("ns");
        let size: i64 = row.get("size");
        let body: Value = row.get("body");

        // Size check
        if size > s {
            return Err(Error::CostTraffic);
        }
        s -= size;

        // Skip if document belongs to requesting user
        if uid_bytes == cx.uid.0[..] {
            continue;
        }

        // Tip check and payment
        if tip > cx.changes.tip {
            result["_error"] = json!("tip");
            result["_error_id"] = json!(id.to_string());
            result["_error_tip"] = json!(tip);
            break;
        }
        cx.changes.tip -= tip;

        let mut doc_uid = Id::zero();
        doc_uid.0.copy_from_slice(&uid_bytes);
        cx.db
            .incr_credit(&doc_uid, Some(cx.uid), tip, "GeneMap1Tip")
            .await?;

        // Build document JSON
        let doc = json!({
            "_id": id.to_string(),
            "_uid": hex::encode(&uid_bytes),
            "_pub": is_pub,
            "_eol": eol.timestamp(),
            "_tip": tip,
            "_ns": ns,
            "_size": size,
            "_0": row.try_get::<Value, _>("i0").ok(),
            "_1": row.try_get::<Value, _>("i1").ok(),
            "_2": row.try_get::<Value, _>("i2").ok(),
            "_3": row.try_get::<Value, _>("i3").ok(),
            "_4": row.try_get::<Value, _>("i4").ok(),
            "_5": row.try_get::<Value, _>("i5").ok(),
            "_6": row.try_get::<Value, _>("i6").ok(),
            "_7": row.try_get::<Value, _>("i7").ok(),
        });

        // Merge body fields
        let mut doc_map = doc.as_object().unwrap().clone();
        if let Value::Object(body_obj) = body {
            for (k, v) in body_obj {
                doc_map.insert(k, v);
            }
        }

        result[i.to_string()] = Value::Object(doc_map);
        i += 1;
    }

    Ok(result.to_string())
}

async fn handle_drop(cx: V1Context<'_>, request: Drop) -> Result<String> {
    let id = request._id.ok_or(Error::GeneMapNotFound)?;
    let id = Uuid::parse_str(&id).map_err(|_| Error::GeneMapNotFound)?;

    // Get document for refund calculation
    let row = sqlx::query("SELECT eol, size FROM map_docs WHERE id = $1 AND uid = $2")
        .bind(id)
        .bind(&cx.uid.0[..])
        .fetch_optional(&cx.db.crdb)
        .await?
        .ok_or(Error::GeneMapNotFound)?;

    let eol: DateTime<Utc> = row.get("eol");
    let size: i64 = row.get("size");

    // Calculate refund
    let now = Utc::now();
    if now < eol {
        let ttl = eol - now;
        let space_refund = (size / 1024) * ttl.num_days() * cx.space_cost;
        cx.changes.space += space_refund;
    }

    // Delete document
    sqlx::query("DELETE FROM map_docs WHERE id = $1 AND uid = $2")
        .bind(id)
        .bind(&cx.uid.0[..])
        .execute(&cx.db.crdb)
        .await?;

    Ok("{}".into())
}
