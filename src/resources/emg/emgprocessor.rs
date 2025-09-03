use biquad::{Biquad, DirectForm1, Coefficients, Type, Hertz};

pub struct EmgProcessor {
    // Filters
    filter_high: DirectForm1<f32>,   // baseline drift removal
    filter_notch: DirectForm1<f32>,  // mains interference
    filter_low: DirectForm1<f32>,    // band-limiting
    filter_post_rect: DirectForm1<f32>, // envelope smoothing (10 Hz)

    // Moving average
    buffer: Vec<f32>,
    buffer_index: usize,
    sum: f32,
    ma_size: usize,

    // Calibration
    pub baseline: f32,
    pub calibrated: bool,

    // Envelope readiness
    pub window_filled: bool,
}

impl EmgProcessor {
    pub fn new(
        fs: f32,
        notch_freq: f32,
        highpass_freq: f32,
        lowpass_freq: f32,
        ma_size: usize,
    ) -> Self {
        let coeff_high = Coefficients::<f32>::from_params(
            Type::HighPass,
            Hertz::from_hz(fs).unwrap(),
            Hertz::from_hz(highpass_freq).unwrap(),
            0.707,
        ).unwrap();

        let coeff_notch = Coefficients::<f32>::from_params(
            Type::Notch,
            Hertz::from_hz(fs).unwrap(),
            Hertz::from_hz(notch_freq).unwrap(),
            30.0,
        ).unwrap();

        let coeff_low = Coefficients::<f32>::from_params(
            Type::LowPass,
            Hertz::from_hz(fs).unwrap(),
            Hertz::from_hz(lowpass_freq).unwrap(),
            0.707,
        ).unwrap();

        let coeff_post = Coefficients::<f32>::from_params(
            Type::LowPass,
            Hertz::from_hz(fs).unwrap(),
            Hertz::from_hz(10.0).unwrap(), // 10 Hz envelope smoothing
            0.707,
        ).unwrap();

        Self {
            filter_high: DirectForm1::new(coeff_high),
            filter_notch: DirectForm1::new(coeff_notch),
            filter_low: DirectForm1::new(coeff_low),
            filter_post_rect: DirectForm1::new(coeff_post),
            buffer: vec![0.0; ma_size],
            buffer_index: 0,
            sum: 0.0,
            ma_size,
            baseline: 512.0,  // default, but will be updated in calibration
            calibrated: false,
            window_filled: false,
        }
    }

    /// Calibrate baseline (collect N samples at rest)
    pub fn calibrate_processer_baseline(&mut self, raw_adc_samples: &[u16]) {
        let avg: f32 = raw_adc_samples.iter().map(|&x| x as f32).sum::<f32>() 
                       / raw_adc_samples.len() as f32;
        self.baseline = avg;
        self.calibrated = true;
    }

    /// Process one ADC sample
    pub fn process_sample(&mut self, raw_adc: f32) -> Option<f32> {
        if !self.calibrated {
            // Default fallback if calibration not run
            return None;
        }

        // Convert ADC to bipolar signal relative to baseline
        let mut sample = (raw_adc - self.baseline) / self.baseline;

        // Band-pass filtering
        sample = self.filter_high.run(sample);
        sample = self.filter_notch.run(sample);
        sample = self.filter_low.run(sample);

        // Rectify
        sample = sample.abs();

        // Envelope smoothing
        sample = self.filter_post_rect.run(sample);

        // moving average
        self.sum -= self.buffer[self.buffer_index];   // remove oldest
        self.buffer[self.buffer_index] = sample;      // insert newest
        self.sum += sample;                           // update sum
        self.buffer_index = (self.buffer_index + 1) % self.ma_size;

        if self.buffer_index == 0 {
            self.window_filled = true;
        }

        if self.window_filled {
            Some(self.sum / self.ma_size as f32)
        } else {
            None
        }
    }
}
