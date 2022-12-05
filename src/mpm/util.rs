use micromath::F32Ext;

pub fn validate_window_size_lag_count(window_size: usize, lag_count: usize) {
    if lag_count > window_size {
        panic!("Lag count must not be greater than the window size");
    }
}

/// Converts a frequency to a MIDI note number (with a fractional part)
pub fn freq_to_midi_note(f: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::vec;
    use crate::alloc::vec::Vec;
    use crate::common::autocorr::autocorr_sum;

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
}
