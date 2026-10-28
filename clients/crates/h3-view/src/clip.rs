//! The result window's clip (E17 M5 T4b2b; `docs/epics/E17-design.md` §7.21): which Bink clip the
//! window plays in its slot ([`of`]), which of its frames is on the screen a time after the window
//! opened ([`shown`]) and its pixels as the game's `BINKW32.DLL` (RAD's Bink 0.5b) puts them on the
//! 16-bit screen ([`rgb565`], [`Video`]). The decoding is `h3-bink`'s; the adapters read the clips
//! from the video archives.
//!
//! **The clip.** `battle_result_dialog` 0x4702D0 picks the clip's index by the branches that pick
//! its tune ([`crate::sound::end_tune`]) and plays it at the screen's (272, 89)
//! (`video_play_in_dialog` 0x597570 → `video_open_in_dialog` 0x44D830): an entry of the clip table
//! 0x6839C0, a start and, for four of the six, a loop that takes over from it.
//!
//! **The timing**, measured on `BINKW32.DLL` itself (`clients/capture/binkprobe.c`, the DLL on the
//! clips under Wine, its clock ours) and confirmed by the original's captures: the DLL lets a clip
//! decode its p-th frame (`BinkDoFrame`, p counted from 0 for the first) once its clock reaches the
//! first one's time + ⌊1000 p / rate⌋ ms (`BinkWait`). Showing the window decodes the start's first
//! frame twice before the game's first tick (`dialog_show` 0x5FEAE0 at 0x5FEBD4 and the window's
//! own `battle_result_dialog_show` 0x471AF0 at 0x471B17, each through `video_redraw` 0x44D9E0,
//! which decodes while the clip is at its first frame); the tick (`video_tick` 0x44DAA0) decodes
//! it a third time at p = 2. So frame 0 is on the screen at once and frame k ≥ 1 from
//! ⌊1000 (k + 2) / rate⌋ ms. At the start's last frame the tick closes the start and shows the
//! loop's rectangles, none before the loop's first frame (`video_show` 0x5979D0), then decodes the
//! loop's first frame at once: the start's last frame never reaches the screen, the loop's frame j
//! is there from that time + ⌊1000 j / rate⌋, the loop circling on. A start without a loop circles
//! itself the same way, its last frame shown. Not modelled: a redraw of the window while a clip is
//! at its first frame decodes it once more and delays the rest by a frame (none happens in the
//! window's own course; a press over the window may). The client fixes both slips (CLT-Q31, the
//! user, 2026-10-28): frame 0 is on the screen for one period as every other frame, the start's
//! last frame for its period before the loop's first; the comparisons with the original's
//! captures play its timing (`h3_render::Bugs::Original`).
//!
//! **The colours** (`BinkCopyToBuffer` into the 5-6-5 screen, surface type 3; found by running the
//! DLL on every combination of Y, U and V and read in its colour tables, with the user's leave of
//! 2026-10-28): each pixel takes its 2 × 2 block's chroma as it is; with L = 0 for Y ≤ 16, else
//! (Y − 16) · 38 154 / 32 768, each division rounded toward zero, the top five bits are
//! clamp(L + 66 101 (V − 128) / 32 768) / 8, the middle six
//! clamp(L − 26 639 (U − 128) / 32 768 − 12 837 (V − 128) / 32 768) / 4 and the low five
//! clamp(L + 52 299 (U − 128) / 32 768) / 8, clamp to 0..255 and U, V the decoder's planes 1 and 2
//! (FFmpeg's names). The DLL's tables end at sums below −256 or above 511, where it reads its
//! neighbouring tables; the clips' sums lie within (their Y 13 to 234, U 24 to 219, V 87 to 205),
//! and here every sum is clamped.

use alloc::vec::Vec;

use h3_bink::{Clip as BinkClip, Decoder, Error, Frame};
use h3_render::Bugs;

use crate::sound::Tune;

/// One entry of the clip table (0x6839C0, 20 bytes each: the start's name, the loop's, flags):
/// the files the window plays, each name with the extension the lookup adds (`.bik`, 0x44D5A0),
/// in the executable's spelling (the archives' names match it ignoring case).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clip {
    /// The clip played first.
    pub start: &'static str,
    /// The clip that takes over at the start's last frame and circles, if any (the table's empty
    /// name: none).
    pub looped: Option<&'static str>,
}

