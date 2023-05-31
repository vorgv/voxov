pub type Int = i64;
pub type Uint = u64;
pub type Hash = [u8; 32]; // SHA-256 = SHA-8*32

pub mod id;
pub mod query;
pub mod reply;

pub use id::{Id, IDL};
pub use query::Query;
pub use reply::Reply;

use crate::error::Error;
use hex::FromHex;
use hyper::{body::Incoming, Request};

#[derive(Debug)]
pub struct Costs {
    pub time: Uint,
    pub space: Uint,
    pub traffic: Uint,
    pub tips: Uint,
}

#[derive(Debug)]
pub struct Head {
    access: Id,
    costs: Costs,
    fed: Option<Id>,
}

#[derive(Debug)]
pub struct Raw {
    raw: Box<[u8]>, //TODO: use stream
    time: Int,
}

impl Costs {
    pub fn sum(&self) -> Uint {
        self.time + self.space + self.traffic + self.tips
    }
    pub fn try_get(req: &Request<Incoming>) -> Result<Self, Error> {
        Ok(Costs {
            time: try_get_uint(req, "time")?,
            space: try_get_uint(req, "space")?,
            traffic: try_get_uint(req, "traffic")?,
            tips: try_get_uint(req, "tips")?,
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

pub fn try_get_uint(req: &Request<Incoming>, key: &str) -> Result<Uint, Error> {
    let s = Query::retrieve(req, key)?;
    match s.parse::<Uint>() {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::ApiParseUint),
    }
}

fn try_get_hash(req: &Request<Incoming>) -> Result<Hash, Error> {
    let s = Query::retrieve(req, "key")?;
    match <[u8; 32]>::from_hex(s) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::ApiParseHash),
    }
}
