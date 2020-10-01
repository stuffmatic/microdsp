use crate::key_maximum::KeyMaximum;

fn validate_window_size_lag_count(window_size: usize, lag_count: usize) {
    if lag_count > window_size {
        panic!("Lag count must not be greater than the window size");
    }
}

/// Computes m' defined in eq (6), using the incremental subtraction
/// algorithm described in section 6 - Efficient calculation of SDF.
fn m_prime_incremental(window: &[f32], autocorr_at_lag_0: f32, result: &mut [f32]) {
    let lag_count = result.len();
    let window_size = window.len();
    validate_window_size_lag_count(window_size, lag_count);

    result[0] = 2.0 * autocorr_at_lag_0;
    for i in 1..lag_count {
        let v1 = window[window_size - i];
        let v2 = window[i - 1];
        result[i] = result[i - 1] - v1 * v1 - v2 * v2;
    }
}

/// Computes the length of the FFT needed to compute the autocorrelation
/// for a given window size and lag count to avoid circular convolution effects.
fn autocorr_fft_size(window_size: usize, lag_count: usize) -> usize {
    validate_window_size_lag_count(window_size, lag_count);

    let min_length = window_size + lag_count - 1;
    let mut result: usize = 16; // Start at microfft's minimum size
    while result < min_length {
        result = result << 1;
    }
    result
}

/// Performs an in-place FFT on a given buffer.
fn fft_in_place(buffer: &mut [microfft::Complex32]) {
    let fft_size = buffer.len();
    match fft_size {
        16 => {
            let _ = microfft::complex::cfft_16(buffer);
        }
        32 => {
            let _ = microfft::complex::cfft_32(buffer);
        }
        64 => {
            let _ = microfft::complex::cfft_64(buffer);
        }
        128 => {
            let _ = microfft::complex::cfft_128(buffer);
        }
        256 => {
            let _ = microfft::complex::cfft_256(buffer);
        }
        512 => {
            let _ = microfft::complex::cfft_512(buffer);
        }
        1024 => {
            let _ = microfft::complex::cfft_1024(buffer);
        }
        2048 => {
            let _ = microfft::complex::cfft_2048(buffer);
        }
        4096 => {
            let _ = microfft::complex::cfft_4096(buffer);
        }
        _ => panic!("Unsupported fft size {}", fft_size),
    }
}

fn autocorr_fft(window: &[f32], result: &mut [microfft::Complex32], lag_count: usize) {
    // Sanity checks
    let fft_size = autocorr_fft_size(window.len(), lag_count);
    if result.len() != fft_size {
        panic!(
            "Got autocorr fft buffer of length {}, expected {}.",
            result.len(),
            fft_size
        )
    }

    validate_window_size_lag_count(window.len(), lag_count);

    // TODO: exploit the fact that the signals are real-only.

    // Build FFT input signal
    for (i, sample) in window.iter().enumerate() {
        result[i].re = *sample;
        result[i].im = 0.0;
    }
    for i in window.len()..fft_size {
        result[i].re = 0.0;
        result[i].im = 0.0;
    }

    // Perform the FFT in place
    fft_in_place(&mut result[..]);

    // Compute the power spectral density by point-wise multiplication by the complex conjugate.
    for sample in result.iter_mut() {
        sample.re = sample.re * sample.re + sample.im * sample.im;
        sample.im = 0.0;
    }

    // Perform an inverse FFT to get the autocorrelation. This is done in two steps:
    // 1. Reorder the power spectral density
    result[1..].reverse();
    // 2. Compute the FFT in place, which thanks to the reordering above becomes the inverse FFT (up to a scale)
    fft_in_place(&mut result[..]);

    // Apply scaling factor
    let scale = 1.0 / (fft_size as f32);
    for i in 0..lag_count {
        result[i].re = scale * result[i].re;
        result[i].im = scale * result[i].re;
    }
}

/// The maximum number of key maxima to gather during the peak finding phase.
pub const MAX_KEY_MAXIMA_COUNT: usize = 16;
/// A pitch detection result.
pub struct PitchDetectionResult {
    /// The estimated pitch frequency in Hz.
    pub frequency: f32,
    /// The value of the NSDF at the maximum corresponding to the pitch period.
    /// Between 0 and 1 (inclusive). Values close to 1 indicate pure tones and values
    /// close to 0 indicate lack of a discernable pitch.
    pub clarity: f32,
    /// The value of the NSDF at twice the pitch period. Useful for
    /// determining if the result has a discernable fundamental frequency,
    /// in which case both `clarity` and this value should be close to 1.
    /// This value may be None if the lag corresponding to the double pitch
    /// period exceeds the maximum lag.
    pub clarity_at_double_period: Option<f32>,
    /// The [MIDI note number](https://newt.phys.unsw.edu.au/jw/notes.html) corresponding to the pitch frequency.
    pub note_number: f32,
    /// The estimated pitch period in samples.
    pub pitch_period: f32,
    /// The analyzed window.
    pub window: Vec<f32>, // TODO: should be a slice
    /// The normalized square difference function
    pub nsdf: Vec<f32>,
    /// The number of key maxima found during the peak picking phase. May be 0, in which case
    /// the result is considered invalid.
    pub key_max_count: usize,
    /// A fixed array of key maxima. The first `key_max_count` maxima are valid.
    pub key_maxima: [KeyMaximum; MAX_KEY_MAXIMA_COUNT],
    /// The index into `key_maxima` of the selected key maximum
    pub selected_key_max_index: usize,
    r_prime: Vec<microfft::Complex32>, // TODO: should be a slice
}

