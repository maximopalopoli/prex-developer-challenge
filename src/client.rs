use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Client {
    pub(crate) balance: Decimal,
    pub(crate) client_name: String,
    pub(crate) birth_date: NaiveDate,
    pub(crate) document_number: String,
    pub(crate) country: String,
}
