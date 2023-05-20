pub type Int = i64;
pub type Hash = [u8; 32]; // SHA-256 = SHA-8*32

pub mod id;
pub mod query;
pub mod reply;

pub use id::{Id, IDL};
pub use query::Query;
pub use reply::Reply;

#[derive(Debug)]
pub struct Costs {
    pub time: Int,
    pub space: Int,
    pub tips: Int,
}

#[derive(Debug)]
pub struct Head {
    access: Id,
    costs: Costs,
    fed: Option<Id>,
}

#[derive(Debug)]
pub struct Raw {
    raw: Box<[u8]>,
    time: Int,
}

use strum_macros::Display;

#[derive(Display, Debug)]
pub enum Error {
    Api,
    Auth,
    Cost,
    Fed,
    Gene,
    Meme,
    Redis,
    Os,
    Logical,
    NotFound,
}

impl std::error::Error for Error {}
