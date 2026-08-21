#[derive(Debug, PartialEq)]
pub(crate) enum ServiceError {
    ClientDoesNotExist,
    NonPositiveAmount,
    DuplicateDocument,
    EmptyField(&'static str),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientDoesNotExist => write!(f, "client not found"),
            Self::NonPositiveAmount => write!(f, "amount must be greater than zero"),
            Self::DuplicateDocument => write!(f, "document number already registered"),
            Self::EmptyField(field) => write!(f, "the {field} field cannot be empty"),
        }
    }
}
