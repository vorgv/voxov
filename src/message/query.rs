use super::{Costs, Hash, Head, Id, Raw};
use crate::error::Error;
use hyper::{body::Incoming, Request};

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
        refresh: Id,
        phone: String,
        message: Id,
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
    /// Get the access token from query
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
    /// Get the cost struct from query
    pub fn get_costs(&self) -> &Costs {
        match self {
            Query::MemeMeta { head, .. } => &head.costs,
            Query::MemeRawPut { head, .. } => &head.costs,
            Query::MemeRawGet { head, .. } => &head.costs,
            Query::GeneMeta { head, .. } => &head.costs,
            Query::GeneCall { head, .. } => &head.costs,
            _ => panic!("Query not passed through Cost: {:?}", self),
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
impl TryFrom<&Request<Incoming>> for Query {
    type Error = Error;
    fn try_from(req: &Request<Incoming>) -> Result<Self, Self::Error> {
        match Query::retrieve(req, "type") {
            Ok(v) => match v {
                "AuthSessionStart" => Ok(Query::AuthSessionStart),
                "AuthSessionRefresh" => Ok(Query::AuthSessionRefresh {
                    refresh: Id::try_get(req, "refresh")?,
                }),
                "AuthSessionEnd" => Ok(Query::AuthSessionEnd {
                    access: Id::try_get(req, "access")?,
                    option_refresh: Id::opt(req, "refresh"),
                }),
                "AuthSmsSendTo" => Ok(Query::AuthSmsSendTo {
                    access: Id::try_get(req, "access")?,
                }),
                "AuthSmsSent" => Ok(Query::AuthSmsSent {
                    access: Id::try_get(req, "access")?,
                    refresh: Id::try_get(req, "refresh")?,
                    phone: Query::retrieve(req, "phone")?.to_string(),
                    message: Id::try_get(req, "message")?,
                }),
                _ => Err(Error::ApiUnknownQueryType),
            },
            Err(_) => Err(Error::ApiMissingQueryType),
        }
    }
}
