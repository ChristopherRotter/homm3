//! The spells a side may aim and what the engine lets them be cast at (E17 M5 T3b; design §7.18):
//! the engine's part of an aim ([`h3_view::mouse::aim::Aim`]: its targets, area and kind, from the
//! battle and the ruleset), then the original's numbers the mouse needs (its spell number and
//! polarity, Anti-Magic's handle; from the asset tables).

use h3_battle::{BattleSide, BattleState, CastTarget, CastTargets, Viewer};
use h3_content_battle::BattleRuleset;
use h3_content_battle::SpellTarget;
use h3_protocol::map;
use h3_protocol::wire::Observation;
use h3_view::mouse::aim::{Aim, Area, Target};
use h3_view::mouse::{COLUMNS, ROWS};
use h3_view::tables::Tables;

use crate::motion::{spell_entry_in, spell_numbered_in};

/// Anti-Magic's number in the original.
const ANTI_MAGIC: u16 = 34;

/// The engine's part of the hero's aim of `spell` for `side` in `state`: none when the engine
/// casts it without a target or at nothing (the book casts it at once, or refuses it).
#[must_use]
pub fn hero(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: u16,
) -> Option<Aim> {
    let mut found = CastTargets::new();
    h3_battle::cast_targets(
        state,
        ruleset,
        side,
        h3_kernel::Handle::from_raw(spell),
        &mut found,
    );
    if found.is_empty() || found.iter().any(|t| *t == CastTarget::None) {
        return None;
    }
    let mut targets = targets_of(&found);
    targets.sort_unstable();
    // The hero's mastery of it, as its side's own view shows it.
    let own = map::observe(state, ruleset, Viewer::Side(side));
    let mastery = own
        .own_hero
        .as_ref()
        .and_then(|h| h.spell_book.iter().find(|b| b.spell == spell))
        .map_or(0, |b| b.mastery.min(3));
    Some(engine_part(ruleset, spell, mastery, targets, false))
}

/// The engine's part of the Faerie Dragon's aim of its drawn spell (the key F, 0x475C17) for
/// `side` in `state`: the engine's targets of its cast (`creature_cast_targets`, BAT-MAG-049),
/// the spell `own` (the side's own view) names as drawn; none for another stack (only a random
/// caster draws) or one that may not cast.
#[must_use]
pub fn dragon(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    own: &Observation,
) -> Option<Aim> {
    let spell = own.drawn?;
    let mut found = CastTargets::new();
    h3_battle::creature_cast_targets(state, ruleset, side, &mut found);
    let mut targets = targets_of(&found);
    if targets.is_empty() {
        return None;
    }
    targets.sort_unstable();
    // The original asks the acting side's hero's mastery (0x59F74D); the areas a Faerie Dragon
    // draws are the same at every mastery.
    Some(engine_part(ruleset, spell, 0, targets, true))
}

/// Whether `ruleset`'s `spell` is cast at a second target too (Teleport's hex, Sacrifice's
/// victim: BAT-MAG-029): the aims of their own are not built (E17 M5 T8b2, R4b-4: the ruleset's,
/// not the asset tables' number, which a spell the tables lack has none of).
#[must_use]
pub fn second_target(ruleset: &BattleRuleset, spell: u16) -> bool {
    (ruleset.spells().get(usize::from(spell))).is_some_and(|s| {
        let magic = &s.magic;
        magic.teleports() || magic.sacrifices()
    })
}

/// The aim of `spell` from a server's answer `targets` (E17 M5 T6b1: a seat on a server asks the
/// protocol's `Query`, where no engine runs here), a creature's cast or the hero's at its
/// `mastery`, in the battle of the `stacks` numbered: none when the server casts it without a
/// target or at nothing. The answer is the server's: each target once, a hex off the field or a
/// stack the battle does not hold left out (E17 M5 T8b2, R4b-6). The ruleset's part (the area,
/// the level, a stack spell at that mastery) only with the game's own content (`ruleset`: the
/// content check's `Same`); without it the aim marks no area, aims at a stack when every target
/// is one, at level 0 (*provisional*: a preview is the client's own content's).
#[must_use]
pub fn served(
    ruleset: Option<&BattleRuleset>,
    (spell, mastery, creature): (u16, u8, bool),
    targets: &[h3_protocol::wire::Target],
    stacks: impl IntoIterator<Item = u8>,
) -> Option<Aim> {
    use h3_protocol::wire::Target as Wire;
    if targets.is_empty() || targets.iter().any(|t| matches!(t, Wire::None {})) {
        return None;
    }
    let mut held = [false; 256];
    for id in stacks {
        held[usize::from(id)] = true;
    }
    let field = COLUMNS * ROWS;
    let mut aimed: Vec<Target> = targets
        .iter()
        .filter_map(|t| match *t {
            Wire::Stack { stack } => held[usize::from(stack)].then_some(Target::Stack(stack)),
            Wire::Hex { hex } => (hex < field).then_some(Target::Hex(hex)),
            Wire::None {} => None,
        })
        .collect();
    aimed.sort_unstable();
    aimed.dedup();
    if aimed.is_empty() {
        return None;
    }
    Some(match ruleset {
        Some(ruleset) => engine_part(ruleset, spell, mastery.min(3), aimed, creature),
        None => Aim {
            spell,
            original: 0,
            creature,
            stacks: aimed.iter().all(|t| matches!(t, Target::Stack(_))),
            targets: aimed,
            area: None,
            polarity: 0,
            level: 0,
            anti_magic: None,
        },
    })
}

