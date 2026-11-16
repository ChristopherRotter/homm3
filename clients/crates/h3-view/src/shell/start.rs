//! The start (E27 M2 T5a; `docs/epics/E27-design.md` §2.1, §10): the clips before the main menu,
//! as `game_main` plays them (`start_clips` 0x4EE7A6), timed on the game's clock and drawn as the
//! game draws them, into its back buffer and onto the screen. [`Start`] is the timing: what the
//! game does at each reading of its clock (its [`Op`]s); [`Picture`] carries the ops out on the two
//! buffers with the clips' decoders.
//!
//! **The sequence.** The back buffer filled black and shown (0x4EE776); clip 28 `3DOLogo`
//! full screen (`video_play_fullscreen` 0x5972D0, always Smacker); if it ran to its end, clip 29
//! `NWCLogo` full screen; if that ran to its end, clip 30 `H3Intro` framed in `introrim.pcx`
//! (`video_play_framed` 0x4EE1B0). `NWCLogo` and `H3Intro` play from Bink with the setting Bink
//! Video on (Video Quality High), else from their Smacker copies (CLT-Q96); a clip that does not
//! open ends the sequence. Then the main menu.
//!
//! **The frames' times.** `SMACKW32.DLL`'s `SmackWait` (0x10007840, read; no sound track) counts in
//! 10 µs (the clock's milliseconds × 100): its first call sets the first frame's due time to the
//! call's time + the frame time and waits; a call at or after the due time lets the frame through
//! and sets the next due time to the call's time + the frame time, unless the call came between one
//! and two frame times late (an unsigned comparison of a negative difference: [`smack_wait`]), when
//! it is the due time + the frame time. So a Smacker clip's frames are due from the last one let
//! through: at 1 ms steps 67 ms apart at a frame time of 66.66 ms, at 10 ms steps 70. That drift is
//! the original's profile's ([`Bugs::Original`], the comparisons'); in the client's (CLT-Q129 (b),
//! the user, 2026-11-16: fix) each frame is due a frame time after the last one's due time, at any
//! step (`H3INTRO.SMK` silent then ends as with its sound, about 161.7 s in, not 169.8 s at 10 ms
//! steps). `BINKW32.DLL`'s `BinkWait` lets frame p through once the clock reaches the first frame's
//! time + ⌊1000 p / rate⌋ ms (E17 M5 T4b2b's measure, [`crate::clip`]): Bink keeps its rate. The
//! game's loop decodes a frame let through into the back buffer and shows what changed
//! (`video_show` 0x5979D0): a Smacker clip's rectangles from `SmackToBufferRect`, a Bink clip's
//! from `BinkGetRects` (`h3_bink::rects`), the union of each list as the game's own arithmetic
//! takes it ([`shown_union`]: a lower rectangle further left cuts off the right of those above it,
//! the screen there keeping an older frame until a later union covers it; CLT-Q124, decision 32:
//! the client shows the clip's whole rectangle, the union the comparisons'). `NWCLogo`'s Bink copy
//! (400 × 300, clip 29: `video_show`'s own case) is drawn at the back buffer's corner and stretched
//! onto the whole screen by DirectDraw. A Smacker clip's rectangles are `SMACKW32.DLL`'s
//! (`h3_smacker::Rects`, E27 M2 T5a2), already in the back buffer: `SmackToBuffer` was given the
//! clip's place.
//!
//! **The ends.** A clip's last frame is decoded but not shown: the clips with the clip table's
//! +0xA (28 to 31) end in the window manager's fade (0x602C50 with 1, 4, 0 → 0x6030E0): the back
//! buffer shown at full, half and a quarter of each channel's level, each step 50 ms on the clock
//! after the last (`wait_until`: the first reading at or past it), then the back buffer filled
//! black and shown, 50 ms after the third. So `NWCLogo`'s Bink copy fades from the back buffer,
//! at the corner unstretched (CLT-Q123; its last frame is black, so only a skip shows it;
//! decision 32: the client fades the screen as shown, the corner the comparisons').
//!
//! **Skipping** (as the first run's reading is not the client's, CLT-Q85: never the first run): a
//! key down but F4, a left or a right press stops the clip playing. The full-screen clips then
//! fade from what the back buffer holds; `H3Intro` stops at once (`video_stop_all`, no fade).
//! Either way the sequence ends. An input during a fade is lost when a clip follows (its loop
//! flushes the input queue first); when none does, the main menu takes it as it opens
//! (CLT-Q130 (a), the user, 2026-11-16: as the original, in both profiles; [`Start::keeps_input`],
//! the caller keeping the inputs for the menu).
//!
//! **The sound** (E27 M2 T5b1, design §11): with the sound on (a digital driver and the effects
//! setting above 0: `smacker_open_in_dialog` 0x598AF0 opens a Smacker clip's tracks then, 0x69FE58)
//! a clip's track plays from its first sample. A Smacker clip's sound goes through a DirectSound
//! buffer `SMACKW32.DLL` makes on the DirectSound object Miles opened (design §11.1), and its
//! frames are then timed by that buffer's play cursor, not by `SmackWait`'s clock
//! ([`SmackSound`]): frame 0 at once, the buffer's Play at
//! the DLL's sound service's next run (10 ms of the clock after its last), each later frame once
//! the bytes played + 8 reach that many frames' sound. A Bink clip's frames keep their clock
//! (`BinkWait` reads only it); its sound starts when `BINKW32.DLL` loads Miles' first buffer
//! (T5b2, design §12.2), in the `BinkDoFrame` of the first frame whose sound takes the DLL's ring
//! past one of Miles' buffers: the first frame, as every game clip's carries 0.76 s of sound
//! (`bink-check.sh`'s `start` lines, `binkprobe start`). Either stops at its clip's close: a
//! skip ([`Op::Quiet`]) or the last frame, before the fade ([`Op::Closed`]: the last frame's
//! sound never plays, CLT-Q125; decision 34: the client plays the track out, the cut the
//! comparisons'). [`Op::Sound`] says when it starts; the
//! cursor is an ideal device's (it moves on the clock, no lead), as the probes' stand-in
//! DirectSound (`clients/capture/vsound.h`).
//!
//! Pure: the times are the caller's clock readings (the replay's, `super::replay`; the window's).

