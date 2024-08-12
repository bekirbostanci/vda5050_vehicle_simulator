use serde::Serialize;
use std::option::Option;
use crate::protocol::vda5050_common::{AgvPosition, HeaderId, Velocity};

/// AGV position and/or velocity for visualization purposes. Can be published at a higher rate if wanted. Since bandwidth may be expensive depening on the update rate for this topic, all fields are optional.
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Visualization {
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
    /// Current position of the AGV on the map. Optional: Can only be omitted for AGVs without the capability to localize themselves, e.g. line guided AGVs.
    pub agv_position: Option<AgvPosition>,
    /// The AGVs velocity in vehicle coordinates.
    pub velocity: Option<Velocity>
}