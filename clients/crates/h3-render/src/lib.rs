//! The software compositor (E17, ADR-0020 point 3; E17 M1 T6):
//! indexed sprites drawn into one frame in the original's screen pixel format, with the keyed
//! colours (transparency, shadow, outline and flag, player colours) applied as the executable's
//! drawers apply them (`docs/epics/E17-design.md` §3). [`screen`]: the frame and its pixel
//! format; [`palette`]: palettes in that format, player colours; [`blit`]: the drawers;
//! [`text`]: text in the bitmap fonts; [`tint`]: a rectangle's hue and saturation (E17 M3 T4a);
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

/// Whether the original's drawing bugs are reproduced (the user, 2026-10-15): the client draws
/// them fixed; only the comparisons with the original's captures reproduce them, so that those
/// stay exact. Neither a player's option nor a quirk profile (`docs/epics/E17-design.md` §3.6;
/// the first: [`text`]'s long word).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Bugs {
    /// Fixed: what the client draws.
    #[default]
    Fixed,
    /// As the original's code has them: for the comparisons with its captures.
    Original,
}

pub mod blit;
pub mod palette;
pub mod png;
pub mod recolour;
pub mod screen;
pub mod text;
pub mod tint;