use alloc::vec::Vec;

use h3_bink::Decoder as BinkDecoder;
use h3_render::Bugs;
use h3_render::screen::Screen as Pixels;

/// The screen's width and height.
pub const WIDTH: u32 = 800;
/// See [`WIDTH`].
pub const HEIGHT: u32 = 600;
/// A fade's step (`0x6030E0`: each step waits until 50 ms after the last began).
pub const FADE_STEP: u32 = 50;
/// Where `H3Intro` plays inside its frame (`video_play_framed` 0x4EE1B0: (80, 187)).
pub const FRAMED_AT: [u32; 2] = [80, 187];

/// The start's clips, in their order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// Clip 28 `3DOLogo` (full screen, Smacker always).
    Logo,
    /// Clip 29 `NWCLogo` (full screen).
    Nwc,
    /// Clip 30 `H3Intro` (in `introrim.pcx`'s frame).
    Intro,
}

/// A clip's file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// Smacker (`.smk`).
    Smacker,
    /// Bink (`.bik`).
    Bink,
}

impl Part {
    /// The three, in their order.
    pub const ALL: [Self; 3] = [Self::Logo, Self::Nwc, Self::Intro];

    /// The clip table's index (0x6839C0).
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::Logo => 28,
            Self::Nwc => 29,
            Self::Intro => 30,
        }
    }

    /// The format played: Bink for `NWCLogo` and `H3Intro` with the setting Bink Video on
    /// (`bink_video`, 0x6987A8: the table's +8 set for them), else Smacker.
    #[must_use]
    pub const fn codec(self, bink_video: bool) -> Codec {
        match self {
            Self::Nwc | Self::Intro if bink_video => Codec::Bink,
            _ => Codec::Smacker,
        }
    }

    /// The file played, as the video archives name it (their lookup ignores case).
    #[must_use]
    pub const fn file(self, bink_video: bool) -> &'static str {
        match (self, self.codec(bink_video)) {
            (Self::Logo, _) => "3DOLOGO.SMK",
            (Self::Nwc, Codec::Bink) => "NWCLOGO.BIK",
            (Self::Nwc, Codec::Smacker) => "NWCLOGO.SMK",
            (Self::Intro, Codec::Bink) => "H3INTRO.BIK",
            (Self::Intro, Codec::Smacker) => "H3INTRO.SMK",
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Logo => Some(Self::Nwc),
            Self::Nwc => Some(Self::Intro),
            Self::Intro => None,
        }
    }
}

/// How a clip's frames are timed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    /// Smacker's: the frame time in 10 µs ([`smack_period`]).
    Smacker(u32),
    /// Bink's: its rate, frames per `den` seconds.
    Bink {
        /// Frames…
        num: u32,
        /// …per so many seconds.
        den: u32,
    },
}

/// What the start needs of a clip's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// The picture's width and height.
    pub size: [u32; 2],
    /// Its frames.
    pub frames: u32,
    /// Their pace.
    pub pace: Pace,
}

/// A Smacker header's frame time in 10 µs, as `SmackOpen` takes it: a negative value is in
/// 10 µs already, a positive one in ms, 0 is 100 ms.
#[must_use]
pub const fn smack_period(rate: i32) -> u32 {
    if rate < 0 {
        rate.unsigned_abs()
    } else if rate > 0 {
        rate.unsigned_abs().saturating_mul(100)
    } else {
        10_000
    }
}

