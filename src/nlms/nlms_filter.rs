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
            x: vec![0.0; order],
            μ: mu,
            ε: eps,
            buffer_pos: 0,
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
        self.x[self.buffer_pos] = x;
        let order = self.order();

        let prev_idx = |i: usize, buffer_pos: usize| -> usize {
            if i > buffer_pos {
                (order + buffer_pos) - i
            } else {
                buffer_pos - i
            }
        };

        // Compute input signal power. Used to scale step size.
        let mut power = 0.0;
        for x in self.x.iter() {
            power += x * x;
        }

        // Compute filter output y = h applied to x.
        let mut y = 0.0;
        for (i, h) in self.h.iter().enumerate() {
            let x_idx = prev_idx(i, self.buffer_pos);
            y += h * self.x[x_idx]
        }

        let e = d - y;
        let delta_scale = self.μ * e / (power + self.ε);
        for (i, h) in self.h.iter_mut().enumerate() {
            let x_idx = prev_idx(i, self.buffer_pos);
            let delta = delta_scale * self.x[x_idx];
            *h += delta;
        }

        self.buffer_pos = if self.buffer_pos == self.order() - 1 {
            0
        } else {
            self.buffer_pos + 1
        };

        e
    }

    pub fn reset(&mut self) {
        for i in 0..self.order() {
            self.h[i] = 0.0;
            self.x[i] = 0.0;
            self.buffer_pos = 0;
        }
    }
}
