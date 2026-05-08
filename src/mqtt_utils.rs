use paho_mqtt as mqtt;
use uuid::Uuid;

use crate::config::{Config, MqttBrokerConfig};

pub fn build_server_uri(broker_config: &MqttBrokerConfig) -> String {
    let scheme = if broker_config.tls.unwrap_or(false) { "ssl" } else { "tcp" };
    format!("{}://{}:{}", scheme, broker_config.host, broker_config.port)
}

pub fn resolve_client_id(broker_config: &MqttBrokerConfig) -> String {
    let tls_enabled = broker_config.tls.unwrap_or(false);
    match (tls_enabled, broker_config.client_id.as_ref()) {
        (true, Some(id)) => id.clone(),
        _ => Uuid::new_v4().to_string(),
    }
}

pub fn build_ssl_options(broker_config: &MqttBrokerConfig) -> mqtt::SslOptions {
    let mut builder = mqtt::SslOptionsBuilder::new();
    if let Some(ca_file) = &broker_config.ca_file {
        builder.trust_store(ca_file).expect("invalid ca_file path");
    }
    if let Some(cert_file) = &broker_config.cert_file {
        builder.key_store(cert_file).expect("invalid cert_file path");
    }
    if let Some(key_file) = &broker_config.key_file {
        builder.private_key(key_file).expect("invalid key_file path");
    }
    builder.finalize()
}

pub fn mqtt_create_opts(config: &Config) -> mqtt::CreateOptions {
    let server_uri = build_server_uri(&config.mqtt_broker);
    let client_id = resolve_client_id(&config.mqtt_broker);

    println!(
        "Creating client to: {:?}, client_id: {:?}",
        server_uri, client_id
    );
    mqtt::CreateOptionsBuilder::new()
        .server_uri(&server_uri)
        .client_id(&client_id)
        .finalize()
}

pub async fn mqtt_publish(mqtt_cli: &mqtt::AsyncClient, topic: &str, data: &str) -> mqtt::Result<()> {
    let json: serde_json::Value = serde_json::from_str(data).unwrap();
    let payload = serde_json::to_vec(&json).unwrap();
    let msg = mqtt::Message::new(topic, payload, mqtt::QOS_1);
    mqtt_cli.publish(msg).await?;
    Ok(())
}

pub fn generate_vda_mqtt_base_topic(
    vda_interface: &str,
    vda_version: &str,
    manufacturer: &str,
    serial_number: &str,
) -> String {
    let vda5050_base_topic = format!(
        "{}/{}/{}/{}",
        vda_interface, vda_version, manufacturer, serial_number
    );

    return vda5050_base_topic;
}
