use std::vec;

use crate::result::Result;

/// Handles collecting input samples into (possibly overlapping) windows
/// and performing pitch detection on each newly filled window.
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
}

impl Detector {
    pub fn new(sample_rate: f32, window_size: usize, window_distance: usize) -> Self {
        let lag_count = window_size / 2;

        if window_size == 0 {
            panic!("Window size must be greater than 0")
        }
        if window_distance > window_size || window_distance <= 0 {
            panic!("Window distance must be > 0 and <= window_size")
        }

        Detector {
            sample_rate,
            window_size,
            window_distance,
            window_distance_counter: 0,
            processed_window_count: 0,
            input_buffer_write_index: 0,
            input_buffer: (vec![0.0; window_size]).into_boxed_slice(),
            has_filled_input_buffer: false,
            result: Result::new(window_size, lag_count),
        }
    }

    pub fn process<F>(&mut self, samples: &[f32], mut result_handler: F) -> bool
    where
        F: FnMut(usize, &Result),
    {
        for sample_index in 0..samples.len() {
            // Accumulate this sample
            self.input_buffer[self.input_buffer_write_index] = samples[sample_index];

            // Advance write index, wrapping around the end of the input buffer
            self.input_buffer_write_index = (self.input_buffer_write_index + 1) % self.window_size;

            if !self.has_filled_input_buffer && self.input_buffer_write_index == 0 {
                // This is the first time the write index wrapped around to zero,
                // meaning we have filled the entire input buffer.
                self.has_filled_input_buffer = true
            }

            if self.has_filled_input_buffer {
                let should_process_window = self.window_distance_counter == 0;
                if should_process_window {
                    // Extract the window to analyze.
                    for target_index in 0..self.window_size {
                        let src_index =
                            (self.input_buffer_write_index + target_index) % self.window_size;
                        self.result.window[target_index] = self.input_buffer[src_index];
                    }

                    // Perform pitch detection
                    self.result.compute(self.sample_rate as f32);
                    self.processed_window_count += 1;
                }
                self.window_distance_counter =
                    (self.window_distance_counter + 1) % self.window_distance;
                if should_process_window {
                    result_handler(sample_index + 1, &self.result);
                }
            }
        }

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
        self.window_size
    }

    /// Returns the current sample rate in Hz.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Sets the sample rate in Hz.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_detection() {
        let window_size = 1024;
        let window_distance = 512;
        let f: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let mut window: Vec<f32> = vec![0.0; window_size];
        for i in 0..window_size {
            let sine_value = (2.0 * std::f32::consts::PI * f * (i as f32) / sample_rate).sin();
            window[i] = sine_value;
        }
        let mut detector = Detector::new(sample_rate, window_size, window_distance);

        detector.process(&window[..], |sample_offset: usize, result: &Result| {
            assert!((f - result.frequency).abs() <= 0.001);
        });
    }

    #[test]
    #[should_panic]
    fn test_invalid_overlap() {
        Detector::new(44100.0, 100, 101);
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
        run_windowing_test(10, 4);
        run_windowing_test(10, 10);
        run_windowing_test(10, 1);
    }

    fn run_windowing_test(window_size: usize, window_distance: usize) {
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

        let mut detector = Detector::new(44100.0, window_size, window_distance);

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