/// `SmackWait`'s schedule (module doc): the next due time in 10 µs (none before the first call).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SmackSchedule {
    next: Option<u32>,
}

/// One call of `SmackWait` at `now` ms with the frame time `period` (10 µs): whether a frame goes
/// through, in the profile `bugs`. The arithmetic is the DLL's, on 32-bit words. In the
/// original's profile ([`Bugs::Original`], the comparisons') the next frame is due a period after
/// the call that let this one through, as the DLL dates it (a clip's pace drifts with the
/// clock's step: module doc); in the client's ([`Bugs::Fixed`], CLT-Q129 (b)) a period after this
/// one's due time, every frame on its time whatever the step (a reading late by several periods
/// lets the frames due through at once, as Bink's wait does).
pub fn smack_wait(schedule: &mut SmackSchedule, now: u32, period: u32, bugs: Bugs) -> bool {
    let now = now.wrapping_mul(100);
    let Some(next) = schedule.next else {
        schedule.next = Some(now.wrapping_add(period));
        return false;
    };
    if now < next {
        return false;
    }
    let after = next.wrapping_add(period);
    // The DLL compares `now - after` with the period unsigned: a call before `after` (the usual
    // case) gives a huge difference, so the next frame is due a period after this call.
    schedule.next = Some(match bugs {
        Bugs::Original if now.wrapping_sub(after) > period => now.wrapping_add(period),
        Bugs::Original | Bugs::Fixed => after,
    });
    true
}

/// The service's least gap (`SMACKW32.DLL` 0x10008F50: it runs at a reading 10 ms or more after
/// its last run, and not before).
const SERVICE_GAP: u32 = 10;

/// A Smacker clip's sound as `SMACKW32.DLL` times the clip's frames by it (E27 M2 T5b1, design
/// §11; checked against the DLL's on every frame by `smackprobe wait`, `smacker-check.sh`): its
/// DirectSound buffer (half a second of the track, 0x10009750) played on an ideal device, the
/// DLL's sound service (0x10008F50: the cursor read, at most every 10 ms), its position
/// (0x100096A0: the bytes played as the service last saw them, carried on by the clock since that
/// value was first seen) and `SmackWait`'s test (0x10007840: a frame waits while the frames'
/// sound so far exceeds the position + 8). The DLL's ring and its refills are not modelled: the
/// buffer never runs dry in a clip (the first frame's sound fills it), so the position never
/// reaches the bytes written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmackSound {
    /// The track's bytes a second (the sound's +0x14).
    rate: u32,
    /// A frame's bytes × 1024 (+0x1C).
    per_frame: u64,
    /// The buffer's bytes (+0x44).
    size: u32,
    /// The service's next run (0x100148E4), none before its first.
    next: Option<u32>,
    /// A frame's sound in the DLL's ring (the first frame decoded).
    filled: bool,
    /// The clock at the buffer's Play.
    played: Option<u32>,
    /// The play cursor as the service last read it (+0x78) and the bytes it has passed.
    cursor: u32,
    heard: u32,
    /// The position's last value (+0x84), when it was first seen (+0x88), and its most (+0x8C).
    last: u32,
    at: u32,
    most: u32,
    /// The handle's +0x448: a frame decoded, the next waits on the sound.
    held: bool,
    /// The frames' sound so far × 1024 (+0x20, +0x24).
    due: u64,
}

impl SmackSound {
    /// A clip's sound of `rate` bytes a second, its frame time `period` in 10 µs, at its open.
    #[must_use]
    pub const fn new(rate: u32, period: u32) -> Self {
        Self {
            rate,
            per_frame: ((rate as u64) << 10) * period as u64 / 100_000,
            size: ((rate >> 1) + 0xFFF) & !0xFFF,
            next: None,
            filled: false,
            played: None,
            cursor: 0,
            heard: 0,
            last: 0,
            at: 0,
            most: 0,
            held: false,
            due: 0,
        }
    }

    /// The DLL's sound service at `now`: whether it started the buffer playing.
    fn service(&mut self, now: u32) -> bool {
        if self.next.is_some_and(|next| now < next) {
            return false;
        }
        self.next = Some(now.wrapping_add(SERVICE_GAP));
        match self.played {
            None if self.filled => {
                self.played = Some(now);
                true
            }
            None => false,
            Some(since) => {
                // The ideal device: the clock's milliseconds times the bytes a second, whole
                // samples (4 bytes), around the buffer.
                let moved = u64::from(now.wrapping_sub(since)) * u64::from(self.rate) / 1000;
                let at = u32::try_from((moved & !3) % u64::from(self.size)).unwrap_or(0);
                let passed = if at >= self.cursor {
                    at - self.cursor
                } else {
                    self.size - self.cursor + at
                };
                self.heard = self.heard.wrapping_add(passed);
                self.cursor = at;
                false
            }
        }
    }

