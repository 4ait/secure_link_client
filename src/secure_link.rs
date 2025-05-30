use std::net::ToSocketAddrs;
use std::sync::Arc;
use rustls::{ClientConfig, RootCertStore};
use crate::global_channel::GlobalChannel;
use crate::SecureLinkError;

pub struct SecureLink {
    global_channel: Option<GlobalChannel>
}

impl SecureLink {
    
    pub async fn connect_to_global_channel(
        secure_link_server_host: &str, 
        secure_link_server_port: u16,
        auth_token: &str,
    ) -> Result<SecureLink, SecureLinkError> {

        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        #[cfg(feature = "load_dev_certs")]
        crate::dev_cert_loader::DevCertLoader::load_dev_certs(&mut root_cert_store).await.unwrap();
        
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let tls_config = Arc::new(config);

        let socket_addr = match (secure_link_server_host, secure_link_server_port).to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(addr) => addr,
                None => {
                    log::error!("Unable to resolve server address");
                    return Err(SecureLinkError::BadHostError);
                }
            },
            Err(e) => {
                log::error!("Invalid server address: {}", e);
                return Err(SecureLinkError::BadHostError);
            }
        };
        
        let global_channel = 
            GlobalChannel::create_global_channel(
                socket_addr,
                secure_link_server_host.to_string(),
                tls_config,
                auth_token.to_string()
            ).await?;
        
        
        Ok(SecureLink { global_channel: Some(global_channel) })
        
    }
    
    pub async fn run_message_loop(self) -> Result<(), SecureLinkError> {
        self.global_channel.unwrap().run_message_loop().await?;
        Ok(())
    }
    
}