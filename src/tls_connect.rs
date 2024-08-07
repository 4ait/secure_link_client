use std::net::SocketAddr;
use std::sync::Arc;
use log::info;
use rustls::ClientConfig;
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::{TlsConnector, TlsStream};

pub async fn connect_to_domain(config: Arc<ClientConfig>, socket_addr: SocketAddr, domain: String) -> Result<TlsStream<TcpStream>, anyhow::Error> {
    
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