use vda5050_vehicle_simulator::{
    vehicle_simulator::VehicleSimulator,
    config::{Config, MqttBrokerConfig, VehicleConfig, Settings},
    protocol::vda_2_0_0::{
        vda5050_2_0_0_action::{Action, ActionParameter, ActionParameterValue, BlockingType},
        vda5050_2_0_0_instant_actions::InstantActions,
        vda5050_2_0_0_order::{Order, Node, Edge},
        vda5050_2_0_0_state::ActionStatus,
    },
    protocol::vda5050_common::NodePosition,
    utils,
};

fn create_test_config() -> Config {
    Config {
        mqtt_broker: MqttBrokerConfig {
            host: "localhost".to_string(),
            port: "1883".to_string(),
            vda_interface: "uagv".to_string(),
        },
        vehicle: VehicleConfig {
            serial_number: "TEST-AGV-001".to_string(),
            manufacturer: "TEST".to_string(),
            vda_version: "v2".to_string(),
            vda_full_version: "2.0.0".to_string(),
        },
        settings: Settings {
            map_id: "test_map".to_string(),
            state_frequency: 1,
            visualization_frequency: 5,
            action_time: 1.0,
            robot_count: 1,
            speed: 0.1,
        },
    }
}

fn create_init_position_action() -> Action {
    Action {
        action_type: "initPosition".to_string(),
        action_id: "init_pos_001".to_string(),
        action_description: Some("Initialize vehicle position".to_string()),
        blocking_type: BlockingType::Hard,
        action_parameters: Some(vec![
            ActionParameter {
                key: "x".to_string(),
                value: ActionParameterValue::Float(10.5),
            },
            ActionParameter {
                key: "y".to_string(),
                value: ActionParameterValue::Float(20.3),
            },
            ActionParameter {
                key: "theta".to_string(),
                value: ActionParameterValue::Float(1.57), // 90 degrees
            },
            ActionParameter {
                key: "mapId".to_string(),
                value: ActionParameterValue::Str("test_map".to_string()),
            },
            ActionParameter {
                key: "lastNodeId".to_string(),
                value: ActionParameterValue::Str("node_001".to_string()),
            },
        ]),
    }
}

fn create_small_order() -> Order {
    Order {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        order_id: "order_001".to_string(),
        order_update_id: 0,
        zone_set_id: None,
        nodes: vec![
            Node {
                node_id: "node_001".to_string(),
                sequence_id: 1,
                node_description: Some("Start node".to_string()),
                released: true,
                node_position: Some(NodePosition {
                    x: 10.5,
                    y: 20.3,
                    theta: Some(1.57),
                    allowed_deviation_xy: Some(0.1),
                    allowed_deviation_theta: Some(0.1),
                    map_id: "test_map".to_string(),
                    map_description: None,
                }),
                actions: vec![],
            },
            Node {
                node_id: "node_002".to_string(),
                sequence_id: 2,
                node_description: Some("End node".to_string()),
                released: true,
                node_position: Some(NodePosition {
                    x: 15.0,
                    y: 25.0,
                    theta: Some(0.0),
                    allowed_deviation_xy: Some(0.1),
                    allowed_deviation_theta: Some(0.1),
                    map_id: "test_map".to_string(),
                    map_description: None,
                }),
                actions: vec![],
            },
        ],
        edges: vec![
            Edge {
                edge_id: "edge_001".to_string(),
                sequence_id: 1,
                edge_description: Some("Path from start to end".to_string()),
                released: true,
                start_node_id: "node_001".to_string(),
                end_node_id: "node_002".to_string(),
                max_speed: Some(0.5),
                max_height: None,
                min_height: None,
                orientation: None,
                orientation_type: None,
                direction: None,
                rotation_allowed: Some(true),
                max_rotation_speed: None,
                length: Some(6.5),
                trajectory: None,
                actions: vec![],
            },
        ],
    }
}

#[test]
fn test_init_position_instant_action() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Create instant actions with initPosition
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![create_init_position_action()],
    };
    
    // Accept instant actions
    simulator.accept_instant_actions(instant_actions);
    
    // Verify action state was added
    assert_eq!(simulator.state.action_states.len(), 1);
    assert_eq!(simulator.state.action_states[0].action_id, "init_pos_001");
    assert_eq!(simulator.state.action_states[0].action_status, ActionStatus::Waiting);
    
    // Process instant actions
    simulator.process_instant_actions();
    
    // Verify action was executed and finished
    assert_eq!(simulator.state.action_states[0].action_status, ActionStatus::Finished);
    
    // Verify position was updated
    let agv_position = simulator.state.agv_position.as_ref().unwrap();
    assert_eq!(agv_position.x, 10.5);
    assert_eq!(agv_position.y, 20.3);
    assert_eq!(agv_position.theta, 1.57);
    assert_eq!(agv_position.map_id, "test_map");
    assert_eq!(agv_position.position_initialized, true);
    assert_eq!(simulator.state.last_node_id, "node_001");
    
    // Verify visualization was updated
    let viz_position = simulator.visualization.agv_position.as_ref().unwrap();
    assert_eq!(viz_position.x, 10.5);
    assert_eq!(viz_position.y, 20.3);
    assert_eq!(viz_position.theta, 1.57);
}

