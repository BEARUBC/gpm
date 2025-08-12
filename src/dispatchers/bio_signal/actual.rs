use std::time::Duration;

use log::*;
use tokio::time::interval;

use super::BioSignalDispatcher;
use crate::ManagerChannelMap;
use crate::config::Config;
use crate::dispatchers::Dispatcher;
use crate::dispatchers::emg::EmgDispatcher;

impl Dispatcher for BioSignalDispatcher {
    async fn run(manager_channel_map: ManagerChannelMap) {
        let bio_signal_config = Config::global()
            .dispatcher
            .bio_signal
            .as_ref()
            .expect("Bio Signal config should be defined.");
        let mut emg_dispatcher = EmgDispatcher;
        let mut emg_idle = interval(Duration::from_millis(0));
        let mut fsr_idle = interval(Duration::from_millis(0));

        // for each sensor get their config for their pause interval
        for sensor in &bio_signal_config.sensors {
            match sensor.as_str() {
                "emg" => {
                    let emg_config = Config::global()
                        .dispatcher
                        .emg
                        .as_ref()
                        .expect("Emg config should be defined.");

                    emg_idle = interval(Duration::from_millis(emg_config.sampling_speed_ms))
                },
                "fsr" => {
                    let fsr_config = Config::global()
                        .dispatcher
                        .fsr
                        .as_ref()
                        .expect("Emg config should be defined.");

                    fsr_idle = interval(Duration::from_millis(fsr_config.pause_duration_ms))
                },

                _ => {
                    info!("{sensor} is not defined. Skipping...");
                },
            }
        }

        // do the select block here and call process_idle_task for each sensor
        loop {
            tokio::select! {
                _ = emg_idle.tick() => {
                    emg_dispatcher.process_idle_task(&manager_channel_map).await;
                }
                _ = fsr_idle.tick() => {
                    //Todo: dispatch fsr idle tasks
                }
            }
        }
    }
}
