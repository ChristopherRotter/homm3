//! The viewer's scene model (E17, ADR-0020 points 2 and 8): `h3-protocol` messages and a time in,
//! what is on screen at that time out, as a draw list for `h3-render`. [`record`] reads a record,
//! [`anim`] plays its commands at the original's timing, [`scene`] makes the draw list of a
//! snapshot at a time ([`hero`]: the heroes and their flags), [`batch`] holds a checkpoint's decoded assets,
//! [`compose`] draws the list into a frame; [`tables`] and [`layout`] are the asset tables and
//! the screen layouts, made of the dialog items of the common crate (`h3_ui::ui`), [`panel`] the
//! battle's panel of them and [`message`] the message box laid out as them; [`keys`] is what the
//! battle window's keys do (through the common table, `h3_ui::keys`); [`mouse`] is what the
//! mouse shows over the battle (the cursor's kind, the status line's hint, the stack it marks);
//! [`sound`] is what a battle's opening plays; [`battle_log`] the battle log's lines composed from
//! the string catalogue (`h3_ui::text`). What both viewers share is the common crate `h3-ui`'s
//! (E24 M3 T1a: the clock in the window, the game's mouse cursor, the dialog items, the string
//! catalogue, the key table, the music). The adapters (the real-asset gate, the headless renderer,
//! the window through `h3-sdl`) are the binary's modules, not this library.
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
pub mod battle_log;
pub mod book;
pub mod compose;
pub mod hero;
pub mod keys;
pub mod layout;
pub mod message;
pub mod mouse;
pub mod panel;
pub mod ray;
pub mod record;
pub mod scene;
pub mod siege;
pub mod sound;
pub mod tables;
