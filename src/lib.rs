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
//!
//! Streaming API, handing windowing and overlapping windows.
//! Used for passing (sequences of) chunks of arbitrary size to the processor.
//!
//! ```
//! /*let sample_rate = 44100;
//! let mut detector = PitchDetector::new(sample_rate, window_size, window_distance, true);
//! // Keep processing until we reach the last sample of the chunk.
//!
//! // This is the current offset into the chunk
//! let mut sample_offset: usize = 0;
//! // As long as there are samples left in the chunk, call process. This consumes
//! // at most W samples, where W is the window size.
//! while sample_offset < chunk.len() {
//!     match detector.process(&chunk[..], sample_offset) {
//!         ProcessingResult::ProcessedWindow { sample_index } => {
//!             // Consumed enough samples to fill and process another window. Inspect the result
//!             let result = &detector.result;
//!             if result.is_valid() {
//!                 println!("pitch {} Hz, clarity {}", result.pitch, result.clarity);
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
//! }*/
//! ```
//! Single window API, used to process a window directly. Useful for profiling and testing.
//! ```
//! /*let sample_rate = 44100;
//! let result = PitchDetectionResult::new(window_size, lag_count);
//! result.window[i] = ...;
//! result.compute(sample_rate);*/
//! ```

mod equal_loudness_filter;
pub mod key_maximum;
pub mod pitch_detection_result;
pub mod pitch_detector;