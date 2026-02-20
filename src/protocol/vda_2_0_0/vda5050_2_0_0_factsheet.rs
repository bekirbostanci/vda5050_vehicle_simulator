use serde::{Deserialize, Serialize};

use crate::protocol::vda5050_common::HeaderId;
use crate::protocol::vda_2_0_0::vda5050_2_0_0_action::BlockingType;

/// The factsheet provides basic information about a specific AGV type series.
/// Published by the AGV shortly after connecting to the broker.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Factsheet {
    /// Header ID of the message, incremented per topic with each sent message.
    pub header_id: HeaderId,
    /// Timestamp in ISO8601 format (YYYY-MM-DDTHH:mm:ss.ffZ).
    pub timestamp: String,
    /// Version of the VDA5050 protocol [Major].[Minor].[Patch], e.g. 2.0.0.
    pub version: String,
    /// Manufacturer of the AGV.
    pub manufacturer: String,
    /// Serial number of the AGV.
    pub serial_number: String,
    /// General classification and capabilities of the AGV type series.
    pub type_specification: TypeSpecification,
    /// Basic physical properties of the AGV.
    pub physical_parameters: PhysicalParameters,
    /// Protocol limitations of the AGV.
    pub protocol_limits: ProtocolLimits,
    /// Supported features of the VDA5050 protocol.
    pub protocol_features: ProtocolFeatures,
    /// Detailed definition of AGV geometry.
    pub agv_geometry: AgvGeometry,
    /// Abstract specification of load capabilities.
    pub load_specification: LoadSpecification,
    /// Vehicle configuration including software versions and network settings.
    pub vehicle_config: Option<VehicleConfig>,
}

/// General classification and capabilities of the AGV type series.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TypeSpecification {
    /// Free text generalized series name as specified by manufacturer.
    pub series_name: String,
    /// Free text human readable description of the AGV type series.
    pub series_description: Option<String>,
    /// Simplified description of AGV kinematics type.
    pub agv_kinematic: AgvKinematic,
    /// Simplified description of AGV class.
    pub agv_class: AgvClass,
    /// Maximum loadable mass in kg.
    pub max_load_mass: f32,
    /// Simplified description of localization type(s).
    pub localization_types: Vec<LocalizationType>,
    /// List of path planning types supported by the AGV, sorted by priority.
    pub navigation_types: Vec<NavigationType>,
}

/// Simplified description of AGV kinematics type.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgvKinematic {
    /// Differential drive.
    Diff,
    /// Omnidirectional drive.
    Omni,
    /// Three-wheel drive.
    Threewheel,
}

/// Simplified description of AGV class.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgvClass {
    Forklift,
    Conveyor,
    Tugger,
    Carrier,
}

/// Simplified description of localization type.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalizationType {
    Natural,
    Reflector,
    Rfid,
    Dmc,
    Spot,
    Grid,
}

/// Path planning type supported by the AGV.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NavigationType {
    PhysicalLineGuided,
    VirtualLineGuided,
    Autonomous,
}

/// Basic physical properties of the AGV.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalParameters {
    /// Minimal controlled continuous speed of the AGV in m/s.
    pub speed_min: f32,
    /// Maximum speed of the AGV in m/s.
    pub speed_max: f32,
    /// Maximum acceleration with maximum load in m/s².
    pub acceleration_max: f32,
    /// Maximum deceleration with maximum load in m/s².
    pub deceleration_max: f32,
    /// Minimum height of AGV in m.
    pub height_min: Option<f32>,
    /// Maximum height of AGV in m.
    pub height_max: f32,
    /// Width of AGV in m.
    pub width: f32,
    /// Length of AGV in m.
    pub length: f32,
}

/// Protocol limitations of the AGV.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolLimits {
    /// Maximum lengths of strings.
    pub max_string_lens: MaxStringLens,
    /// Maximum lengths of arrays.
    pub max_array_lens: MaxArrayLens,
    /// Timing information.
    pub timing: Timing,
}

