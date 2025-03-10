## Grasp Primary Module (GPM)
This repository contains the source code for the primary software module of the Grasp project. This project is still very much in its early stages, but the goal is to create a highly extensible, asynchronous framework for building embedded systems on the Raspberry Pi.

## Purpose
The purpose of the GPM is to manage all systems of the arm. It essentially serves as an asynchronous, non-blocking task scheduler. Some example tasks include BMS monitoring, EMG processing, servo motor controls and overall system health monitoring. It is based on Tokio's "task" asbtraction (a.k.a green threads). This design decision was made to avoid the overhead of OS-level threads given that GPM runs on the single-core Raspberry Pi Zero.

## Installation
```bash
# After cloning the repository, run the following commands to initialize the `sgcp` submodule
git submodule init
git submodule update

# Install rust on your system by following these instructions: https://www.rust-lang.org/tools/install
# You would need to install the nightly rust compiler -- run the following commands:
rustup toolchain install nightly
rustup default nightly

# Finally, start GPM
cd <REPOSITORY>
cargo run

# Command you must add for when you are using pi for actuation of arm vs not using Pi
```
## File Structure
```
sgcp/               -> Protobuf definitions needed to (de)serialize requests
src/                
├─ managers/        -> Holds source for all "resource managers" ("resource" refers to a distinct component of the arm, such as BMS)
├─ config.rs
├─ connection.rs    -> Implements a simple prefix-length framing abstraction to enable streaming protobufs
├─ exporter.rs      -> Source for the HTTP telemetry server exposing a custom prometheus exporter endpoint    
├─ macros.rs        -> A collection of helpful macros used across the codebase
├─ main.rs          
├─ server.rs        -> Source for the main TCP server   
```

## System Overview
GPM and Analytics make up the embedded software for GRASP. While GPM is at the heart of the arm and is responsible for interfacing to every component, the Analytics module is the "brain" of the arm and is responsible for making the decisions on grip type based on EMG readings from the pilot and the camera feed. The GRM (Grasp Remote Module) component hosts a Prometheus server that scrapes data from GPM and displays a Grafana dashboard for monitoring.


## Core Components

### 1. **Servo Controller (Maestro)**

**File:** `maestro.rs`

The servo controller handles the arm's physical movements, such as opening and closing the hand. It translates high-level commands into precise motor control signals.

#### Key Responsibilities:

- **Initialization:** Configures the controller with predefined baud rates and timing parameters.
  ```rust
  let mut controller = Builder::default()
      .baudrate(Baudrate::Baudrate11520)
      .block_duration(Duration::from_millis(100))
      .try_into()
      .unwrap();
  ```
- **Task Execution:** Uses PWM values to achieve motions like `OpenFist` or `CloseFist` by controlling servo channels.
  ```rust
  set_target!(self.metadata, Channel::Channel0 => MAX_QTR_PWM);
  ```

### 2. **Muscle Sensor Interface (EMG)**

**File:** `emg.rs`

The EMG module reads electrical activity from muscles to interpret gestures. Though currently in a placeholder state, it sets the framework for future integrations.

#### Key Responsibilities:

- **Signal Mapping:** Planned feature to translate muscle signals into actionable commands for the arm.
  ```rust
  pub struct Emg { }
  ```

### 3. **Battery Management System (BMS)**

**File:** `bms.rs`

This module ensures the arm operates safely and efficiently by monitoring battery health and power usage.

#### Key Responsibilities:

- **Initialization:** Sets up the module for basic operations.
- **Health Metrics:** Future updates will allow retrieving detailed battery metrics.
  ```rust
  Task::GetHealthMetrics => todo!(),
  ```

### 4. **Task Management Framework**

**File:** `main.rs` and `mod.rs`

The task management framework is the backbone of the system, delegating incoming commands to respective hardware modules.

#### Key Responsibilities:

- **Manager Initialization:** Spawns resource managers for all components dynamically.
  ```rust
  let manager_channel_map = init_resource_managers! {
      Resource::Bms => Manager::<Bms>::new(),
      Resource::Emg => Manager::<Emg>::new(),
      Resource::Maestro => Manager::<Maestro>::new()
  };
  ```
- **Task Dispatching:** Routes commands like `move arm` to the appropriate module.

### 5. **Telemetry Exporter**

**File:** `exporter.rs`

The telemetry exporter collects and serves system performance data, ensuring transparency in operations.

#### Key Responsibilities:

- **Metrics Collection:** Tracks CPU and memory usage using system information libraries.
  ```rust
  gauge.set(sys.used_memory() as i64);
  ```
- **HTTP Server:** Exposes metrics through an endpoint, enabling external monitoring tools to access them.
  ```rust
  let listener = TcpListener::bind(TELEMETRY_TCP_ADDR).await.unwrap();
  ```

---

## High-Level Processes

### 1. **Command Handling**

1. A task request is sent to the system over a TCP connection.
2. The central control loop identifies the relevant module and forwards the task.
3. The module processes the task and returns a success or error response.

### 2. **Movement Control**

1. Servo motors execute tasks like `OpenFist` or `CloseFist` by adjusting motor positions through PWM signals.
2. The movement parameters are fine-tuned for precise control.

### 3. **System Monitoring**

1. CPU and memory usage metrics are collected at regular intervals.
2. Metrics are served over HTTP for real-time monitoring by external tools.

### 4. **GPIO Pin Monitoring**

1. GPIO pins detect external triggers, such as muscle contractions.
2. Trigger events map to predefined actions, like opening or closing the hand.


> A high-level overview of the GRASP system components:
<!-- <p align="left">
  <img width="703" alt="image" src="https://github.com/user-attachments/assets/5e40bb3e-f838-440a-9d8c-5f6db26e568f">
</p> -->
<p align="left">
  <img alt="image" src=assets/grasp_2_edit.png>
</p>

## Next Steps
The way that GPM is designed hints at the possibility to create a rather simple framework for designing "task" based embedded software for the Pi. We would want to explore this further and spin up a PoC. For more on embedded system frameworks, have a look at NASA's [f-prime](https://github.com/nasa/fprime).
