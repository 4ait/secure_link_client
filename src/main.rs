use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use dotenv::dotenv;
use log::info;
use tokio::fs::File;
use tokio::io::{AsyncReadExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::{TlsConnector, TlsStream};
use x509_parser::pem::{Pem};

async fn load_dev_cert(cert_path: &str) -> Result<Vec<CertificateDer>, Box<dyn std::error::Error>> {
   
    let mut cert_file = File::open(cert_path).await?;
    let mut data = Vec::new();
    cert_file.read_to_end(&mut data).await?;
    
    let der_certs: Vec<CertificateDer> = 
        Pem::iter_from_buffer(&data).map(|pem| {
        
            let pem = pem.expect("Reading next PEM block failed");
            let x509 = pem.parse_x509().expect("X.509: decoding DER failed");
            
            println!("{}", x509.subject);
    
            CertificateDer::from(pem.contents)
        
        }).collect();
    
    
    Ok(der_certs)
}

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

    let server_name = ServerName::IpAddress(socket_addr.ip().into());

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

    let addr = "127.0.0.1:6001";
    let socket_addr = addr.to_socket_addrs()?.next().ok_or("Unable to resolve domain")?;

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

    let config = Arc::new(config);

    let mut connection = connect_to_domain(config, socket_addr, "127.0.0.1".into()).await.unwrap();
    
    let mut buff = [0; 1];
    
    connection.read_exact(&mut buff).await.unwrap();
    
    Ok(())
}
