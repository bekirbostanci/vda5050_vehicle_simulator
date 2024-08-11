use crate::protocol::vda5050_common::HeaderId;
use serde::Serialize;
use std::string::String;

/// AGV connection state reported as a last will message. Has to be sent with retain flag. Once the AGV comes online, it has to send this message on its connect topic, with the connection_state enum set to "ONLINE". The last will message is to be configured with the connection state set to "CONNECTIONBROKEN". Thus, if the AGV disconnects from the broker, master control gets notified via the topic "connection". If the AGV is disconnecting in an orderly fashion (e.g. shutting down, sleeping), the AGV is to publish a message on this topic with the connection_state set to "OFFLINE".
pub fn connection_topic(
    vda_interface: &str,
    vda_version: &str,
    manufacturer: &str,
    serial_number: &str,
) -> String {
    let vda5050_connection_topic = format!(
        "{}/{}/{}/{}/connection",
        vda_interface, vda_version, manufacturer, serial_number
    );

    return vda5050_connection_topic;
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
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
    /// Connection state.
    pub connection_state: ConnectionState,
}

/// Connection state.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionState {
    /// The Connection between AGV and broker is active.
    Online,
    /// The Connection between AGV and broker has gone offline in a coordinated way.
    Offline,
    /// The connection between AGV and broker has unexpectedly ended.
    ConnectionBroken,
}
