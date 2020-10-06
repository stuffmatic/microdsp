use std::vec;

use crate::result::Result;

/// Handles collecting input samples into (possibly overlapping) windows
/// and performs pitch detection on each newly filled window.
/// TODO: ALSO HANDLES PREPROCESSING.
pub struct Detector {
    /// The audio sample rate in Hz.
    sample_rate: f32,
    /// The size of the windows to analyze.
    window_size: usize,
    /// The number of samples that consecutive windows
    /// have in common, i.e the last `window_overlap` samples of window
    /// `i` are the same as the first `window_overlap` of window `i + 1`.
    /// Must be less than `window_size`.
    window_overlap: usize,
    window_distance_counter: usize, // TODO: rename
    input_buffer_write_index: usize,
    input_buffer: Box<[f32]>,
    has_filled_input_buffer: bool,
    result: Result,
}

impl Detector {
    pub fn new(sample_rate: f32, window_size: usize, window_overlap: usize) -> Detector {
        let lag_count = window_size / 2;

        if window_size == 0 {
            panic!("Window size must be greater than 0")
        }

        if window_overlap >= window_size {
            panic!("Window overlap must be less than window size.")
        }

        Detector {
            sample_rate,
            window_size,
            window_overlap,
            window_distance_counter: 0,
            input_buffer_write_index: 0,
            input_buffer: (vec![0.0; window_size]).into_boxed_slice(),
            has_filled_input_buffer: false,
            result: Result::new(window_size, lag_count),
        }
    }

    pub fn process_window(&mut self, samples: &[f32]) -> &Result {
        if samples.len() != self.window_size {
            panic!("The input buffer size must equal the window size")
        }
        self.result.window.copy_from_slice(&samples[..]);
        self.result.compute(self.sample_rate);
        &self.result
    }

    pub fn process<F>(&mut self, samples: &[f32], mut result_handler: F) -> bool
    where
        F: FnMut(usize, &Result),
    {
        let window_distance = self.window_size - self.window_overlap;
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
                }
                self.window_distance_counter = (self.window_distance_counter + 1) % window_distance;
                if should_process_window {
                    result_handler(sample_index + 1, &self.result);
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_detection() {
        let window_size = 1024;
        let window_overlap = 512;
        let f: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let mut window: Vec<f32> = vec![0.0; window_size];
        for i in 0..window_size {
            let sine_value = (2.0 * std::f32::consts::PI * f * (i as f32) / sample_rate).sin();
            window[i] = sine_value;
        }
        let mut detector = Detector::new(sample_rate, window_size, window_overlap);

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
    fn test_windowing() {
        run_windowing_test(10, 4);
        run_windowing_test(10, 0);
        run_windowing_test(10, 1);
    }

    fn run_windowing_test(window_size: usize, window_overlap: usize) {
        let mut buffer: Vec<f32> = vec![0.0; 2 * window_size];
        let window_distance = window_size - window_overlap;
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

        let mut detector = Detector::new(44100.0, window_size, window_overlap);

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