/// Maximum string lengths supported by the AGV.
/// A zero or absent value means no explicit limit.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaxStringLens {
    /// Maximum MQTT message length.
    pub msg_len: Option<u64>,
    /// Maximum length of the serial-number part in MQTT topics.
    pub topic_serial_len: Option<u64>,
    /// Maximum length of all other parts in MQTT topics.
    pub topic_elem_len: Option<u64>,
    /// Maximum length of ID strings.
    pub id_len: Option<u64>,
    /// If true, ID strings must contain numerical values only.
    pub id_numerical_only: Option<bool>,
    /// Maximum length of ENUM and key strings.
    pub enum_len: Option<u64>,
    /// Maximum length of loadId strings.
    pub load_id_len: Option<u64>,
}

/// Maximum array lengths supported by the AGV.
/// Field names follow the VDA5050 dot-notation convention.
/// A zero or absent value means no explicit limit.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct MaxArrayLens {
    /// Maximum number of nodes per order processable by the AGV.
    #[serde(rename = "order.nodes")]
    pub order_nodes: Option<u64>,
    /// Maximum number of edges per order processable by the AGV.
    #[serde(rename = "order.edges")]
    pub order_edges: Option<u64>,
    /// Maximum number of actions per node processable by the AGV.
    #[serde(rename = "node.actions")]
    pub node_actions: Option<u64>,
    /// Maximum number of actions per edge processable by the AGV.
    #[serde(rename = "edge.actions")]
    pub edge_actions: Option<u64>,
    /// Maximum number of parameters per action processable by the AGV.
    #[serde(rename = "actions.actionsParameters")]
    pub actions_actions_parameters: Option<u64>,
    /// Maximum number of instant actions per message processable by the AGV.
    #[serde(rename = "instantActions")]
    pub instant_actions: Option<u64>,
    /// Maximum number of knots per trajectory processable by the AGV.
    #[serde(rename = "trajectory.knotVector")]
    pub trajectory_knot_vector: Option<u64>,
    /// Maximum number of control points per trajectory processable by the AGV.
    #[serde(rename = "trajectory.controlPoints")]
    pub trajectory_control_points: Option<u64>,
    /// Maximum number of nodeStates sent by the AGV / nodes in the base.
    #[serde(rename = "state.nodeStates")]
    pub state_node_states: Option<u64>,
    /// Maximum number of edgeStates sent by the AGV / edges in the base.
    #[serde(rename = "state.edgeStates")]
    pub state_edge_states: Option<u64>,
    /// Maximum number of load objects sent by the AGV.
    #[serde(rename = "state.loads")]
    pub state_loads: Option<u64>,
    /// Maximum number of actionStates sent by the AGV.
    #[serde(rename = "state.actionStates")]
    pub state_action_states: Option<u64>,
    /// Maximum number of errors sent by the AGV in one state message.
    #[serde(rename = "state.errors")]
    pub state_errors: Option<u64>,
    /// Maximum number of information objects sent by the AGV in one state message.
    #[serde(rename = "state.information")]
    pub state_information: Option<u64>,
    /// Maximum number of error references sent by the AGV for each error.
    #[serde(rename = "error.errorReferences")]
    pub error_error_references: Option<u64>,
    /// Maximum number of info references sent by the AGV for each information object.
    #[serde(rename = "information.infoReferences")]
    pub information_info_references: Option<u64>,
}

/// Timing constraints for communication intervals.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    /// Minimum interval for sending order messages to the AGV in seconds.
    pub min_order_interval: f32,
    /// Minimum interval for sending state messages in seconds.
    pub min_state_interval: f32,
    /// Default interval for sending state messages in seconds.
    pub default_state_interval: Option<f32>,
    /// Default interval for sending messages on the visualization topic in seconds.
    pub visualization_interval: Option<f32>,
}

/// Supported features of the VDA5050 protocol.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolFeatures {
    /// List of supported and/or required optional parameters.
    pub optional_parameters: Vec<OptionalParameter>,
    /// List of all actions with parameters supported by this AGV.
    pub agv_actions: Vec<AgvAction>,
}

/// A supported optional VDA5050 parameter.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionalParameter {
    /// Full name of the optional parameter, e.g. "order.nodes.nodePosition.allowedDeviationTheta".
    pub parameter: String,
    /// Type of support for the optional parameter.
    pub support: ParameterSupport,
    /// Free text description of the optional parameter.
    pub description: Option<String>,
}

