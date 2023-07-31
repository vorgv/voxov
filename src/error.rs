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
    CostSpaceTooLarge,
    CostTraffic,
    CostTips,

    Fed,

    Gene,
    GeneInvalidId,

    MemeNotFound,
    MemePut,
    MemeGet,
    Redis(redis::RedisError),
    MongoDB(mongodb::error::Error),
    S3(s3::error::S3Error),
    Os,
    Hyper(hyper::Error),
    Logical,
    BsonValueAccess(bson::document::ValueAccessError),
    ParseJson(serde_json::error::Error),
    Namespace,
    GeoDim,
    ReservedKey,
}

impl std::error::Error for Error {}

impl From<redis::RedisError> for Error {
    fn from(error: redis::RedisError) -> Self {
        Self::Redis(error)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(error: mongodb::error::Error) -> Self {
        Self::MongoDB(error)
    }
}

impl From<s3::error::S3Error> for Error {
    fn from(error: s3::error::S3Error) -> Self {
        Self::S3(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Self::Hyper(error)
    }
}

impl From<bson::document::ValueAccessError> for Error {
    fn from(error: bson::document::ValueAccessError) -> Self {
        Self::BsonValueAccess(error)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Self::ParseJson(error)
    }
}
