use crate::protocol::vda_2_0_0::vda5050_2_0_0_action::Action;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_instant_actions::InstantActions;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_state::{ActionState, ActionStatus, State};

/// Owns the queue of instant actions received on the dedicated
/// `instantActions` MQTT topic.
///
/// Instant actions live outside the order lifecycle: they are accepted as
/// they arrive and run as soon as the simulator is idle, regardless of
/// whether an order is in progress. The manager keeps the full `Action`
/// bodies so the simulator can execute them while their matching
/// `ActionState` is still `Waiting`.
pub struct InstantActionsManager {
    queue: Vec<Action>,
}

impl InstantActionsManager {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Registers each incoming action as a `Waiting` `ActionState` and
    /// appends it to the internal queue so its body stays addressable
    /// until execution.
    pub fn accept(&mut self, request: InstantActions, state: &mut State) {
        for action in request.actions {
            state.action_states.push(ActionState {
                action_id: action.action_id.clone(),
                action_status: ActionStatus::Waiting,
                action_type: Some(action.action_type.clone()),
                result_description: None,
                action_description: None,
            });
            self.queue.push(action);
        }
    }

    /// Returns clones of every queued action whose `ActionState` is still
    /// `Waiting`, so the caller can execute them without holding a borrow
    /// on `state`.
    pub fn pending(&self, state: &State) -> Vec<Action> {
        self.queue
            .iter()
            .filter(|action| {
                state
                    .action_states
                    .iter()
                    .find(|s| s.action_id == action.action_id)
                    .map(|s| s.action_status == ActionStatus::Waiting)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}
