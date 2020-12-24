#[derive(Copy, Clone)]
/// A key maximum, i.e an NSDF maximum that may or may not correspond
/// to the pitch period.
pub struct KeyMaximum {
    /// The index into the NSDF array corresponding to this maximum
    pub lag_index: usize,
    /// The NSDF value at `lag_index` for this maximum
    pub value_at_lag_index: f32,
    /// The NSDF value for this maximum, approximated using parabolic interpolation.
    pub value: f32,
    /// The lag, in samples, for this maximum, approximated using parabolic interpolation.
    pub lag: f32,
}

impl KeyMaximum {
    pub(crate) fn new() -> Self {
        KeyMaximum {
            lag_index: 0,
            value_at_lag_index: 0.0,
            value: 0.0,
            lag: 0.0,
        }
    }

    pub(crate) fn set(&mut self, nsdf: &[f32], lag_index: usize) {
        self.lag_index = lag_index;
        let value_at_lag_index = nsdf[lag_index];
        self.value_at_lag_index = value_at_lag_index;

        // Use parabolic interpolation to approximate
        // the true maximum using the left and right neighbors
        let left_index = core::cmp::max(0, lag_index - 1);
        let right_index = core::cmp::min(nsdf.len() - 1, lag_index + 1);
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
