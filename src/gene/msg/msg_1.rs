//! Message v1
//!
//! Both FROM and TO can delete the message.
//! No public flag needed, but TO can report.

use crate::{gene::map, ir::Id, Error, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

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
    id: Option<String>, // UUID string
    eol: DateTime<Utc>,
    to: String,
    tip: i64,
    r#type: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct Sent {
    id: Option<String>,
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
    id: Option<String>,
    eol: DateTime<Utc>,
    eol_: DateTime<Utc>,
    from: Option<String>,
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
struct Read {
    id: String,
}

#[derive(Deserialize, Debug)]
struct Unread {
    id: String,
}

#[derive(Deserialize, Debug)]
struct Delete {
    id: String,
}

#[derive(Deserialize, Debug)]
struct Report {
    _id: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum Request {
    Send(Send),
    Sent(Sent),
    Receive(Receive),
    Read(Read),
    Unread(Unread),
    Delete(Delete),
    Report(Report),
}

pub async fn v1(mut cx: map::V1Context<'_>) -> Result<String> {
    let db = &cx.db;

    let request: Request = serde_json::from_str(cx.arg)?;
    match request {
        Request::Send(request) => {
            if let Some(doc_id) = &request.id {
                let id = Uuid::parse_str(doc_id).map_err(|_| Error::GeneInvalidId)?;

                let exists =
                    sqlx::query("SELECT 1 FROM map_docs WHERE id = $1 AND uid = $2 AND ns = $3")
                        .bind(id)
                        .bind(&cx.uid.0[..])
                        .bind(NS)
                        .fetch_optional(&db.crdb)
                        .await?;

                if exists.is_none() {
                    return Err(Error::GeneInvalidId);
                }
            }

            let to = Id::try_from(request.to.as_str())?;

            // Check if recipient user exists in TigerBeetle
            if !db.user_exists(&to).await? {
                return Err(Error::AuthInvalidUid);
            }

            if request.tip < 0 || request.tip > cx.changes.tip {
                return Err(Error::CostTip);
            }

            db.incr_credit(&to, Some(cx.uid), request.tip, "GeneMsg1Tip")
                .await?;

            let arg = json!({
                "_type": "Put",
                "_id": request.id,
                "_eol": request.eol.timestamp(),
                "_ns": NS,
                FROM: cx.uid.to_string(),
                TO: request.to,
                SENT: Utc::now().timestamp(),
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
                "_eol": request.eol.timestamp(),
                "_eol_": request.eol_.timestamp(),
                "_ns": NS,
                FROM: cx.uid.to_string(),
                TO: request.to,
                SENT: request.sent.map(|t| t.timestamp()),
                SENT_: request.sent_.map(|t| t.timestamp()),
                READ: request.read.map(|t| t.timestamp()),
                READ_: request.read_.map(|t| t.timestamp()),
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

        Request::Receive(request) => {
            let arg = json!({
                "_type": "Get",
                "_id": request.id,
                "_eol": request.eol.timestamp(),
                "_eol_": request.eol_.timestamp(),
                "_ns": NS,
                FROM: request.from,
                TO: cx.uid.to_string(),
                SENT: request.sent.map(|t| t.timestamp()),
                SENT_: request.sent_.map(|t| t.timestamp()),
                READ: request.read.map(|t| t.timestamp()),
                READ_: request.read_.map(|t| t.timestamp()),
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

        Request::Read(request) => {
            let id = Uuid::parse_str(&request.id).map_err(|_| Error::GeneMapNotFound)?;
            let uid_str = cx.uid.to_string();

            // Update the body JSONB to set _3 (READ) timestamp
            let result = sqlx::query(
                "UPDATE map_docs SET body = jsonb_set(COALESCE(body, '{}'::jsonb), '{_3}', to_jsonb($1::bigint))
                 WHERE id = $2 AND ns = $3 AND body->>'_1' = $4",
            )
            .bind(Utc::now().timestamp())
            .bind(id)
            .bind(NS)
            .bind(&uid_str)
            .execute(&db.crdb)
            .await?;

            if result.rows_affected() == 0 {
                return Err(Error::GeneMapNotFound);
            }

            Ok("{}".into())
        }

        Request::Unread(request) => {
            let id = Uuid::parse_str(&request.id).map_err(|_| Error::GeneMapNotFound)?;
            let uid_str = cx.uid.to_string();

            // Remove _3 (READ) from body JSONB
            let result = sqlx::query(
                "UPDATE map_docs SET body = body - '_3'
                 WHERE id = $1 AND ns = $2 AND body->>'_1' = $3",
            )
            .bind(id)
            .bind(NS)
            .bind(&uid_str)
            .execute(&db.crdb)
            .await?;

            if result.rows_affected() == 0 {
                return Err(Error::GeneMapNotFound);
            }

            Ok("{}".into())
        }

        Request::Delete(request) => {
            let id = Uuid::parse_str(&request.id).map_err(|_| Error::GeneMapNotFound)?;
            let uid_str = cx.uid.to_string();

            // Delete if user is either FROM or TO
            let result = sqlx::query(
                "DELETE FROM map_docs 
                 WHERE id = $1 AND ns = $2 AND (body->>'_0' = $3 OR body->>'_1' = $3)",
            )
            .bind(id)
            .bind(NS)
            .bind(&uid_str)
            .execute(&db.crdb)
            .await?;

            if result.rows_affected() == 0 {
                return Err(Error::GeneMapNotFound);
            }

            Ok("{}".into())
        }

        Request::Report(_request) => Err(Error::Todo),
    }
}
