//! The Thieves' Guild (E09 T39b1, design note `E09-design.md` §73; TWN-TAV-007 to -009; survey
//! `E09-tavern.md` §6, §13): what the original's window (`thieves_guild_fill` 0x5DDA10) shows a
//! player of every player in the game: the rows its level shows, each ranking the players by a
//! value as places (never the values), and from its level each player's best hero, that hero's
//! primary skills and its best creature. A query: nothing is stored, the window is a snapshot when
//! it opens. Its level is the viewer's towns with a Tavern ([`thieves_guild`]); a Den of Thieves
//! shows everything ([`DEN_LEVEL`]; its visit is E09 T39b2's, `TWN-STUB-20`).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::ops::Bound::{Excluded, Unbounded};

use h3_adventure::HeroOnMap;
use h3_content::{CreatureTypeId, GUILD_ARTIFACTS_BY_BACKPACKS, HeroTypeId, Ruleset};
use h3_contracts::{ArmyStack, PlayerId};
use h3_heroes::HeroDetails;

use super::view::tavern_count;
use crate::{GameRuleset, GameState};

/// The Thieves' Guild's highest level (TWN-TAV-007; `player_tavern_count` 0x4CCE30's count, whose
/// rows stop at five Taverns): the viewer's towns with a standing Tavern are counted to it.
pub const GUILD_LEVELS: u8 = 5;

/// The level a Den of Thieves' visit shows: everything (TWN-TAV-007; `visit_object`'s case 97
/// passes 99).
pub const DEN_LEVEL: u8 = GUILD_LEVELS;

/// The highest primary skill the guild shows (TWN-TAV-009): each shown clamped to 0–99.
const SKILL_SHOWN: u8 = 99;

/// A row of the Thieves' Guild (TWN-TAV-008), in the original's order (`thieves_guild_row_value`
/// 0x5DEE70's cases 0 to 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuildRow {
    /// The towns the player owns.
    Towns,
    /// Its heroes on the map (a garrison hero is not).
    Heroes,
    /// Its treasury's resources of the town part's first guild row (the original's gold).
    Gold,
    /// Those of the second (the original's wood and ore).
    Common,
    /// Those of the third (the original's mercury, sulfur, crystal and gems).
    Rare,
    /// The obelisks it visited: 0 for every player until the obelisks are placed (`TWN-STUB-17`).
    Obelisks,
    /// Its heroes' artifacts on the map: the backpacks' alone under `TWN-QUIRK-13`.
    Artifacts,
    /// Its army strength: its heroes' powers and its towns' shown armies (TD-23).
    Army,
    /// Its gold of a day's start as it stands now.
    Income,
}

impl GuildRow {
    /// Every row, in the original's order.
    pub const ALL: [Self; 9] = [
        Self::Towns,
        Self::Heroes,
        Self::Gold,
        Self::Common,
        Self::Rare,
        Self::Obelisks,
        Self::Artifacts,
        Self::Army,
        Self::Income,
    ];
}

/// A row as the guild shows it (TWN-TAV-008): its kind and the players' places.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildRanking {
    /// The row.
    pub row: GuildRow,
    /// Each player's place, by player id, 0 the first: the players by the row's value from the
    /// highest, equal values sharing one place, the next lower value the next place (the
    /// original's columns, 0x5DDE9F to 0x5DDEDF).
    pub places: Vec<u8>,
}

/// A player's best hero as the guild shows it (TWN-TAV-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuildHero {
    /// Its type (the original shows its portrait).
    pub hero_type: HeroTypeId,
    /// From level 2 its four primary skills (attack, defense, power, knowledge) as the adventure
    /// map reads them (`h3_heroes::primary_skills`), each clamped to 0–99; none below.
    pub skills: Option<[u8; 4]>,
}

/// What the guild shows of one player beside the rows (TWN-TAV-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuildPlayer {
    /// From level 1 its best hero: over its heroes on the map in the adventure's order the
    /// greatest power (as the army row's), a strictly greater one replacing; none below level 1,
    /// without a hero or with every power 0 (TD-23).
    pub hero: Option<GuildHero>,
    /// From level 4 its best creature: over the armies its towns show (in town order) and then
    /// its heroes' on the map (in the same order), the creature of the greatest AI value, a
    /// strictly greater one replacing; none below level 4, without a creature or with every value
    /// 0 (TD-23).
    pub creature: Option<CreatureTypeId>,
}

