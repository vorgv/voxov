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
    Redis,
    MongoDB(mongodb::error::Error),
    S3(s3::error::S3Error),
    Os,
    Hyper(hyper::Error),
    Logical,
    BsonValueAccess(bson::document::ValueAccessError),
}

impl std::error::Error for Error {}
