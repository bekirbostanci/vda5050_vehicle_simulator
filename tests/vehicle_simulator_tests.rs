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
            username: "".to_string(),
            password: "".to_string(),
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

fn create_pick_action(load_id: &str, load_type: &str, weight: f32) -> Action {
    Action {
        action_type: "pick".to_string(),
        action_id: format!("pick_{}", load_id),
        action_description: Some(format!("Pick load {}", load_id)),
        blocking_type: BlockingType::Hard,
        action_parameters: Some(vec![
            ActionParameter {
                key: "loadId".to_string(),
                value: ActionParameterValue::Str(load_id.to_string()),
            },
            ActionParameter {
                key: "loadType".to_string(),
                value: ActionParameterValue::Str(load_type.to_string()),
            },
            ActionParameter {
                key: "weight".to_string(),
                value: ActionParameterValue::Float(weight),
            },
        ]),
    }
}

fn create_pick_action_with_all_params(load_id: &str) -> Action {
    Action {
        action_type: "pick".to_string(),
        action_id: format!("pick_{}", load_id),
        action_description: Some(format!("Pick load {}", load_id)),
        blocking_type: BlockingType::Hard,
        action_parameters: Some(vec![
            ActionParameter {
                key: "loadId".to_string(),
                value: ActionParameterValue::Str(load_id.to_string()),
            },
            ActionParameter {
                key: "loadType".to_string(),
                value: ActionParameterValue::Str("pallet".to_string()),
            },
            ActionParameter {
                key: "loadPosition".to_string(),
                value: ActionParameterValue::Str("front".to_string()),
            },
            ActionParameter {
                key: "weight".to_string(),
                value: ActionParameterValue::Float(50.5),
            },
            ActionParameter {
                key: "boundingBoxReferenceX".to_string(),
                value: ActionParameterValue::Float(0.5),
            },
            ActionParameter {
                key: "boundingBoxReferenceY".to_string(),
                value: ActionParameterValue::Float(0.3),
            },
            ActionParameter {
                key: "boundingBoxReferenceZ".to_string(),
                value: ActionParameterValue::Float(0.2),
            },
            ActionParameter {
                key: "boundingBoxReferenceTheta".to_string(),
                value: ActionParameterValue::Float(0.0),
            },
            ActionParameter {
                key: "loadLength".to_string(),
                value: ActionParameterValue::Float(1.2),
            },
            ActionParameter {
                key: "loadWidth".to_string(),
                value: ActionParameterValue::Float(0.8),
            },
            ActionParameter {
                key: "loadHeight".to_string(),
                value: ActionParameterValue::Float(1.5),
            },
        ]),
    }
}

fn create_drop_action(load_id: &str) -> Action {
    Action {
        action_type: "drop".to_string(),
        action_id: format!("drop_{}", load_id),
        action_description: Some(format!("Drop load {}", load_id)),
        blocking_type: BlockingType::Hard,
        action_parameters: Some(vec![
            ActionParameter {
                key: "loadId".to_string(),
                value: ActionParameterValue::Str(load_id.to_string()),
            },
        ]),
    }
}

fn create_drop_action_without_load_id() -> Action {
    Action {
        action_type: "drop".to_string(),
        action_id: "drop_001".to_string(),
        action_description: Some("Drop first load".to_string()),
        blocking_type: BlockingType::Hard,
        action_parameters: None,
    }
}

#[test]
fn test_pick_action_instant_action() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Initially, no loads
    assert_eq!(simulator.state.loads.len(), 0);
    
    // Create instant actions with pick action
    let pick_action = create_pick_action("LOAD-001", "pallet", 25.5);
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    // Accept instant actions
    simulator.accept_instant_actions(instant_actions);
    
    // Verify action state was added
    assert_eq!(simulator.state.action_states.len(), 1);
    assert_eq!(simulator.state.action_states[0].action_id, "pick_LOAD-001");
    assert_eq!(simulator.state.action_states[0].action_status, ActionStatus::Waiting);
    
    // Process instant actions
    simulator.process_instant_actions();
    
    // Verify action was executed and finished
    assert_eq!(simulator.state.action_states[0].action_status, ActionStatus::Finished);
    
    // Verify load was added to state
    assert_eq!(simulator.state.loads.len(), 1);
    let load = &simulator.state.loads[0];
    assert_eq!(load.load_id, Some("LOAD-001".to_string()));
    assert_eq!(load.load_type, Some("pallet".to_string()));
    assert_eq!(load.weight, Some(25.5));
}

