//! A Smacker video decoder for the original game's clips (E27 M2 T1; decision 5, CLT-Q80): the
//! Smacker file (`.smk`, [`Clip`], its frames read in order by a [`Reader`]) and its video
//! ([`Decoder`]): bytes in, 8-bit indexed frames and their palettes out. Colours, timing,
//! drawing and the sound are the viewer's (the audio tracks' packets are skipped: E27 M2 T2).
//!
//! **Origin.** Ported from FFmpeg 6.1.1 (<https://ffmpeg.org>): the demuxer's header, frame table
//! and frames (`libavformat/smacker.c`, Copyright (c) 2006 Konstantin Shishkov; its
//! `smacker_read_header` and `smacker_read_packet`, the palette's levels `smk_pal`) and the video
//! decoder (`libavcodec/smacker.c`, Copyright (c) 2006 Konstantin Shishkov; its trees'
//! `smacker_decode_tree`, `smacker_decode_bigtree`, `smacker_decode_header_tree`,
//! `decode_header_trees`, `last_reset`, `smk_get_code`, and `decode_init` and `decode_frame`).
//! FFmpeg is free software under the GNU Lesser General Public License version 2.1 or later;
//! this port is distributed under the GNU General Public License version 2 or later, as the
//! LGPL 2.1's section 3 allows and as the rest of the project is (ADR-0009; as `h3-bink`,
//! CLT-Q29).
//!
//! **What it decodes as FFmpeg does:** every file FFmpeg's demuxer accepts, of either version
//! (`SMK2`, the game's, and `SMK4`), with a frame of at most [`MAX_WIDTH`] × [`MAX_HEIGHT`]
//! (FFmpeg takes larger ones, refused here with [`Error::Size`]), and FFmpeg's handling of
//! malformed data: bits past a frame's end read as 0 where FFmpeg's unchecked reader reads its
//! zeroed padding, and fail where it checks; a frame that fails is left partly decoded, the next
//! decoding over it; a palette change applied up to the entry that fails; bytes past the file's
//! end read as 0 by the palette's and the audio's reads, and a frame's video past it fails. The
//! demuxer's frames are given one by one, a failed one as an error and the next read on, as
//! FFmpeg's library gives them (the `ffmpeg` command line stops at the first error a demuxer
//! returns), and without FFmpeg's end of the stream at a read past the file's end (`avio_feof`):
//! the frames after it fail here instead. The unit tests hold these cases.
//!
//! Pure: no I/O, clock, threads or globals, `no_std` + `alloc`, no floating point; never panics
//! on any input (the property tests). Fixed-size tables (the palette, the runs, the levels) are
//! indexed directly where the index is bounded by construction (each noted); the trees and the
//! pixels, whose sizes come from the file, only through checked reads and writes. The header
//! below is checked by `h3-view`'s `tests/purity.rs`.
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

mod bits;
pub mod build;
pub mod check;
mod clip;
pub mod sound;
mod tree;
mod video;

pub use clip::{Clip, Packet, Reader, Track, Version};
pub use sound::{Codec, Samples};
pub use video::{Decoder, Frame};

/// The largest frame the decoder takes: the game's screen (*provisional*; `HSANIM`, `HSLOOP`,
/// `LOSEGAME` and `NWCLOGO` fill it). FFmpeg's own limits are far larger; a bound keeps an
/// untrusted header from allocating much.
pub const MAX_WIDTH: u32 = 800;
/// See [`MAX_WIDTH`].
pub const MAX_HEIGHT: u32 = 600;

/// Why a file or a frame was rejected. Nothing panics: every malformed input ends in one of
/// these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The bytes end before the header, the frame table or the trees end, or before a frame's
    /// video does.
    Truncated,
    /// The signature is neither `SMK2` nor `SMK4` (a Bink file, anything else).
    Magic,
    /// A header field holds a value FFmpeg's demuxer rejects; names the field.
    Field(&'static str),
    /// The frame is empty or larger than [`MAX_WIDTH`] × [`MAX_HEIGHT`].
    Size,
    /// The trees or a frame's data break one of FFmpeg's checks; names it.
    Invalid(&'static str),
    /// A part of the format FFmpeg decodes that is not ported; names it.
    Unsupported(&'static str),
}
