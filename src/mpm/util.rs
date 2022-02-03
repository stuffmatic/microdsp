use core::convert::TryInto;

pub fn validate_window_size_lag_count(window_size: usize, lag_count: usize) {
    if lag_count > window_size {
        panic!("Lag count must not be greater than the window size");
    }
}

/// Converts a frequency to a MIDI note number (with a fractional part)
pub fn freq_to_midi_note(f: f32) -> f32 {
    // note(f) = 12 * Math.log2(f / 440.0) + 69
    //         = 12 * Math.log2(f) + 12 * Math.log2(1/440) + 69
    12.0 * F32Ext::log2(f) - 36.376316562295926
}

/// Computes m' defined in eq (6), using the incremental subtraction
/// algorithm described in section 6 - Efficient calculation of SDF.
pub fn m_prime_incremental(window: &[f32], autocorr_at_lag_0: f32, result: &mut [f32]) {
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
pub fn autocorr_fft_size(window_size: usize, lag_count: usize) -> usize {
    validate_window_size_lag_count(window_size, lag_count);

    let min_length = window_size + lag_count - 1;
    let mut result: usize = 8; // Start at microfft's minimum size
    while result < min_length {
        result = result << 1;
    }
    result
}

pub fn autocorr_fft(
    window: &[f32],
    result: &mut [f32],
    scratch_buffer: &mut [f32],
    lag_count: usize
) {
    // Sanity checks
    let fft_size = autocorr_fft_size(window.len(), lag_count);
    if result.len() != fft_size {
        panic!(
            "Got autocorr fft buffer of length {}, expected {}.",
            result.len(),
            fft_size
        )
    }
    if scratch_buffer.len() < result.len()  {
        panic!("Autocorr fft scatch buffer must not be shorter than result buffer")
    }

    validate_window_size_lag_count(window.len(), lag_count);

    // Build FFT input signal
    result[..window.len()].copy_from_slice(&window[..]);
    for element in result.iter_mut().skip(window.len()) {
        *element = 0.0
    }

    // Perform the FFT in place
    let fft = real_fft_in_place(&mut result[..]);

    // Compute the power spectral density by point-wise multiplication by the complex conjugate.
    scratch_buffer[0] = fft[0].re * fft[0].re;
    let scratch_buffer_length = scratch_buffer.len();
    for (index, fft_value) in fft.iter_mut().skip(1).enumerate() {
        let norm_sq = fft_value.norm_sqr();
        scratch_buffer[index + 1] = norm_sq;
        scratch_buffer[scratch_buffer_length - index - 1] = norm_sq;
    }
    scratch_buffer[fft.len()] = fft[0].im * fft[0].im;

    // 2. Compute the inverse FFT in place to get the autocorrelation (up to a scaling factor)
    let ifft = real_fft_in_place(&mut scratch_buffer[..]);

    // Apply scaling factor
    let scale = 1.0 / (fft_size as f32);
    for i in 0..lag_count {
        result[i] = scale * ifft[i].re;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::vec::Vec;
    use crate::alloc::vec;

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
    fn text_approximate_note_number() {
        // The hz to midi note conversion relies on the approximate log2
        // function of the micromath crate. This test compares this
        // approximation to std's log2 and makes sure the difference
        // is acceptable.

        // The maximum acceptable error in cents. 0.1 is 1/1000th of a semitone.
        let max_cent_error = 0.11_f32;
        for i in 1..10000 {
            let f = i as f32;
            let actual_note_number = 12.0 * (f / 440.0).log2() + 69.0;
            let approx_note_number = freq_to_midi_note(f);
            let delta_cents = 100. * (actual_note_number - approx_note_number);
            if delta_cents.abs() > max_cent_error {
                assert!(false);
            }
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
    fn test_autocorr_fft() {
        // Reference Octave output (https://www.gnu.org/software/octave/index)
        // a = [1   2   3   4   5   6   7   8]
        // conv(a, fliplr(a)) = [8    23    44    70   100   133   168   204   168   133   100    70    44   23     8]
        // ifft(abs(fft([a 0 0 0 0])).^2) = [204.000   168.000   133.000   100.000    70.000    52.000 ....

        let window: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let lag_count = 4;
        let mut autocorr_reference: Vec<f32> = vec![0.0; lag_count];
        autocorr_sum(&window[..], &mut autocorr_reference[..]);

        let fft_size = autocorr_fft_size(window.len(), lag_count);
        let mut fft_buffer: Vec<f32> =
            vec![0.0; fft_size];
        let mut scratch_buffer: Vec<f32> =
            vec![0.0; fft_size];
        autocorr_fft(&window[..], &mut fft_buffer[..], &mut scratch_buffer[..], lag_count);

        let epsilon = 1e-4;
        for (reference, fft_value) in autocorr_reference.iter().zip(fft_buffer.iter()) {
            assert!((*reference - fft_value).abs() <= epsilon);
        }
    }
}
