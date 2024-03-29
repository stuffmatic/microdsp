use micromath::F32Ext;

use crate::alloc::boxed::Box;
use crate::alloc::vec;
use crate::common::freq_to_midi_note;
use crate::common::{autocorr_fft, autocorr_fft_size};
use crate::mpm::key_max::KeyMax;
use crate::mpm::util;

/// The maximum number of key maxima to gather during the peak finding phase.
pub const MAX_KEY_MAXIMA_COUNT: usize = 64;
/// A pitch detection result.
pub struct MpmPitchResult {
    /// The estimated pitch frequency in Hz.
    pub frequency: f32,
    /// The value of the NSDF at the maximum corresponding to the pitch period.
    /// Between 0 and 1 (inclusive). Pure tones have a value close to 1. Note
    /// that non-tonal input may also result in occasional clarity peaks. For
    /// a more robust way of differentiating between tonal and non-tonal input,
    /// see `is_tone`.
    pub clarity: f32,
    /// The [MIDI note number](https://newt.phys.unsw.edu.au/jw/notes.html) corresponding to the pitch frequency.
    pub midi_note_number: f32,
    /// The estimated pitch period in samples.
    pub pitch_period: f32,
    /// The analyzed window.
    pub window: Box<[f32]>,
    /// The normalized square difference function
    pub nsdf: Box<[f32]>,
    /// The number of key maxima found during the peak picking phase. May be 0, in which case
    /// the result is considered invalid.
    pub key_max_count: usize,
    /// A fixed array of key maxima. The first `key_max_count` maxima are valid.
    pub key_maxima: Box<[KeyMax]>,
    /// The index into `key_maxima` of the selected key maximum
    pub selected_key_max_index: usize,
    ///
    r_prime: Box<[f32]>,
    scratch_buffer: Box<[f32]>,
}

impl MpmPitchResult {
    pub fn new(window_size: usize, lag_count: usize) -> Self {
        // Allocate buffers
        let window = (vec![0.0; window_size]).into_boxed_slice();
        let nsdf = (vec![0.0; lag_count]).into_boxed_slice();
        let r_prime = (vec![0.0; autocorr_fft_size(window_size, lag_count)]).into_boxed_slice();
        let scratch_buffer =
            (vec![0.0; autocorr_fft_size(window_size, lag_count)]).into_boxed_slice();

        // Create the instance
        MpmPitchResult {
            frequency: 0.0,
            clarity: 0.0,
            midi_note_number: 0.0,
            window,
            nsdf,
            r_prime,
            scratch_buffer,
            key_max_count: 0,
            key_maxima: vec![KeyMax::new(); MAX_KEY_MAXIMA_COUNT].into_boxed_slice(),
            selected_key_max_index: 0,
            pitch_period: 0.0,
        }
    }

    /// Performs pitch detection on the current contents of `window`.
    pub fn compute(&mut self, sample_rate: f32) {
        self.reset();
        self.compute_nsdf();
        self.perform_peak_picking();
        self.compute_pitch(sample_rate);
    }

    /// Indicates if the detection result has a valid pitch estimate. Note that this does not necessarily
    /// mean that the result corresponds to a tone. See `is_tone` and `is_tone_with_options`.
    pub fn is_valid(&self) -> bool {
        self.key_max_count > 0
    }

    /// Returns the lowest detectable frequency in Hz at a give sample rate.
    pub fn min_detectable_frequency(&self, sample_rate: f32) -> f32 {
        sample_rate / (self.nsdf.len() as f32)
    }

    /// Returns the number of the lowest detectable MIDI note at a give sample rate.
    pub fn min_detectable_note_number(&self, sample_rate: f32) -> f32 {
        freq_to_midi_note(self.min_detectable_frequency(sample_rate))
    }

    /// Returns true if the input window has a discernable fundamental frequency. False otherwise.
    pub fn is_tone(&self) -> bool {
        self.is_tone_with_options(0.9, 0.5, 0.05)
    }

