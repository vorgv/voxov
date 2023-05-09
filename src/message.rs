use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{body::Incoming, Request};
use hyper::{Response, StatusCode};
use std::convert::Infallible;
use std::str::FromStr;

pub type Int = i64;
pub type Hash = [u8; 32]; // SHA-256 = SHA-8*32

pub const IDL: usize = 16;

#[derive(Debug)]
pub struct Id(pub [u8; IDL]);

#[derive(Debug)]
pub struct Cost {
    time: Int,
    space: Int,
    tips: Int,
}
#[derive(Debug)]
pub struct Head {
    access: Id,
    cost: Cost,
    fed: Option<Id>,
}
#[derive(Debug)]
pub struct Raw {
    raw: Box<[u8]>,
    time: Int,
}

#[derive(Debug)]
pub enum Query {
    AuthSessionStart,
    AuthSessionRefresh {
        refresh: Id,
    },
    AuthSessionEnd {
        access: Id,
        option_refresh: Option<Id>,
    },
    AuthSmsSendTo {
        access: Id,
    },
    AuthSmsSent {
        access: Id,
    },
    Pay {
        access: Id,
        vendor: Id,
    },
    MemeMeta {
        head: Head,
        key: Hash,
    },
    MemeRawPut {
        head: Head,
        key: Hash,
        raw: Raw,
    },
    MemeRawGet {
        head: Head,
        key: Hash,
    },
    GeneMeta {
        head: Head,
        id: Id,
    },
    GeneCall {
        head: Head,
        id: Id,
        arg: Box<[u8]>,
    },
}

impl Query {
    pub fn get_access(&self) -> &Id {
        match self {
            Query::Pay { access, .. } => access,
            Query::MemeMeta { head, .. } => &head.access,
            Query::MemeRawPut { head, .. } => &head.access,
            Query::MemeRawGet { head, .. } => &head.access,
            Query::GeneMeta { head, .. } => &head.access,
            Query::GeneCall { head, .. } => &head.access,
            _ => panic!("Query not passed through Auth: {:?}", self),
        }
    }
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

#[derive(Debug)]
pub enum Error {
    Api,
    Auth,
    Cost,
    Fed,
    Gene,
    Meme,
    Redis,
    Os,
    Logical,
    NotFound,
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
                    option_refresh: Id::opt(req, "refresh"),
                }),
                _ => Err(Error::Api),
            },
            Err(_) => Err(Error::Api),
        }
    }
}

use hex::FromHex;

impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match <[u8; 16]>::from_hex(s) {
            Ok(u) => Ok(Id(u)),
            Err(_) => Err(Error::Api),
        }
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}

use std::convert::TryInto;

impl TryFrom<Vec<u8>> for Id {
    type Error = Error;
    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        let _: [u8; 16] = match v.try_into() {
            Ok(a) => return Ok(Id(a)),
            Err(_) => return Err(Error::Logical),
        };
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

use rand::{rngs::ThreadRng, Fill};

const ID0: [u8; 16] = [0_u8; 16];
impl Id {
    pub fn zero() -> Self {
        Id(ID0)
    }
    pub fn is_zero(&self) -> bool {
        self.0 == ID0
    }
    pub fn rand(rng: &mut ThreadRng) -> Result<Self, Error> {
        let mut s = ID0;
        match s.try_fill(rng) {
            Ok(_) => Ok(Id(s)),
            Err(_) => Err(Error::Os),
        }
    }
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

use crate::api::{empty, not_implemented, ok};

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
