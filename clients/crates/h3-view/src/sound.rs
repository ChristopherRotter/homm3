//! The battle's sounds (E17 M2 T5b; `docs/epics/E17-design.md` §8.2): what the opening plays.
//! When a creature's sound starts the timeline's cues say ([`crate::anim::Cue`]), which file it
//! is the asset tables ([`crate::tables::Sounds`]); the window plays them (`h3-view window`).
//!
//! The opening (`battle_start` 0x462A20): a start sound `BATTLE00` to `BATTLE07`, drawn by
//! random(1, 8) − 1 (0x462AEF) while the screen is set up; once the battlefield shows, the battle
//! waits while it plays, at most 10 s (0x46304B), and only then starts its clocks (0x463093)
//! after the music, `COMBAT01` to `COMBAT04` by random(1, 4) (0x46305A). The music loops
//! (`play_music` 0x59ACB0 with a loop count of 0) and fades in over its first second ([`FADE`]);
//! its start stops the effects still playing. The executable draws both from its rules'
//! generator (the C library's `rand()`); the viewer draws them from a sequence of its own seeded
//! with the battle's seed (*provisional*, as its fidgets).
//!
//! The end (E17 M3 T8): once the battle is decided (`battle_winner` 0x4762F0), `battle_end`
//! 0x477470 darkens the screen, builds the result's window and, before the window runs, plays the
//! result's [`Tune`] once (`play_music` with a loop count of 1, the effects ended first: 0x477905,
//! 0x4779B4), with no clock step between. Which tune the window picks ([`end_tune`]).
//!
//! Volumes are Miles' (0 to 127): the music's 127 and every creature sound's 64 (0x43DCEB), each
//! scaled alike by its volume setting (0x5996C0), so an effect plays at about half the music's
//! level when the two settings are equal. A sound played by its name (`play_sound_file`
//! 0x59A470: the start sound, the tent's heal) keeps its loader's default, 127 (the WAV's
//! constructor 0x566DA0 from 0x55C48A and 0x55C6DC: volume 127, played once).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;

use h3_protocol::wire::{Outcome, Side};

use crate::anim::{Cue, Sound};

/// The longest the battle waits for its start sound, in ms (0x463045).
pub const START_WAIT: u32 = 10_000;
/// The longest the battle waits for any other sample, in ms (E17 M5 T5b2): `wait_for_sample`
/// 0x59A4C0 and `sample_wait` 0x599EC0 with their argument −1, as every caller in the battle but
/// the start passes (0x59A4CD).
pub const SAMPLE_WAIT: u32 = 4_000;

/// How long each sample the battle may wait for plays, in ms (E17 M5 T5b2, design §8.3): the
/// stacks' own sounds by stack and kind, the created stacks' by creature and kind (E17 M5 T8b2,
/// R6-1), the battle's other sounds by cue, as the window's sound library resolves a cue (a named
/// sound first, then a created stack's creature's, else the stack's). A sample's length is its
/// frames at its rate, rounded up (`ceil(frames × 1000 / rate)`; the frames the WAV's `fact`
/// count, as the decoder's). The battle waits for its samples only when it plays them: a computer
/// with a sound device and an effects volume above 0 (`play_sample` 0x59A210 plays nothing at 0,
/// so nothing is waited for); without, the timeline has no lengths, as every capture before T5b2
/// (the harness's `sound` runs give the original a device).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lengths {
    /// Each stack's sounds by stack id and kind ([`Sound::Move`] to [`Sound::End`]): the stacks
    /// of the record's first snapshot.
    pub stacks: BTreeMap<(u8, Sound), u32>,
    /// The sounds of each creature the battle may create, by its handle and kind.
    pub creatures: BTreeMap<(u16, Sound), u32>,
    /// The battle's other sounds by cue.
    pub named: BTreeMap<Sound, u32>,
}

impl Lengths {
    /// How long `cue`'s sample plays, if it has one ([`Cue::creature`]: a created stack's by its
    /// creature).
    #[must_use]
    pub fn of(&self, cue: &Cue) -> Option<u32> {
        let sound = cue.sound;
        self.named
            .get(&sound)
            .or_else(|| match cue.creature {
                Some(creature) => self.creatures.get(&(creature, sound)),
                None => self.stacks.get(&(cue.stack, sound)),
            })
            .copied()
    }
}

/// A sample's length in ms: `frames` at `rate` Hz, rounded up ([`Lengths`]; a rate of 0 none).
#[must_use]
pub fn length_ms(frames: u64, rate: u32) -> u32 {
    if rate == 0 {
        return 0;
    }
    let ms = frames.saturating_mul(1000).div_ceil(u64::from(rate));
    u32::try_from(ms).unwrap_or(u32::MAX)
}
/// The music's volume.
pub const MUSIC_VOLUME: u8 = 127;
/// A creature sound's volume (0x43DCEB).
pub const EFFECT_VOLUME: u8 = 64;
/// The volume of a sound played by its name, the start sound and the tent's heal: its loader's
/// default (0x566DA0).
pub const FILE_VOLUME: u8 = 127;
/// The music's fade-in (0x59AB5D): it starts silent and at once takes the first of these, then
/// the next every [`FADE_STEP`] ms (a tenth of [`MUSIC_VOLUME`] more each, added up in f32 and
/// truncated), then the whole volume.
pub const FADE: [u8; 9] = [12, 25, 38, 50, 63, 76, 88, 101, 114];
/// One step of the fade-in, in ms (0x59AB9A).
pub const FADE_STEP: u32 = 100;

