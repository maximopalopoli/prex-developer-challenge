use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError, http::StatusCode};

use crate::error::ServiceError::{self, ClientDoesNotExist, DuplicateDocument, NonPositiveAmount};

#[derive(Debug)]
pub enum ApiError {
    Domain(ServiceError),
    LockPoisoned,
    FileWrite(std::io::Error),
    BlockingTask,
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
            Self::LockPoisoned => write!(f, "internal server error"),
            Self::FileWrite(_) => write!(f, "internal server error"),
            Self::BlockingTask => write!(f, "internal server error"),
        }
    }
}

impl ResponseError for ApiError {
    /// Internal failures are loggged with their cause and returned with the generic message of `Display`.
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Domain(_) => {}
            Self::LockPoisoned => {
                log::error!("the service lock was poisoned by a panicking request")
            }
            Self::FileWrite(error) => log::error!("could not write the balances file: {error}"),
            Self::BlockingTask => {
                log::error!("the task that writes the balances file did not finish")
            }
        }

        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Domain(ClientDoesNotExist) => StatusCode::NOT_FOUND,
            Self::Domain(DuplicateDocument) => StatusCode::CONFLICT,
            Self::Domain(NonPositiveAmount) => StatusCode::BAD_REQUEST,
            Self::LockPoisoned => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FileWrite(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BlockingTask => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
