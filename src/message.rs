use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::Response;
use hyper::{body::Incoming, Request};
use std::convert::Infallible;
use std::str::FromStr;
use std::u128;

pub struct Id(u128);
pub type Int = i64;
pub type Hash = [u8; 32]; // SHA-256 = SHA-8*32

pub struct Cost {
    time: Int,
    space: Int,
    tips: Int,
}
pub struct Head {
    access: Id,
    cost: Cost,
    fed: Option<Id>,
}
pub struct Raw {
    raw: Box<[u8]>,
    time: Int,
}

pub enum Query {
    AuthSessionStart,
    AuthSessionRefresh { refresh: Id },
    AuthSessionEnd { access: Id, refresh: Option<Id> },
    AuthSmsSendTo { access: Id },
    AuthSmsSent { access: Id },
    Pay { access: Id, vendor: Id },
    MemeMeta { head: Head, key: Hash },
    MemeRawPut { head: Head, key: Hash, raw: Raw },
    MemeRawGet { head: Head, key: Hash },
    GeneMeta { head: Head, id: Id },
    GeneCall { head: Head, id: Id, arg: Box<[u8]> },
}

pub enum Reply {
    Unimplemented,
    AuthSessionStart {
        access: Id,
        refresh: Id,
    },
    AuthSessionRefresh {
        access: Id,
    },
    AuthSessionEnd {
        result: Result<(), Error>,
    },
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

#[derive(Debug)]
pub enum Error {
    Api,
    Auth,
    Cost,
    Fed,
    Gene,
    Meme,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
impl std::error::Error for Error {}

/// Try from http request to rust struct
impl TryFrom<&Request<Incoming>> for Query {
    type Error = Error;
    fn try_from(req: &Request<Incoming>) -> Result<Self, Self::Error> {
        match retrieve(req, "type") {
            Ok(v) => match v {
                "AuthSessionStart" => Ok(Query::AuthSessionStart),
                "AuthSessionRefresh" => Ok(Query::AuthSessionRefresh {
                    refresh: Id::try_get(req, "refresh")?,
                }),
                "AuthSessionEnd" => Ok(Query::AuthSessionEnd {
                    access: Id::try_get(req, "access")?,
                    refresh: Id::opt(req, "refresh"),
                }),
                _ => Err(Error::Api),
            },
            Err(_) => Err(Error::Api),
        }
    }
}

impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match u128::from_str_radix(s, 16) {
            Ok(u) => Ok(Id(u)),
            Err(_) => Err(Error::Api),
        }
    }
}

impl Id {
    fn try_get(req: &Request<Incoming>, key: &str) -> Result<Self, Error> {
        Id::from_str(retrieve(req, key)?)
    }
    fn opt(req: &Request<Incoming>, key: &str) -> Option<Self> {
        match Id::try_get(req, key) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

/// Retrive value by key from header map
fn retrieve<'a>(req: &'a Request<Incoming>, key: &'a str) -> Result<&'a str, Error> {
    if let Some(r) = req.headers().get(key) {
        if let Ok(s) = r.to_str() {
            return Ok(s);
        }
    }
    Err(Error::Api)
}

impl Reply {
    pub fn to_response(&self) -> Response<BoxBody<Bytes, Infallible>> {
        use crate::api::full;
        match self {
            Reply::Unimplemented => Response::new(full("Unimplemented")),
            _ => Response::new(full("to_response unimplemented")),
        }
    }
}
