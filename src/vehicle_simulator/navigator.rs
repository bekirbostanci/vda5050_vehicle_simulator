use crate::protocol::vda5050_common::{AgvPosition, NodePosition};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_state::{EdgeState, NodeState, State};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_visualization::Visualization;
use crate::utils;

/// Drives the AGV one tick along the currently released path.
///
/// The navigator is stateless besides its configured `speed`; a single tick
/// is computed from the borrowed `State` and the result is written back into
/// the same `State` plus the live `Visualization`.
pub struct Navigator {
    speed: f32,
}

impl Navigator {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    /// Advance the vehicle one simulation tick. Pops nodes/edges as they are
    /// reached and keeps `visualization.agv_position` in lockstep with the
    /// authoritative `state.agv_position`.
    pub fn step(&self, state: &mut State, visualization: &mut Visualization) {
        if state.agv_position.is_none() || state.node_states.is_empty() {
            return;
        }

        if state.node_states.len() == 1 {
            state.node_states.remove(0);
            return;
        }

        let Some(next_node) = self.next_node(state).cloned() else {
            return;
        };
        if !next_node.released {
            return;
        }

        let Some(next_node_position) = next_node.node_position.clone() else {
            return;
        };

        let vehicle_position = state.agv_position.as_ref().unwrap().clone();

        let (new_x, new_y, new_theta) = self.calculate_new_position(
            &vehicle_position,
            &next_node_position,
            &next_node,
            &state.edge_states,
        );

        let distance = utils::get_distance(
            vehicle_position.x,
            vehicle_position.y,
            next_node_position.x,
            next_node_position.y,
        );
        let should_arrive = distance < self.speed + 0.1;

        if let Some(agv_pos) = &mut state.agv_position {
            agv_pos.x = new_x;
            agv_pos.y = new_y;
            agv_pos.theta = new_theta;
            visualization.agv_position = Some(agv_pos.clone());
        }

        if should_arrive {
            if !state.node_states.is_empty() {
                state.node_states.remove(0);
            }
            if !state.edge_states.is_empty() {
                state.edge_states.remove(0);
            }
            state.last_node_id = next_node.node_id;
            state.last_node_sequence_id = next_node.sequence_id;
        }
    }

    fn next_node<'a>(&self, state: &'a State) -> Option<&'a NodeState> {
        let last_node_index = state
            .node_states
            .iter()
            .position(|node| node.sequence_id == state.last_node_sequence_id)
            .unwrap_or(0);

        if last_node_index >= state.node_states.len() - 1 {
            return None;
        }

        Some(&state.node_states[last_node_index + 1])
    }

    fn calculate_new_position(
        &self,
        vehicle_position: &AgvPosition,
        next_node_position: &NodePosition,
        next_node: &NodeState,
        edge_states: &[EdgeState],
    ) -> (f32, f32, f32) {
        let next_edge = edge_states
            .iter()
            .find(|edge| edge.sequence_id + 1 == next_node.sequence_id);

        match next_edge.and_then(|edge| edge.trajectory.clone()) {
            Some(trajectory) => utils::iterate_position_with_trajectory(
                vehicle_position.x,
                vehicle_position.y,
                next_node_position.x,
                next_node_position.y,
                self.speed,
                trajectory,
            ),
            None => utils::iterate_position(
                vehicle_position.x,
                vehicle_position.y,
                next_node_position.x,
                next_node_position.y,
                self.speed,
            ),
        }
    }
}
