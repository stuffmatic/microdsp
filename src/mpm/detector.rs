use crate::common::window_processor::WindowProcessor;
use crate::mpm::result::Result;

pub struct Detector {
    sample_rate: f32,
    window_processor: WindowProcessor,
    result: Result,
}

impl Detector {
    pub fn new(sample_rate: f32, window_size: usize, hop_size: usize) -> Self {
        Detector::from_options(sample_rate, window_size, hop_size, window_size / 2, 1)
    }

    pub fn from_options(
        sample_rate: f32,
        downsampled_window_size: usize,
        downsampled_hop_size: usize,
        downsampled_lag_count: usize,
        downsampling: usize,
    ) -> Self {
        // TODO: validate lag count

        Detector {
            sample_rate,
            result: Result::new(downsampled_window_size, downsampled_lag_count),
            window_processor: WindowProcessor::new(
                downsampled_window_size,
                downsampled_hop_size,
                downsampling,
            ),
        }
    }

    pub fn process<F>(&mut self, buffer: &[f32], mut result_handler: F)
    where
        F: FnMut(&Result),
    {
        let result = &mut self.result;
        let downsampling = self.window_processor.downsampling();
        let sample_rate = self.sample_rate;
        self.window_processor.process(buffer, |window| {
            result.window.copy_from_slice(window); // TODO: this copy could be avoided
            result.compute(sample_rate / (downsampling as f32));
            result_handler(result);
        });
    }

    /// Returns the most recently computed pitch detection result.
    pub fn result(&self) -> &Result {
        &self.result
    }

    /// Returns the current sample rate in Hz.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Sets the sample rate in Hz.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn downsampled_window_size(&self) -> usize {
        self.window_processor.downsampled_window_size()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::alloc::vec::Vec;

    fn generate_sine(sample_rate: f32, frequency: f32, sample_count: usize) -> Vec<f32> {
        let mut window: Vec<f32> = vec![0.0; sample_count];
        for i in 0..sample_count {
            let sine_value =
                (2.0 * core::f32::consts::PI * frequency * (i as f32) / sample_rate).sin();
            window[i] = sine_value;
        }
        return window;
    }

    #[test]
    fn test_sine_detection() {
        let window_size = 1024;
        let window_distance = 512;
        let frequency: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let window = generate_sine(sample_rate, frequency, window_size);

        let mut detector = Detector::new(sample_rate, window_size, window_distance);

        detector.process(&window[..], |result: &Result| {
            assert!((frequency - result.frequency).abs() <= 0.001);
        });
    }

    #[test]
    fn test_downsampled_sine_detection() {
        let window_size = 2048;
        let lag_count = window_size / 2;
        let window_distance = window_size;
        let frequency: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let window = generate_sine(sample_rate, frequency, window_size);
        let downsampling_factor = 4;
        let mut detector = Detector::from_options(
            sample_rate,
            window_size,
            window_distance,
            lag_count,
            downsampling_factor,
        );
        let downsampled_window_size = detector.downsampled_window_size();

        detector.process(&window[..], |result: &Result| {
            assert!(result.window.len() == downsampled_window_size);
            assert!((frequency - result.frequency).abs() <= 0.05);
        });
        detector.process(&window[..], |result: &Result| {
            assert!(result.window.len() == downsampled_window_size);
            assert!((frequency - result.frequency).abs() <= 0.05);
        });
    }

    #[test]
    #[should_panic]
    fn test_zero_downsampling_factor() {
        let _ = Detector::from_options(44100., 512, 256, 256, 0);
    }

    #[test]
    #[should_panic]
    fn test_nondivisible_downsampling_factor_1() {
        // Make sure we panic if the window size is not evenly divisible by the downsampling factor
        let _ = Detector::from_options(44100., 521, 256, 256, 4);
    }

    #[test]
    #[should_panic]
    fn test_nondivisible_downsampling_factor_2() {
        // Make sure we panic if the window distance is not evenly divisible by the downsampling factor
        let _ = Detector::from_options(44100., 512, 250, 256, 4);
    }

    #[test]
    #[should_panic]
    fn test_zero_window_size() {
        Detector::new(44100.0, 0, 0);
    }

    #[test]
    #[should_panic]
    fn test_zero_window_distance() {
        Detector::new(44100.0, 10, 0);
    }

    #[test]
    #[should_panic]
    fn test_too_large_window_distance() {
        Detector::new(44100.0, 10, 11);
    }
}
