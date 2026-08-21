use std::sync::Mutex;

use actix_web::{App, HttpServer, web};

use prex_challenge::api::handlers;
use prex_challenge::service::Service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let state = web::Data::new(Mutex::new(Service::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(handlers::new_client)
            .service(handlers::new_credit_transaction)
            .service(handlers::new_debit_transaction)
            .service(handlers::client_balance)
            .service(handlers::store_balances)
    })
    .bind((host, port.parse().expect("PORT must be a number")))?
    .run()
    .await
}
