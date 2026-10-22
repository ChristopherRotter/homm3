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
use h3_kernel::order;

use super::boosts::Signed;
use super::harms::poisons;
use super::{Candidate, Kind, View};

/// The rounds the original keeps for a spell that lasts to the battle's end (Disrupting Ray's
/// 255).
const FOREVER: u32 = 255;

impl View<'_> {
    /// A removal's value (0x439A80; BAT-AI-016) on stack `s`: over the spells lasting on it in
    /// the spells' order, each once, those `ends` selects (the spell, its record and whether it
    /// lasts to the battle's end): the spell ends on a copy of the stack ([`computer::end_spell`]:
    /// Disrupting Ray stays, as the original's end routine never ends it, a permanent Acid Breath
    /// does not; only the slips select a permanent spell), so later
    /// spells are valued without the earlier ones, and its evaluator values it on the copy with
    /// the stack's mastery of it and its rounds left as the power and the rounds, as a removal
    /// (no resistance). A harmful spell on the half's own side, or a helpful one on the other
    /// side, adds its value in the other side's half; the rest subtract theirs in this one: a
    /// removal is worth what the removed spells are worth to their caster.
    ///
    /// The original's copy is a lone record (0x437A00), the battle untouched. With BAT-QUIRK-25
    /// (survey slip 24) the copy is valued so ([`View::on_copy`]): the battle's records, its
    /// record of the stack with every spell among them, are walked beside it and none is the copy
    /// (Disrupting Ray, Slow, Berserk, Hypnotize), and a two-hex shooter's copy is blocked by the
    /// stack itself where either is berserk or the copy is hypnotized
    /// ([`computer::block_by_itself`]). Without, the copy is the stack itself: a clone of the
    /// battle with the spells ended.
    // rule: BAT-AI-016
    fn removal(&self, s: StackId, ends: impl Fn(SpellId, &BattleSpell, bool) -> bool) -> i32 {
        let ruleset = self.ruleset;
        let mut lasting: Vec<(ActiveEffect, &BattleSpell)> = self
            .state
            .effects()
            .iter()
            .filter(|e| e.stack == s)
            .filter_map(|e| ruleset.spell(e.spell).map(|sp| (*e, sp)))
            .collect();
        // A stack holds one effect a spell (a recast renews it): the spells are the unique key.
        let _ = order::sort_by_unique_key(&mut lasting, |(e, _)| e.spell.index());
        let lone = self.slips().then_some(self.root.state);
        let mut copy = self.state.clone();
        let own = self.stack(s).side == self.side;
        let mut v = 0i32;
        for (e, spell) in lasting {
            if !ends(e.spell, spell, e.rounds.is_none()) {
                continue;
            }
            // The original's end routine (0x444510) keeps only Disrupting Ray; the rule keeps
            // every permanent effect (BAT-MAG-010), which only the slips select.
            let ray = Kind::of(spell) == Some(Kind::DisruptingRay);
            computer::end_spell(&mut copy, ruleset, (s, e.spell), !ray);
            if let Some(battle) = lone {
                computer::block_by_itself(&mut copy, battle, s);
            }
            let rounds = e.rounds.map_or(FOREVER, u32::from);
            let c = Candidate {
                spell: e.spell,
                mastery: e.mastery,
                power: rounds,
                rounds,
                cast: false,
            };
            let harmful = spell.polarity == SpellPolarity::Negative;
            let half = if harmful == own { self.other() } else { *self };
            let view = if lone.is_some() {
                half.on_copy(&copy)
            } else {
                half.on(&copy)
            };
            let x = view.value(s, c);
            v = if harmful == own {
                v.wrapping_add(x)
            } else {
                v.wrapping_sub(x)
            };
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
        let cure = ruleset.spell(c.spell);
        let slip = self.slips();
        let removed = self.removal(s, |id, spell, permanent| {
            if slip {
                spell.polarity == SpellPolarity::Negative
            } else {
                !permanent && cure.is_some_and(|cure| cure.dispels.contains(&id))
            }
        });
        let stack = self.stack(s);
        let lost = stack.max_hp.saturating_sub(stack.top_hp);
        let factor = cure.map_or(0, |cure| cure.power);
        let heal = u64::from(factor) * u64::from(c.power) + u64::from(self.effect(c));
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

    /// Dispel's evaluator (0x439BC0; BAT-AI-017): the removal of the spells on `s`, with
    /// BAT-QUIRK-25 every lasting spell but Poison (as the removal without its flag reads them;
    /// survey slip 17: Disrupting Ray too), without those the rule ends ([`computer::dispel_ends`],
    /// BAT-MAG-016).
    // rule: BAT-AI-017
    pub(crate) fn dispel(&self, s: StackId, c: Candidate) -> i32 {
        let dispel = self.ruleset.spell(c.spell);
        let slip = self.slips();
        self.removal(s, |id, spell, permanent| {
            if slip {
                !poisons(&spell.magic.effects)
            } else {
                dispel.is_some_and(|d| computer::dispel_ends(self.ruleset, d, (id, permanent)))
            }
        })
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
