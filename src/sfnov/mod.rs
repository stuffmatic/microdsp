//! [Audio onset detection](https://en.wikipedia.org/wiki/Onset_(audio)) using
//! [spectral flux novelty](https://krishnasubramani.web.illinois.edu/data/Energy-Weighted%20Multi-Band%20Novelty%20Functions%20for%20Onset%20Detection%20in%20Piano%20Music.pdf).
//!
mod compression_function;
mod spectral_flux;
mod spectral_flux_novelty_detector;

pub use compression_function::{CompressionFunction, HardKneeCompression, QuarticCompression};
pub use spectral_flux::SpectralFlux;
pub use spectral_flux_novelty_detector::SpectralFluxNoveltyDetector;
