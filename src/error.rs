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
