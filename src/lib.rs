use microfft;
use std::vec;

// https://en.wikipedia.org/wiki/Infinite_impulse_response
struct IIRFilter {
    a_coeffs: Vec<f32>,
    b_coeffs: Vec<f32>,
    inputs: Vec<f32>,
    inputs_pos: usize,
    outputs: Vec<f32>,
    outputs_pos: usize,
}

impl IIRFilter {
    fn new(a_coeffs: Vec<f32>, b_coeffs: Vec<f32>) -> IIRFilter {
        let inputs_count = b_coeffs.len();
        let outputs_count = a_coeffs.len();
        IIRFilter {
            a_coeffs,
            b_coeffs,
            inputs: vec![0.0; inputs_count],
            inputs_pos: 0,
            outputs: vec![0.0; outputs_count],
            outputs_pos: 0,
        }
    }

    fn process(&mut self, input_samples: &[f32], output_samples: &mut [f32]) {
        if input_samples.len() != output_samples.len() {
            panic!("IIR filter input and output buffers must have the same size");
        }
        for (index, input) in input_samples.iter().enumerate() {
            output_samples[index] = input_samples[index];
        }
    }
}

struct EqualLoudnessFilter {
    butterworth: IIRFilter,
    yule_walk: IIRFilter,
}

impl EqualLoudnessFilter {
    fn new(sample_rate: f32) -> EqualLoudnessFilter {
        if sample_rate as usize != 44100 {
            panic!("Only a sample rate of 44100 Hz is supported")
        }
        EqualLoudnessFilter {
            butterworth: IIRFilter::new(
                vec![1.00000000000000, -1.96977855582618, 0.97022847566350],
                vec![0.98500175787242, -1.97000351574484, 0.98500175787242],
            ),
            yule_walk: IIRFilter::new(
                vec![
                    1.00000000000000,
                    -3.47845948550071,
                    6.36317777566148,
                    -8.54751527471874,
                    9.47693607801280,
                    -8.81498681370155,
                    6.85401540936998,
                    -4.39470996079559,
                    2.19611684890774,
                    -0.75104302451432,
                    0.13149317958808,
                ],
                vec![
                    0.05418656406430,
                    -0.02911007808948,
                    -0.00848709379851,
                    -0.00851165645469,
                    -0.00834990904936,
                    0.02245293253339,
                    -0.02596338512915,
                    0.01624864962975,
                    -0.00240879051584,
                    0.00674613682247,
                    -0.00187763777362,
                ],
            ),
        }
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.yule_walk.process(input, output);
        self.butterworth.process(input, output);
    }
}

const MAX_KEY_MAXIMA_COUNT: usize = 16;
#[derive(Copy, Clone)]
pub struct KeyMaximum {
    lag_index: usize,
    value_at_lag_index: f32,
    value: f32,
    lag: f32,
}

impl KeyMaximum {
    fn new() -> KeyMaximum {
        KeyMaximum {
            lag_index: 0,
            value_at_lag_index: 0.0,
            value: 0.0,
            lag: 0.0,
        }
    }

    fn set(&mut self, nsdf: &[f32], lag_index: usize) {
        self.lag_index = lag_index;
        let value_at_lag_index = nsdf[lag_index];
        self.value_at_lag_index = value_at_lag_index;

        // Use parabolic interpolation to approximate
        // the true maximum using the left and right neighbors
        let left_index = std::cmp::max(0, lag_index - 1);
        let right_index = std::cmp::min(nsdf.len() - 1, lag_index + 1);
        let left = nsdf[left_index];
        let right = nsdf[right_index];

        // Compute coefficients of a parabola ax^2 + bx + c passing through
        // (-1, left), (0, max), (1, right)
        let a = 0.5 * (right - 2.0 * value_at_lag_index + left);
        let b = 0.5 * (right - left);
        let c = value_at_lag_index;
        // Find the x value where the derivative is zero, i.e where the parabola has its maximum
        let x_max = if a != 0.0 { -b / (2.0 * a) } else { 0.0 };
        let value = a * x_max * x_max + b * x_max + c;
        let lag = (lag_index as f32) + x_max;

        self.value = value;
        self.lag = lag;
    }
}

