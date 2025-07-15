# VDA5050 Robot Simulator

This project is a VDA5050-compliant robot simulator written in Rust. It simulates the behavior of automated guided vehicles (AGVs) following the VDA5050 standard, using an MQTT broker to communicate. The simulator is configurable via a TOML file, and it supports basic customization like vehicle configuration, state update frequency, and more. Also simulator supoorts create multiple simulator at the same time.

## Features

- Simulates AGVs using the VDA5050 standard.
- Communicates with a broker via MQTT.
- Configurable vehicle, map, and simulator settings.
- Supports visualization updates and actions.
- Support trajectory

## Configuration

You can configure the simulator using a `config.toml` file. Below is an example configuration:

```toml
[mqtt_broker]
host = "localhost"                  # MQTT broker address
port = "1883"                        # MQTT broker port
vda_interface = "uagv"               # VDA interface to use

[vehicle]
serial_number = "s1"                 # Serial number of the AGV
manufacturer = "rikeb"               # Manufacturer name
vda_version = "v2"                   # VDA standard version
vda_full_version = "2.0.0"           # Full VDA version

[settings]
map_id = "webots"                    # Map identifier
state_frequency = 1                  # Frequency for state updates (in Hz)
visualization_frequency = 1          # Frequency for visualization updates (in Hz)
action_time = 1.0                    # Action execution time (in seconds)
robot_count = 1                      # Number of robots to simulate
speed = 0.05                         # Robot speed in meters per second
```

### MQTT Broker Section
- **host**: The address of the MQTT broker (default: localhost).
- **port**: The port of the MQTT broker (default: 1883).
- **vda_interface**: The type of VDA interface used.

### Vehicle Section

- **serial_number**: The serial number of the simulated robot.
- **manufacturer**: The name of the robot manufacturer.
- **vda_version**: The version of the VDA standard being used.
- **vda_full_version**: The full version number of the VDA standard.

### Settings Section

- **map_id**: Identifier for the map used in the simulation (e.g., "webots").
- **state_frequency**: Frequency of state updates in Hertz (Hz). Determines how often the robot sends its current state to the broker.
- **visualization_frequency**: Frequency of visualization updates in Hertz (Hz). Controls how often the simulator will send data for visualization purposes.
- **action_time**: The time it takes to complete an action (in seconds). This controls how long each task or action will take for the robot to execute.
- **robot_count**: The number of robots being simulated. This allows you to simulate multiple robots within the same environment.
- **speed**: The speed of the robot in meters per second, which dictates how fast the robot will move in the simulation.

## Requirements

- **Rust**: Ensure that Rust is installed. You can follow the installation instructions [here](https://www.rust-lang.org/tools/install) if you don’t have it installed.
  
- **MQTT Broker**: You'll need an MQTT broker such as [Mosquitto](https://mosquitto.org/). Install and run it on your machine to handle communication between the simulator and the system.

## Getting Started

To set up and run the VDA5050 robot simulator, follow these steps:

1. Build the project using Cargo, Rust’s package manager:

    ```bash
    cargo build --release
    ```

2. Configure the simulator by modifying the `config.toml` file. You can adjust parameters such as the MQTT broker address, vehicle details, and simulation settings.

3. Run the simulator:

    ```bash
    cargo run --release
    ```

## Usage

Once the simulator is running, it will start sending messages to the MQTT broker according to the configuration in the `config.toml` file. You can monitor the robot's state, actions, and other telemetry by subscribing to the relevant MQTT topics using a client or tool such as [MQTT Explorer](https://mqtt-explorer.com/).

To visualize the robot's status and actions, you can adjust the `visualization_frequency` setting in `config.toml`.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.


## Libraries Used

This simulator was built using the [`vda5050-types-rs`](https://github.com/kKdH/vda5050-types-rs) library for handling VDA5050 standard data types and message structures.
