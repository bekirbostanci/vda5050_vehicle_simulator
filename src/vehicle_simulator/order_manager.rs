use crate::protocol::vda_2_0_0::vda5050_2_0_0_action::Action;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_order::Order;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_state::{
    ActionState, ActionStatus, EdgeState, NodeState, State,
};
use crate::utils;

/// Owns the lifecycle of the current order for a single vehicle.
///
/// `OrderManager` keeps the only authoritative copy of the most recent
/// `Order` payload. It mutates the shared `State` in-place when it accepts
/// work and exposes thin queries that the `VehicleSimulator` orchestrator
/// uses to drive node-level action execution. Instant actions live in a
/// sibling `InstantActionsManager`.
pub struct OrderManager {
    order: Option<Order>,
}

impl OrderManager {
    pub fn new() -> Self {
        Self { order: None }
    }

    pub fn has_order(&self) -> bool {
        self.order.is_some()
    }

    /// Vehicle has finished any in-flight work and is ready to receive a
    /// brand-new order.
    pub fn is_vehicle_ready_for_new_order(state: &State) -> bool {
        state.node_states.is_empty()
            && state.edge_states.is_empty()
            && state
                .agv_position
                .as_ref()
                .map_or(false, |pos| pos.position_initialized)
    }

    /// Returns the next waiting action attached to the node the vehicle is
    /// currently sitting on, or `None` if there is nothing to run.
    pub fn next_node_action(&self, state: &State) -> Option<Action> {
        let order = self.order.as_ref()?;
        let last_node = order
            .nodes
            .iter()
            .find(|node| node.sequence_id == state.last_node_sequence_id)?;

        last_node.actions.iter().find_map(|action| {
            let action_state = state
                .action_states
                .iter()
                .find(|s| s.action_id == action.action_id)?;
            (action_state.action_status == ActionStatus::Waiting).then(|| action.clone())
        })
    }

    /// Top-level dispatch for an incoming order, distinguishing a brand-new
    /// order id from an update to the current one.
    pub fn process_order(&mut self, request: Order, state: &mut State) {
        if request.order_id != state.order_id {
            self.handle_new_order(request, state);
        } else {
            self.handle_order_update(request, state);
        }
    }

    fn handle_new_order(&mut self, request: Order, state: &mut State) {
        if !self.can_accept_new_order(state) {
            return;
        }

        if Self::is_vehicle_ready_for_new_order(state) {
            state.action_states.clear();
            self.accept_order(request, state);
        } else {
            Self::reject_order("There are active order states or edge states");
        }
    }

    fn handle_order_update(&mut self, request: Order, state: &mut State) {
        if request.order_update_id <= state.order_update_id {
            Self::reject_order("Order update ID is lower than current");
            return;
        }

        if !self.can_accept_new_order(state) {
            return;
        }

        state.action_states.clear();
        self.accept_order(request, state);
    }

    fn can_accept_new_order(&self, state: &State) -> bool {
        let has_unreleased_nodes = state.node_states.iter().any(|node| !node.released);

        if has_unreleased_nodes
            && state.node_states[0].sequence_id != state.last_node_sequence_id
        {
            Self::reject_order("Vehicle has not arrived at the latest released node");
            return false;
        }

        if !Self::is_vehicle_close_to_last_released_node(state) {
            Self::reject_order("Vehicle is not close enough to last released node");
            return false;
        }

        true
    }

    fn is_vehicle_close_to_last_released_node(state: &State) -> bool {
        let Some(last_released_node) = state.node_states.iter().find(|node| node.released) else {
            return true;
        };
        let (Some(node_position), Some(vehicle_position)) =
            (&last_released_node.node_position, &state.agv_position)
        else {
            return true;
        };

        let distance = utils::get_distance(
            vehicle_position.x,
            vehicle_position.y,
            node_position.x,
            node_position.y,
        );
        distance <= 0.1
    }

    fn accept_order(&mut self, request: Order, state: &mut State) {
        println!("Accepting order: {}", request.order_id);

        state.order_id = request.order_id.clone();
        state.order_update_id = request.order_update_id;
        if request.order_update_id == 0 {
            state.last_node_sequence_id = 0;
        }

        state.action_states.clear();
        state.node_states.clear();
        state.edge_states.clear();

        for node in &request.nodes {
            state.node_states.push(NodeState {
                node_id: node.node_id.clone(),
                sequence_id: node.sequence_id,
                released: node.released,
                node_description: node.node_description.clone(),
                node_position: node.node_position.clone(),
            });
            for action in &node.actions {
                state.action_states.push(action_state_for(action));
            }
        }

        for edge in &request.edges {
            state.edge_states.push(EdgeState {
                edge_id: edge.edge_id.clone(),
                sequence_id: edge.sequence_id,
                released: edge.released,
                edge_description: edge.edge_description.clone(),
                trajectory: edge.trajectory.clone(),
            });
            for action in &edge.actions {
                state.action_states.push(action_state_for(action));
            }
        }

        self.order = Some(request);
    }

    fn reject_order(reason: &str) {
        println!("Rejecting order: {}", reason);
    }
}

fn action_state_for(action: &Action) -> ActionState {
    ActionState {
        action_id: action.action_id.clone(),
        action_type: Some(action.action_type.clone()),
        action_description: action.action_description.clone(),
        action_status: ActionStatus::Waiting,
        result_description: None,
    }
}
