
use core::f32::consts::PI;

#[derive(Clone, Copy)]
/// [Window function](https://en.wikipedia.org/wiki/Window_function) type.
pub enum WindowFunctionType {
    /// [Hann window](https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
    Hann,
    /// [Welch window](<https://en.wikipedia.org/wiki/Window_function#Welch_window>)
    Welch,
}

/// Performs point-wise multiplication of a buffer and a window function of a given type.
pub fn apply_window_function(window_function: WindowFunctionType, buffer: &mut [f32]) {
    match window_function {
        WindowFunctionType::Hann => hann_window(buffer),
        WindowFunctionType::Welch => welch_window(buffer),
    }
}

/// Performs point-wise multiplication of a buffer and the Hann window function.
fn hann_window(buffer: &mut [f32]) {
    // sin(0.5 * pi * x) can be approximated with a
    // max error below 0.0003 and exactly matching endpoints on [-1, 1] as
    // ax^5 + bx^3 + cx,
    // where
    // a = pi / 2 - 1.5
    // b = 2.5 - pi
    // c = pi / 2

    let a = 0.5 * (PI / 2. - 1.5);
    let b = 0.5 * (2.5 - PI);
    let c = 0.5 * PI / 2.;
    let d = 0.5;

    let len = buffer.len();
    let dx = 4. / ((len - 1) as f32);
    let len_is_even = len % 2 == 0;

    // Evaluate window in two halves starting with the left
    let left_half_end_len = if len_is_even { len / 2 } else { len / 2 + 1 };
    let mut x = -1.0;
    for value in buffer.iter_mut().take(left_half_end_len) {
        let x3 = x * x * x;
        let x5 = x3 * x * x;
        let window_value = a * x5 + b * x3 + c * x + d;
        *value *= window_value;
        x += dx; // TODO: this causes drift for large windows?
    }

    // Right half
    x = if len_is_even {
        1.0 - 0.5 * dx
    } else {
        1.0 - dx
    };
    for value in buffer.iter_mut().skip(left_half_end_len) {
        let x3 = x * x * x;
        let x5 = x3 * x * x;
        let window_value = a * x5 + b * x3 + c * x + d;
        *value *= window_value;
        x -= dx; // TODO: this causes drift for large windows?
    }
}

/// Performs point-wise multiplication of a buffer and the Welch window function.
fn welch_window(buffer: &mut [f32]) {
    if buffer.len() < 2 {
        for value in buffer.iter_mut() {
            *value = 0.0;
        }
        return
    }
    let len = buffer.len();
    let dx = 2. / ((len - 1) as f32);
    for (i, value) in buffer.iter_mut().enumerate() {
        let x = -1.0 + dx * (i as f32);
        let window_value = 1. - x * x;
        *value *= window_value;
    }
}

#[cfg(test)]
mod tests {
    use crate::common::window_function::{hann_window, welch_window};
    use alloc::vec;
    use core::f32::consts::PI;

    #[test]
    fn test_hann_window() {
        let hann_exact = |n: usize, size: usize| -> f32 {
            if n == 0 {
                return 0.0;
            }
            let sin = (PI * (n as f32) / ((size - 1) as f32)).sin();
            sin * sin
        };
        let eps = 0.0003;
        for window_size in [1, 2, 128, 4096, 100000] {
            let mut window = vec![1.0; window_size];
            hann_window(&mut window);
            for (i, value_approx) in window.iter().enumerate() {
                let exact_value = hann_exact(i, window.len());
                let error = (exact_value - value_approx).abs();
                assert!(error < eps);
            }
        }
    }

    #[test]
    fn test_welch_window() {
        let welch_exact = |n: usize, size: usize| -> f32 {
            if size < 2 {
                return 0.0;
            }
            let x = -1.0 + 2.0 * (n as f32) / ((size - 1) as f32);
            1.0 - x * x
        };

        let eps = 1e-6;
        for window_size in [1, 2, 128, 4096, 100000] {
            let mut window = vec![1.0; window_size];
            welch_window(&mut window);
            for (i, value_approx) in window.iter().enumerate() {
                let exact_value = welch_exact(i, window.len());
                let error = (exact_value - value_approx).abs();
                assert!(error < eps);
            }
        }
    }
}
