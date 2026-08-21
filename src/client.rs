use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Client {
    pub(crate) balance: Decimal,
    pub(crate) client_name: String,
    pub(crate) birth_date: String, // Change to NaiveDate
    pub(crate) document_number: String,
    pub(crate) country: String,
}
