// This file contains a tiny http server which exposes system metrics
// and health check endpoints. These are then scraped by the 
// Prometheus server running remotely. 
use psutil::cpu::CpuPercentCollector;
use tokio::time;
use std::time::Duration;
use chrono::Utc;

use anyhow::Result;

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use super::DataPoint;

pub mod sgcp {
    include!(concat!(env!("OUT_DIR"), "/sgcp.telemetry.rs"));
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9999));
    let listener = TcpListener::bind(addr).await.unwrap();
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

pub async fn handle_telemetry_task(task_code: i32) -> Result<()> {
    match sgcp::Tasks::try_from(task_code).unwrap() {
        sgcp::Tasks::CheckCpuUsageAndMemory => {
            info!("Checking cpu usage and memory");
            check_cpu_usage_and_memory().await.unwrap();
        }
        _ => println!("Unmatched task, ignoring...")
    }
    Ok(())    
}

pub async fn check_cpu_usage_and_memory() -> Result<Vec<DataPoint>> {
    check_cpu_usage().await
    // check_memory_usage().await.unwrap();
    // Ok(())
}

pub async fn check_cpu_usage() -> Result<Vec<DataPoint>> {
    let mut cpu_collector = CpuPercentCollector::new().unwrap();
    let mut res: Vec<DataPoint> = Vec::new();
    info!("Checking CPU usage...");
    for _ in 0..5 { // fetch CPU usage every second for 5 seconds
        let cpu_percent = cpu_collector.cpu_percent().unwrap();
        let data_point = DataPoint {
            timestamp: Utc::now(),
            value: cpu_percent
        };
        res.push(data_point);
        info!("Current CPU Usage: {:.2}%", cpu_percent);
        time::sleep(Duration::from_secs(1)).await;
    }
    Ok(res)
}

pub async fn check_memory_usage() -> Result<()> {
    Ok(())
}

async fn get_metrics(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let data = check_cpu_usage_and_memory().await.unwrap();
    let json = serde_json::to_string(&data).expect("Failed to serialize data");
    Ok(Response::new(Full::new(Bytes::from(json))))
}