use chrono::{DateTime, Utc};
use paho_mqtt as mqtt;
use std::time::Duration;
use tokio::time::sleep;

use crate::config;
use crate::mqtt_utils;
use crate::utils;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_action::{Action, ActionParameterValue};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_connection::{Connection, ConnectionState};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_state::{State, ActionState, ActionStatus, NodeState, EdgeState, OperatingMode, BatteryState, SafetyState, EStop};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_visualization::Visualization;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_order::Order;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_instant_actions::InstantActions;
use crate::protocol::vda5050_common::{AgvPosition, NodePosition};

pub struct VehicleSimulator {
    connection_topic: String,
    connection: Connection,
    state_topic: String,
    state: State,
    visualization_topic: String,
    visualization: Visualization,

    order: Option<Order>,
    instant_actions: Option<InstantActions>,

    config: config::Config,
    action_start_time: Option<DateTime<Utc>>,
}

impl VehicleSimulator {
    pub fn new(config: config::Config) -> Self {
        let base_topic = mqtt_utils::generate_vda_mqtt_base_topic(
            &config.mqtt_broker.vda_interface,
            &config.vehicle.vda_version,
            &config.vehicle.manufacturer,
            &config.vehicle.serial_number,
        );

        let connection_topic = format!("{}/connection", base_topic);
        let state_topic = format!("{}/state", base_topic);
        let visualization_topic = format!("{}/visualization", base_topic);

        let connection = Self::create_initial_connection(&config);
        let (state, agv_position) = Self::create_initial_state(&config);
        let visualization = Self::create_initial_visualization(&config, &agv_position);

        Self {
            connection_topic,
            connection,
            state_topic,
            state,
            visualization_topic,
            visualization,
            order: None,
            instant_actions: None,
            action_start_time: None,
            config,
        }
    }

    fn create_initial_connection(config: &config::Config) -> Connection {
        Connection {
            header_id: 0,
            timestamp: utils::get_timestamp(),
            version: String::from(&config.vehicle.vda_full_version),
            manufacturer: String::from(&config.vehicle.manufacturer),
            serial_number: String::from(&config.vehicle.serial_number),
            connection_state: ConnectionState::ConnectionBroken,
        }
    }

