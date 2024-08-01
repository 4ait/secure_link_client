use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use dotenv::dotenv;
use log::info;
use rustls::pki_types::CertificateDer;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::pki_types;
use tokio_rustls::{TlsConnector, TlsStream};



async fn connect_to_domain(config: Arc<ClientConfig>, socket_addr: SocketAddr, domain: String) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {

    let connector = TlsConnector::from(config);
    let server_name = ServerName::try_from(domain)?;

    // Create a TCP connection
    let tcp_stream = TcpStream::connect(&socket_addr).await?;
    info!("Connected to the server via TCP");

    // Establish a TLS connection
    let tls_stream = connector.connect(server_name, tcp_stream).await?;
    info!("TLS connection established");

    Ok(tls_stream.into())
}

async fn connect_to_address(config: Arc<ClientConfig>, socket_addr: SocketAddr) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {


    let connector = TlsConnector::from(config);

    let server_name =
        ServerName::IpAddress(
            pki_types::IpAddr::from(socket_addr.ip())
        );

    // Create a TCP connection
    let tcp_stream = TcpStream::connect(&socket_addr).await?;
    info!("Connected to the server via TCP");

    // Establish a TLS connection
    let tls_stream = connector.connect(server_name, tcp_stream).await?;
    info!("TLS connection established");

    Ok(tls_stream.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();
    env_logger::init();

    //let addr = "rdp.4ait.ru:443";
    let addr = "127.0.0.1:6001";
    let socket_addr = addr.to_socket_addrs()?.next().ok_or("Unable to resolve domain")?;

    // Load system root certificates
    let mut root_cert_store = RootCertStore::empty();
    let root_certs = rustls_native_certs::load_native_certs()
        .expect("Could not load platform dev_certs");

    for cert in root_certs {
        root_cert_store.add(cert).unwrap();
    }

    let mut buff = String::new();

    File::open("dev_certs/server.crt").await?.read_to_string(&mut buff).await.expect("TODO: panic message");

    let cert = CertificateDer::from(buff.as_bytes());

    root_cert_store.add(cert).unwrap();

    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let config = Arc::new(config);

    connect_to_address(config, socket_addr).await.unwrap();



    Ok(())

    // Now you can use tls_stream to communicate securely with the server


}