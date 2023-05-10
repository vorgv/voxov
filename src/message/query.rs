use super::{Error, Hash, Head, Id, Raw};
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
    /// Retrive value by key from header map
    pub fn retrieve<'a>(req: &'a Request<Incoming>, key: &'a str) -> Result<&'a str, Error> {
        if let Some(r) = req.headers().get(key) {
            if let Ok(s) = r.to_str() {
                return Ok(s);
            }
        }
        Err(Error::Api)
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
                _ => Err(Error::Api),
            },
            Err(_) => Err(Error::Api),
        }
    }
}
