use super::{Costs, Hash, Id, Int};
use crate::api::{empty, full, not_implemented};
use crate::body::ResponseBody as RB;
use crate::error::Error;
use http_body_util::StreamBody;
use hyper::{body::Bytes, Response, StatusCode};
use s3::request::ResponseDataStream;
use std::mem::replace;
use std::pin::Pin;

type BoxS3Stream = Pin<Box<ResponseDataStream>>;

pub enum Reply {
    Unimplemented,
    Error { error: Error },
    AuthSessionStart { access: Id, refresh: Id },
    AuthSessionRefresh { access: Id },
    AuthSessionEnd,
    AuthSmsSendTo { phone: &'static String, message: Id },
    AuthSmsSent { uid: Id },
    CostPay { uri: String },
    CostGet { credit: Int },
    GeneMeta { changes: Costs, meta: String },
    GeneCall { changes: Costs, result: String },
    MemeMeta { changes: Costs, meta: String },
    MemePut { changes: Costs, hash: Hash },
    MemeGet { changes: Costs, raw: BoxS3Stream },
}

impl Reply {
    pub fn to_response(&mut self) -> Response<RB> {
        macro_rules! response_changes {
            ($changes: expr) => {
                Response::builder()
                    .header("time", $changes.time)
                    .header("space", $changes.space)
                    .header("traffic", $changes.traffic)
                    .header("tip", $changes.tip)
            };
        }

        if let Reply::MemeGet { changes, raw } = self {
            return response_changes!(changes)
                .header("type", "MemeGet")
                .body(RB::Stream(StreamBody::new(replace(
                    raw.bytes(),
                    Box::pin(tokio_stream::empty::<Bytes>()),
                ))))
                .unwrap();
        }

        match self {
            Reply::Unimplemented => not_implemented(),
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
                .header("phone", *phone)
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
            Reply::GeneMeta { changes, meta } => response_changes!(changes)
                .header("type", "GeneMeta")
                .body(full(meta.clone()))
                .unwrap(),
            Reply::GeneCall { changes, result } => response_changes!(changes)
                .header("type", "GeneCall")
                .body(full(result.clone()))
                .unwrap(),
            Reply::MemeMeta { changes, meta } => response_changes!(changes)
                .header("type", "MemeMeta")
                .body(full(meta.clone()))
                .unwrap(),
            Reply::MemePut { changes, hash } => response_changes!(changes)
                .header("type", "MemePut")
                .header("hash", hex::encode(hash))
                .body(empty())
                .unwrap(),
            _ => not_implemented(),
        }
    }
}
