use super::{Costs, Hash, Id};
use crate::api::{empty, full, not_implemented};
use crate::error::Error;
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Response, StatusCode};
use std::convert::Infallible;

pub enum Reply {
    Unimplemented,
    Error { error: Error },
    AuthSessionStart { access: Id, refresh: Id },
    AuthSessionRefresh { access: Id },
    AuthSessionEnd,
    AuthSmsSendTo { phone: &'static String, message: Id },
    AuthSmsSent { uid: Id },
    CostPay { uri: String },
    GeneMeta { change: Costs, meta: String },
    GeneCall { change: Costs, result: String },
    MemeMeta { change: Costs, meta: String },
    MemeRawPut { change: Costs, hash: Hash },
    MemeRawGet { change: Costs, raw: Incoming },
}

impl Reply {
    pub fn to_response(&self) -> Response<BoxBody<Bytes, Infallible>> {
        macro_rules! response_change {
            ($change: expr) => {
                Response::builder()
                    .header("time", $change.time)
                    .header("space", $change.space)
                    .header("traffic", $change.traffic)
                    .header("tips", $change.tips)
            };
        }
        match self {
            Reply::Unimplemented => not_implemented(),
            Reply::Error { error } => Response::builder()
                .header("type", "Error")
                .header("error", error.to_string())
                .status(match error {
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                })
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
                .header("uri", uri)
                .body(empty())
                .unwrap(),
            Reply::GeneMeta { change, meta } => response_change!(change)
                .header("type", "GeneMeta")
                .body(full(meta.clone()))
                .unwrap(),
            Reply::GeneCall { change, result } => response_change!(change)
                .header("type", "GeneCall")
                .body(full(result.clone()))
                .unwrap(),
            Reply::MemeMeta { change, meta } => response_change!(change)
                .header("type", "MemeMeta")
                .body(full(meta.clone()))
                .unwrap(),
            Reply::MemeRawPut { change, hash } => response_change!(change)
                .header("type", "MemeRawPut")
                .header("type", hex::encode(hash))
                .body(empty())
                .unwrap(),
            _ => not_implemented(),
        }
    }
}
