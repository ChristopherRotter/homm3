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
