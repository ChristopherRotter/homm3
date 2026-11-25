//! The adventure map client (E24; ADR-0020 point 2, ADR-0024): the adapters around the map
//! viewer's pure library (`h3_map_view`) and the `h3-map-view` command line's commands. Until E24
//! M5 T2a these modules were `h3-map-view`'s binary; they became this library so that another host
//! (the game shell, E24 M5: the user's decision at M5 T0, "the shell on top") can run the map in
//! its own window, the binary `h3-map-view` (`src/bin/h3-map-view/`) now one host of them. Nothing
//! in them changed with the move.
//!
//! - **The commands:** [`gate`] (the map's real-asset gate), [`render`] (a still of the adventure
//!   screen), [`sequences`] (the recorded sequences replayed), [`records`] (a whole game's record
//!   played headless), [`situation`] (a situation's map written), [`window`] (the map in a window:
//!   the loop, the keys, the mouse, the frames).
//! - **Their parts:** `paint` (a map set up to play and its drawing), `audio` and `sounds` (the
//!   map's sound and its files), `battles` (a record's battles hosted through `h3-battle-client`),
//!   `texts` (the texts the panels show), `files` (the files the commands read).
//!
//! An adapter, not pure: it reads files and the archives, measures time, prints, runs threads and
//! drives the window and the audio device through `h3-sdl`. Left out of the coverage gate as the
//! binaries are (`clients/scripts/check.sh`); `h3-map-view window --check`, `drawn` and the
//! harness exercise it, beside its own tests.

mod audio;
mod battles;
mod files;
pub mod gate;
mod paint;
pub mod records;
pub mod render;
pub mod sequences;
pub mod situation;
mod sounds;
mod texts;
pub mod window;
