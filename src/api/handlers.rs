use std::sync::{Mutex, MutexGuard};

use actix_web::{HttpResponse, post, web};

use crate::{
    api::{
        dto::{
            NewClientRequest, NewClientResponse, NewCreditTransactionRequest,
            NewCreditTransactionResponse,
        },
        error::ApiError,
    },
    service::Service,
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
