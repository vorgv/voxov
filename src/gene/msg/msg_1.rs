#![allow(clippy::just_underscores_and_digits)]

use crate::{
    database::Database,
    message::{Costs, Id, Uint},
    Result,
};
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tokio::time::Instant;

const NS: &str = "_chan";
const FROM: &str = "_0";
const TO: &str = "_1";
const SENT: &str = "_2";
const READ: &str = "_3";
const TYPE: &str = "_4";
const VALUE: &str = "value";

#[derive(Deserialize, Debug)]
struct Send {
    to: String,
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
#[serde(tag = "_type")]
enum Request {
    Send(Send),
    Sent(Sent),
    Receive(Receive),
    Read(Read),
    Unread(Unread),
}

pub async fn v1(
    uid: &Id,
    arg: &str,
    changes: &mut Costs,
    deadline: Instant,
    space_cost: Uint,
    traffic_cost: Uint,
    db: &'static Database,
) -> Result<String> {
    todo!()
}