#[test]
fn test_pick_action_with_all_parameters() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Create instant actions with pick action containing all parameters
    let pick_action = create_pick_action_with_all_params("LOAD-002");
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    simulator.accept_instant_actions(instant_actions);
    simulator.process_instant_actions();
    
    // Verify load was added with all parameters
    assert_eq!(simulator.state.loads.len(), 1);
    let load = &simulator.state.loads[0];
    assert_eq!(load.load_id, Some("LOAD-002".to_string()));
    assert_eq!(load.load_type, Some("pallet".to_string()));
    assert_eq!(load.load_position, Some("front".to_string()));
    assert_eq!(load.weight, Some(50.5));
    
    // Verify bounding box reference
    assert!(load.bounding_box_reference.is_some());
    let bbox = load.bounding_box_reference.as_ref().unwrap();
    assert_eq!(bbox.x, 0.5);
    assert_eq!(bbox.y, 0.3);
    assert_eq!(bbox.z, 0.2);
    assert_eq!(bbox.theta, Some(0.0));
    
    // Verify load dimensions
    assert!(load.load_dimensions.is_some());
    let dims = load.load_dimensions.as_ref().unwrap();
    assert_eq!(dims.length, 1.2);
    assert_eq!(dims.width, 0.8);
    assert_eq!(dims.height, Some(1.5));
}

#[test]
fn test_drop_action_instant_action_with_load_id() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // First, pick a load
    let pick_action = create_pick_action("LOAD-003", "box", 10.0);
    let instant_actions_pick = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    simulator.accept_instant_actions(instant_actions_pick);
    simulator.process_instant_actions();
    
    // Verify load was added
    assert_eq!(simulator.state.loads.len(), 1);
    assert_eq!(simulator.state.loads[0].load_id, Some("LOAD-003".to_string()));
    
    // Now drop the load
    let drop_action = create_drop_action("LOAD-003");
    let instant_actions_drop = InstantActions {
        header_id: 2,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![drop_action],
    };
    
    simulator.accept_instant_actions(instant_actions_drop);
    simulator.process_instant_actions();
    
    // Verify load was removed
    assert_eq!(simulator.state.loads.len(), 0);
}

#[test]
fn test_drop_action_instant_action_without_load_id() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // First, pick a load
    let pick_action = create_pick_action("LOAD-004", "box", 15.0);
    let instant_actions_pick = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    simulator.accept_instant_actions(instant_actions_pick);
    simulator.process_instant_actions();
    
    // Verify load was added
    assert_eq!(simulator.state.loads.len(), 1);
    
    // Now drop without specifying loadId (should remove first load)
    let drop_action = create_drop_action_without_load_id();
    let instant_actions_drop = InstantActions {
        header_id: 2,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![drop_action],
    };
    
    simulator.accept_instant_actions(instant_actions_drop);
    simulator.process_instant_actions();
    
    // Verify load was removed
    assert_eq!(simulator.state.loads.len(), 0);
}

#[test]
fn test_multiple_pick_actions() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Pick multiple loads
    let pick_action_1 = create_pick_action("LOAD-005", "pallet", 20.0);
    let pick_action_2 = create_pick_action("LOAD-006", "box", 5.0);
    
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action_1, pick_action_2],
    };
    
    simulator.accept_instant_actions(instant_actions);
    simulator.process_instant_actions();
    
    // Verify both loads were added
    assert_eq!(simulator.state.loads.len(), 2);
    assert!(simulator.state.loads.iter().any(|l| l.load_id == Some("LOAD-005".to_string())));
    assert!(simulator.state.loads.iter().any(|l| l.load_id == Some("LOAD-006".to_string())));
}

#[test]
fn test_drop_specific_load_from_multiple() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Pick multiple loads
    let pick_action_1 = create_pick_action("LOAD-007", "pallet", 30.0);
    let pick_action_2 = create_pick_action("LOAD-008", "box", 8.0);
    
    let instant_actions_pick = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action_1, pick_action_2],
    };
    
    simulator.accept_instant_actions(instant_actions_pick);
    simulator.process_instant_actions();
    
    // Verify both loads were added
    assert_eq!(simulator.state.loads.len(), 2);
    
    // Drop only LOAD-007
    let drop_action = create_drop_action("LOAD-007");
    let instant_actions_drop = InstantActions {
        header_id: 2,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![drop_action],
    };
    
    simulator.accept_instant_actions(instant_actions_drop);
    simulator.process_instant_actions();
    
    // Verify only LOAD-007 was removed, LOAD-008 remains
    assert_eq!(simulator.state.loads.len(), 1);
    assert_eq!(simulator.state.loads[0].load_id, Some("LOAD-008".to_string()));
}

