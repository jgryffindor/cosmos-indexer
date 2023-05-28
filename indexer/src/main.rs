extern crate lazy_static;

pub mod transactions;
pub mod types;

const DEVELOPMENT: bool = cfg!(feature = "development");

const DOMAIN: &str = if cfg!(test) || DEVELOPMENT {
    "localhost"
} else {
    "localhost"
};
const PORT: u16 = 9000;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};

use env_logger::Env;
use rocksdb::Options;
use rocksdb::DB;

use std::sync::Arc;
use transactions::database::transaction_info_thread;

#[get("/transactions/send_to_eth")]
async fn get_all_msg_send_to_eth_transactions(db: web::Data<Arc<DB>>) -> impl Responder {
    transactions::endpoints::get_all_msg_send_to_eth_transactions(db).await
}

#[get("/transactions/ibc_transfer")]
async fn get_all_msg_ibc_transfer_transactions(db: web::Data<Arc<DB>>) -> impl Responder {
    transactions::endpoints::get_all_msg_ibc_transfer_transactions(db).await
}

#[get("/transactions/send_to_eth/time")]
async fn get_send_to_eth_transaction_totals(db: web::Data<Arc<DB>>) -> impl Responder {
    transactions::endpoints::get_send_to_eth_transaction_totals(db).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // starts a background thread for downloading transactions
    let mut db_options = Options::default();
    db_options.create_if_missing(true);
    let db = Arc::new(DB::open(&db_options, "transactions").expect("Failed to open database"));
    let api_db = web::Data::new(db.clone());
    transaction_info_thread(db.clone());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
            .app_data(api_db.clone())
            .service(get_all_msg_send_to_eth_transactions)
            .service(get_all_msg_ibc_transfer_transactions)
            .service(get_send_to_eth_transaction_totals)
    });

    let server = server.bind(format!("{}:{}", DOMAIN, PORT))?;

    server.run().await?;

    Ok(())
}
