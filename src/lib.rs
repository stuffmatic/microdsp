//! A rust implementation of the MPM (McLeod Pitch Method) [pitch](https://en.wikipedia.org/wiki/Pitch_%28music%29) detection algorithm.
//! The algorithm is used for detecting pitch in monophonic, primarily musical, sounds. It
//! cannot be used to detect multiple pitches at once, like in a musical chord.
//! The algorithm is described in the paper [A smarter way to find pitch](http://www.cs.otago.ac.nz/tartini/papers/A_Smarter_Way_to_Find_Pitch.pdf)
//! by Philip McLeod and Geoff Wyvill.
//!
//! * Reasonably performant - implements the optimizations suggested in the paper,
//! including FFT based autocorrelation computation.
//! * Suitable for real time audio use - only allocates a modest amount of memory on initialization.
//!
//! # Examples
//! ## High level API (recommended)
//! Handles collecting input samples into possibly overlapping windows and processing each newly filled window.
//! ```
//! use mpm_pitch::Detector;
//!
//! // Create a pitch detector instance
//! let sample_rate = 44100.0;
//! let window_size = 512; // The number of samples to perform pitch detection on.
//! let window_overlap = 128; // Pitch is computed every window_size - window_overlap samples
//! let mut detector = Detector::new(sample_rate, window_size, window_overlap);
//!
//! // Create an input buffer containing a pure tone at 440 Hz.
//! let mut chunk: Vec<f32> = vec![0.0; 10000];
//! for i in 0..chunk.len() {
//!     let sine_value = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sample_rate).sin();
//!     chunk[i] = sine_value;
//! }
//!
//! // Perform pitch detection. The detector extracts and processes windows and
//! // invokes the provided callback when a new window has been analyzed.
//! detector.process(&chunk[..], |sample_index, result| {
//!     let time_s = sample_rate * (sample_index as f32);
//!     if result.is_tone() {
//!         println!("t = {} s, Frequency {} Hz, clarity {}", time_s, result.frequency, result.clarity);
//!     } else {
//!         // No discernable pitch detected.
//!     }
//! });
//! ```
//! ## Low level API
//! Used to process a window directly. Useful for profiling and testing or if you want roll your own window handling.
//! ```
//! use mpm_pitch::Result;
//!
//! // Create an instance of PitchDetectionResult
//! let sample_rate = 44100.0;
//! let window_size = 512;
//! let lag_count = 256;
//! let mut result = Result::new(window_size, lag_count);
//!
//! // Fill the window to process with a pure tone at 440 Hz.
//! for i in 0..window_size {
//!     let sine_value = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sample_rate).sin();
//!     result.window[i] = sine_value;
//! }
//!
//! // Perform pitch detection
//! result.compute(sample_rate);
//! println!("Frequency {} Hz, clarity {}", result.frequency, result.clarity);
//! ```

mod detector;
mod key_maximum;
mod result;
mod util;

pub use detector::Detector;
pub use key_maximum::KeyMaximum;
pub use result::Result;
