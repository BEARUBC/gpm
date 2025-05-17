// This files defines a function that monitors GPIO pins and dispatches fist open/close commands
// based on the pin state. This is an alternate strategy of dispatching commands than the
// SGCP-based commands sent over TCP. We usually use this for testing the arm with a button to
// control it.

#![cfg(feature = "pi")]

use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use crate::config::Config;
use crate::connection::Connection;
use crate::exporter::Exporter;
use crate::managers::Bms;
use crate::managers::Emg;
use crate::managers::Maestro;
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::sgcp;
use anyhow::Result;
use bytes::BytesMut;
use log::*;
use prost::Message;
use rppal::gpio::Gpio;
use rppal::gpio::InputPin;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::time::sleep;

pub async fn monitor_pin(maestro_tx: Sender<ManagerChannelData>) {
    let gpio_monitor_config = Config::global().gpio_monitor.as_ref().unwrap();
    info!(
        "Started GPIO pin monitor for pin {:?}",
        gpio_monitor_config.pin
    );
    let gpio = Gpio::new().expect("Failed to initialize GPIO");
    let mut pin = gpio
        .get(gpio_monitor_config.pin)
        .expect("Failed to access pin")
        .into_input_pullup();
    loop {
        let (resp_tx, resp_rx) = oneshot::channel::<String>();
        if pin.is_high() {
            maestro_tx.send(ManagerChannelData {
                task_code: sgcp::maestro::Task::OpenFist.as_str_name().to_string(),
                task_data: None,
                resp_tx,
            });
        } else {
            maestro_tx.send(ManagerChannelData {
                task_code: sgcp::maestro::Task::CloseFist.as_str_name().to_string(),
                task_data: None,
                resp_tx,
            });
        }
        let res = resp_rx.await.unwrap();
        info!("Receieved response from Maestro manager: {:?}", res);
        sleep(Duration::from_millis(100)).await;
    }
}
