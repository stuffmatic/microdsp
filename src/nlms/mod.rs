//! [Normalized least mean squares](https://en.wikipedia.org/wiki/Least_mean_squares_filter#Normalized_least_mean_squares_filter_(NLMS))
//! adaptive filter.
//!
//! # Examples
//! ## Noise cancellation
//!
//! This example uses the same noise signal for x(n) and d(n) (notation from [here](https://en.wikipedia.org/wiki/Least_mean_squares_filter#Normalized_least_mean_squares_filter_(NLMS))).
//! The expected result is convergence to an identity filter with all
//! zeros except a 1 at index 0.
//!
//! ```
//! use rand::{rngs::StdRng, Rng, SeedableRng};
//! use microdsp::common::F32ArrayExt;
//! use microdsp::nlms::NlmsFilter;
//!
//! // Generate noise signal
//! let sample_count = 10000;
//! let mut signal = vec![0.0; sample_count];
//! let mut rng = StdRng::seed_from_u64(123);
//! for i in 0..sample_count {
//!     signal[i] = rng.gen_range(-1.0..=1.0);
//! }
//!
//! // Create filter instance
//! let mut filter = NlmsFilter::from_options(10, 0.5, 0.00001);
//!
//! // Perform filtering
//! for (i, x) in signal.iter().enumerate() {
//!     let d = *x;
//!     let e = filter.update(*x, d);
//!
//!     // Give the filter time to converge
//!     if i > 200 {
//!         // The signal should be almost completely cancelled out
//!         assert!(e.abs() < 0.001);
//!         // The filter should have an identity response.
//!         assert!((filter.h()[0] - 1.0).abs() < 1e-5);
//!         assert!(filter.h()[1..].peak_level() < 1e-5);
//!     }
//! }

mod nlms_filter;

pub use nlms_filter::NlmsFilter;
