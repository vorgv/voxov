//! Internal representation of messages.
//! Shoud work with both http and GraphQL APIs.

pub type Hash = [u8; 32]; // BLAKE3

pub mod id;
pub mod query;
pub mod reply;

pub use id::{Id, IDL};
pub use query::Query;
pub use reply::Reply;

use crate::error::Error;
use crate::Result;
use hex::FromHex;
use hyper::{body::Incoming, Request};
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct Costs {
    pub time: u64,
    pub space: u64,
    pub traffic: u64,
    pub tip: u64,
}

#[derive(Debug)]
pub struct Head {
    pub access: Id,
    pub costs: Costs,
    pub fed: Option<Id>,
}

impl Costs {
    pub fn sum(&self) -> u64 {
        self.time + self.space + self.traffic + self.tip
    }
    pub fn try_get(req: &Request<Incoming>) -> Result<Self> {
        Ok(Costs {
            time: try_get::<u64>(req, "time")?,
            space: try_get::<u64>(req, "space")?,
            traffic: try_get::<u64>(req, "traffic")?,
            tip: try_get::<u64>(req, "tip")?,
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
