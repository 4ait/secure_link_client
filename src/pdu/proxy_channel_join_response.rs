use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProxyChannelJoinResponse {

    #[serde(rename = "proxy_channel_join_confirmed")]
    ProxyChannelJoinConfirmed(ProxyChannelJoinConfirmed),

    #[serde(rename = "proxy_channel_join_denied")]
    ProxyChannelJoinDenied(ProxyChannelJoinDenied)

}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelJoinConfirmed {
    proxy_channel_id: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelJoinDenied{}
