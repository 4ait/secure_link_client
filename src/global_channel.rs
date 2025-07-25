use std::net::{SocketAddr};
use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use log::{debug, error, info, warn};
use rustls::ClientConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use crate::cs_global_chanel_sender::CsGlobalChannelSender;
use crate::protocol::global_channel_join_request::GlobalChannelJoinRequest;
use crate::protocol::global_channel_join_response::GlobalChannelJoinResponse;
use crate::protocol::global_channel_message::{CsGlobalChannelMessage, ProxyChannelOpenResponse, ProxyChannelOpenResponseResult, ScGlobalChannelMessage};
use crate::protocol::global_channel_message::ScGlobalChannelMessage::ProxyChannelOpenRequest;
use crate::proxy_channel::ProxyChannel;
use crate::SecureLinkError;
use crate::tls_connect::connect_to_domain;

pub struct GlobalChannel {
    secure_link_session_id: String,
    secure_link_server_socket_addr: SocketAddr,
    secure_link_server_domain: String,
    tls_stream: TlsStream<TcpStream>,
    tls_config: Arc<ClientConfig>,
    running_health_check_channel: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>
}

impl GlobalChannel {

    pub async fn create_global_channel(secure_link_server_socket_addr: SocketAddr, secure_link_server_domain: String, tls_config: Arc<ClientConfig>, auth_token: String) -> Result<GlobalChannel, SecureLinkError> {

        let mut tls_stream = 
            connect_to_domain(
                tls_config.clone(), 
                secure_link_server_socket_addr,
                secure_link_server_domain.clone()
            )
            .await
            .map_err(|err| { SecureLinkError::GlobalChannelConnectError(err.into()) })?;
        
        let global_channel_join_request = GlobalChannelJoinRequest::new(auth_token);

        let request_json =
            serde_json::to_string(&global_channel_join_request)
                .map_err(|err| { SecureLinkError::ProtocolSerializationError(Box::new(err)) })?;

        let pdu_length = (request_json.len() as u32).to_be_bytes();

        let mut global_channel_join_request_pdu= vec![0u8];

        global_channel_join_request_pdu.extend_from_slice(&pdu_length);
        global_channel_join_request_pdu.extend_from_slice(request_json.as_bytes());

        tls_stream.write(&global_channel_join_request_pdu).await
            .map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;

        let _reserved = tls_stream.read_u8().await.map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;
        let length = tls_stream.read_u32().await.map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;

        let mut message = vec![0; length as usize];

        tls_stream.read_exact(&mut message).await.map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;

        let channel_join_response =
            serde_json::from_slice::<GlobalChannelJoinResponse>(&message)
                .map_err(|err| { SecureLinkError::ProtocolSerializationError(Box::new(err)) })?;

        match channel_join_response {
            GlobalChannelJoinResponse::GlobalChannelJoinConfirmed(global_channel_join_confirmed) => {

                let global_channel =
                    GlobalChannel {
                        secure_link_session_id: global_channel_join_confirmed.secure_link_session_id,
                        secure_link_server_socket_addr,
                        secure_link_server_domain,
                        tls_stream,
                        tls_config,
                        running_health_check_channel: Arc::new(Mutex::new(None))
                    };

                Ok(global_channel)

            }
            GlobalChannelJoinResponse::GlobalChannelJoinDenied(_) => {
                Err(SecureLinkError::UnauthorizedError)
            }
        }


    }


