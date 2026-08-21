use std::sync::{Mutex, MutexGuard};

use actix_web::{HttpResponse, get, post, web};

use crate::{
    api::{
        dto::{
            ClientBalanceRequest, ClientBalanceResponse, NewClientRequest, NewClientResponse,
            NewCreditTransactionRequest, NewCreditTransactionResponse, NewDebitTransactionRequest,
            NewDebitTransactionResponse, StoreBalancesResponse,
        },
        error::ApiError,
    },
    service::Service,
    storage,
};

fn get_lock(lock: &Mutex<Service>) -> Result<MutexGuard<'_, Service>, ApiError> {
    match lock.lock() {
        Ok(guard) => Ok(guard),
        Err(_) => Err(ApiError::Internal),
    }
}

#[post("/new_client")]
async fn new_client(
    info: web::Json<NewClientRequest>,
    service: web::Data<Mutex<Service>>,
) -> Result<HttpResponse, ApiError> {
    let req_data = info.into_inner();

    let new_client_id = {
        let mut serv = get_lock(&service)?;

        serv.create_account(
            req_data.client_name,
            req_data.birth_date,
            req_data.document_number,
            req_data.country,
        )?
    };

    Ok(HttpResponse::Created().json(NewClientResponse {
        client_id: new_client_id,
    }))
}

#[post("/new_credit_transaction")]
async fn new_credit_transaction(
    info: web::Json<NewCreditTransactionRequest>,
    service: web::Data<Mutex<Service>>,
) -> Result<HttpResponse, ApiError> {
    let req_data = info.into_inner();

    let new_client_balance = {
        let mut serv = get_lock(&service)?;

        serv.create_credit_transaction(req_data.client_id, req_data.credit_amount)?
    };

    Ok(HttpResponse::Ok().json(NewCreditTransactionResponse {
        client_balance: new_client_balance,
    }))
}

#[post("/new_debit_transaction")]
async fn new_debit_transaction(
    info: web::Json<NewDebitTransactionRequest>,
    service: web::Data<Mutex<Service>>,
) -> Result<HttpResponse, ApiError> {
    let req_data = info.into_inner();

    let new_client_balance = {
        let mut serv = get_lock(&service)?;

        serv.create_debit_transaction(req_data.client_id, req_data.debit_amount)?
    };

    Ok(HttpResponse::Ok().json(NewDebitTransactionResponse {
        client_balance: new_client_balance,
    }))
}

#[get("/client_balance")]
async fn client_balance(
    info: web::Query<ClientBalanceRequest>,
    service: web::Data<Mutex<Service>>,
) -> Result<HttpResponse, ApiError> {
    let req_data = info.into_inner();

    let client_info = {
        let serv = get_lock(&service)?;

        serv.client_info(req_data.user_id)?
    };

    Ok(HttpResponse::Ok().json(ClientBalanceResponse {
        client_name: client_info.client_name,
        birth_date: client_info.birth_date,
        document_number: client_info.document_number,
        country: client_info.country,
        client_balance: client_info.balance,
    }))
}

#[post("/store_balances")]
async fn store_balances(service: web::Data<Mutex<Service>>) -> Result<HttpResponse, ApiError> {
    let (balances, file_number) = {
        let mut serv = get_lock(&service)?;

        serv.store_balances()
    };

    let generated_file_name =
        storage::save_state(balances, file_number).map_err(|_| ApiError::Internal)?;

    Ok(HttpResponse::Ok().json(StoreBalancesResponse {
        generated_file_name,
    }))
}
