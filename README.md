# microdsp

microdsp is a collection of [DSP](https://en.wikipedia.org/wiki/Digital_signal_processing)
algorithms and utilities suitable for use in embedded systems.
The crate is `no_std` compatible and relies
on [`alloc`](https://doc.rust-lang.org/alloc/).
When building for targets without a default allocator,
one must be provided by the user. This can be accomplished in stable Rust 1.67 and higher
using `#[global_allocator]` and `#[default_alloc_error_handler]`.

## Algorithms

* Monophonic [pitch](https://en.wikipedia.org/wiki/Pitch_%28music%29) detection using the [MPM](papers/A smarter way to find pitch.pdf) (McLeod Pitch Method) algorithm.
* [Normalized least mean squares](https://en.wikipedia.org/wiki/Least_mean_squares_filter#Normalized_least_mean_squares_filter_(NLMS)) adaptive filter. Can for example be used for echo cancellation and time delay estimation.
* [Audio onset detection](https://en.wikipedia.org/wiki/Onset_(audio)) using [spectral flux novelty](https://krishnasubramani.web.illinois.edu/data/Energy-Weighted%20Multi-Band%20Novelty%20Functions%20for%20Onset%20Detection%20in%20Piano%20Music.pdf). Used for detecting the start of musical notes and other sounds.

## Examples

`portaudio`

## Demos

* web demo - wasm
* running on nrf bla bla
* running on desktop



