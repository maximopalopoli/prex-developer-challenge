use std::sync::{Mutex, MutexGuard};

use rust_decimal::Decimal;

use actix_web::{HttpResponse, get, post, web};

use crate::{
    api::{
        dto::{
            BalanceResponse, ClientBalanceRequest, ClientBalanceResponse, NewClientRequest,
            NewClientResponse, NewCreditTransactionRequest, NewDebitTransactionRequest,
            StoreBalancesResponse,
        },
        error::ApiError,
    },
    service::Service,
    storage,
};

fn lock_service(lock: &Mutex<Service>) -> Result<MutexGuard<'_, Service>, ApiError> {
    match lock.lock() {
        Ok(guard) => Ok(guard),
        Err(_) => Err(ApiError::LockPoisoned),
    }
}

#[post("/new_client")]
async fn new_client(
    info: web::Json<NewClientRequest>,
    service: web::Data<Mutex<Service>>,
) -> Result<HttpResponse, ApiError> {
    let req_data = info.into_inner();

    let new_client_id = {
        let mut serv = lock_service(&service)?;

        serv.create_client(
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
        let mut serv = lock_service(&service)?;

        serv.create_credit_transaction(req_data.client_id, req_data.credit_amount)?
    };

    Ok(HttpResponse::Ok().json(BalanceResponse {
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
        let mut serv = lock_service(&service)?;

        serv.create_debit_transaction(req_data.client_id, req_data.debit_amount)?
    };

    Ok(HttpResponse::Ok().json(BalanceResponse {
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
        let serv = lock_service(&service)?;

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
        let mut serv = lock_service(&service)?;

        serv.store_balances()
    };

    // The cut already zeroed the balances in memory, so a write that fails would make that money disappear.
    // The snapshot is kept to put it back.
    let snapshot = balances.clone();

    let written = web::block(move || storage::save_balances(balances, file_number))
        .await
        .map_err(|_| ApiError::BlockingTask)
        .and_then(|result| result.map_err(ApiError::FileWrite));

    let generated_file_name = match written {
        Ok(file_name) => file_name,
        Err(error) => {
            restore_after_a_failed_cut(&service, snapshot);
            return Err(error);
        }
    };

    Ok(HttpResponse::Ok().json(StoreBalancesResponse {
        generated_file_name,
    }))
}

/// Adds a cut back to the balances after the file could not be written. If the
/// lock cannot be taken there is nothing left to do but write the amounts to
/// the log, so that the cut can be rebuilt by hand.
fn restore_after_a_failed_cut(service: &Mutex<Service>, snapshot: Vec<(u64, Decimal)>) {
    match service.lock() {
        Ok(mut serv) => {
            log::error!(
                "the cut could not be written, restoring {} balances",
                snapshot.len()
            );
            serv.restore_balances(snapshot);
        }
        Err(_) => {
            log::error!("the cut could not be written or restored, balances were {snapshot:?}")
        }
    }
}
