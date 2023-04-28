use std::convert::From;
use hyper::{Request, body::Incoming};

type Id = u128;
type Int = i64;
type Hash = [u8; 32];   // SHA-256 = SHA-8*32

struct Cost { time: Int, space: Int, tips: Int }
struct Head { access: Id, cost: Cost, fed: Option<Id>}

enum Query{
    AuthSessionStart,
    AuthSessionRefresh { refresh: Id },
    AuthSessionEnd { access: Id },
    AuthSmsSendTo { access: Id },
    AuthSmsSent { access: Id },
    Pay { access: Id, vendor: Id },
    MemeMeta { head: Head, key: Hash },
    MemeRaw { head: Head, key: Hash },
}

enum Reply{
    AuthSessionStart { access: Id, refresh: Id},
    AuthSessionRefresh { access: Id },
    AuthSessionEnd { access: Option<Id>, refresh: Option<Id> },
    AuthSmsSendTo { phone: String, message: String },
    AuthSmsSent { pid: Id },
    Pay { uri: String },
    MemeMeta { meta: String },
    MemeRaw {},
}

impl From<Request<Incoming>> for Query {
    fn from(req: Request<Incoming>) -> Self {
        //TODO
        Query::AuthSessionStart
    }
}
