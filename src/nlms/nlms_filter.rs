use alloc::{vec, vec::Vec};

/// An adaptive [normalized least mean squares filter](https://en.wikipedia.org/wiki/Least_mean_squares_filter#Normalized_least_mean_squares_filter_(NLMS)).
/// Using the same notation as in the linked description.
pub struct NlmsFilter {
    /// FIR filter coefficients
    h: Vec<f32>,
    /// Most recent input values. Newest sample is at index 0.
    x: Vec<f32>,
    /// Step size scale
    μ: f32,
    /// Running sum of current input signal power.
    x_power: f32,
    /// Constant added to the update step denominator to avoid division by zero.
    ε: f32,
    buffer_pos: usize,
}

impl NlmsFilter {
    pub fn new(order: usize) -> Self {
        NlmsFilter::from_options(order, 0.02, 1e-3)
    }

    pub fn from_options(order: usize, mu: f32, eps: f32) -> Self {
        NlmsFilter {
            h: vec![0.0; order],
            // Trade memory usage for update speed:
            // double size for x to avoid index wrapping in the update method.
            x: vec![0.0; 2 * order],
            μ: mu,
            ε: eps,
            buffer_pos: 0,
            x_power: 0.0
        }
    }

    pub fn h(&mut self) -> &[f32] {
        &self.h
    }

    pub fn order(&mut self) -> usize {
        self.h.len()
    }

    pub fn update(&mut self, x: f32, d: f32) -> f32 {
        assert!(self.buffer_pos < self.order());
        let order = self.order();
        self.x[self.buffer_pos] = x;
        self.x[self.buffer_pos + order] = x;

        // Add current input to signal power
        self.x_power += x * x;

        // Compute filter output y = h applied to x.
        let mut y = 0.0;
        for (h, x) in self.h.iter().zip(self.x.iter().skip(self.buffer_pos)) {
            y += h * *x
        }

        let e = d - y;
        let delta_scale = self.μ * e / (self.x_power + self.ε);
        for (h, x) in self.h.iter_mut().zip(self.x.iter().skip(self.buffer_pos)) {
            *h += delta_scale * *x;
        }

        // Subtract oldest input sample from signal power and advance buffer position
        let next_buffer_pos = if self.buffer_pos == 0 {
            self.order() - 1
        } else {
            self.buffer_pos - 1
        };
        let x_oldest = self.x[next_buffer_pos];
        self.x_power -= x_oldest * x_oldest;
        self.buffer_pos = next_buffer_pos;

        e
    }

    /*pub fn update_old(&mut self, x: f32, d: f32) -> f32 {
        assert!(self.buffer_pos < self.order());
        self.x[self.buffer_pos] = x;
        let order = self.order();

        let prev_idx = |i: usize, buffer_pos: usize| -> usize {
            if i > buffer_pos {
                (order + buffer_pos) - i
            } else {
                buffer_pos - i
            }
        };

        // Add current input to signal power
        self.x_power += x * x;

        // Compute filter output y = h applied to x.
        let mut y = 0.0;
        for (i, h) in self.h.iter().enumerate() {
            let x_idx = prev_idx(i, self.buffer_pos);
            y += h * self.x[x_idx]
        }

        let e = d - y;
        let delta_scale = self.μ * e / (self.x_power + self.ε);
        for (i, h) in self.h.iter_mut().enumerate() {
            let x_idx = prev_idx(i, self.buffer_pos);
            let delta = delta_scale * self.x[x_idx];
            *h += delta;
        }

        // Subtract oldest input sample from signal power
        let next_buffer_pos = if self.buffer_pos == self.order() - 1 {
            0
        } else {
            self.buffer_pos + 1
        };
        let x_oldest = self.x[next_buffer_pos];
        self.x_power -= x_oldest * x_oldest;
        self.buffer_pos = next_buffer_pos;

        e
    }*/

    pub fn reset(&mut self) {
        for i in 0..self.order() {
            self.h[i] = 0.0;
            self.x[i] = 0.0;
            self.buffer_pos = 0;
            self.x_power = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::common::F32ArrayExt;

    use super::*;

    #[test]
    fn test_nlms_iterations() {
        let h_expected: [[f32; 3]; 6] = [
            [0.0, 0.0, 0.0],
            [1.4985014, 0.0, 0.0],
            [1.6990607, 0.10027966, 0.0],
            [1.9885677, 0.29328436, 0.09650234],
            [1.9866968, 0.29188123, 0.09556692],
            [1.7673157, 0.116376325, -0.036061756],
        ];
        let mut filter = NlmsFilter::from_options(3, 0.5, 0.001);
        let x: [f32; 6] = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let d: [f32; 6] = [1.0, 3.0, 4.0, 8.0, 9.0, 7.0];
        for (i, (x, d)) in x.iter().zip(d.iter()).enumerate() {
            filter.update(*x, *d);
            for (h, h_expected) in filter.h().iter().zip(h_expected[i].iter()) {
                assert_eq!(*h, *h_expected);
            }
        }
    }

    #[test]
    fn test_nlms_cancellation() {
        // Use the same noise signal as x(n) and d(n). The expected
        // result is convergence to an identity FIR filter with all
        // zeros except a 1 at index 0.

        // Generate noise signal
        let sample_count = 10000;
        let mut signal = vec![0.0; sample_count];
        let mut rng = StdRng::seed_from_u64(123);
        for i in 0..sample_count {
            signal[i] = rng.gen_range(-1.0..=1.0);
        }

        // Create filter instance
        let mut filter = NlmsFilter::from_options(10, 0.5, 0.00001);

        // Perform filtering
        for (i, x) in signal.iter().enumerate() {
            let e = filter.update(*x, *x);

            // Give the filter time to converge
            if i > 200 {
                // The signal should be almost completely cancelled out
                assert!(e.abs() < 0.001);
                // The FIR filter should have an identity response.
                assert!((filter.h()[0] - 1.0).abs() < 1e-5);
                assert!(filter.h()[1..].peak_level() < 1e-5);

            }
        }
    }
}
