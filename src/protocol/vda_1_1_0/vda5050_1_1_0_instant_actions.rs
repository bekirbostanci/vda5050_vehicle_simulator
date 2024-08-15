use serde::{Deserialize, Serialize};

use crate::protocol::vda_1_1_0::vda5050_1_1_0_action::Action;
use crate::protocol::vda5050_common::HeaderId;

/// Instant actions that the AGV is to execute as soon as they arrive.
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstantActions {
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
    /// Array of actions that need to be performed immediately and are not part of the regular order.
    pub instant_actions: Vec<Action>
}