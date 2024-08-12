use serde::Serialize;
use std::option::Option;
use crate::protocol::vda5050_common::{AgvPosition, BoundingBoxReference, HeaderId, LoadDimensions, NodePosition,  Trajectory, Velocity};

/// All encompassing state of the AGV.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    /// header_id of the message. The header_id is defined per topic and incremented by 1 with each sent (but not necessarily received) message.
    pub header_id: HeaderId,
    /// Timestamp (ISO8601, UTC); YYYY-MM-DDTHH:mm:ss.ssZ; e.g. 2017-04-15T11:40:03.12Z
    pub timestamp: String,
    /// Version of the protocol [Major].[Minor].[Patch], e.g. 1.3.2
    pub version: String,
    /// Manufacturer of the AGV
    pub manufacturer: String,
    /// Serial number of the AGV
    pub serial_number: String,
    /// Unique order identification of the current order or the previous finished order. The order_id is kept until a new order is received. Empty string ("") if no previous order_id is available.
    pub order_id: String,
    /// Order Update Identification to identify that an order update has been accepted by the AGV. 0 if no previous order_update_id is available.
    pub order_update_id: u64,
    /// Unique ID of the zone set that the AGV currently uses for path planning. Must be the same as the one used in the order, otherwise the AGV is to reject the order. Optional: If the AGV does not use zones, this field can be omitted.
    pub zone_set_id: Option<String>,
    /// nodeID of last reached node or, if AGV is currently on a node, current node (e. g. node7). Empty string ("") if no last_node_id is available.
    pub last_node_id: String,
    /// sequence_id of the last reached node or, if the AGV is currently on a node, sequence_id of current node. 0 if no last_node_sequence_id is available.
    pub last_node_sequence_id: u64,
    /// True: indicates that the AGV is driving and/or rotating. Other movements of the AGV (e.g. lift movements) are not included here. False: indicates that the AGV is neither driving nor rotating driving: bool,
    pub driving: bool,
    /// True: AGV is currently in a paused state, either because of the push of a physical button on the AGV or because of an instantAction. The AGV can resume the order. False: The AGV is currently not in a paused state.
    pub paused: Option<bool>,
    /// True: AGV is almost at the end of the base and will reduce speed if no new base is transmitted. Trigger for MC to send new base False: no base update required
    pub new_base_request: Option<bool>,
    /// Used by line guided vehicles to indicate the distance it has been driving past the last_node_id. Distance is in meters
    pub distance_since_last_node: Option<f32>,
    /// Current operating mode of the AGV. For additional information, see the table OperatingModes in chapter 6.10.6.
    pub operating_mode: OperatingMode,
    /// Information about the nodes the AGV still has to drive over. Empty list if idle.
    pub node_states: Vec<NodeState>,
    /// Information about the edges the AGV still has to drive over. Empty list if the AGV is idle.
    pub edge_states: Vec<EdgeState>,
    /// Current position of the AGV on the map. Optional: Can only be omitted for AGVs without the capability to localize themselves, e.g. line guided AGVs.
    pub agv_position: Option<AgvPosition>,
    /// The AGVs velocity in vehicle coordinates.
    pub velocity: Option<Velocity>,
    /// Array for information about the loads that an AGV currently carries, if the AGV has any information about them. This array is optional: if an AGV cannot reason about its load state, it shall not send this field. If an empty field is sent, MC is to assume that the AGV can reason about its load state and that the AGV currently does not carry a load.
    pub loads: Vec<Load>,
    /// Contains a list of the current actions and the actions which are yet to be finished. This may include actions from previous nodes that are still in progress. When an action is completed, an updated state message is published with actionStatus set to finished and if applicable with the corresponding resultDescription. The action_states are kept until a new order is received.
    pub action_states: Vec<ActionState>,
    pub battery_state: BatteryState,
    /// Array of error objects. All active errors of the AGV should be in the list. An empty array indicates that the AGV has no active errors.
    pub errors: Vec<Error>,
    /// Array of information objects. An empty array indicates that the AGV has no information. This should only be used for visualization or debugging – it must not be used for logic in master control. Objects are only for visualization/debugging. There's no specification when these objects are deleted.
    pub information: Vec<Information>,
    /// Object that holds information about the safety status
    pub safety_state: SafetyState
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeState {
    /// Unique node identification.
    pub node_id: String,
    /// Sequence id of the node.
    pub sequence_id: u64,
    /// Verbose node description.
    pub node_description: Option<String>,
    /// Node position.
    pub node_position: Option<NodePosition>,
    /// True: indicates that the node is part of the base. False: indicates that the node is part of the horizon.
    pub released: bool
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeState {
    /// Unique edge identification.
    pub edge_id: String,
    /// sequence_id of the edge.
    pub sequence_id: u64,
    /// Verbose Edge description
    pub edge_description: Option<String>,
    /// True: Edge is part of base. False: Edge is part of horizon.
    pub released: bool,
    /// The trajectory is to be communicated as a NURBS and is defined in chapter 6.4. Trajectory segments are from the point where the AGV starts to enter the edge until the point where it reports that the next node was traversed.
    pub trajectory: Option<Trajectory>
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionState {
    /// Unique action_id, e.g. blink_123jdaimoim234
    pub action_id: String,
    /// action_type of the action. Optional: Only for informational or visualization purposes. Order knows the type.
    pub action_type: Option<String>,
    /// Additional information on the action.
    pub action_description: Option<String>,
    /// Action status. WAITING: Action was received by AGV but the node where it triggers was not yet reached or the edge where it is active was not yet entered. INITIALIZING: Action was triggered, preparatory measures are initiated. RUNNING: The action is running. PAUSED: The action is paused because of a pause instantAction or external trigger (pause button on AGV). FINISHED: The action is finished. A result is reported via the result_description. FAILED: Action could not be finished for whatever reason.
    pub action_status: ActionStatus,
    /// Description of the result, e.g. the result of a rfid-read.
    pub result_description: Option<String>
}

/// Status of an Action.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionStatus {
    /// Action was received by AGV but the node where it triggers was not yet reached or the edge where it is active was not yet entered.
    Waiting,
    /// Action was triggered, preparatory measures are initiated.
    Initializing,
    /// The action is paused because of a pause instantAction or external trigger (pause button on AGV).
    Paused,
    /// The action is running.
    Running,
    /// The action is finished. A result is reported via the resultDescription.
    Finished,
    /// Action could not be finished for whatever reason.
    Failed
}

/// Load object that describes the load if the AGV has information about it.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Load {
    /// Unique identification number of the load (e. g. barcode or RFID) Empty field if the AGV can identify the load but didn't identify the load yet. Optional if the AGV has cannot identify the load.
    pub load_id: Option<String>,
    /// Type of load.
    pub load_type: Option<String>,
    /// Indicates which load handling/carrying unit of the AGV is used, e. g. in case the AGV has multiple spots/positions to carry loads. For example: front, back, positionC1, etc. Optional for vehicles with only one load_position.
    pub load_position: Option<String>,
    /// This point describes the loads position on the AGV in the vehicle coordinates. The bounding_box_reference point is in the middle of the footprint of the load, so length/2 and width/2.
    pub bounding_box_reference: Option<BoundingBoxReference>,
    /// Dimensions of the load's bounding box in meters.
    pub load_dimensions: Option<LoadDimensions>,
    /// Weight of load in kg
    pub weight: Option<f32>
}

