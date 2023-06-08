use strum_macros::Display;

#[derive(Display, Debug)]
pub enum Error {
    ApiParseId,
    ApiParseNum,
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
    CostTraffic,
    CostTime,

    Fed,

    Gene,
    GeneInvalidId,

    MemeNotFound,
    Redis,
    MongoDB,
    Os,
    Logical,
}

impl std::error::Error for Error {}