/// The music's volume `ms` after it started.
#[must_use]
pub fn music_volume(ms: u32) -> u8 {
    usize::try_from(ms / FADE_STEP)
        .ok()
        .and_then(|step| FADE.get(step))
        .copied()
        .unwrap_or(MUSIC_VOLUME)
}

/// What a battle's opening plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opening {
    /// The start sound's number, 0 to 7.
    pub start: u8,
    /// The music's number, 1 to 4.
    pub music: u8,
}

impl Opening {
    /// The start sound's entry in the sound archives.
    #[must_use]
    pub fn start_sound(&self) -> String {
        format!("BATTLE{:02}", self.start)
    }

    /// The music's file stem in the installation's `MP3` folder (`mp3\%s.mp3`).
    #[must_use]
    pub fn music_file(&self) -> String {
        format!("COMBAT{:02}", self.music)
    }
}

/// The opening of a battle seeded with `seed`: the start sound, then the music, each a draw of
/// the C library's `rand()` sequence started at `seed`.
#[must_use]
pub fn opening(seed: u32) -> Opening {
    let mut state = seed;
    let start = rand(&mut state) % 8;
    let music = 1 + rand(&mut state) % 4;
    Opening {
        start: u8::try_from(start).unwrap_or_default(),
        music: u8::try_from(music).unwrap_or(1),
    }
}

/// A tune the battle's end plays once, the result's window's choice (`battle_end`'s 0x4702D0 sets
/// 0x695014, the index into the names 0x6701A8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tune {
    /// The side shown won (0).
    Win,
    /// The side shown lost (1).
    Lose,
    /// The side shown defended its town and won (2).
    DefendCastle,
    /// The side shown lost, and a side retreated (3).
    Retreat,
    /// The side shown lost, and a side surrendered (4).
    Surrender,
    /// The side shown lost at its town (5).
    LoseCastle,
}

impl Tune {
    /// The tune's file stem in the installation's `MP3` folder (`mp3\%s.mp3`; the executable's
    /// names, found ignoring case).
    #[must_use]
    pub const fn file(self) -> &'static str {
        match self {
            Self::Win => "win battle",
            Self::Lose => "losecombat",
            Self::DefendCastle => "defend castle",
            Self::Retreat => "retreat battle",
            Self::Surrender => "surrender battle",
            Self::LoseCastle => "losecastle",
        }
    }
}

/// The tune a battle's end plays, as `battle_end` 0x477470 and its result's window 0x4702D0 pick
/// it. `human`: whether each side's player is a human on this computer (0x4CE970); `town`:
/// whether the battle is at a town (the manager's +0x53C8, a siege or not).
///
/// The window shows the winner's side when its player is human here (0x4778B1), else the other
/// one (a draw: the defender's). Shown the winner, it plays [`Tune::Win`], or
/// [`Tune::DefendCastle`] for the town's defender (0x470B68); shown the loser, it plays
/// [`Tune::LoseCastle`] for the town's defender, else [`Tune::Surrender`] or [`Tune::Retreat`] when
/// a side gave up so (the flags 0x697744 and 0x6985A3, set by either side), else [`Tune::Lose`]
/// (0x470BFE to 0x470CFF). Not distinguished: an opponent of a side that retreated or
/// surrendered left with only stacks created in the battle loses too (BAT-Q7; the window then
/// shows the defender's side), which the outcome does not say; the viewer takes it as the
/// winner.
#[must_use]
pub fn end_tune(outcome: Outcome, human: [bool; 2], town: bool) -> Tune {
    let index = |side: Side| usize::from(side == Side::Defender);
    let other = |side: Side| 1 - index(side);
    let (winner, gave_up) = match outcome {
        Outcome::Victory { side } => (Some(index(side)), None),
        // A ranked game's ends (only in `GameEnded`, never a battle's event): its winner, or none.
        Outcome::Forfeit { side } => (Some(other(side)), None),
        Outcome::Draw {} | Outcome::NoShow {} | Outcome::Unverified {} => (None, None),
        Outcome::Retreat { side } => (Some(other(side)), Some(Tune::Retreat)),
        Outcome::Surrender { side } => (Some(other(side)), Some(Tune::Surrender)),
    };
    match winner {
        Some(w) if human.get(w).copied().unwrap_or(false) => {
            if town && w == 1 {
                Tune::DefendCastle
            } else {
                Tune::Win
            }
        }
        // The loser's side shown, or the defender's after a draw.
        winner => {
            if town && winner.is_none_or(|w| w == 0) {
                Tune::LoseCastle
            } else {
                gave_up.unwrap_or(Tune::Lose)
            }
        }
    }
}