impl PitchDetectionResult {
    pub fn new(window_size: usize, lag_count: usize) -> PitchDetectionResult {
        PitchDetectionResult {
            frequency: 0.0,
            clarity: 0.0,
            note_number: 0.0,
            window: vec![0.0; window_size],
            nsdf: vec![0.0; lag_count],
            clarity_at_double_period: None,
            r_prime: vec![microfft::Complex32::new(0.0, 0.0); autocorr_fft_size(window_size, lag_count)],
            key_max_count: 0,
            key_maxima: [KeyMaximum::new(); MAX_KEY_MAXIMA_COUNT],
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

    /// The maximum absolute value of the input window.
    pub fn window_peak(&self) -> f32 {
        let mut max: f32 = 0.0;
        for sample in self.window.iter() {
            let value = sample.abs();
            if value > max {
                max = value
            }
        }
        max
    }

    /// The maximum absolute value of the input window, in dB relative to 1,
    /// i.e 0 dB corresponds to a level of 1.
    pub fn window_peak_db(&self) -> f32 {
        20. * self.window_peak().log10()
    }

    /// The [root mean square](https://en.wikipedia.org/wiki/Root_mean_square) level
    /// of the input window.
    pub fn window_rms(&self) -> f32 {
        let mut rms: f32 = 0.;
        for sample in self.window.iter() {
            rms += sample * sample
        }
        (rms / (self.window.len() as f32)).sqrt()
    }

    /// The [root mean square](https://en.wikipedia.org/wiki/Root_mean_square) level
    /// of the input window, in dB relative to 1, i.e 0 dB corresponds to a level of 1.
    pub fn window_rms_db(&self) -> f32 {
        20. * self.window_rms().log10()
    }

    /// Indicates if the detection result has a valid pitch estimate.
    pub fn is_valid(&self) -> bool {
        self.key_max_count > 0
    }

    fn reset(&mut self) {
        self.clarity_at_double_period = None;
        self.frequency = 0.0;
        self.clarity = 0.0;
        self.note_number = 0.0;
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
                        let last_max_index = if curr > maximum_value { i } else { maximum_index };
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
        for (key_max_index, key_max) in self.key_maxima.iter().take(self.key_max_count).enumerate() {
            if key_max.value >= threshold {
                self.selected_key_max_index = key_max_index;
                break;
            }
        }

        // Compute the clarity, i.e the NSDF value, at twice the
        // pitch period.
        // TODO: use parabolic interpolation here?
        if self.key_max_count > 0 {
            let maximum = self.key_maxima[self.selected_key_max_index];
            let double_period_lag_index = 2 * maximum.lag_index;
            if double_period_lag_index < self.nsdf.len() {
                self.clarity_at_double_period = Some(self.nsdf[double_period_lag_index])
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

            // TODO: If the selected max really corresponds to the fundamental pitch,
            // there should be high NSDF values at integer multiples of the lag of the selected max.
            // If that's not the case, then we're probably dealing with a spurious peak?

            let pitch_period = self.pitch_period / sample_rate;
            self.frequency = 1.0 / pitch_period;
            self.note_number = (self.frequency / 27.5).log10() / (2.0_f32.powf(1.0 / 12.0)).log10();
        }
    }

    /// Computes the normalized square difference function from the current contents of `window`.
    fn compute_nsdf(&mut self) {
        let window = &self.window[..];
        let nsdf = &mut self.nsdf[..];
        let mut r_prime = &mut self.r_prime[..];

        autocorr_fft(&self.window[..], &mut r_prime, nsdf.len());

        // Compute m' and store it in the nsdf buffer
        let autocorr_at_lag_0 = r_prime[0].re;
        m_prime_incremental(window, autocorr_at_lag_0, nsdf);

        // Compute the NSDF as 2 * r' / m'
        for i in 0..nsdf.len() {
            let denominator = nsdf[i];
            nsdf[i] = if denominator.abs() <= f32::EPSILON { 0.0 } else { 2.0 * r_prime[i].re / denominator };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_seeder;
    use rand_pcg::Pcg64;

    /// Computes the autocorrelation as a naive inefficient summation.
    /// Only used for testing purposes.
    fn autocorr_sum(window: &[f32], result: &mut [f32]) {
        let window_size = window.len();
        if window_size < result.len() {
            panic!("Result vector must not be longer than the window.");
        }

        let lag_count = result.len();

        for tau in 0..lag_count {
            let mut sum: f32 = 0.0;
            let j_min: usize = 0;
            let j_max = window_size - 1 - tau + 1;
            for j in j_min..j_max {
                let xj = window[j];
                let xj_plus_tau = window[j + tau];
                sum += xj * xj_plus_tau;
            }
            result[tau] = sum;
        }
    }

    // Computes m', defined in eq (6), as a naive inefficient summation.
    // Only used for testing purposes.
    fn m_prime_sum(window: &[f32], result: &mut [f32]) {
        let window_size = window.len();
        if window_size < result.len() {
            panic!("Result vector must not be longer than the window.");
        }

        let lag_count = result.len();

        for tau in 0..lag_count {
            let mut sum: f32 = 0.0;
            let j_min: usize = 0;
            let j_max = window_size - 1 - tau + 1;
            for j in j_min..j_max {
                let xj = window[j];
                let xj_plus_tau = window[j + tau];
                sum += xj * xj + xj_plus_tau * xj_plus_tau;
            }
            result[tau] = sum;
        }
    }

    #[test]
    fn test_silence() {
        let sample_rate = 44100.0;
        let window_size = 1024;

        let mut result = PitchDetectionResult::new(window_size, window_size / 2);
        result.compute(sample_rate);
        assert_eq!(result.nsdf[0], 0.);
        assert_eq!(result.key_max_count, 0);
    }

    #[test]
    fn test_low_sine() {
        // Tests the case when the expected pitch period > 0.5 * lag_count and < lag_count
        for f in [154.0_f32, 190.0_f32].iter() {
            let window_size = 1024;
            let lag_count = window_size / 2;
            let sample_rate: f32 = 44100.0;
            let expected_pitch_period = sample_rate / f;

            // Verify pre-condition
            assert!(expected_pitch_period < (lag_count as f32));

            // Generate a pure tone and perform pitch detection
            let mut result = PitchDetectionResult::new(window_size, lag_count);
            for i in 0..window_size {
                let sine_value = (2.0 * std::f32::consts::PI * f * (i as f32) / sample_rate).sin();
                result.window[i] = sine_value;
            }

            result.compute(sample_rate);

            assert!((f - result.frequency).abs() <= 0.001, "Wrong detected frequency");
            // We should have one actual maximum and one maximum at the last NSDF sample
            assert_eq!(result.key_max_count, 2, "Unexpected key max count");

            // The value of the last key max should be reasonable
            let last_max = result.key_maxima[result.key_max_count - 1];
            let last_max_lag = last_max.lag;
            let last_max_lag_index = last_max.lag_index;
            let last_max_value = last_max.value;
            let last_max_value_at_lag_index = last_max.value;
            assert!((last_max_lag - (last_max_lag_index as f32)).abs() < 1., "Unreasonable interpolated key max lag");
            assert!((last_max_value - last_max_value_at_lag_index).abs() < 0.001, "Unreasonable interpolated key max value");
        }
    }

    /*#[test]
    fn test_noise() {
        // TODO: fix this
        let rng: Pcg64 = rand_seeder::Seeder::from("hello seed").make_rng();

        let sample_rate = 44100.0;
        let window_size = 1024;
        let mut result = PitchDetectionResult::new(window_size, window_size / 2);
        for i in 0..result.window.len() {
            // result.window[i] = rng.ne
            // rng.next_u32();
        }
        result.compute(sample_rate);
        let a = 0;

        // TODO: ADD ASSERTS
    }*/

    #[test]
    fn test_incremental_m_prime() {
        let signal: Vec<f32> = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let lag_count: usize = 4;

        // Compute m' by naive summation
        let mut m_prime_naive: Vec<f32> = vec![0.0; lag_count];
        m_prime_sum(&signal[..], &mut m_prime_naive[..]);

        // Compute m' by incremental subtraction
        let mut autocorr: Vec<f32> = vec![0.0; lag_count];
        autocorr_sum(&signal[..], &mut autocorr[..]);
        let mut m_prime_incr: Vec<f32> = vec![0.0; lag_count];
        m_prime_incremental(&signal[..], autocorr[0], &mut m_prime_incr[..]);

        // Make sure the results are the same
        for (naive, incr) in m_prime_naive.iter().zip(m_prime_incr.iter()) {
            assert!((*naive - *incr).abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn test_autocorr_fft() {
        // Reference Octave output
        // a = [1   2   3   4   5   6   7   8]
        // conv(a, fliplr(a)) = [8    23    44    70   100   133   168   204   168   133   100    70    44   23     8]
        // ifft(abs(fft([a 0 0 0 0])).^2) = [204.000   168.000   133.000   100.000    70.000    52.000 ....

        let window: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let lag_count = 4;
        let mut autocorr_reference: Vec<f32> = vec![0.0; lag_count];
        autocorr_sum(&window[..], &mut autocorr_reference[..]);

        let fft_size = autocorr_fft_size(window.len(), lag_count);
        let mut fft_buffer: Vec<microfft::Complex32> =
            vec![microfft::Complex32::new(0.0, 0.0); fft_size];
        autocorr_fft(&window[..], &mut fft_buffer[..], lag_count);

        let epsilon = 1e-4;
        for (reference, fft_value) in autocorr_reference.iter().zip(fft_buffer.iter()) {
            assert!((*reference - fft_value.re).abs() <= epsilon);
        }
    }
}
