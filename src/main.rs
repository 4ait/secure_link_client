use std::net::SocketAddr;
use dotenv::dotenv;
use log::info;

#[tokio::main]
async fn main() {

    dotenv().ok();
    env_logger::init();

}