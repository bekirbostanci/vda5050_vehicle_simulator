use futures_util::StreamExt;
use paho_mqtt as mqtt;
use tokio::sync::Mutex;
use std::{process, time::Duration};
use std::sync::Arc;
 
mod config;
mod mqtt_utils;
mod protocol;
mod utils;

struct VehicleSimulator {
    connection_topic: String,
    connection: protocol::vda_1_1_0::vda5050_1_1_0_connection::Connection,
    state_topic: String,
    state: protocol::vda_1_1_0::vda5050_1_1_0_state::State,
}

impl VehicleSimulator {
    fn new(config: config::Config) -> VehicleSimulator {
        // Connection
        let connection_topic = protocol::vda_1_1_0::vda5050_1_1_0_connection::connection_topic(
            &config.mqtt_broker.vda_interface,
            &config.vehicle.vda_version,
            &config.vehicle.manufacturer,
            &config.vehicle.serial_number,
        );

        let connection = protocol::vda_1_1_0::vda5050_1_1_0_connection::Connection {
            header_id: 0,
            timestamp: utils::get_timestamp(),
            version: String::from(&config.vehicle.vda_version),
            manufacturer: String::from(&config.vehicle.manufacturer),
            serial_number: String::from(&config.vehicle.serial_number),
            connection_state:
                protocol::vda_1_1_0::vda5050_1_1_0_connection::ConnectionState::ConnectionBroken,
        };

        // State
        let state_topic = protocol::vda_1_1_0::vda5050_1_1_0_state::state_topic(
            &config.mqtt_broker.vda_interface,
            &config.vehicle.vda_version,
            &config.vehicle.manufacturer,
            &config.vehicle.serial_number,
        );
        let agv_position: protocol::vda5050_common::AgvPosition =
            protocol::vda5050_common::AgvPosition {
                x: 0.0,
                y: 0.0,
                position_initialized: true,
                theta: 0.0,
                map_id: String::from("test"),
                deviation_range: None,
                map_description: None,
                localization_score: None,
            };

        let state = protocol::vda_1_1_0::vda5050_1_1_0_state::State {
            header_id: 0,
            timestamp: utils::get_timestamp(),
            version: String::from(config.vehicle.vda_full_version),
            manufacturer: String::from(config.vehicle.manufacturer),
            serial_number: String::from(config.vehicle.serial_number),
            driving: false,
            distance_since_last_node: None,
            operating_mode: protocol::vda_1_1_0::vda5050_1_1_0_state::OperatingMode::Automatic,
            node_states: vec![],
            edge_states: vec![],
            last_node_id: String::from(""),
            order_id: String::from(""),
            order_update_id: 0,
            last_node_sequence_id: 0,
            action_states: vec![],
            information: vec![],
            loads: vec![],
            errors: vec![],
            battery_state: protocol::vda_1_1_0::vda5050_1_1_0_state::BatteryState {
                battery_charge: 0.0,
                battery_voltage: None,
                battery_health: None,
                charging: false,
                reach: None,
            },
            safety_state: protocol::vda_1_1_0::vda5050_1_1_0_state::SafetyState {
                e_stop: protocol::vda_1_1_0::vda5050_1_1_0_state::EStop::None,
                field_violation: false,
            },
            paused: None,
            new_base_request: None,
            agv_position: Some(agv_position),
            velocity: None,
            zone_set_id: None,
        };

        VehicleSimulator {
            connection_topic,
            connection,
            state_topic,
            state,
        }
    }



    async fn publish_connection(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        // VDA Documentation mention first connection state is "ConnectionBroken"
        // After first connection, it should be "Online"
        let json_connection_broken = serde_json::to_string(&self.connection).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.connection_topic, &json_connection_broken)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        self.connection.header_id = self.connection.header_id + 1;
        self.connection.timestamp = utils::get_timestamp();
        self.connection.connection_state =
        protocol::vda_1_1_0::vda5050_1_1_0_connection::ConnectionState::Online;
        let json_connection_online = serde_json::to_string(&self.connection).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.connection_topic, &json_connection_online)
            .await
            .unwrap();
    }

    async fn publish_state(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        self.state.header_id = self.state.header_id + 1;
        self.state.timestamp = utils::get_timestamp();
        let serialized = serde_json::to_string(&self.state).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.state_topic, &serialized)
            .await
            .unwrap();
    }
 
    fn print_hello(&self){
        println!("header: {:?}", self.state.header_id);
    }
}

