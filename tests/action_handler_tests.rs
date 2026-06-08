//! Integration tests for `ActionHandler`.
//!
//! Tests exercise the public API only. Parameter-coercion edge cases that
//! used to be tested directly against the private `ActionParams` helper are
//! now covered indirectly through `execute` — the round-trip from action
//! parameters to mutated `State` is what actually matters.

mod common;

use common::*;
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_action::ActionParameterValue;
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_state::ActionStatus;
use vda5050_vehicle_simulator::vehicle_simulator::ActionHandler;

// -- ActionHandler busy timer ------------------------------------------

#[test]
fn is_busy_returns_false_initially() {
    let handler = ActionHandler::new(5.0);
    assert!(!handler.is_busy());
}

#[test]
fn is_busy_returns_true_immediately_after_mark_started() {
    let mut handler = ActionHandler::new(5.0);
    handler.mark_started();
    assert!(handler.is_busy());
}

#[test]
fn is_busy_returns_false_when_action_time_already_elapsed() {
    // action_time = 0 means the busy window closes the moment it opens.
    let mut handler = ActionHandler::new(0.0);
    handler.mark_started();
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(!handler.is_busy());
}

// -- ActionHandler::execute dispatch -----------------------------------

#[test]
fn execute_returns_silently_when_action_id_not_in_state() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let action = make_action("pick", "missing");

    handler.execute(&action, &mut state, &mut viz);

    assert!(state.loads.is_empty());
    assert!(state.action_states.is_empty());
}

#[test]
fn execute_unknown_action_type_still_marks_action_finished() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let action = make_action("undefined", "act-1");
    push_action_state(&mut state, &action, ActionStatus::Waiting);

    handler.execute(&action, &mut state, &mut viz);

    assert_eq!(state.action_states[0].action_status, ActionStatus::Finished);
}

// -- initPosition ------------------------------------------------------

#[test]
fn execute_init_position_writes_position_into_state_and_visualization() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let action = make_init_position_action("init-1", 10.5, 20.3, 1.57, "node_001");
    push_action_state(&mut state, &action, ActionStatus::Waiting);

    handler.execute(&action, &mut state, &mut viz);

    let pos = state.agv_position.as_ref().expect("position written");
    assert_eq!(pos.x, 10.5);
    assert_eq!(pos.y, 20.3);
    assert_eq!(pos.theta, 1.57);
    assert_eq!(pos.map_id, MAP_ID);
    assert!(pos.position_initialized);
    assert_eq!(state.last_node_id, "node_001");

    let viz_pos = viz.agv_position.as_ref().expect("viz mirrored");
    assert_eq!(viz_pos.x, 10.5);
    assert_eq!(viz_pos.y, 20.3);
    assert_eq!(state.action_states[0].action_status, ActionStatus::Finished);
}

// -- pick --------------------------------------------------------------

#[test]
fn execute_pick_appends_load_with_basic_params() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let action = make_action_with_params(
        "pick",
        "pick-1",
        vec![
            ("loadId", ActionParameterValue::Str("LOAD-1".into())),
            ("loadType", ActionParameterValue::Str("pallet".into())),
            ("weight", ActionParameterValue::Float(25.5)),
        ],
    );
    push_action_state(&mut state, &action, ActionStatus::Waiting);

    handler.execute(&action, &mut state, &mut viz);

    assert_eq!(state.loads.len(), 1);
    let load = &state.loads[0];
    assert_eq!(load.load_id.as_deref(), Some("LOAD-1"));
    assert_eq!(load.load_type.as_deref(), Some("pallet"));
    assert_eq!(load.weight, Some(25.5));
    assert!(load.bounding_box_reference.is_none());
    assert!(load.load_dimensions.is_none());
}

