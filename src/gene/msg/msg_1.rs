#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(clippy::just_underscores_and_digits)]

//! Message v1
//!
//! Both FROM and TO can delete the message.
//! No public flag needed, but TO can report.

use crate::{gene::map, Result};
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use serde::Deserialize;

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

pub async fn v1(cx: map::V1Context<'_>) -> Result<String> {
    let request: Request = serde_json::from_str(cx.arg)?;
    match request {
        Request::Send(request) => todo!(),
        Request::Sent(request) => todo!(),
        Request::Receive(request) => todo!(),
        Request::Read(request) => todo!(),
        Request::Unread(request) => todo!(),
        Request::Report(request) => todo!(),
    }
}
