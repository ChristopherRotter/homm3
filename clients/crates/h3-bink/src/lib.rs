//! A Bink video decoder for the original game's clips (E17 M5 T4b2a; CLT-Q29): the Bink file
//! (`.bik`, [`Clip`]) and its video of revision `b`, the one the game's battle clips use
//! ([`Decoder`]): bytes in, YUV 4:2:0 frames out. Colours, timing and drawing are the viewer's.
//!
//! **Origin.** Ported from FFmpeg 6.1.1 (<https://ffmpeg.org>): the demuxer's header and frame
//! index (`libavformat/bink.c`, Copyright (c) 2008-2010 Peter Ross, (c) 2009 Daniel Verkamp), the
//! video decoder's revision-`b` path (`libavcodec/bink.c`, Copyright (c) 2009 Konstantin
//! Shishkov, (C) 2011 Peter Ross; its `binkb_*` functions, `read_dct_coeffs`, `read_residue` and
//! `decode_frame`), its transforms (`libavcodec/binkdsp.c`, Copyright (c) 2009 Konstantin
//! Shishkov) and its tables (`libavcodec/binkdata.h`, Copyright (C) 2009 Konstantin Shishkov).
//! FFmpeg is free software under the GNU Lesser General Public License version 2.1 or later;
//! this port is distributed under the GNU General Public License version 2 or later, as the
//! LGPL 2.1's section 3 allows and as the rest of the project is (ADR-0009; the user's choice,
//! CLT-Q29).
//!
//! **What it decodes as FFmpeg does:** every file FFmpeg's demuxer accepts with a video of
//! revision `b` without an alpha plane, frame by frame into the same planes, including FFmpeg's
//! handling of malformed data (bits past a packet's end read as 0; a motion vector out of the
//! plane leaves the block as it was; a bundle's values once used up are read again as they were
//! left). Checked against FFmpeg's decoder on the game's ten battle clips and on synthetic clips
//! of every block type (`clients/scripts/bink-check.sh`; [`build::sample`]). Two differences
//! lie in FFmpeg's buffers, not its decoding: (1) FFmpeg pads a plane's rows to its SIMD
//! alignment (64 bytes in the build checked: a 64-pixel-wide clip's luma rows take 128), so
//! where its rows are longer than the block grid's, a motion vector crossing a row's end reads
//! other pixels than here; the game's clips are 256 pixels wide, whose rows it does not pad.
//! (2) Revision `b` decodes in place over the frame before; while the caller still holds that
//! frame (the `ffmpeg` command line does), FFmpeg decodes into a new buffer with only the
//! picture's rows copied (`ff_reget_buffer`), so the block grid's rows below the picture (the
//! chroma's rows 60 to 63 of a 120-row clip) hold another frame's pixels, which motion vectors
//! copy into the picture. Here they keep the frame before's, as in FFmpeg when each frame is
//! released before the next is decoded. Not ported: the other revisions (`f` to `k`,
//! Huffman-coded), the alpha plane, the audio (its packets are skipped), Bink 2 and Smacker.
//!
//! Pure: no I/O, clock, threads or globals, `no_std` + `alloc`; never panics on any input (the
//! property tests). Fixed-size tables and blocks are indexed directly where the index is bounded
//! by construction (a mask or the coefficient lists' structure, each noted); the planes and the
//! bundles, whose sizes come from the file, only through checked reads and writes
//! (*provisional*: `h3-assets` denies `clippy::indexing_slicing` outright). The header below is
//! checked by `h3-view`'s `tests/purity.rs`.
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
mod dsp;
mod tables;
mod video;

pub use clip::{Clip, FrameEntry};
pub use video::{Decoder, FLAG_ALPHA, Frame, Plane};

/// The largest frame the decoder takes: the game's screen (*provisional*; the battle clips are
/// 256 × 120). FFmpeg's own limits are far larger; a bound keeps an untrusted header from
/// allocating much.
pub const MAX_WIDTH: u32 = 800;
/// See [`MAX_WIDTH`].
pub const MAX_HEIGHT: u32 = 600;

/// Why a file or a frame was rejected. Nothing panics: every malformed input ends in one of
/// these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The bytes end before the header, the frame index or a frame's data end.
    Truncated,
    /// The signature is not `BIK` (a Smacker file, Bink 2, anything else).
    Magic,
    /// A header field holds a value FFmpeg's demuxer rejects; names the field.
    Field(&'static str),
    /// The video's revision is not `b` (the signature's fourth byte).
    Revision(u8),
    /// The video has an alpha plane, which revision `b` is not decoded with here.
    Alpha,
    /// The frame is empty or larger than [`MAX_WIDTH`] × [`MAX_HEIGHT`].
    Size,
    /// A frame's data break one of FFmpeg's decoder's checks; names it. The frame is left
    /// partly decoded, as FFmpeg leaves it, and the next one decodes over it.
    Invalid(&'static str),
}