    /// The bytes played at `now` as the DLL's position takes them.
    fn position(&mut self, now: u32) -> (u32, bool) {
        let started = self.service(now);
        let seen = self.heard;
        let mut at = seen;
        if seen == self.last && seen != 0 {
            let more = u64::from(now.wrapping_sub(self.at)) * u64::from(self.rate) / 1000;
            at = seen.wrapping_add(u32::try_from(more).unwrap_or(u32::MAX));
        } else {
            self.last = seen;
            self.at = now;
        }
        self.most = self.most.max(at);
        (self.most, started)
    }

    /// `SmackWait` at `now`: whether the frame goes through, and whether the buffer started.
    pub fn wait(&mut self, now: u32) -> (bool, bool) {
        let mut started = self.service(now);
        if !self.held {
            return (true, started);
        }
        let (at, s) = self.position(now);
        started |= s;
        let due = u32::try_from(self.due >> 10).unwrap_or(u32::MAX);
        if due > at.wrapping_add(8) {
            return (false, started);
        }
        self.held = false;
        (true, started)
    }

    /// `SmackDoFrame` at `now` after [`SmackSound::wait`] let the frame through: its service
    /// (which never starts the buffer: `SmackWait`'s ran at the same reading, and the service runs
    /// at most once in 10 ms), then the frame's sound.
    pub fn frame(&mut self, now: u32) {
        let _ = self.service(now);
        self.held = true;
        self.due += self.per_frame;
        self.filled = true;
    }

    /// The clock when the buffer began to play, if it has.
    #[must_use]
    pub const fn played(&self) -> Option<u32> {
        self.played
    }

    /// The bytes the ideal device has played by `now` (the sound heard when the clip closes
    /// then: the track's first so many, `smackprobe wait`'s played file).
    #[must_use]
    pub fn heard_by(&self, now: u32) -> u32 {
        self.played.map_or(0, |since| {
            let moved = u64::from(now.wrapping_sub(since)) * u64::from(self.rate) / 1000;
            u32::try_from(moved & !3).unwrap_or(u32::MAX)
        })
    }
}

/// What the game does to its buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// The back buffer filled black and shown whole (the start's first screen, a fade's end).
    Black,
    /// `introrim.pcx` copied whole onto the back buffer (`H3Intro`'s frame).
    Rim,
    /// The clip's frame decoded into the back buffer at its place (the frames come in order).
    Frame(Part, u32),
    /// The clip's rectangle shown: the back buffer's onto the screen (`NWCLogo`'s Bink copy
    /// stretched onto the whole screen).
    Show(Part),
    /// The whole back buffer shown.
    ShowAll,
    /// A fade's step: the screen the back buffer with each channel's level shifted down so many
    /// bits (0 to 2).
    Fade(u8),
    /// The clip's sound starts, from its first sample (E27 M2 T5b1).
    Sound(Part),
    /// The clip's sound stops: its clip stopped by a skip.
    Quiet(Part),
    /// The clip closed after its last frame: the original's sound stops there, its last frame's
    /// sound unplayed (CLT-Q125); the client's plays its track out (decision 34).
    Closed(Part),
}

/// A clip playing: its part and header, the frames let through so far, Smacker's schedule or
/// Bink's first frame's time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Playing {
    part: Part,
    header: Header,
    done: u32,
    smack: SmackSchedule,
    first: Option<u32>,
    /// With the sound on: a Smacker clip's sound clock, or a Bink clip's sound to start.
    sound: Option<SmackSound>,
    audible: bool,
    sounding: bool,
}

/// What the start is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// A clip plays.
    Playing(Playing),
    /// A fade: its last step, the time it began, what comes after it.
    Fading {
        step: u8,
        at: u32,
        then: Option<Part>,
    },
    /// The sequence ended: the main menu.
    Ended,
}

/// The start's timing (module doc): the clips' headers and what plays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Start {
    headers: [Option<Header>; 3],
    /// With the sound on, each clip's sound in bytes a second (none: silent or no track).
    sounds: [Option<u32>; 3],
    bink_video: bool,
    /// The profile: a silent Smacker clip's pace ([`smack_wait`]).
    bugs: Bugs,
    state: State,
}

impl Start {
    /// The start at its first reading of the clock (`now`), each clip's header (none for a clip
    /// that does not open), the setting Bink Video and the profile `bugs` (a silent Smacker
    /// clip's pace, [`smack_wait`]): the screen black, `3DOLogo` opened. The ops of that reading
    /// go to `ops`.
    #[must_use]
    pub fn new(
        headers: [Option<Header>; 3],
        bink_video: bool,
        bugs: Bugs,
        now: u32,
        ops: &mut Vec<Op>,
    ) -> Self {
        Self::with_sound(headers, [None; 3], bink_video, bugs, now, ops)
    }

