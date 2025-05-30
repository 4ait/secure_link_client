use std::env;
use std::ffi::OsString;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use futures::executor::block_on;
use rustls::{ClientConfig, RootCertStore};
use windows_service::{define_windows_service, service_dispatcher};
use crate::global_channel::GlobalChannel;

define_windows_service!(ffi_secure_link_service_main, secure_link_service_main);

fn secure_link_service_main(arguments: Vec<OsString>) {
    
    block_on(
        async move || {
            
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
                    .to_socket_addrs()
                    .expect("invalid socket_addr")
                    .next()
                    .expect("Unable to resolve domain");


            let global_channel =
                GlobalChannel::create_global_channel(
                    socket_addr,
                    secure_link_server_host,
                    tls_config,
                    auth_token
                ).await.expect("global channel creation failed");

            global_channel.run_message_loop().await?;

            
        }
    )
    
}

fn run_secure_link_service() -> Result<(), windows_service::Error> {
    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start("myservice", ffi_secure_link_service_main)?;
    Ok(())
}