/// Type of support for an optional parameter.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParameterSupport {
    /// Optional parameter is supported as specified.
    Supported,
    /// Optional parameter is required for proper AGV operation.
    Required,
}

/// An action supported by this AGV, including its parameters and constraints.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgvAction {
    /// Unique actionType corresponding to action.actionType.
    pub action_type: String,
    /// Free text description of the action.
    pub action_description: Option<String>,
    /// List of allowed scopes for using this action type.
    pub action_scopes: Vec<ActionScope>,
    /// List of parameters for this action. Absent means the action has no parameters.
    pub action_parameters: Option<Vec<AgvActionParameter>>,
    /// Free text description of the action result.
    pub result_description: Option<String>,
    /// Array of possible blocking types for this action.
    pub blocking_types: Option<Vec<BlockingType>>,
}

/// Allowed scope for an action.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionScope {
    /// Usable as an instant action.
    Instant,
    /// Usable on nodes.
    Node,
    /// Usable on edges.
    Edge,
}

/// A parameter for an AGV action as declared in the factsheet.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgvActionParameter {
    /// Key string for the parameter.
    pub key: String,
    /// Data type of the parameter value.
    pub value_data_type: ValueDataType,
    /// Free text description of the parameter.
    pub description: Option<String>,
    /// True if the parameter is optional.
    pub is_optional: Option<bool>,
}

/// Data type of an action parameter value.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValueDataType {
    Bool,
    Number,
    Integer,
    Float,
    String,
    Object,
    Array,
}

/// Detailed definition of AGV geometry.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgvGeometry {
    /// List of wheels, containing wheel arrangement and geometry.
    pub wheel_definitions: Option<Vec<WheelDefinition>>,
    /// List of AGV envelope curves in 2D.
    pub envelopes2d: Option<Vec<Envelope2d>>,
    /// List of AGV envelope curves in 3D.
    pub envelopes3d: Option<Vec<Envelope3d>>,
}

/// Definition of a single wheel including its position and properties.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WheelDefinition {
    /// Wheel type.
    pub r#type: WheelType,
    /// True if the wheel is actively driven.
    pub is_active_driven: bool,
    /// True if the wheel is actively steered.
    pub is_active_steered: bool,
    /// Position of the wheel in the AGV coordinate system.
    pub position: WheelPosition,
    /// Nominal diameter of the wheel in m.
    pub diameter: f32,
    /// Nominal width of the wheel in m.
    pub width: f32,
    /// Nominal displacement of the wheel center to the rotation point in m.
    /// Required for caster wheels; assumed 0 if absent.
    pub center_displacement: Option<f32>,
    /// Free text constraints defined by the manufacturer.
    pub constraints: Option<String>,
}

/// Wheel type classification.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WheelType {
    Drive,
    Caster,
    Fixed,
    Mecanum,
}

/// Position of a wheel in the AGV coordinate system.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WheelPosition {
    /// X-position in AGV coordinate system in m.
    pub x: f32,
    /// Y-position in AGV coordinate system in m.
    pub y: f32,
    /// Orientation of the wheel in AGV coordinate system in rad. Required for fixed wheels.
    pub theta: Option<f32>,
}

/// AGV envelope curve in 2D.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Envelope2d {
    /// Name of the envelope curve set.
    pub set: String,
    /// Envelope curve as a closed, non-self-intersecting x/y polygon.
    pub polygon_points: Vec<PolygonPoint>,
    /// Free text description of the envelope curve set.
    pub description: Option<String>,
}

/// A point in a 2D envelope polygon.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolygonPoint {
    /// X-position of the polygon point in m.
    pub x: f32,
    /// Y-position of the polygon point in m.
    pub y: f32,
}

/// AGV envelope curve in 3D.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Envelope3d {
    /// Name of the envelope curve set.
    pub set: String,
    /// Format of the 3D envelope data, e.g. "DXF".
    pub format: String,
    /// 3D envelope curve data in the format specified by `format`.
    pub data: Option<serde_json::Value>,
    /// URL for downloading the 3D envelope curve data.
    pub url: Option<String>,
    /// Free text description of the envelope curve set.
    pub description: Option<String>,
}

