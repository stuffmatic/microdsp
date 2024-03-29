//! An implementation of the MPM [pitch](https://en.wikipedia.org/wiki/Pitch_%28music%29) detection algorithm,
//! described in the paper [A smarter way to find pitch](http://www.cs.otago.ac.nz/tartini/papers/A_Smarter_Way_to_Find_Pitch.pdf).
//!
//! # Examples
//! ## Streaming input
//! Handles collecting input samples into possibly
//! overlapping windows and processing each newly filled window.
//! ```
//! use microdsp::mpm::MpmPitchDetector;
//!
//! // Create an input buffer containing a pure tone at 440 Hz.
//! let sample_rate = 44100.0;
//! let sine_frequency = 440.0;
//! let mut chunk: Vec<f32> = vec![0.0; 10000];
//! for i in 0..chunk.len() {
//!     let sine_value = (2.0 * core::f32::consts::PI * sine_frequency * (i as f32) / sample_rate).sin();
//!     chunk[i] = sine_value;
//! }
//!
//! // Create a pitch detector instance
//! let window_size = 512; // The number of samples to perform pitch detection on.
//! let hop_size = 128; // Pitch is computed every hop_size samples
//! let mut detector = MpmPitchDetector::new(sample_rate, window_size, hop_size);
//!
//! // Perform pitch detection. The detector extracts and processes windows and
//! // invokes the provided callback when a new window has been analyzed.
//! detector.process(&chunk[..], |result| {
//!     if result.is_tone() {
//!         println!("Frequency {} Hz, clarity {}", result.frequency, result.clarity);
//!         assert!((sine_frequency - result.frequency).abs() <= 0.01);
//!     } else {
//!         // No discernable pitch detected. Should not end up here, since
//!         // the input signal is a pure tone.
//!         assert!(false);
//!     }
//! });
//! ```
//! ## Single window
//! Used to process a window directly.
//! ```
//! use microdsp::mpm::MpmPitchResult;
//!
//! // Create a Result instance
//! let sample_rate = 44100.0;
//! let sine_frequency = 440.0;
//! let window_size = 512;
//! let lag_count = 256;
//! let mut result = MpmPitchResult::new(window_size, lag_count);
//!
//! // Fill the window to process with a pure tone at 440 Hz.
//! for i in 0..window_size {
//!     let sine_value = (2.0 * std::f32::consts::PI * sine_frequency * (i as f32) / sample_rate).sin();
//!     result.window[i] = sine_value;
//! }
//!
//! // Perform pitch detection
//! result.compute(sample_rate);
//! println!("Expected frequency {}, Detected frequency {} Hz, clarity {}", sine_frequency, result.frequency, result.clarity);
//! assert!((sine_frequency - result.frequency).abs() <= 0.01);
//! ```
//! # A note on clarity and false positives
//! TL;DR: Use the [is_tone](struct.Result.html#method.is_tone) method to check if
//! the input signal is a tone, i.e has a strong fundamental frequency.
//!
//! The result from the MPM algorithm includes a normalized clarity value,
//! which is a number between zero and one that indicates to what degree
//! an input signal is a pure tone. The clarity is defined as the value of the
//! normalized square difference function (NSDF) at the peak assumed to
//! correspond to the pitch period. However, noisy non-tonal input may
//! give rise to occasional large NSDF peaks, which means that for a single window,
//! looking at the clarity value alone is not enough to tell whether
//! the input signal has a discernable fundamental frequency.
//!
//! An input signal with a strong fundamental frequency will result
//! in a number of equispaced NSDF maxima, the distance between which corresponds to the
//! fundamental period. If no such maxima exist, a result can be safely categorized
//! as non-tonal. This check is implemented in the [is_tone](struct.Result.html#method.is_tone)
//! method, which is the recommended way to determine if the input signal has a
//! strong fundamental frequency.

mod key_max;
mod mpm_pitch_detector;
mod result;
mod util;

pub use key_max::KeyMax;
pub use mpm_pitch_detector::MpmPitchDetector;
pub use result::MpmPitchResult;
