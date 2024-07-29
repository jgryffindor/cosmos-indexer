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
use actix_web::{get, web, App, HttpServer, Responder,};
use actix_web::web::Path;

use env_logger::Env;
use rocksdb::Options;
use rocksdb::DB;

use std::sync::Arc;
use transactions::database::transaction_info_thread;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "http://66.172.36.142:2119")]
    chain_node_grpc: String,

    #[clap(long, default_value = "manifest")]
    chain_prefix: String,

    #[clap(long)]
    test_mode: bool,

    #[clap(long, default_value = "100000")]
    test_block_limit: u64,
}

#[get("/transactions/send")]
async fn get_all_msg_send_transactions(db: web::Data<Arc<DB>>) -> impl Responder {
    transactions::endpoints::get_all_msg_send_transactions(db).await
}

#[get("/transactions/ibc_transfer")]
async fn get_all_msg_ibc_transfer_transactions(db: web::Data<Arc<DB>>) -> impl Responder {
    transactions::endpoints::get_all_msg_ibc_transfer_transactions(db).await
}

#[get("/transactions/send/{address}")]
async fn get_msg_send_transactions_by_address(
    db: web::Data<Arc<DB>>,
    address: Path<String>,
) -> impl Responder {
    transactions::endpoints::get_msg_send_transactions_by_address(db, address.into_inner()).await
}

#[get("/transactions/send/{address}/{direction}")]
async fn get_msg_send_transactions_by_address_and_direction(
    db: web::Data<Arc<DB>>,
    path: Path<(String, String)>,
) -> impl Responder {
    let (address, direction) = path.into_inner();
    transactions::endpoints::get_msg_send_transactions_by_address_and_direction(db, address, direction).await
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    openssl_probe::init_ssl_cert_env_vars();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut db_options = Options::default();
    db_options.create_if_missing(true);
    let db = Arc::new(DB::open(&db_options, "transactions").expect("Failed to open database"));
    let api_db = web::Data::new(db.clone());

    // Pass the arguments to the transaction_info_thread
    transaction_info_thread(db.clone(), args.chain_node_grpc, args.chain_prefix, args.test_mode, args.test_block_limit);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
            .app_data(api_db.clone())
            .service(get_all_msg_send_transactions)
            .service(get_all_msg_ibc_transfer_transactions)
    });

    let server = server.bind(format!("{}:{}", DOMAIN, PORT))?;
    server.run().await?;
    Ok(())
}