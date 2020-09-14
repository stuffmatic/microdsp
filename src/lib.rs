//! A rust implementation of the MPM (McLeod Pitch Method) [pitch](https://en.wikipedia.org/wiki/Pitch_%28music%29) detection algorithm.
//! The algorithm is used for detecting pitch in monophonic, primarily musical, sounds. It
//! cannot be used to detect multiple pitches at once, like in a musical chord.
//! The algorithm is described in the paper [A smarter way to find pitch](http://www.cs.otago.ac.nz/tartini/papers/A_Smarter_Way_to_Find_Pitch.pdf)
//! by Philip McLeod and Geoff Wyvill.
//!
//! TODO: FFT accelerated using microfft.

mod equal_loudness_filter;
pub mod key_maximum;
pub mod pitch_detection_result;
pub mod pitch_detector;