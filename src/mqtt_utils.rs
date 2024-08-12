use paho_mqtt as mqtt;
use uuid::Uuid;

pub fn mqtt_create_opts() -> mqtt::CreateOptions {
    let config = crate::config::get_config();
    
    let server_uri = format!(
        "tcp://{}:{}",
        config.mqtt_broker.host, config.mqtt_broker.port
    );
    let client_id = Uuid::new_v4().to_string();


    println!(
        "Creating client to: {:?}, client_id: {:?}",
        server_uri, client_id
    );
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(&server_uri)
        .client_id(&client_id)
        .finalize();
    return create_opts;
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