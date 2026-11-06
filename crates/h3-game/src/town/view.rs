//! What a viewer sees of the towns (TWN-VEW-001 to -003; E09 T8c1, the town epic's design note
//! §12 and §17; survey `E09-towns.md` §12, the original's map picture, hint and right-click
//! popup): every town a tile of which the viewer has explored, linked from its object on the
//! adventure map ([`h3_adventure::ObjectView::town`]), with its public facts, its garrison by the
//! viewer's level and, for its owner's team or a viewer whose hero sees it by a Rogue's Visions
//! (E09 T8c2), the facts of the popup's top level (`docs/06-security.md`, "Towns").

use alloc::string::String;
use alloc::vec::Vec;

use h3_adventure::{AdventureObservation, AdventureState, Pos, allied};
use h3_content::{CreatureTypeId, FactionId, ResourceId};
use h3_contracts::{ArmyStack, FortLevel, PlayerId, TownId};
use h3_towns::{TownRuleset, Towns};

use crate::{GameRuleset, GameState};

/// The garrison's level of a viewer of the town's owner's team or whose hero sees it by Visions
/// (TWN-VEW-003): the popup's top level, the stacks' counts and [`TownView::owner_facts`]; the
/// garrison by slot ([`GarrisonView::army`]) is the owner's team's only.
pub const OWNER_LEVEL: u8 = 3;

/// The most a viewer's taverns give it (TWN-VEW-002): the original counts the viewer's towns with
/// a Tavern up to two.
pub const TAVERN_LEVELS: u8 = 2;

/// The level from which a viewer sees each stack's size word (TWN-VEW-002).
const WORDS_LEVEL: u8 = 2;

/// A town as a viewer sees it (TWN-VEW-001 to -003; E09 T8c1): what the original shows another
/// player on the map, in the hint and in the right-click popup, and its owner's team more. Named
/// by its `TownId` (ADR-0028 point 3), the object's [`h3_adventure::ObjectView::town`]. Never its
/// other buildings, the ever-built set, the defender's bonuses or the Escape Tunnel, nor a draw's
/// values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownView {
    /// The town.
    pub town: TownId,
    /// Its object's anchor.
    pub anchor: Pos,
    /// Its owner (its flag), none for a neutral town.
    pub owner: Option<PlayerId>,
    /// Its faction (its picture and hint).
    pub faction: FactionId,
    /// Its name as the map gives it (the hint); none for a town the map leaves unnamed, whose
    /// name is the client's (`h3_towns::TownStatic::name`).
    pub name: Option<String>,
    /// Its fort level (the popup's, a finer fact than the picture's fort).
    pub fort: FortLevel,
    /// Whether it has a Capitol (its picture).
    pub capitol: bool,
    /// Its garrison as the viewer's level shows it.
    pub garrison: GarrisonView,
    /// The popup's top level's facts, for a viewer at [`OWNER_LEVEL`] (the owner's team, or a
    /// hero's Visions in reach); none for every other viewer.
    pub owner_facts: Option<TownOwnerView>,
}

/// A town's garrison as a viewer sees it (TWN-VEW-002, -003; survey `E09-towns.md` §12.3,
/// `quick_view_army` 0x530990): by the viewer's level, its stacks packed in slot order (how many
/// and of which creatures, never their slots) from level 1, each stack's size word from level 2,
/// its count at [`OWNER_LEVEL`]; nothing at level 0, not even whether it is empty. The visiting
/// hero is not in it: it stands on the entrance and is seen as a hero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GarrisonView {
    /// The viewer's level, 0 to [`OWNER_LEVEL`]: [`OWNER_LEVEL`] for the owner's team or a
    /// viewer one of whose heroes sees the town by Visions, else the number of the viewer's towns
    /// with a standing Tavern, at most [`TAVERN_LEVELS`].
    pub level: u8,
    /// The stacks as the level shows them, in slot order; none at level 0.
    pub stacks: Vec<GarrisonStackView>,
    /// The garrison by slot, for a viewer of the owner's team only (the town screen's, which the
    /// garrison's `StackChanged` events change, E09 T8b2b); none for every other viewer.
    pub army: Option<Vec<ArmyStack>>,
}

