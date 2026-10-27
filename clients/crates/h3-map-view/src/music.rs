//! The terrain music as the original's music thread plays it (E24 M2 T6a2, *provisional*;
//! `docs/epics/E24-design.md` §3.11): a model of `play_music` 0x59ACB0, `music_thread` 0x59A840
//! and `music_stop_fade` 0x59B080 on a clock of the frames heard, which the window's music
//! stream follows.
//!
//! - **A request** (`play_music`) is dropped when its name is the one pending (0x59AD0D) or the
//!   one the thread opened last while its stream plays (0x59AD43 to 0x59ADA6: a fade-out's track
//!   among them); else it becomes the pending name and starts a thread.
//! - **A thread** (`music_thread`) holds the manager's lock (+0xA8) from its start through the
//!   new track's fade-in, so a thread started meanwhile waits for it. It fades the track playing
//!   out (`music_stop_fade`: its place kept first, 0x59B146 to 0x59B1A8; then 9 steps of 100 ms
//!   from its volume, the whole after a fade-in, then paused), then opens the pending name *as it
//!   is then* (0x59A920, the last asked for) where that name's last fade began, and fades it in
//!   (9 steps of 100 ms, then the whole; E17 §8.2).
//!
//! [`Music`] is that state; [`Queue`] runs it twice for a stream queued ahead: once at the
//! frames heard (where a request lands) and once at the frames queued (what to queue next),
//! re-queuing from the heard point when a request is taken. Only looping tracks (the map's), at
//! the music's whole volume (a setting of 10); a track is known by its length in frames once
//! decoded ([`Track`]): the original opens its file at once, the window waits for its decoding.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// The music's whole volume on Miles' scale (127, a setting of 10).
pub const FULL: u8 = 127;
/// The fade-in's volumes, one a step (`music_thread` 0x59AB5D: a tenth of 127 added as an f32,
/// truncated), the whole after the last.
pub const FADE_IN: [u8; 9] = [12, 25, 38, 50, 63, 76, 88, 101, 114];
/// The fade-out's volumes from the whole, one a step (`music_stop_fade` 0x59B21B: a tenth taken
/// away as an f32, truncated), paused after the last.
pub const FADE_OUT: [u8; 9] = [114, 101, 88, 76, 63, 50, 38, 25, 12];

/// A fade's steps (both fades: 9 of 100 ms, `Sleep(100)` between).
const STEPS: u64 = 9;

/// What the window knows of a track the thread opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    /// Being decoded: the thread waits (the original opens its file at once).
    Decoding,
    /// Missing or not decodable: nothing plays (`AIL_open_stream` failed, 0x59AA01).
    Missing,
    /// Decoded, this many frames long.
    Frames(u64),
}

/// A run of one track's frames at one volume, for the stream: `frames` frames from `from`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// The track.
    pub name: String,
    /// Its first frame.
    pub from: u64,
    /// How many (never past the track's end: the next run starts again at 0).
    pub frames: u64,
    /// The volume, 0 to [`FULL`].
    pub volume: u8,
}

/// What `play_music` did with a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// Taken: the pending name, a thread started.
    Taken,
    /// Dropped: the name is pending already (0x59AD0D).
    Pending,
    /// Dropped: the name is the track opened last and its stream plays (0x59ADA6).
    Playing,
}

/// The music stream (the global 0x69FE78 and the manager's +0x8C).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Stream {
    /// None open: nothing yet, or the last track missing.
    Closed,
    /// The track opened last being decoded (the window's; the thread holds its lock).
    Opening,
    /// The track opened last playing at frame `at` of `len`, `heard` frames since its fade-in
    /// began (at most a fade's): the thread holds its lock until the fade-in's end.
    Playing { len: u64, at: u64, heard: u64 },
    /// The track opened last fading out at frame `at` of `len`, `heard` frames into the fade;
    /// the thread opens the pending track at its end.
    Fading { len: u64, at: u64, heard: u64 },
}

/// The original's music: the names, the stream and the places (`music_positions` 0x69FEC0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Music {
    /// A fade's step in frames (100 ms).
    step: u64,
    /// The track asked for and not yet opened (`music_pending_name` 0x6A3394).
    pending: Option<String>,
    /// The track the thread opened last (`music_playing_name` 0x6A3290): kept through its
    /// fade-out until the next opens.
    playing: Option<String>,
    stream: Stream,
    /// Where each track's last fade-out began, by name.
    places: BTreeMap<String, u64>,
}

impl Music {
    /// No music yet, on a stream of `rate` frames a second.
    #[must_use]
    pub fn new(rate: u32) -> Self {
        Self {
            step: (u64::from(rate) / 10).max(1),
            pending: None,
            playing: None,
            stream: Stream::Closed,
            places: BTreeMap::new(),
        }
    }

    /// A fade's length in frames (900 ms).
    fn fade(&self) -> u64 {
        self.step * STEPS
    }

    /// `play_music` 0x59ACB0 asked for `name` (a looping track, the effects kept).
    pub fn request(&mut self, name: &str) -> Request {
        if self.pending.as_deref() == Some(name) {
            return Request::Pending;
        }
        let plays = matches!(
            self.stream,
            Stream::Opening | Stream::Playing { .. } | Stream::Fading { .. }
        );
        if plays && self.playing.as_deref() == Some(name) {
            return Request::Playing;
        }
        self.pending = Some(name.into());
        Request::Taken
    }

    /// The track opened last, while its stream is open (fading out or being decoded too).
    #[must_use]
    pub fn playing(&self) -> Option<&str> {
        match self.stream {
            Stream::Closed => None,
            _ => self.playing.as_deref(),
        }
    }

    /// The track asked for and not yet opened.
    #[must_use]
    pub fn pending(&self) -> Option<&str> {
        self.pending.as_deref()
    }

