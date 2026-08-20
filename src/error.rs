#[derive(Debug, PartialEq)]
pub(crate) enum ServiceError {
    ClientDoesNotExist,
    NonPositiveAmount,
    DuplicateDocument,
}
