use futures_util::StreamExt;
use paho_mqtt as mqtt;
use std::sync::Arc;
use std::{process, time::Duration};
use tokio::sync::Mutex;

use crate::config;
use crate::mqtt_utils;
use crate::utils;
use crate::vehicle_simulator::VehicleSimulator;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_order::Order;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_instant_actions::InstantActions;

pub async fn subscribe_vda_messages(
    config: config::Config, 
    simulator: Arc<Mutex<VehicleSimulator>>
) {
    let base_topic = format!(
        "{}/{}/{}/{}",
        config.mqtt_broker.vda_interface,
        config.vehicle.vda_version,
        config.vehicle.manufacturer,
        config.vehicle.serial_number,
    );

    let topics = vec![
        format!("{}/order", base_topic),
        format!("{}/instantActions", base_topic),
    ];

    if topics.is_empty() {
        println!("Error: No topics specified!");
        process::exit(-1);
    }

    let qos = vec![1; topics.len()];
    let mut mqtt_client = create_mqtt_client();
    let mut message_stream = mqtt_client.get_stream(25);

    connect_to_broker(&mqtt_client).await;
    subscribe_to_topics(&mqtt_client, &topics, &qos).await;

    println!("Waiting for messages on topics: {:?}", topics);
    
    while let Some(msg_opt) = message_stream.next().await {
        if let Some(msg) = msg_opt {
            handle_incoming_message(msg, &simulator).await;
        } else {
            handle_connection_loss(&mqtt_client).await;
        }
    }
}

pub async fn publish_vda_messages(
    simulator: Arc<Mutex<VehicleSimulator>>,
    state_frequency: u64,
    visualization_frequency: u64,
) {
    let mqtt_client = create_mqtt_client();
    connect_to_broker(&mqtt_client).await;

    // Publish initial connection
    simulator.lock().await.publish_connection(&mqtt_client).await;

    // Main publishing loop
    let tick_time = 50;
    let mut state_counter = 0;
    let mut visualization_counter = 0;

    loop {
        simulator.lock().await.update_state();

        // Publish state at specified frequency
        state_counter += 1;
        if state_counter * tick_time > 1000 / state_frequency {
            state_counter = 0;
            simulator.lock().await.publish_state(&mqtt_client).await;
        }

        // Publish visualization at specified frequency
        visualization_counter += 1;
        if visualization_counter * tick_time > 1000 / visualization_frequency {
            visualization_counter = 0;
            simulator.lock().await.publish_visualization(&mqtt_client).await;
        }

        tokio::time::sleep(Duration::from_millis(tick_time)).await;
    }
}

fn create_mqtt_client() -> mqtt::AsyncClient {
    mqtt::AsyncClient::new(mqtt_utils::mqtt_create_opts()).unwrap_or_else(|e| {
        println!("Error creating MQTT client: {:?}", e);
        process::exit(-1);
    })
}

async fn connect_to_broker(mqtt_client: &mqtt::AsyncClient) {
    let conn_opts = mqtt::ConnectOptionsBuilder::with_mqtt_version(mqtt::MQTT_VERSION_5)
        .clean_start(true)
        .finalize();

    mqtt_client.connect(conn_opts).await.unwrap();
}

async fn subscribe_to_topics(
    mqtt_client: &mqtt::AsyncClient, 
    topics: &[String], 
    qos: &[i32]
) {
    println!("Subscribing to topics: {:?}", topics);
    mqtt_client.subscribe_many(topics, qos).await.unwrap();
}

async fn handle_incoming_message(
    msg: mqtt::Message, 
    simulator: &Arc<Mutex<VehicleSimulator>>
) {
    if msg.retained() {
        print!("(R) ");
    }

    let topic = msg.topic();
    let topic_type = utils::get_topic_type(topic);
    let payload = String::from_utf8_lossy(msg.payload()).to_string();

    match topic_type.as_ref() {
        "order" => handle_order_message(&payload, simulator).await,
        "instantActions" => handle_instant_actions_message(&payload, simulator).await,
        _ => println!("Unknown topic type: {}", topic_type),
    }
}

async fn handle_order_message(payload: &str, simulator: &Arc<Mutex<VehicleSimulator>>) {
    match serde_json::from_str::<Order>(payload) {
        Ok(order) => {
            simulator.lock().await.process_order(order);
        }
        Err(e) => {
            println!("Error parsing order message: {}", e);
        }
    }
}

async fn handle_instant_actions_message(payload: &str, simulator: &Arc<Mutex<VehicleSimulator>>) {
    match serde_json::from_str::<InstantActions>(payload) {
        Ok(instant_actions) => {
            simulator.lock().await.accept_instant_actions(instant_actions);
        }
        Err(e) => {
            println!("Error parsing instant actions message: {}", e);
        }
    }
}

async fn handle_connection_loss(mqtt_client: &mqtt::AsyncClient) {
    println!("Lost connection. Attempting to reconnect...");
    
    while let Err(err) = mqtt_client.reconnect().await {
        println!("Error reconnecting: {}", err);
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    
    println!("Successfully reconnected to MQTT broker");
} 