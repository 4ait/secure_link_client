mod pdu;
mod global_channel;
mod proxy_channel;
mod tls_connect;

#[cfg(feature = "load_dev_certs")]
mod dev_cert_loader;

use std::env;
use std::net::{ToSocketAddrs};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenv::dotenv;
use rustls::pki_types::CertificateDer;
use rustls_native_certs::load_native_certs;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use x509_parser::nom::AsBytes;
use crate::global_channel::GlobalChannel;

async fn fetch_lets_encrypt_certs() -> Vec<CertificateDer<'static>> {
    
    let isrg_root = reqwest::get("https://letsencrypt.org/certs/isrgrootx1.der")
        .await
        .expect("Failed to fetch ISRG Root X1")
        .bytes()
        .await
        .expect("Failed to read ISRG Root X1");

    let isrg_rootx2 = reqwest::get("https://letsencrypt.org/certs/isrg-root-x2.der")
        .await
        .expect("Failed to fetch ISRG Root X2") // Fixed error message
        .bytes()
        .await
        .expect("Failed to read ISRG Root X2"); // Fixed error message

    let e6_intermediate = reqwest::get("https://letsencrypt.org/certs/2024/e6.der")
        .await
        .expect("Failed to fetch Let's Encrypt E6")
        .bytes()
        .await
        .expect("Failed to read Let's Encrypt E6");

    // Parse DER certificates and convert them to owned data
    let mut certs = Vec::new();

    for der in [isrg_root, isrg_rootx2, e6_intermediate] {
        // Convert to Vec<u8> to take ownership of the data
        let owned_der = der.to_vec();
        // Now create CertificateDer with 'static lifetime using owned data
        certs.push(CertificateDer::from(owned_der));
    }

    certs
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    
    let auth_token = env::var("AUTH_TOKEN")
        .expect("AUTH_TOKEN environment variable not set");

    let secure_link_server_host = env::var("SECURE_LINK_SERVER_HOST")
        .expect("SECURE_LINK_SERVER_HOST environment variable not set");

    let secure_link_server_port: u16 = env::var("SECURE_LINK_SERVER_PORT")
        .expect("SECURE_LINK_SERVER_PORT environment variable not set")
        .parse()
        .expect("SECURE_LINK_SERVER_PORT must be a number");


    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());


    for cert in load_native_certs().expect("could not load platform certs") {
        root_cert_store.add(cert).unwrap();
    }
    
    let lets_encrypt_certs = fetch_lets_encrypt_certs();

    for cert in fetch_lets_encrypt_certs().await {
        root_cert_store.add(cert).unwrap();
    }
    
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let tls_config = Arc::new(config);
    
    let socket_addr =
        (secure_link_server_host.clone(), secure_link_server_port)
            .to_socket_addrs()?
            .next()
            .ok_or("Unable to resolve domain")?;
    
    
    let global_channel =
        GlobalChannel::create_global_channel(
            socket_addr,
            secure_link_server_host,
            tls_config,
            auth_token
        ).await?;
    
    global_channel.run_message_loop().await?;
    
    Ok(())
}