/// The Thieves' Guild as a viewer sees it (TWN-TAV-007 to -009): the same for every viewer of a
/// level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThievesGuildView {
    /// Its level, 0 to [`GUILD_LEVELS`].
    pub level: u8,
    /// The rows the level shows, in the original's order: 2 at 0 or 1, 4 at 2, 6 at 3, 8 at 4,
    /// all 9 from 5.
    pub rows: Vec<GuildRanking>,
    /// Every player of the game, by id (none is defeated before E10).
    pub players: Vec<GuildPlayer>,
}

/// The Thieves' Guild `viewer` may open now (TWN-TAV-007; E09 decision 119, design note §73):
/// while its turn runs and it owns a town with a standing Tavern (the original's tavern dialog's
/// button, 0x5D7BFF) or its open decision is a map Tavern's (GAM-DEC-010); otherwise none. Its
/// level is the viewer's towns with a standing Tavern, counted to [`GUILD_LEVELS`]: 0 at a map
/// Tavern without one.
// rule: TWN-TAV-007
pub fn thieves_guild(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
) -> Option<ThievesGuildView> {
    if !state.is_acting(viewer) {
        return None;
    }
    let (adventure, towns) = (state.adventure(), state.towns());
    let level = tavern_count(adventure, towns, rules.towns(), viewer, GUILD_LEVELS);
    (level > 0 || state.tavern(viewer).is_some()).then(|| thieves_guild_at(state, rules, level))
}

/// The Thieves' Guild at `level` (TWN-TAV-007 to -009; a Den of Thieves' at [`DEN_LEVEL`]): the
/// rows the level shows, each its places ([`GuildRanking`]), and every player's parts
/// ([`GuildPlayer`]). From level 3 the original also shows each player's tactic: not shown
/// (`TWN-STUB-18`).
// rule: TWN-TAV-007, TWN-TAV-008, TWN-TAV-009
pub(crate) fn thieves_guild_at(
    state: &GameState,
    rules: &GameRuleset,
    level: u8,
) -> ThievesGuildView {
    let players: Vec<PlayerId> = (0..=u8::MAX)
        .take(state.players())
        .map(PlayerId::new)
        .collect();
    let shown = match level {
        0 | 1 => 2,
        2 => 4,
        3 => 6,
        4 => 8,
        _ => GuildRow::ALL.len(),
    };
    let rows = GuildRow::ALL[..shown]
        .iter()
        .map(|&row| {
            let values: Vec<u64> = (players.iter())
                .map(|&player| value(state, rules, row, player))
                .collect();
            GuildRanking {
                row,
                places: places(&values),
            }
        })
        .collect();
    // stub: TWN-STUB-18 (the players' tactics from level 3: the computer's play, E13)
    let players = (players.iter())
        .map(|&player| GuildPlayer {
            hero: if level >= 1 {
                best_hero(state, rules, player, level)
            } else {
                None
            },
            creature: if level >= 4 {
                best_creature(state, rules, player)
            } else {
                None
            },
        })
        .collect();
    ThievesGuildView {
        level,
        rows,
        players,
    }
}

/// Each value's place (TWN-TAV-008): the number of distinct values above it.
pub(super) fn places(values: &[u64]) -> Vec<u8> {
    let distinct: BTreeSet<u64> = values.iter().copied().collect();
    (values.iter())
        .map(|&value| {
            let above = distinct.range((Excluded(value), Unbounded)).count();
            u8::try_from(above).unwrap_or(u8::MAX)
        })
        .collect()
}

/// `player`'s heroes on the map, in the adventure's order, with their details.
fn heroes<'a>(
    state: &'a GameState,
    player: PlayerId,
) -> impl Iterator<Item = (&'a HeroOnMap, &'a HeroDetails)> + 'a {
    (state.adventure().heroes().iter())
        .filter(move |hero| hero.owner == player)
        .filter_map(|hero| state.hero(hero.id).map(|details| (hero, details)))
}