pub struct MPMPitch {
    pub window: Vec<f32>, // TODO: should be a slice
    pub frequency: f32,
    pub note_number: f32,
    pub clarity: f32,
    pub nsdf: Vec<f32>,
    // The number of key maxima
    pub key_max_count: usize,
    // List of current key maxima. Allocated once.
    pub key_maxima: [KeyMaximum; MAX_KEY_MAXIMA_COUNT],
    // The index of the selected key maximum
    pub selected_key_max_index: usize,
    // The final pitch period (in samples)
    pub pitch_period: f32,
    r_prime: Vec<microfft::Complex32>, // TODO: should be a slice
}

impl MPMPitch {
    // Computes pitch based on the current contents of window
    fn compute(&mut self, sample_rate: f32) {
        let window = &self.window[..];
        let nsdf = &mut self.nsdf[..];
        let mut r_prime = &mut self.r_prime[..];

        autocorr_fft(&self.window[..], &mut r_prime, nsdf.len());

        // Compute m' and store it in the nsdf buffer
        let autocorr_at_lag_0 = r_prime[0].re;
        m_prime_incremental(window, autocorr_at_lag_0, nsdf);

        // Compute the NSDF as 2 * r' / m'
        for i in 0..nsdf.len() {
            nsdf[i] = 2.0 * r_prime[i].re / nsdf[i];
        }

        // Perform peak picking. First, gather key maxima
        self.key_max_count = 0;
        let mut is_detecting = false;
        let mut maximum_value: f32 = 0.0;
        let mut maximum_index: usize = 0;
        let mut prev = nsdf[0];
        for i in 1..nsdf.len() {
            let curr = nsdf[i];
            if prev <= 0.0 && curr > 0.0 {
                // positive zero crossing, going from - to +.
                // start looking for a key maximum
                is_detecting = true;
                maximum_value = curr;
                maximum_index = i;
            } else if prev >= 0.0 && curr < 0.0 || i == nsdf.len() - 1 {
                // We reached either a negative zero crossing (going from + to -) or
                // the end of the nsdf. Stop looking for a key maximum and store the one we've got
                // (unless we have collected the maximum number of key maxima)
                if is_detecting && self.key_max_count < self.key_maxima.len() {
                    self.key_maxima[self.key_max_count].set(&nsdf, maximum_index);
                    self.key_max_count += 1
                }
                is_detecting = false;
            }

            if is_detecting && curr > maximum_value {
                // If we're looking for a key maximum and the current
                // value is greater than the current max, set a new max.
                maximum_value = curr;
                maximum_index = i;
            }

            prev = curr;
        }

        // Then select the final maximum
        let mut largest_key_maximum: f32 = 0.0;
        for (i, key_max) in self.key_maxima.iter().enumerate() {
            let value = key_max.value;
            if value > largest_key_maximum || i == 0 {
                largest_key_maximum = value;
            }
        }

        let k: f32 = 0.8;
        self.pitch_period = 0.0;
        self.clarity = 0.0;
        self.frequency = 0.0;
        self.selected_key_max_index = 0;

        let threshold = k * largest_key_maximum;
        for (key_max_index, key_max) in self.key_maxima.iter().enumerate() {
            if key_max.value >= threshold {
                self.selected_key_max_index = key_max_index;
                self.pitch_period = key_max.lag;
                self.clarity = if key_max.value > 1.0 {
                    1.0
                } else {
                    key_max.value
                };
                let pitch_period = self.pitch_period / sample_rate;
                self.frequency = 1.0 / pitch_period;
                break;
            }
        }
    }
}

/// Computes m' defined in eq (6), using the incremental subtraction
/// algorithm described in section 6 - Efficient calculation of SDF.
fn m_prime_incremental(window: &[f32], autocorr_at_lag_0: f32, result: &mut [f32]) {
    let lag_count = result.len();
    let window_size = window.len();
    if lag_count > window_size {
        panic!("Lag count must not be greater than the window size");
    }

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
    let min_length = window_size + lag_count - 1;
    let mut result: usize = 16; // Start at microfft's minimum size
    while result < min_length {
        result = result << 1;
    }
    result
}