    /// [`Start::new`] with the sound on (E27 M2 T5b1): `sounds` each clip's track in bytes a
    /// second (none for a clip without one); a Smacker clip's frames then follow its sound
    /// ([`SmackSound`]), and [`Op::Sound`] and [`Op::Quiet`] say when each clip's sound plays.
    #[must_use]
    pub fn with_sound(
        headers: [Option<Header>; 3],
        sounds: [Option<u32>; 3],
        bink_video: bool,
        bugs: Bugs,
        now: u32,
        ops: &mut Vec<Op>,
    ) -> Self {
        let mut start = Self {
            headers,
            sounds,
            bink_video,
            bugs,
            state: State::Ended,
        };
        ops.push(Op::Black);
        start.open(Part::Logo, now, ops);
        start
    }

    /// Whether the sequence ended (the main menu next).
    #[must_use]
    pub const fn ended(&self) -> bool {
        matches!(self.state, State::Ended)
    }

    /// The clip playing, if any (none during a fade).
    #[must_use]
    pub const fn playing(&self) -> Option<Part> {
        match self.state {
            State::Playing(Playing { part, .. }) => Some(part),
            _ => None,
        }
    }

    /// Whether an input given now waits for the main menu (CLT-Q130 (a), the user: as the
    /// original): during a fade after which no clip opens (`H3Intro`'s end, a skipped clip's, or
    /// one before a clip that does not open, *provisional*: the game's data never has one) the
    /// game's input queue keeps it, and the menu takes it as it opens; before a next clip that
    /// clip's loop flushes the queue (the input lost). The caller keeps such inputs and gives
    /// them to the menu.
    #[must_use]
    pub fn keeps_input(&self) -> bool {
        match self.state {
            State::Fading { then, .. } => then.is_none_or(|p| self.headers[p as usize].is_none()),
            _ => false,
        }
    }

    /// The setting Bink Video the start plays with.
    #[must_use]
    pub const fn bink_video(&self) -> bool {
        self.bink_video
    }

    /// Opens `part` at `now`: `H3Intro`'s frame first; the clip's first reading of its clock.
    fn open(&mut self, part: Part, now: u32, ops: &mut Vec<Op>) {
        let Some(header) = self.headers[part as usize] else {
            self.state = State::Ended;
            return;
        };
        let rate = self.sounds[part as usize];
        let sound = match (rate, header.pace) {
            (Some(rate), Pace::Smacker(period)) => Some(SmackSound::new(rate, period)),
            _ => None,
        };
        let playing = Playing {
            part,
            header,
            done: 0,
            smack: SmackSchedule::default(),
            first: None,
            sound,
            audible: rate.is_some(),
            sounding: false,
        };
        if part == Part::Intro {
            // `video_play_framed`: the rim onto the back buffer, a tick of the clip, the whole
            // screen shown.
            ops.push(Op::Rim);
            self.tick(playing, now, ops);
            ops.push(Op::ShowAll);
        } else {
            self.state = State::Playing(playing);
        }
        self.read(now, ops);
    }

    /// The game's readings of its clock at `now` (a reading after the last one's, or the same):
    /// what is due runs, its ops into `ops`.
    pub fn read(&mut self, now: u32, ops: &mut Vec<Op>) {
        // At one reading the game runs until it waits on the clock: a frame let through, a
        // fade's step, the next clip opened, until nothing more is due.
        loop {
            let before = self.state;
            match self.state {
                State::Playing(playing) => self.tick(playing, now, ops),
                State::Fading { step, at, then } => {
                    if now.wrapping_sub(at) as i32 >= FADE_STEP as i32 {
                        self.fade_step(step, now, then, ops);
                    }
                }
                State::Ended => return,
            }
            if self.state == before {
                return;
            }
        }
    }

    /// One tick of the clip `playing` at `now`: a frame let through when due.
    fn tick(&mut self, mut playing: Playing, now: u32, ops: &mut Vec<Op>) {
        let Playing {
            part, header, done, ..
        } = playing;
        let due = match (header.pace, playing.sound.as_mut()) {
            (_, Some(sound)) => {
                let (due, started) = sound.wait(now);
                if started {
                    playing.sounding = true;
                    ops.push(Op::Sound(part));
                }
                due
            }
            (Pace::Smacker(period), None) => smack_wait(&mut playing.smack, now, period, self.bugs),
            (Pace::Bink { num, den }, None) => {
                let start = *playing.first.get_or_insert(now);
                let at = u64::from(done) * 1000 * u64::from(den) / u64::from(num.max(1));
                u64::from(now.wrapping_sub(start)) >= at
            }
        };
        if !due {
            self.state = State::Playing(playing);
            return;
        }
        ops.push(Op::Frame(part, done));
        if let Some(sound) = playing.sound.as_mut() {
            sound.frame(now);
        } else if playing.audible && !playing.sounding {
            // A Bink clip's sound with its first frame (module doc: Miles' first buffer).
            playing.sounding = true;
            ops.push(Op::Sound(part));
        }
        if done + 1 >= header.frames {
            // The last frame: decoded, not shown; the clip closed (its sound cut) and the fade
            // (+0xA).
            if playing.sounding {
                ops.push(Op::Closed(part));
            }
            self.fade(now, part.next(), ops);
        } else {
            ops.push(Op::Show(part));
            playing.done = done + 1;
            self.state = State::Playing(playing);
        }
    }

