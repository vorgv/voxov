use std::num::TryFromIntError;
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
    AuthInvalidUid,
    AuthTokensMismatch,

    CostInsufficientCredit,
    CostTime,
    CostSpace,
    CostSpaceTooLarge,
    CostTraffic,
    CostTip,
    CostCheckInTooEarly,

    Fed,

    Gene,
    GeneInvalidId,
    GeneMapNotFound,
    GeneMapExpired,

    MemeNotFound,
    MemePut,
    MemeGet,

    Redis(redis::RedisError),
    MongoDB(mongodb::error::Error),
    Bson(bson::ser::Error),
    BsonValueAccess(bson::document::ValueAccessError),
    S3(s3::error::S3Error),

    Rand(rand::Error),
    Hyper(hyper::Error),
    ParseJson(serde_json::error::Error),

    Todo,
    GeoDim,
    Logical,
    Namespace,
    NumCheck,
    ReservedKey,
    TryFromIntError(TryFromIntError),
}

impl std::error::Error for Error {}

impl From<rand::Error> for Error {
    fn from(error: rand::Error) -> Self {
        Self::Rand(error)
    }
}
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

impl From<bson::ser::Error> for Error {
    fn from(error: bson::ser::Error) -> Self {
        Self::Bson(error)
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

impl From<TryFromIntError> for Error {
    fn from(error: TryFromIntError) -> Self {
        Self::TryFromIntError(error)
    }
}
