use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalChannelJoinRequest {
    pub r#type: String,
    pub auth_token: String
}

impl GlobalChannelJoinRequest {
    const TYPE: &'static str = "global_channel_join_request";

    pub(crate) fn new(auth_token: String) -> Self {
        GlobalChannelJoinRequest {
            r#type: Self::TYPE.to_string(),
            auth_token
        }
    }
}