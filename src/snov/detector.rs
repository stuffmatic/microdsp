use crate::common::window_function::{HannWindow, WindowFunction};
use crate::common::window_processor::WindowProcessor;
use crate::snov::{
    compression_function::{CompressionFunction, HardKneeCompression},
    novelty::SpectralFluxNovelty,
};

pub struct SpectralNoveltyDetector<W: WindowFunction, C: CompressionFunction> {
    window_processor: WindowProcessor,
    novelty: SpectralFluxNovelty,
    window_func: W,
    compression_func: C,
}

impl SpectralNoveltyDetector<HannWindow, HardKneeCompression> {
    pub fn new(window_size: usize) -> Self {
        SpectralNoveltyDetector {
            window_processor: WindowProcessor::new(window_size, window_size / 2, 1),
            window_func: HannWindow::new(),
            compression_func: HardKneeCompression::new(),
            novelty: SpectralFluxNovelty::new(window_size),
        }
    }
}

impl<W: WindowFunction, C: CompressionFunction> SpectralNoveltyDetector<W, C> {
    pub fn from_options(
        window_func: W,
        compression_func: C,
        downsampled_window_size: usize,
        downsampling: usize,
        downsampled_hop_size: usize,
    ) -> Self {
        SpectralNoveltyDetector {
            window_processor: WindowProcessor::new(
                downsampled_window_size,
                downsampled_hop_size,
                downsampling,
            ),
            window_func,
            compression_func,
            novelty: SpectralFluxNovelty::new(downsampled_window_size),
        }
    }

    pub fn window_function(&mut self) -> &W {
        &mut self.window_func
    }

    pub fn compression_function(&mut self) -> &C {
        &mut self.compression_func
    }

    pub fn reset(&mut self) {
        self.window_processor.reset();
        self.novelty.clear()
    }

    pub fn process<F>(&mut self, buffer: &[f32], mut handler: F)
    where
        F: FnMut(&SpectralFluxNovelty),
    {
        let novelty = &mut self.novelty;
        let window_func = &self.window_func;
        let compression_func = &self.compression_func;
        self.window_processor.process(buffer, |window| {
            if novelty.process_window(window, window_func, compression_func) {
                handler(&novelty)
            }
        })
    }
}
