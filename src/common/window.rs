use micromath::F32Ext;

pub trait Window {
    fn peak_level(&self) -> f32;
    fn peak_level_db(&self) -> f32;
    fn rms_level(&self) -> f32;
    fn rms_level_db(&self) -> f32;
}

impl Window for [f32] {
    /// The maximum absolute value of the input window.
    fn peak_level(&self) -> f32 {
        let mut max: f32 = 0.0;
        for sample in self.iter() {
            let value = sample.abs();
            if value > max {
                max = value
            }
        }
        max
    }

    /// The maximum absolute value of the input window, in dB relative to 1,
    /// i.e 0 dB corresponds to a level of 1.
    fn peak_level_db(&self) -> f32 {
        20. * F32Ext::log10(self.peak_level())
    }

    /// The [root mean square](https://en.wikipedia.org/wiki/Root_mean_square) level
    /// of the input window.
    fn rms_level(&self) -> f32 {
        let mut rms: f32 = 0.;
        for sample in self.iter() {
            rms += sample * sample
        }
        F32Ext::sqrt(rms / (self.len() as f32))
    }

    /// The [root mean square](https://en.wikipedia.org/wiki/Root_mean_square) level
    /// of the input window, in dB relative to 1, i.e 0 dB corresponds to a level of 1.
    fn rms_level_db(&self) -> f32 {
        20. * F32Ext::log10(self.rms_level())
    }
}

#[cfg(test)]
mod tests {
    use super::Window;

    #[test]
    fn test_empty_window() {
        let window: [f32; 0] = [];
        assert!(window.rms_level() == 0.0)
    }
}