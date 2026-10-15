//! The viewer's scene model (E17, ADR-0020 points 2 and 8): `h3-protocol` messages and a time in,
//! what is on screen at that time out, as a draw list for `h3-render`. [`record`] reads a record,
//! [`anim`] plays its commands at the original's timing, [`scene`] makes the draw list of a
//! snapshot at a time, [`batch`] holds a checkpoint's decoded assets,
//! [`compose`] draws the list into a frame; [`tables`] and [`layout`] are the asset tables and
//! the screen layouts; [`cursor`] is the game's mouse cursor and [`clock`] the battle's clock in
//! the window; [`sound`] is what a battle's opening plays. The adapters (the real-asset gate, the headless renderer, the window through
//! `h3-sdl`) are the binary's modules, not this library.
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

pub mod anim;
pub mod batch;
pub mod clock;
pub mod compose;
pub mod cursor;
pub mod layout;
pub mod ray;
pub mod record;
pub mod scene;
pub mod sound;
pub mod tables;
