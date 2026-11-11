//! What a viewer sees of the towns (TWN-VEW-001 to -003; E09 T8c1, the town epic's design note
//! §12 and §17; survey `E09-towns.md` §12, the original's map picture, hint and right-click
//! popup): every town a tile of which the viewer has explored, linked from its object on the
//! adventure map ([`h3_adventure::ObjectView::town`]), with its public facts, its garrison by the
//! viewer's level and, for its owner's team or a viewer whose hero sees it by a Rogue's Visions
//! (E09 T8c2), the facts of the popup's top level (`docs/06-security.md`, "Towns"). Since E09
//! T18b2a the day's mark to every viewer and the buildings to the owner (TWN-VEW-004), and each
//! viewer's changed views told ([`Told`], SRV-Q26: a server keeps a viewer's views by events).

use alloc::string::String;
use alloc::vec::Vec;

use h3_adventure::{AdventureObservation, AdventureState, Pos, allied};
use h3_content::{CreatureTypeId, FactionId, ResourceId};
use h3_contracts::{ArmyStack, FortLevel, PlayerId, TownId};
use h3_towns::{BuildingSet, TownRuleset, Towns};

use crate::command::{GameEvent, Sink};
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
/// buildings but to its owner (TWN-VEW-004), the defender's bonuses or the Escape Tunnel, nor a
/// draw's values. A change of it is told to the viewer ([`GameEvent::TownChanged`], E09 T18b2a).
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
    /// Whether it was built in today (TWN-VEW-004; E09 T18b2a): public, as the original's town
    /// list and popup frame a town's portrait at every level (`town_list_frame` 0x5BD700).
    pub built_today: bool,
    /// Its garrison as the viewer's level shows it.
    pub garrison: GarrisonView,
    /// The popup's top level's facts, for a viewer at [`OWNER_LEVEL`] (the owner's team, or a
    /// hero's Visions in reach); none for every other viewer.
    pub owner_facts: Option<TownOwnerView>,
    /// Its buildings, for its owner alone (TWN-VEW-004; E09 T18b2a): the original opens the town
    /// screen from its owner's town list at any time, an ally's only at a visit (E10's teams,
    /// `TWN-STUB-11`) and Visions' top level never (the popup shows no buildings).
    pub buildings: Option<TownBuildingsView>,
}

/// A town's buildings as its owner sees them (TWN-VEW-004; E09 T18b2a, the town epic's design
/// note §31), each set over its faction's town record: the standing ones derive from the first
/// with the part (`h3_towns::Town::standing`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TownBuildingsView {
    /// Every building it has had, the superseded ones too (`h3_towns::Town::built`).
    pub built: BuildingSet,
    /// The buildings the map forbids it (`h3_towns::Town::forbidden`).
    pub forbidden: BuildingSet,
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
/// ([`AdventureState::observe`]), sees of the towns (TWN-VEW-001 to -004): a view of each town
/// among its objects a tile of which it has explored, in `TownId` order (the objects' order). A
/// town only on its fringe, which the `faithful` profile shows as an object (ADV-QUIRK-1), has
/// none: the original's popup reads an explored tile only (survey `E09-towns.md` §12.3; *agent's
/// call*). Each view is [`Seer::view`]'s, as the told changes' ([`Told`]).
// rule: TWN-VEW-001, TWN-VEW-002, TWN-VEW-003, TWN-VEW-004
pub(crate) fn town_views(
    state: &GameState,
    rules: &GameRuleset,
    observation: &AdventureObservation,
) -> Vec<TownView> {
    let mut seer = Seer::new(state, rules, observation.viewer);
    observation
        .objects
        .iter()
        .filter_map(|object| object.town)
        .filter_map(|town| seer.view(town, |pos| observation.is_explored(pos)))
        .collect()
}

/// `viewer`'s views of `towns`, in their order, each a town a tile of which it has explored
/// (TWN-VEW-001 to -004; the others are left out): [`town_views`]'s for those towns alone, for
/// what a viewer newly sees (SRV-Q26 (a), `crate::reveal`; the adventure lane, `cross-lane`).
pub(crate) fn views_of(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
    towns: &[TownId],
) -> Vec<TownView> {
    let adventure = state.adventure();
    let mut seer = Seer::new(state, rules, viewer);
    towns
        .iter()
        .filter_map(|&town| seer.view(town, |pos| adventure.is_explored(viewer, pos)))
        .collect()
}

