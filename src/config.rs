use config_file::FromConfigFile;
use serde::Deserialize;

pub fn get_config() -> Config {
    return Config::from_config_file("config.toml").unwrap();
}

#[derive(Deserialize, Clone)]
pub struct MqttBrokerConfig {
    pub host: String,
    pub port: String,
    pub vda_interface: String,
}

#[derive(Deserialize, Clone)]
pub struct VehicleConfig {
    pub manufacturer: String,
    pub serial_number: String,
    pub vda_version: String,
    pub vda_full_version: String,
}

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub action_time: f32,
    pub speed: f32,
    pub robot_count: u32,
    pub state_frequency: u64,
    pub visualization_frequency: u64,
    pub map_id: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mqtt_broker: MqttBrokerConfig,
    pub vehicle: VehicleConfig,
    pub settings: Settings
}
