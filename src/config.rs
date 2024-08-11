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
pub struct Config {
    pub mqtt_broker: MqttBrokerConfig,
    pub vehicle: VehicleConfig,
}
