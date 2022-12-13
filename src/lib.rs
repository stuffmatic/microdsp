//! microdsp is a collection of [DSP](https://en.wikipedia.org/wiki/Digital_signal_processing)
//! algorithms and utilities. The crate is `no_std` compatible and suitable for embedded systems.
//! microdsp relies on [`alloc`](https://doc.rust-lang.org/alloc/).
//! When building for targets without a default allocator,
//! one must be provided by the user. This can be accomplished in stable Rust 1.XXX and higher
//! using `#[global_allocator]` and `#[default_alloc_error_handler]`.

#![no_std]
extern crate alloc;

pub mod common;
pub mod mpm;
pub mod nlms;
pub mod sfnov;
