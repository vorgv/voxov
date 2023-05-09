use super::{Cost, Error, Hash, Id};
use crate::api::{empty, not_implemented, ok};
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::convert::Infallible;

pub enum Reply {
    Unimplemented,
    AuthSessionStart {
        access: Id,
        refresh: Id,
    },
    AuthSessionRefresh {
        access: Id,
    },
    AuthSessionEnd,
    AuthSmsSendTo {
        phone: String,
        message: String,
    },
    AuthSmsSent {
        pid: Id,
    },
    AuthError {
        error: Error,
    },
    Pay {
        uri: String,
    },
    MemeMeta {
        cost: Cost,
        meta: Result<String, Error>,
    },
    MemeRawPut {
        cost: Cost,
        key: Hash,
    },
    MemeRawGet {
        cost: Cost,
        raw: Result<Box<[u8]>, Error>,
    },
    GeneMeta {
        cost: Cost,
        meta: Result<String, Error>,
    },
    GeneCall {
        cost: Cost,
        result: Result<Option<Box<[u8]>>, Error>,
    },
}

impl Reply {
    pub fn to_response(&self) -> Response<BoxBody<Bytes, Infallible>> {
        match self {
            Reply::Unimplemented => not_implemented(),
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
            Reply::AuthSessionEnd => ok(),
            Reply::AuthError { error } => Response::builder()
                .header("type", "AuthError")
                .header("error", error.to_string())
                .status(StatusCode::UNAUTHORIZED)
                .body(empty())
                .unwrap(),
            _ => not_implemented(),
        }
    }
}
