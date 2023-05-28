pub type Int = i64;
pub type Uint = u64;
pub type Hash = [u8; 32]; // SHA-256 = SHA-8*32

pub mod id;
pub mod query;
pub mod reply;

pub use id::{Id, IDL};
pub use query::Query;
pub use reply::Reply;

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
    raw: Box<[u8]>,
    time: Int,
}

impl Costs {
    pub fn sum(&self) -> Uint {
        self.time + self.space + self.traffic + self.tips
    }
}