    pub async fn run_message_loop(self) -> Result<(), SecureLinkError> {

        let (mut tls_stream_reader, tls_stream_writer) = tokio::io::split(self.tls_stream);

        let (unrecoverable_error_in_channels_sender, mut unrecoverable_error_in_channels_receiver) = tokio::sync::mpsc::channel::<SecureLinkError>(1);

        let (health_check_failed_sender, mut health_check_receiver) = tokio::sync::mpsc::channel::<()>(1);
        
        let global_channel_sender = CsGlobalChannelSender::new(tls_stream_writer);

        let secure_link_server_socket_addr = self.secure_link_server_socket_addr;
        let secure_link_server_domain = self.secure_link_server_domain;
        let tls_config = self.tls_config;
        let secure_link_session_id = self.secure_link_session_id;

        let running_health_check_channel_clone = self.running_health_check_channel.clone();
        let global_channel_sender_clone = global_channel_sender.clone();
        
        tokio::spawn(async move {
           
            health_check_loop(
                running_health_check_channel_clone,
                global_channel_sender_clone,
                health_check_failed_sender
            ).await;
            
        });
        
        loop {

            let global_channel_message =
                receive_next_sc_global_channel_message(
                    &mut tls_stream_reader
                ).await?;

            let running_health_check_channel = self.running_health_check_channel.clone();
            
            let handle_sc_global_channel_message_future = handle_sc_global_channel_message(
                global_channel_message,
                &secure_link_server_socket_addr,
                &secure_link_server_domain,
                tls_config.clone(),
                &secure_link_session_id,
                &global_channel_sender,
                &unrecoverable_error_in_channels_sender,
                running_health_check_channel
            );


            tokio::select! {
                
                handle_sc_global_channel_result = handle_sc_global_channel_message_future => {

                    match handle_sc_global_channel_result {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(err);
                        }
                    }

                }

                Some(unrecoverable_error_in_channel) = unrecoverable_error_in_channels_receiver.recv() => {
                    return Err(unrecoverable_error_in_channel)
                }
                
                Some(()) = health_check_receiver.recv() => {
                    return Err(SecureLinkError::SecureLinkServerConnectionLost(
                        anyhow!("health check failed").into())
                    )
                }
                
            }

        }

        async fn health_check_loop(
            running_health_check_channel: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
            global_channel_sender: CsGlobalChannelSender,
            health_check_failed_sender: tokio::sync::mpsc::Sender<()>
        ) {
            use tokio::time::{interval, timeout, Duration};

            const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;
            const HEALTH_CHECK_TIMEOUT_SECS: u64 = 10;

            let mut interval = interval(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS));

