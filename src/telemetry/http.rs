// This file contains a tiny http server which exposes system metrics
// and health check endpoints. These are then scraped by the 
// Prometheus server running remotely. 
use psutil::cpu::CpuPercentCollector;
use psutil::memory::virtual_memory;
use tokio::time;
use std::time::Duration;
use chrono::Utc;
use log::*;

use anyhow::Result;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::collections::HashMap;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use super::MetricDataPoint;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9999));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Listening on port 9999");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(get_metrics))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn check_cpu_usage_and_memory() -> Result<HashMap<String, Vec<MetricDataPoint>>> {
    let mut map: HashMap<String, Vec<MetricDataPoint>> = HashMap::new();
    let mut cpu_usage: Vec<MetricDataPoint> = Vec::new();
    let mut memory_usage: Vec<MetricDataPoint> = Vec::new();
    for _ in 0..5 { // collect cpu and memory usage once every second for the next 5 seconds
        cpu_usage.push(check_cpu_usage().unwrap());
        memory_usage.push(check_memory_usage().unwrap());
        time::sleep(Duration::from_secs(1)).await;
    }
    map.insert("cpu_usage".to_string(), cpu_usage);
    map.insert("memory_usage".to_string(), memory_usage);
    Ok(map)
}

fn check_cpu_usage() -> Result<MetricDataPoint> {
    let mut cpu_collector = CpuPercentCollector::new().unwrap();
    let cpu_usage = cpu_collector.cpu_percent().unwrap();
    let data_point: MetricDataPoint = MetricDataPoint {
        timestamp: Utc::now(),
        value: cpu_usage
    };
    info!("Current CPU Usage: {:.2}%", cpu_usage);
    Ok(data_point)
}

fn check_memory_usage() -> Result<MetricDataPoint> {
    let memory = virtual_memory().expect("Failed to get virtual memory usage");
    let memory_usage = memory.percent();
    let data_point: MetricDataPoint = MetricDataPoint {
        timestamp: Utc::now(),
        value: memory_usage
    };
    info!("Current Memory Usage: {:.2}%", memory_usage);
    Ok(data_point)
}

async fn get_metrics(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let data = check_cpu_usage_and_memory().await.unwrap();
    let json = serde_json::to_string(&data).expect("Failed to serialize data");
    Ok(Response::new(Full::new(Bytes::from(json))))
}