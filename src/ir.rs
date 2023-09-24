//! Internal representation of messages.
//! Shoud work with both http and GraphQL APIs.

pub type Hash = [u8; 32]; // BLAKE3

pub mod id;
pub mod query;
pub mod reply;

pub use id::{Id, IDL};
pub use query::Query;
pub use reply::Reply;

use crate::{Error, Result};
use hex::FromHex;
use hyper::{body::Incoming, Request};
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct Costs {
    pub time: i64,
    pub space: i64,
    pub traffic: i64,
    pub tip: i64,
}

#[derive(Debug)]
pub struct Head {
    pub access: Id,
    pub costs: Costs,
    pub fed: Option<Id>,
}

impl Costs {
    pub fn sum(&self) -> i64 {
        self.time + self.space + self.traffic + self.tip
    }
    pub fn try_get(req: &Request<Incoming>) -> Result<Self> {
        Ok(Costs {
            time: try_get::<i64>(req, "time")?,
            space: try_get::<i64>(req, "space")?,
            traffic: try_get::<i64>(req, "traffic")?,
            tip: try_get::<i64>(req, "tip")?,
        })
    }
}

impl Head {
    pub fn try_get(req: &Request<Incoming>) -> Result<Self> {
        Ok(Head {
            access: Id::try_get(req, "access")?,
            costs: Costs::try_get(req)?,
            fed: Id::opt(req, "fed"),
        })
    }
}

pub fn try_get<T: FromStr>(req: &Request<Incoming>, key: &str) -> Result<T> {
    let s = Query::retrieve(req, key)?;
    match s.parse::<T>() {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::ApiParseNum),
    }
}

fn try_get_hash(req: &Request<Incoming>) -> Result<Hash> {
    let s = Query::retrieve(req, "hash")?;
    match <[u8; 32]>::from_hex(s) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::ApiParseHash),
    }
}
