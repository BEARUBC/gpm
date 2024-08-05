## Grasp Primary Module (GPM)
This repository contains the source code for the primary software module of the Grasp project. This project is still very much in its early stages, but the goal is to create a highly extensible, asynchronous framework for building embedded systems on the Raspberry Pi.

## Purpose
The purpose of the GPM is to manage all systems of the arm. It essentially serves as an asynchronous, non-blocking task scheduler. Some example tasks include BMS monitoring, EMG processing, servo motor controls and overall system health monitoring / telemetry. It heavily relies on Tokio's "task" asbtraction (a.k.a green threads). This design decision was made to avoid the overhead of OS-level threads given that GPM runs on the single-core Raspberry Pi Zero.
