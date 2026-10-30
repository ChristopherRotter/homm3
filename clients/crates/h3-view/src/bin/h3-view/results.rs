//! The battle's result window from the battle's end (E17 M5 T4b1; `docs/epics/E17-design.md`
//! §7.20): what `h3_view::result` lays out, taken from the engine's outcome (`BattleOutcome`: the
//! result, every stack's start and end), the record's opening (the heroes, a town, the human
//! sides), the asset tables (the portraits, the creatures' original ids), the ruleset (hit points,
//! AI values) and the catalogue. Shared by the window's battle (`crate::battle`) and the capture
//! set's stills and sequences (`crate::spellbook`, `crate::sequences`).

use h3_content::{CreatureTypeId, Ruleset};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, BattleOutcome, BattleResult, BattleSide, StackOrigin, StackRecord};
use h3_protocol::wire::{Manifest, Observation};
use h3_ui::text::{Args, Catalogue};
use h3_view::layout::BattleLayout;
use h3_view::message::MessageBox;
use h3_view::result::{self, Face, GaveUp, Side, Texts};
use h3_view::sound::Tune;
use h3_view::tables::Tables;

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
/// colours of `player` (the viewing player's); with the tune it plays, which picks its clip too
/// (E17 M5 T4b2b, [`tune`]). None when a hero's or creature's entry is missing.
pub fn window(
    sources: &Sources<'_>,
    opening: &Observation,
    outcome: &BattleOutcome,
    player: u8,
) -> Option<(MessageBox, Tune)> {
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
        experience(
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
                let (creature, several) = strongest(stacks, original, ai_value)?;
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
            losses: losses(stacks, original),
        });
    }
    let sides: [Side; 2] = sides.try_into().ok()?;
    let texts = Texts::of(sources.catalogue, text);
    let window = result::layout(&sources.layout.result, &texts, &sides, player);
    Some((window, tune(won, shown, town, gave_up)))
}

/// The window's tune, and so its clip (`h3_view::clip::of`), by the branches that pick its text
/// (0x470B68 to 0x470CFF; `h3_view::sound::end_tune` picks the same from the record's outcome):
/// the shown side won, [`Tune::DefendCastle`] for a town's defender, else [`Tune::Win`]; it lost,
/// [`Tune::LoseCastle`] for a town's defender, else the way a side gave up, else [`Tune::Lose`].
pub fn tune(won: bool, shown: usize, town: bool, gave_up: Option<GaveUp>) -> Tune {
    let defender_at_town = town && shown == 1;
    match (won, gave_up) {
        (true, _) if defender_at_town => Tune::DefendCastle,
        (true, _) => Tune::Win,
        (false, _) if defender_at_town => Tune::LoseCastle,
        (false, Some(GaveUp::Surrender)) => Tune::Surrender,
        (false, Some(GaveUp::Retreat)) => Tune::Retreat,
        (false, None) => Tune::Lose,
    }
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

/// The records the window reads of a side: its first 20 (the executable's loops run over 20 of a
/// side's 21 stack slots).
const RECORDS: usize = 20;
/// The arrow tower (149), which a side's strongest creature never is (0x4705BD).
const ARROW_TOWER: u32 = 149;
/// The points a beaten hero gives (0x46A3DF), taken back after a retreat or a surrender
/// (0x46A3FC), and given to an attacker that took a town (0x46A416).
const HERO_POINTS: i64 = 500;

/// A side's losses (0x470D89 to 0x470E48): over its records' (`stacks`, the engine's, in stack
/// order) creatures lost (`start − end`), each creature's sum in the order it first lost some, by
/// its original id (`original`; a creature without one is left out).
#[must_use]
pub fn losses(
    stacks: &[StackRecord],
    original: impl Fn(CreatureTypeId) -> Option<u32>,
) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for record in stacks.iter().take(RECORDS) {
        let lost = record.start.saturating_sub(record.end);
        let Some(id) = original(record.creature).filter(|_| lost > 0) else {
            continue;
        };
        match out.iter_mut().find(|(o, _)| *o == id) {
            Some((_, sum)) => *sum = sum.saturating_add(lost),
            None => out.push((id, lost)),
        }
    }
    out
}

