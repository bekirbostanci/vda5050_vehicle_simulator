//! Shared fixtures for integration tests.
//!
//! Lives under `tests/common/` so Cargo treats it as a private helper module
//! rather than a test crate. Pulled into each integration test file with
//! `mod common;`. Each helper builds a minimal but protocol-valid struct so
//! individual test cases stay focused on the behaviour under test.

#![allow(dead_code)]

use vda5050_vehicle_simulator::protocol::vda5050_common::{AgvPosition, NodePosition};
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_action::{
    Action, ActionParameter, ActionParameterValue, BlockingType,
};
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_instant_actions::InstantActions;
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_order::{Edge, Node, Order};
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_state::{
    ActionState, ActionStatus, BatteryState, EStop, EdgeState, NodeState, OperatingMode,
    SafetyState, State,
};
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_visualization::Visualization;

pub const MANUFACTURER: &str = "TEST";
pub const SERIAL_NUMBER: &str = "TEST-AGV-001";
pub const VERSION: &str = "2.0.0";
pub const MAP_ID: &str = "test_map";

pub fn make_state() -> State {
    State {
        header_id: 0,
        timestamp: String::new(),
        version: VERSION.into(),
        manufacturer: MANUFACTURER.into(),
        serial_number: SERIAL_NUMBER.into(),
        order_id: String::new(),
        order_update_id: 0,
        zone_set_id: None,
        last_node_id: String::new(),
        last_node_sequence_id: 0,
        driving: false,
        paused: None,
        new_base_request: None,
        distance_since_last_node: None,
        operating_mode: OperatingMode::Automatic,
        node_states: vec![],
        edge_states: vec![],
        agv_position: None,
        velocity: None,
        loads: vec![],
        action_states: vec![],
        battery_state: BatteryState {
            battery_charge: 100.0,
            battery_voltage: None,
            battery_health: None,
            charging: false,
            reach: None,
        },
        errors: vec![],
        information: vec![],
        safety_state: SafetyState {
            e_stop: EStop::None,
            field_violation: false,
        },
    }
}

/// State with a fully initialized AGV position at `(x, y)` so order
/// acceptance and navigation paths can run.
pub fn make_state_with_position(x: f32, y: f32) -> State {
    let mut state = make_state();
    state.agv_position = Some(AgvPosition {
        x,
        y,
        theta: 0.0,
        map_id: MAP_ID.into(),
        map_description: None,
        position_initialized: true,
        localization_score: None,
        deviation_range: None,
    });
    state
}

pub fn make_visualization() -> Visualization {
    Visualization {
        header_id: 0,
        timestamp: String::new(),
        version: VERSION.into(),
        manufacturer: MANUFACTURER.into(),
        serial_number: SERIAL_NUMBER.into(),
        agv_position: None,
        velocity: None,
    }
}

pub fn make_action(action_type: &str, action_id: &str) -> Action {
    Action {
        action_type: action_type.into(),
        action_id: action_id.into(),
        action_description: None,
        blocking_type: BlockingType::Hard,
        action_parameters: None,
    }
}

pub fn make_action_with_params(
    action_type: &str,
    action_id: &str,
    params: Vec<(&str, ActionParameterValue)>,
) -> Action {
    Action {
        action_type: action_type.into(),
        action_id: action_id.into(),
        action_description: None,
        blocking_type: BlockingType::Hard,
        action_parameters: Some(
            params
                .into_iter()
                .map(|(key, value)| ActionParameter {
                    key: key.into(),
                    value,
                })
                .collect(),
        ),
    }
}

pub fn make_init_position_action(
    action_id: &str,
    x: f32,
    y: f32,
    theta: f32,
    last_node_id: &str,
) -> Action {
    make_action_with_params(
        "initPosition",
        action_id,
        vec![
            ("x", ActionParameterValue::Float(x)),
            ("y", ActionParameterValue::Float(y)),
            ("theta", ActionParameterValue::Float(theta)),
            ("mapId", ActionParameterValue::Str(MAP_ID.into())),
            (
                "lastNodeId",
                ActionParameterValue::Str(last_node_id.into()),
            ),
        ],
    )
}

