use std::pin::Pin;

use super::{try_get, try_get_hash, Costs, Hash, Head, Id, Uint};
use crate::error::Error;
use hyper::{body::Incoming, Request};

type OptionId = Option<Id>;

pub type QueryBody = Pin<Box<Incoming>>;

#[derive(Debug)]
pub enum Query {
    AuthSessionStart,
    AuthSessionRefresh {
        refresh: Id,
    },
    AuthSessionEnd {
        access: Id,
        option_refresh: OptionId,
    },
    AuthSmsSendTo {
        access: Id,
    },
    AuthSmsSent {
        access: Id,
        refresh: Id,
        phone: String,
        message: Id,
    },
    CostPay {
        access: Id,
        vendor: Id,
    },
    CostGet {
        access: Id,
    },
    GeneMeta {
        head: Head,
        id: usize,
    },
    GeneCall {
        head: Head,
        id: usize,
        arg: String,
    },
    MemeMeta {
        head: Head,
        hash: Hash,
    },
    MemeRawPut {
        head: Head,
        days: Uint,
        raw: QueryBody,
    },
    MemeRawGet {
        head: Head,
        hash: Hash,
        public: bool,
    },
    //TODO: FedMemeClone, FedMemeVisa, FedCreditClaim
}

impl Query {
    /// Get the access token from query
    pub fn get_access(&self) -> &Id {
        match self {
            Query::CostPay { access, .. } => access,
            Query::CostGet { access, .. } => access,
            Query::MemeMeta { head, .. } => &head.access,
            Query::MemeRawPut { head, .. } => &head.access,
            Query::MemeRawGet { head, .. } => &head.access,
            Query::GeneMeta { head, .. } => &head.access,
            Query::GeneCall { head, .. } => &head.access,
            _ => panic!("Query not passed through Auth: {:?}", self),
        }
    }
    /// Get the cost struct from query
    pub fn get_costs(&self) -> Costs {
        match self {
            Query::MemeMeta { head, .. } => head.costs,
            Query::MemeRawPut { head, .. } => head.costs,
            Query::MemeRawGet { head, .. } => head.costs,
            Query::GeneMeta { head, .. } => head.costs,
            Query::GeneCall { head, .. } => head.costs,
            _ => panic!("Query not passed through Cost: {:?}", self),
        }
    }
    /// Get the fed id from query
    pub fn get_fed(&self) -> &Option<Id> {
        match self {
            Query::MemeMeta { head, .. } => &head.fed,
            Query::GeneMeta { head, .. } => &head.fed,
            Query::GeneCall { head, .. } => &head.fed,
            _ => &None,
        }
    }
    /// Retrive value by key from header map
    pub fn retrieve<'a>(req: &'a Request<Incoming>, key: &'a str) -> Result<&'a str, Error> {
        if let Some(r) = req.headers().get(key) {
            if let Ok(s) = r.to_str() {
                return Ok(s);
            }
        }
        Err(Error::ApiMissingEntry)
    }
}

/// Try from http request to rust struct
impl TryFrom<Request<Incoming>> for Query {
    type Error = Error;
    fn try_from(req: Request<Incoming>) -> Result<Self, Self::Error> {
        match Query::retrieve(&req, "type") {
            Ok(v) => match v {
                "AuthSessionStart" => Ok(Query::AuthSessionStart),
                "AuthSessionRefresh" => Ok(Query::AuthSessionRefresh {
                    refresh: Id::try_get(&req, "refresh")?,
                }),
                "AuthSessionEnd" => Ok(Query::AuthSessionEnd {
                    access: Id::try_get(&req, "access")?,
                    option_refresh: Id::opt(&req, "refresh"),
                }),
                "AuthSmsSendTo" => Ok(Query::AuthSmsSendTo {
                    access: Id::try_get(&req, "access")?,
                }),
                "AuthSmsSent" => Ok(Query::AuthSmsSent {
                    access: Id::try_get(&req, "access")?,
                    refresh: Id::try_get(&req, "refresh")?,
                    phone: Query::retrieve(&req, "phone")?.to_string(),
                    message: Id::try_get(&req, "message")?,
                }),
                "CostPay" => Ok(Query::CostPay {
                    access: Id::try_get(&req, "access")?,
                    vendor: Id::try_get(&req, "vendor")?,
                }),
                "CostGet" => Ok(Query::CostGet {
                    access: Id::try_get(&req, "access")?,
                }),
                "GeneMeta" => Ok(Query::GeneMeta {
                    head: Head::try_get(&req)?,
                    id: try_get::<usize>(&req, "id")?,
                }),
                "GeneCall" => Ok(Query::GeneCall {
                    head: Head::try_get(&req)?,
                    id: try_get::<usize>(&req, "id")?,
                    arg: Query::retrieve(&req, "arg")?.to_string(),
                }),
                "MemeMeta" => Ok(Query::MemeMeta {
                    head: Head::try_get(&req)?,
                    hash: try_get_hash(&req)?,
                }),
                "MemeRawPut" => Ok(Query::MemeRawPut {
                    head: Head::try_get(&req)?,
                    days: try_get::<Uint>(&req, "days")?,
                    raw: Box::pin(req.into_body()),
                }),
                "MemeRawGet" => Ok(Query::MemeRawGet {
                    head: Head::try_get(&req)?,
                    hash: try_get_hash(&req)?,
                    public: try_get::<bool>(&req, "public")?,
                }),
                _ => Err(Error::ApiUnknownQueryType),
            },
            Err(_) => Err(Error::ApiMissingQueryType),
        }
    }
}
