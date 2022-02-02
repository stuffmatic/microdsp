use core::convert::TryInto;
use alloc::{vec, boxed::Box};
use microfft;

use crate::{snov::{compression::CompressionFunction}, common::window::WindowFunction};

pub fn real_fft_in_place(buffer: &mut [f32]) -> &mut [microfft::Complex32] {
    let fft_size = buffer.len();
    match fft_size {
        8 => microfft::real::rfft_8(buffer.try_into().unwrap()),
        16 => microfft::real::rfft_16(buffer.try_into().unwrap()),
        32 => microfft::real::rfft_16(buffer.try_into().unwrap()),
        64 => microfft::real::rfft_64(buffer.try_into().unwrap()),
        128 => microfft::real::rfft_128(buffer.try_into().unwrap()),
        256 => microfft::real::rfft_256(buffer.try_into().unwrap()),
        512 => microfft::real::rfft_512(buffer.try_into().unwrap()),
        1024 => microfft::real::rfft_1024(buffer.try_into().unwrap()),
        2048 => microfft::real::rfft_2048(buffer.try_into().unwrap()),
        4096 => microfft::real::rfft_4096(buffer.try_into().unwrap()),
        _ => panic!("Unsupported fft size {}", fft_size),
    }
}
// https://www.audiolabs-erlangen.de/resources/MIR/FMP/C6/C6S1_NoveltySpectral.html
pub struct SpectralFluxNovelty {
    power_0: Box<[f32]>,
    power_1: Box<[f32]>,
    d_power: Box<[f32]>,
    novelty: f32,
    prev_is_1: bool,
    has_processed_second_window: bool,
}

impl SpectralFluxNovelty {
    pub fn new(downsampled_window_size: usize) -> Self {
        SpectralFluxNovelty {
            power_0: vec![0.; downsampled_window_size / 2].into_boxed_slice(),
            power_1: vec![0.; downsampled_window_size / 2].into_boxed_slice(),
            d_power: vec![0.; downsampled_window_size].into_boxed_slice(),
            novelty: 0.,
            prev_is_1: true,
            has_processed_second_window: false,
        }
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

    pub fn process_window<W: WindowFunction, C: CompressionFunction>(
        &mut self,
        window: &[f32],
        window_func: &W,
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
        window_func.apply(&mut self.d_power);
        let fft = real_fft_in_place(&mut self.d_power);
        // Clear real-valued coefficient at the Nyquist frequency, which is packed into the
        // imaginary part of the DC bin.
        fft[0].im = 0.;

        for (power, z) in power.iter_mut().zip(fft) {
            *power = compression_func.compress(z.norm_sqr());
        }

        let mut novelty = 0.;
        if self.has_processed_second_window {
            for i in 0..power.len() { // TODO: use zip etc
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
