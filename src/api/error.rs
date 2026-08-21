use std::fmt::Display;

use actix_web::{ResponseError, http::StatusCode};

use crate::error::ServiceError::{self, ClientDoesNotExist, DuplicateDocument, NonPositiveAmount};

#[derive(Debug)]
pub enum ApiError {
    Domain(ServiceError),
    Internal,
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        ApiError::Domain(error)
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => write!(f, "{error}"),
            Self::Internal => write!(f, "internal server error"),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Domain(ClientDoesNotExist) => StatusCode::NOT_FOUND,
            Self::Domain(DuplicateDocument) => StatusCode::CONFLICT,
            Self::Domain(NonPositiveAmount) => StatusCode::BAD_REQUEST,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
