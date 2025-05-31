// This files defines a function that monitors GPIO pins and dispatches fist open/close commands
// based on the pin state. This is an alternate strategy of dispatching commands than the
// SGCP-based commands sent over TCP. We usually use this for testing the arm with a button to
// control it.
use crate::managers::ManagerChannelData;
use tokio::sync::mpsc::Sender;

use crate::config::Config;
use crate::sgcp;
use log::*;
#[cfg(feature = "pi")]
use rppal::gpio::{Gpio, InputPin};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::sleep;

pub async fn run_gpio_monitor_loop(maestro_tx: Sender<ManagerChannelData>) {
    #[cfg(feature = "pi")]
    {
        let gpio_monitor_config = Config::global()
            .gpio_monitor
            .as_ref()
            .expect("Expected GPIO monitor config to be defined");

        info!(
            "Started GPIO pin monitor for pin {:?}",
            gpio_monitor_config.pin
        );

        let gpio = Gpio::new().expect("Failed to initialize GPIO");
        let pin = gpio
            .get(gpio_monitor_config.pin)
            .expect("Failed to access pin")
            .into_input_pullup();

        loop {
            let (resp_tx, resp_rx) = oneshot::channel::<String>();
            let task_code = if pin.is_high() {
                sgcp::maestro::Task::OpenFist.as_str_name().to_string()
            } else {
                sgcp::maestro::Task::CloseFist.as_str_name().to_string()
            };

            if let Err(e) = maestro_tx
                .send(ManagerChannelData {
                    task_code,
                    task_data: None,
                    resp_tx,
                })
                .await
            {
                error!("Failed to send command to Maestro: {}", e);
                continue;
            }

            match resp_rx.await {
                Ok(res) => info!("Receieved response from Maestro manager: {:?}", res),
                Err(e) => error!("Response channel closed before receiving response: {}", e),
            }

            sleep(Duration::from_millis(100)).await;
        }
    }
    panic!("Cannot use GPIO monitor outside the Pi");
}