/// A viewer's views of the towns of a game, what they read of the viewer computed once and only
/// when a view needs it: its towns' Taverns and its heroes' Visions (TWN-VEW-002, -003).
struct Seer<'a> {
    state: &'a GameState,
    rules: &'a GameRuleset,
    viewer: PlayerId,
    taverns: Option<u8>,
    seers: Option<Vec<(Pos, u64)>>,
}

impl<'a> Seer<'a> {
    const fn new(state: &'a GameState, rules: &'a GameRuleset, viewer: PlayerId) -> Self {
        Self {
            state,
            rules,
            viewer,
            taverns: None,
            seers: None,
        }
    }

    /// The viewer's view of `town` if it has explored a tile of it (`explored`), else none
    /// (TWN-VEW-001). Its garrison's level is [`OWNER_LEVEL`] for a town of the viewer's team (a
    /// neutral town is nobody's) or one whose entrance a hero of the viewer's sees by Visions
    /// ([`visions`]), else the number of the viewer's own towns with a standing Tavern
    /// (`h3_towns::has_tavern`), at most [`TAVERN_LEVELS`]; the garrison by slot is the owner's
    /// team's only; the buildings the owner's alone (TWN-VEW-004).
    // rule: TWN-VEW-001, TWN-VEW-002, TWN-VEW-003, TWN-VEW-004
    fn view(&mut self, town: TownId, explored: impl Fn(Pos) -> bool) -> Option<TownView> {
        let (state, rules, viewer) = (self.state, self.rules, self.viewer);
        let (adventure, towns) = (state.adventure(), state.towns());
        let (part, map) = (rules.towns(), adventure.map());
        // rule: TWN-VEW-001
        let (index, record) = map
            .town_object(town)
            .zip(towns.get(town))
            .filter(|&(index, _)| map.objects()[index].tiles(map).any(&explored))?;
        let object = &map.objects()[index];
        let owner = adventure.owner(index);
        let ours = owner.is_some_and(|owner| allied(owner, viewer));
        // rule: TWN-VEW-003
        let top = ours
            || object.entrance(map).is_some_and(|entrance| {
                let seers = self
                    .seers
                    .get_or_insert_with(|| visions(state, rules, viewer));
                in_visions(seers, entrance)
            });
        // rule: TWN-VEW-002, TWN-VEW-003
        let level = if top {
            OWNER_LEVEL
        } else {
            *self
                .taverns
                .get_or_insert_with(|| tavern_level(adventure, towns, part, viewer))
        };
        let army = adventure.town_garrison(index).unwrap_or_default();
        Some(TownView {
            town,
            anchor: object.anchor,
            owner,
            faction: record.faction(),
            name: towns.town_static(town).and_then(|s| s.name.clone()),
            fort: h3_towns::fort_level(record, part),
            capitol: towns.capitol(town, part),
            // rule: TWN-VEW-004
            built_today: record.built_today(),
            garrison: garrison_view(rules, army, level, ours),
            // rule: TWN-VEW-003
            owner_facts: top.then(|| TownOwnerView {
                hall: h3_towns::hall_level(record, part),
                income: h3_towns::daily_income(record, part),
            }),
            // rule: TWN-VEW-004
            buildings: (owner == Some(viewer)).then(|| TownBuildingsView {
                built: record.built(),
                forbidden: record.forbidden(),
            }),
        })
    }
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

/// Each viewer's views of some towns before a change, kept to tell each viewer the views the
/// change made (E09 T18b2a; SRV-Q26: a server keeps a viewer's views current by events alone, the
/// server lane's design note `E16-design.md` §14). A change that may move a town's view (a build,
/// a capture, a siege's merge, a garrison's battle, the day's start) takes the views before it
/// ([`Self::before`]) and tells the changed ones after it ([`Self::tell`]). Not told: a view
/// that Visions' reach changes (`TWN-STUB-13`), and a town the viewer comes to see by exploring
/// (SRV-Q26's query for newly seen tiles).
#[derive(Debug, Default)]
pub(crate) struct Told(Vec<(PlayerId, Vec<Seen>)>);

/// A town and a viewer's view of it before the change (none: it did not see it).
type Seen = (TownId, Option<TownView>);

impl Told {
    /// The views of `towns` by every player of the game, and of every town by each player of
    /// `every` (one whose Taverns the change may count, TWN-VEW-002: a Tavern's build, a capture's
    /// two owners), in player order, each player's in `TownId` order. Nothing for a sink that
    /// keeps nothing ([`Sink::RECORDS`]): nobody reads what it would tell.
    pub(crate) fn before<S: Sink>(
        state: &GameState,
        rules: &GameRuleset,
        towns: &[TownId],
        every: &[PlayerId],
    ) -> Self {
        if !S::RECORDS {
            return Self::default();
        }
        let adventure = state.adventure();
        let count = adventure.map().towns().len();
        let viewers = (0..state.players()).filter_map(|p| u8::try_from(p).ok().map(PlayerId::new));
        Self(
            viewers
                .map(|viewer| {
                    let all = every.contains(&viewer);
                    let mut seer = Seer::new(state, rules, viewer);
                    let views = (0..count)
                        .filter_map(|t| u16::try_from(t).ok().map(TownId::new))
                        .filter(|town| all || towns.contains(town))
                        .map(|town| {
                            let explored = |pos| adventure.is_explored(viewer, pos);
                            (town, seer.view(town, explored))
                        })
                        .collect();
                    (viewer, views)
                })
                .collect(),
        )
    }

