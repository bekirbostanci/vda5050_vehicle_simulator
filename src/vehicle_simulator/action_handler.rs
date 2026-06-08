use chrono::{DateTime, Utc};

use crate::protocol::vda5050_common::{AgvPosition, BoundingBoxReference, LoadDimensions};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_action::{Action, ActionParameterValue};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_state::{ActionStatus, Load, State};
use crate::protocol::vda_2_0_0::vda5050_2_0_0_visualization::Visualization;

/// Executes VDA5050 actions and tracks blocking-action duration.
///
/// The handler is intentionally state-light: it only knows about the
/// `action_time` budget and the timestamp of the action currently in flight.
/// The mutable protocol state is borrowed at call sites so the top-level
/// `VehicleSimulator` keeps full ownership of `State` and `Visualization`.
pub struct ActionHandler {
    action_time: f32,
    action_start_time: Option<DateTime<Utc>>,
}

impl ActionHandler {
    pub fn new(action_time: f32) -> Self {
        Self {
            action_time,
            action_start_time: None,
        }
    }

    /// Returns true while a previously started action is still within its
    /// configured execution window.
    pub fn is_busy(&self) -> bool {
        let Some(start_time) = self.action_start_time else {
            return false;
        };
        let now = Utc::now().timestamp();
        now < start_time.timestamp() + self.action_time as i64
    }

    /// Marks the start of a blocking action so `is_busy` reports true until
    /// `action_time` seconds elapse.
    pub fn mark_started(&mut self) {
        self.action_start_time = Some(Utc::now());
    }

    /// Executes a single action, transitioning the matching `ActionState`
    /// from `Running` to `Finished`. Unknown action types are logged and
    /// ignored.
    pub fn execute(&self, action: &Action, state: &mut State, visualization: &mut Visualization) {
        let Some(idx) = Self::find_action_state_index(state, &action.action_id) else {
            return;
        };

        state.action_states[idx].action_status = ActionStatus::Running;

        match action.action_type.as_str() {
            "initPosition" => Self::run_init_position(action, state, visualization),
            "pick" => Self::run_pick(action, state),
            "drop" => Self::run_drop(action, state),
            other => println!("Unknown action type: {}", other),
        }

        state.action_states[idx].action_status = ActionStatus::Finished;
    }

    fn find_action_state_index(state: &State, action_id: &str) -> Option<usize> {
        state
            .action_states
            .iter()
            .position(|x| x.action_id == action_id)
    }

    fn run_init_position(action: &Action, state: &mut State, visualization: &mut Visualization) {
        println!("Executing init position action");

        let params = ActionParams::new(action);
        let agv_position = AgvPosition {
            x: params.float("x"),
            y: params.float("y"),
            position_initialized: true,
            theta: params.float("theta"),
            map_id: params.string("mapId"),
            deviation_range: None,
            map_description: None,
            localization_score: None,
        };

        state.last_node_id = params.string("lastNodeId");
        state.agv_position = Some(agv_position.clone());
        visualization.agv_position = Some(agv_position);
    }

    fn run_pick(action: &Action, state: &mut State) {
        println!("Executing pick action");
        let load = ActionParams::new(action).into_load();
        state.loads.push(load);
        println!("Load added. Total loads: {}", state.loads.len());
    }

    fn run_drop(action: &Action, state: &mut State) {
        println!("Executing drop action");

        let load_id = ActionParams::new(action).string("loadId");

        if !load_id.is_empty() {
            state
                .loads
                .retain(|load| load.load_id.as_ref().map_or(true, |id| id != &load_id));
            println!(
                "Load with ID '{}' removed. Total loads: {}",
                load_id,
                state.loads.len()
            );
        } else if !state.loads.is_empty() {
            state.loads.remove(0);
            println!("First load removed. Total loads: {}", state.loads.len());
        }
    }
}

/// Thin view over an `Action`'s parameter list that knows how to coerce
/// stringly-typed VDA5050 parameters into typed values.
struct ActionParams<'a> {
    action: &'a Action,
}

impl<'a> ActionParams<'a> {
    fn new(action: &'a Action) -> Self {
        Self { action }
    }

    fn has(&self, key: &str) -> bool {
        self.action
            .action_parameters
            .as_ref()
            .map_or(false, |params| params.iter().any(|x| x.key == key))
    }

    fn string(&self, key: &str) -> String {
        self.action
            .action_parameters
            .as_ref()
            .and_then(|params| params.iter().find(|x| x.key == key))
            .map(|param| match &param.value {
                ActionParameterValue::Str(s) => s.clone(),
                ActionParameterValue::Int(i) => i.to_string(),
                ActionParameterValue::Float(f) => f.to_string(),
            })
            .unwrap_or_default()
    }

    fn float(&self, key: &str) -> f32 {
        self.action
            .action_parameters
            .as_ref()
            .and_then(|params| params.iter().find(|x| x.key == key))
            .map(|param| match &param.value {
                ActionParameterValue::Str(s) => s.parse::<f32>().unwrap_or(0.0),
                ActionParameterValue::Float(f) => *f,
                ActionParameterValue::Int(i) => *i as f32,
            })
            .unwrap_or(0.0)
    }

    fn optional_float(&self, key: &str) -> Option<f32> {
        if self.has(key) {
            Some(self.float(key))
        } else {
            None
        }
    }

    fn into_load(self) -> Load {
        let load_id = self.string("loadId");
        let load_type = self.string("loadType");
        let load_position = self.string("loadPosition");
        let weight = self.float("weight");

        let bounding_box_reference = self.has("boundingBoxReferenceX").then(|| BoundingBoxReference {
            x: self.float("boundingBoxReferenceX"),
            y: self.float("boundingBoxReferenceY"),
            z: self.float("boundingBoxReferenceZ"),
            theta: self.optional_float("boundingBoxReferenceTheta"),
        });

        let load_dimensions = self.has("loadLength").then(|| LoadDimensions {
            length: self.float("loadLength"),
            width: self.float("loadWidth"),
            height: self.optional_float("loadHeight"),
        });

        Load {
            load_id: empty_to_none(load_id),
            load_type: empty_to_none(load_type),
            load_position: empty_to_none(load_position),
            bounding_box_reference,
            load_dimensions,
            weight: (weight > 0.0).then_some(weight),
        }
    }
}

fn empty_to_none(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
