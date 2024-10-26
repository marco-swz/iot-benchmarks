use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub websocket: WebsocketConfig,
    pub tcp: TcpConfig,
    pub opcua: OpcuaConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MqttConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
    pub topic_send: String,
    pub topic_recv: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebsocketConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TcpConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpcuaConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ScheduleConfig {
    pub start_req_per_sec: f64,
    pub stop_req_per_sec: f64,
    pub steps: usize,
    pub secs_per_step: u64,
}
