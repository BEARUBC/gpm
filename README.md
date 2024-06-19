## Grasp Primary Module (GPM)
This repository contains the source code for the primary software module of the Grasp project. It is still in its early stages. The purpose of the GPM is to manage all systems of the arm. These include BMS, EMG processing, servo motor controls and overall system health monitoring / telemetry. It is designed to be run on a Pi Zero. Given the fact that the Pi Zero has a single-core CPU, we did not want to introduce the overhead of OS-level threads, thus the GPM is essentially a single async event loop. We use the Tokio async-runtime and Tokio “Tasks” to achieve this design.

<img width="707" alt="image" src="https://github.com/BEARUBC/gpm/assets/83952444/34cc34ef-b0c5-4f95-836a-b956bfd55171">




The idea is simple - we have a main TCP listener waiting for connections from either our remote monitoring module or the analytics module. We use protocol buffers to encode messages. This makes it easy to enforce the same message format across modules. The messages have the following shape:
```
message CommandRequest {
    Component component = 1;
    string    taskCode  = 2;
}

enum Component {
    UNKNOWN_COMPONENT = 0;
    BMS               = 1;
    EMG               = 2;
    SERVO             = 3;
    TELEMETRY         = 4;
}
```
Each task associated to the components get assigned a unique task ID which the GRM and analytics module use to invoke the desired task. Again, using protocol buffers means we can package this shared information easily. Given that these tasks are almost exclusively IO-bound, they work extremely well with Tokio and enable fair scheduling.
