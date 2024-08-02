use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelJoinRequest {
    pub r#type: String,
    pub channel_token: String
}

impl ProxyChannelJoinRequest {
    const TYPE: &'static str = "proxy_channel_join_request";

    pub(crate) fn new(channel_token: String) -> Self {
        ProxyChannelJoinRequest {
            r#type: Self::TYPE.to_string(),
            channel_token
        }
    }
}