/// Abstract specification of load capabilities.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadSpecification {
    /// Valid values for state.loads[].loadPosition and for the "lhd" action parameter.
    /// If absent or empty, the AGV has no load handling device.
    pub load_positions: Option<Vec<String>>,
    /// List of load sets that can be handled by the AGV.
    pub load_sets: Option<Vec<LoadSet>>,
}

/// A set of load handling parameters for a specific load type.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadSet {
    /// Unique name of the load set, e.g. "DEFAULT".
    pub set_name: String,
    /// Type of load, e.g. "EPAL".
    pub load_type: String,
    /// Load positions this load set is valid for. If absent, valid for all positions.
    pub load_positions: Option<Vec<String>>,
    /// Bounding box reference point in vehicle coordinates.
    pub bounding_box_reference: Option<BoundingBoxReference>,
    /// Dimensions of the load's bounding box.
    pub load_dimensions: Option<LoadDimensions>,
    /// Maximum weight of this load type in kg.
    pub max_weight: Option<f32>,
    /// Minimum allowed height for handling this load type in m.
    pub min_load_handling_height: Option<f32>,
    /// Maximum allowed height for handling this load type in m.
    pub max_load_handling_height: Option<f32>,
    /// Minimum allowed depth for this load type in m.
    pub min_load_handling_depth: Option<f32>,
    /// Maximum allowed depth for this load type in m.
    pub max_load_handling_depth: Option<f32>,
    /// Minimum allowed tilt for this load type in rad.
    pub min_load_handling_tilt: Option<f32>,
    /// Maximum allowed tilt for this load type in rad.
    pub max_load_handling_tilt: Option<f32>,
    /// Maximum allowed speed when carrying this load type in m/s.
    pub agv_speed_limit: Option<f32>,
    /// Maximum allowed acceleration when carrying this load type in m/s².
    pub agv_acceleration_limit: Option<f32>,
    /// Maximum allowed deceleration when carrying this load type in m/s².
    pub agv_deceleration_limit: Option<f32>,
    /// Approximate time for picking up this load type in s.
    pub pick_time: Option<f32>,
    /// Approximate time for dropping this load type in s.
    pub drop_time: Option<f32>,
    /// Free text description of the load handling set.
    pub description: Option<String>,
}

/// Bounding box reference point in vehicle coordinates.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBoxReference {
    /// X-coordinate of the reference point.
    pub x: f32,
    /// Y-coordinate of the reference point.
    pub y: f32,
    /// Z-coordinate of the reference point.
    pub z: f32,
    /// Orientation of the load's bounding box in rad.
    pub theta: Option<f32>,
}

/// Dimensions of a load's bounding box in meters.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadDimensions {
    /// Absolute length of the load's bounding box in m.
    pub length: f32,
    /// Absolute width of the load's bounding box in m.
    pub width: f32,
    /// Absolute height of the load's bounding box in m.
    pub height: Option<f32>,
}

/// Vehicle configuration including software versions and network settings.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VehicleConfig {
    /// Array of hardware and software versions running on the vehicle.
    pub versions: Option<Vec<VehicleConfigVersion>>,
    /// Network configuration of the vehicle.
    pub network: Option<VehicleNetwork>,
}

/// A key-value pair describing a vehicle software or hardware version.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VehicleConfigVersion {
    /// The version key, e.g. "softwareVersion".
    pub key: String,
    /// The version value, e.g. "1.0.0".
    pub value: String,
}

/// Network configuration of the vehicle.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VehicleNetwork {
    /// List of DNS servers used by the vehicle.
    pub dns_servers: Option<Vec<String>>,
    /// A priori assigned IP address used to communicate with the MQTT broker.
    pub local_ip_address: Option<String>,
    /// List of NTP servers used by the vehicle.
    pub ntp_servers: Option<Vec<String>>,
    /// Network subnet mask.
    pub netmask: Option<String>,
    /// Default gateway used by the vehicle.
    pub default_gateway: Option<String>,
}
