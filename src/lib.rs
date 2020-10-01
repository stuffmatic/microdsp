//! A rust implementation of the MPM (McLeod Pitch Method) [pitch](https://en.wikipedia.org/wiki/Pitch_%28music%29) detection algorithm.
//! The algorithm is used for detecting pitch in monophonic, primarily musical, sounds. It
//! cannot be used to detect multiple pitches at once, like in a musical chord.
//! The algorithm is described in the paper [A smarter way to find pitch](http://www.cs.otago.ac.nz/tartini/papers/A_Smarter_Way_to_Find_Pitch.pdf)
//! by Philip McLeod and Geoff Wyvill.
//!
//! Features
//! * Includes the optimizations suggested in the above paper, including
//! FFT accelerated autocorrelation computation
//! * No allocations, suitable for real time audio use.
//!
//! # Examples
//! ## Streaming API
//! Used for passing (sequences of) chunks of arbitrary size to the processor. Handles collecting
//! samples into (possibly overlapping) windows and processing each newly filled window.
//!
//! ```
//! use mpm_pitch::PitchDetector;
//! use mpm_pitch::ProcessingResult;
//!
//! // Create a pitch detector instance
//! let sample_rate = 44100.0;
//! let window_size = 512;
//! let window_overlap = 128;
//! let mut detector = PitchDetector::new(sample_rate, window_size, window_overlap, true);
//!
//! // Create an input buffer containing a pure tone at 440 Hz.
//! let mut chunk: Vec<f32> = vec![0.0; 10000];
//! for i in 0..chunk.len() {
//!     let sine_value = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sample_rate).sin();
//!     chunk[i] = sine_value;
//! }
//!
//! // Call process until all samples have been consumed. Each call consumes
//! // at most W samples, where W is the window size.
//! let mut sample_offset: usize = 0;
//! while sample_offset < chunk.len() {
//!     match detector.process(&chunk[..], sample_offset) {
//!         ProcessingResult::ProcessedWindow { sample_index } => {
//!             // Consumed enough samples to fill and process another window.
//!             // Inspect the result if it's valid.
//!             let result = &detector.result;
//!             if result.is_valid() {
//!                 println!("Frequency {} Hz, clarity {}", result.frequency, result.clarity);
//!             }
//!
//!             // Sample_index is the index of the next sample to process.
//!             sample_offset = sample_index;
//!         },
//!         ProcessingResult::ReachedEndOfBuffer => {
//!             // Finished processing the current chunk.
//!             break;
//!         }
//!     }
//! }
//! ```
//! ## Single window API
//! Used to process a window directly. Useful for profiling and testing.
//! ```
//! use mpm_pitch::PitchDetectionResult;
//!
//! // Create an instance of PitchDetectionResult
//! let sample_rate = 44100.0;
//! let window_size = 512;
//! let lag_count = 256;
//! let mut result = PitchDetectionResult::new(window_size, lag_count);
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

mod key_maximum;
mod equal_loudness_filter;
mod pitch_detection_result;
mod pitch_detector;

pub use key_maximum::KeyMaximum;
pub use pitch_detection_result::PitchDetectionResult;
pub use pitch_detector::PitchDetector;
pub use pitch_detector::ProcessingResult;
