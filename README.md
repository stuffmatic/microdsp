# microdsp

[![Crates.io](https://img.shields.io/crates/v/microdsp)](https://crates.io/crates/microdsp)
[![Docs.rs](https://docs.rs/microdsp/badge.svg)](https://docs.rs/microdsp)

microdsp is a collection of [DSP](https://en.wikipedia.org/wiki/Digital_signal_processing)
algorithms and utilities written in Rust. The code is `no_std` compatible and suitable for use in embedded systems. Available algorithms include

* Monophonic [pitch](https://en.wikipedia.org/wiki/Pitch_%28music%29) detection using the [MPM algorithm](http://www.cs.otago.ac.nz/tartini/papers/A_Smarter_Way_to_Find_Pitch.pdf). Supports downsampling and overlapping windows.
* [Audio onset detection](https://en.wikipedia.org/wiki/Onset_(audio)) using [spectral flux novelty](https://krishnasubramani.web.illinois.edu/data/Energy-Weighted%20Multi-Band%20Novelty%20Functions%20for%20Onset%20Detection%20in%20Piano%20Music.pdf). Used to detect transients and "starts of sounds". Supports downsampling and overlapping windows.
* [Normalized least mean squares](https://en.wikipedia.org/wiki/Least_mean_squares_filter#Normalized_least_mean_squares_filter_(NLMS)) adaptive filter. Can for example be used for signal cancellation and time delay estimation.

To see microdsp running on a microcontroller, check out [these videos](https://github.com/stuffmatic/microdsp-zephyr-demos#demos).

## Installing

Add the following line to your Cargo.toml file:

```
microdsp = "0.1"
```

microdsp is `no_std` compatible and relies
on [`alloc`](https://doc.rust-lang.org/alloc/).
When building for targets without a default allocator,
one must be provided by the user. This can be accomplished in stable Rust 1.68 and higher
using `#[global_allocator]` and `#[default_alloc_error_handler]`.

## Usage

See [the crate documentation](https://docs.rs/microdsp).

## Demos

### Cargo examples

The [`examples`](examples) folder contains a number of demos that can be run with

```
cargo run --example [filename without .rs extension]
```

for example `cargo run --example mpm`. Some of these use `rust-portaudio` for real time audio input. If you run into portaudio related issues, you may find some pointers [here](https://github.com/RustAudio/rust-portaudio).

### Embedded

The [microdsp-zephyr-demos](https://github.com/stuffmatic/microdsp-zephyr-demos/) repo contains demos showing how to do real time audio processing on a microcontroller using microdsp and [Zephyr](https://zephyrproject.org/).

## License

This project is released under the MIT license.