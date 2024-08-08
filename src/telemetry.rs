// This file contains a tiny http server which exposes system metrics
// and health check endpoints. These are then scraped by the Prometheus
// server running remotely.
use std::collections::HashMap;
use std::convert::Infallible;
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
use psutil::cpu::CpuPercentCollector;
use psutil::memory::virtual_memory;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::time;
use tokio::time::interval;

use crate::config::MAX_CONCURRENT_CONNECTIONS;
use crate::config::TELEMETRY_MAX_TICKS;
use crate::config::TELEMETRY_TCP_ADDR;
use crate::config::TELEMETRY_TICK_INTERVAL_IN_SECONDS;
use crate::retry;

/// Represents a timestamped data point to feed into Grafana
#[derive(Serialize, Deserialize, Debug)]
pub struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}

/// Starts the HTTP telemetry server -- can handle at most MAX_CONNCURRENT_CONNECTIONS connections
/// at any given time
/// TODO: @krarpit telemetry needs access to manager channel map in order to probe resource health
///                this needs to be cleaned up and tested
pub async fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(TELEMETRY_TCP_ADDR).await.unwrap();
    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));
    info!("Telemetry server listening on {:?}", TELEMETRY_TCP_ADDR);
    loop {
        let sem_clone = Arc::clone(&sem);
        let (stream, client_addr) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            // Bounds number of concurrent connections
            if let Ok(_) = retry!(sem_clone.try_acquire()) {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(get_metrics))
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

async fn check_cpu_usage_and_memory() -> Result<HashMap<String, Vec<DataPoint>>> {
    let mut map: HashMap<String, Vec<DataPoint>> = HashMap::new();
    let mut cpu_usage: Vec<DataPoint> = Vec::new();
    let mut memory_usage: Vec<DataPoint> = Vec::new();

    let mut interval = interval(Duration::from_secs(TELEMETRY_TICK_INTERVAL_IN_SECONDS));
    for _ in 0..TELEMETRY_MAX_TICKS {
        interval.tick().await;
        cpu_usage.push(check_cpu_usage().unwrap());
        memory_usage.push(check_memory_usage().unwrap());
    }

    info!("CPU usage={:?}", cpu_usage);
    info!("Memory usage={:?}", memory_usage);

    map.insert("cpu_usage".to_string(), cpu_usage);
    map.insert("memory_usage".to_string(), memory_usage);
    Ok(map)
}

fn check_cpu_usage() -> Result<DataPoint> {
    let mut cpu_collector = CpuPercentCollector::new().unwrap();
    let cpu_usage = cpu_collector.cpu_percent().unwrap();
    let data_point: DataPoint = DataPoint {
        timestamp: Utc::now(),
        value: cpu_usage,
    };
    Ok(data_point)
}

fn check_memory_usage() -> Result<DataPoint> {
    let memory = virtual_memory().expect("Failed to get virtual memory usage");
    let memory_usage = memory.percent();
    let data_point: DataPoint = DataPoint {
        timestamp: Utc::now(),
        value: memory_usage,
    };
    Ok(data_point)
}

async fn get_metrics(
    _: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    info!("Received request for metrics");
    let data = check_cpu_usage_and_memory().await.unwrap();
    let json = serde_json::to_string(&data).expect("Failed to serialize data");
    Ok(Response::new(Full::new(Bytes::from(json))))
}
