//! [Normalized least mean squares](https://en.wikipedia.org/wiki/Least_mean_squares_filter#Normalized_least_mean_squares_filter_(NLMS))
//! adaptive filter. Can for example be used for echo cancellation and time delay estimation.

mod nlms_filter;

pub use nlms_filter::NlmsFilter;