    /// Returns true if the input window has a discernable fundamental frequency. False otherwise.
    /// Compares the selected key maximum _m_, and the key maximum _n_ closest to the double period to
    /// a number of thresholds.
    /// # Arguments
    ///
    /// * `clarity_threshold` - The clarity of _m_ must be greater than this value.
    /// * `clarity_tolerance` - The clarity of _n_ must not be more than this below the clarity of _m_.
    /// * `period_tolerance` - The relative difference between the lag of _m_ and the lag difference between _n_ and _m_ must not be greater than this value.
    pub fn is_tone_with_options(
        &self,
        clarity_threshold: f32,
        clarity_tolerance: f32,
        period_tolerance: f32,
    ) -> bool {
        if !self.is_valid() {
            // No key maxima, can't be a tone
            return false;
        }

        let is_tone = match self.key_max_closest_to_double_period() {
            Some(next_max) => {
                let max = self.key_maxima[self.selected_key_max_index];

                // Does the closest max meet the period tolerance, i.e was the key max closest
                // to the double period found at a lag sufficiently close to the double period?
                let delta_lag = next_max.lag - max.lag;
                let rel_lag_error = F32Ext::abs(delta_lag - max.lag) / max.lag;
                let meets_period_tolerance = rel_lag_error < period_tolerance;

                // Does the closest max meet the clarity tolerance, i.e does the key max closest
                // to the double period have a sufficiently high clarity?
                let delta_clarity = next_max.value - max.value;
                let meets_clarity_tolerance = delta_clarity > -clarity_tolerance;

                // println!("rel_lag_difference {}, delta_value {}", rel_lag_difference, delta_value);
                self.clarity > clarity_threshold
                    && meets_period_tolerance
                    && meets_clarity_tolerance
            }
            None => self.clarity > clarity_threshold,
        };
        is_tone
    }

    fn key_max_closest_to_double_period(&self) -> Option<KeyMax> {
        if self.key_max_count == 0 {
            return None;
        }

        let selected_max = &self.key_maxima[self.selected_key_max_index];
        let lag_of_next_expected_max = 2.0 * selected_max.lag;
        let mut min_distance: f32 = 0.;
        let mut min_index: usize = 0;
        let mut found_max = false;
        let start_index = self.selected_key_max_index + 1;
        for i in start_index..self.key_max_count {
            let key_max = self.key_maxima[i];
            if key_max.lag_index == self.nsdf.len() - 1 {
                // Ignore the key max at the last lag, since it's
                // probably not a proper key maximum.
                break;
            }
            let distance = (key_max.lag - lag_of_next_expected_max).abs();
            if i == start_index {
                min_distance = distance;
                min_index = i;
            } else {
                if distance < min_distance {
                    min_distance = distance;
                    min_index = i;
                }
            }
            found_max = true;
        }

        if found_max {
            assert!(min_index > self.selected_key_max_index);
            return Some(self.key_maxima[min_index]);
        }
        None
    }

    fn reset(&mut self) {
        self.frequency = 0.0;
        self.clarity = 0.0;
        self.midi_note_number = 0.0;
        self.key_max_count = 0;
        self.selected_key_max_index = 0;
        self.pitch_period = 0.0;
    }

    fn perform_peak_picking(&mut self) {
        let nsdf = &mut self.nsdf[..];

        // Perform peak picking.
        // Step 1: gather key maxima.
        self.key_max_count = 0;
        let mut is_detecting = false;
        let mut maximum_value: f32 = 0.0;
        let mut maximum_index: usize = 0;
        let mut prev = nsdf[0];
        for i in 1..nsdf.len() {
            let is_last_lag = i == nsdf.len() - 1;
            let curr = nsdf[i];
            if prev <= 0.0 && curr > 0.0 {
                // positive zero crossing, going from - to +.
                // start looking for a key maximum
                is_detecting = true;
                maximum_value = curr;
                maximum_index = i;
            } else if prev >= 0.0 && curr < 0.0 {
                // We reached a negative zero crossing (going from + to -) or the last lag.
                // Stop looking for a key maximum and store the one we've got
                // (unless we have collected the maximum number of key maxima)
                if is_detecting && self.key_max_count < self.key_maxima.len() {
                    self.key_maxima[self.key_max_count].set(&nsdf, maximum_index);
                    self.key_max_count += 1
                }
                is_detecting = false;
            }

            if is_detecting {
                if is_last_lag {
                    // Reached the last lag while looking for a new max.
                    if self.key_max_count < self.key_maxima.len() {
                        let last_max_index = if curr > maximum_value {
                            i
                        } else {
                            maximum_index
                        };
                        self.key_maxima[self.key_max_count].set(&nsdf, last_max_index);
                        self.key_max_count += 1
                    }
                } else if curr > maximum_value {
                    // If we're looking for a key maximum and the current
                    // value is greater than the current max, set a new max.
                    maximum_value = curr;
                    maximum_index = i;
                }
            }

            prev = curr;
        }

        // Step 2: Find the largest key maximum
        let mut largest_key_maximum: f32 = 0.0;
        for (i, key_max) in self.key_maxima.iter().enumerate() {
            let value = key_max.value_at_lag_index;
            if value > largest_key_maximum || i == 0 {
                largest_key_maximum = value;
            }
        }

        // Step 3: Select the final maximum
        let k: f32 = 0.9;
        let threshold = k * largest_key_maximum;
        for (key_max_index, key_max) in self.key_maxima.iter().take(self.key_max_count).enumerate()
        {
            if key_max.value >= threshold {
                self.selected_key_max_index = key_max_index;
                break;
            }
        }
    }

