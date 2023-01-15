use super::fft::real_fft;

/// Computes the length of the FFT needed to compute the autocorrelation
/// for a given window size and lag count to avoid circular convolution effects.
///
/// # Arguments
///
/// * `buffer_size` - The size of the input buffer.
/// * `lag_count` - The length of the computed autocorrelation.
pub fn autocorr_fft_size(buffer_size: usize, lag_count: usize) -> usize {
    assert!(lag_count <= buffer_size);
    let min_length = buffer_size + lag_count - 1;
    let mut result: usize = 8; // Start at microfft's minimum size
    while result < min_length {
        result = result << 1;
    }
    result
}

/// Computes the [autocorrelation](https://en.wikipedia.org/wiki/Autocorrelation)
/// of a given buffer using FFT.
///
/// # Arguments
///
/// * `buffer` - Input buffer
/// * `result` - A buffer to write the result to.
/// * `scratch_buffer` - A scratch buffer used for temporary storage.
/// * `lag_count` - The length of the computed autocorrelation.
pub fn autocorr_fft(
    buffer: &[f32],
    result: &mut [f32],
    scratch_buffer: &mut [f32],
    lag_count: usize,
) {
    // Sanity checks
    let fft_size = autocorr_fft_size(buffer.len(), lag_count);
    if result.len() != fft_size {
        panic!(
            "Got autocorr fft buffer of length {}, expected {}.",
            result.len(),
            fft_size
        )
    }
    if scratch_buffer.len() < result.len() {
        panic!("Autocorr fft scatch buffer must not be shorter than result buffer")
    }

    // Build FFT input signal
    result[..buffer.len()].copy_from_slice(&buffer[..]);
    for element in result.iter_mut().skip(buffer.len()) {
        *element = 0.0
    }

    // Perform the FFT in place
    let fft = real_fft(&mut result[..]);

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
    let ifft = real_fft(&mut scratch_buffer[..]);

    // Apply scaling factor
    let scale = 1.0 / (fft_size as f32);
    for (result, ifft) in result.iter_mut().zip(ifft) {
        *result = scale * (*ifft).re;
    }
}

/// Computes the [autocorrelation](https://en.wikipedia.org/wiki/Autocorrelation)
/// of a given buffer using time domain convolution.
pub fn autocorr_conv(window: &[f32], result: &mut [f32]) {
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

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::{autocorr_conv, autocorr_fft, autocorr_fft_size};

    #[test]
    fn test_autocorr_fft() {
        // Reference Octave output (https://www.gnu.org/software/octave/index)
        // a = [1   2   3   4   5   6   7   8]
        // conv(a, fliplr(a)) = [8    23    44    70   100   133   168   204   168   133   100    70    44   23     8]
        // ifft(abs(fft([a 0 0 0 0])).^2) = [204.000   168.000   133.000   100.000    70.000    52.000 ....

        let window: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let lag_count = 4;
        let mut autocorr_reference: Vec<f32> = vec![0.0; lag_count];
        autocorr_conv(&window[..], &mut autocorr_reference[..]);

        let fft_size = autocorr_fft_size(window.len(), lag_count);
        let mut fft_buffer: Vec<f32> = vec![0.0; fft_size];
        let mut scratch_buffer: Vec<f32> = vec![0.0; fft_size];
        autocorr_fft(
            &window[..],
            &mut fft_buffer[..],
            &mut scratch_buffer[..],
            lag_count,
        );

        let epsilon = 1e-4;
        for (reference, fft_value) in autocorr_reference.iter().zip(fft_buffer.iter()) {
            assert!((*reference - fft_value).abs() <= epsilon);
        }
    }
}
