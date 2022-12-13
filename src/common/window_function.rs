//! [Window functions](https://en.wikipedia.org/wiki/Window_function).

use core::f32::consts::PI;

#[derive(Clone, Copy)]
pub enum WindowFunction {
    /// <https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows>
    Hann,
    /// <https://en.wikipedia.org/wiki/Window_function#Welch_window>
    Welch,
}

/// Performs point-wise multiplication of a buffer and a window function of a given type.
pub fn apply_window_function(window_function: WindowFunction, buffer: &mut [f32]) {
    match window_function {
        WindowFunction::Hann => hann_window(buffer),
        WindowFunction::Welch => welch_window(buffer),
    }
}

/// Performs point-wise multiplication of a buffer and the Hann window function.
fn hann_window(buffer: &mut [f32]) {
    // sin(0.5 * pi * x) can be approximated with a
    // max error below 0.001 and exactly matching endpoints on [-1, 1] as
    // ax^5 + bx^3 + cx,
    // where
    // a = pi / 2 - 1.5
    // b = 2.5 - pi
    // c = pi / 2

    let a = 0.5 * (PI / 2. - 1.5);
    let b = 0.5 * (2.5 - PI);
    let c = 0.5 * PI / 2.;
    let d = 0.5;

    // Evaluate window in two halves
    let len = buffer.len();
    let len_is_even = len % 2 == 0;
    let left_half_end_len = if len_is_even { len / 2 } else { len / 2 + 1 };
    let dx = 4. / ((len - 1) as f32);
    let mut x = -1.0;
    for value in buffer.iter_mut().take(left_half_end_len) {
        let x3 = x * x * x;
        let x5 = x3 * x * x;
        let window_value = a * x5 + b * x3 + c * x + d;
        *value *= window_value;
        x += dx;
    }

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
        x -= dx;
    }
}

/// Performs point-wise multiplication of a buffer and the Welch window function.
fn welch_window(buffer: &mut [f32]) {
    let len = buffer.len();
    let dx = 2. / ((len - 1) as f32);
    let mut x = -1.0;
    for value in buffer.iter_mut() {
        let window_value = 1. - x * x;
        *value *= window_value;
        x += dx;
    }
}

#[cfg(test)]
mod tests {
    use crate::common::window_function::{hann_window, welch_window};

    #[test]
    fn test_hann_window() {
        {
            let mut buffer = [1.0_f32; 11];
            hann_window(&mut buffer);
            assert_eq!(buffer[0], 0.);
            assert_eq!(buffer[5], 1.);
            assert_eq!(buffer[10], 0.);
        }

        {
            let mut buffer = [1.0_f32; 5];
            hann_window(&mut buffer);
            assert_eq!(buffer[0], 0.0);
            assert_eq!(buffer[1], 0.5);
            assert_eq!(buffer[2], 1.0);
            assert_eq!(buffer[3], 0.5);
            assert_eq!(buffer[4], 0.0);
        }

        {
            let mut buffer = [1.0_f32; 6];
            hann_window(&mut buffer);
            assert_eq!(buffer[0], 0.0);
            assert_eq!(buffer[1], 0.345475435);
            assert_eq!(buffer[2], 0.904699444);
            assert_eq!(buffer[3], 0.904699444);
            assert_eq!(buffer[4], 0.345475435);
            assert_eq!(buffer[5], 0.0);
        }
    }

    #[test]
    fn test_welch_window() {
        let mut buffer: [f32; 1025] = [1.; 1025];
        welch_window(&mut buffer);
        assert_eq!(buffer[0], 0.);
        assert_eq!(buffer[512], 1.);
        assert_eq!(buffer[1024], 0.);
    }
}