#[test]
fn test_pick_action_as_order_action() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Initialize position first
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
    
    // Create order with pick action on a node
    let pick_action = create_pick_action("LOAD-009", "pallet", 40.0);
    let mut order = create_small_order();
    order.nodes[0].actions.push(pick_action.clone());
    
    // Process order
    simulator.process_order(order);
    
    // Verify action state was added
    assert!(simulator.state.action_states.iter().any(|a| a.action_id == "pick_LOAD-009"));
    
    // Initially, no loads
    assert_eq!(simulator.state.loads.len(), 0);
    
    // Set last_node_sequence_id to match the first node so actions can be processed
    simulator.state.last_node_sequence_id = 1;
    
    // Process node actions (this should execute the pick action)
    simulator.update_state();
    
    // Verify load was added
    assert_eq!(simulator.state.loads.len(), 1);
    assert_eq!(simulator.state.loads[0].load_id, Some("LOAD-009".to_string()));
}

#[test]
fn test_drop_action_as_order_action() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Initialize position first
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
    
    // First, pick a load via instant action
    let pick_action = create_pick_action("LOAD-010", "box", 12.0);
    let instant_actions_pick = InstantActions {
        header_id: 2,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    simulator.accept_instant_actions(instant_actions_pick);
    simulator.process_instant_actions();
    
    // Verify load was added
    assert_eq!(simulator.state.loads.len(), 1);
    
    // Create order with drop action on a node
    let drop_action = create_drop_action("LOAD-010");
    let mut order = create_small_order();
    order.nodes[0].actions.push(drop_action.clone());
    
    // Process order
    simulator.process_order(order);
    
    // Verify action state was added
    assert!(simulator.state.action_states.iter().any(|a| a.action_id == "drop_LOAD-010"));
    
    // Set last_node_sequence_id to match the first node so actions can be processed
    simulator.state.last_node_sequence_id = 1;
    
    // Process node actions (this should execute the drop action)
    simulator.update_state();
    
    // Verify load was removed
    assert_eq!(simulator.state.loads.len(), 0);
}

#[test]
fn test_pick_and_drop_sequence() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Pick load
    let pick_action = create_pick_action("LOAD-011", "pallet", 35.0);
    let instant_actions_pick = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    simulator.accept_instant_actions(instant_actions_pick);
    simulator.process_instant_actions();
    
    assert_eq!(simulator.state.loads.len(), 1);
    assert_eq!(simulator.state.loads[0].load_id, Some("LOAD-011".to_string()));
    
    // Drop load
    let drop_action = create_drop_action("LOAD-011");
    let instant_actions_drop = InstantActions {
        header_id: 2,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![drop_action],
    };
    
    simulator.accept_instant_actions(instant_actions_drop);
    simulator.process_instant_actions();
    
    assert_eq!(simulator.state.loads.len(), 0);
    
    // Pick another load
    let pick_action_2 = create_pick_action("LOAD-012", "box", 7.5);
    let instant_actions_pick_2 = InstantActions {
        header_id: 3,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action_2],
    };
    
    simulator.accept_instant_actions(instant_actions_pick_2);
    simulator.process_instant_actions();
    
    assert_eq!(simulator.state.loads.len(), 1);
    assert_eq!(simulator.state.loads[0].load_id, Some("LOAD-012".to_string()));
}

#[test]
fn test_drop_nonexistent_load() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Try to drop a load that doesn't exist
    let drop_action = create_drop_action("NONEXISTENT-LOAD");
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![drop_action],
    };
    
    simulator.accept_instant_actions(instant_actions);
    simulator.process_instant_actions();
    
    // Should not crash, loads should remain empty
    assert_eq!(simulator.state.loads.len(), 0);
}

#[test]
fn test_pick_action_with_minimal_parameters() {
    let config = create_test_config();
    let mut simulator = VehicleSimulator::new(config);
    
    // Create pick action with only loadId
    let pick_action = Action {
        action_type: "pick".to_string(),
        action_id: "pick_minimal".to_string(),
        action_description: None,
        blocking_type: BlockingType::Hard,
        action_parameters: Some(vec![
            ActionParameter {
                key: "loadId".to_string(),
                value: ActionParameterValue::Str("LOAD-MINIMAL".to_string()),
            },
        ]),
    };
    
    let instant_actions = InstantActions {
        header_id: 1,
        timestamp: utils::get_timestamp(),
        version: "2.0.0".to_string(),
        manufacturer: "TEST".to_string(),
        serial_number: "TEST-AGV-001".to_string(),
        actions: vec![pick_action],
    };
    
    simulator.accept_instant_actions(instant_actions);
    simulator.process_instant_actions();
    
    // Verify load was added with minimal parameters
    assert_eq!(simulator.state.loads.len(), 1);
    let load = &simulator.state.loads[0];
    assert_eq!(load.load_id, Some("LOAD-MINIMAL".to_string()));
    assert_eq!(load.load_type, None);
    assert_eq!(load.load_position, None);
    assert_eq!(load.weight, None);
    assert!(load.bounding_box_reference.is_none());
    assert!(load.load_dimensions.is_none());
} 