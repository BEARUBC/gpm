use core::panic;
// All tasks operating on the EMG system live in this file
use std::io;
use std::thread;
use std::time::Duration;

use anyhow::Error;
use anyhow::Result;
use chrono::Utc;
use gpm::sgcp;
use log::*;
use rand::Rng;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use rppal::spi::Bus;
use rppal::spi::Mode;
use rppal::spi::SlaveSelect;
use rppal::spi::Spi;

use super::EmgData;
use crate::config::Config;
use crate::resources::Resource;
use crate::resources::common::Adc;
use crate::resources::emg::emgprocessor::EmgProcessor;

pub struct Emg {
    pub adc: Adc,
    pub buffer_size: usize,
    pub inner_threshold: f32,
    pub outer_threshold: f32,
    pub prev_grip_state: i32,
    pub inter_channel_sample_duration: u64, /* different from sampling speed, this is the time
                                             * between reading the inner and outer channels */
    pub emg_processor_outer: EmgProcessor,
    pub emg_processor_inner: EmgProcessor,
    pub open_counter : usize,
    pub close_counter : usize,  
    pub current_channel_0: f32,
    pub current_channel_1: f32,
}

impl Resource for Emg {
    fn init() -> Self {
        let emg_config = Config::global()
            .dispatcher
            .emg
            .as_ref()
            .expect("Expected emg config to be defined");

        let adc = Adc::init(emg_config.cs_pin, emg_config.clock_speed);
        
        let processor_outer = EmgProcessor::new(1000.0, 60.0, 20.0, 450.0, emg_config.buffer_size);
        let processor_inner = EmgProcessor::new(1000.0, 60.0, 20.0, 450.0, emg_config.buffer_size);

        let mut emg = Emg {
            adc,
            buffer_size: emg_config.buffer_size,
            inner_threshold: 0.0,
            outer_threshold: 0.0,
            prev_grip_state: 0,
            inter_channel_sample_duration: emg_config.pause_duration_ms,
            emg_processor_outer: processor_outer,
            emg_processor_inner: processor_inner,
            open_counter: 0,
            close_counter: 0,
            current_channel_0: 0.0,
            current_channel_1: 0.0,
        };

        if let Err(_) = Emg::calibrate_emg(&mut emg) {
            panic!("Unable to calibrate EMG.");
        }

        emg
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl Emg {
    pub fn process_data(&mut self, values: Vec<f32>) -> Result<i32> {
        const OPEN_FIST: i32 = 1;
        const CLOSE_FIST: i32 = 0;
        const HOLD_TIME: usize = 5; 

        if values.len() != 2 {
            return Err(Error::msg("Expected 2 EMG values"));
        }

        if values[0] >= self.inner_threshold && values[1] <= self.outer_threshold {
            self.open_counter += 1;
            self.close_counter = 0;
    
            if self.open_counter >= HOLD_TIME {
                self.prev_grip_state = OPEN_FIST;
                return Ok(OPEN_FIST);
            }
        } 
        else if values[0] <= self.inner_threshold && values[1] >= self.outer_threshold {
            self.close_counter += 1;
            self.open_counter = 0;
    
            if self.close_counter >= HOLD_TIME {
                self.prev_grip_state = CLOSE_FIST;
                return Ok(CLOSE_FIST);
            }
        } 
        else {
            // Reset counters if neither condition met
            self.open_counter = 0;
            self.close_counter = 0;

            info!("EMG values out of expected range. Holding previous action.");
        }
        Ok(self.prev_grip_state)
    }

    // send adc values to exporter
    pub fn get_current(&self) -> EmgData {
        EmgData {
            channel_0: self.current_channel_0 as f64,
            channel_1: self.current_channel_1 as f64,
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }

    /// Calibrate EMG system: first the baseline at rest, then activation thresholds
    pub fn calibrate_emg(&mut self) -> Result<()> {
        // --- Step 1: Calibrate baseline while relaxed ---
        info!("Calibrating baseline. Please relax your muscles...");
        let baseline_samples_inner = self.read_samples(0, "baseline inner"); // collect baseline samples
        let baseline_samples_outer = self.read_samples(1, "baseline outer");
        self.emg_processor_inner.calibrate_processer_baseline(baseline_samples_inner.as_ref()); 
        self.emg_processor_outer.calibrate_processer_baseline(baseline_samples_outer.as_ref()); 
        info!("Baseline inner calibrated at {:.2}", self.emg_processor_inner.baseline);
        info!("Baseline outer calibrated at {:.2}", self.emg_processor_outer.baseline);

        // --- Step 2: Calibrate activation thresholds ---
        info!("Now calibrating activation thresholds.");
        info!("Press ENTER when you're ready to start inner sampling...");
        let _ = io::stdin().read_line(&mut String::new());
        let inner_buffer = self.read_samples(0, "inner");

        info!("Press ENTER when you're ready to start outer sampling...");
        let _ = io::stdin().read_line(&mut String::new());
        let outer_buffer = self.read_samples(1, "outer");

        // --- Step 3: Process samples through EMG pipeline ---
        let processed_inner: Vec<f32> = inner_buffer
            .iter()
            .filter_map(|&raw| self.emg_processor_inner.process_sample(raw as f32))
            .collect();

        let processed_outer: Vec<f32> = outer_buffer
            .iter()
            .filter_map(|&raw| self.emg_processor_outer.process_sample(raw as f32))
            .collect();

        // --- Step 4: Compute average processed values as thresholds ---
        let avg_inner = if !processed_inner.is_empty() {
            processed_inner.iter().copied().sum::<f32>() / processed_inner.len() as f32
        } else {
            0.0
        };

        let avg_outer = if !processed_outer.is_empty() {
            processed_outer.iter().copied().sum::<f32>() / processed_outer.len() as f32
        } else {
            0.0
        };

        self.inner_threshold = avg_inner;
        self.outer_threshold = avg_outer;

        info!(
            "Thresholds set: inner = {:.2}, outer = {:.2}",
            self.inner_threshold, self.outer_threshold
        );

        Ok(())
    }



    pub fn read_samples(&mut self, channel: u8, label: &str) -> Vec<u16> {
        let calibrate_buffer_size = 100;
        let mut buffer = Vec::with_capacity(calibrate_buffer_size);
        if label.contains("baseline") {
            info!("Get ready to flex your {}", label);
            thread::sleep(Duration::from_secs(3));
            info!("Flex {}", label);
        }
        else {
            info!("About to collect {} samples, please relax...", label);
            thread::sleep(Duration::from_secs(3));
            info!("Collecting {} samples now...", label);
        }
       
        while buffer.len() < calibrate_buffer_size {
            match self.adc.read_channel(channel) {
                Ok(value) => {
                    info!("Channel {channel} SPI value: {value}");
                    buffer.push(value);
                },
                Err(_) => info!("Error reading SPI on channel {channel} during {label}"),
            }
            thread::sleep(Duration::from_millis(self.inter_channel_sample_duration));
        }

        buffer
    }

    pub fn read_adc_channels(&mut self, channels: &[u8]) -> Result<Vec<f32>> {
        self.adc.read_channels(channels)
            .map(|vec_u16| vec_u16.into_iter().map(|x| x as f32).collect())
    }

    // function no to be used, included to help compile
    pub fn read_adc() -> EmgData {
        let mut rng = rand::rng();
        EmgData {
            channel_0: rng.random_range(-1.0..1.0),
            channel_1: rng.random_range(-0.5..0.5),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}