pub fn make_pick_action(action_id: &str, load_id: &str) -> Action {
    make_action_with_params(
        "pick",
        action_id,
        vec![("loadId", ActionParameterValue::Str(load_id.into()))],
    )
}

pub fn make_drop_action(action_id: &str, load_id: Option<&str>) -> Action {
    match load_id {
        Some(id) => make_action_with_params(
            "drop",
            action_id,
            vec![("loadId", ActionParameterValue::Str(id.into()))],
        ),
        None => make_action("drop", action_id),
    }
}

pub fn make_node(
    node_id: &str,
    sequence_id: u64,
    x: f32,
    y: f32,
    released: bool,
    actions: Vec<Action>,
) -> Node {
    Node {
        node_id: node_id.into(),
        sequence_id,
        node_description: None,
        released,
        node_position: Some(NodePosition {
            x,
            y,
            theta: Some(0.0),
            allowed_deviation_xy: Some(0.1),
            allowed_deviation_theta: Some(0.1),
            map_id: MAP_ID.into(),
            map_description: None,
        }),
        actions,
    }
}

pub fn make_node_state(
    node_id: &str,
    sequence_id: u64,
    x: f32,
    y: f32,
    released: bool,
) -> NodeState {
    NodeState {
        node_id: node_id.into(),
        sequence_id,
        node_description: None,
        released,
        node_position: Some(NodePosition {
            x,
            y,
            theta: Some(0.0),
            allowed_deviation_xy: Some(0.1),
            allowed_deviation_theta: Some(0.1),
            map_id: MAP_ID.into(),
            map_description: None,
        }),
    }
}

pub fn make_edge(
    edge_id: &str,
    sequence_id: u64,
    start: &str,
    end: &str,
    released: bool,
    actions: Vec<Action>,
) -> Edge {
    Edge {
        edge_id: edge_id.into(),
        sequence_id,
        edge_description: None,
        released,
        start_node_id: start.into(),
        end_node_id: end.into(),
        max_speed: None,
        max_height: None,
        min_height: None,
        orientation: None,
        orientation_type: None,
        direction: None,
        rotation_allowed: None,
        max_rotation_speed: None,
        length: None,
        trajectory: None,
        actions,
    }
}

pub fn make_edge_state(edge_id: &str, sequence_id: u64, released: bool) -> EdgeState {
    EdgeState {
        edge_id: edge_id.into(),
        sequence_id,
        edge_description: None,
        released,
        trajectory: None,
    }
}

pub fn make_order(
    order_id: &str,
    order_update_id: u64,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
) -> Order {
    Order {
        header_id: 0,
        timestamp: String::new(),
        version: VERSION.into(),
        manufacturer: MANUFACTURER.into(),
        serial_number: SERIAL_NUMBER.into(),
        order_id: order_id.into(),
        order_update_id,
        zone_set_id: None,
        nodes,
        edges,
    }
}

pub fn make_instant_actions(actions: Vec<Action>) -> InstantActions {
    InstantActions {
        header_id: 0,
        timestamp: String::new(),
        version: VERSION.into(),
        manufacturer: MANUFACTURER.into(),
        serial_number: SERIAL_NUMBER.into(),
        actions,
    }
}

/// Push an `ActionState` into `state.action_states` mirroring the action's
/// id and type with the given status.
pub fn push_action_state(state: &mut State, action: &Action, status: ActionStatus) {
    state.action_states.push(ActionState {
        action_id: action.action_id.clone(),
        action_type: Some(action.action_type.clone()),
        action_description: action.action_description.clone(),
        action_status: status,
        result_description: None,
    });
}
