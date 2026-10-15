//! The software compositor (E17, ADR-0020 point 3; E17 M1 T6):
//! indexed sprites drawn into one frame in the original's screen pixel format, with the keyed
//! colours (transparency, shadow, outline and flag, player colours) applied as the executable's
//! drawers apply them (`docs/epics/E17-design.md` §3). [`screen`]: the frame and its pixel
//! format; [`palette`]: palettes in that format, player colours; [`blit`]: the drawers;
//! [`png`]: the frame as PNG bytes. The window, and writing the PNG to a file, are adapters.
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

extern crate alloc;

pub mod blit;
pub mod palette;
pub mod png;
pub mod screen;
pub mod text;
pub mod tint;
