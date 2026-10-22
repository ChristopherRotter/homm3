//! Chain Lightning (E17 M3 T7c-3; `docs/epics/E17-design.md` §7.16): `chain_lightning`
//! 0x5A6360's own drawing, its rays between the stacks it strikes ([`crate::ray::chain`]) and the
//! draws they turn by.

use alloc::vec::Vec;

use h3_protocol::wire::Event;

use super::{Beam, Player, Riding, STANDING, STRUCK, Sound, SpellLook, TICK};
use crate::battle_log::Logged;
use crate::ray;

/// Chain Lightning (the executable's spell number).
pub const CHAIN_LIGHTNING: u16 = 19;
/// Its opening animation over the first stack struck (0x5A6416), at 0 ms a frame: drawn, never
/// on the screen (CLT-Q11, built as the original's).
pub const CHAIN_START: u8 = 37;

/// The draws Chain Lightning's rays turn by: the kept ones in order ([`super::Casts::rays`]),
/// then the viewer's own, `random_between` 0x50B230 over the C library's `rand()` sequence (as
/// the fidgets', *provisional*: the draws the original makes between them are not followed).
#[derive(Debug, Clone)]
pub(super) struct Draws {
    kept: Vec<i32>,
    next: usize,
    seed: u32,
}

impl Draws {
    /// The draws, `kept` first.
    pub(super) fn new(kept: &[i32]) -> Self {
        Self {
            kept: kept.to_vec(),
            next: 0,
            seed: 0,
        }
    }

    /// The next draw from `lo` to `hi` (`lo` < `hi`).
    fn between(&mut self, lo: i32, hi: i32) -> i32 {
        if let Some(&v) = self.kept.get(self.next) {
            self.next += 1;
            return v;
        }
        self.seed = self.seed.wrapping_mul(214_013).wrapping_add(2_531_011);
        let r = i32::try_from((self.seed >> 16) & 0x7FFF).unwrap_or(0);
        lo + r % (hi - lo + 1)
    }
}

impl Player<'_> {
    /// Chain Lightning's handler (`chain_lightning` 0x5A6360, design §7.16; E17 M3 T7c-3,
    /// `bolt`) on the stacks its `effects` strike, in the chain's order: the mouse hidden
    /// (0x5A6380); [`CHAIN_START`] over the first at 0 ms a frame; then a ray from each struck
    /// stack's centre (`stack_screen_x`, `stack_screen_y`) to the next's ([`ray::chain`], its
    /// ticks at once), on the screen for trunc(100 F) outside the deadline (0x5A6517) and erased
    /// by a redraw at once; then the spell's animation over every stack struck with their
    /// reactions (`strike_animation`, player 2), its damage's line (the first stack's value with
    /// the caster's bonuses and every kill, named by the word for creatures: `battle_log_damage`
    /// without a stack, 0x5A667C), a redraw at once, the mouse back. Returns whether it drew it
    /// (a stack was struck).
    pub(super) fn chain(&mut self, spell: u16, look: &SpellLook, effects: &[Event]) -> bool {
        let hits: Vec<(u8, u64, u32)> = effects
            .iter()
            .filter_map(|e| match *e {
                Event::SpellDamage {
                    target,
                    ref caster,
                    kills,
                    ..
                } => Some((target, caster.0, kills)),
                _ => None,
            })
            .collect();
        let Some(&(first, value, _)) = hits.first() else {
            return false;
        };
        self.cue(first, Sound::Spell(spell), false);
        let hidden = self.clock.now;
        let beams = self.beams.len();
        let wait = self.speed.scale(TICK);
        let mut from = None;
        for &(t, ..) in &hits {
            let Some(live) = self.live(t) else {
                continue;
            };
            let point = self.centre(&live);
            match from {
                None => {
                    self.over_one(CHAIN_START, (t, &live), 0, false);
                }
                Some(start) => {
                    let draws = &mut self.draws;
                    let ticks = ray::chain(start, point, &mut |lo, hi| draws.between(lo, hi));
                    let now = self.clock.now;
                    self.beams.push(Beam {
                        from: now,
                        step: 0,
                        until: now + wait,
                        ticks,
                        colours: ray::CHAIN_COLOURS.to_vec(),
                        cursor: [hidden, u32::MAX],
                    });
                    self.clock.wait(wait);
                    self.redraw();
                }
            }
            from = Some(point);
        }
        let hit: Vec<(u8, u32)> = hits.iter().map(|&(t, _, k)| (t, k)).collect();
        let riding = look.animation.and_then(|number| {
            let look = self.stage.magic.get(&number).cloned()?;
            // Over the marked stacks in their order (`strike_animation`'s, 0x46905A).
            let mut over: Vec<_> = hit
                .iter()
                .filter_map(|&(t, _)| {
                    let live = self.live(t)?;
                    Some((t, self.magic_at(&live, &look), !live.right))
                })
                .collect();
            over.sort_by_key(|o| o.0);
            Some(Riding { number, look, over })
        });
        let reactions = self.reactions(&hit);
        self.iterate(
            None,
            (0, 0, 0, 2),
            (&reactions, false),
            [STANDING, STRUCK],
            riding.as_ref(),
        );
        let line = Logged::AreaDamage {
            spell,
            damage: value,
            kills: hits.iter().map(|h| h.2).sum(),
            target: None,
        };
        self.log.push((self.clock.now, line));
        self.redraw();
        let back = self.clock.now;
        for b in &mut self.beams[beams..] {
            b.cursor[1] = back;
        }
        true
    }
}
