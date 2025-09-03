use biquad::{DirectForm1, Coefficients, Type};
use std::collections::VecDeque;

pub struct EmgProcessor {
    // Filters
    filter_notch: DirectForm1<f32>,
    filter_high: DirectForm1<f32>,
    filter_low: DirectForm1<f32>,
    filter_post_rect: DirectForm1<f32>, // optional 10Hz smoothing

    // Rolling buffer for moving average
    window: VecDeque<f32>,
    ma_size: usize,

    // Flag to indicate if window is filled
    window_filled: bool,
}

impl EmgProcessor {
    pub fn new(fs: f32, notch_freq: f32, highpass_freq: f32, lowpass_freq: f32, ma_size: usize) -> Self {
        let coeff_notch = Coefficients::<f32>::from_params(Type::Notch, fs, notch_freq, 30.0).unwrap();
        let coeff_high = Coefficients::<f32>::from_params(Type::HighPass, fs, highpass_freq, 0.707).unwrap();
        let coeff_low  = Coefficients::<f32>::from_params(Type::LowPass, fs, lowpass_freq, 0.707).unwrap();
        let coeff_post = Coefficients::<f32>::from_params(Type::LowPass, fs, 10.0, 0.707).unwrap();

        Self {
            filter_notch: DirectForm1::new(coeff_notch),
            filter_high: DirectForm1::new(coeff_high),
            filter_low: DirectForm1::new(coeff_low),
            filter_post_rect: DirectForm1::new(coeff_post),
            window: VecDeque::with_capacity(ma_size),
            ma_size,
            window_filled: false,
        }
    }

    /// Process a single new sample from ADC
    /// Returns Some(processed_value) only after window is full
    pub fn process_sample(&mut self, raw_adc: u16) -> Option<f32> {
        // Convert ADC 8-bit value (0-255) to -1.0..1.0
        let mut sample = (raw_adc as f32 - 128.0) / 128.0;

        // Streaming filtering: notch → highpass → lowpass
        sample = self.filter_notch.run(sample);
        sample = self.filter_high.run(sample);
        sample = self.filter_low.run(sample);

        // Rectify
        sample = sample.abs();

        // Optional smoothing (10Hz low-pass)
        sample = self.filter_post_rect.run(sample);

        // Add to moving average window
        self.window.push_back(sample);
        if self.window.len() > self.ma_size {
            self.window.pop_front();
            self.window_filled = true; // window is now full
        }

        // Only output if window is full
        if self.window_filled {
            let avg: f32 = self.window.iter().sum::<f32>() / (self.window.len() as f32);
            Some(avg)
        } else {
            None // window not yet full
        }
    }
}
