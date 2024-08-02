use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum GlobalChannelMessage {
    #[serde(rename = "proxy_channel_open_request")]
    ProxyChannelOpenRequest(ProxyChannelOpenRequest)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelOpenRequest {
    pub channel_token: String,
    pub destination: ProxyDestination
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyDestination {
    pub ip: String,
    pub port: u16
}