use std::env::set_var;

// GPM TCP listener configs
pub const MAX_CONCURRENT_CONNECTIONS: usize = 1;
pub const GPM_TCP_ADDR: &str = "127.0.0.1:4760";
pub const READ_BUFFER_CAPACITY: usize = 1024;
pub const FRAME_PREFIX_LENGTH: usize = 8;

// Telemetry HTTP server configs
pub const TELEMETRY_TCP_ADDR: &str = "127.0.0.1:9999";
pub const TELEMETRY_TICK_INTERVAL_IN_SECONDS: u64 = 1;
pub const TELEMETRY_MAX_TICKS: usize = 5;

// Logging configs and constants
const RUST_LOG_KEY: &str = "RUST_LOG";
const LOG_LEVEL: &str = "trace";
const GRASP_ASCII: &str = r"
   ______                    
  / ____/________ __________ 
 / / __/ ___/ __ `/ ___/ __ \
/ /_/ / /  / /_/ (__  ) /_/ /
\____/_/   \__,_/____/ .___/ 
                    /_/      ";
const VERSION_LINE: &str = "Version 0.0.1";
const BYLINE: &str = "Developed at UBC Bionics";
const NEW_LINE: &str = "\n";

/// Initializes env_logger and prints metadata
pub fn init() {
    set_var(RUST_LOG_KEY, LOG_LEVEL);
    env_logger::init();
    println!("{}", GRASP_ASCII);
    println!("{}", NEW_LINE);
    println!("{}", VERSION_LINE);
    println!("{}", BYLINE);
    println!("{}", NEW_LINE);
}
