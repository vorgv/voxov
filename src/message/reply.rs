use super::{Costs, Hash, Id};
use crate::api::{empty, full, not_implemented};
use crate::body::{BoxStream, ResponseBody as RB, StreamItem};
use crate::error::Error;
use http_body_util::StreamBody;
use hyper::{Response, StatusCode};
use std::mem::replace;

pub enum Reply {
    Unimplemented,
    Error { error: Error },
    AuthSessionStart { access: Id, refresh: Id },
    AuthSessionRefresh { access: Id },
    AuthSessionEnd,
    AuthSmsSendTo { phone: &'static String, message: Id },
    AuthSmsSent { uid: Id },
    CostPay { uri: String },
    GeneMeta { changes: Costs, meta: String },
    GeneCall { changes: Costs, result: String },
    MemeMeta { changes: Costs, meta: String },
    MemeRawPut { changes: Costs, hash: Hash },
    MemeRawGet { changes: Costs, raw: BoxStream },
}

impl Reply {
    pub fn to_response(&mut self) -> Response<RB> {
        macro_rules! response_changes {
            ($changes: expr) => {
                Response::builder()
                    .header("time", $changes.time)
                    .header("space", $changes.space)
                    .header("traffic", $changes.traffic)
                    .header("tips", $changes.tips)
            };
        }

        if let Reply::MemeRawGet { changes, raw } = self {
            return response_changes!(changes)
                .header("type", "MemeRawGet")
                .body(RB::Stream(StreamBody::new(replace(
                    raw,
                    Box::pin(tokio_stream::empty::<StreamItem>()),
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
            Reply::MemeRawPut { changes, hash } => response_changes!(changes)
                .header("type", "MemeRawPut")
                .header("hash", hex::encode(hash))
                .body(empty())
                .unwrap(),
            _ => not_implemented(),
        }
    }
}
