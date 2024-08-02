mod pdu;
mod global_channel;
mod proxy_channel;
mod tls_connect;

use std::net::{ToSocketAddrs};
use std::sync::Arc;
use dotenv::dotenv;
use tokio::fs::File;
use tokio::io::{AsyncReadExt};
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::{CertificateDer};
use x509_parser::pem::{Pem};
use x509_parser::x509::X509Version;
use crate::global_channel::GlobalChannel;

async fn load_dev_cert(cert_path: &str) -> Result<Vec<CertificateDer>, anyhow::Error> {

    let mut cert_file = File::open(cert_path).await?;
    let mut data = Vec::new();
    cert_file.read_to_end(&mut data).await?;

    let der_certs: Vec<CertificateDer> =
        Pem::iter_from_buffer(&data).map(|pem| {

            let pem = pem.expect("Reading next PEM block failed");
            let x509 = pem.parse_x509().expect("X.509: decoding DER failed");

            assert_eq!(x509.tbs_certificate.version, X509Version::V3);

            CertificateDer::from(pem.contents)

        }).collect();


    Ok(der_certs)
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    
    // Load system root certificates
    let mut root_cert_store = RootCertStore::empty();
    let root_certs = rustls_native_certs::load_native_certs()
        .expect("Could not load platform dev_certs");

    for cert in root_certs {
        root_cert_store.add(cert).unwrap();
    }

    // Load the development certificate
    let dev_cert_path = "dev_certs/localhost.crt";
    let dev_certs = load_dev_cert(dev_cert_path).await?;

    for dev_cert in dev_certs {
        root_cert_store.add(dev_cert).unwrap();
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let tls_config = Arc::new(config);
    
    let addr = "127.0.0.1:6001";
    let socket_addr = addr.to_socket_addrs()?.next().ok_or("Unable to resolve domain")?;
    let domain = "127.0.0.1".to_string();
    
    let auth_token = "abc".to_string();

    let global_channel = GlobalChannel::create_global_channel(socket_addr, domain, tls_config, auth_token).await?;
    
    global_channel.run_message_loop().await?;
    
    Ok(())
}
