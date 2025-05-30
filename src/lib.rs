mod protocol;
mod global_channel;
mod proxy_channel;
mod tls_connect;
#[cfg(feature = "load_dev_certs")]
mod dev_cert_loader;
mod secure_link;

mod cs_global_chanel_sender;

#[derive(thiserror::Error, Debug)]
pub enum SecureLinkError {

    #[error("DevCertificatesLoadingError")] DevCertificatesLoadingError,
    #[error("BadHostError")] BadHostError,
    #[error("GlobalChannelConnectError")] GlobalChannelConnectError,
    #[error("ProtocolSerializationError")] ProtocolSerializationError,
    #[error("TlsStreamError")] TlsStreamError,
    #[error("GlobalChannelJoinDeniedError")] GlobalChannelJoinDeniedError,
    #[error("SecureLinkServerConnectionLost")] SecureLinkServerConnectionLost,
    #[error("ProxyChannelJoinDenied")] ProxyChannelJoinDenied
}

pub use secure_link::SecureLink;