/// The engine's targets as the aim's (a cast at nothing left out).
fn targets_of(found: &CastTargets) -> Vec<Target> {
    found
        .iter()
        .filter_map(|t| match *t {
            CastTarget::Stack(s) => Some(Target::Stack(s.0)),
            CastTarget::Hex(h) => Some(Target::Hex(h.0)),
            CastTarget::None => None,
        })
        .collect()
}

/// An aim's engine part: its area (a spell aimed at a hex with an area: the original's flags
/// 0x280 and Berserk), whether it is aimed at a stack at `mastery`, its level.
fn engine_part(
    ruleset: &BattleRuleset,
    spell: u16,
    mastery: u8,
    targets: Vec<Target>,
    creature: bool,
) -> Aim {
    let record = ruleset.spells().get(usize::from(spell));
    let magic = record.map(|s| &s.magic);
    let area = magic
        .filter(|m| m.target == SpellTarget::Hex)
        .and_then(|m| m.area)
        .map(|a| Area {
            radius: a.radius[usize::from(mastery)],
            centre: a.inner == 0,
        });
    let stacks = magic.is_some_and(|m| {
        m.target == SpellTarget::Stack && m.mass_from.is_none_or(|from| mastery < from)
    });
    Aim {
        spell,
        original: 0,
        creature,
        targets,
        area,
        stacks,
        polarity: 0,
        level: record.map_or(0, |s| s.level),
        anti_magic: None,
    }
}

/// Fills `aim`'s original numbers from the asset `tables` by the record's spell `keys`: the
/// spell's number and polarity (the spell record's +0), Anti-Magic's handle.
pub fn name(aim: &mut Aim, keys: &[String], tables: &Tables) {
    if let Some(entry) = spell_entry_in(keys, tables, aim.spell) {
        aim.original = entry.original;
        aim.polarity = entry.sign;
    }
    aim.anti_magic = spell_numbered_in(keys, tables, ANTI_MAGIC).map(|(handle, _)| handle);
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use h3_protocol::wire::Target as Wire;

    use super::*;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn a_server_s_targets_are_kept_once_on_the_field() {
        // E17 M5 T8b2 (R4b-6): a hostile answer, its targets repeated, unsorted, a hex off the
        // field and a stack the battle does not hold among them.
        let targets = [
            Wire::Hex { hex: 200 },
            Wire::Stack { stack: 3 },
            Wire::Hex { hex: 40 },
            Wire::Stack { stack: 9 },
            Wire::Stack { stack: 1 },
            Wire::Hex { hex: 40 },
            Wire::Stack { stack: 3 },
            Wire::Hex { hex: 186 },
            Wire::Hex { hex: 187 },
        ];
        let aim = served(None, (7, 2, false), &targets, [0, 1, 2, 3]).unwrap();
        assert_eq!(
            aim.targets,
            [
                Target::Stack(1),
                Target::Stack(3),
                Target::Hex(40),
                Target::Hex(186)
            ]
        );
        assert!(!aim.stacks && aim.area.is_none() && aim.level == 0);
        // Only stacks: aimed at a stack.
        let stacks = [Wire::Stack { stack: 3 }, Wire::Stack { stack: 3 }];
        let aim = served(None, (7, 2, true), &stacks, [3]).unwrap();
        assert_eq!(aim.targets, [Target::Stack(3)]);
        assert!(aim.stacks && aim.creature);
        // Nothing left, no target, a cast at nothing: no aim.
        let off = [Wire::Hex { hex: 255 }, Wire::Stack { stack: 9 }];
        assert_eq!(served(None, (7, 0, false), &off, [3]), None);
        assert_eq!(served(None, (7, 0, false), &[], [3]), None);
        let none = [Wire::Stack { stack: 3 }, Wire::None {}];
        assert_eq!(served(None, (7, 0, false), &none, [3]), None);
    }

    #[test]
    fn teleport_and_sacrifice_take_a_second_target_by_the_ruleset() {
        // E17 M5 T8b2 (R4b-4): the synthetic pack's Teleport ("blink") and Sacrifice ("offer").
        let scenario = repo().join("packs/synthetic/scenarios/full.toml");
        let (ruleset, _) = crate::scenario::battle(&scenario, &repo().join("packs"), 1).unwrap();
        let count = u16::try_from(ruleset.spells().len()).unwrap();
        let second: Vec<String> = (0..=count)
            .filter(|&spell| second_target(&ruleset, spell))
            .map(|spell| ruleset.spells()[usize::from(spell)].key.to_string())
            .collect();
        assert_eq!(second, ["synthetic:blink", "synthetic:offer"]);
    }
}
