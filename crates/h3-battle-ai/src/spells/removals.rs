//! The removals (0x439A80 with Cure 0x439C30 and Anti-Magic 0x439D40; §7.46, §7.53;
//! BAT-AI-016): a spell that ends others on a stack is worth what the ended spells are worth to
//! their casters, each valued on a copy of the stack with the earlier ones ended.

use alloc::vec::Vec;

use h3_battle::computer;
use h3_battle::{ActiveEffect, StackId};
use h3_content::SpellCategory;
use h3_content::SpellId;
use h3_content_battle::{BattleSpell, SpellPolarity};
use h3_kernel::arith::X87;

use super::boosts::Signed;
use super::harms::poisons;
use super::{Candidate, View};

/// The rounds the original keeps for a spell that lasts to the battle's end (Disrupting Ray's
/// 255).
const FOREVER: u32 = 255;

impl View<'_> {
    /// A removal's value (0x439A80; BAT-AI-016) on stack `s`: over the spells lasting on it in
    /// the spells' order, each once, those `ends` selects (the spell, its record and whether it
    /// lasts to the battle's end): the spell ends on a copy of the battle ([`computer::end_spell`]:
    /// a permanent one stays, as the original's end routine never ends Disrupting Ray), so later
    /// spells are valued without the earlier ones, and its evaluator values it on the copy with
    /// the stack's mastery of it and its rounds left as the power and the rounds, as a removal
    /// (no resistance). A harmful spell on the half's own side, or a helpful one on the other
    /// side, adds its value in the other side's half; the rest subtract theirs in this one: a
    /// removal is worth what the removed spells are worth to their caster.
    // rule: BAT-AI-016
    fn removal(&self, s: StackId, ends: impl Fn(SpellId, &BattleSpell, bool) -> bool) -> i32 {
        let ruleset = self.ruleset;
        let mut lasting: Vec<ActiveEffect> = self
            .state
            .effects()
            .iter()
            .filter(|e| e.stack == s)
            .copied()
            .collect();
        lasting.sort_by_key(|e| e.spell.index());
        lasting.dedup_by_key(|e| e.spell);
        let mut copy = self.state.clone();
        let own = self.stack(s).side == self.side;
        let mut v = 0i32;
        for e in lasting {
            let Some(spell) = ruleset.spell(e.spell) else {
                continue;
            };
            if !ends(e.spell, spell, e.rounds.is_none()) {
                continue;
            }
            computer::end_spell(&mut copy, ruleset, s, e.spell);
            let rounds = e.rounds.map_or(FOREVER, u32::from);
            let c = Candidate {
                spell: e.spell,
                mastery: e.mastery,
                power: rounds,
                rounds,
                cast: false,
            };
            let harmful = spell.polarity == SpellPolarity::Negative;
            if harmful == own {
                v = v.wrapping_add(self.other().on(&copy).value(s, c));
            } else {
                v = v.wrapping_sub(self.on(&copy).value(s, c));
            }
        }
        v
    }

    /// Cure's evaluator (0x439C30; BAT-AI-016): the removal of the harmful spells on `s` (with
    /// BAT-QUIRK-25 every harmful one, Disrupting Ray included, survey slip 17; without, those the
    /// rule ends, BAT-MAG-016) plus the healing `x = min(lost, power × p + v)` (`lost` the top
    /// creature's lost hit points, `p` the spell's power factor, `v` its value): `trunc(x × worth /
    /// h + removal)` (0x442A50 with its shot flag, `h` a creature's hit points), the healing left
    /// out when the predictions beat the enemy and the stack loses no creature to the planned
    /// damage, or without any.
    // rule: BAT-AI-016
    pub(crate) fn cure(&self, s: StackId, c: Candidate) -> i32 {
        let ruleset = self.ruleset;
        let Some(cure) = ruleset.spell(c.spell) else {
            return 0;
        };
        let slip = self.slips();
        let removed = self.removal(s, |id, spell, permanent| {
            if slip {
                spell.polarity == SpellPolarity::Negative
            } else {
                !permanent && cure.dispels.contains(&id)
            }
        });
        let stack = self.stack(s);
        let lost = stack.max_hp.saturating_sub(stack.top_hp);
        let heal = u64::from(cure.power) * u64::from(c.power) + u64::from(self.effect(c));
        let x = u64::from(lost).min(heal);
        let creature = u64::from(stack.max_hp);
        let spared = self.planned.on(s).saturating_add(u64::from(lost)) < creature;
        if (self.beaten && spared) || x == 0 {
            return removed;
        }
        let shot = computer::shoots(self.state, ruleset, s);
        let worth = computer::worth(self.state, ruleset, self.ctx, s, shot);
        let healed = X87::int(x)
            .times(worth)
            .over(X87::int(creature.max(1)))
            .unwrap_or(X87::ZERO);
        let removal = Signed {
            negative: removed < 0,
            magnitude: X87::int(u64::from(removed.unsigned_abs())),
        };
        Signed {
            negative: false,
            magnitude: healed,
        }
        .plus(removal)
        .ftol_times(X87::ONE)
    }

    /// Anti-Magic's evaluator (0x439D40; BAT-AI-016) with its level `v`: the removal of the
    /// spells on `s` plus the spell protection's value against every school, nothing kept
    /// ([`Self::spell_protection`]). With BAT-QUIRK-25 (survey slip 11) the removal of every
    /// lasting spell but Poison and the protection up to level `v`; without, as the rule
    /// (BAT-MAG-021): the harmful spells below `v` that are no creature's ability and last no
    /// longer than their rounds, and the protection below `v`.
    // rule: BAT-AI-016
    pub(crate) fn anti_magic(&self, s: StackId, c: Candidate) -> i32 {
        let level = u8::try_from(self.effect(c)).unwrap_or(u8::MAX);
        let slip = self.slips();
        let removed = self.removal(s, |_, spell, permanent| {
            if slip {
                !poisons(&spell.magic.effects)
            } else {
                !permanent
                    && spell.polarity == SpellPolarity::Negative
                    && spell.category != SpellCategory::Ability
                    && spell.level < level
            }
        });
        let limit = if slip { level } else { level.saturating_sub(1) };
        removed.wrapping_add(self.spell_protection(s, (None, limit, 0), c.rounds))
    }
}