    /// The fade begins at `now`: its first step shown.
    fn fade(&mut self, now: u32, then: Option<Part>, ops: &mut Vec<Op>) {
        ops.push(Op::Fade(0));
        self.state = State::Fading {
            step: 0,
            at: now,
            then,
        };
    }

    /// The fade's next step after `step`, due at `now`.
    fn fade_step(&mut self, step: u8, now: u32, then: Option<Part>, ops: &mut Vec<Op>) {
        if step < 2 {
            ops.push(Op::Fade(step + 1));
            self.state = State::Fading {
                step: step + 1,
                at: now,
                then,
            };
            return;
        }
        ops.push(Op::Black);
        match then {
            Some(part) => self.open(part, now, ops),
            None => self.state = State::Ended,
        }
    }

    /// An input that stops a clip (a key down but F4, a left or a right press) at `now`: the
    /// full-screen clips fade, `H3Intro` stops at once; the sequence ends. Nothing during a fade.
    pub fn skip(&mut self, now: u32, ops: &mut Vec<Op>) {
        let State::Playing(playing) = self.state else {
            return;
        };
        if playing.sounding {
            ops.push(Op::Quiet(playing.part));
        }
        if playing.part == Part::Intro {
            self.state = State::Ended;
        } else {
            self.fade(now, None, ops);
        }
    }
}

/// Whether the key down named `name` (the adapter's name, case ignored) stops a start's clip
/// (`video_play_fullscreen`'s and `video_play_framed`'s loops): every key but F4 (scan code
/// 0x3E). A left or a right press always does ([`Start::skip`]). The one rule of the replay's
/// inputs (`super::replay::skips`) and the window's.
#[must_use]
pub fn key_skips(name: &str) -> bool {
    !name.eq_ignore_ascii_case("F4")
}

/// A clip's decoding for [`Picture`].
#[derive(Debug, Clone)]
pub enum Source<'c, 'a> {
    /// A Smacker clip's frames in order.
    Smacker {
        /// Its reader.
        reader: alloc::boxed::Box<h3_smacker::Reader<'c, 'a>>,
        /// Its decoder.
        decoder: alloc::boxed::Box<h3_smacker::Decoder>,
    },
    /// A Bink clip's.
    Bink {
        /// The clip.
        clip: &'c h3_bink::Clip<'a>,
        /// Its decoder.
        decoder: alloc::boxed::Box<BinkDecoder>,
        /// The next frame to decode.
        next: usize,
    },
}

/// Why a frame was not drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureError {
    /// A frame asked for out of order, or of a clip without its source.
    Order(Part, u32),
    /// The Smacker decoder's.
    Smacker(h3_smacker::Error),
    /// The Bink decoder's.
    Bink(h3_bink::Error),
}

/// The game's back buffer and screen, the start's ops carried out on them (module doc).
#[derive(Debug, Clone)]
pub struct Picture<'c, 'a> {
    back: Pixels,
    screen: Pixels,
    sources: [Option<Source<'c, 'a>>; 3],
    places: [[u32; 4]; 3],
    rim: Option<&'c Pixels>,
    stretched: bool,
    bugs: Bugs,
    decoded: [u32; 3],
    /// Each Bink clip's rectangles of its last frame (`h3_bink::rects`), the first kept from a
    /// frame before when the last marked nothing (the DLL leaves it in the handle).
    rects: [Vec<h3_bink::Rect>; 3],
    /// Each Smacker clip's `SmackToBufferRect` (`h3_smacker::Rects`, at its place in the back
    /// buffer; a Bink clip's unused), drained when its frame is shown.
    smack: [h3_smacker::Rects; 3],
    /// What a fade fades: the back buffer as the game's fade copies it (`Bugs::Original`), the
    /// screen as shown (`Bugs::Fixed`, CLT-Q123); taken at its first step.
    fading: Option<Pixels>,
}

