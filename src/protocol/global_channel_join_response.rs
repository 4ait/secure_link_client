use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GlobalChannelJoinResponse {
    #[serde(rename = "global_channel_join_confirmed")]
    GlobalChannelJoinConfirmed(GlobalChannelJoinConfirmed),

    #[serde(rename = "global_channel_join_denied")]
    GlobalChannelJoinDenied(GlobalChannelJoinDenied)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalChannelJoinConfirmed { 
    pub secure_link_session_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalChannelJoinDenied{}