/// Contains all battery-related information.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryState {
    /// State of Charge in percent as a float value: If AGV only provides values for good or bad battery levels, these will be indicated as 20% (bad) and 80% (good).
    pub battery_charge: f32,
    /// Battery voltage
    pub battery_voltage: Option<f32>,
    /// State of health in percent as an integer within range [0..100]
    pub battery_health: Option<u32>,
    /// If true: Charging in progress. If false: AGV is currently not charging.
    pub charging: bool,
    /// Estimated reach with current State of Charge (in meter as uint32)
    pub reach: Option<f32>
}

/// Current operating mode of the AGV. For additional information, see the table OperatingModes in chapter 6.10.6.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperatingMode {
    Automatic,
    Semiautomatic,
    Manual,
    Service,
    Teachin
}

/// An error object.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    /// Type / name of error.
    pub error_type: String,
    /// Array of references to identify the source of the error (e.g. header_id, order_id, action_id, ...). For additional information see "Best Practice" chapter 7.
    pub error_references: Vec<ErrorReference>,
    /// Verbose description of error.
    pub error_description: Option<String>,
    /// Error level.
    pub error_level: ErrorLevel
}

/// Object that holds the error reference (e.g. order_id, order_update_id, action_id...) as key-value pairs.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorReference {
    /// References the type of reference (e. g. header_id, order_id, action_id, ...).
    pub reference_key: String,
    /// References the value, which belongs to the reference key.
    pub reference_value: String
}

/// Error level.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorLevel {
    /// AGV is ready to start (e.g. maintenance cycle expiration warning).
    Warning,
    /// AGV is not in running condition, user intervention required (e.g. laser scanner is contaminated).
    Fatal
}

/// An information object.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Information {
    /// Type / name of information.
    pub info_type: String,
    /// Array of references.
    pub info_references: Vec<InfoReference>,
    /// Info description.
    pub info_description: Option<String>,
    /// Info level.
    pub info_level: InfoLevel
}

/// Object that holds the info reference (e.g. order_id, order_update_id, action_id...) as key-value pairs.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoReference {
    /// References the type of reference (e.g. header_id, order_id, action_id, ...).
    pub reference_key: String,
    /// References the value, which belongs to the reference key.
    pub reference_value: String
}

/// Info level.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InfoLevel {
    /// Used for visualization.
    Info,
    /// Used for debugging.
    Debug
}

/// Object that holds information about the safety status.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyState {
    /// Acknowledge type of e_stop.
    pub e_stop: EStop,
    /// Protective field violation. true: field is violated. false: field is not violated.
    pub field_violation: bool
}

/// Acknowledge type of e_stop.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EStop {
    /// Auto-acknowledgeable e-stop is activated e.g. by bumper or protective field.
    Autoack,
    /// E-stop has to be acknowledged manually at the vehicle.
    Manual,
    /// Facility e-stop has to be acknowledged remotely.
    Remote,
    /// No e-stop activated.
    None
}