//! The effects of a turn's and a round's start (E17 M6 T5d1; `docs/epics/E17-design.md` §12.22):
//! a stack's regeneration as its pick starts its turn (`stack_turn_start` 0x446E30), a Wraith's
//! mana drain as it is activated (`stack_activation` 0x465330) and Poison's tenth at a round's
//! start (`stack_round_start` 0x447120). Each plays its sound, writes its line, draws its magic
//! animation over the stack and waits for the sound (design §8.3).

use h3_protocol::wire::{Event, Side};

use super::{POISON, Player, Sound, TICK};
use crate::battle_log::Logged;

/// Regeneration's animation over the stack (0x44708D: `sp12_`, the First Aid Tent's heal).
pub const REGENERATION: u8 = 79;
/// A Wraith's mana drain over it (0x4657BD: `sp08_`).
pub const MANA_DRAIN: u8 = 77;
/// Poison's tenth at a round's start over the poisoned stack (`strike_animation` 0x4472F5:
/// `sp11_`, with its hit).
pub const POISON_TENTH: u8 = 67;

/// The side whose hero's mana `stack`'s activation drains, if it does, from the events `after`
/// its `TurnStarted` and its regeneration (the engine's `activation`: after the checks that pass
/// it over, its side's opening spells first); none once another turn or round starts, or the
/// stack is passed over (`TurnPassed`) or frozen. The drain's own event is then drawn already:
/// the command's loop passes over it.
#[must_use]
pub fn drain_of(after: &[Event], stack: u8) -> Option<Side> {
    after
        .iter()
        .take_while(|e| {
            !matches!(
                e,
                Event::TurnStarted { .. }
                    | Event::RoundStarted { .. }
                    | Event::TurnPassed { .. }
                    | Event::BadMorale { .. }
                    | Event::Frightened { .. }
            )
        })
        .find_map(|e| match *e {
            Event::ManaDrained { stack: s, side, .. } if s == stack => Some(side),
            _ => None,
        })
}

impl Player<'_> {
    /// `stack` regenerates as its pick starts its first turn of the round (`stack_turn_start`
    /// 0x446F32 to 0x447097; E17 M6 T5d1, design §12.22): `Regener.wav` (the heal's sound,
    /// category 2), the log's line (GENRLTXT 371 or 372 by its number), animation
    /// [`REGENERATION`] over it without its hit, a frame a redraw at trunc(100 F), then the sound
    /// waited for. The pick runs before the turn starts, so the stack that acted last stays
    /// outlined ([`Player::acting`], as a freeze's; none at a round's first pick).
    pub(super) fn regenerated(&mut self, stack: u8) {
        let Some(live) = self.live(stack) else {
            return;
        };
        let cue = self.cue(stack, Sound::Heal, false);
        let line = Logged::Regenerated {
            stack,
            count: live.count,
        };
        self.log.push((self.clock.now, line));
        let time = self.speed.scale(TICK);
        if let Some(first) = self.over_one(REGENERATION, (stack, &live), time, false) {
            self.outlined.push((first, self.acting));
        }
        self.wait_for(cue);
    }

    /// `stack` drains the mana of the hero of `side` as it is activated (`stack_activation`
    /// 0x465626 to 0x4657CA; E17 M6 T5d1, design §12.22): `ManaDrai.wav` (category 2), the log's
    /// line naming the hero (GENRLTXT 374 or 375 by the stack's number), animation [`MANA_DRAIN`]
    /// over the Wraiths without their hit, then the sound waited for; the stack that acted last
    /// still outlined (none at a round's first pick; the activation clears the outline after it,
    /// 0x465804).
    pub(super) fn mana_drained(&mut self, stack: u8, side: Side) {
        let Some(live) = self.live(stack) else {
            return;
        };
        let cue = self.cue(stack, Sound::ManaDrain, false);
        let line = Logged::ManaDrained {
            stack,
            count: live.count,
            side: u8::from(side == Side::Defender),
        };
        self.log.push((self.clock.now, line));
        let time = self.speed.scale(TICK);
        if let Some(first) = self.over_one(MANA_DRAIN, (stack, &live), time, false) {
            self.outlined.push((first, self.acting));
        }
        self.wait_for(cue);
    }

    /// Poison takes another tenth of `stack`'s hit points at a round's start (`stack_round_start`
    /// 0x4472AB to 0x44730E; E17 M6 T5d1, design §12.22): `Poison.wav` (the spell's sound as a
    /// file, category 2), the log's line (Poison's on a stack, GENRLTXT 562 or 563,
    /// `battle_log_cast` 0x4472DE), `strike_animation` with [`POISON_TENTH`] over the stack and
    /// its hit (player 2), then the sound waited for. The round's start clears the outline only at
    /// its end (0x4761EE, after its line): `outlined`, the stack that acted last, stays outlined
    /// through the tenths.
    pub(super) fn poison_tenth(&mut self, stack: u8, outlined: Option<u8>) {
        let Some(live) = self.live(stack) else {
            return;
        };
        let cue = self
            .spell_handle(POISON)
            .map(|h| self.cue(stack, Sound::SpellFile(h), false));
        let line = Logged::Poisoned {
            stack,
            count: live.count,
        };
        self.log.push((self.clock.now, line));
        let before = self.magics.len();
        self.marked((stack, &live), 0, Some(POISON_TENTH));
        if let Some(first) = self.magics.get(before).map(|k| k.shown) {
            self.outlined.push((first, outlined));
        }
        if let Some(cue) = cue {
            self.wait_for(cue);
        }
    }
}
