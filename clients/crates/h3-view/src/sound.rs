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
//! Volumes are Miles' (0 to 127): the music's 127 and every creature sound's 64 (0x43DCEB), each
//! scaled alike by its volume setting (0x5996C0), so an effect plays at about half the music's
//! level when the two settings are equal. The start sound's volume is not read (its loader's
//! default; the viewer takes 64, *provisional*).

use alloc::format;
use alloc::string::String;

/// The longest the battle waits for its start sound, in ms (0x463045).
pub const START_WAIT: u32 = 10_000;
/// The music's volume.
pub const MUSIC_VOLUME: u8 = 127;
/// A creature sound's volume (and the start sound's, *provisional*).
pub const EFFECT_VOLUME: u8 = 64;
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

    /// The music's file stem in the installation's `MP3` folder (`mp3\%s.mp3`, 0x684B34).
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