/// The C library's `rand()`: 15 bits of a linear congruential sequence.
fn rand(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(214_013).wrapping_add(2_531_011);
    (*state >> 16) & 0x7FFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_music_fades_in_over_its_first_second() {
        assert_eq!(music_volume(0), 12);
        assert_eq!(music_volume(99), 12);
        assert_eq!(music_volume(100), 25);
        assert_eq!(music_volume(899), 114);
        assert_eq!(music_volume(900), MUSIC_VOLUME);
        assert_eq!(music_volume(u32::MAX), MUSIC_VOLUME);
        // A tenth of 127 more each step, added up in f32 and truncated.
        for (i, &v) in FADE.iter().enumerate() {
            assert_eq!(u32::from(v), (u32::try_from(i).unwrap() + 1) * 127 / 10);
        }
    }

    #[test]
    fn the_end_plays_the_shown_sides_tune() {
        use Side::{Attacker as A, Defender as D};
        let both = [true, true];
        let red = [true, false];
        let blue = [false, true];
        let none = [false, false];
        // A human winner is shown: hot seat always so.
        assert_eq!(
            end_tune(Outcome::Victory { side: A }, both, false),
            Tune::Win
        );
        assert_eq!(
            end_tune(Outcome::Victory { side: D }, blue, false),
            Tune::Win
        );
        assert_eq!(
            end_tune(Outcome::Retreat { side: D }, both, false),
            Tune::Win
        );
        assert_eq!(
            end_tune(Outcome::Surrender { side: A }, blue, false),
            Tune::Win
        );
        // Else the loser is: how it lost.
        assert_eq!(
            end_tune(Outcome::Victory { side: D }, red, false),
            Tune::Lose
        );
        assert_eq!(
            end_tune(Outcome::Victory { side: A }, none, false),
            Tune::Lose
        );
        assert_eq!(
            end_tune(Outcome::Retreat { side: A }, red, false),
            Tune::Retreat
        );
        assert_eq!(
            end_tune(Outcome::Surrender { side: A }, red, false),
            Tune::Surrender
        );
        // A draw shows the defender's side, the loser's.
        assert_eq!(end_tune(Outcome::Draw {}, both, false), Tune::Lose);
        // The ranked game's ends, which no battle's event carries.
        assert_eq!(
            end_tune(Outcome::Forfeit { side: D }, red, false),
            Tune::Win
        );
        assert_eq!(end_tune(Outcome::NoShow {}, both, false), Tune::Lose);
        assert_eq!(end_tune(Outcome::Unverified {}, both, false), Tune::Lose);
        // At a town: its defender's own tunes.
        assert_eq!(
            end_tune(Outcome::Victory { side: D }, both, true),
            Tune::DefendCastle
        );
        assert_eq!(
            end_tune(Outcome::Victory { side: A }, both, true),
            Tune::Win
        );
        assert_eq!(
            end_tune(Outcome::Victory { side: A }, blue, true),
            Tune::LoseCastle
        );
        assert_eq!(
            end_tune(Outcome::Surrender { side: D }, blue, true),
            Tune::LoseCastle
        );
        assert_eq!(end_tune(Outcome::Draw {}, none, true), Tune::LoseCastle);
        assert_eq!(
            end_tune(Outcome::Victory { side: D }, red, true),
            Tune::Lose
        );
        assert_eq!(
            end_tune(Outcome::Retreat { side: A }, red, true),
            Tune::Retreat
        );
    }

    #[test]
    fn the_tunes_are_the_executables_names() {
        let all = [
            Tune::Win,
            Tune::Lose,
            Tune::DefendCastle,
            Tune::Retreat,
            Tune::Surrender,
            Tune::LoseCastle,
        ];
        let names: alloc::vec::Vec<&str> = all.iter().map(|t| t.file()).collect();
        assert_eq!(
            names,
            [
                "win battle",
                "losecombat",
                "defend castle",
                "retreat battle",
                "surrender battle",
                "losecastle"
            ]
        );
    }

    #[test]
    fn an_opening_draws_its_start_sound_and_its_music() {
        // The sequence from 1: 41, 18 467 (the C library's first draws).
        let o = opening(1);
        assert_eq!(o, Opening { start: 1, music: 4 });
        assert_eq!(o.start_sound(), "BATTLE01");
        assert_eq!(o.music_file(), "COMBAT04");
        let all: alloc::collections::BTreeSet<(u8, u8)> = (0..200)
            .map(|seed| {
                let o = opening(seed);
                (o.start, o.music)
            })
            .collect();
        assert!(all.iter().all(|&(s, m)| s < 8 && (1..=4).contains(&m)));
        assert_eq!(all.len(), 32);
    }
}
