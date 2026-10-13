//! The viewer's scene model (E17, ADR-0020 points 2 and 8): `h3-protocol` messages and a time in,
//! what is on screen at that time out, as a draw list for `h3-render`. [`record`] reads a record,
//! [`scene`] makes a snapshot's draw list, [`batch`] holds a checkpoint's decoded assets,
//! [`compose`] draws the list into a frame; [`tables`] and [`layout`] are the asset tables and
//! the screen layouts. The adapters (the real-asset gate, the headless renderer, later the window)
//! are the binary's modules, not this library.
//!
//! Pure: no I/O, clock, threads or globals, `no_std` + `alloc`; time is an input. The header below
//! is checked by `tests/purity.rs`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;

pub mod batch;
pub mod compose;
pub mod layout;
pub mod record;
pub mod scene;
pub mod tables;
