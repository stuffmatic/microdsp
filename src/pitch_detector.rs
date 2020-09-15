use std::vec;

use crate::equal_loudness_filter::EqualLoudnessFilter;
use crate::pitch_detection_result::PitchDetectionResult;

/// Handles windowing and preprocessing of input samples.
pub struct PitchDetector {
    sample_rate: usize,
    window_size: usize,
    window_distance: usize,
    window_write_index: usize,
    window: Vec<f32>,          // TODO: should be a slice
    filtered_window: Vec<f32>, // TODO: should be a slice
    has_full_window: bool,
    equal_loudness_filter: EqualLoudnessFilter,
    pub result: PitchDetectionResult,
}

/// The result of passing a chunk to the pitch detector.
pub enum ProcessingResult {
    /// Enough samples were consumed to fill and process a new window.
    /// `sample_index` is the index of the next sample to process in the input chunk.
    ProcessedWindow { sample_index: usize },
    /// Reached the last sample of the chunk.
    ReachedEndOfBuffer,
}

impl PitchDetector {
    pub fn new(
        sample_rate: usize,
        window_size: usize,
        window_distance: usize,
        use_equal_loudness_filter: bool,
    ) -> PitchDetector {
        let lag_count = window_size;
        PitchDetector {
            sample_rate,
            window_size,
            window_distance,
            window_write_index: 0,
            window: vec![0.0; window_size],
            filtered_window: vec![0.0; window_size],
            has_full_window: false,
            equal_loudness_filter: EqualLoudnessFilter::new(sample_rate as f32),
            result: PitchDetectionResult::new(window_size, lag_count),
        }
    }

    pub fn process(&mut self, samples: &[f32], sample_offset: usize) -> ProcessingResult {
        let mut did_process_window = false;
        for sample_index in sample_offset..samples.len() {
            // Accumulate this sample
            self.window[self.window_write_index] = samples[sample_index];

            // Advance the window write index, wrapping around the end of the window.
            self.window_write_index = (self.window_write_index + 1) % self.window_size;

            // If we replaced the last blank sample of the window, remember
            // that we have a full window from now on.
            if self.window_write_index == 0 && !self.has_full_window {
                self.has_full_window = true
            }

            if self.has_full_window && self.window_write_index % self.window_distance == 0 {
                // Time to process the current window
                // TODO: handle filtering differently, since the windows overlap.
                // self.equal_loudness_filter
                //    .process(&self.window[..], &mut self.filtered_window[..]);

                // Extract the buffer to analyze.
                // The start sample of the buffer to analyze may not be at the start
                // of the accumulated window.
                for target_index in 0..self.window_size {
                    let src_index = (self.window_write_index + target_index) % self.window_size;
                    self.result.window[target_index] = self.window[src_index];
                }

                // Analyze the current buffer.
                self.result.compute(self.sample_rate as f32);

                did_process_window = true;
            }

            // Notify the caller that a new pitch has been computed
            if did_process_window {
                return ProcessingResult::ProcessedWindow {
                    sample_index: sample_index + 1,
                };
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
        let window_distance = window_size;
        let f: f32 = 467.0;
        let sample_rate: f32 = 44100.0;
        let mut window: Vec<f32> = vec![0.0; window_size];
        for i in 0..window_size {
            let sine_value = (2.0 * std::f32::consts::PI * f * (i as f32) / sample_rate).sin();
            window[i] = sine_value;
        }
        let mut detector =
            PitchDetector::new(sample_rate as usize, window_size, window_distance, true);

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
    fn test_windowing() {
        // generate a buffer of 0,1,2,3...n - 1, 0,1,2,3...n
        let window_sizes: [usize; 4] = [10, 10, 1024, 1333];
        let window_distances: [usize; 4] = [1, 11, 512, 127];

        for window_size in window_sizes.iter() {
            for window_distance in window_distances.iter() {
                let mut buffer: Vec<f32> = vec![0.0; 2 * window_size];
                for i in 0..*window_size {
                    let value = (i % window_size) as f32;
                    buffer[i] = value;
                    buffer[i + window_size] = value;
                }

                let mut detector = PitchDetector::new(44100, *window_size, *window_distance, true);

                // Verify that the buffer to process in callback i starts with the value i
                let mut result_count = 0;
                let mut sample_offset: usize = 0;

                while sample_offset < buffer.len() {
                    match detector.process(&buffer[..], sample_offset) {
                        ProcessingResult::ProcessedWindow { sample_index } => {
                            // The sample index should advance in steps equal to the window distance
                            // except for the first time, where the step should equal the window size.
                            if result_count == 0 {
                                assert_eq!(sample_index, *window_size);
                            } else {
                                assert_eq!(sample_index - sample_offset, *window_distance);
                            }
                            sample_offset = sample_index;

                            // The sample offset should never be less than the window size
                            assert!(sample_offset >= *window_size);

                            //
                            let first_window_sample = detector.result.window[0];
                            assert_eq!(first_window_sample as usize, result_count % window_size);
                            result_count += 1;
                        }
                        _ => break,
                    }
                }
            }
        }
    }
}
