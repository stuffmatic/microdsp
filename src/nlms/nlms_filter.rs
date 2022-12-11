/// An adaptive normalized least mean squares filter.
/// https://en.wikipedia.org/wiki/Least_mean_squares_filter
pub struct NlmsFilter<const O: usize> {
  h: [f32; O],
  x: [f32; O],
  mu: f32,
  eps: f32,
  buffer_pos: usize,
}

impl<const O: usize> NlmsFilter<O> {
  pub fn new() -> Self {
      NlmsFilter::from_options(0.02, 1e-3)
  }

  pub fn from_options(mu: f32, eps: f32) -> Self {
      NlmsFilter {
          h: [0.0; O],
          x: [0.0; O],
          mu,
          eps,
          buffer_pos: 0,
      }
  }

  pub fn h(&mut self) -> &[f32] {
    &self.h
  }

  pub fn update(&mut self, x: f32, d: f32) -> f32 {
      assert!(self.buffer_pos < O);
      self.x[self.buffer_pos] = x;

      let prev_idx = |i: usize, buffer_pos: usize| -> usize {
          if i > buffer_pos {
              (O + buffer_pos) - i
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
      let delta_scale = self.mu * e / (power + self.eps);
      for (i, h) in self.h.iter_mut().enumerate() {
          let x_idx = prev_idx(i, self.buffer_pos);
          let delta = delta_scale * self.x[x_idx];
          *h += delta;
      }

      self.buffer_pos = if self.buffer_pos == O - 1 {
          0
      } else {
          self.buffer_pos + 1
      };

      e
  }

  pub fn reset(&mut self) {
      for i in 0..O {
          self.h[i] = 0.0;
          self.x[i] = 0.0;
          self.buffer_pos = 0;
      }
  }
}