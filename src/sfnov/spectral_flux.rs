use alloc::{boxed::Box, vec};

use crate::{
    common::{real_fft, apply_window_function},
    sfnov::compression_function::CompressionFunction,
};

// https://www.audiolabs-erlangen.de/resources/MIR/FMP/C6/C6S1_NoveltySpectral.html
pub struct SpectralFlux {
    power_0: Box<[f32]>,
    power_1: Box<[f32]>,
    d_power: Box<[f32]>,
    novelty: f32,
    prev_is_1: bool,
    has_processed_second_window: bool,
}

struct AllocatedBuffers {
    power_0: Box<[f32]>,
    power_1: Box<[f32]>,
    d_power: Box<[f32]>,
}

impl AllocatedBuffers {
    fn new(window_size: usize) -> Self {
        AllocatedBuffers {
            power_0: vec![0.; window_size / 2].into_boxed_slice(),
            power_1: vec![0.; window_size / 2].into_boxed_slice(),
            d_power: vec![0.; window_size].into_boxed_slice(),
        }
    }
}

impl SpectralFlux {
    pub fn new(window_size: usize) -> Self {
        let buffers = AllocatedBuffers::new(window_size);
        SpectralFlux {
            power_0: buffers.power_0,
            power_1: buffers.power_1,
            d_power: buffers.d_power,
            novelty: 0.,
            prev_is_1: true,
            has_processed_second_window: false,
        }
    }

    pub fn reallocate(&mut self, window_size: usize) {
        let buffers = AllocatedBuffers::new(window_size);
        self.power_0 = buffers.power_0;
        self.power_1 = buffers.power_1;
        self.d_power = buffers.d_power;
    }

    pub fn novelty(&self) -> f32 {
        self.novelty
    }

    pub fn clear(&mut self) {
        self.prev_is_1 = true;
        self.has_processed_second_window = false;
        self.novelty = 0.;
    }

    pub fn power_spectrum(&self) -> &[f32] {
        if self.prev_is_1 {
            &self.power_0
        } else {
            &self.power_1
        }
    }

    pub fn power_spectrum_prev(&self) -> &[f32] {
        if self.prev_is_1 {
            &self.power_1
        } else {
            &self.power_0
        }
    }

    pub fn d_power(&self) -> &[f32] {
        &self.d_power
    }

    pub fn process_window<C: CompressionFunction>(
        &mut self,
        window: &[f32],
        window_func: crate::common::WindowFunctionType,
        compression_func: &C,
    ) -> bool {
        let (power, power_prev) = if self.prev_is_1 {
            (&mut self.power_0, &mut self.power_1)
        } else {
            (&mut self.power_1, &mut self.power_0)
        };

        if !self.prev_is_1 && !self.has_processed_second_window {
            self.has_processed_second_window = true;
        }

        self.d_power.copy_from_slice(window);
        apply_window_function(window_func, &mut self.d_power);
        let fft = real_fft(&mut self.d_power);
        // Clear real-valued coefficient at the Nyquist frequency, which is packed into the
        // imaginary part of the DC bin.
        fft[0].im = 0.;

        for (power, z) in power.iter_mut().zip(fft) {
            // magnitude is compressed in https://www.audiolabs-erlangen.de/resources/MIR/FMP/C6/C6S1_NoveltySpectral.html
            // TODO: compressing norm s
            *power = compression_func.compress(z.norm_sqr());
        }

        let mut novelty = 0.;
        if self.has_processed_second_window {
            for i in 0..power.len() {
                // TODO: use zip etc
                let delta = power[i] - power_prev[i];
                self.d_power[i] = delta;
                if delta > 0. {
                    novelty += delta;
                }
            }
        }
        self.novelty = novelty / (self.d_power.len() as f32); // TODO: proper normalization
        self.prev_is_1 = !self.prev_is_1;
        self.has_processed_second_window
    }
}