/// The union of a clip's rectangles as `video_show` (0x5979D0) takes it: from the first, each
/// later one's left and top edges taken when further left or up, its right and bottom edges when
/// further out than the union's width and height from the edges as they now are, so a rectangle
/// further left that does not reach as far right shortens the union by as much (the rectangles
/// come top to bottom: one lower and further left cuts off the right of those above it until a
/// later frame shows them; CLT-Q124). None for none.
#[must_use]
pub fn shown_union(rects: &[h3_bink::Rect]) -> Option<h3_bink::Rect> {
    let (&[mut x, mut y, mut w, mut h], rest) = rects.split_first()?;
    for &[rx, ry, rw, rh] in rest {
        x = x.min(rx);
        y = y.min(ry);
        if rx + rw > w + x {
            w = rx + rw - x;
        }
        if ry + rh > h + y {
            h = ry + rh - y;
        }
    }
    Some([x, y, w, h])
}

/// Where a clip's frame goes in the back buffer: a full-screen clip centred, `H3Intro` in its
/// frame, `NWCLogo`'s Bink copy at the corner (`video_play_fullscreen_bink` 0x44DD20's case of
/// clip 29).
#[must_use]
pub fn place(part: Part, codec: Codec, size: [u32; 2]) -> [u32; 4] {
    let [w, h] = size;
    let [x, y] = match (part, codec) {
        (Part::Intro, _) => FRAMED_AT,
        (Part::Nwc, Codec::Bink) => [0, 0],
        _ => [WIDTH.wrapping_sub(w) / 2, HEIGHT.wrapping_sub(h) / 2],
    };
    [x, y, w, h]
}

impl<'c, 'a> Picture<'c, 'a> {
    /// The buffers as `black` (the screen, 800 × 600 and black), each clip's source and header
    /// (none for one that does not open), the rim (`introrim.pcx`, 800 × 600), the setting Bink
    /// Video and `bugs`: [`Bugs::Original`] each frame as the DLLs copy it (a Smacker frame's
    /// written blocks through its palette, a Bink frame's changed cells: `h3_bink::copies`;
    /// CLT-Q119, CLT-Q122), its rectangles shown as the game takes their union ([`shown_union`],
    /// CLT-Q124) and a fade from the back buffer (`NWCLogo`'s at the corner, CLT-Q123);
    /// [`Bugs::Fixed`] every pixel, the clip's whole rectangle and a fade from the screen as
    /// shown (decisions 24, 28, 32).
    #[must_use]
    pub fn new(
        black: Pixels,
        sources: [Option<(Source<'c, 'a>, Header)>; 3],
        rim: Option<&'c Pixels>,
        bink_video: bool,
        bugs: Bugs,
    ) -> Self {
        let mut places = [[0; 4]; 3];
        let mut kept = [None, None, None];
        let mut smack = [const { h3_smacker::Rects::new([0, 0], [0, 0]) }; 3];
        for ((k, part), source) in Part::ALL.iter().enumerate().zip(sources) {
            if let Some((source, header)) = source {
                let [x, y, w, h] = place(*part, part.codec(bink_video), header.size);
                places[k] = [x, y, w, h];
                smack[k] = h3_smacker::Rects::new([w, h], [x, y]);
                kept[k] = Some(source);
            }
        }
        Self {
            back: black.clone(),
            screen: black,
            sources: kept,
            places,
            rim,
            stretched: bink_video,
            bugs,
            decoded: [0; 3],
            rects: [Vec::new(), Vec::new(), Vec::new()],
            smack,
            fading: None,
        }
    }

    /// The screen.
    #[must_use]
    pub const fn screen(&self) -> &Pixels {
        &self.screen
    }

    /// The back buffer.
    #[must_use]
    pub const fn back(&self) -> &Pixels {
        &self.back
    }

    /// Carries out `op`.
    ///
    /// # Errors
    /// A frame out of order or not decoded ([`PictureError`]): the buffers as they were.
    pub fn apply(&mut self, op: Op) -> Result<(), PictureError> {
        match op {
            Op::Black => {
                self.back.fill(0);
                self.screen.fill(0);
                self.fading = None;
            }
            Op::Rim => {
                if let Some(rim) = self.rim {
                    self.back
                        .copy_rect(rim, [0, 0, i64::from(WIDTH), i64::from(HEIGHT)]);
                }
            }
            Op::Frame(part, n) => self.frame(part, n)?,
            Op::Show(part) => self.show(part),
            Op::ShowAll => self.screen.clone_from(&self.back),
            Op::Fade(shift) => self.fade(shift),
            Op::Sound(_) | Op::Quiet(_) | Op::Closed(_) => {}
        }
        Ok(())
    }

    fn rect(&self, part: Part) -> [i64; 4] {
        self.places[part as usize].map(i64::from)
    }

