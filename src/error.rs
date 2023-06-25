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
    CostTime,
    CostSpace,
    CostTraffic,
    CostTips,

    Fed,

    Gene,
    GeneInvalidId,

    MemeNotFound,
    MemeRawPut,
    MemeRawGet,
    Redis,
    MongoDB,
    S3,
    Os,
    Logical,
}

impl std::error::Error for Error {}