    /// Tells each viewer of [`Self::before`] its view of a town that the change made, where it
    /// differs from the one before and the viewer sees the town ([`tell`]), in the same order.
    pub(crate) fn tell<S: Sink>(self, state: &GameState, rules: &GameRuleset, sink: &mut S) {
        // stub: TWN-STUB-13 (a change of Visions' reach, a hero's step or army or Power, is not
        // told: only these changes are)
        let adventure = state.adventure();
        for (viewer, views) in self.0 {
            let mut seer = Seer::new(state, rules, viewer);
            for (town, before) in views {
                let explored = |pos| adventure.is_explored(viewer, pos);
                let after = seer.view(town, explored);
                if let Some(view) = after.filter(|view| before.as_ref() != Some(view)) {
                    tell(viewer, &view, sink);
                }
            }
        }
    }
}

/// Tells `viewer` its view `view` of a town (TWN-VEW-001 to -004; E09 T18b2a): a
/// [`GameEvent::TownChanged`] with its facts, then a [`GameEvent::TownStackSeen`] for each stack
/// its garrison's level shows, in slot order, then a [`GameEvent::TownIncomeSeen`] for each line
/// of the top level's income, in the core's order (`GameEvent` stays `Copy`, as an army's
/// `StackChanged`: *agent's call*). The viewer's view after them is the told one, its anchor,
/// faction and name as they were.
fn tell<S: Sink>(viewer: PlayerId, view: &TownView, sink: &mut S) {
    let (player, town) = (viewer, view.town);
    let army = view.garrison.army.as_deref();
    sink.emit(GameEvent::TownChanged {
        player,
        town,
        owner: view.owner,
        fort: view.fort,
        capitol: view.capitol,
        built_today: view.built_today,
        level: view.garrison.level,
        slots: army.is_some(),
        hall: view.owner_facts.as_ref().map(|facts| facts.hall),
        buildings: view.buildings,
    });
    for (index, &stack) in view.garrison.stacks.iter().enumerate() {
        let slot = army
            .and_then(|army| army.get(index))
            .map(|stack| stack.slot);
        sink.emit(GameEvent::TownStackSeen {
            player,
            town,
            stack,
            slot,
        });
    }
    let income = view.owner_facts.iter().flat_map(|facts| &facts.income);
    for &(resource, amount) in income {
        sink.emit(GameEvent::TownIncomeSeen {
            player,
            town,
            resource,
            amount,
        });
    }
}
