use futures_util::StreamExt;
use paho_mqtt as mqtt;
use std::option::Option;
use std::sync::Arc;
use std::{process, time::Duration};
use tokio::sync::Mutex;

mod config;
mod mqtt_utils;
mod protocol;
mod utils;

struct VehicleSimulator {
    connection_topic: String,
    connection: protocol::vda_1_1_0::vda5050_1_1_0_connection::Connection,
    state_topic: String,
    state: protocol::vda_1_1_0::vda5050_1_1_0_state::State,
    order: Option<protocol::vda_1_1_0::vda5050_1_1_0_order::Order>,
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
            order: None,
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
        self.state_iterate();
        let serialized = serde_json::to_string(&self.state).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.state_topic, &serialized)
            .await
            .unwrap();
    }

    fn order_accept_procedure(&mut self) {
        // Order procedure
        if self.order.is_none() {
            return;
        }

        if self.order.as_ref().unwrap().order_id != self.state.order_id {
            // Empty string (""), if no previous orderId is available.
            if self.state.order_id == "" {
                self.order_accept();
                return;
            }

            // TODO: check action states
            // self.state.action_states.iter().all(|action_state| action_state.action_status != protocol::vda_1_1_0::vda5050_1_1_0_state::ActionStatus::Finished);
            if self.state.node_states.is_empty() == false && self.state.edge_states.is_empty() {
                // Delete action states
                self.state.action_states = vec![];
                self.order_accept();
                return;
            } else {
                self.order_reject();
                return;
            }
        } else {
            if self.order.as_ref().unwrap().order_update_id > self.state.order_update_id {
                if self.state.node_states.is_empty() == false && self.state.edge_states.is_empty() {
                    // Delete action states
                    self.state.action_states = vec![];
                    self.order_accept();
                    return;
                } else {
                    self.order_reject();
                    return;
                }
            } else {
                self.order_reject();
                return;
            }
        }
    }

    fn order_accept(&mut self) {
        // Check order
        if self.order.is_none() {
            return;
        }

        // Set orderId
        // Set orderUpdateId
        let last_node_id = &self.order.as_ref().unwrap().nodes[0].node_id;
        self.state.last_node_id = String::from(last_node_id);
        println!("Order Node: {:?}", last_node_id);
        self.state.order_id = self.order.as_ref().unwrap().order_id.clone();
        self.state.order_update_id = self.order.as_ref().unwrap().order_update_id;

        // Set nodeStates
        // Set edgeStates
        // Set actionStates
        for node in &self.order.as_ref().unwrap().nodes {
            let node_state = protocol::vda_1_1_0::vda5050_1_1_0_state::NodeState {
                node_id: node.node_id.clone(),
                sequence_id: node.sequence_id.clone(),
                released: node.released.clone(),
                node_description: node.node_description.clone(),
                node_position: node.node_position.clone(),
            };
            self.state.node_states.push(node_state);

            // TODO: add action states
            // for action in node.actions {
            //     let action_state = protocol::vda_1_1_0::vda5050_1_1_0_state::ActionState {
            //         action_id: action.action_id.clone(),
            //         action_type: action.action_type.clone(),
            //         action_status: action.action_status.clone(),
            //         action_description: action.action_description.clone(),
            //         result_description: action.result_description.clone()
            //     };
            //     self.state.action_states.push(action_state);
            // }
        }

        for edge in &self.order.as_ref().unwrap().edges {
            let edge_state = protocol::vda_1_1_0::vda5050_1_1_0_state::EdgeState {
                edge_id: edge.edge_id.clone(),
                sequence_id: edge.sequence_id.clone(),
                released: edge.released.clone(),
                edge_description: edge.edge_description.clone(),
                trajectory: None,
            };
            self.state.edge_states.push(edge_state);

            // TODO: add action states
        }
    }

    fn order_reject(&mut self) {}

    fn state_iterate(&mut self) {
        // Check order
        if self.order.is_none() {
            return;
        }
        // Check last node arrived
        if self.state.last_node_sequence_id == self.state.node_states.last().unwrap().sequence_id {
            return;
        }
        // Check vehicle position
        if self.state.agv_position.is_none() {
            return;
        }

        let vehicle_position: protocol::vda5050_common::AgvPosition =
            self.state.agv_position.clone().unwrap();
        let last_node_index = self
            .state
            .node_states
            .iter()
            .position(|node_state| node_state.sequence_id == self.state.last_node_sequence_id);

        // Last node is not found
        if last_node_index.is_none() {
            return;
        }

        let next_node: protocol::vda_1_1_0::vda5050_1_1_0_state::NodeState =
            self.state.node_states[last_node_index.unwrap() + 1].clone();

        let next_node_position: protocol::vda5050_common::NodePosition =
            next_node.node_position.unwrap();
        let updated_vehicle_position = utils::iterate_position(
            vehicle_position.x,
            vehicle_position.y,
            next_node_position.x,
            next_node_position.y,
            0.1,
        );

        self.state.agv_position.as_mut().unwrap().x = updated_vehicle_position.0;
        self.state.agv_position.as_mut().unwrap().y = updated_vehicle_position.1;

        let distance_to_next_node = utils::get_distance(
            vehicle_position.x,
            vehicle_position.y,
            next_node_position.x,
            next_node_position.y,
        );

        if distance_to_next_node < 0.1 {
            self.state.last_node_id = next_node.node_id.clone();
            self.state.last_node_sequence_id = next_node.sequence_id.clone();
        }
    }
}