    fn show(&mut self, part: Part) {
        let [x, y, w, h] = self.rect(part);
        if part == Part::Nwc && self.stretched {
            // DirectDraw's stretch of the corner onto the whole screen (`video_show`'s `Blt`),
            // the nearest pixel as Wine's software blit takes it: a step of the source's size
            // over the screen's in 16.16 fixed point (400 × 300: each pixel doubled).
            let back = self.back.pixels();
            let (sw, sh) = (WIDTH as usize, HEIGHT as usize);
            let x_step = ((w as usize) << 16) / sw;
            let y_step = ((h as usize) << 16) / sh;
            let out = self.screen.pixels_mut();
            for (row, line) in out.chunks_mut(sw).enumerate() {
                let from = ((row * y_step) >> 16) * sw;
                for (col, pixel) in line.iter_mut().enumerate() {
                    *pixel = back
                        .get(from + ((col * x_step) >> 16))
                        .copied()
                        .unwrap_or(0);
                }
            }
            return;
        }
        let k = part as usize;
        if let (Some(Source::Smacker { decoder, .. }), Bugs::Original) =
            (&self.sources[k], self.bugs)
        {
            // `SmackToBufferRect`'s rectangles' union as the game takes it, in the back
            // buffer already (CLT-Q124: the comparisons'; never none: the game reads one).
            let shown: Vec<h3_bink::Rect> = self.smack[k]
                .shown(decoder.frame())
                .iter()
                .map(|r| r.map(|v| i32::try_from(v).unwrap_or(i32::MAX)))
                .collect();
            let [rx, ry, rw, rh] = shown_union(&shown).unwrap_or_default().map(i64::from);
            self.screen
                .copy_rect(&self.back, [rx, ry, rx + rw, ry + rh]);
            return;
        }
        let bink = matches!(self.sources[k], Some(Source::Bink { .. }));
        if bink && self.bugs == Bugs::Original {
            // The DLL's rectangles' union as the game takes it (CLT-Q124: the comparisons').
            if let Some([rx, ry, rw, rh]) = shown_union(&self.rects[part as usize]) {
                let (rx, ry) = (x + i64::from(rx), y + i64::from(ry));
                self.screen
                    .copy_rect(&self.back, [rx, ry, rx + i64::from(rw), ry + i64::from(rh)]);
            }
            return;
        }
        self.screen.copy_rect(&self.back, [x, y, x + w, y + h]);
    }

    fn fade(&mut self, shift: u8) {
        let from = match (shift, self.fading.take()) {
            (1.., Some(from)) => from,
            _ => match self.bugs {
                Bugs::Original => self.back.clone(),
                Bugs::Fixed => self.screen.clone(),
            },
        };
        let masks = from.format().channel_masks();
        let shift = u32::from(shift);
        for (out, &pixel) in self.screen.pixels_mut().iter_mut().zip(from.pixels()) {
            let p = u32::from(pixel);
            let faded = masks
                .iter()
                .fold(0, |acc, &mask| acc | (((p & mask) >> shift) & mask));
            *out = u16::try_from(faded).unwrap_or(0);
        }
        self.fading = Some(from);
    }

    fn frame(&mut self, part: Part, n: u32) -> Result<(), PictureError> {
        let k = part as usize;
        let order = PictureError::Order(part, n);
        if self.decoded[k] != n {
            return Err(order);
        }
        let [x0, y0, _, _] = self.places[k].map(|v| v as usize);
        let bugs = self.bugs;
        let width = WIDTH as usize;
        let back = self.back.pixels_mut();
        match self.sources[k].as_mut().ok_or(order)? {
            Source::Smacker { reader, decoder } => {
                reader
                    .next()
                    .ok_or(order)?
                    .and_then(|packet| decoder.decode(&packet).map(|_| ()))
                    .map_err(PictureError::Smacker)?;
                self.smack[k].decoded();
                // The DLL's copy, its written blocks (CLT-Q119: the comparisons'), or every
                // pixel (decision 24).
                decoder.frame().copy_16(
                    h3_smacker::check::Format::Rgb565,
                    bugs == Bugs::Fixed,
                    back,
                    width,
                    [x0, y0],
                );
            }
            Source::Bink {
                clip,
                decoder,
                next,
            } => {
                clip.video(*next)
                    .and_then(|packet| decoder.decode(packet).map(|_| ()))
                    .map_err(PictureError::Bink)?;
                *next += 1;
                let rects = h3_bink::rects(decoder);
                if rects.is_empty() {
                    self.rects[k].truncate(1);
                } else {
                    self.rects[k] = rects;
                }
                let frame = decoder.frame();
                let pixels = crate::clip::rgb565(frame);
                let w = frame.planes()[0].width();
                for (at, &pixel) in pixels.iter().enumerate() {
                    let (x, y) = (at % w.max(1), at / w.max(1));
                    let copied = match bugs {
                        Bugs::Fixed => true,
                        Bugs::Original => h3_bink::copies(decoder, x, y),
                    };
                    let at = (y0 + y) * width + x0 + x;
                    if let Some(p) = back.get_mut(at).filter(|_| copied && x0 + x < width) {
                        *p = pixel;
                    }
                }
            }
        }
        self.decoded[k] = n + 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
