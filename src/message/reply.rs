use super::{Costs, Hash, Id, Uint};
use crate::api::{empty, full, not_implemented};
use crate::error::Error;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::convert::Infallible;

pub enum Reply {
    Unimplemented,
    Error {
        error: Error,
    },
    AuthSessionStart {
        access: Id,
        refresh: Id,
    },
    AuthSessionRefresh {
        access: Id,
    },
    AuthSessionEnd,
    AuthSmsSendTo {
        phone: &'static String,
        message: Id,
    },
    AuthSmsSent {
        uid: Id,
    },
    CostPay {
        uri: String,
    },
    GeneMeta {
        change: Costs,
        meta: String,
    },
    GeneCall {
        change: Costs,
        result: String,
    },
    MemeMeta {
        change: Costs,
        meta: String,
    },
    MemeRawPut {
        change: Costs,
        key: Hash,
    },
    MemeRawGet {
        change: Costs,
        raw: Result<Box<[u8]>, Error>,
    },
}

impl Reply {
    pub fn to_response(&self) -> Response<BoxBody<Bytes, Infallible>> {
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
                .status(StatusCode::OK)
                .body(empty())
                .unwrap(),
            Reply::AuthSessionRefresh { access } => Response::builder()
                .header("type", "AuthSessionRefresh")
                .header("access", access.to_string())
                .status(StatusCode::OK)
                .body(empty())
                .unwrap(),
            Reply::AuthSessionEnd => Response::builder()
                .header("type", "AuthSessionEnd")
                .status(StatusCode::OK)
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
            Reply::GeneMeta {
                change: costs,
                meta,
            } => Response::builder()
                .header("type", "GeneMeta")
                .header("time", costs.time)
                .header("space", costs.space)
                .header("traffic", costs.traffic)
                .header("tips", costs.tips)
                .body(full(meta.clone()))
                .unwrap(),
            Reply::GeneCall {
                change: costs,
                result,
            } => Response::builder()
                .header("type", "GeneCall")
                .header("time", costs.time)
                .header("space", costs.space)
                .header("traffic", costs.traffic)
                .header("tips", costs.tips)
                .body(full(result.clone()))
                .unwrap(),
            _ => not_implemented(),
        }
    }
}
