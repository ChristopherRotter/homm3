//! The battle's result window from the battle's end (E17 M5 T4b1; `docs/epics/E17-design.md`
//! §7.20): what `h3_view::result` lays out, taken from the engine's outcome (`BattleOutcome`: the
//! result, every stack's start and end), the record's opening (the heroes, a town, the human
//! sides), the asset tables (the portraits, the creatures' original ids), the ruleset (hit points,
//! AI values) and the catalogue. Shared by the window's battle (`crate::battle`) and the capture
//! set's stills and sequences (`crate::spellbook`, `crate::sequences`).

use h3_content::{CreatureTypeId, Ruleset};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleOutcome, BattleResult, BattleSide};
use h3_protocol::wire::{Manifest, Observation};
use h3_view::layout::BattleLayout;
use h3_view::message::MessageBox;
use h3_view::result::{self, Face, GaveUp, Side, Texts};
use h3_view::tables::Tables;
use h3_view::text::{Args, Catalogue};

/// What the window is made from besides the battle: the record's manifest, the asset tables, the
/// ruleset, the catalogue and the layout.
pub struct Sources<'a> {
    /// The record's manifest (its keys).
    pub manifest: &'a Manifest,
    /// The asset tables.
    pub tables: &'a Tables,
    /// The battle's ruleset.
    pub ruleset: &'a BattleRuleset,
    /// The catalogue.
    pub catalogue: &'a Catalogue,
    /// The battle's layout.
    pub layout: &'a BattleLayout,
}

/// The result window of the battle that opened as `opening` (the record's first snapshot) and
/// ended as `outcome`, shown as `battle_end` 0x477470 shows it: the winner's side when its player
/// is a human here (`opening`'s sides' `human`), else the other one; its background in the
/// colours of `player` (the viewing player's). None when a hero's or creature's entry is missing.
pub fn window(
    sources: &Sources<'_>,
    opening: &Observation,
    outcome: &BattleOutcome,
    player: u8,
) -> Option<MessageBox> {
    let winner = outcome.result.winner().map(BattleSide::index);
    let gave_up = match outcome.result {
        BattleResult::Retreat(_) => Some(GaveUp::Retreat),
        BattleResult::Surrender(_) => Some(GaveUp::Surrender),
        BattleResult::Victory(_) | BattleResult::Draw => None,
    };
    let human = [0, 1].map(|i| opening.sides.get(i).is_some_and(|s| s.human));
    let shown = result::shown_side(winner, human);
    let won = winner == Some(shown);
    let town = opening.siege.is_some();
    let names: [Option<String>; 2] = [0, 1].map(|i| hero_name(sources, opening, i));
    let core: &Ruleset = sources.ruleset;
    let hit_points = |c: CreatureTypeId| sources.ruleset.creature(c).map_or(0, |t| t.hit_points);
    let experience = if won {
        let loser = 1 - shown;
        result::experience(
            outcome.stacks.get(loser).map_or(&[], Vec::as_slice),
            hit_points,
            (
                names[loser].is_some(),
                gave_up.is_some(),
                town && shown == 0,
            ),
        )
    } else {
        0
    };
    let text = result::text(
        sources.catalogue,
        won,
        gave_up,
        names[shown].as_deref(),
        experience,
    );
    let original = |c: CreatureTypeId| {
        let key = sources.manifest.keys.creatures.get(usize::from(c.raw()))?;
        sources
            .tables
            .creatures
            .get(key)
            .map(|e| u32::from(e.original))
    };
    let ai_value = |c: CreatureTypeId| core.creature(c).map_or(0, |t| t.ai_value);
    let mut sides = Vec::with_capacity(2);
    for (index, name) in names.into_iter().enumerate() {
        let stacks = outcome.stacks.get(index).map_or(&[][..], Vec::as_slice);
        let face = match name {
            Some(name) => Face::Hero {
                name,
                portrait: hero_portrait(sources, opening, index)?,
            },
            None => {
                let (creature, several) = result::strongest(stacks, original, ai_value)?;
                let key = sources
                    .manifest
                    .keys
                    .creatures
                    .get(usize::from(creature.raw()))?;
                let count = if several { 2 } else { 1 };
                Face::Creature {
                    name: sources
                        .catalogue
                        .text(&format!("creature.{key}"), &Args::new().count(count)),
                    original: original(creature)?,
                }
            }
        };
        sides.push(Side {
            face,
            victorious: winner == Some(index),
            losses: result::losses(stacks, original),
        });
    }
    let sides: [Side; 2] = sides.try_into().ok()?;
    let texts = Texts::of(sources.catalogue, text);
    Some(result::layout(
        &sources.layout.result,
        &texts,
        &sides,
        player,
    ))
}

/// The hero type's key of side `index`'s hero, if it has one.
fn hero_key<'a>(sources: &Sources<'a>, opening: &Observation, index: usize) -> Option<&'a str> {
    let hero = opening.sides.get(index)?.hero.as_ref()?;
    sources
        .manifest
        .keys
        .hero_types
        .get(usize::from(hero.hero_type))
        .map(String::as_str)
}

/// Side `index`'s hero's name: its type's (a map's own name for a hero is not read).
fn hero_name(sources: &Sources<'_>, opening: &Observation, index: usize) -> Option<String> {
    let key = hero_key(sources, opening, index)?;
    Some(sources.catalogue.text(&format!("hero.{key}"), &Args::new()))
}

/// Side `index`'s hero's large portrait: its type's (*provisional*: the hero's own portrait, its
/// +0x34 the executable reads, is its type's unless a map sets another, which the client does not
/// know).
fn hero_portrait(sources: &Sources<'_>, opening: &Observation, index: usize) -> Option<String> {
    let key = hero_key(sources, opening, index)?;
    sources.tables.heroes.get(key)?.portrait.clone()
}
