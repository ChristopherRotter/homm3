//! The adventure map's viewer (E24; ADR-0020): what the view of a map shows, drawn as the
//! original draws it. Two inputs (`docs/epics/E24-map-client.md`, "How the viewer sees a map"):
//! the **static art** of a map, read from the raw `.h3m` ([`art`]: every tile's pictures, every
//! template's DEF and masks, every object's anchor), and its **live part** ([`live`]: which
//! objects stand, what a random object became, the owners, the heroes and boats, the tiles the
//! viewer has explored, the moment). [`tables`] are the map's asset tables
//! (`clients/map/assets/`), [`layout`] the adventure screen's layout (`clients/map/layouts/`),
//! [`check`] what the real-asset gate checks of the archives' entries they name. [`objects`] are
//! the tiles' draw lists of objects, from the templates' `.msk` shapes. [`scene`] is
//! what the screen shows at a view and a moment, as a draw list in the original's order, the
//! adventure screen's panels in it as [`panels`] gives them (E24 M3 T2a);
//! [`batch`] the assets it needs, decoded; [`compose`] draws it into a frame in the original's
//! screen format with the view's drawers ([`draw`]) and the compositor's (`h3-render`), a frame
//! kept drawing only what changed, [`damage`] what changed between frames (E24 M3 T2b2).
//! [`situation`] is a still of the capture set: a map built with the writer, its view and the
//! tiles left unexplored. [`timeline`] is the map over time (E24 M2 T2): the original's loop on a
//! clock of milliseconds, its redraws, palette steps, scroll and cursor; [`sequence`] the recorded
//! sequences it replays. For the window (E24 M2 T4a): [`replay`] runs the timeline forward as its
//! clock (`h3_ui::clock`) needs, [`keys`] are the window's commands (the table of what each key
//! does: `h3_ui::keys`); [`sound`] the map's sound as cues (M2 T5), its music the original's music
//! thread (`h3_ui::music`, M2 T6a2). What both viewers share is the common crate `h3-ui`'s (E24
//! M3 T1a: the clock, the cursor, the dialog items, the string catalogue, the key table, the
//! music). The adapters (the gate,
//! `render`, `situation`, `sequences`, `window`) are the binary's modules, not this library.
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

pub mod art;
pub mod batch;
pub mod check;
pub mod compose;
pub mod damage;
pub mod draw;
pub mod keys;
pub mod layout;
pub mod live;
pub mod objects;
pub mod panels;
pub mod replay;
pub mod scene;
pub mod sequence;
pub mod situation;
pub mod sound;
pub mod tables;
pub mod timeline;
