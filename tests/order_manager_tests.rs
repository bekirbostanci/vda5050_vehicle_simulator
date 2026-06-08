//! Integration tests for `OrderManager`.
//!
//! Every test uses the public `OrderManager` API. The private rejection
//! reasons (`is_vehicle_close_to_last_released_node`, `can_accept_new_order`)
//! are covered indirectly through `process_order` outcomes.

mod common;

use common::*;
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_state::ActionStatus;
use vda5050_vehicle_simulator::vehicle_simulator::OrderManager;

// -- is_vehicle_ready_for_new_order ------------------------------------

#[test]
fn ready_is_false_when_position_is_none() {
    let state = make_state();
    assert!(!OrderManager::is_vehicle_ready_for_new_order(&state));
}

#[test]
fn ready_is_false_when_position_uninitialized() {
    let mut state = make_state_with_position(0.0, 0.0);
    state.agv_position.as_mut().unwrap().position_initialized = false;
    assert!(!OrderManager::is_vehicle_ready_for_new_order(&state));
}

#[test]
fn ready_is_false_when_node_states_present() {
    let mut state = make_state_with_position(0.0, 0.0);
    state
        .node_states
        .push(make_node_state("n1", 1, 0.0, 0.0, true));
    assert!(!OrderManager::is_vehicle_ready_for_new_order(&state));
}

#[test]
fn ready_is_true_when_initialized_and_idle() {
    let state = make_state_with_position(0.0, 0.0);
    assert!(OrderManager::is_vehicle_ready_for_new_order(&state));
}

// -- next_node_action --------------------------------------------------

#[test]
fn next_node_action_returns_none_when_no_order() {
    let manager = OrderManager::new();
    let state = make_state_with_position(0.0, 0.0);
    assert!(manager.next_node_action(&state).is_none());
}

#[test]
fn next_node_action_returns_first_waiting_action_on_current_node() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let pick = make_pick_action("pick-1", "L-1");
    let drop = make_drop_action("drop-1", Some("L-1"));
    let order = make_order(
        "ord-1",
        0,
        vec![make_node(
            "n1",
            1,
            0.0,
            0.0,
            true,
            vec![pick.clone(), drop.clone()],
        )],
        vec![],
    );
    manager.process_order(order, &mut state);
    state.last_node_sequence_id = 1;

    let next = manager.next_node_action(&state).expect("has next");
    assert_eq!(next.action_id, "pick-1");
}

#[test]
fn next_node_action_skips_already_finished_actions() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let pick = make_pick_action("pick-1", "L-1");
    let drop = make_drop_action("drop-1", Some("L-1"));
    let order = make_order(
        "ord-1",
        0,
        vec![make_node(
            "n1",
            1,
            0.0,
            0.0,
            true,
            vec![pick.clone(), drop.clone()],
        )],
        vec![],
    );
    manager.process_order(order, &mut state);
    state.last_node_sequence_id = 1;
    state.action_states[0].action_status = ActionStatus::Finished;

    let next = manager.next_node_action(&state).expect("has next");
    assert_eq!(next.action_id, "drop-1");
}

#[test]
fn next_node_action_returns_none_when_all_actions_finished() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let pick = make_pick_action("pick-1", "L-1");
    let order = make_order(
        "ord-1",
        0,
        vec![make_node("n1", 1, 0.0, 0.0, true, vec![pick.clone()])],
        vec![],
    );
    manager.process_order(order, &mut state);
    state.last_node_sequence_id = 1;
    state.action_states[0].action_status = ActionStatus::Finished;

    assert!(manager.next_node_action(&state).is_none());
}

// -- has_order ---------------------------------------------------------

#[test]
fn has_order_reflects_acceptance() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    assert!(!manager.has_order());

    let order = make_order(
        "ord-1",
        0,
        vec![make_node("n1", 1, 0.0, 0.0, true, vec![])],
        vec![],
    );
    manager.process_order(order, &mut state);
    assert!(manager.has_order());
}

// -- process_order: new orders -----------------------------------------