#[test]
fn execute_pick_with_full_params_extracts_bounding_box_and_dimensions() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let action = make_action_with_params(
        "pick",
        "pick-1",
        vec![
            ("loadId", ActionParameterValue::Str("LOAD-1".into())),
            ("loadPosition", ActionParameterValue::Str("front".into())),
            ("boundingBoxReferenceX", ActionParameterValue::Float(0.5)),
            ("boundingBoxReferenceY", ActionParameterValue::Float(0.3)),
            ("boundingBoxReferenceZ", ActionParameterValue::Float(0.2)),
            ("boundingBoxReferenceTheta", ActionParameterValue::Float(0.0)),
            ("loadLength", ActionParameterValue::Float(1.2)),
            ("loadWidth", ActionParameterValue::Float(0.8)),
            ("loadHeight", ActionParameterValue::Float(1.5)),
        ],
    );
    push_action_state(&mut state, &action, ActionStatus::Waiting);

    handler.execute(&action, &mut state, &mut viz);

    let load = &state.loads[0];
    let bbox = load.bounding_box_reference.as_ref().unwrap();
    assert_eq!((bbox.x, bbox.y, bbox.z, bbox.theta), (0.5, 0.3, 0.2, Some(0.0)));
    let dims = load.load_dimensions.as_ref().unwrap();
    assert_eq!((dims.length, dims.width, dims.height), (1.2, 0.8, Some(1.5)));
    assert_eq!(load.load_position.as_deref(), Some("front"));
}

#[test]
fn execute_pick_with_minimal_params_yields_none_optionals() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let action = make_pick_action("pick-1", "LOAD-1");
    push_action_state(&mut state, &action, ActionStatus::Waiting);

    handler.execute(&action, &mut state, &mut viz);

    let load = &state.loads[0];
    assert_eq!(load.load_id.as_deref(), Some("LOAD-1"));
    assert!(load.load_type.is_none());
    assert!(load.load_position.is_none());
    assert!(load.weight.is_none());
    assert!(load.bounding_box_reference.is_none());
    assert!(load.load_dimensions.is_none());
}

// -- drop --------------------------------------------------------------

#[test]
fn execute_drop_with_load_id_removes_only_matching_load() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();

    let pick_a = make_pick_action("pick-A", "LOAD-A");
    let pick_b = make_pick_action("pick-B", "LOAD-B");
    push_action_state(&mut state, &pick_a, ActionStatus::Waiting);
    push_action_state(&mut state, &pick_b, ActionStatus::Waiting);
    handler.execute(&pick_a, &mut state, &mut viz);
    handler.execute(&pick_b, &mut state, &mut viz);
    assert_eq!(state.loads.len(), 2);

    let drop = make_drop_action("drop-A", Some("LOAD-A"));
    push_action_state(&mut state, &drop, ActionStatus::Waiting);
    handler.execute(&drop, &mut state, &mut viz);

    assert_eq!(state.loads.len(), 1);
    assert_eq!(state.loads[0].load_id.as_deref(), Some("LOAD-B"));
}

#[test]
fn execute_drop_without_load_id_removes_first_load() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();

    let pick_a = make_pick_action("pick-A", "LOAD-A");
    let pick_b = make_pick_action("pick-B", "LOAD-B");
    push_action_state(&mut state, &pick_a, ActionStatus::Waiting);
    push_action_state(&mut state, &pick_b, ActionStatus::Waiting);
    handler.execute(&pick_a, &mut state, &mut viz);
    handler.execute(&pick_b, &mut state, &mut viz);

    let drop = make_drop_action("drop-any", None);
    push_action_state(&mut state, &drop, ActionStatus::Waiting);
    handler.execute(&drop, &mut state, &mut viz);

    assert_eq!(state.loads.len(), 1);
    assert_eq!(state.loads[0].load_id.as_deref(), Some("LOAD-B"));
}

#[test]
fn execute_drop_with_empty_loads_is_a_noop() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let drop = make_drop_action("drop-1", None);
    push_action_state(&mut state, &drop, ActionStatus::Waiting);

    handler.execute(&drop, &mut state, &mut viz);

    assert!(state.loads.is_empty());
    assert_eq!(state.action_states[0].action_status, ActionStatus::Finished);
}

#[test]
fn execute_drop_with_unknown_load_id_keeps_existing_loads() {
    let handler = ActionHandler::new(1.0);
    let mut state = make_state();
    let mut viz = make_visualization();
    let pick = make_pick_action("pick-1", "LOAD-1");
    push_action_state(&mut state, &pick, ActionStatus::Waiting);
    handler.execute(&pick, &mut state, &mut viz);

    let drop = make_drop_action("drop-1", Some("UNKNOWN"));
    push_action_state(&mut state, &drop, ActionStatus::Waiting);
    handler.execute(&drop, &mut state, &mut viz);

    assert_eq!(state.loads.len(), 1);
    assert_eq!(state.loads[0].load_id.as_deref(), Some("LOAD-1"));
}
