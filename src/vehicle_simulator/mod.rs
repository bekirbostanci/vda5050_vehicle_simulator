mod action_handler;
mod instant_actions_manager;
mod navigator;
mod order_manager;

pub use action_handler::ActionHandler;
pub use instant_actions_manager::InstantActionsManager;
pub use navigator::Navigator;
pub use order_manager::OrderManager;

use paho_mqtt as mqtt;
use std::time::Duration;
use tokio::time::sleep;

use crate::config;
use crate::mqtt_utils;
use crate::protocol::vda5050_common::AgvPosition;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_connection::{Connection, ConnectionState};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_instant_actions::InstantActions;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_order::Order;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_state::{
    BatteryState, EStop, OperatingMode, SafetyState, State,
};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_visualization::Visualization;
use crate::utils;

/// MQTT topics derived once at construction time.
struct Topics {
    connection: String,
    state: String,
    visualization: String,
}

/// Top-level vehicle simulator. Acts as a thin facade that owns the protocol
/// payloads and delegates behaviour to focused collaborators:
///
/// - [`ActionHandler`] runs `initPosition` / `pick` / `drop` actions.
/// - [`OrderManager`] tracks the current order.
/// - [`InstantActionsManager`] tracks actions received on the
///   `instantActions` topic.
/// - [`Navigator`] advances the AGV one tick at a time.
pub struct VehicleSimulator {
    topics: Topics,
    connection: Connection,
    pub state: State,
    pub visualization: Visualization,

    action_handler: ActionHandler,
    order_manager: OrderManager,
    instant_actions_manager: InstantActionsManager,
    navigator: Navigator,
}

impl VehicleSimulator {
    pub fn new(config: config::Config) -> Self {
        let base_topic = mqtt_utils::generate_vda_mqtt_base_topic(
            &config.mqtt_broker.vda_interface,
            &config.vehicle.vda_version,
            &config.vehicle.manufacturer,
            &config.vehicle.serial_number,
        );

        let topics = Topics {
            connection: format!("{}/connection", base_topic),
            state: format!("{}/state", base_topic),
            visualization: format!("{}/visualization", base_topic),
        };

        let connection = build_initial_connection(&config);
        let (state, agv_position) = build_initial_state(&config);
        let visualization = build_initial_visualization(&config, &agv_position);

        Self {
            topics,
            connection,
            state,
            visualization,
            action_handler: ActionHandler::new(config.settings.action_time),
            order_manager: OrderManager::new(),
            instant_actions_manager: InstantActionsManager::new(),
            navigator: Navigator::new(config.settings.speed),
        }
    }

    // -- Order / instant action intake -------------------------------------

    pub fn process_order(&mut self, order_request: Order) {
        self.order_manager
            .process_order(order_request, &mut self.state);
    }

    pub fn accept_instant_actions(&mut self, request: InstantActions) {
        self.instant_actions_manager
            .accept(request, &mut self.state);
    }

    pub fn is_vehicle_ready_for_new_order(&self) -> bool {
        OrderManager::is_vehicle_ready_for_new_order(&self.state)
    }

    // -- Simulation tick ---------------------------------------------------

    /// Drives one simulation tick: runs any pending instant actions, then
    /// any waiting node action, and finally advances the AGV's position.
    pub fn update_state(&mut self) {
        if self.action_handler.is_busy() {
            return;
        }

        self.process_instant_actions();

        if !self.order_manager.has_order() {
            return;
        }

        if let Some(action) = self.order_manager.next_node_action(&self.state) {
            println!("Executing action type: {:?}", action.action_type);
            self.action_handler.mark_started();
            self.action_handler
                .execute(&action, &mut self.state, &mut self.visualization);
        }

        self.navigator
            .step(&mut self.state, &mut self.visualization);
    }

    /// Run any instant actions that are still in the `Waiting` state.
    pub fn process_instant_actions(&mut self) {
        let pending = self.instant_actions_manager.pending(&self.state);
        for action in pending {
            self.action_handler
                .execute(&action, &mut self.state, &mut self.visualization);
        }
    }

    // -- MQTT publishing ---------------------------------------------------

    pub async fn publish_connection(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        // Initial "broken" announce so subscribers know the previous session ended.
        let json_broken = serde_json::to_string(&self.connection).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.topics.connection, &json_broken)
            .await
            .unwrap();

        sleep(Duration::from_millis(1000)).await;

        self.connection.header_id += 1;
        self.connection.timestamp = utils::get_timestamp();
        self.connection.connection_state = ConnectionState::Online;

        let json_online = serde_json::to_string(&self.connection).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.topics.connection, &json_online)
            .await
            .unwrap();
    }

    pub async fn publish_visualization(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        self.visualization.header_id += 1;
        self.visualization.timestamp = utils::get_timestamp();

        let payload = serde_json::to_string(&self.visualization).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.topics.visualization, &payload)
            .await
            .unwrap();
    }

    pub async fn publish_state(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        self.state.header_id += 1;
        self.state.timestamp = utils::get_timestamp();

        let payload = serde_json::to_string(&self.state).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.topics.state, &payload)
            .await
            .unwrap();
    }
}

// -- Initial protocol payloads ---------------------------------------------

fn build_initial_connection(config: &config::Config) -> Connection {
    Connection {
        header_id: 0,
        timestamp: utils::get_timestamp(),
        version: config.vehicle.vda_full_version.clone(),
        manufacturer: config.vehicle.manufacturer.clone(),
        serial_number: config.vehicle.serial_number.clone(),
        connection_state: ConnectionState::ConnectionBroken,
    }
}

fn build_initial_state(config: &config::Config) -> (State, AgvPosition) {
    // Spawn each robot inside a small random patch around the origin so a
    // multi-robot run does not stack every AGV on the same point.
    let random_x = rand::random::<f32>() * 5.0 - 2.5;
    let random_y = rand::random::<f32>() * 5.0 - 2.5;

    let agv_position = AgvPosition {
        x: random_x,
        y: random_y,
        position_initialized: false,
        theta: 0.0,
        map_id: config.settings.map_id.clone(),
        deviation_range: None,
        map_description: None,
        localization_score: None,
    };

    let state = State {
        header_id: 0,
        timestamp: utils::get_timestamp(),
        version: config.vehicle.vda_full_version.clone(),
        manufacturer: config.vehicle.manufacturer.clone(),
        serial_number: config.vehicle.serial_number.clone(),
        driving: false,
        distance_since_last_node: None,
        operating_mode: OperatingMode::Automatic,
        node_states: vec![],
        edge_states: vec![],
        last_node_id: String::new(),
        order_id: String::new(),
        order_update_id: 0,
        last_node_sequence_id: 0,
        action_states: vec![],
        information: vec![],
        loads: vec![],
        errors: vec![],
        battery_state: BatteryState {
            battery_charge: 100.0,
            battery_voltage: None,
            battery_health: None,
            charging: false,
            reach: None,
        },
        safety_state: SafetyState {
            e_stop: EStop::None,
            field_violation: false,
        },
        paused: None,
        new_base_request: None,
        agv_position: Some(agv_position.clone()),
        velocity: None,
        zone_set_id: None,
    };

    (state, agv_position)
}

fn build_initial_visualization(
    config: &config::Config,
    agv_position: &AgvPosition,
) -> Visualization {
    Visualization {
        header_id: 0,
        timestamp: utils::get_timestamp(),
        version: config.vehicle.vda_full_version.clone(),
        manufacturer: config.vehicle.manufacturer.clone(),
        serial_number: config.vehicle.serial_number.clone(),
        agv_position: Some(agv_position.clone()),
        velocity: None,
    }
}