/// A side without a hero's face (0x4705A2 to 0x470627): of its records but the arrow towers, the
/// first with the highest AI value (`ai_value`), its name plural when there are several
/// (`more`), as the executable reads them; its creature and whether there are several.
#[must_use]
pub fn strongest(
    stacks: &[StackRecord],
    original: impl Fn(CreatureTypeId) -> Option<u32>,
    ai_value: impl Fn(CreatureTypeId) -> u32,
) -> Option<(CreatureTypeId, bool)> {
    let mut best: Option<(CreatureTypeId, u32)> = None;
    let mut count = 0;
    for record in stacks.iter().take(RECORDS) {
        if original(record.creature) == Some(ARROW_TOWER) {
            continue;
        }
        count += 1;
        let value = ai_value(record.creature);
        if best.is_none_or(|(_, b)| value > b) {
            best = Some((record.creature, value));
        }
    }
    best.map(|(creature, _)| (creature, count > 1))
}

/// The points the result's text says the winner's hero receives (`battle_experience` 0x46A350):
/// the hit points (`hit_points`) of the creatures the loser's army and raised stacks lost
/// (`loser`, its records; not its war machines nor the stacks the battle created, which the
/// engine's `StackOrigin` tells as the executable's flags do), 500 more when the loser had a
/// hero (`beaten_hero`) unless a side gave up (`gave_up`), 500 more when the attacker took a town
/// (`town_taken`). *Not built:* the winner's Learning (×, `crt_ftol`): the client's heroes have
/// none (*provisional*: the scenarios name no Learning).
#[must_use]
pub fn experience(
    loser: &[StackRecord],
    hit_points: impl Fn(CreatureTypeId) -> u32,
    (beaten_hero, gave_up, town_taken): (bool, bool, bool),
) -> u32 {
    let killed: i64 = loser
        .iter()
        .take(RECORDS)
        .filter(|s| matches!(s.origin, StackOrigin::Army(_) | StackOrigin::Raised(_)))
        .map(|s| i64::from(s.start.saturating_sub(s.end)) * i64::from(hit_points(s.creature)))
        .sum();
    let mut points = killed;
    if beaten_hero {
        points += HERO_POINTS;
    }
    if gave_up {
        points -= HERO_POINTS;
    }
    if town_taken {
        points += HERO_POINTS;
    }
    u32::try_from(points.max(0)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_kernel::Handle;

    fn record(origin: StackOrigin, creature: u16, start: u32, end: u32) -> StackRecord {
        StackRecord {
            origin,
            creature: Handle::from_raw(creature),
            start,
            end,
        }
    }

    #[test]
    fn the_windows_tune_is_the_one_the_battles_end_plays() {
        use h3_protocol::wire::{Outcome, Side as Seat};
        let seat = |i: usize| {
            if i == 0 {
                Seat::Attacker
            } else {
                Seat::Defender
            }
        };
        for human in [[true, true], [true, false], [false, true], [false, false]] {
            for town in [false, true] {
                for winner in [None, Some(0), Some(1)] {
                    for gave_up in [None, Some(GaveUp::Retreat), Some(GaveUp::Surrender)] {
                        let outcome = match (winner, gave_up) {
                            (None, _) => Outcome::Draw {},
                            (Some(w), None) => Outcome::Victory { side: seat(w) },
                            (Some(w), Some(GaveUp::Retreat)) => {
                                Outcome::Retreat { side: seat(1 - w) }
                            }
                            (Some(w), Some(GaveUp::Surrender)) => {
                                Outcome::Surrender { side: seat(1 - w) }
                            }
                        };
                        let gave_up = winner.and(gave_up);
                        let shown = result::shown_side(winner, human);
                        let won = winner == Some(shown);
                        assert_eq!(
                            tune(won, shown, town, gave_up),
                            h3_view::sound::end_tune(outcome, human, town),
                            "{human:?} {town} {winner:?} {gave_up:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn losses_sum_by_creature_in_the_order_they_first_lost() {
        let stacks = vec![
            record(StackOrigin::Army(0), 5, 10, 10),
            record(StackOrigin::Army(1), 3, 10, 4),
            record(StackOrigin::Army(2), 5, 8, 0),
            record(StackOrigin::WarMachine, 7, 1, 0),
            record(StackOrigin::Army(3), 3, 2, 1),
            record(StackOrigin::Created, 9, 4, 0),
        ];
        let original = |c: CreatureTypeId| (c.raw() != 9).then_some(u32::from(c.raw()) * 10);
        assert_eq!(losses(&stacks, original), vec![(30, 7), (50, 8), (70, 1)]);
    }

    #[test]
    fn the_strongest_creature_is_the_first_of_the_highest_value_without_the_towers() {
        let stacks = vec![
            record(StackOrigin::Army(0), 1, 10, 0),
            record(StackOrigin::Army(1), 2, 10, 0),
            record(StackOrigin::Army(2), 3, 10, 0),
            record(StackOrigin::WarMachine, 149, 1, 1),
        ];
        let original = |c: CreatureTypeId| Some(u32::from(c.raw()));
        let value = |c: CreatureTypeId| match c.raw() {
            2 | 3 => 50,
            149 => 1000,
            _ => 10,
        };
        assert_eq!(
            strongest(&stacks, original, value),
            Some((Handle::from_raw(2), true))
        );
        assert_eq!(
            strongest(&stacks[..1], original, value),
            Some((Handle::from_raw(1), false))
        );
        assert_eq!(strongest(&[], original, value), None);
    }

    #[test]
    fn experience_counts_the_loser_s_army_and_the_heroes() {
        let loser = vec![
            record(StackOrigin::Army(0), 1, 10, 0),
            record(StackOrigin::Raised(1), 2, 4, 1),
            record(StackOrigin::WarMachine, 3, 1, 0),
            record(StackOrigin::Created, 4, 5, 0),
        ];
        let hp = |c: CreatureTypeId| u32::from(c.raw()) * 10;
        // 10 × 10 + 3 × 20; the machine and the created stack give nothing.
        assert_eq!(experience(&loser, hp, (false, false, false)), 160);
        assert_eq!(experience(&loser, hp, (true, false, false)), 660);
        assert_eq!(experience(&loser, hp, (true, true, false)), 160);
        assert_eq!(experience(&loser, hp, (true, false, true)), 1160);
        assert_eq!(experience(&[], hp, (false, true, false)), 0);
    }
}

/// The outcome of a battle on a server as a seat's record shows it (E17 M5 T6b1; no engine runs
/// here): the game's end `result` (a forfeit the other side's victory; a void game a draw), and
/// each side's stacks in id order as the battle `end` (the seat's, `h3_view::remote::seen`) shows
/// them: where each came from, its creatures at the `opening` (or when the battle created it) and
/// at the end. What the result window reads only (*provisional*: the survivors, the heroes'
/// outcomes and the surrender's cost are left empty).
pub fn served(
    opening: &Observation,
    end: &Observation,
    result: h3_protocol::wire::Outcome,
) -> BattleOutcome {
    use h3_protocol::wire::{Outcome, Side as Wire};
    let side = |s: Wire| match s {
        Wire::Attacker => BattleSide::Attacker,
        Wire::Defender => BattleSide::Defender,
    };
    let result = match result {
        Outcome::Victory { side: s } => BattleResult::Victory(side(s)),
        Outcome::Retreat { side: s } => BattleResult::Retreat(side(s)),
        Outcome::Surrender { side: s } => BattleResult::Surrender(side(s)),
        Outcome::Forfeit { side: s } => BattleResult::Victory(side(s).opponent()),
        Outcome::Draw {} | Outcome::NoShow {} | Outcome::Unverified {} => BattleResult::Draw,
    };
    let mut stacks: [Vec<StackRecord>; 2] = [Vec::new(), Vec::new()];
    for stack in &end.stacks {
        let origin = if stack.war_machine {
            StackOrigin::WarMachine
        } else if stack.raised {
            StackOrigin::Raised(stack.slot)
        } else if stack.created {
            StackOrigin::Created
        } else {
            StackOrigin::Army(stack.slot)
        };
        let start = opening
            .stacks
            .iter()
            .find(|s| s.id == stack.id && s.creature == stack.creature)
            .map_or(stack.base_count, |s| s.count);
        stacks[usize::from(stack.side == Wire::Defender)].push(StackRecord {
            origin,
            creature: CreatureTypeId::from_raw(stack.creature),
            start,
            end: if stack.alive { stack.count } else { 0 },
        });
    }
    BattleOutcome {
        result,
        survivors: [Army::default(), Army::default()],
        war_machines: [Vec::new(), Vec::new()],
        stacks,
        surrender_cost: None,
        heroes: [None, None],
        rounds: end.round,
    }
}
