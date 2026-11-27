//! The computer's spell gains of a hero (E31 T2e, AI-Q36; survey
//! `docs/rules/surveys/E31-computer-player.md` §2.4): its caster record on its map tile
//! ([`record`]: `ai_caster_record` 0x526D40, whose spell values live in `h3_battle::estimate`
//! beside the simulated combat, ADR-0030 point 6), what a spell adds to it (`ai_hero_spell_gain`
//! 0x527640, [`gain`]) and its values per unit at the turn's start (`ai_hero_effectiveness`
//! 0x527760 but its experience weight, [`gains`]). Nothing here changes the hero: the original
//! raises the record's numbers and lowers them back, the port values a changed copy.

use h3_battle::estimate::{CasterRecord, EstimateHero};
use h3_content::SpellId;
use h3_content_battle::{SpellAiKind, SpellAiKinds};

use super::fight::{self, Fighter};
use crate::{GameRuleset, GameState};

/// The least gain of a Spell Power and of a Knowledge point (0x52780D, 0x527885: 10).
const LEAST_GAIN: i32 = 10;

/// The mana a Knowledge point's gain is measured on (0x527850: 30) and its divisor (3).
const KNOWLEDGE_MANA: (i32, i32) = (30, 3);

/// Every AI kind (the original's mask `0x1F8000`).
pub(super) fn every_kind() -> SpellAiKinds {
    SpellAiKind::ALL
        .into_iter()
        .fold(SpellAiKinds::default(), SpellAiKinds::with)
}

/// The damage kinds a spell's gain is measured against (the mask `0x38000`).
const DAMAGE_KINDS: [SpellAiKind; 3] = [
    SpellAiKind::Damage,
    SpellAiKind::DamageOnce,
    SpellAiKind::DamageAll,
];

/// Whether `kinds` holds a damage kind (the mask `0x38000`): for the spells of the books the
/// spell table's flag 0x200 (deals damage) exactly (E31 T2f: 15 to 26 and 57 have both, no other
/// spell either).
pub(super) fn damage_kinds(kinds: SpellAiKinds) -> bool {
    DAMAGE_KINDS.into_iter().any(|k| kinds.contains(k))
}

/// The caster record of `hero` (`ai_caster_record` 0x526D40; CPU-SPL-001): its battle hero
/// with no visit bonuses and its available spells, its army, the special battlefield of its
/// map tile (`hero_tile_spell_field` 0x4E4FA0: the spell masteries' field, its special ground's).
pub(super) fn record(state: &GameState, rules: &GameRuleset, hero: Fighter<'_>) -> CasterRecord {
    let (battle_hero, spells) = fight::battle_hero(rules, hero);
    let ground = fight::ground(state, rules, hero.on_map.pos).battlefield;
    let reads = EstimateHero {
        hero: &battle_hero,
        spells: &spells,
    };
    CasterRecord::new(rules.battle(), reads, &hero.on_map.army, ground)
}

/// What spell `spell` adds to the hero of `record` (`ai_hero_spell_gain` 0x527640;
/// CPU-SPL-007): 0 without power (no spell book, the Orb of Inhibition); else its value
/// (`CasterRecord::value`), and for a spell of a damage kind its value less the best of the
/// hero's spells sharing one of its damage kinds, 1 when that best is at least its value.
// rule: CPU-SPL-007
pub(super) fn gain(rules: &GameRuleset, record: &CasterRecord, spell: SpellId) -> i32 {
    if record.power <= 0 {
        return 0;
    }
    let battle = rules.battle();
    let v = record.value(battle, spell);
    let kinds = battle
        .spell(spell)
        .map_or(SpellAiKinds::default(), |s| s.ai.kinds);
    let shared = DAMAGE_KINDS
        .into_iter()
        .filter(|&k| kinds.contains(k))
        .fold(SpellAiKinds::default(), SpellAiKinds::with);
    if shared == SpellAiKinds::default() {
        return v;
    }
    let best = record.best(battle, shared);
    if best >= v { 1 } else { v.wrapping_sub(best) }
}

/// A hero's spell values per unit (`ai_hero_effectiveness` 0x527760's +0x47E to +0x48E, H3API's
/// AI fields of the hero; CPU-STA-007): what one more of each is worth to its best spell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SpellGains {
    /// One Spell Power (+0x47E, at least 10).
    pub(super) power: i32,
    /// One round of its spells (+0x482).
    pub(super) rounds: i32,
    /// A Knowledge point (+0x486, at least 10).
    pub(super) knowledge: i32,
    /// Its mana raised to twice its maximum (+0x48A; 0 when it has as much).
    pub(super) doubled: i32,
    /// Its mana refilled to its maximum (+0x48E; 0 when it has as much).
    pub(super) refilled: i32,
}

/// The spell values per unit of a hero with the caster record `record` and `mana` spell points
/// left (`ai_hero_effectiveness` 0x527760 after its experience weight; CPU-STA-007), each the
/// best value of every kind (`CasterRecord::best`) of the record changed, less the record's own
/// (32 bits): one Spell Power, the power and the rounds one higher, at least 10; one round; 30
/// more mana over 3 (toward 0), at least 10; with the mana left as the record's, the mana
/// refilled to the maximum and raised to twice it, each less the best with the mana left, 0 when
/// the hero has that much already. A hero without power has no spell values: 10, 0, 10, 0, 0.
// rule: CPU-STA-007
pub(super) fn gains(rules: &GameRuleset, record: &CasterRecord, mana: i32) -> SpellGains {
    let battle = rules.battle();
    let all = every_kind();
    let best = |r: &CasterRecord| r.best(battle, all);
    let base = best(record);
    let mut r = record.clone();
    r.power = r.power.wrapping_add(1);
    r.rounds = r.rounds.wrapping_add(1);
    let power = best(&r).wrapping_sub(base).max(LEAST_GAIN);
    r.power = record.power;
    let rounds = best(&r).wrapping_sub(base);
    r.rounds = record.rounds;
    let (more, divisor) = KNOWLEDGE_MANA;
    r.mana = record.mana.wrapping_add(more);
    let knowledge = (best(&r).wrapping_sub(base) / divisor).max(LEAST_GAIN);
    let max = record.mana;
    r.mana = mana;
    let left = best(&r);
    let raised = |to: i32, r: &mut CasterRecord| {
        if mana >= to {
            return 0;
        }
        r.mana = to;
        let value = best(r).wrapping_sub(left);
        r.mana = mana;
        value
    };
    let refilled = raised(max, &mut r);
    let doubled = raised(max.wrapping_mul(2), &mut r);
    SpellGains {
        power,
        rounds,
        knowledge,
        doubled,
        refilled,
    }
}

#[cfg(test)]
mod tests;
