//! Internal representation of messages.
//! Shoud work with both http and GraphQL APIs.

pub type Int = i64;
pub type Uint = u64;
pub type Hash = [u8; 32]; // BLAKE3

pub mod id;
pub mod query;
pub mod reply;

use std::str::FromStr;

pub use id::{Id, IDL};
pub use query::Query;
pub use reply::Reply;

use crate::error::Error;
use hex::FromHex;
use hyper::{body::Incoming, Request};

#[derive(Debug, Copy, Clone)]
pub struct Costs {
    pub time: Uint,
    pub space: Uint,
    pub traffic: Uint,
    pub tip: Uint,
}

#[derive(Debug)]
pub struct Head {
    pub access: Id,
    pub costs: Costs,
    pub fed: Option<Id>,
}

impl Costs {
    pub fn sum(&self) -> Uint {
        self.time + self.space + self.traffic + self.tip
    }
    pub fn try_get(req: &Request<Incoming>) -> Result<Self, Error> {
        Ok(Costs {
            time: try_get::<Uint>(req, "time")?,
            space: try_get::<Uint>(req, "space")?,
            traffic: try_get::<Uint>(req, "traffic")?,
            tip: try_get::<Uint>(req, "tip")?,
        })
    }
}

impl Head {
    pub fn try_get(req: &Request<Incoming>) -> Result<Self, Error> {
        Ok(Head {
            access: Id::try_get(req, "access")?,
            costs: Costs::try_get(req)?,
            fed: Id::opt(req, "fed"),
        })
    }
}

pub fn try_get<T: FromStr>(req: &Request<Incoming>, key: &str) -> Result<T, Error> {
    let s = Query::retrieve(req, key)?;
    match s.parse::<T>() {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::ApiParseNum),
    }
}

fn try_get_hash(req: &Request<Incoming>) -> Result<Hash, Error> {
    let s = Query::retrieve(req, "hash")?;
    match <[u8; 32]>::from_hex(s) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::ApiParseHash),
    }
}
