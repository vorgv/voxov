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
    MemePut,
    MemeGet,
    Redis,
    MongoDB,
    S3(s3::error::S3Error),
    Os,
    Logical,
    BsonValueAccess(bson::document::ValueAccessError),
}

impl std::error::Error for Error {}
