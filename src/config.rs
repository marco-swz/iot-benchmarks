use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub websocket: WebsocketConfig,
    pub tcp: TcpConfig,
    pub opcua: OpcuaConfig,
}

#[derive(Deserialize, Debug)]
pub struct MqttConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct WebsocketConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct TcpConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct OpcuaConfig {
    pub address: String,
    pub schedule: ScheduleConfig,
    pub message_size: usize,
}

#[derive(Deserialize, Debug)]
pub struct ScheduleConfig {
    pub start_req_per_sec: f64,
    pub stop_req_per_sec: f64,
    pub steps: usize,
    pub secs_per_step: u64,
}
