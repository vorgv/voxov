#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(clippy::just_underscores_and_digits)]

//! Message v1
//!
//! Both FROM and TO can delete the message.
//! No public flag needed, but TO can report.

use crate::{
    database::{namespace::UID2PHONE, ns},
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
const READ: &str = "_3";
const TIP: &str = "_4";
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
    to: Option<String>,
    sent: Option<DateTime<Utc>>,
    sent_: Option<DateTime<Utc>>,
    read: Option<DateTime<Utc>>,
    read_: Option<DateTime<Utc>>,
    tip: Option<i64>,
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
            let u2c = ns(UID2PHONE, &to);
            if db.exits(&u2c[..]).await? < 1 {
                return Err(Error::AuthInvalidUid);
            }

            if request.tip < 0 {
                return Err(Error::CostTip);
            }

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
        Request::Sent(request) => todo!(),
        Request::Receive(request) => todo!(),
        Request::Read(request) => todo!(),
        Request::Unread(request) => todo!(),
        Request::Report(request) => todo!(),
    }
}
