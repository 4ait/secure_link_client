use std::sync::{Arc};
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use crate::protocol::global_channel_message::CsGlobalChannelMessage;
use crate::SecureLinkError;

#[derive(Clone)]
pub struct CsGlobalChannelSender(Arc<CsGlobalChannelSenderInner>);

impl CsGlobalChannelSender {

    pub fn new(sender: WriteHalf<TlsStream<TcpStream>>) -> CsGlobalChannelSender {
        CsGlobalChannelSender(
            Arc::new(
                CsGlobalChannelSenderInner {
                    sender: tokio::sync::Mutex::new(sender),
                }
            )
        )
    }

    pub async fn send_cs_global_channel_message(&self, global_channel_message: CsGlobalChannelMessage) -> Result<(), SecureLinkError> {
        self.0.send_cs_global_channel_message(global_channel_message).await
    }
}

struct CsGlobalChannelSenderInner {
    sender: tokio::sync::Mutex<WriteHalf<TlsStream<TcpStream>>>
}

impl CsGlobalChannelSenderInner {

    pub async fn send_cs_global_channel_message(
        &self,
        message: CsGlobalChannelMessage
    ) -> Result<(), SecureLinkError> {

        let message_json =
            serde_json::to_string(&message)
                .map_err(|_err| { SecureLinkError::ProtocolSerializationError })?;

        let pdu_length = (message_json.len() as u32).to_be_bytes();

        let mut global_channel_cs_pdu = vec![0u8];

        global_channel_cs_pdu.extend_from_slice(&pdu_length);
        global_channel_cs_pdu.extend_from_slice(message_json.as_bytes());

        // Scope the lock so it's dropped before the await
        {
            let mut sender = self.sender.lock().await;
            sender.write_all(&global_channel_cs_pdu).await
                .map_err(|_err| { SecureLinkError::TlsStreamError })?;
        } // MutexGuard is dropped here

        Ok(())
    }
}