fn fft_in_place(buffer: &mut [microfft::Complex32]) {
    let fft_size = buffer.len();
    match fft_size {
        16 => {
            let _ = microfft::complex::cfft_16(buffer);
        },
        32 => {
            let _ = microfft::complex::cfft_32(buffer);
        },
        64 => {
            let _ = microfft::complex::cfft_64(buffer);
        },
        128 => {
            let _ = microfft::complex::cfft_128(buffer);
        },
        256 => {
            let _ = microfft::complex::cfft_256(buffer);
        },
        512 => {
            let _ = microfft::complex::cfft_512(buffer);
        },
        1024 => {
            let _ = microfft::complex::cfft_1024(buffer);
        },
        2048 => {
            let _ = microfft::complex::cfft_2048(buffer);
        },
        4096 => {
            let _ = microfft::complex::cfft_4096(buffer);
        }
        _ => panic!("Unsupported fft size {}", fft_size)
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

    if window.len() < lag_count {
        panic!("Window size must not be less than the lag count")
    }

    // Build FFT input signal
    for (i, sample) in window.iter().enumerate() {
        result[i].re = *sample;
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

pub struct MPMPitchDetector {
    sample_rate: usize,
    window_size: usize,
    window_distance: usize,
    window_write_index: usize,
    window: Vec<f32>,          // TODO: should be a slice
    filtered_window: Vec<f32>, // TODO: should be a slice
    has_full_window: bool,
    equal_loudness_filter: EqualLoudnessFilter,
    pub result: MPMPitch,
}

impl MPMPitchDetector {
    pub fn new(
        sample_rate: usize,
        window_size: usize,
        window_distance: usize,
        use_equal_loudness_filter: bool,
    ) -> MPMPitchDetector {
        let lag_count = window_size;
        let fft_size = autocorr_fft_size(window_size, lag_count);
        MPMPitchDetector {
            sample_rate,
            window_size,
            window_distance,
            window_write_index: 0,
            window: vec![0.0; window_size],
            filtered_window: vec![0.0; window_size],
            has_full_window: false,
            equal_loudness_filter: EqualLoudnessFilter::new(sample_rate as f32),
            result: MPMPitch {
                frequency: 0.0,
                clarity: 0.0,
                note_number: 0.0,
                window: vec![0.0; window_size],
                nsdf: vec![0.0; lag_count],
                r_prime: vec![microfft::Complex32::new(0.0, 0.0); fft_size],
                key_max_count: 0,
                key_maxima: [KeyMaximum::new(); MAX_KEY_MAXIMA_COUNT],
                selected_key_max_index: 0,
                pitch_period: 0.0,
            },
        }
    }

    pub fn process<F: FnMut(usize, &MPMPitch)>(&mut self, samples: &[f32], mut callback: F) {
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

    // Computes the autocorrelation as a naive inefficient summation.
    // Only used for testing purposes.
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

    // Computes m' as a naive inefficient summation.
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
    fn test_440_hz_sine() {
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
            MPMPitchDetector::new(sample_rate as usize, window_size, window_distance, true);
        detector.process(&window[..], |sample_index, result: &MPMPitch| {});

        assert!((f - detector.result.frequency).abs() <= 0.001);
    }

    #[test]
    fn key_maximum_interpolation() {
        {
            let nsdf: [f32; 4] = [0.0, 0.0, 3.0, 0.0];
            let mut key_max = KeyMaximum::new();
            key_max.set(&nsdf, 2);
            assert!((key_max.lag - 2.0).abs() <= f32::EPSILON);
            assert!((key_max.value - 3.0).abs() <= f32::EPSILON);
        }

        {
            let nsdf: [f32; 3] = [-2.0, 0.0, -1.0];
            let mut key_max = KeyMaximum::new();
            key_max.set(&nsdf, 1);
            assert!((key_max.lag - 1.1666666_f32).abs() <= f32::EPSILON);
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
        let mut detector = MPMPitchDetector::new(44100, window_size, window_distance, true);

        // Verify that the buffer to process in callback i starts with the value i
        let mut result_count = 0;
        detector.process(&buffer[..], |sample_index: usize, result: &MPMPitch| {
            let first_window_sample = result.window[0];
            assert_eq!(first_window_sample as usize, result_count % window_size);
            result_count += 1;
        });
    }
}
