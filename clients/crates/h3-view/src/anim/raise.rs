//! Raised stacks (E17 M3 T7c-3; `docs/epics/E17-design.md` §7.16, player 4): `raise_stack`
//! 0x5A7560 brings creatures back to a stack, a dead one up from the corpses, for Resurrection
//! and Animate Dead (`cast_spell` 0x5A1907), Sacrifice's target (0x5A1A49), an Archangel's raise
//! (`creature_cast_action` 0x4485BE) and a Phoenix's rebirth (0x4695AD).

use alloc::vec;

use h3_protocol::wire::Event;
use h3_render::Bugs;

use super::{CORPSE, MagicKey, Player, Pose, Sound, SpellLook, TICK};
use crate::battle_log::Logged;

/// Resurrection (the executable's spell number).
pub const RESURRECTION: u16 = 38;
/// Animate Dead.
pub const ANIMATE_DEAD: u16 = 39;
/// Sacrifice.
pub const SACRIFICE: u16 = 40;
/// Resurrection's animation (spell 38's +8), which `raise_stack` plays whatever raised the stack
/// (0x5A7740): Animate Dead's own (4) never plays (CLT-Q13; with the bugs fixed, E17 M3 T9d,
/// Animate Dead plays its own, [`Player::raising`]).
pub const RAISE: u8 = 50;
/// Sacrifice's animation over its victim (`cast_spell` 0x5A1A37; the spell's own is 52).
pub const SACRIFICED: u8 = 51;

impl Player<'_> {
    /// A hero's raise among a cast's `effects`, if `look` is a raising spell's (`cast_spell`
    /// 0x5A1907, 0x5A199C): the spell's sound on the stack raised (0x5A0331); Sacrifice first
    /// kills its victim: the cast's line on it (written after its damage, so by its number 0:
    /// 0x5A1A30, CLT-Q17) and animation [`SACRIFICED`] over it with its death
    /// (`strike_animation`), leaving no corpse: it dissolves ([`super::GONE`]; E17 M3 T9d,
    /// recorded in `sacrifice`); then
    /// [`Player::raise`] with [`RAISE`]. With the bugs fixed (E17 M3 T9d) the line names the victim
    /// by its number before its death (CLT-Q17) and Animate Dead raises with its own animation,
    /// the spell's (CLT-Q13). Returns whether it drew a raise.
    pub(super) fn raising(
        &mut self,
        side: u8,
        (spell, look): (u16, &SpellLook),
        effects: &[Event],
    ) -> bool {
        let raised = effects.iter().find_map(|e| match *e {
            Event::SpellRaised {
                target, creatures, ..
            } => Some((target, creatures)),
            _ => None,
        });
        let raising = matches!(look.original, RESURRECTION | ANIMATE_DEAD | SACRIFICE);
        let Some((target, creatures)) = raised.filter(|_| raising) else {
            return false;
        };
        self.cue(target, Sound::Spell(spell), false);
        let fixed = self.stage.bugs == Bugs::Fixed;
        let victim = effects.iter().find_map(|e| match *e {
            Event::Sacrificed { victim, creatures } => Some((victim, creatures)),
            _ => None,
        });
        if let Some((victim, live, killed)) = victim.and_then(|(v, k)| Some((v, self.live(v)?, k)))
        {
            let line = Logged::Cast {
                side,
                spell,
                target: victim,
                count: if fixed { live.count } else { 0 },
            };
            self.log.push((self.clock.now, line));
            self.vanishing.insert(victim);
            self.marked((victim, &live), killed, Some(SACRIFICED));
        }
        let number = match look.animation {
            Some(own) if fixed && look.original == ANIMATE_DEAD => own,
            _ => RAISE,
        };
        self.raise(target, creatures, number);
        true
    }

    /// `raise_stack` 0x5A7560 (player 4, design §7.16; E17 M3 T7c-3, `raise`): `creatures` brought
    /// back to `stack`, living or among the corpses. Its line first (GENRLTXT 117, 118 by the
    /// creatures raised, 0x5A772F), then L = max(n, d) redraws of trunc(100 F) on the deadline (n
    /// the frames of magic animation `number`, the original's always [`RAISE`], d of the stack's
    /// death group), each with the animation's frame i over the stack (none after n) and a lying
    /// stack's death group backwards (frame d − 1 − i) until it stands (i ≥ d), still among the
    /// corpses; then its dead bit and mark cleared and a redraw at once: it stands with its new
    /// count, the animation gone (0x5A7867 to 0x5A7881).
    pub(super) fn raise(&mut self, stack: u8, creatures: u32, number: u8) {
        let lying = !self.stacks.contains_key(&stack);
        let Some(mut live) = self
            .stacks
            .get(&stack)
            .or_else(|| self.corpses.get(&stack))
            .cloned()
        else {
            return;
        };
        let line = Logged::Raised {
            stack,
            count: creatures,
        };
        self.log.push((self.clock.now, line));
        let death = self.stage.groups.death;
        let d = live.look.count(death);
        let look = self.stage.magic.get(&number).cloned().unwrap_or_default();
        let over = vec![(stack, self.magic_at(&live, &look), !live.right)];
        let tick = self.speed.scale(TICK);
        for i in 0..look.frames.max(d) {
            if lying {
                let pose = match d.checked_sub(i + 1) {
                    Some(f) => Self::posed(&live, death, f, live.hex),
                    None => self.standing(&live, live.hex),
                };
                self.pending.push((
                    stack,
                    Pose {
                        layer: CORPSE,
                        ..pose
                    },
                ));
            }
            let frame = (i < look.frames).then_some(i);
            self.magic_frame((number, &look), &over, frame, tick);
        }
        live.count += creatures;
        let now = self.clock.now;
        self.magics.push(MagicKey {
            animation: number,
            drawn: now,
            shown: now,
            frame: None,
            def: look.def,
            translucent: look.translucent,
            over,
        });
        self.pending.push((stack, self.standing(&live, live.hex)));
        self.pending_counts.push((stack, live.count));
        self.redraw();
        self.corpses.remove(&stack);
        self.lying.remove(&stack);
        self.stacks.insert(stack, live);
    }

    /// An Archangel raises `target` (`creature_cast_action` 0x448563 to 0x4485DE; E17 M3 T7c-3,
    /// *provisional*: not recorded): its casting frames toward the corpse, `Resurect.wav` (spell
    /// 38's sound, started as a file) on the stack raised, [`Player::raise`], the sound waited for
    /// (0x4485DE, E17 M5 T5b2), then its own sound and it turns back.
    pub(super) fn creature_raise(&mut self, stack: u8, target: u8, creatures: u32) {
        let at = self
            .corpses
            .get(&target)
            .or_else(|| self.stacks.get(&target))
            .map(|l| l.position(l.hex));
        let Some(hex) = at else {
            return;
        };
        let turned = self.creature_casting(stack, hex);
        let cue = self
            .spell_handle(RESURRECTION)
            .map(|spell| self.cue(target, Sound::SpellFile(spell), false));
        self.raise(target, creatures, RAISE);
        if let Some(cue) = cue {
            self.wait_for(cue);
        }
        self.creature_cast_over(stack, turned);
    }

    /// A Phoenix reborn after the action that killed it (0x469588 to 0x4695AD; E17 M3 T7c-3,
    /// *provisional*: not recorded): Resurrection's sound (category 3), then [`Player::raise`].
    pub(super) fn reborn(&mut self, stack: u8, creatures: u32) {
        if let Some(spell) = self.spell_handle(RESURRECTION) {
            self.cue(stack, Sound::Spell(spell), false);
        }
        self.raise(stack, creatures, RAISE);
    }
}
