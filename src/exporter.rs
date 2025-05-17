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

use anyhow::Ok;
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

use crate::config::Config;
use crate::config::TelemetryConfig;
use crate::retry;

type Label = Vec<(String, String)>;
type GaugeMetric = Family<Label, Gauge>;

/// Holds the registry of metrics and each metric definition
pub struct Exporter {
    registry: Arc<Registry>,
    cpu_usage: GaugeMetric,
    memory_usage: GaugeMetric,
}

impl Exporter {
    pub fn new() -> Self {
        let mut registry = <Registry>::default();
        let cpu_usage = GaugeMetric::default();
        let memory_usage = GaugeMetric::default();
        registry.register(
            "cpu_usage",
            "Current CPU load percentage",
            cpu_usage.clone(),
        );
        registry.register(
            "memory_usage",
            "Current memory utilization",
            memory_usage.clone(),
        );
        Self {
            registry: Arc::new(registry),
            cpu_usage,
            memory_usage,
        }
    }

    /// Starts the HTTP telemetry server -- can handle at most MAX_CONNCURRENT_CONNECTIONS
    /// connections at any given time
    /// TODO: @krarpit telemetry needs access to manager channel map in order to probe resource
    /// health                this needs to be cleaned up and tested
    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::global();
        let server_config = config.server.as_ref().unwrap();
        let telemetry_config = config.telemetry.as_ref().unwrap();
        let listener = TcpListener::bind(telemetry_config.address.clone())
            .await
            .unwrap();
        let sem = Arc::new(Semaphore::new(
            server_config.max_concurrent_connections as usize,
        ));
        info!(
            "Telemetry server listening on {:?}",
            telemetry_config.address
        );
        loop {
            // let sem_clone = Arc::clone(&sem);
            let (stream, client_addr) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let registry = self.registry.clone();
            let cpu_usage = self.cpu_usage.clone();
            let memory_usage = self.memory_usage.clone();
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(|req| async {
                            info!("Responding to metrics request");
                            Exporter::get_cpu_usage(&cpu_usage);
                            Exporter::get_memory_usage(&memory_usage);
                            let mut buffer = String::new();
                            encode(&mut buffer, &registry).unwrap();
                            Ok::<Response<Full<Bytes>>>(Response::new(Full::new(Bytes::from(
                                buffer.clone(),
                            ))))
                        }),
                    )
                    .await
                {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    fn get_cpu_usage(gauge: &GaugeMetric) {
        let mut sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        sys.refresh_cpu_all();
        // wait a bit because CPU usage is based on diff
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_all();
        gauge
            .get_or_create(&vec![])
            // returning just the first core since the Pi Zero is a single core machine
            .set(sys.cpus()[0].cpu_usage() as i64);
    }

    fn get_memory_usage(gauge: &GaugeMetric) {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_memory();
        gauge.get_or_create(&vec![]).set(sys.used_memory() as i64);
    }
}
