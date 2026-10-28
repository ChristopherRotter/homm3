//! What both viewers share (E24 M3 T1a; the user's decision at E24 M3 T0, 2026-10-28): the battle
//! viewer's (`h3-view`, the client lane's) and the adventure map's (`h3-map-view`, the map client
//! lane's), common to both lanes as the client foundation is. [`clock`] is a viewer's clock in the
//! window, [`cursor`] the game's mouse cursor with [`sprite`], the decoded DEF it draws; [`ui`] the
//! dialog items a screen is made of and their draw items; [`text`] the string catalogue and its
//! language packs; [`keys`] the table of what each key does, generic over a viewer's commands;
//! [`music`] the original's music thread for a window's music stream. What is one viewer's alone
//! (the battle's hexes and layouts, the map's tiles) stays in that viewer.
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

pub mod clock;
pub mod cursor;
pub mod keys;
pub mod music;
pub mod sprite;
pub mod text;
pub mod ui;
