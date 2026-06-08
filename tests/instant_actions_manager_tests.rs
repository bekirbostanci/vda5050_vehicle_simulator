//! Integration tests for `InstantActionsManager`.
//!
//! Tests exercise only the public API: `accept` registers `Waiting`
//! `ActionState` entries on the shared `State`, and `pending` returns the
//! queued `Action` bodies whose state is still `Waiting`.

mod common;

use common::*;
use vda5050_vehicle_simulator::protocol::vda_2_0_0::vda5050_2_0_0_state::ActionStatus;
use vda5050_vehicle_simulator::vehicle_simulator::InstantActionsManager;

#[test]
fn accept_appends_waiting_action_states() {
    let mut manager = InstantActionsManager::new();
    let mut state = make_state();
    let request = make_instant_actions(vec![
        make_action("ping", "a-1"),
        make_action("ping", "a-2"),
    ]);

    manager.accept(request, &mut state);

    assert_eq!(state.action_states.len(), 2);
    assert_eq!(state.action_states[0].action_id, "a-1");
    assert_eq!(state.action_states[1].action_id, "a-2");
    assert!(state
        .action_states
        .iter()
        .all(|s| s.action_status == ActionStatus::Waiting));
}

#[test]
fn pending_is_empty_when_nothing_accepted() {
    let manager = InstantActionsManager::new();
    let state = make_state();
    assert!(manager.pending(&state).is_empty());
}

#[test]
fn pending_returns_only_waiting_entries() {
    let mut manager = InstantActionsManager::new();
    let mut state = make_state();
    let request = make_instant_actions(vec![
        make_action("ping", "a-1"),
        make_action("ping", "a-2"),
    ]);
    manager.accept(request, &mut state);

    state.action_states[0].action_status = ActionStatus::Finished;

    let pending = manager.pending(&state);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].action_id, "a-2");
}

#[test]
fn accept_accumulates_across_multiple_batches() {
    // Sanity check that a second batch does not orphan a still-Waiting
    // action from the first batch: both should still be returned by
    // `pending` until they are executed.
    let mut manager = InstantActionsManager::new();
    let mut state = make_state();

    manager.accept(
        make_instant_actions(vec![make_action("ping", "a-1")]),
        &mut state,
    );
    manager.accept(
        make_instant_actions(vec![make_action("ping", "a-2")]),
        &mut state,
    );

    let pending = manager.pending(&state);
    assert_eq!(pending.len(), 2);
    assert!(pending.iter().any(|a| a.action_id == "a-1"));
    assert!(pending.iter().any(|a| a.action_id == "a-2"));
}
