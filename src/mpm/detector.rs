use crate::alloc::vec;
use crate::alloc::boxed::Box;
use crate::mpm::result::Result;

/// * Collects input samples into (possibly overlapping) windows
/// * Performs pitch detection on each newly filled window
/// * Handles downsampling
pub struct Detector {
    /// The audio sample rate in Hz.
    sample_rate: f32,
    /// The size of the windows to analyze.
    window_size: usize,
    /// The number of samples between consecutive (possibly overlapping)
    /// windows. Must not be greater than `window_size`.
    window_distance: usize,
    /// For counting the number of samples from the start of the previous window.
    window_distance_counter: usize,
    processed_window_count: usize,
    input_buffer_write_index: usize,
    input_buffer: Box<[f32]>,
    has_filled_input_buffer: bool,
    result: Result,
    downsampling_factor: usize,
    sample_skip: usize // TODO: rename this
}

impl Detector {
    pub fn new(sample_rate: f32, window_size: usize, window_distance: usize) -> Self {
        Detector::from_options(sample_rate, window_size, window_distance, window_size / 2, 1)
    }

    pub fn from_options(sample_rate: f32, window_size: usize, window_distance: usize, lag_count: usize, downsampling_factor: usize) -> Self {
        if window_size == 0 {
            panic!("Window size must be greater than 0")
        }
        if window_distance > window_size || window_distance <= 0 {
            panic!("Window distance must be > 0 and <= window_size")
        }
        if downsampling_factor == 0 {
            panic!("Downsampling factor must be greater than 0")
        }
        if window_size % downsampling_factor != 0 {
            panic!("window_size must be evenly divisible by downsampling_factor")
        }
        if window_distance % downsampling_factor != 0 {
            panic!("window_distance must be evenly divisible by downsampling_factor")
        }

        // TODO: validate lag count

        let downsampled_window_size = window_size / downsampling_factor;
        let downsampled_lag_count = lag_count / downsampling_factor;

        Detector {
            sample_rate,
            window_size,
            window_distance,
            window_distance_counter: 0,
            processed_window_count: 0,
            input_buffer_write_index: 0,
            input_buffer: (vec![0.0; downsampled_window_size]).into_boxed_slice(),
            has_filled_input_buffer: false,
            result: Result::new(downsampled_window_size, downsampled_lag_count),
            downsampling_factor,
            sample_skip: 0
        }
    }

    pub fn process<F>(&mut self, samples: &[f32], mut result_handler: F) -> bool
    where
        F: FnMut(usize, &Result),
    {
        for (sample_index, _) in samples.iter().enumerate().skip(self.sample_skip).step_by(self.downsampling_factor) {
            // Accumulate this sample
            self.input_buffer[self.input_buffer_write_index] = samples[sample_index];

            // Advance write index, wrapping around the end of the input buffer
            self.input_buffer_write_index = (self.input_buffer_write_index + 1) % self.downsampled_window_size();

            if !self.has_filled_input_buffer && self.input_buffer_write_index == 0 {
                // This is the first time the write index wrapped around to zero,
                // meaning we have filled the entire input buffer.
                self.has_filled_input_buffer = true
            }

            if self.has_filled_input_buffer {
                let should_process_window = self.window_distance_counter == 0;
                if should_process_window {
                    // Extract the window to analyze.
                    for target_index in 0..self.downsampled_window_size() {
                        let src_index =
                            (self.input_buffer_write_index + target_index) % self.downsampled_window_size();
                        self.result.window[target_index] = self.input_buffer[src_index];
                    }

                    // Perform pitch detection
                    self.result.compute(self.sample_rate / (self.downsampling_factor as f32));
                    self.processed_window_count += 1;
                }
                self.window_distance_counter =
                    (self.window_distance_counter + 1) % self.downsampled_window_distance();
                if should_process_window {
                    result_handler(sample_index + self.downsampling_factor, &self.result);
                }
            }
        }

        self.sample_skip = (self.sample_skip + samples.len()) % self.downsampling_factor;

        false
    }

    /// Returns the most recently computed pitch detection result.
    pub fn result(&self) -> &Result {
        &self.result
    }

    /// Returns the number of processed windows since the
    /// detector was created.
    pub fn processed_window_count(&self) -> usize {
        self.processed_window_count
    }

    /// Returns the fixed number of samples in a window.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Returns the number of samples between windows.
    pub fn window_distance(&self) -> usize {
        self.window_distance
    }

    /// Returns the downsampling factor
    pub fn downsampling_factor(&self) -> usize {
        self.downsampling_factor
    }

    /// Returns the current sample rate in Hz.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Sets the sample rate in Hz.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn downsampled_window_size(&self) -> usize {
        self.window_size / self.downsampling_factor
    }

    fn downsampled_window_distance(&self) -> usize {
        self.window_distance / self.downsampling_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::vec::Vec;

    fn generate_sine(sample_rate: f32, frequency: f32, sample_count: usize) -> Vec<f32> {
        let mut window: Vec<f32> = vec![0.0; sample_count];
        for i in 0..sample_count {
            let sine_value = (2.0 * core::f32::consts::PI * frequency * (i as f32) / sample_rate).sin();
            window[i] = sine_value;
        }
        return window
    }

    #[test]
    fn test_sine_detection() {
        let window_size = 1024;
        let window_distance = 512;
        let frequency: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let window = generate_sine(sample_rate, frequency, window_size);

        let mut detector = Detector::new(sample_rate, window_size, window_distance);

        detector.process(&window[..], |_: usize, result: &Result| {
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
        let mut detector = Detector::from_options(sample_rate, window_size, window_distance, lag_count, downsampling_factor);
        let downsampled_window_size = detector.downsampled_window_size();

        detector.process(&window[..], |_: usize, result: &Result| {
            assert!(result.window.len() == downsampled_window_size);
            assert!((frequency - result.frequency).abs() <= 0.05);
        });
        detector.process(&window[..], |_: usize, result: &Result| {
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

    #[test]
    fn test_windowing() {
        run_windowing_test(10, 4, 1);
        run_windowing_test(10, 10, 2);
        run_windowing_test(40, 20, 4);
        run_windowing_test(10, 1, 1);
    }

    fn run_windowing_test(window_size: usize, window_distance: usize, downsampling_factor: usize) {
        let mut buffer: Vec<f32> = vec![0.0; 2 * window_size];
        for i in 0..buffer.len() {
            let is_start_of_window = i % window_distance == 0;
            let window_index = i / window_distance;
            let value = if is_start_of_window {
                window_index
            } else {
                100 * window_index + i
            };
            buffer[i] = value as f32;
        }

        let mut detector = Detector::from_options(44100.0, window_size, window_distance,window_size / 2, downsampling_factor);

        // Verify that the buffer to process in callback i starts with the value i
        let mut result_count = 0;
        let mut sample_offset: usize = 0;

        detector.process(&buffer[..], |sample_index, result| {
            // The sample index should advance in steps equal to the window distance
            // except for the first time, where the step should equal the window size.
            if result_count == 0 {
                assert_eq!(sample_index, window_size);
            } else {
                if sample_index - sample_offset != window_distance {
                    assert_eq!(sample_index - sample_offset, window_distance);
                }
            }
            sample_offset = sample_index;

            // The sample offset should never be less than the window size
            assert!(sample_offset >= window_size);

            //
            let first_window_sample = result.window[0];
            assert_eq!(first_window_sample as usize, result_count);
            result_count += 1;
        });
    }
}
