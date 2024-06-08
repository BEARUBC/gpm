// This file contains the main TCPListener loop and is responsible for
// dispatching the required task.
mod taskdefs;

use anyhow::Result;
use tokio::{
    io::{
        AsyncBufReadExt, 
        AsyncReadExt, 
        AsyncWriteExt, 
        BufReader
    }, 
    net::{
        TcpListener, 
        TcpStream
    }
};
use taskdefs::*;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4760").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await.unwrap();
        });
    }
}

// Reads request contents. Currently using the HTTP protocol as a PoC, we should 
// design a custom communication format better suited to our needs.
async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let buffer = read_request_bytes(&mut stream).await.unwrap();
    match String::from_utf8(buffer) {
        Ok(string) => {
            println!("Converted string: {}", string);
            let task_type = string.split("=").nth(1).unwrap();
            handle_task(task_type).await.unwrap();
        }
        Err(e) => {
            println!("Failed to convert byte buffer to string: {}", e);
        }
    }
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).await.unwrap();
    Ok(())
}

async fn handle_task(task_type: &str) -> Result<()> {
    match task_type {
        "BMS" => {
            println!("Dispatching BMS task");
            tokio::spawn(bms::check_battery_usage());
        }
        "EMG" => {
            println!("Dispatching EMG task");
            tokio::spawn(emg::read_edc());
        }
        "SERVO" => {
            println!("Dispatching SERVO task");
            tokio::spawn(servo::send_command());
        }
        "TELEMETRY" => {
            println!("Dispatching TELEMETRY task");
            tokio::spawn(telemetry::check_health());
        }
        _ => {
            println!("Unmatched task, ignoring...")
        }
    }
    Ok(())
}

// A helper function to read form values from a HTTP post request. Again, HTTP is just
// used as a proof of concept, we should refactor communications with a custom protocol.
async fn read_request_bytes(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut reader = BufReader::new(stream);
    let mut name = String::new();
    loop {
        let r = reader.read_line(&mut name).await.unwrap();
        if r < 3 {
            break;
        }
    }
    let mut size = 0;
    let linesplit = name.split("\n");
    for l in linesplit {
        if l.starts_with("Content-Length") {
            let sizeplit = l.split(":");
            for s in sizeplit {
                if !(s.starts_with("Content-Length")) {
                    size = s.trim().parse::<usize>().unwrap();
                }
            }
        }
    }
    let mut buffer = vec![0; size];
    reader.read_exact(&mut buffer).await.unwrap();
    Ok(buffer)
}
