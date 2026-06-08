//! Integration tests for `Navigator`.
//!
//! Drives `Navigator::step` through its public surface. The private
//! `next_node` helper used to be tested directly; those scenarios are now
//! covered indirectly via `step` (the only caller).

mod common;

use common::*;
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_state::{
    EdgeState, NodeState, State,
};
use vda5050_vehicle_simulator::vehicle_simulator::Navigator;

/// Build a state at `(x, y)` with a sequence of node states and matching
/// edge states between them, all released by default.
fn state_with_path(x: f32, y: f32, nodes: Vec<NodeState>, edges: Vec<EdgeState>) -> State {
    let mut state = make_state_with_position(x, y);
    state.node_states = nodes;
    state.edge_states = edges;
    state
}

#[test]
fn step_is_noop_when_position_is_none() {
    let nav = Navigator::new(0.5);
    let mut state = make_state();
    state.node_states = vec![make_node_state("n1", 1, 5.0, 0.0, true)];
    let mut viz = make_visualization();

    nav.step(&mut state, &mut viz);

    assert_eq!(state.node_states.len(), 1);
    assert!(state.agv_position.is_none());
    assert!(viz.agv_position.is_none());
}

#[test]
fn step_is_noop_when_node_states_is_empty() {
    let nav = Navigator::new(0.5);
    let mut state = make_state_with_position(0.0, 0.0);
    let mut viz = make_visualization();

    nav.step(&mut state, &mut viz);

    let pos = state.agv_position.as_ref().unwrap();
    assert_eq!((pos.x, pos.y), (0.0, 0.0));
}

#[test]
fn step_pops_single_remaining_node_without_moving() {
    let nav = Navigator::new(0.5);
    let mut state = state_with_path(
        0.0,
        0.0,
        vec![make_node_state("n1", 1, 0.0, 0.0, true)],
        vec![],
    );
    let mut viz = make_visualization();

    nav.step(&mut state, &mut viz);

    assert!(state.node_states.is_empty());
    let pos = state.agv_position.as_ref().unwrap();
    assert_eq!((pos.x, pos.y), (0.0, 0.0));
}

#[test]
fn step_does_not_move_when_next_node_is_unreleased() {
    let nav = Navigator::new(0.5);
    let mut state = state_with_path(
        0.0,
        0.0,
        vec![
            make_node_state("n1", 1, 0.0, 0.0, true),
            make_node_state("n2", 2, 10.0, 0.0, false),
        ],
        vec![make_edge_state("e1", 1, true)],
    );
    let mut viz = make_visualization();

    nav.step(&mut state, &mut viz);

    let pos = state.agv_position.as_ref().unwrap();
    assert_eq!((pos.x, pos.y), (0.0, 0.0));
    assert_eq!(state.node_states.len(), 2);
}

#[test]
fn step_advances_position_toward_next_node() {
    let nav = Navigator::new(0.5);
    let mut state = state_with_path(
        0.0,
        0.0,
        vec![
            make_node_state("n1", 1, 0.0, 0.0, true),
            make_node_state("n2", 2, 10.0, 0.0, true),
        ],
        vec![make_edge_state("e1", 1, true)],
    );
    let mut viz = make_visualization();

    nav.step(&mut state, &mut viz);

    let pos = state.agv_position.as_ref().unwrap();
    assert!((pos.x - 0.5).abs() < 1e-5, "x = {}", pos.x);
    assert!(pos.y.abs() < 1e-5, "y = {}", pos.y);
    let vpos = viz.agv_position.as_ref().unwrap();
    assert_eq!((vpos.x, vpos.y), (pos.x, pos.y));
    assert_eq!(state.node_states.len(), 2);
}

#[test]
fn step_pops_node_and_edge_on_arrival_and_updates_last_node() {
    let nav = Navigator::new(0.5);
    // Vehicle is already 0.4 from the next node, so distance < speed + 0.1.
    let mut state = state_with_path(
        9.6,
        0.0,
        vec![
            make_node_state("n1", 1, 0.0, 0.0, true),
            make_node_state("n2", 2, 10.0, 0.0, true),
        ],
        vec![make_edge_state("e1", 1, true)],
    );
    let mut viz = make_visualization();

    nav.step(&mut state, &mut viz);

    assert_eq!(state.node_states.len(), 1);
    assert!(state.edge_states.is_empty());
    assert_eq!(state.last_node_id, "n2");
    assert_eq!(state.last_node_sequence_id, 2);
}

#[test]
fn step_keeps_visualization_in_sync_each_tick() {
    let nav = Navigator::new(0.5);
    let mut state = state_with_path(
        0.0,
        0.0,
        vec![
            make_node_state("n1", 1, 0.0, 0.0, true),
            make_node_state("n2", 2, 5.0, 0.0, true),
        ],
        vec![make_edge_state("e1", 1, true)],
    );
    let mut viz = make_visualization();

    for _ in 0..3 {
        nav.step(&mut state, &mut viz);
        let s = state.agv_position.as_ref().unwrap();
        let v = viz.agv_position.as_ref().unwrap();
        assert_eq!((s.x, s.y, s.theta), (v.x, v.y, v.theta));
    }
}

#[test]
fn step_walks_through_a_three_node_path_and_pops_each_on_arrival() {
    // Indirect coverage of the next_node lookup logic: as the vehicle
    // arrives at each node, last_node_sequence_id should advance and the
    // navigator should move toward the next node.
    let nav = Navigator::new(0.5);
    let mut state = state_with_path(
        0.0,
        0.0,
        vec![
            make_node_state("n1", 1, 0.0, 0.0, true),
            make_node_state("n2", 2, 0.4, 0.0, true),
            make_node_state("n3", 3, 0.8, 0.0, true),
        ],
        vec![
            make_edge_state("e1", 1, true),
            make_edge_state("e2", 2, true),
        ],
    );
    let mut viz = make_visualization();

    // Tick 1: 0.0 -> ~n2 (distance 0.4 < speed + 0.1, so we arrive).
    nav.step(&mut state, &mut viz);
    assert_eq!(state.last_node_sequence_id, 2);
    assert_eq!(state.node_states.len(), 2);

    // Tick 2: n2 -> n3 (also within speed + 0.1).
    nav.step(&mut state, &mut viz);
    assert_eq!(state.last_node_sequence_id, 3);
    assert_eq!(state.node_states.len(), 1);

    // Tick 3: only one node left -> popped without moving.
    nav.step(&mut state, &mut viz);
    assert!(state.node_states.is_empty());
}
