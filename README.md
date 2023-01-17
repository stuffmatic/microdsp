# microdsp

[![Crates.io](https://img.shields.io/crates/v/microdsp)](https://crates.io/crates/microdsp)
[![Docs.rs](https://docs.rs/microdsp/badge.svg)](https://docs.rs/microdsp)

microdsp is a collection of [DSP](https://en.wikipedia.org/wiki/Digital_signal_processing)
algorithms and utilities written in Rust. The code is `no_std` compatible and suitable for use in embedded systems.

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

The [microdsp-zephyr-demos](https://github.com/stuffmatic/microdsp-zephyr-demos/) repo contains demos showing how to do real time audio processing on a microcontroller using microdsp and Zephyr.

## License

This project is released under the MIT license.