/// A garrison's stack as a viewer's level shows it ([`GarrisonView`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GarrisonStackView {
    /// The creature (from level 1).
    pub creature: CreatureTypeId,
    /// The word of its count (from level 2; `h3_adventure::MonsterParams::count_word`, E08's
    /// monster words, the original's `creature_quantity_word` 0x44ADE0).
    pub count_word: Option<u8>,
    /// Its count (at [`OWNER_LEVEL`]).
    pub count: Option<u32>,
}

/// The facts of a town the popup shows only at its top level (TWN-VEW-003; survey
/// `E09-towns.md` §12.3): to its owner's team and to a viewer whose hero sees it by Visions. The
/// garrison hero's mark comes with the garrison hero (increment 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownOwnerView {
    /// Its hall level, 1 the Village Hall to 4 the Capitol (`h3_towns::hall_level`).
    pub hall: u8,
    /// Its day's income, its gold and its silo's resources, by resource in the core's order
    /// (`h3_towns::daily_income`).
    pub income: Vec<(ResourceId, u32)>,
}

/// What the viewer of `observation`, the adventure map of `state` as it sees it
/// ([`AdventureState::observe`]), sees of the towns (TWN-VEW-001 to -003): a view of each town
/// among its objects a tile of which it has explored, in `TownId` order (the objects' order). A
/// town only on its fringe, which the `faithful` profile shows as an object (ADV-QUIRK-1), has
/// none: the original's popup reads an explored tile only (survey `E09-towns.md` §12.3; *agent's
/// call*). Its garrison's level is [`OWNER_LEVEL`] for a town of the viewer's team (a neutral town
/// is nobody's) or one whose entrance a hero of the viewer's sees by Visions ([`visions`]), else
/// the number of the viewer's own towns with a standing Tavern (`h3_towns::has_tavern`), at most
/// [`TAVERN_LEVELS`]. The garrison by slot is the owner's team's only.
// rule: TWN-VEW-001, TWN-VEW-002, TWN-VEW-003
pub(crate) fn town_views(
    state: &GameState,
    rules: &GameRuleset,
    observation: &AdventureObservation,
) -> Vec<TownView> {
    let (adventure, towns) = (state.adventure(), state.towns());
    let (part, map, viewer) = (rules.towns(), adventure.map(), observation.viewer);
    let (mut taverns, mut seers) = (None, None);
    observation
        .objects
        .iter()
        .filter_map(|object| {
            let town = object.town?;
            // rule: TWN-VEW-001
            let (index, record) =
                map.town_object(town)
                    .zip(towns.get(town))
                    .filter(|&(index, _)| {
                        map.objects()[index]
                            .tiles(map)
                            .any(|pos| observation.is_explored(pos))
                    })?;
            let ours = object.owner.is_some_and(|owner| allied(owner, viewer));
            // rule: TWN-VEW-003
            let top = ours
                || map.objects()[index].entrance(map).is_some_and(|entrance| {
                    let seers = seers.get_or_insert_with(|| visions(state, rules, viewer));
                    in_visions(seers, entrance)
                });
            // rule: TWN-VEW-002, TWN-VEW-003
            let level = if top {
                OWNER_LEVEL
            } else {
                *taverns.get_or_insert_with(|| tavern_level(adventure, towns, part, viewer))
            };
            let army = adventure.town_garrison(index).unwrap_or_default();
            Some(TownView {
                town,
                anchor: object.anchor,
                owner: object.owner,
                faction: record.faction(),
                name: towns.town_static(town).and_then(|s| s.name.clone()),
                fort: h3_towns::fort_level(record, part),
                capitol: towns.capitol(town, part),
                garrison: garrison_view(rules, army, level, ours),
                // rule: TWN-VEW-003
                owner_facts: top.then(|| TownOwnerView {
                    hall: h3_towns::hall_level(record, part),
                    income: h3_towns::daily_income(record, part),
                }),
            })
        })
        .collect()
}

