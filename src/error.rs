#[derive(Debug, PartialEq)]
pub(crate) enum ServiceError {
    ClientDoesNotExist,
    NonPositiveAmount,
    DuplicateDocument,
    EmptyField(&'static str),
    FutureBirthDate,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientDoesNotExist => write!(f, "client not found"),
            Self::NonPositiveAmount => write!(f, "amount must be greater than zero"),
            Self::DuplicateDocument => write!(f, "document number already registered"),
            Self::EmptyField(field) => write!(f, "the {field} field cannot be empty"),
            Self::FutureBirthDate => write!(f, "the birth date cannot be in the future"),
        }
    }
}
