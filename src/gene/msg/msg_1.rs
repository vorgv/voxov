#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(clippy::just_underscores_and_digits)]

//! Message v1
//!
//! Both FROM and TO can delete the message.
//! No public flag needed, but TO can report.

use crate::{
    database::{namespace::UID2CREDIT, ns},
    error::Error,
    gene::map,
    ir::Id,
    Result,
};
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use tokio::time::Instant;

const NS: &str = "_chan";
const FROM: &str = "_0";
const TO: &str = "_1";
const SENT: &str = "_2";
const SENT_: &str = "_2_";
const READ: &str = "_3";
const READ_: &str = "_3_";
const TIP: &str = "_4";
const TIP_: &str = "_4_";
const TYPE: &str = "_5";
const VALUE: &str = "value";

#[derive(Deserialize, Debug)]
struct Send {
    id: Option<ObjectId>,
    eol: DateTime<Utc>,
    to: String,
    tip: i64,
    r#type: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct Sent {
    id: Option<ObjectId>,
    eol: DateTime<Utc>,
    eol_: DateTime<Utc>,
    to: Option<String>,
    sent: Option<DateTime<Utc>>,
    sent_: Option<DateTime<Utc>>,
    read: Option<DateTime<Utc>>,
    read_: Option<DateTime<Utc>>,
    tip: Option<i64>,
    tip_: Option<i64>,
    size: Option<i64>,
    size_: Option<i64>,
    r#type: Option<String>,
    n: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Receive {
    from: Option<String>,
    sent: Option<DateTime<Utc>>,
    sent_: Option<DateTime<Utc>>,
    read: Option<DateTime<Utc>>,
    read_: Option<DateTime<Utc>>,
    tip: Option<i64>,
    r#type: Option<String>,
    n: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Read {
    id: ObjectId,
}

#[derive(Deserialize, Debug)]
struct Unread {
    id: ObjectId,
}

#[derive(Deserialize, Debug)]
struct Report {
    id: ObjectId,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum Request {
    Send(Send),
    Sent(Sent),
    Receive(Receive),
    Read(Read),
    Unread(Unread),
    Report(Report),
}

pub async fn v1(mut cx: map::V1Context<'_>) -> Result<String> {
    let request: Request = serde_json::from_str(cx.arg)?;
    match request {
        Request::Send(request) => {
            let db = &cx.db;
            let map = &db.map;

            if let Some(doc_id) = request.id {
                let filter = doc! {
                    "_id": doc_id,
                    "_uid": cx.uid.to_string(),
                    "_ns": NS,
                };

                let options = FindOneOptions::builder()
                    .max_time(cx.deadline - Instant::now())
                    .build();

                if map.find_one(filter, options).await?.is_none() {
                    return Err(Error::GeneInvalidId);
                }
            }

            let to = Id::try_from(&request.to)?;
            let u2c = ns(UID2CREDIT, &to);

            if db.exits(&u2c[..]).await? < 1 {
                return Err(Error::AuthInvalidUid);
            }

            if 0 < request.tip || request.tip < cx.changes.tip {
                return Err(Error::CostTip);
            }

            db.incrby(&u2c[..], request.tip).await?;

            let arg = json!({
                "_type": "Put",
                "_id": request.id,
                "_eol": request.eol,
                "_ns": NS,
                TO: request.to,
                TIP: request.tip,
                TYPE: request.r#type,
                VALUE: request.value,
            })
            .to_string();

            cx.arg = &arg;
            map::v1(cx, true).await
        }

        Request::Sent(request) => {
            let arg = json!({
                "_type": "Get",
                "_id": request.id,
                "_uid": cx.uid.to_string(),
                "_eol": request.eol,
                "_eol_": request.eol_,
                "_ns": NS,
                TO: request.to,
                SENT: request.sent,
                SENT_: request.sent_,
                READ: request.read,
                READ_: request.read_,
                TIP: request.tip,
                TIP_: request.tip_,
                "_size": request.size,
                "_size_": request.size_,
                TYPE: request.r#type,
                "_n": request.n,
            })
            .to_string();

            cx.arg = &arg;
            map::v1(cx, true).await
        }

        Request::Receive(request) => todo!(),
        Request::Read(request) => todo!(),
        Request::Unread(request) => todo!(),
        Request::Report(request) => todo!(),
    }
}