#[test]
fn process_order_accepts_new_order_when_vehicle_is_ready() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let order = make_order(
        "ord-1",
        0,
        vec![
            make_node("n1", 1, 0.0, 0.0, true, vec![]),
            make_node("n2", 2, 5.0, 0.0, true, vec![]),
        ],
        vec![make_edge("e1", 1, "n1", "n2", true, vec![])],
    );

    manager.process_order(order, &mut state);

    assert_eq!(state.order_id, "ord-1");
    assert_eq!(state.order_update_id, 0);
    assert_eq!(state.node_states.len(), 2);
    assert_eq!(state.edge_states.len(), 1);
    assert_eq!(state.last_node_sequence_id, 0);
}

#[test]
fn process_order_rejects_new_order_when_position_not_initialized() {
    let mut manager = OrderManager::new();
    let mut state = make_state();
    let order = make_order(
        "ord-1",
        0,
        vec![make_node("n1", 1, 0.0, 0.0, true, vec![])],
        vec![],
    );

    manager.process_order(order, &mut state);

    assert_eq!(state.order_id, "");
    assert!(!manager.has_order());
}

#[test]
fn process_order_rejects_when_vehicle_too_far_from_last_released_node() {
    let mut manager = OrderManager::new();
    // Vehicle at origin, but a previous order has a released node at (5, 0)
    // which is way more than 0.1 away.
    let mut state = make_state_with_position(0.0, 0.0);
    state
        .node_states
        .push(make_node_state("prev", 1, 5.0, 0.0, true));
    let order = make_order(
        "ord-1",
        0,
        vec![make_node("n1", 1, 0.0, 0.0, true, vec![])],
        vec![],
    );

    manager.process_order(order, &mut state);

    assert!(!manager.has_order());
    assert_eq!(state.order_id, "");
}

// -- process_order: updates --------------------------------------------

#[test]
fn process_order_rejects_update_with_lower_or_equal_update_id() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let original = make_order(
        "ord-1",
        5,
        vec![make_node("n1", 1, 0.0, 0.0, true, vec![])],
        vec![],
    );
    manager.process_order(original, &mut state);

    let same = make_order(
        "ord-1",
        5,
        vec![make_node("n2", 2, 1.0, 0.0, true, vec![])],
        vec![],
    );
    manager.process_order(same, &mut state);
    assert_eq!(state.order_update_id, 5);
    assert_eq!(state.node_states[0].node_id, "n1");

    let lower = make_order(
        "ord-1",
        4,
        vec![make_node("n3", 3, 2.0, 0.0, true, vec![])],
        vec![],
    );
    manager.process_order(lower, &mut state);
    assert_eq!(state.order_update_id, 5);
    assert_eq!(state.node_states[0].node_id, "n1");
}

#[test]
fn process_order_accepts_update_with_higher_update_id() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let original = make_order(
        "ord-1",
        1,
        vec![make_node("n1", 1, 0.0, 0.0, true, vec![])],
        vec![],
    );
    manager.process_order(original, &mut state);
    state.last_node_sequence_id = 1;

    let updated = make_order(
        "ord-1",
        2,
        vec![
            make_node("n1", 1, 0.0, 0.0, true, vec![]),
            make_node("n2", 2, 1.0, 0.0, true, vec![]),
        ],
        vec![make_edge("e1", 1, "n1", "n2", true, vec![])],
    );
    manager.process_order(updated, &mut state);

    assert_eq!(state.order_update_id, 2);
    assert_eq!(state.node_states.len(), 2);
    assert_eq!(state.edge_states.len(), 1);
    assert_eq!(state.last_node_sequence_id, 1);
}

#[test]
fn accept_order_registers_action_states_for_node_and_edge_actions() {
    let mut manager = OrderManager::new();
    let mut state = make_state_with_position(0.0, 0.0);
    let pick = make_pick_action("pick-1", "L-1");
    let drop = make_drop_action("drop-1", Some("L-1"));
    let order = make_order(
        "ord-1",
        0,
        vec![
            make_node("n1", 1, 0.0, 0.0, true, vec![pick.clone()]),
            make_node("n2", 2, 1.0, 0.0, true, vec![]),
        ],
        vec![make_edge("e1", 1, "n1", "n2", true, vec![drop.clone()])],
    );

    manager.process_order(order, &mut state);

    assert_eq!(state.action_states.len(), 2);
    let ids: Vec<_> = state.action_states.iter().map(|s| &s.action_id).collect();
    assert!(ids.iter().any(|id| id.as_str() == "pick-1"));
    assert!(ids.iter().any(|id| id.as_str() == "drop-1"));
}