    fn create_initial_state(config: &config::Config) -> (State, AgvPosition) {
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
            version: String::from(&config.vehicle.vda_full_version),
            manufacturer: String::from(&config.vehicle.manufacturer),
            serial_number: String::from(&config.vehicle.serial_number),
            driving: false,
            distance_since_last_node: None,
            operating_mode: OperatingMode::Automatic,
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

    fn create_initial_visualization(
        config: &config::Config, 
        agv_position: &AgvPosition
    ) -> Visualization {
        Visualization {
            header_id: 0,
            timestamp: utils::get_timestamp(),
            version: String::from(&config.vehicle.vda_full_version),
            manufacturer: String::from(&config.vehicle.manufacturer),
            serial_number: String::from(&config.vehicle.serial_number),
            agv_position: Some(agv_position.clone()),
            velocity: None,
        }
    }

    pub fn run_action(&mut self, action: Action) {
        if let Some(action_state_index) = self.find_action_state_index(&action.action_id) {
            self.state.action_states[action_state_index].action_status = 
                ActionStatus::Running;
            
            match action.action_type.as_str() {
                "initPosition" => self.handle_init_position_action(&action),
                _ => println!("Unknown action type: {}", action.action_type),
            }

            self.state.action_states[action_state_index].action_status = 
                ActionStatus::Finished;
        }
    }

    fn find_action_state_index(&self, action_id: &str) -> Option<usize> {
        self.state.action_states.iter().position(|x| x.action_id == action_id)
    }

    fn handle_init_position_action(&mut self, action: &Action) {
        println!("Executing init position action");
        
        let init_params = self.extract_init_position_parameters(action);
        
        self.state.agv_position = Some(AgvPosition {
            x: init_params.x,
            y: init_params.y,
            position_initialized: true,
            theta: init_params.theta,
            map_id: init_params.map_id,
            deviation_range: None,
            map_description: None,
            localization_score: None,
        });
        
        self.state.last_node_id = init_params.last_node_id;
        self.visualization.agv_position = self.state.agv_position.clone();
    }

    fn extract_init_position_parameters(&self, action: &Action) -> InitPositionParams {
        let extract_float_param = |key: &str| -> f32 {
            action.action_parameters
                .as_ref()
                .and_then(|params| params.iter().find(|x| x.key == key))
                .map(|param| match &param.value {
                    ActionParameterValue::Str(s) => s.parse::<f32>().unwrap_or(0.0),
                    ActionParameterValue::Float(f) => *f,
                    _ => 0.0,
                })
                .unwrap_or(0.0)
        };

        let extract_string_param = |key: &str| -> String {
            action.action_parameters
                .as_ref()
                .and_then(|params| params.iter().find(|x| x.key == key))
                .map(|param| match &param.value {
                    ActionParameterValue::Str(s) => s.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        };

        InitPositionParams {
            x: extract_float_param("x"),
            y: extract_float_param("y"),
            theta: extract_float_param("theta"),
            map_id: extract_string_param("mapId"),
            last_node_id: extract_string_param("lastNodeId"),
        }
    }

    pub async fn publish_connection(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        // Publish initial connection broken state
        let json_connection_broken = serde_json::to_string(&self.connection).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.connection_topic, &json_connection_broken)
            .await
            .unwrap();

        // Wait for connection message to be published
        sleep(Duration::from_millis(1000)).await;

        // Update and publish online state
        self.connection.header_id += 1;
        self.connection.timestamp = utils::get_timestamp();
        self.connection.connection_state = ConnectionState::Online;
        
        let json_connection_online = serde_json::to_string(&self.connection).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.connection_topic, &json_connection_online)
            .await
            .unwrap();
    }

    pub async fn publish_visualization(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        self.visualization.header_id += 1;
        self.visualization.timestamp = utils::get_timestamp();
        
        let json_visualization = serde_json::to_string(&self.visualization).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.visualization_topic, &json_visualization)
            .await
            .unwrap();
    }

    pub async fn publish_state(&mut self, mqtt_cli: &mqtt::AsyncClient) {
        self.state.header_id += 1;
        self.state.timestamp = utils::get_timestamp();
        
        let serialized = serde_json::to_string(&self.state).unwrap();
        mqtt_utils::mqtt_publish(mqtt_cli, &self.state_topic, &serialized)
            .await
            .unwrap();
    }

    pub fn accept_instant_actions(&mut self, instant_action_request: InstantActions) {
        self.instant_actions = Some(instant_action_request);
        
        // Add instant actions to action states
        for instant_action in &self.instant_actions.as_ref().unwrap().actions {
            let action_state = ActionState {
                action_id: instant_action.action_id.clone(),
                action_status: ActionStatus::Waiting,
                action_type: Some(instant_action.action_type.clone()),
                result_description: None,
                action_description: None,
            };
            self.state.action_states.push(action_state);
        }
    }

    pub fn process_order(&mut self, order_request: Order) {
        if order_request.order_id != self.state.order_id {
            self.handle_new_order(order_request);
        } else {
            self.handle_order_update(order_request);
        }
    }

    fn handle_new_order(&mut self, order_request: Order) {
        if !self.can_accept_new_order() {
            return;
        }

        if self.is_vehicle_ready_for_new_order() {
            self.state.action_states.clear();
            self.accept_order(order_request);
        } else {
            self.reject_order("There are active order states or edge states".to_string());
        }
    }

    fn handle_order_update(&mut self, order_request: Order) {
        if order_request.order_update_id > self.state.order_update_id {
            if !self.can_accept_new_order() {
                return;
            }

            self.state.action_states.clear();
            self.accept_order(order_request);
        } else {
            self.reject_order("Order update ID is lower than current".to_string());
        }
    }

    fn can_accept_new_order(&self) -> bool {
        let has_unreleased_nodes = self.state.node_states.iter().any(|node| !node.released);
        
        if has_unreleased_nodes && self.state.node_states[0].sequence_id != self.state.last_node_sequence_id {
            self.reject_order("Vehicle has not arrived at the latest released node".to_string());
            return false;
        }

        if !self.is_vehicle_close_to_last_released_node() {
            self.reject_order("Vehicle is not close enough to last released node".to_string());
            return false;
        }

        true
    }

