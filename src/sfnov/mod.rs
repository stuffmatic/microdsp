//! [Audio onset detection](https://en.wikipedia.org/wiki/Onset_(audio)) using
//! [spectral flux novelty](https://krishnasubramani.web.illinois.edu/data/Energy-Weighted%20Multi-Band%20Novelty%20Functions%20for%20Onset%20Detection%20in%20Piano%20Music.pdf).
//! Used for detecting the start of musical notes and other sounds.
//!
//! # Examples
//! ## Streaming input
//! Handles collecting input samples into possibly
//! overlapping windows and processing each newly filled window.
//!
pub mod compression_function;
pub mod spectral_flux_novelty_detector;
pub mod spectral_flux;