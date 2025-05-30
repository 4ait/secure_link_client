mod pdu;
mod global_channel;
mod proxy_channel;
mod tls_connect;

#[cfg(feature = "load_dev_certs")]
mod dev_cert_loader;
mod secure_link_windows_service;
mod secure_link_service;

use std::env;
use std::net::{ToSocketAddrs};
use std::sync::Arc;
use dotenv::dotenv;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use crate::global_channel::GlobalChannel;

#[macro_use]
extern crate windows_service;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    dotenv().ok();
    env_logger::init();

    
    Ok(())
}
