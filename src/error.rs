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

    ScyllaQuery(scylla::transport::errors::QueryError),
    ScyllaRows(scylla::transport::query_result::RowsExpectedError),
    ScyllaFromRow(scylla::cql_to_rust::FromRowError),
    Sqlx(sqlx::Error),
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

impl From<scylla::transport::errors::QueryError> for Error {
    fn from(error: scylla::transport::errors::QueryError) -> Self {
        Self::ScyllaQuery(error)
    }
}

impl From<scylla::transport::query_result::RowsExpectedError> for Error {
    fn from(error: scylla::transport::query_result::RowsExpectedError) -> Self {
        Self::ScyllaRows(error)
    }
}

impl From<scylla::cql_to_rust::FromRowError> for Error {
    fn from(error: scylla::cql_to_rust::FromRowError) -> Self {
        Self::ScyllaFromRow(error)
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