/// The clip table's first six entries (0x6839C0): the result window's, by its index.
pub const CLIPS: [Clip; 6] = [
    Clip {
        start: "Win3.bik",
        looped: None,
    },
    Clip {
        start: "LoseCstl.bik",
        looped: Some("LoseCslp.bik"),
    },
    Clip {
        start: "rtstart.bik",
        looped: Some("rtloop.bik"),
    },
    Clip {
        start: "Surrender.bik",
        looped: None,
    },
    Clip {
        start: "Defendall.bik",
        looped: Some("defendloop.bik"),
    },
    Clip {
        start: "lbstart.bik",
        looped: Some("lbloop.bik"),
    },
];

/// The window's clip, picked by the branches that pick its tune (0x470B68 to 0x470CFF): won, 0
/// (`Win3`), or 4 (`Defendall`) for a town's defender; lost, 1 (`LoseCstl`) for a town's
/// defender, 3 (`Surrender`) or 2 (`rtstart`) when a side gave up so, else 5 (`lbstart`).
#[must_use]
pub const fn of(tune: Tune) -> &'static Clip {
    let index = match tune {
        Tune::Win => 0,
        Tune::LoseCastle => 1,
        Tune::Retreat => 2,
        Tune::Surrender => 3,
        Tune::DefendCastle => 4,
        Tune::Lose => 5,
    };
    &CLIPS[index]
}

/// How often the window decodes the start's first frame before its first tick (`dialog_show`'s
/// and the window's own redraw).
const REDRAWS: u64 = 2;

/// A clip's timing: how many frames and how many a second (`[num, den]`: num / den).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// Its frames.
    pub frames: usize,
    /// Its frame rate as the file gives it.
    pub rate: [u32; 2],
}

impl Timing {
    /// The last p whose time ⌊1000 p den / num⌋ has come `since` ms after the clip's first frame
    /// (none for a rate of 0).
    fn due(self, since: u32) -> u64 {
        let [num, den] = self.rate.map(u64::from);
        if num == 0 || den == 0 {
            return 0;
        }
        ((u64::from(since) + 1) * num - 1) / (1000 * den)
    }

    /// The time of its p-th decoded frame after its first.
    fn at(self, p: u64) -> u64 {
        let [num, den] = self.rate.map(u64::from);
        (1000 * p * den).checked_div(num).unwrap_or(u64::MAX)
    }
}

/// How a profile plays a start of `frames` frames (CLT-Q31): its decodes before the first tick,
/// and the decode at which the loop takes over.
const fn schedule(bugs: Bugs, frames: u64) -> (u64, u64) {
    match bugs {
        // The two redraws; the hand-over at the start's last decode, which never shows.
        Bugs::Original => (REDRAWS, frames + REDRAWS - 1),
        // Frame 0 its one period as every other, the start's last frame shown its period too.
        Bugs::Fixed => (0, frames),
    }
}

/// Which clip of the window's a frame is from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// The start.
    Start,
    /// The loop.
    Loop,
}

/// The frame on the screen `since` ms after the window opened, of a start with `start`'s timing
/// and its loop with `looped`'s (module doc), in the profile `bugs` (CLT-Q31): which clip, which
/// frame. None for a start without frames, or a loop without frames once the start hands over.
#[must_use]
pub fn shown(
    since: u32,
    start: Timing,
    looped: Option<Timing>,
    bugs: Bugs,
) -> Option<(Part, usize)> {
    if start.frames == 0 {
        return None;
    }
    let p = start.due(since);
    let n = u64::try_from(start.frames).unwrap_or(u64::MAX);
    let (redraws, switch) = schedule(bugs, n);
    match looped {
        Some(looped) if p >= switch => {
            let handed = start.at(switch);
            let since = u32::try_from(u64::from(since).saturating_sub(handed)).unwrap_or(u32::MAX);
            let j = usize::try_from(looped.due(since)).unwrap_or(usize::MAX);
            j.checked_rem(looped.frames).map(|j| (Part::Loop, j))
        }
        _ => {
            let k = usize::try_from(p.saturating_sub(redraws)).unwrap_or(usize::MAX);
            Some((Part::Start, k % start.frames))
        }
    }
}

