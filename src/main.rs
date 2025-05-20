mod pdu;
mod global_channel;
mod proxy_channel;
mod tls_connect;

#[cfg(feature = "load_dev_certs")]
mod dev_cert_loader;

use std::env;
use std::net::{ToSocketAddrs};
use std::sync::Arc;
use dotenv::dotenv;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use crate::global_channel::GlobalChannel;

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
    
    #[cfg(feature = "load_dev_certs")]
    {
        // Load the development certificate
        let dev_cert_path = "dev_certs/localhost.crt";
        let dev_certs = dev_cert_loader::DevCertLoader::load_dev_cert(dev_cert_path).await?;

        for dev_cert in dev_certs {
            root_cert_store.add(dev_cert).unwrap();
        }
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
