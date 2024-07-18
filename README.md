## Grasp Primary Module (GPM)
This repository contains the source code for the primary software module of the Grasp project. This project is still very much in its early stages, but the goal is to create a highly extensible asynchronous framework for building embedded systems (much like NASA's [f-prime](https://nasa.github.io/fprime/)). 

## Purpose
The purpose of the GPM is to manage all systems of the arm. These include BMS, EMG processing, servo motor controls and overall system health monitoring / telemetry. It heavily relies on Tokio's tasks (also known as green threads). This design decision was made to avoid the overhead of OS-level threads on the single-core Raspberry Pi Zero we are using.