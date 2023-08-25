use super::Query;
use crate::error::Error;
use crate::Result;
use core::fmt;
use hex::FromHex;
use hyper::{body::Incoming, Request};
use rand::{rngs::ThreadRng, Fill};
use std::convert::TryInto;
use std::str::FromStr;

pub const IDL: usize = 16;
const ID0: [u8; IDL] = [0_u8; IDL];

#[derive(Debug)]
pub struct Id(pub [u8; IDL]);

impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match <[u8; 16]>::from_hex(s) {
            Ok(u) => Ok(Id(u)),
            Err(_) => Err(Error::ApiParseId),
        }
    }
}

impl TryFrom<&String> for Id {
    type Error = Error;
    fn try_from(s: &String) -> Result<Self> {
        Id::from_str(s)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl TryFrom<Vec<u8>> for Id {
    type Error = Error;
    fn try_from(v: Vec<u8>) -> Result<Self> {
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

impl Id {
    pub fn zero() -> Self {
        Id(ID0)
    }
    pub fn is_zero(&self) -> bool {
        self.0 == ID0
    }
    pub fn rand(rng: &mut ThreadRng) -> Result<Self> {
        let mut s = ID0;
        match s.try_fill(rng) {
            Ok(_) => Ok(Id(s)),
            Err(_) => Err(Error::Os),
        }
    }
    pub fn try_get(req: &Request<Incoming>, key: &str) -> Result<Self> {
        Id::from_str(Query::retrieve(req, key)?)
    }
    pub fn opt(req: &Request<Incoming>, key: &str) -> Option<Self> {
        match Id::try_get(req, key) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}
