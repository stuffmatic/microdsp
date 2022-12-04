pub trait CompressionFunction {
    fn compress(&self, input: f32) -> f32;
}

pub struct QuarticCompression {
    a: f32,
    a_4: f32,
    b_m_a: f32,
    scale: f32,
}

impl QuarticCompression {
    pub fn new(a: f32, b: f32) -> Self {
        let a_4 = a * a * a * a;
        let b_4 = b * b * b * b;
        let b_m_a = b - a;
        let scale = 1. / (a_4 - b_4);
        QuarticCompression {
            a,
            a_4,
            b_m_a,
            scale,
        }
    }
}

impl CompressionFunction for QuarticCompression {
    fn compress(&self, input: f32) -> f32 {
        let x = self.a + input * self.b_m_a;
        let x_2 = x * x;
        let x_4 = x_2 * x_2;

        (self.a_4 - x_4) * self.scale
    }
}

pub struct HardKneeCompression {
    k_0: f32,
    m_0: f32,
    k_1: f32,
    m_1: f32,
    x_knee: f32,
}

impl HardKneeCompression {
    pub fn new() -> Self {
        HardKneeCompression::from_options(0.1, 0.7)
    }

    pub fn from_options(x_knee: f32, y_knee: f32) -> Self {
        let mut instance = HardKneeCompression {
            k_0: 0.,
            k_1: 0.,
            m_0: 0.,
            m_1: 0.,
            x_knee: 0.,
        };
        instance.set(x_knee, y_knee);
        instance
    }

    pub fn set(&mut self, x_knee: f32, y_knee: f32) {
        let k_0 = y_knee / x_knee;
        let m_0 = 0.;
        let k_1 = (y_knee - 1.) / (x_knee - 1.);
        let m_1 = 1. - k_1;
        self.k_0 = k_0;
        self.m_0 = m_0;
        self.k_1 = k_1;
        self.m_1 = m_1;
        self.x_knee = x_knee;
    }
}

impl CompressionFunction for HardKneeCompression {
    fn compress(&self, input: f32) -> f32 {
        let (k, l) = if input < self.x_knee {
            (self.k_0, self.m_0)
        } else {
            (self.k_1, self.m_1)
        };
        k * input + l
    }
}

#[cfg(test)]
mod tests {
    use crate::snov::compression_function::{CompressionFunction, QuarticCompression};

    #[test]
    fn test_quartic_compression() {
        let function = QuarticCompression::new(-24., -4.);
        let c_0 = function.compress(0.);
        let c_1 = function.compress(1.);
        assert_eq!(c_0, 0.);
        assert_eq!(c_1, 1.);
        assert_eq!(function.compress(0.6), 0.9382239);
    }
}
