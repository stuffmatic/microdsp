use crate::common::WindowProcessor;
use crate::mpm::result::MpmPitchResult;

pub struct MpmPitchDetector {
    sample_rate: f32,
    window_processor: WindowProcessor,
    result: MpmPitchResult,
}

impl MpmPitchDetector {
    pub fn new(sample_rate: f32, window_size: usize, hop_size: usize) -> Self {
        MpmPitchDetector::from_options(sample_rate, window_size, hop_size, window_size / 2, 1)
    }

    pub fn from_options(
        sample_rate: f32,
        downsampled_window_size: usize,
        downsampled_hop_size: usize,
        downsampled_lag_count: usize,
        downsampling: usize,
    ) -> Self {
        // TODO: validate lag count

        MpmPitchDetector {
            sample_rate,
            result: MpmPitchResult::new(downsampled_window_size, downsampled_lag_count),
            window_processor: WindowProcessor::new(
                downsampling,
                downsampled_window_size,
                downsampled_hop_size,
            ),
        }
    }

    pub fn process<F>(&mut self, buffer: &[f32], mut result_handler: F)
    where
        F: FnMut(&MpmPitchResult),
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
    pub fn result(&self) -> &MpmPitchResult {
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
        let hop_size = 512;
        let frequency: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let window = generate_sine(sample_rate, frequency, window_size);

        let mut detector = MpmPitchDetector::new(sample_rate, window_size, hop_size);

        detector.process(&window[..], |result: &MpmPitchResult| {
            assert!((frequency - result.frequency).abs() <= 0.001);
        });
    }

    #[test]
    fn test_downsampled_sine_detection() {
        let window_size = 2048;
        let lag_count = window_size / 2;
        let hop_size = window_size;
        let frequency: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let window = generate_sine(sample_rate, frequency, window_size);
        let downsampling_factor = 4;
        let mut detector = MpmPitchDetector::from_options(
            sample_rate,
            window_size,
            hop_size,
            lag_count,
            downsampling_factor,
        );
        let downsampled_window_size = detector.downsampled_window_size();

        detector.process(&window[..], |result: &MpmPitchResult| {
            assert!(result.window.len() == downsampled_window_size);
            assert!((frequency - result.frequency).abs() <= 0.05);
        });
        detector.process(&window[..], |result: &MpmPitchResult| {
            assert!(result.window.len() == downsampled_window_size);
            assert!((frequency - result.frequency).abs() <= 0.05);
        });
    }
}
