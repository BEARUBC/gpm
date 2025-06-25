use log::LevelFilter;
use serde::Deserialize;
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub max_concurrent_connections: i32,
    pub address: String,
    pub read_buffer_capacity_in_bytes: i32,
    pub frame_prefix_length_in_bytes: i32,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {
    pub address: String,
    pub tick_interval_in_seconds: i32,
}

#[derive(Debug, Deserialize)]
pub struct GpioMonitorConfig {
    pub pin: u8,
}

#[derive(Debug, Deserialize)]
pub struct EmgConfig {
    pub buffer_size: usize,
    pub pause_duration_ms: u64,
    pub sampling_speed_ms: u64,
    pub cs_pin: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandDispatchStrategy {
    Tcp,
    Gpio,
    Emg,
}

impl Default for CommandDispatchStrategy {
    fn default() -> Self {
        CommandDispatchStrategy::Tcp
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub telemetry: Option<TelemetryConfig>,
    pub gpio_monitor: Option<GpioMonitorConfig>,
    pub emg_sensor: Option<EmgConfig>,
    #[serde(default)]
    pub command_dispatch_strategy: CommandDispatchStrategy,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub const fn get_config_path() -> &'static str {
    if cfg!(feature = "pi") {
        "./gpm.config.pi.toml"
    } else {
        "./gpm.config.dev.toml"
    }
}

impl Config {
    pub fn global() -> &'static Self {
        CONFIG.get_or_init(|| {
            let config_path = get_config_path();
            let content = fs::read_to_string(config_path).expect("Failed to read config");
            toml::from_str(&content).expect("Failed to parse config")
        })
    }
}

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
pub fn logger_init() {
    env_logger::builder().filter_level(LOG_LEVEL).init();
    println!("{}", GRASP_ASCII);
    println!("{}", VERSION_LINE);
    println!("{}", BYLINE);
    println!("{}", NEW_LINE);
}

