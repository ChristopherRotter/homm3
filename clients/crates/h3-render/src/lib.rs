//! The software compositor (E17, ADR-0020 point 3):
//! indexed sprites drawn into one frame in the original's screen pixel format, with the keyed
//! colours (transparency, shadow, selection outline, player colours) applied as the executable
//! applies them. The window and the PNG writer show that same frame; they are adapters, not here.
//!
//! Pure: no I/O, clock, threads or globals, `no_std` + `alloc`; the frame size and every layout are
//! data. The header below is checked by `h3-view`'s `tests/purity.rs`.
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

pub mod blit;
pub mod frame;
