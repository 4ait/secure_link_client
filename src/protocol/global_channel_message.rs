use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScGlobalChannelMessage {
    #[serde(rename = "proxy_channel_open_request")]
    ProxyChannelOpenRequest(ProxyChannelOpenRequest),
    #[serde(rename = "health_check_request")]
    HealthCheckRequest,
    #[serde(rename = "health_check_response")]
    HealthCheckResponse
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelOpenRequest {
    pub proxy_channel_id: String,
    pub channel_token: String,
    pub destination: ProxyDestination
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyDestination {
    pub ip: String,
    pub port: u16
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CsGlobalChannelMessage {
    #[serde(rename = "proxy_channel_open_response")]
    ProxyChannelOpenResponse(ProxyChannelOpenResponse),
    #[serde(rename = "health_check_request")]
    HealthCheckRequest,
    #[serde(rename = "health_check_response")]
    HealthCheckResponse
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChannelOpenResponse {
    pub proxy_channel_id: String,
    pub result: ProxyChannelOpenResponseResult
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProxyChannelOpenResponseResult {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "bad_destination_address")]
    BadDestinationAddress,
    #[serde(rename = "could_not_reach_destination")]
    CouldNotReachDestination
}
