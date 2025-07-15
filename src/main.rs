use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

mod config;
mod mqtt_utils;
mod protocol;
mod utils;
mod vehicle_simulator;
mod mqtt_handler;

use vehicle_simulator::VehicleSimulator;
use mqtt_handler::{subscribe_vda_messages, publish_vda_messages};

#[tokio::main]
async fn main() {
    let config = crate::config::get_config();

    for robot_index in 0..config.settings.robot_count {
        spawn_vehicle_simulator(config.clone(), robot_index).await;
    }

    // Keep the main thread alive
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn spawn_vehicle_simulator(config: config::Config, robot_index: u32) {
    // Create vehicle-specific configuration
    let mut vehicle_config = config.clone();
    vehicle_config.vehicle.serial_number = format!("{}{}", config.vehicle.serial_number, robot_index + 1);
    
    // Create and share vehicle simulator
    let vehicle_simulator = VehicleSimulator::new(vehicle_config.clone());
    let shared_simulator = Arc::new(Mutex::new(vehicle_simulator));
    
    // Clone for async tasks
    let simulator_for_subscribe = Arc::clone(&shared_simulator);
    let simulator_for_publish = Arc::clone(&shared_simulator);

    // Spawn MQTT subscription task
    tokio::spawn(subscribe_vda_messages(
        vehicle_config,
        simulator_for_subscribe,
    ));

    // Spawn MQTT publishing task
    tokio::spawn(publish_vda_messages(
        simulator_for_publish,
        config.settings.state_frequency,
        config.settings.visualization_frequency,
    ));
}
