use strum_macros::Display;

#[derive(Display, Debug)]
pub enum Error {
    ApiParseId,
    ApiParseUint,
    ApiParseHash,
    ApiMissingEntry,
    ApiUnknownQueryType,
    ApiMissingQueryType,

    AuthInvalidAccessToken,
    AuthInvalidRefreshToken,
    AuthNotAuthenticated,
    AuthInvalidPhone,
    AuthTokensMismatch,

    CostInsufficientCredit,
    CostTimeout,

    Fed,
    Gene,
    Meme,
    Redis,
    Os,
    Logical,
}

impl std::error::Error for Error {}
