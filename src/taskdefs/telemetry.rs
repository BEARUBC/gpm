// All tasks relating to the overall system health checking live in this file.
// These tasks will be triggered by our remote telemetry tool <TO-BE-NAMED>. 
use psutil::cpu::CpuPercentCollector;
use tokio::time;
use std::time::Duration;
use chrono::{DateTime, Utc};

use anyhow::Result;
pub mod sgcp {
    include!(concat!(env!("OUT_DIR"), "/sgcp.telemetry.rs"));
}

pub struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}

pub async fn handle_telemetry_task(task_code: i32) -> Result<()> {
    match sgcp::Tasks::try_from(task_code).unwrap() {
        sgcp::Tasks::CheckCpuUsageAndMemory => {
            println!("Checking cpu usage and memory");
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
    for _ in 0..5 { // fetch CPU usage every second for 5 seconds
        let cpu_percent = cpu_collector.cpu_percent().unwrap();
        let data_point = DataPoint {
            timestamp: Utc::now(),
            value: cpu_percent
        };
        res.push(data_point);
        println!("Current CPU Usage: {:.2}%", cpu_percent);
        time::sleep(Duration::from_secs(1)).await;
    }
    Ok(res)
}

pub async fn check_memory_usage() -> Result<()> {
    Ok(())
}