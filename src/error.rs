use std::num::TryFromIntError;
use strum_macros::Display;

#[derive(Display, Debug)]
pub enum Error {
    ApiParseId,
    ApiParseNum,
    ApiParseHash,
    ApiMethod,
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

    ScyllaQuery(Box<scylla::errors::ExecutionError>),
    ScyllaRows(Box<scylla::response::query_result::IntoRowsResultError>),
    ScyllaRowsDeser(scylla::response::query_result::RowsError),
    ScyllaDeserialize(scylla::errors::DeserializationError),
    Sqlx(sqlx::Error),
    S3(s3::error::S3Error),

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

impl From<scylla::errors::ExecutionError> for Error {
    fn from(error: scylla::errors::ExecutionError) -> Self {
        Self::ScyllaQuery(Box::new(error))
    }
}

impl From<scylla::response::query_result::IntoRowsResultError> for Error {
    fn from(error: scylla::response::query_result::IntoRowsResultError) -> Self {
        Self::ScyllaRows(Box::new(error))
    }
}

impl From<scylla::response::query_result::RowsError> for Error {
    fn from(error: scylla::response::query_result::RowsError) -> Self {
        Self::ScyllaRowsDeser(error)
    }
}

impl From<scylla::errors::DeserializationError> for Error {
    fn from(error: scylla::errors::DeserializationError) -> Self {
        Self::ScyllaDeserialize(error)
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
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
