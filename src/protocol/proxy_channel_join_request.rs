use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelJoinRequest {
    pub r#type: String,
    pub secure_link_session_id: String,
    pub channel_token: String
}

impl ProxyChannelJoinRequest {
    const TYPE: &'static str = "proxy_channel_join_request";

    pub(crate) fn new(secure_link_session_id: String, channel_token: String) -> Self {
        ProxyChannelJoinRequest {
            r#type: Self::TYPE.to_string(),
            secure_link_session_id,
            channel_token
        }
    }
}

