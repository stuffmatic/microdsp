# microdsp

microdsp is a collection of [DSP](https://en.wikipedia.org/wiki/Digital_signal_processing)
algorithms and utilities written in Rust and suitable for use in embedded systems. See [the crate documentation]() for more info.

microdsp is `no_std` compatible and relies
on [`alloc`](https://doc.rust-lang.org/alloc/).
When building for targets without a default allocator,
one must be provided by the user. This can be accomplished in stable Rust 1.68 and higher
using `#[global_allocator]` and `#[default_alloc_error_handler]`.

## Demos

### Desktop

The `examples` folder contains a number of demos that can be run with `cargo run --example [filename without .rs extension]`, for example `cargo run --example mpm`. Some of these use `rust-portaudio` , so if you run into portaudio related issues, check out the  

### Web

microdsp can be compiled to wasm to run in modern web browsers.

### Embedded

A demo showing how to do real time audio processing using microdsp and [Zephyr](https://zephyrproject.org/) can be found in the [microdsp-zephyr-demos](https://github.com/stuffmatic/microdsp-zephyr-demos) repo.