/// When the frame on the screen `since` ms after the window opened was decoded, and when the next
/// decode comes (none: never), both in ms after the opening, in the profile `bugs`: the times a
/// viewer draws the clip at. A decode may give the frame already shown (the original's start's
/// first, at its tick).
#[must_use]
pub fn decodes(
    since: u32,
    start: Timing,
    looped: Option<Timing>,
    bugs: Bugs,
) -> (u32, Option<u32>) {
    let time = |t: u64| u32::try_from(t).ok();
    let p = start.due(since);
    let n = u64::try_from(start.frames).unwrap_or(u64::MAX);
    let (redraws, switch) = schedule(bugs, n);
    let (began, next) = match looped {
        Some(looped) if p >= switch => {
            let handed = start.at(switch);
            let after = u32::try_from(u64::from(since).saturating_sub(handed)).unwrap_or(u32::MAX);
            let j = looped.due(after);
            (
                handed.saturating_add(looped.at(j)),
                handed.saturating_add(looped.at(j + 1)),
            )
        }
        // The original's two redraws decode at the opening itself, whatever the DLL's schedule
        // says.
        _ => {
            let p = p.max(redraws.saturating_sub(1));
            let began = if p < redraws { 0 } else { start.at(p) };
            (began, start.at(p + 1))
        }
    };
    (time(began).unwrap_or(since), time(next))
}

/// A clip decoded for the screen: its frames' pixels in the screen's 5-6-5, row by row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Video {
    /// The frame's width.
    pub width: usize,
    /// The frame's height.
    pub height: usize,
    /// Its frame rate (`[num, den]`).
    pub rate: [u32; 2],
    /// Each frame's pixels, `width × height`.
    pub frames: Vec<Vec<u16>>,
}

impl Video {
    /// Decodes every frame of the Bink file `bytes` and converts it as the DLL does.
    ///
    /// # Errors
    /// What `h3-bink` rejects: the file, or a frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        let clip = BinkClip::parse(bytes)?;
        let mut decoder = Decoder::new(&clip)?;
        let frames = (0..clip.frames().len())
            .map(|n| Ok(rgb565(decoder.decode(clip.video(n)?)?)))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            width: usize::try_from(clip.width).unwrap_or(0),
            height: usize::try_from(clip.height).unwrap_or(0),
            rate: [clip.rate_num, clip.rate_den],
            frames,
        })
    }

    /// Its timing.
    #[must_use]
    pub fn timing(&self) -> Timing {
        Timing {
            frames: self.frames.len(),
            rate: self.rate,
        }
    }
}

/// The luma's part (`L`, module doc).
fn luma(y: u8) -> i32 {
    if y <= 16 {
        0
    } else {
        (i32::from(y) - 16) * 38_154 / 32_768
    }
}

/// A sum clamped to a byte, its top `bits` bits.
fn level(sum: i32, bits: u32) -> u16 {
    u16::try_from(sum.clamp(0, 255)).unwrap_or(0) >> (8 - bits)
}

/// One pixel from its Y and its block's planes 1 and 2 (module doc).
fn pixel(y: u8, u: u8, v: u8) -> u16 {
    let l = luma(y);
    let (u, v) = (i32::from(u) - 128, i32::from(v) - 128);
    let top = level(l + 66_101 * v / 32_768, 5);
    let middle = level(l - 26_639 * u / 32_768 - 12_837 * v / 32_768, 6);
    let low = level(l + 52_299 * u / 32_768, 5);
    (top << 11) | (middle << 5) | low
}

/// A decoded frame's picture as `BINKW32.DLL` copies it to the 5-6-5 screen (module doc).
#[must_use]
pub fn rgb565(frame: &Frame) -> Vec<u16> {
    let [luma_plane, first, second] = frame.planes();
    let (width, height) = (luma_plane.width(), luma_plane.height());
    let mut out = Vec::with_capacity(width * height);
    for y in 0..height {
        let (u_row, v_row) = (first.row(y / 2), second.row(y / 2));
        for (x, &l) in luma_plane.row(y).iter().enumerate() {
            let u = u_row.get(x / 2).copied().unwrap_or(128);
            let v = v_row.get(x / 2).copied().unwrap_or(128);
            out.push(pixel(l, u, v));
        }
    }
    out
}

#[cfg(test)]
mod tests;
