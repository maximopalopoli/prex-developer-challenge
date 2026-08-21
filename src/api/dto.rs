use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// NewClient

#[derive(Deserialize)]
pub(crate) struct NewClientRequest {
    pub(crate) client_name: String,
    pub(crate) birth_date: String,
    pub(crate) document_number: String,
    pub(crate) country: String,
}

#[derive(Serialize)]
pub(crate) struct NewClientResponse {
    pub(crate) client_id: u64,
}

// NewCreditTransaction

#[derive(Deserialize)]
pub(crate) struct NewCreditTransactionRequest {
    pub(crate) client_id: u64,
    pub(crate) credit_amount: Decimal,
}

#[derive(Serialize)]
pub(crate) struct NewCreditTransactionResponse {
    pub(crate) client_balance: Decimal,
}