async fn subscribe_vda_messages(clone: Arc<Mutex<VehicleSimulator>>) {
    let vda5050_interface = "uagv";
    let topics = vec![
        format!("{}/+/+/+/order", vda5050_interface),
        format!("{}/+/+/+/instantActions", vda5050_interface),
        format!("{}/+/+/+/state", vda5050_interface),
    ];

    if topics.is_empty() {
        println!("Error: topic must be specified! via --topic=abc");
        process::exit(-1);
    }
    let qos = vec![1; topics.len()];
    let mut mqtt_cli =
        mqtt::AsyncClient::new(mqtt_utils::mqtt_create_opts()).unwrap_or_else(|e| {
            println!("Error on creating client: {:?}", e);
            process::exit(-1);
        });
    // Get message stream before connecting.
    let mut strm = mqtt_cli.get_stream(25);

    let conn_opts = mqtt::ConnectOptionsBuilder::with_mqtt_version(mqtt::MQTT_VERSION_5)
        .clean_start(true)
        .finalize();

    // Make the connection to the broker
    mqtt_cli.connect(conn_opts).await.unwrap();

    println!("Subscribing to topics: {:?}", topics);
    mqtt_cli.subscribe_many(&topics, &qos).await.unwrap();
    // Just loop on incoming messages.
    println!("Waiting for messages...");
    while let Some(msg_opt) = strm.next().await {
        if let Some(msg) = msg_opt {
            if msg.retained() {
                print!("(R) ");
            }

            clone.lock().await.print_hello();
            let topic = msg.topic();
            println!("Topic: {:?}", topic);
            // let table_name = utils::get_topic_name(topic);
            // let payload = msg.payload();
            // let message = String::from_utf8_lossy(payload).to_string();
            // let robot_name = utils::get_robot_name(topic);
        } else {
            // A "None" means we were disconnected. Try to reconnect...
            println!("Lost connection. Attempting reconnect.");
            while let Err(err) = mqtt_cli.reconnect().await {
                println!("Error reconnecting: {}", err);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn publish_vda_topics(clone: Arc<Mutex<VehicleSimulator>>) {
    let mqtt_cli = mqtt::AsyncClient::new(mqtt_utils::mqtt_create_opts()).unwrap_or_else(|e| {
        println!("Error on creating client: {:?}", e);
        process::exit(-1);
    });
    let conn_opts = mqtt::ConnectOptionsBuilder::with_mqtt_version(mqtt::MQTT_VERSION_5)
        .clean_start(true)
        .finalize();
    // Make the connection to the broker
    mqtt_cli.connect(conn_opts).await.unwrap();
    
    
    clone.lock().await.publish_connection(&mqtt_cli).await;

    loop {
        println!("Publishing state");
        clone.lock().await.publish_state(&mqtt_cli).await;
        tokio::time::sleep(Duration::from_millis(3000)).await;
    }
}

#[tokio::main]
async fn main() {
    let config = crate::config::get_config();

    // tokio::spawn(mqtt_vda_message_subscriber());

    let vehicle_simulator = VehicleSimulator::new(config);
    let shared_vehicle_simulator = Arc::new(Mutex::new(vehicle_simulator));
    let clone_vehicle_simulator = Arc::clone(&shared_vehicle_simulator);
    let clone_vehicle_simulator_2 = Arc::clone(&shared_vehicle_simulator);
    tokio::spawn(subscribe_vda_messages(clone_vehicle_simulator_2));
    tokio::spawn(publish_vda_topics(clone_vehicle_simulator));
    
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
