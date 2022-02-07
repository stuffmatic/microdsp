use core::f32::consts::PI;

#[derive(Clone, Copy)]
pub enum WindowFunction {
    /// <https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows>
    Hann,
    /// <https://en.wikipedia.org/wiki/Window_function#Welch_window>
    Welch,
}

fn apply_hann_window(buffer: &mut [f32]) {
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

    // Evaluate window in two halves with opposite y scales.
    let len = buffer.len();
    let half_len = len / 2;
    let dx = 2. / ((half_len - 1) as f32);
    let mut x = -1.0;
    for value in buffer.iter_mut().take(half_len) {
        let x3 = x * x * x;
        let x5 = x3 * x * x;
        let window_value = a * x5 + b * x3 + c * x + d;
        *value *= window_value;
        x += dx;
    }

    x -= 2.0;
    for value in buffer.iter_mut().skip(half_len) {
        let x3 = x * x * x;
        let x5 = x3 * x * x;
        let window_value = -a * x5 - b * x3 - c * x + d;
        *value *= window_value;
        x += dx;
    }
}

fn apply_welch_window(buffer: &mut [f32]) {
    let len = buffer.len();
    let dx = 2. / ((len - 1) as f32);
    let mut x = -1.0;
    for value in buffer.iter_mut() {
        let window_value = 1. - x * x;
        *value *= window_value;
        x += dx;
    }
}

pub fn apply_window_function(window_function: WindowFunction, buffer: &mut [f32]) {
    match window_function {
        WindowFunction::Hann => apply_hann_window(buffer),
        WindowFunction::Welch => apply_welch_window(buffer),
    }
}

#[cfg(test)]
mod tests {
    use crate::common::window_function::{apply_welch_window, apply_hann_window};

    #[test]
    fn test_welch_window() {
        let mut buffer: [f32; 1025] = [1.; 1025];
        apply_welch_window(&mut buffer);
        assert_eq!(buffer[0], 0.);
        assert_eq!(buffer[1024], 0.);
        assert_eq!(buffer[512], 1.);
    }

    #[test]
    fn test_hann_window() {
        let mut buffer: [f32; 1025] = [1.; 1025];
        apply_hann_window(&mut buffer);
        assert_eq!(buffer[0], 0.);
        assert_eq!(buffer[1024], 0.);
        assert_eq!(buffer[512], 1.);
    }
}