/// The tiles of `viewer`'s heroes that see towns by Visions, each with the square of its radius
/// (TWN-VEW-003; survey `E09-towns.md` §12.3): the heroes on the map whose army holds a creature
/// of the town part's Visions creatures (the original's `hero_visions_level` 0x4E5DE0, raised to
/// expert by a Rogue; Visions itself is not cast yet), the radius the part's Visions factor times
/// the hero's Power (`h3_heroes::power`), at least the part's least radius
/// (`hero_visions_reach` 0x4E5E10). Only the viewer's own heroes, as the original's popup asks.
// rule: TWN-VEW-003
fn visions(state: &GameState, rules: &GameRuleset, viewer: PlayerId) -> Vec<(Pos, u64)> {
    let heroes = state.adventure().heroes();
    rules
        .towns()
        .params()
        .into_iter()
        .flat_map(|params| {
            heroes
                .iter()
                .filter(move |hero| {
                    hero.owner == viewer
                        && hero
                            .army
                            .iter()
                            .any(|stack| params.visions_creatures.contains(&stack.creature))
                })
                .map(move |hero| {
                    let power = state
                        .hero(hero.id)
                        .map_or(0, |details| h3_heroes::power(details, rules.core()));
                    let radius = (u64::from(params.visions_factor) * u64::from(power))
                        .max(u64::from(params.visions_min_radius));
                    (hero.pos, radius * radius)
                })
        })
        .collect()
}

/// Whether one of `seers` ([`visions`]) sees the town whose entrance is `entrance` (TWN-VEW-003;
/// `hero_visions_reach` 0x4E5E10, from the town's entrance as `town_popup` 0x4167A0 passes it):
/// on its level, the squared distance below the squared radius.
// rule: TWN-VEW-003
pub(super) fn in_visions(seers: &[(Pos, u64)], entrance: Pos) -> bool {
    seers.iter().any(|&(pos, reach)| {
        let dx = u64::from(pos.x.abs_diff(entrance.x));
        let dy = u64::from(pos.y.abs_diff(entrance.y));
        pos.level == entrance.level && dx * dx + dy * dy < reach
    })
}

/// The number of `viewer`'s own towns with a standing Tavern, at most [`TAVERN_LEVELS`]
/// (TWN-VEW-002; survey `E09-towns.md` §12.3, `player_tavern_count` 0x4CCE30).
// rule: TWN-VEW-002
fn tavern_level(
    adventure: &AdventureState,
    towns: &Towns,
    part: &TownRuleset,
    viewer: PlayerId,
) -> u8 {
    let count = adventure
        .map()
        .towns()
        .iter()
        .zip((0..=u16::MAX).map(TownId::new))
        .filter(|&(&index, _)| adventure.owner(index) == Some(viewer))
        .filter_map(|(_, town)| towns.get(town))
        .filter(|town| h3_towns::has_tavern(town, part))
        .take(usize::from(TAVERN_LEVELS))
        .count();
    u8::try_from(count).unwrap_or(TAVERN_LEVELS)
}

/// The garrison `army` (by slot) as `level` shows it, its slots to the owner's team (`ours`).
// rule: TWN-VEW-002, TWN-VEW-003
fn garrison_view(rules: &GameRuleset, army: &[ArmyStack], level: u8, ours: bool) -> GarrisonView {
    let words = rules.adventure().params().monsters();
    let stacks = if level == 0 {
        Vec::new()
    } else {
        army.iter()
            .map(|stack| GarrisonStackView {
                creature: stack.creature,
                count_word: (level >= WORDS_LEVEL)
                    .then(|| words.map_or(0, |w| w.count_word(stack.count))),
                count: (level >= OWNER_LEVEL).then_some(stack.count),
            })
            .collect()
    };
    GarrisonView {
        level,
        stacks,
        army: ours.then(|| army.to_vec()),
    }
}
