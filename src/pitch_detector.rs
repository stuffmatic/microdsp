use std::vec;

use crate::equal_loudness_filter::EqualLoudnessFilter;
use crate::pitch_detection_result::PitchDetectionResult;

/// Handles collecting input samples into (possibly overlapping) windows
/// and performs pitch detection on each newly filled window.
/// TODO: ALSO HANDLES PREPROCESSING.
pub struct PitchDetector {
    /// The audio sample rate in Hz.
    sample_rate: usize,
    /// The size of the windows to analyze.
    window_size: usize,
    /// The number of samples that consecutive windows
    /// overlap. Must be less than `window_size`
    window_overlap: usize,
    window_distance_counter: usize, // TODO: rename
    input_buffer_write_index: usize,
    input_buffer: Vec<f32>,          // TODO: should be a slice
    filtered_input_buffer: Vec<f32>, // TODO: should be a slice
    has_filled_input_buffer: bool,
    equal_loudness_filter: EqualLoudnessFilter,
    pub result: PitchDetectionResult,
}

/// The result of passing a chunk to the pitch detector.
pub enum ProcessingResult {
    /// Enough samples were consumed to fill and process a new window.
    /// `sample_index` is the index of the next sample to process in the input chunk,
    /// i.e the sample offset to pass to the next call to `process`.
    ProcessedWindow { sample_index: usize },
    /// Reached the last sample of the chunk.
    ReachedEndOfBuffer,
}

impl PitchDetector {
    pub fn new(
        sample_rate: usize,
        window_size: usize,
        window_overlap: usize,
        use_equal_loudness_filter: bool,
    ) -> PitchDetector {
        let lag_count = window_size;

        if window_size == 0 {
            panic!("Window size must be greater than 0")
        }

        if window_overlap >= window_size {
            panic!("Window overlap must be less than window size.")
        }

        PitchDetector {
            sample_rate,
            window_size,
            window_overlap,
            window_distance_counter: 0,
            input_buffer_write_index: 0,
            input_buffer: vec![0.0; window_size],
            filtered_input_buffer: vec![0.0; window_size],
            has_filled_input_buffer: false,
            equal_loudness_filter: EqualLoudnessFilter::new(sample_rate as f32),
            result: PitchDetectionResult::new(window_size, lag_count),
        }
    }

    pub fn process(&mut self, samples: &[f32], sample_offset: usize) -> ProcessingResult {
        let window_distance = self.window_size - self.window_overlap;
        for sample_index in sample_offset..samples.len() {
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
                        let src_index = (self.input_buffer_write_index + target_index) % self.window_size;
                        self.result.window[target_index] = self.input_buffer[src_index];
                    }

                    // Perform pitch detection
                    self.result.compute(self.sample_rate as f32);
                }
                self.window_distance_counter = (self.window_distance_counter + 1) % window_distance;
                if should_process_window {
                    return ProcessingResult::ProcessedWindow {
                        sample_index: sample_index + 1,
                    }
                }
            }
        }

        ProcessingResult::ReachedEndOfBuffer
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
        let mut detector =
            PitchDetector::new(sample_rate as usize, window_size, window_overlap, true);

        let mut sample_offset: usize = 0;
        while sample_offset < window.len() {
            match detector.process(&window[..], sample_offset) {
                ProcessingResult::ProcessedWindow { sample_index } => {
                    sample_offset = sample_index;

                    // All windows should have (pretty much) the same pitch.
                    assert!((f - detector.result.frequency).abs() <= 0.001);
                }
                _ => break,
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_invalid_overlap() {
        PitchDetector::new(44100, 100, 101, true);
    }

    #[test]
    #[should_panic]
    fn test_zero_window_size() {
        PitchDetector::new(44100, 0, 0, true);
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
            let value = if is_start_of_window { window_index } else { 100 * window_index + i } ;
            buffer[i] = value as f32;
        }

        let mut detector = PitchDetector::new(44100, window_size, window_overlap, true);

        // Verify that the buffer to process in callback i starts with the value i
        let mut result_count = 0;
        let mut sample_offset: usize = 0;

        while sample_offset < buffer.len() {
            match detector.process(&buffer[..], sample_offset) {
                ProcessingResult::ProcessedWindow { sample_index } => {
                    // The sample index should advance in steps equal to the window distance
                    // except for the first time, where the step should equal the window size.
                    if result_count == 0 {
                        assert_eq!(sample_index, window_size);
                    } else {
                        if sample_index - sample_offset != window_distance {
                            let a = 0;
                            assert_eq!(sample_index - sample_offset, window_distance);
                        }
                    }
                    sample_offset = sample_index;

                    // The sample offset should never be less than the window size
                    assert!(sample_offset >= window_size);

                    //
                    let first_window_sample = detector.result.window[0];
                    assert_eq!(first_window_sample as usize, result_count);
                    result_count += 1;
                }
                _ => break,
            }
        }
    }
}
