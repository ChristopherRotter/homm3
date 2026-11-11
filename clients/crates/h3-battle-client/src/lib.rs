//! The battle client (E17; ADR-0020 point 2, ADR-0024): the adapters around the pure scene model
//! (`h3_view`) and the `h3-view` command line's commands. Until E17 M7 T3a1 these modules were
//! `h3-view`'s binary; they became this library so that another host (the adventure map's viewer,
//! E24 M4 T5) can run a battle in its own window, the binary `h3-view` (`src/bin/h3-view/`) now
//! one host of them. Nothing in them changed with the move.
//!
//! - **The window's battle:** [`window`] (`h3-view window`: the loop, the keys, the mouse, the
//!   frames), [`battle`] (a battle ready to draw at any time, grown as it is played), [`live`] (a
//!   battle fought here), [`online`], [`net`], [`connect`] and [`account`] (a seat on a server),
//!   [`pointer`], [`spellbook`], [`boxes`], [`results`], [`stats`] (what the mouse, the spell
//!   book, the message boxes, the result window and the stats dialog do), [`audio`] and
//!   [`sounds`] (the sound), [`motion`] and [`aims`] (the stage's numbers, a cast's aim),
//!   [`videos`] (the result window's clips), [`log`] (the battle log's lines), [`bot`] (the fixed
//!   bot).
//! - **The other commands' adapters:** [`gate`] (the real-asset gate), [`render`] (the headless
//!   renderer), [`sequences`] (the harness's sequences), [`scenario`] (a scenario file and its
//!   packs), [`sprite`], [`text`], [`video`], [`assets`] (the asset tables and layouts read).
//!
//! An adapter, not pure: it reads files and the archives, measures time, prints, runs threads and
//! drives the window and the audio device through `h3-sdl`. Left out of the coverage gate as the
//! binaries are (`clients/scripts/check.sh`); `h3-view window --check`, `drawn` and the harness
//! exercise it, beside its own tests.
#![forbid(unsafe_code)]

pub mod account;
pub mod aims;
pub mod assets;
pub mod audio;
pub mod battle;
pub mod bot;
pub mod boxes;
#[cfg(test)]
mod clicks;
pub mod connect;
pub mod gate;
pub mod live;
pub mod log;
pub mod motion;
pub mod net;
pub mod online;
pub mod pointer;
pub mod render;
pub mod results;
pub mod scenario;
pub mod sequences;
pub mod sounds;
pub mod spellbook;
pub mod sprite;
pub mod stats;
pub mod text;
pub mod video;
pub mod videos;
pub mod window;