async fn subscribe_vda_messages(config: config::Config, clone: Arc<Mutex<VehicleSimulator>>) {
    let base_topic = format!(
        "{}/{}/{}/{}",
        config.mqtt_broker.vda_interface,
        config.vehicle.vda_version,
        config.vehicle.manufacturer,
        config.vehicle.serial_number,
    );
    // TODO: with using config
    let topics = vec![
        format!("{}/order", base_topic),
        format!("{}/instantActions", base_topic),
    ];

    if topics.is_empty() {
        println!("Error: topic must be specified! via --topic=abc");
        process::exit(-1);
    }
    let qos = vec![1; topics.len()];
    let mut mqtt_cli = mqtt::AsyncClient::new(mqtt_utils::mqtt_create_opts()).unwrap_or_else(|e| {
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

            let topic = msg.topic();
            println!("Topic: {:?}", topic);
            // let payload = msg.payload();
            // let message = String::from_utf8_lossy(payload).to_string();
            // let robot_name = utils::get_robot_name(topic);
            if utils::get_topic_name(topic) == "order" {
                let payload = msg.payload();
                let message = String::from_utf8_lossy(payload).to_string();

                let order: protocol::vda_1_1_0::vda5050_1_1_0_order::Order =
                    serde_json::from_str(&message).unwrap();
                clone.lock().await.order = Some(order);
                clone.lock().await.order_accept_procedure();
            }
        } else {
            // A "None" means we were disconnected. Try to reconnect...
            println!("Lost connection. Attempting reconnect.");
            while let Err(err) = mqtt_cli.reconnect().await {
                println!("Error reconnecting: {}", err);
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }
}

async fn publish_vda_messages(clone: Arc<Mutex<VehicleSimulator>>) {
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
        clone.lock().await.publish_state(&mqtt_cli).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    let config = crate::config::get_config();
    let clone_config = config.clone();
    // tokio::spawn(mqtt_vda_message_subscriber());

    let vehicle_simulator = VehicleSimulator::new(config);
    let shared_vehicle_simulator = Arc::new(Mutex::new(vehicle_simulator));
    let clone_vehicle_simulator = Arc::clone(&shared_vehicle_simulator);
    let clone_vehicle_simulator_2 = Arc::clone(&shared_vehicle_simulator);
    
    // Spawn tasks and collect their handles
    let vda_subscribe_handle = tokio::spawn(subscribe_vda_messages(
        clone_config,
        clone_vehicle_simulator_2,
    ));

    let vda_publish_handle = tokio::spawn(publish_vda_messages(clone_vehicle_simulator));

    // Wait for both tasks to complete
    let _ = tokio::try_join!(vda_subscribe_handle, vda_publish_handle);
}
