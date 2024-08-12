use std::string::String;
use std::vec::Vec;
use std::option::Option;
use serde::{Serialize, Deserialize};

pub type HeaderId = u64;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]

/// Current position of the AGV on the map. Optional: Can only be omitted for AGVs without the capability to localize themselves, e.g. line guided AGVs.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgvPosition {
    /// X-position on the map in reference to the map coordinate system. Precision is up to the specific implementation.
    pub x: f32,
    /// Y-position on the map in reference to the map coordinate system. Precision is up to the specific implementation.
    pub y: f32,
    /// Range: \[-pi..pi\] Orientation of the AGV.
    pub theta: f32,
    /// Unique identification of the map in which the position is referenced. Each map has the same origin of coordinates. When an AGV uses an elevator, e.g. leading from a departure floor to a target floor, it will disappear off the map of the departure floor and spawn in the related lift node on the map of the target floor.
    pub map_id: String,
    /// Additional information on the map.
    pub map_description: Option<String>,
    /// True if the AGVs position is initialized, false, if position is not initialized.
    pub position_initialized: bool,
    /// Describes the quality of the localization and therefore, can be used e.g. by SLAM-AGVs to describe how accurate the current position information is. 0.0: position unknown 1.0: position known Optional for vehicles that cannot estimate their localization score. Only for logging and visualization purposes
    pub localization_score: Option<f32>,
    /// Value for the deviation range of the position in meters. Optional for vehicles that cannot estimate their deviation e.g. grid-based localization. Only for logging and visualization purposes.
    pub deviation_range: Option<f32>
}

/// This point describes the loads position on the AGV in the vehicle coordinates. The bounding_box_reference point is in the middle of the footprint of the load, so length/2 and width/2.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBoxReference {
    /// x-coordinate of the point of reference.
    pub x: f32,
    /// y-coordinate of the point of reference.
    pub y: f32,
    /// z-coordinate of the point of reference.
    pub z: f32,
    /// Orientation of the loads bounding box. Important for tugger trains etc.
    pub theta: Option<f32>
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlPoint {
    /// X coordinate described in the world coordinate system.
    pub x: f32,
    /// Y coordinate described in the world coordinate system.
    pub y: f32,
    /// Range: (0..Infinity). The weight with which this control point pulls on the curve. When not defined, the default will be 1.0.
    pub weight: Option<f32>,
    /// Range: \[-pi..pi\]. Orientation of the AGV on this position of the curve. The orientation is in world coordinates. When not defined the orientation of the AGV will be tangential to the curve.
    pub orientation: Option<f32>
}

/// Dimensions of the load's bounding box in meters.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDimensions {
    /// Absolute length of the loads bounding box in meter.
    pub length: f32,
    /// Absolute width of the loads bounding box in meter.
    pub width: f32,
    /// Absolute height of the loads bounding box in meter. Optional: Set value only if known.
    pub height: Option<f32>
}

/// Node position. The object is defined in chapter 6.6. Optional: master control has this information. Can be sent additionally, e.g. for debugging purposes.
#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePosition {
    /// X coordinate described in the world coordinate system.
    pub x: f32,
    /// Y coordinate described in the world coordinate system.
    pub y: f32,
    /// Range: \[-pi..pi\]. Orientation of the AGV on the node. Optional: vehicle can plan the path by itself. If defined, the AGV has to assume the theta angle on this node. If previous edge disallows rotation, the AGV is to rotate on the node. If following edge has a differing orientation defined but disallows rotation, the AGV is to rotate on the node to the edges desired rotation before entering the edge.
    pub theta: Option<f32>,
    /// Indicates how exact an AGV has to drive over a node in order for it to count as traversed. If = 0: no deviation is allowed (no deviation means within the normal tolerance of the AGV manufacturer). If > 0: allowed deviation-radius in meters. If the AGV passes a node within the deviation-radius, the node is considered to have been traversed.
    pub allowed_deviation_xy: Option<f32>,
    /// Indicates how big the deviation of theta angle can be. The lowest acceptable angle is theta - allowed_deviation_theta and the highest acceptable angle is theta + allowed_deviation_theta. If = 0: no deviation is allowed (no deviation means within the normal tolerance of the AGV manufacturer).
    pub allowed_deviation_theta: Option<f32>,
    /// Unique identification of the map in which the position is referenced.
    /// Each map has the same origin of coordinates. When an AGV uses an elevator,
    /// e.g. leading from a departure floor to a target floor, it will disappear
    /// off the map of the departure floor and spawn in the related lift node on
    /// the map of the target floor.
    pub map_id: String,
    /// Verbose description of the Map.
    pub map_description: Option<String>
}

/// The trajectory is to be communicated as a NURBS and is defined in chapter 6.4. Trajectory segments are from the point where the AGV starts to enter the edge until the point where it reports that the next node was traversed.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trajectory {
    /// Defines the number of control points that influence any given point on the curve. Increasing the degree increases continuity. If not defined, the default value is 1.
    pub degree: i64,
    /// Sequence of parameter values that determine where and how the control points affect the NURBS curve. knot_vector has size of number of control points + degree + 1
    pub knot_vector: Vec<f32>,
    /// List of JSON controlPoint objects defining the control points of the NURBS. This includes the start and end point.
    pub control_points: Vec<ControlPoint>
}

/// The AGVs velocity in vehicle coordinates.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Velocity {
    /// The AGVs velocity in its x direction.
    pub vx: Option<f32>,
    /// The AGVs velocity in its y direction.
    pub vy: Option<f32>,
    /// The AGVs turning speed around its z axis.
    pub omega: Option<f32>
}