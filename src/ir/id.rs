use super::Query;
use crate::{Error, Result};
use core::fmt;
use hex::FromHex;
use hyper::{body::Incoming, Request};
use rand::{rngs::ThreadRng, Fill};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryInto;
use std::str::FromStr;

pub const IDL: usize = 16;
const ID0: [u8; IDL] = [0_u8; IDL];

#[derive(Debug)]
pub struct Id(pub [u8; IDL]);

impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match <[u8; IDL]>::from_hex(s) {
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
        let _: [u8; IDL] = match v.try_into() {
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
        s.try_fill(rng)?;
        Ok(Id(s))
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

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Id, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IdVisitor)
    }
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a fixed size hex string")
    }

    fn visit_str<E>(self, v: &str) -> std::prelude::v1::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Id::from_str(v).map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
    }
}

#[test]
fn test_ser_de() {
    let mut rng = rand::thread_rng();
    let id = Id::rand(&mut rng).unwrap();
    let id_ser = serde_json::to_string(&id).unwrap();
    let id_ser_de: Id = serde_json::from_str(&id_ser).unwrap();
    assert_eq!(id, id_ser_de);
}
