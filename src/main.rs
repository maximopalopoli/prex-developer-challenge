use std::sync::Mutex;

use actix_web::{App, HttpServer, web};

use prex_challenge::api::handlers;
use prex_challenge::service::Service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(Mutex::new(Service::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(handlers::new_client)
            .service(handlers::new_credit_transaction)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
