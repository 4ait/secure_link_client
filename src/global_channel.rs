use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use rustls::ClientConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use x509_parser::nom::AsBytes;
use crate::pdu::global_channel_join_request::GlobalChannelJoinRequest;
use crate::pdu::global_channel_join_response::GlobalChannelJoinResponse;
use crate::pdu::global_channel_message::GlobalChannelMessage;
use crate::proxy_channel::ProxyChannel;
use crate::tls_connect::connect_to_domain;

pub struct GlobalChannel {
    connection_id: String,
    secure_link_server_socket_addr: SocketAddr,
    secure_link_server_domain: String,
    tls_stream: TlsStream<TcpStream>,
    tls_config: Arc<ClientConfig>
}

impl GlobalChannel {

    pub async fn create_global_channel(secure_link_server_socket_addr: SocketAddr, secure_link_server_domain: String, tls_config: Arc<ClientConfig>, auth_token: String) -> Result<GlobalChannel, anyhow::Error> {
        
        let mut tls_stream = 
            connect_to_domain(
                tls_config.clone(), 
                secure_link_server_socket_addr,
                secure_link_server_domain.clone()
            )
            .await
            .unwrap();
        
        let global_channel_join_request = GlobalChannelJoinRequest::new(auth_token);

        let request_json = serde_json::to_string(&global_channel_join_request).unwrap();

        let pdu_length = (request_json.len() as u32).to_be_bytes();

        let mut global_channel_join_request_pdu= vec![0u8];

        global_channel_join_request_pdu.extend_from_slice(&pdu_length);
        global_channel_join_request_pdu.extend_from_slice(request_json.as_bytes());

        tls_stream.write(&global_channel_join_request_pdu).await?;

        let _reserved = tls_stream.read_u8().await?;
        let length = tls_stream.read_u32().await?;

        let mut message = vec![0; length as usize];

        tls_stream.read_exact(&mut message).await?;

        let channel_join_response = serde_json::from_slice::<GlobalChannelJoinResponse>(&message)?;

        match channel_join_response {
            GlobalChannelJoinResponse::GlobalChannelJoinConfirmed(global_channel_join_confirmed) => {

                let global_channel =
                    GlobalChannel {
                        connection_id: global_channel_join_confirmed.connection_id,
                        secure_link_server_socket_addr,
                        secure_link_server_domain,
                        tls_stream,
                        tls_config
                    };

                Ok(global_channel)

            }
            GlobalChannelJoinResponse::GlobalChannelJoinDenied(_) => {

                Err(anyhow!("GlobalChannelJoinDenied"))

            }
        }


    }


    pub async fn run_message_loop(self) -> Result<(), anyhow::Error> {

        let mut tls_stream = self.tls_stream;
        let secure_link_server_socket_addr = self.secure_link_server_socket_addr;
        let secure_link_server_domain = self.secure_link_server_domain;
        let tls_config = self.tls_config;

        loop {

            let _reserved = tls_stream.read_u8().await?;
            let length = tls_stream.read_u32().await?;

            let mut global_channel_message_bytes = vec![0; length as usize];

            tls_stream.read_exact(&mut global_channel_message_bytes).await?;

            let global_channel_message =
                serde_json::from_slice::<GlobalChannelMessage>(&global_channel_message_bytes)?;

            match global_channel_message  {
                GlobalChannelMessage::ProxyChannelOpenRequest(proxy_channel_open_request) => {

                    let destination = proxy_channel_open_request.destination;

                    let ip_address =  IpAddr::from_str(&destination.ip).unwrap();

                    let destination_socket_addr =
                        SocketAddr::new(
                            ip_address,
                            destination.port
                        );

                    let secure_link_server_socket_addr = secure_link_server_socket_addr.clone();
                    let secure_link_server_domain = secure_link_server_domain.clone();
                    let tls_config = tls_config.clone();
                    
                    tokio::spawn(async move {
                        
                        let tcp_stream = TcpStream::connect(&destination_socket_addr).await.unwrap();
                        
                        let proxy_channel = 
                            ProxyChannel::create_proxy_channel(
                                secure_link_server_socket_addr,
                                secure_link_server_domain,
                                tls_config,
                                tcp_stream,
                                proxy_channel_open_request.channel_token
                            ).await.unwrap();
                        
                        proxy_channel.run_proxy().await.unwrap();
                        
                    });

                }
            }


        }

    }

}