    /// Computes pitch parameters from the currently selected key maximum.
    fn compute_pitch(&mut self, sample_rate: f32) {
        if self.key_max_count > 0 {
            let selected_max = self.key_maxima[self.selected_key_max_index];

            self.pitch_period = selected_max.lag;
            self.clarity = if selected_max.value > 1.0 {
                1.0
            } else {
                selected_max.value
            };

            let pitch_period = self.pitch_period / sample_rate;
            self.frequency = 1.0 / pitch_period;
            self.midi_note_number = freq_to_midi_note(self.frequency);
        }
    }

    /// Computes the normalized square difference function from the current contents of `window`.
    fn compute_nsdf(&mut self) {
        let window = &self.window[..];
        let nsdf = &mut self.nsdf[..];
        let mut r_prime = &mut self.r_prime[..];
        let mut scratch_buffer = &mut self.scratch_buffer[..];

        autocorr_fft(
            &self.window[..],
            &mut r_prime,
            &mut scratch_buffer,
            nsdf.len(),
        );

        // Compute m' and store it in the nsdf buffer
        let autocorr_at_lag_0 = r_prime[0];
        util::m_prime_incremental(window, autocorr_at_lag_0, nsdf);

        // Compute the NSDF as 2 * r' / m'
        for i in 0..nsdf.len() {
            let denominator = nsdf[i];
            nsdf[i] = if F32Ext::abs(denominator) <= f32::EPSILON {
                0.0
            } else {
                2.0 * r_prime[i] / denominator
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence() {
        let sample_rate = 44100.0;
        let window_size = 1024;

        let mut result = MpmPitchResult::new(window_size, window_size / 2);
        result.compute(sample_rate);
        assert_eq!(result.nsdf[0], 0.);
        assert_eq!(result.key_max_count, 0);
    }

    #[test]
    fn test_low_sine() {
        for f in [154.0_f32, 190.0_f32].iter() {
            let window_size = 1024;
            let lag_count = window_size / 2;
            let sample_rate: f32 = 44100.0;
            let expected_pitch_period = sample_rate / f;

            // Verify pre-condition
            assert!(expected_pitch_period < (lag_count as f32));

            // Generate a pure tone and perform pitch detection
            let mut result = MpmPitchResult::new(window_size, lag_count);
            for i in 0..window_size {
                let sine_value = (2.0 * core::f32::consts::PI * f * (i as f32) / sample_rate).sin();
                result.window[i] = sine_value;
            }

            result.compute(sample_rate);

            assert!(
                (f - result.frequency).abs() <= 0.001,
                "Wrong detected frequency"
            );
            // We should have one actual maximum and one maximum at the last NSDF sample
            assert_eq!(result.key_max_count, 2, "Unexpected key max count");

            // The value of the last key max should be reasonable
            let last_max = result.key_maxima[result.key_max_count - 1];
            let last_max_lag = last_max.lag;
            let last_max_lag_index = last_max.lag_index;
            let last_max_value = last_max.value;
            let last_max_value_at_lag_index = last_max.value;
            assert!(
                (last_max_lag - (last_max_lag_index as f32)).abs() < 1.,
                "Unreasonable interpolated key max lag"
            );
            assert!(
                (last_max_value - last_max_value_at_lag_index).abs() < 0.001,
                "Unreasonable interpolated key max value"
            );
        }
    }
}
