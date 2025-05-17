use log::LevelFilter;

// GPM TCP listener configs
pub const MAX_CONCURRENT_CONNECTIONS: usize = 1;
#[cfg(not(feature = "pi"))]
pub const GPM_TCP_ADDR: &str = "127.0.0.1:4760";
#[cfg(feature = "pi")]
pub const GPM_TCP_ADDR: &str = "0.0.0.0:4760";
pub const READ_BUFFER_CAPACITY: usize = 1024;
pub const FRAME_PREFIX_LENGTH: usize = 8;

// Telemetry HTTP server configs
#[cfg(not(feature = "pi"))]
pub const TELEMETRY_TCP_ADDR: &str = "127.0.0.1:9999";
#[cfg(feature = "pi")]
pub const TELEMETRY_TCP_ADDR: &str = "0.0.0.0:9999";
pub const TELEMETRY_TICK_INTERVAL_IN_SECONDS: u64 = 1;
pub const TELEMETRY_MAX_TICKS: usize = 5;

// Logging configs and constants
const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
const GRASP_ASCII: &str = r"
   ______                    
  / ____/________ __________ 
 / / __/ ___/ __ `/ ___/ __ \
/ /_/ / /  / /_/ (__  ) /_/ /
\____/_/   \__,_/____/ .___/ 
                    /_/      ";
const VERSION_LINE: &str = "Grasp primary control module | Version 0.0.1";
const BYLINE: &str = "Developed at UBC Bionics (http://www.ubcbionics.com)";
const NEW_LINE: &str = "\n";

/// Initializes env_logger and prints metadata
pub fn init() {
    env_logger::builder().filter_level(LOG_LEVEL).init();
    println!("{}", GRASP_ASCII);
    println!("{}", VERSION_LINE);
    println!("{}", BYLINE);
    println!("{}", NEW_LINE);
}
