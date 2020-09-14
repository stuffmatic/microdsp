use microfft;
use std::vec;

use crate::key_maximum::KeyMaximum;
use crate::equal_loudness_filter::EqualLoudnessFilter;
use crate::pitch_detection_result::autocorr_fft_size;
use crate::pitch_detection_result::PitchDetectionResult;

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

impl PitchDetector {
    pub fn new(
        sample_rate: usize,
        window_size: usize,
        window_distance: usize,
        use_equal_loudness_filter: bool,
    ) -> PitchDetector {
        let lag_count = window_size;
        let fft_size = autocorr_fft_size(window_size, lag_count);
        PitchDetector {
            sample_rate,
            window_size,
            window_distance,
            window_write_index: 0,
            window: vec![0.0; window_size],
            filtered_window: vec![0.0; window_size],
            has_full_window: false,
            equal_loudness_filter: EqualLoudnessFilter::new(sample_rate as f32),
            result: PitchDetectionResult {
                frequency: 0.0,
                clarity: 0.0,
                note_number: 0.0,
                window: vec![0.0; window_size],
                nsdf: vec![0.0; lag_count],
                r_prime: vec![microfft::Complex32::new(0.0, 0.0); fft_size],
                key_max_count: 0,
                key_maxima: [KeyMaximum::new(); crate::pitch_detection_result::MAX_KEY_MAXIMA_COUNT],
                selected_key_max_index: 0,
                pitch_period: 0.0,
            },
        }
    }

    pub fn process<F: FnMut(usize, &PitchDetectionResult)>(&mut self, samples: &[f32], mut callback: F) {
        for (sample_index, sample) in samples.iter().enumerate() {
            // Accumulate this sample
            self.window[self.window_write_index] = *sample;

            // If we replaced the last blank sample of the window, remember
            // that we have a full window from now on.
            if self.window_write_index == self.window_size - 1 && !self.has_full_window {
                self.has_full_window = true
            }

            if self.has_full_window && (self.window_write_index + 1) % self.window_distance == 0 {
                // Time to process the current window
                // TODO: handle filtering differently, since the windows overlap.
                self.equal_loudness_filter
                    .process(&self.window[..], &mut self.filtered_window[..]);

                // Extract the buffer to analyze.
                // The start sample of the buffer to analyze may not be at the start
                // of the accumulated window.
                for target_index in 0..self.window_size {
                    let src_index = (target_index + self.window_write_index + 1) % self.window_size;
                    self.result.window[target_index] = self.filtered_window[src_index];
                }

                // Analyze the current buffer.
                self.result.compute(self.sample_rate as f32);

                // Notify the caller that a new pitch has been computed
                callback(sample_index, &self.result)
            }

            // Finally, advance the window write index, wrapping around the end of the window.
            self.window_write_index = (self.window_write_index + 1) % self.window_size;
        }
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
        detector.process(&window[..], |sample_index, result: &PitchDetectionResult| {});

        assert!((f - detector.result.frequency).abs() <= 0.001);
    }

    #[test]
    fn test_window_distance() {
        // generate a buffer of 0,1,2,3...n - 1, 0,1,2,3...n
        let window_size = 30;
        let mut buffer: Vec<f32> = vec![0.0; 2 * window_size];
        for i in 0..window_size {
            buffer[i] = i as f32;
            buffer[i + window_size] = i as f32;
        }

        // create detector with window distance 1
        let window_distance = 1;
        let mut detector = PitchDetector::new(44100, window_size, window_distance, true);

        // Verify that the buffer to process in callback i starts with the value i
        let mut result_count = 0;
        detector.process(&buffer[..], |sample_index: usize, result: &PitchDetectionResult| {
            let first_window_sample = result.window[0];
            assert_eq!(first_window_sample as usize, result_count % window_size);
            result_count += 1;
        });
    }
}
