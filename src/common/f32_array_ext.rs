//! `[f32]` extensions.

use micromath::F32Ext;

/// `[f32]` extensions.
pub trait F32ArrayExt {
    /// Returns the maximum absolute value.
    fn peak_level(&self) -> f32;
    /// Returns the maximum absolute value in dB relative to 1,
    /// i.e 0 dB corresponds to a level of 1.
    fn peak_level_db(&self) -> f32;
    /// Returns the [root mean square](https://en.wikipedia.org/wiki/Root_mean_square)
    /// level.
    fn rms_level(&self) -> f32;
    /// Returns the [root mean square](https://en.wikipedia.org/wiki/Root_mean_square)
    /// level in dB relative to 1, i.e 0 dB corresponds to a level of 1.
    fn rms_level_db(&self) -> f32;
}

impl F32ArrayExt for [f32] {
    fn peak_level(&self) -> f32 {
        if self.len() == 0 {
            return 0.0;
        };

        let mut max: f32 = 0.0;
        for sample in self.iter() {
            let value = sample.abs();
            if value > max {
                max = value
            }
        }
        max
    }

    fn peak_level_db(&self) -> f32 {
        20. * F32Ext::log10(self.peak_level())
    }

    fn rms_level(&self) -> f32 {
        if self.len() == 0 {
            return 0.0;
        };
        let mut rms: f32 = 0.;
        for sample in self.iter() {
            rms += sample * sample
        }
        F32Ext::sqrt(rms / (self.len() as f32))
    }

    fn rms_level_db(&self) -> f32 {
        20. * F32Ext::log10(self.rms_level())
    }
}

#[cfg(test)]
mod tests {
    use super::F32ArrayExt;

    #[test]
    fn test_empty_window() {
        let window: [f32; 0] = [];
        assert!(window.rms_level() == 0.0);
        assert!(window.peak_level() == 0.0);
    }
}
