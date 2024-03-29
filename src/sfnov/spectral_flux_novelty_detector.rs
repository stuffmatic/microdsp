use crate::common::WindowFunctionType;
use crate::common::WindowProcessor;
use crate::sfnov::{
    compression_function::{CompressionFunction, HardKneeCompression},
    spectral_flux::SpectralFlux,
};

pub struct SpectralFluxNoveltyDetector<C: CompressionFunction> {
    window_processor: WindowProcessor,
    flux: SpectralFlux,
    window_func: WindowFunctionType,
    compression_func: C,
}

impl SpectralFluxNoveltyDetector<HardKneeCompression> {
    pub fn new(window_size: usize) -> Self {
        SpectralFluxNoveltyDetector {
            window_processor: WindowProcessor::new(1, window_size, window_size / 2),
            window_func: WindowFunctionType::Hann,
            compression_func: HardKneeCompression::new(),
            flux: SpectralFlux::new(window_size),
        }
    }
}

impl<C: CompressionFunction> SpectralFluxNoveltyDetector<C> {
    pub fn from_options(
        window_func: WindowFunctionType,
        compression_func: C,
        downsampled_window_size: usize,
        downsampling: usize,
        downsampled_hop_size: usize,
    ) -> Self {
        SpectralFluxNoveltyDetector {
            window_processor: WindowProcessor::new(
                downsampled_window_size,
                downsampled_hop_size,
                downsampling,
            ),
            window_func,
            compression_func,
            flux: SpectralFlux::new(downsampled_window_size),
        }
    }

    pub fn compression_function(&mut self) -> &C {
        &mut self.compression_func
    }

    pub fn reset(&mut self) {
        self.window_processor.reset();
        self.flux.clear()
    }

    pub fn novelty(&self) -> &SpectralFlux {
        &self.flux
    }

    pub fn process<F>(&mut self, buffer: &[f32], mut handler: F)
    where
        F: FnMut(&SpectralFlux),
    {
        let flux = &mut self.flux;
        let window_func = self.window_func;
        let compression_func = &self.compression_func;
        self.window_processor.process(buffer, |window| {
            if flux.process_window(window, window_func, compression_func) {
                handler(&flux)
            }
        })
    }
}