    fn is_vehicle_close_to_last_released_node(&self) -> bool {
        if let Some(last_released_node) = self.state.node_states.iter().find(|node| node.released) {
            if let (Some(node_position), Some(vehicle_position)) = (&last_released_node.node_position, &self.state.agv_position) {
                let distance = utils::get_distance(
                    vehicle_position.x,
                    vehicle_position.y,
                    node_position.x,
                    node_position.y,
                );
                return distance <= 0.1;
            }
        }
        true
    }

    fn is_vehicle_ready_for_new_order(&self) -> bool {
        self.state.node_states.is_empty() 
            && self.state.edge_states.is_empty() 
            && self.state.agv_position.as_ref().map_or(false, |pos| pos.position_initialized)
    }

    fn accept_order(&mut self, order_request: Order) {
        println!("Accepting order: {}", order_request.order_id);
        self.order = Some(order_request);

        // Update order information
        self.state.order_id = self.order.as_ref().unwrap().order_id.clone();
        self.state.order_update_id = self.order.as_ref().unwrap().order_update_id;
        
        if self.state.order_update_id == 0 {
            self.state.last_node_sequence_id = 0;
        }

        // Clear existing states
        self.state.action_states.clear();
        self.state.node_states.clear();
        self.state.edge_states.clear();

        // Process nodes and edges
        self.process_order_nodes();
        self.process_order_edges();
    }

    fn process_order_nodes(&mut self) {
        let order = self.order.as_ref().unwrap();
        let nodes = order.nodes.clone();
        for node in &nodes {
            let node_state = NodeState {
                node_id: node.node_id.clone(),
                sequence_id: node.sequence_id.clone(),
                released: node.released.clone(),
                node_description: node.node_description.clone(),
                node_position: node.node_position.clone(),
            };
            self.state.node_states.push(node_state);

            // Add node actions
            for action in &node.actions {
                self.add_action_state(action);
            }
        }
    }

    fn process_order_edges(&mut self) {
        let order = self.order.as_ref().unwrap();
        let edges = order.edges.clone();
        for edge in &edges {
            let edge_state = EdgeState {
                edge_id: edge.edge_id.clone(),
                sequence_id: edge.sequence_id.clone(),
                released: edge.released.clone(),
                edge_description: edge.edge_description.clone(),
                trajectory: edge.trajectory.clone(),
            };
            self.state.edge_states.push(edge_state);

            // Add edge actions
            for action in &edge.actions {
                self.add_action_state(action);
            }
        }
    }

    fn add_action_state(&mut self, action: &Action) {
        let action_state = ActionState {
            action_id: action.action_id.clone(),
            action_type: Some(action.action_type.clone()),
            action_description: action.action_description.clone(),
            action_status: ActionStatus::Waiting,
            result_description: None,
        };
        self.state.action_states.push(action_state);
    }

    fn reject_order(&self, reason: String) {
        println!("Rejecting order: {}", reason);
    }

    pub fn update_state(&mut self) {
        if self.is_action_in_progress() {
            return;
        }

        self.process_instant_actions();
        
        if self.order.is_none() {
            return;
        }

        self.process_node_actions();
        self.update_vehicle_position();
    }

    fn is_action_in_progress(&self) -> bool {
        if let Some(start_time) = self.action_start_time {
            let current_time = chrono::Utc::now().timestamp();
            let action_duration = self.config.settings.action_time as i64;
            current_time < start_time.timestamp() + action_duration
        } else {
            false
        }
    }

    fn process_instant_actions(&mut self) {
        if let Some(instant_actions) = &self.instant_actions {
            let actions = instant_actions.actions.clone();
            for action in actions {
                if let Some(action_state) = self.state.action_states.iter().find(|state| state.action_id == action.action_id) {
                    if action_state.action_status == ActionStatus::Waiting {
                        self.run_action(action);
                    }
                }
            }
        }
    }

