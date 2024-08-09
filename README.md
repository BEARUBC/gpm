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
cd <REPOSITORY>
cargo run
```
## File Structure
```
sgcp/               -> Protobuf definitions needed to (de)serialize requests
src/                
├─ managers/        -> Holds source for all "resource managers" ("resource" refers to a distinct component of the arm, such as BMS)
├─ config.rs       
├─ macros.rs        -> A collection of helpful macros used across the codebase
├─ main.rs          
├─ server.rs        -> Source for the main TCP server
├─ streaming.rs     -> Implements a simple prefix-length framing abstraction to enable streaming protobufs
├─ telemetry.rs     -> Source for the HTTP telemetry server       
```

## System Overview
GPM and Analytics make up the embedded software for GRASP. While GPM is at the heart of the arm and is responsible for interfacing to every component, the Analytics module is the "brain" of the arm and is responsible for making the decisions on grip type based on EMG readings from the pilot and the camera feed. The GRM (Grasp Remote Module) component hosts a Prometheus server that scrapes data from GPM and displays a Grafana dashboard for monitoring.

> A high-level overview of the GRASP system components
<p align="left">
  <img width="705" alt="image" src="https://github.com/user-attachments/assets/b0cfa099-0c1a-48e3-a561-07305f334a5e">
</p>

## Next Steps
The way that GPM is designed hints at the possibility to create a rather simple framework for designing "task" based embedded software for the Pi. We would want to explore this further and spin up a PoC. For more on embedded system frameworks, have a look at NASA's [f-prime](https://github.com/nasa/fprime).
