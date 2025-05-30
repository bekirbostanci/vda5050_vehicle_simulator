use serde::{Deserialize, Serialize};

/// Node Action Object
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    ///  Name of action as described in the first column of "Actions and Parameters" Identifies the function of the action.
    pub action_type: String,
    ///  ID to distinguish between multiple actions, either instant or with the same type on the same node/edge.
    pub action_id: String,
    ///  Additional information on the action.
    pub action_description: Option<String>,
    ///  Regulates if the action is allowed to be executed during movement and/or parallel to other actions.
    pub blocking_type: BlockingType,
    ///  Array of actionParameter objects for the indicated action e.g. deviceId, loadId, external triggers.
    pub action_parameters: Option<Vec<ActionParameter>>,
}

/// Regulates if the action is allowed to be executed during movement and/or parallel to other actions.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockingType {
    /// Action can happen in parallel with others, including movement.
    None,
    /// Action can happen simultaneously with others, but not while moving.
    Soft,
    /// No other actions can be performed while this action is running.
    Hard,
}

/// ActionParameter Object
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionParameter {
    /// The key of the action parameter. For example, duration, direction, signal.
    pub key: String,
    /// The value of the action parameter. For example: 103.2, "left", true, [1, 2, 3].
    pub value: ActionParameterValue,
}

/// Enum to represent the possible types for the value of an action parameter.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ActionParameterValue {
    /// Represents an integer value.
    Int(i32),
    Float(f32),
    /// Represents a string value.
    Str(String),
}