            loop {
                interval.tick().await;

                debug!("Sending health check request");

                // Create a channel for this specific health check
                let (response_sender, mut response_receiver) = tokio::sync::mpsc::channel::<()>(1);

                // Store the response channel
                {
                    let mut guard = running_health_check_channel.lock().unwrap();
                    *guard = Some(response_sender);
                }

                // Send health check request
                if let Err(err) = global_channel_sender.send_cs_global_channel_message(
                    CsGlobalChannelMessage::HealthCheckRequest
                ).await {
                    error!("Failed to send health check request: {}", err);

                    // Clean up the channel
                    {
                        let mut guard = running_health_check_channel.lock().unwrap();
                        *guard = None;
                    }

                    // Signal health check failure
                    let _ = health_check_failed_sender.send(()).await;
                    return;
                }

                // Wait for response with timeout
                match timeout(Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS), response_receiver.recv()).await {
                    Ok(Some(())) => {
                        debug!("Health check response received");
                        // Clean up the channel
                        {
                            let mut guard = running_health_check_channel.lock().unwrap();
                            *guard = None;
                        }
                    }
                    Ok(None) => {
                        error!("Health check channel closed unexpectedly");
                        // Clean up the channel
                        {
                            let mut guard = running_health_check_channel.lock().unwrap();
                            *guard = None;
                        }
                        // Signal health check failure
                        let _ = health_check_failed_sender.send(()).await;
                        return;
                    }
                    Err(_) => {
                        error!("Health check timeout - no response received within {} seconds", HEALTH_CHECK_TIMEOUT_SECS);

                        // Clean up the channel
                        {
                            let mut guard = running_health_check_channel.lock().unwrap();
                            *guard = None;
                        }

                        // Signal health check failure
                        let _ = health_check_failed_sender.send(()).await;
                        return;
                    }
                }
            }
        }

        async fn handle_sc_global_channel_message(
            global_channel_message: ScGlobalChannelMessage,
            secure_link_server_socket_addr: &SocketAddr,
            secure_link_server_domain: &str,
            tls_config: Arc<ClientConfig>,
            _secure_link_session_id: &str,
            global_channel_sender: &CsGlobalChannelSender,
            unrecoverable_error_in_channels_sender: &tokio::sync::mpsc::Sender<SecureLinkError>,
            running_health_check_channel: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>
        ) -> Result<(), SecureLinkError> {

            match global_channel_message {

                ScGlobalChannelMessage::HealthCheckRequest => {

                    let _result = global_channel_sender.send_cs_global_channel_message(
                        CsGlobalChannelMessage::HealthCheckResponse
                    ).await;
                    
                }

                ScGlobalChannelMessage::HealthCheckResponse => {
                    
                    let channel = {
                        running_health_check_channel.lock().unwrap().take()
                    };
                    
                    if let Some(running_health_check_channel) = channel {
                        
                        if let Err(err) = running_health_check_channel.send(()).await {
                            warn!("failed to process health check result: {}", err);
                        }
                    }
                    
                }
                
                ProxyChannelOpenRequest(proxy_channel_open_request) => {

                    let proxy_channel_id = proxy_channel_open_request.proxy_channel_id;
                    let destination = proxy_channel_open_request.destination;

                    // Create destination address string that can handle both IP and DNS
                    let destination_addr = format!("{}:{}", destination.ip, destination.port);

                    let secure_link_server_socket_addr = secure_link_server_socket_addr.clone();
                    let tls_config = tls_config.clone();
                    let global_channel_sender = global_channel_sender.clone();
                    let unrecoverable_error_in_channels_sender = unrecoverable_error_in_channels_sender.clone();
                    let secure_link_server_domain = secure_link_server_domain.to_string();

                    tokio::spawn(async move {

                        // TcpStream::connect can handle both IP addresses and DNS names
                        match TcpStream::connect(&destination_addr).await {

                            Ok(dst_tcp_stream) => {

                                let proxy_channel_create_result =
                                    ProxyChannel::create_proxy_channel_with_secure_link_server(
                                        secure_link_server_socket_addr,
                                        secure_link_server_domain,
                                        tls_config,
                                        dst_tcp_stream,
                                        proxy_channel_open_request.channel_token
                                    ).await;

                                match proxy_channel_create_result {

                                    Ok(proxy_channel) => {

                                        let proxy_channel_run_result =
                                            proxy_channel.run_proxy_between_sender_and_secure_link_server().await;

                                        match proxy_channel_run_result {
                                            Ok(()) => {
                                                info!("proxy channel down");
                                            }
                                            Err(err) => {
                                                warn!("proxy channel down with error: {}", err);
                                            }
                                        }


                                    }
                                    Err(err) => {

                                        error!("SecureLinkServerConnectionLost in proxy channel: {}", err);

                                        let _result = unrecoverable_error_in_channels_sender.send(
                                            SecureLinkError::SecureLinkServerConnectionLost(Box::new(err))
                                        ).await;

                                    }
                                };

                            }
                            Err(err) => {

                                let _result = global_channel_sender.send_cs_global_channel_message(
                                    CsGlobalChannelMessage::ProxyChannelOpenResponse(
                                        ProxyChannelOpenResponse {
                                            proxy_channel_id,
                                            result: ProxyChannelOpenResponseResult::CouldNotReachDestination
                                        }
                                    )
                                ).await;

                                warn!("failed to connect to requested dst {:?}", err)

                            }
                        }

                    });

                }
            }

            Ok(())

        }

        async fn receive_next_sc_global_channel_message(tls_stream_reader: &mut ReadHalf<TlsStream<TcpStream>>) -> Result<ScGlobalChannelMessage, SecureLinkError>{

            let _reserved = tls_stream_reader.read_u8().await.map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;
            let length = tls_stream_reader.read_u32().await.map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;

            let mut global_channel_message_bytes = vec![0; length as usize];

            tls_stream_reader.read_exact(&mut global_channel_message_bytes).await
                .map_err(|err| { SecureLinkError::TlsStreamError(Box::new(err)) })?;
            
            let global_channel_message =
                serde_json::from_slice::<ScGlobalChannelMessage>(&global_channel_message_bytes)
                    .map_err(|err| { SecureLinkError::ProtocolSerializationError(Box::new(err)) })?;

            Ok(global_channel_message)

        }

    }

}