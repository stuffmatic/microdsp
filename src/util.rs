pub fn validate_window_size_lag_count(window_size: usize, lag_count: usize) {
    if lag_count > window_size {
        panic!("Lag count must not be greater than the window size");
    }
}

/// Converts a frequency to a MIDI note number (with a fractional part)
pub fn freq_to_midi_note(f: f32) -> f32 {
    // https://www.inspiredacoustics.com/en/MIDI_note_numbers_and_center_frequencies
    // MIDI note 21 is A0 at 27.5 Hz
    21.0_f32 + (f / 27.5).log10() / (2.0_f32.powf(1.0 / 12.0)).log10()
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
    let mut result: usize = 16; // Start at microfft's minimum size
    while result < min_length {
        result = result << 1;
    }
    result
}

/// Performs an in-place FFT on a given buffer.
pub fn fft_in_place(buffer: &mut [microfft::Complex32]) {
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

pub fn autocorr_fft(window: &[f32], result: &mut [microfft::Complex32], lag_count: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_midi_note_conversion() {
        // https://www.inspiredacoustics.com/en/MIDI_note_numbers_and_center_frequencies

        let note_number_a0 = 21.0_f32;
        let f_a0 = 27.5_f32;
        let computed_note_number_a0 = freq_to_midi_note(f_a0);
        assert!((computed_note_number_a0 - note_number_a0).abs() <= f32::EPSILON);

        let note_number_a4 = 69.0_f32;
        let f_a4 = 440.0_f32;
        let computed_note_number_a4 = freq_to_midi_note(f_a4);
        assert!((computed_note_number_a4 - note_number_a4).abs() <= 0.0001);
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