    /// Where `name`'s last fade-out began.
    #[must_use]
    pub fn place(&self, name: &str) -> Option<u64> {
        self.places.get(name).copied()
    }

    /// The next `frames` frames heard, as runs; fewer when nothing plays (none asked for, a track
    /// missing or being decoded: the clock waits with the stream). `tracks` tells what the window
    /// knows of a track.
    pub fn advance(&mut self, frames: u64, tracks: &impl Fn(&str) -> Track) -> Vec<Run> {
        let mut runs = Vec::new();
        let mut left = frames;
        self.settle(tracks);
        while left > 0 {
            let (step, fade) = (self.step, self.fade());
            let (len, at, heard, volumes) = match &mut self.stream {
                Stream::Playing { len, at, heard } => (*len, at, heard, &FADE_IN),
                Stream::Fading { len, at, heard } => (*len, at, heard, &FADE_OUT),
                Stream::Closed | Stream::Opening => break,
            };
            let volume = usize::try_from(*heard / step)
                .ok()
                .and_then(|i| volumes.get(i))
                .copied()
                .unwrap_or(FULL);
            let to_step = if *heard < fade {
                step - *heard % step
            } else {
                u64::MAX
            };
            let n = left.min(to_step).min(len - *at);
            runs.push(Run {
                name: self.playing.clone().unwrap_or_default(),
                from: *at,
                frames: n,
                volume,
            });
            *at = (*at + n) % len;
            *heard = (*heard + n).min(fade);
            left -= n;
            self.settle(tracks);
        }
        runs
    }

    /// The thread's steps due now: a fade's end, a waiting thread let in, a decoded track opened.
    fn settle(&mut self, tracks: &impl Fn(&str) -> Track) {
        let fade = self.fade();
        loop {
            match self.stream {
                // The lock held through the fade-in and the fade-out.
                Stream::Playing { heard, .. } | Stream::Fading { heard, .. } if heard < fade => {
                    return;
                }
                // The fade-out's end: paused and closed (0x59A8F4); the pending track opens next.
                Stream::Fading { .. } => self.stream = Stream::Closed,
                // A thread let in once the fade-in ended: `music_stop_fade` keeps the place, then
                // fades the stream out from the whole.
                Stream::Playing { len, at, .. } => {
                    if self.pending.is_none() {
                        return;
                    }
                    let name = self.playing.clone().unwrap_or_default();
                    self.places.insert(name, at);
                    self.stream = Stream::Fading { len, at, heard: 0 };
                }
                // `music_thread` opens the pending name as it is now (0x59A96A, cleared at
                // 0x59A9A8).
                Stream::Closed => {
                    let Some(name) = self.pending.take() else {
                        return;
                    };
                    self.playing = Some(name);
                    self.stream = Stream::Opening;
                }
                Stream::Opening => {
                    let name = self.playing.as_deref().unwrap_or_default();
                    self.stream = match tracks(name) {
                        Track::Decoding => return,
                        Track::Missing | Track::Frames(0) => Stream::Closed,
                        Track::Frames(len) => Stream::Playing {
                            len,
                            at: self.place(name).filter(|&p| p < len).unwrap_or(0),
                            heard: 0,
                        },
                    };
                }
            }
        }
    }
}

/// The music on a stream queued ahead: the model at the frames heard and a copy at the frames
/// queued (M2 T6a2, the reviewer's R2-3 and R2-11). The window tells it how many frames the stream
/// still holds; a request lands at the frames heard, and when it is taken what is queued is
/// taken back (the window clears the stream) and queued again from there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queue {
    heard: Music,
    ahead: Music,
    /// Frames put on the stream since it started (less what was taken back).
    put: u64,
    /// Frames heard, as far as `heard` has run.
    heard_at: u64,
}

impl Queue {
    /// No music yet, on a stream of `rate` frames a second.
    #[must_use]
    pub fn new(rate: u32) -> Self {
        Self {
            heard: Music::new(rate),
            ahead: Music::new(rate),
            put: 0,
            heard_at: 0,
        }
    }

    /// The music at the frames heard.
    #[must_use]
    pub fn heard(&self) -> &Music {
        &self.heard
    }

    /// Frames heard so far.
    #[must_use]
    pub fn heard_at(&self) -> u64 {
        self.heard_at
    }

    /// The model run to the frames heard, the stream holding `queued` frames still.
    fn hear(&mut self, queued: u64, tracks: &impl Fn(&str) -> Track) {
        let now = self.put.saturating_sub(queued).max(self.heard_at);
        let _ = self.heard.advance(now - self.heard_at, tracks);
        self.heard_at = now;
    }

    /// `name` asked for while the stream holds `queued` frames. Taken, the stream must be cleared:
    /// what it holds is queued again from the frames heard.
    pub fn request(&mut self, name: &str, queued: u64, tracks: &impl Fn(&str) -> Track) -> Request {
        self.hear(queued, tracks);
        let request = self.heard.request(name);
        if request == Request::Taken {
            self.cleared();
        }
        request
    }

    /// The stream cleared (by a request, or by the window): queued again from the frames heard.
    pub fn cleared(&mut self) {
        self.put = self.heard_at;
        self.ahead = self.heard.clone();
    }

    /// The runs to put on the stream, which holds `queued` frames, so that it holds `want`.
    pub fn top_up(&mut self, queued: u64, want: u64, tracks: &impl Fn(&str) -> Track) -> Vec<Run> {
        self.hear(queued, tracks);
        let runs = self.ahead.advance(want.saturating_sub(queued), tracks);
        self.put += runs.iter().map(|r| r.frames).sum::<u64>();
        runs
    }
}

#[cfg(test)]
mod tests;