#[test]
fn test_small_order_completion() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // First, initialize position
    let init_action = create_init_position_action();
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![init_action],
    };
    
    simulator.accept_instant_actions(instant_actions);
    simulator.process_instant_actions();
    
    // Verify position is initialized
    assert!(simulator.state.agv_position.as_ref().unwrap().position_initialized);
    
    // Create and process small order
    let order = create_small_order();
    simulator.process_order(order);
    
    // Verify order was accepted
    assert_eq!(simulator.state.order_id, "order_001");
    assert_eq!(simulator.state.order_update_id, 0);
    assert_eq!(simulator.state.node_states.len(), 2);
    assert_eq!(simulator.state.edge_states.len(), 1);
    
    // Verify initial state
    assert_eq!(simulator.state.node_states[0].node_id, "node_001");
    assert_eq!(simulator.state.node_states[0].released, true);
    assert_eq!(simulator.state.node_states[1].node_id, "node_002");
    assert_eq!(simulator.state.node_states[1].released, true);
    
    // Simulate vehicle movement to complete the order
    // First, move to first node (should be already there since position matches)
    simulator.update_state();
    
    // Since the vehicle starts at the same position as the first node,
    // it should immediately be considered to have reached that node
    // The first node should be removed and last_node_sequence_id should be updated
    if simulator.state.node_states.len() == 1 {
        // First node was removed, verify we're now targeting the second node
        assert_eq!(simulator.state.last_node_sequence_id, 1);
    } else {
        // First node is still there, which means we need to move to it
        assert_eq!(simulator.state.node_states.len(), 2);
        assert_eq!(simulator.state.last_node_sequence_id, 0);
    }
    
    // Continue moving to complete the order
    for _ in 0..100 { // Allow enough iterations to reach the target
        simulator.update_state();
        
        // Check if order is completed
        if simulator.state.node_states.is_empty() && simulator.state.edge_states.is_empty() {
            break;
        }
    }
    
    // Verify order completion
    assert!(simulator.state.node_states.is_empty());
    assert!(simulator.state.edge_states.is_empty());
    
    // Verify final position is close to target
    let final_position = simulator.state.agv_position.as_ref().unwrap();
    let target_x = 15.0;
    let target_y = 25.0;
    let distance = utils::get_distance(final_position.x, final_position.y, target_x, target_y);
    assert!(distance < 0.2, "Final position too far from target: distance = {}", distance);
}

#[test]
fn test_init_position_parameter_extraction() {
    let action = create_init_position_action();
    
    // Test parameter extraction (this would need to be made public or tested through public interface)
    // For now, we'll test the action structure
    assert_eq!(action.action_type, "initPosition");
    assert_eq!(action.action_id, "init_pos_001");
    
    let params = action.action_parameters.as_ref().unwrap();
    assert_eq!(params.len(), 5);
    
    // Verify x parameter
    let x_param = params.iter().find(|p| p.key == "x").unwrap();
    match &x_param.value {
        ActionParameterValue::Float(val) => assert_eq!(*val, 10.5),
        _ => panic!("Expected Float value for x parameter"),
    }
    
    // Verify y parameter
    let y_param = params.iter().find(|p| p.key == "y").unwrap();
    match &y_param.value {
        ActionParameterValue::Float(val) => assert_eq!(*val, 20.3),
        _ => panic!("Expected Float value for y parameter"),
    }
    
    // Verify theta parameter
    let theta_param = params.iter().find(|p| p.key == "theta").unwrap();
    match &theta_param.value {
        ActionParameterValue::Float(val) => assert_eq!(*val, 1.57),
        _ => panic!("Expected Float value for theta parameter"),
    }
    
    // Verify mapId parameter
    let map_id_param = params.iter().find(|p| p.key == "mapId").unwrap();
    match &map_id_param.value {
        ActionParameterValue::Str(val) => assert_eq!(val, "test_map"),
        _ => panic!("Expected String value for mapId parameter"),
    }
    
    // Verify lastNodeId parameter
    let last_node_id_param = params.iter().find(|p| p.key == "lastNodeId").unwrap();
    match &last_node_id_param.value {
        ActionParameterValue::Str(val) => assert_eq!(val, "node_001"),
        _ => panic!("Expected String value for lastNodeId parameter"),
    }
}

#[test]
fn test_vehicle_ready_for_new_order() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Initially, vehicle should not be ready (position not initialized)
    assert!(!simulator.is_vehicle_ready_for_new_order());
    
    // Initialize position
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![create_init_position_action()],
    };
    
    simulator.accept_instant_actions(instant_actions);
    simulator.process_instant_actions();
    
    // Now vehicle should be ready
    assert!(simulator.is_vehicle_ready_for_new_order());
}

#[test]
fn test_order_rejection_when_not_ready() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Try to process order without initializing position
    let order = create_small_order();
    simulator.process_order(order);
    
    // Order should not be accepted (no order_id set)
    assert_eq!(simulator.state.order_id, "");
    assert_eq!(simulator.state.order_update_id, 0);
}

#[test]
fn test_action_state_management() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Create instant actions
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![create_init_position_action()],
    };
    
    simulator.accept_instant_actions(instant_actions);
    
    // Verify action state was created
    assert_eq!(simulator.state.action_states.len(), 1);
    let action_state = &simulator.state.action_states[0];
    assert_eq!(action_state.action_id, "init_pos_001");
    assert_eq!(action_state.action_status, ActionStatus::Waiting);
    assert_eq!(action_state.action_type, Some("initPosition".to_string()));
    
    // Process the action
    simulator.process_instant_actions();
    
    // Verify action state was updated
    let action_state = &simulator.state.action_states[0];
    assert_eq!(action_state.action_status, ActionStatus::Finished);
} 