/// A hero's power on the map (ADV-MON-007's, `hero_army_power` 0x427650).
fn power(core: &Ruleset, (hero, details): (&HeroOnMap, &HeroDetails)) -> u64 {
    h3_adventure::power(core, &hero.army, Some(details))
}

/// `player`'s value in `row` (TWN-TAV-008; `thieves_guild_row_value` 0x5DEE70), each sum
/// saturating (the original's 32-bit sums, which no game reaches).
// rule: TWN-TAV-008
pub(super) fn value(
    state: &GameState,
    rules: &GameRuleset,
    row: GuildRow,
    player: PlayerId,
) -> u64 {
    let (adventure, core) = (state.adventure(), rules.core());
    let params = rules.towns().params();
    let treasury = |index: usize| {
        let held = state.treasury(player).unwrap_or_default();
        let row = params.map_or(&[][..], |p| p.guild_resources[index].as_slice());
        (row.iter())
            .filter_map(|resource| held.get(resource.index()))
            .fold(0u64, |sum, &amount| sum.saturating_add(u64::from(amount)))
    };
    let count = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
    match row {
        GuildRow::Towns => count(
            (adventure.map().towns().iter())
                .filter(|&&index| adventure.owner(index) == Some(player))
                .count(),
        ),
        GuildRow::Heroes => count(heroes(state, player).count()),
        GuildRow::Gold => treasury(0),
        GuildRow::Common => treasury(1),
        GuildRow::Rare => treasury(2),
        // stub: TWN-STUB-17 (the obelisks visited: no obelisk is placed before increment 6)
        GuildRow::Obelisks => 0,
        // TWN-QUIRK-13: the backpacks alone with its flag on (`faithful`).
        GuildRow::Artifacts => {
            let backpacks = core.quirks().is_enabled(GUILD_ARTIFACTS_BY_BACKPACKS);
            heroes(state, player).fold(0u64, |sum, (_, details)| {
                let worn = if backpacks {
                    0
                } else {
                    details.worn.iter().flatten().count()
                };
                sum.saturating_add(count(details.backpack.len() + worn))
            })
        }
        GuildRow::Army => {
            let heroes = heroes(state, player).map(|hero| power(core, hero));
            let towns =
                (adventure.upper_bars(player)).map(|army| h3_adventure::army_value(core, army));
            heroes.chain(towns).fold(0u64, u64::saturating_add)
        }
        GuildRow::Income => params.map_or(0, |p| {
            let income = state.day_income(rules, player, false);
            income.get(p.gold.index()).copied().map_or(0, u64::from)
        }),
    }
}

/// `player`'s best hero at `level` (TWN-TAV-009, TD-23): the first of its heroes on the map of the
/// greatest power above 0, with its skills from level 2.
// rule: TWN-TAV-009
fn best_hero(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    level: u8,
) -> Option<GuildHero> {
    let core = rules.core();
    let (best, _) = heroes(state, player).fold((None, 0), |(best, most), hero| {
        let power = power(core, hero);
        if power > most {
            (Some(hero.1), power)
        } else {
            (best, most)
        }
    });
    best.map(|details| GuildHero {
        hero_type: details.hero_type,
        skills: (level >= 2)
            .then(|| h3_heroes::primary_skills(details, core).map(|skill| skill.min(SKILL_SHOWN))),
    })
}

/// `player`'s best creature (TWN-TAV-009, TD-23): over its towns' shown armies, then its heroes'
/// on the map, the first stack's creature of the greatest AI value above 0.
// rule: TWN-TAV-009
fn best_creature(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
) -> Option<CreatureTypeId> {
    let core = rules.core();
    let towns = state.adventure().upper_bars(player);
    let armies = towns.chain(heroes(state, player).map(|(hero, _)| &hero.army[..]));
    let stacks = armies.flat_map(<[ArmyStack]>::iter).filter(|s| s.count > 0);
    let (best, _) = stacks.fold((None, 0), |(best, most), stack| {
        let value = core
            .creature(stack.creature)
            .map_or(0, |info| info.ai_value);
        if value > most {
            (Some(stack.creature), value)
        } else {
            (best, most)
        }
    });
    best
}
