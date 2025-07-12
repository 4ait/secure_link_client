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
    #[error("GlobalChannelConnectError")] GlobalChannelConnectError(Box<dyn std::error::Error + Send>),
    #[error("ProtocolSerializationError")] ProtocolSerializationError(Box<dyn std::error::Error + Send>),
    #[error("TlsStreamError")] TlsStreamError(Box<dyn std::error::Error + Send>),
    #[error("UnauthorizedError")] UnauthorizedError,
    #[error("SecureLinkServerConnectionLost")] SecureLinkServerConnectionLost(Box<dyn std::error::Error + Send>),
    #[error("ProxyChannelJoinDenied")] ProxyChannelJoinDenied
}

pub use secure_link::SecureLink;

static_assertions::assert_impl_all!(SecureLink: Send, Sync);
static_assertions::assert_impl_all!(SecureLinkError: Send);


