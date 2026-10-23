//! # zirvstdlib
//!
//! The Zirvium standard library.  It is a `no_std` library written from
//! scratch for 64-bit systems.  It provides the small, dependency-free set of
//! primitives that the Zirvium kernel, drivers, compositor and (eventually)
//! userspace share:
//!
//! * memory-safe spin locks and one-time initialisation
//! * geometric primitives used by DisplayJet
//! * colour and alpha blending
//! * bitmaps and ring buffers
//! * hashing, random numbers and time conversion
//! * the Zirvium syscall ABI
//!
//! Collection types are backed by Rust's `alloc` crate.  `alloc` is Rust
//! source, not a C library, and the kernel supplies the allocator.  Keeping
//! the collection implementations in `alloc` lets us focus on operating
//! system functionality while remaining entirely Rust.

#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

pub mod abi;
pub mod bits;
pub mod collections;
pub mod color;
pub mod error;
pub mod fmt;
pub mod geometry;
pub mod hash;
pub mod io;
pub mod log;
pub mod math;
pub mod mem;
pub mod path;
pub mod prelude;
pub mod rand;
pub mod ring;
pub mod sync;
pub mod sys;
pub mod time;

pub use alloc::format;
pub use alloc::vec;
pub use alloc::{boxed::Box, string::String, vec::Vec};
pub use error::{Error, Result};

/// The version of this standard library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A short, human readable description of the platform ABI.
pub const TARGET: &str = "x86_64-unknown-none";