    fn process_node_actions(&mut self) {
        if let Some(order_last_node_index) = self.find_order_last_node_index() {
            let node_actions = &self.order.as_ref().unwrap().nodes[order_last_node_index].actions;
            
            if !node_actions.is_empty() {
                for action_state in &mut self.state.action_states {
                    for check_action in node_actions {
                        if action_state.action_id == check_action.action_id 
                            && action_state.action_status == ActionStatus::Waiting {
                            println!("Executing action type: {:?}", action_state.action_type);
                            action_state.action_status = ActionStatus::Finished;
                            self.action_start_time = Some(chrono::Utc::now());
                            return;
                        }
                    }
                }
            }
        }
    }

    fn find_order_last_node_index(&self) -> Option<usize> {
        self.order
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .position(|node| node.sequence_id == self.state.last_node_sequence_id)
    }

    fn update_vehicle_position(&mut self) {
        if self.state.agv_position.is_none() || self.state.node_states.is_empty() {
            return;
        }

        // Remove completed nodes
        if self.state.node_states.len() == 1 {
            self.state.node_states.remove(0);
            return;
        }

        // Collect data before any mutable operations
        let next_node = match self.get_next_node() {
            Some(node) => node,
            None => return,
        };
        
        if !next_node.released {
            return;
        }

        let vehicle_position = self.state.agv_position.as_ref().unwrap();
        let next_node_position = match next_node.node_position.as_ref() {
            Some(pos) => pos,
            None => return,
        };

        let updated_position = self.calculate_new_position(vehicle_position, next_node_position, &next_node);
        let distance = utils::get_distance(
            vehicle_position.x,
            vehicle_position.y,
            next_node_position.x,
            next_node_position.y,
        );

        let should_arrive = distance < self.config.settings.speed + 0.1;
        let next_node_id = next_node.node_id.clone();
        let next_node_sequence_id = next_node.sequence_id.clone();
        
        // Update vehicle position
        if let Some(agv_pos) = &mut self.state.agv_position {
            agv_pos.x = updated_position.0;
            agv_pos.y = updated_position.1;
            agv_pos.theta = updated_position.2;
        }

        // Update visualization position
        if let Some(agv_pos) = &self.state.agv_position {
            self.visualization.agv_position = Some(agv_pos.clone());
        }

        // Check if reached next node
        if should_arrive {
            if !self.state.node_states.is_empty() {
                self.state.node_states.remove(0);
            }
            if !self.state.edge_states.is_empty() {
                self.state.edge_states.remove(0);
            }

            self.state.last_node_id = next_node_id;
            self.state.last_node_sequence_id = next_node_sequence_id;
        }
    }

    fn get_next_node(&self) -> Option<&NodeState> {
        let last_node_index = self.state.node_states.iter()
            .position(|node_state| node_state.sequence_id == self.state.last_node_sequence_id)
            .unwrap_or(0);

        if last_node_index >= self.state.node_states.len() - 1 {
            return None;
        }

        Some(&self.state.node_states[last_node_index + 1])
    }

    fn calculate_new_position(
        &self,
        vehicle_position: &AgvPosition,
        next_node_position: &NodePosition,
        next_node: &NodeState,
    ) -> (f32, f32, f32) {
        let next_edge = self.state.edge_states.iter()
            .find(|edge| edge.sequence_id == next_node.sequence_id - 1);

        if let Some(edge) = next_edge {
            if let Some(trajectory) = &edge.trajectory {
                utils::iterate_position_with_trajectory(
                    vehicle_position.x,
                    vehicle_position.y,
                    next_node_position.x,
                    next_node_position.y,
                    self.config.settings.speed,
                    trajectory.clone(),
                )
            } else {
                utils::iterate_position(
                    vehicle_position.x,
                    vehicle_position.y,
                    next_node_position.x,
                    next_node_position.y,
                    self.config.settings.speed,
                )
            }
        } else {
            utils::iterate_position(
                vehicle_position.x,
                vehicle_position.y,
                next_node_position.x,
                next_node_position.y,
                self.config.settings.speed,
            )
        }
    }

}

struct InitPositionParams {
    x: f32,
    y: f32,
    theta: f32,
    map_id: String,
    last_node_id: String,
} 