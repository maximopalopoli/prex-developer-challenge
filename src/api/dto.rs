use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// NewClient

#[derive(Deserialize)]
pub(crate) struct NewClientRequest {
    pub(crate) client_name: String,
    pub(crate) birth_date: NaiveDate,
    pub(crate) document_number: String,
    pub(crate) country: String,
}

#[derive(Serialize)]
pub(crate) struct NewClientResponse {
    pub(crate) client_id: u64,
}

// NewCreditTransaction & NewDebitTransaction

#[derive(Deserialize)]
pub(crate) struct NewCreditTransactionRequest {
    pub(crate) client_id: u64,
    pub(crate) credit_amount: Decimal,
}

#[derive(Deserialize)]
pub(crate) struct NewDebitTransactionRequest {
    pub(crate) client_id: u64,
    pub(crate) debit_amount: Decimal,
}

#[derive(Serialize)]
pub(crate) struct BalanceResponse {
    pub(crate) client_balance: Decimal,
}

// ClientBalance

#[derive(Deserialize)]
pub(crate) struct ClientBalanceRequest {
    pub(crate) user_id: u64,
}

#[derive(Serialize)]
pub(crate) struct ClientBalanceResponse {
    pub(crate) client_name: String,
    pub(crate) birth_date: NaiveDate,
    pub(crate) document_number: String,
    pub(crate) country: String,
    pub(crate) client_balance: Decimal,
}

// Store balances

#[derive(Serialize)]
pub(crate) struct StoreBalancesResponse {
    pub(crate) generated_file_name: String,
}
