use std::clone;
// This file contains a tiny http server which exposes our custom
// prometheus exporter endpoint
use std::collections::HashMap;
use std::convert::Infallible;
use std::default;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use log::*;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::EncodeLabelValue;
use prometheus_client::metrics::counter::Atomic;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry;
use prometheus_client::registry::Registry;
use psutil::cpu::CpuPercentCollector;
use psutil::memory::virtual_memory;
use serde::Deserialize;
use serde::Serialize;
use sysinfo::Components;
use sysinfo::CpuRefreshKind;
use sysinfo::Disks;
use sysinfo::MemoryRefreshKind;
use sysinfo::Networks;
use sysinfo::RefreshKind;
use sysinfo::System;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::time;
use tokio::time::interval;

use crate::config::MAX_CONCURRENT_CONNECTIONS;
use crate::config::TELEMETRY_MAX_TICKS;
use crate::config::TELEMETRY_TCP_ADDR;
use crate::config::TELEMETRY_TICK_INTERVAL_IN_SECONDS;
use crate::retry;

type Label = Vec<(String, String)>;

/// Holds the registry of metrics and each metric definition
pub struct Exporter {
    registry: Registry,
    cpu_usage: Family<Label, Gauge>,
    memory_usage: Family<Label, Gauge>,
}

impl Exporter {
    pub fn new() -> Self {
        Self {
            registry: <Registry>::default(),
            cpu_usage: Family::<Vec<(String, String)>, Gauge>::default(),
            memory_usage: Family::<Vec<(String, String)>, Gauge>::default(),
        }
    }

    /// Starts the HTTP telemetry server -- can handle at most MAX_CONNCURRENT_CONNECTIONS
    /// connections at any given time
    /// TODO: @krarpit telemetry needs access to manager channel map in order to probe resource
    /// health                this needs to be cleaned up and tested
    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(TELEMETRY_TCP_ADDR).await.unwrap();
        let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));
        info!("Telemetry server listening on {:?}", TELEMETRY_TCP_ADDR);
        self.register_metrics();
        loop {
            let sem_clone = Arc::clone(&sem);
            let (stream, client_addr) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            info!("Received metrics request");
            self.get_cpu_usage();
            self.get_memory_usage();
            let buffer = self.dump_registry();
            tokio::task::spawn(async move {
                // Bounds number of concurrent connections
                if let Ok(_) = retry!(sem_clone.try_acquire()) {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(
                            io,
                            service_fn(|req| async {
                                Ok::<Response<Full<Bytes>>, Infallible>(Response::new(Full::new(
                                    Bytes::from(buffer.clone()),
                                )))
                            }),
                        )
                        .await
                    {
                        error!("Error serving connection: {:?}", err);
                    }
                } else {
                    error!("Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}", client_addr, MAX_CONCURRENT_CONNECTIONS)
                }
            });
        }
    }

    fn register_metrics(&mut self) {
        self.registry
            .register("cpu_usage", "Current CPU load percentage", self.cpu_usage.clone());
        self.registry.register(
            "memory_usage",
            "Current memory utilization",
            self.memory_usage.clone(),
        );
    }

    fn get_cpu_usage(&self) {
        let mut sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        // wait a bit because CPU usage is based on diff
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_all();
        self.cpu_usage
            .get_or_create(&vec![])
            // returning just the first core since the Pi Zero is a single core machine
            .set(sys.cpus()[0].cpu_usage() as i64);
    }

    fn get_memory_usage(&self) {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_memory();
        self.memory_usage
            .get_or_create(&vec![])
            .set(sys.used_memory() as i64);
    }

    /// Encode metrics registered with the provided Registry using OpenMetrics text format, return
    /// encoded string
    fn dump_registry(&self) -> String {
        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }
}
