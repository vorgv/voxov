use super::{Costs, Hash, Id};
use crate::api::{empty, full};
use crate::body::ResponseBody as RB;
use crate::body::S3StreamItem;
use crate::Error;
use http::response::Builder;
use http_body_util::StreamBody;
use hyper::{Response, StatusCode};
use s3::request::{DataStream, ResponseDataStream};
use std::mem::replace;
use std::pin::Pin;

type BoxS3Stream = Pin<Box<ResponseDataStream>>;

pub enum Reply {
    Error { error: Error },
    AuthSessionStart { access: Id, refresh: Id },
    AuthSessionRefresh { access: Id },
    AuthSessionEnd,
    AuthSmsSendTo { phone: &'static String, message: Id },
    AuthSmsSent { uid: Id },
    CostPay { uri: String },
    CostGet { credit: i64 },
    CostCheckIn { award: i64 },
    GeneMeta { changes: Costs, meta: String },
    GeneCall { changes: Costs, result: String },
    MemeMeta { changes: Costs, meta: String },
    MemePut { changes: Costs, hash: Hash },
    MemeGet { changes: Costs, raw: BoxS3Stream },
}

impl Reply {
    pub fn to_response(self) -> Response<RB> {
        fn response_changes(changes: Costs) -> Builder {
            Response::builder()
                .header("time", changes.time)
                .header("space", changes.space)
                .header("traffic", changes.traffic)
                .header("tip", changes.tip)
        }

        fn take_inner(mut s: BoxS3Stream) -> DataStream {
            replace(
                &mut s.bytes,
                Box::pin(tokio_stream::empty::<S3StreamItem>()),
            )
        }

        // Safe to unwrap here. Builders are infallible.

        if let Reply::MemeGet { changes, raw } = self {
            return response_changes(changes)
                .header("type", "MemeGet")
                .body(RB::S3Stream(StreamBody::new(take_inner(raw))))
                .unwrap();
        }

        match self {
            Reply::Error { error } => Response::builder()
                .header("type", "Error")
                .header("error", error.to_string())
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(empty())
                .unwrap(),
            Reply::AuthSessionStart { access, refresh } => Response::builder()
                .header("type", "AuthSessionStart")
                .header("access", access.to_string())
                .header("refresh", refresh.to_string())
                .body(empty())
                .unwrap(),
            Reply::AuthSessionRefresh { access } => Response::builder()
                .header("type", "AuthSessionRefresh")
                .header("access", access.to_string())
                .body(empty())
                .unwrap(),
            Reply::AuthSessionEnd => Response::builder()
                .header("type", "AuthSessionEnd")
                .body(empty())
                .unwrap(),
            Reply::AuthSmsSendTo { phone, message } => Response::builder()
                .header("type", "AuthSmsSendTo")
                .header("phone", phone)
                .header("message", message.to_string())
                .body(empty())
                .unwrap(),
            Reply::AuthSmsSent { uid } => Response::builder()
                .header("type", "AuthSmsSent")
                .header("uid", uid.to_string())
                .body(empty())
                .unwrap(),
            Reply::CostPay { uri } => Response::builder()
                .header("type", "CostPay")
                .header("uri", uri.clone())
                .body(empty())
                .unwrap(),
            Reply::CostGet { credit } => Response::builder()
                .header("type", "CostGet")
                .header("credit", credit.to_string())
                .body(empty())
                .unwrap(),
            Reply::CostCheckIn { award } => Response::builder()
                .header("type", "CostCheckIn")
                .header("award", award.to_string())
                .body(empty())
                .unwrap(),
            Reply::GeneMeta { changes, meta } => response_changes(changes)
                .header("type", "GeneMeta")
                .body(full(meta.clone()))
                .unwrap(),
            Reply::GeneCall { changes, result } => response_changes(changes)
                .header("type", "GeneCall")
                .body(full(result.clone()))
                .unwrap(),
            Reply::MemeMeta { changes, meta } => response_changes(changes)
                .header("type", "MemeMeta")
                .body(full(meta.clone()))
                .unwrap(),
            Reply::MemePut { changes, hash } => response_changes(changes)
                .header("type", "MemePut")
                .header("hash", hex::encode(hash))
                .body(empty())
                .unwrap(),
            Reply::MemeGet { .. } => unreachable!(),
        }
    }
}
