//! What a viewer sees of the map's garrisons (ADV-GAR-006; E08 T114b; increment 12's design note
//! §10 and §15; survey `E08-garrisons-bonuses.md` §8, the original's right-click popup
//! `garrison_popup` 0x416F80): every map garrison a tile of which the viewer has explored, by type
//! and anchor (its object among the adventure map's), with its owner, its army by the viewer's
//! level as a town's garrison (TWN-VEW-002, TWN-VEW-003: the same levels, the same
//! [`GarrisonView`]) and, for its owner's team, its removable flag; each viewer's changed views
//! told ([`Told`], SRV-Q26: a server keeps a viewer's views by events).

use alloc::vec::Vec;

use h3_adventure::{BitGrid, ObjectTypeId, Pos, allied};
use h3_contracts::PlayerId;

use crate::command::{GameEvent, Sink};
use crate::town::view::{
    GarrisonView, OWNER_LEVEL, garrison_view, in_visions, tavern_level, visions,
};
use crate::{GameRuleset, GameState};

/// A map garrison as a viewer sees it (ADV-GAR-006; E08 T114b; increment 12's design note §10):
/// its type (the anti-magic flag in it: the picture), its owner (the flag), its army at the
/// viewer's level and, for its owner's team, whether the owner may take its troops back. Named by
/// type and anchor (ADR-0016 point 6: never by index), its object among the adventure map's. A
/// change of it is told to the viewer ([`GameEvent::GarrisonChanged`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapGarrisonView {
    /// Its object type.
    pub object_type: ObjectTypeId,
    /// Its object's anchor.
    pub anchor: Pos,
    /// Its owner (its flag), none for a neutral garrison.
    pub owner: Option<PlayerId>,
    /// Its army as the viewer's level shows it: [`OWNER_LEVEL`] for the owner's team (a neutral
    /// garrison is nobody's) or a viewer one of whose heroes sees its entrance by Visions, else
    /// the viewer's towns with a standing Tavern, at most two; by slot to the owner's team only.
    pub garrison: GarrisonView,
    /// Its removable flag (ADV-GAR-001, ADV-GAR-003), for a viewer of its owner's team only: the
    /// original shows it in the exchange, to the visitor (survey §8); none for every other viewer.
    pub removable: Option<bool>,
}

/// The views of the map garrisons a tile of which `viewer` has explored, in object order
/// (ADV-GAR-006): each as [`Seer::view`] gives it. A garrison only on the viewer's fringe, which
/// the `faithful` profile shows as an object (ADV-QUIRK-1), has none, as a town's (TWN-VEW-001:
/// the popup reads an explored tile only).
// rule: ADV-GAR-006
pub(crate) fn garrison_views(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
) -> Vec<MapGarrisonView> {
    let adventure = state.adventure();
    let mut seer = Seer::new(state, rules, viewer);
    garrisons(state)
        .filter_map(|index| seer.view(index, |pos| adventure.is_explored(viewer, pos)))
        .collect()
}

/// `viewer`'s views of the map garrisons a tile of which it has explored now and none through
/// `since`, an earlier explored grid of its own (SRV-Q26 (a), `crate::reveal`), in object order:
/// [`garrison_views`]' for those garrisons alone. A grid of another map's size counts as none
/// explored, as the adventure's reveal reads it.
// rule: ADV-GAR-006
pub(crate) fn revealed(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
    since: &BitGrid,
) -> Vec<MapGarrisonView> {
    let adventure = state.adventure();
    let map = adventure.map();
    let fits = adventure.explored(viewer).is_some_and(|now| {
        (now.width(), now.height(), now.levels()) == (since.width(), since.height(), since.levels())
    });
    let mut seer = Seer::new(state, rules, viewer);
    garrisons(state)
        .filter(|&index| !fits || !map.objects()[index].tiles(map).any(|pos| since.get(pos)))
        .filter_map(|index| seer.view(index, |pos| adventure.is_explored(viewer, pos)))
        .collect()
}

/// The object indices of the map's garrisons, in object order: the ownable objects holding a
/// garrison (`h3_adventure::Holds::Garrison`, whose removable flag is answered). The map's own
/// objects only: none is added in play.
fn garrisons(state: &GameState) -> impl Iterator<Item = usize> + '_ {
    let adventure = state.adventure();
    adventure
        .map()
        .ownables()
        .iter()
        .copied()
        .filter(move |&index| adventure.garrison_removable(index).is_some())
}

/// A viewer's views of the map garrisons, what they read of the viewer computed once and only
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

    /// The viewer's view of the map garrison at object index `index` if it has explored a tile
    /// of it (`explored`), else none (ADV-GAR-006; survey §8, `garrison_popup` 0x416F80): its
    /// level is [`OWNER_LEVEL`] for a garrison of the viewer's team or one whose entrance a hero
    /// of the viewer's sees by Visions (`hero_visions_reach` 0x4E5E10, from the entrance), else
    /// the number of the viewer's own towns with a standing Tavern, at most two (TWN-VEW-002);
    /// its army by slot and its removable flag the owner's team's only.
    // rule: ADV-GAR-006, TWN-VEW-002, TWN-VEW-003
    fn view(&mut self, index: usize, explored: impl Fn(Pos) -> bool) -> Option<MapGarrisonView> {
        let (state, rules, viewer) = (self.state, self.rules, self.viewer);
        let adventure = state.adventure();
        let map = adventure.map();
        let object = &map.objects()[index];
        let (object_type, removable) = object
            .object_type
            .zip(adventure.garrison_removable(index))
            .filter(|_| object.tiles(map).any(&explored))?;
        let owner = adventure.owner(index);
        let ours = owner.is_some_and(|owner| allied(owner, viewer));
        let top = ours
            || object.entrance(map).is_some_and(|entrance| {
                let seers = self
                    .seers
                    .get_or_insert_with(|| visions(state, rules, viewer));
                in_visions(seers, entrance)
            });
        let level = if top {
            OWNER_LEVEL
        } else {
            *self.taverns.get_or_insert_with(|| {
                tavern_level(adventure, state.towns(), rules.towns(), viewer)
            })
        };
        Some(MapGarrisonView {
            object_type,
            anchor: object.anchor,
            owner,
            garrison: garrison_view(rules, adventure.garrison_army(index), level, ours),
            removable: ours.then_some(removable),
        })
    }
}

/// Each viewer's views of some map garrisons before a change, kept to tell each viewer the views
/// the change made (ADV-GAR-006; E08 T114b; increment 12's design note §15; SRV-Q26: a server
/// keeps a viewer's views current by events alone), as the towns' (`crate::town::Told`, which
/// holds one: the same triggers of a viewer's level, one function for both). A change that may
/// move a garrison's view (its battle's end, an exchange's command, a Tavern's build or a town's
/// capture, which move its viewers' levels) takes the views before it ([`Self::before`]) and
/// tells the changed ones after it ([`Self::tell`]); a capture, which changes the garrison's owner
/// for every viewer, tells each viewer's view after it ([`tell_all`]). Not told: a view that
/// Visions' reach changes (`TWN-STUB-13`, as a town's), and a garrison the viewer comes to see by
/// exploring ([`revealed`]).
#[derive(Debug, Default)]
pub(crate) struct Told(Vec<(PlayerId, Vec<Seen>)>);

/// A map garrison by object index and a viewer's view of it before the change (none: it did not
/// see it).
type Seen = (usize, Option<MapGarrisonView>);

impl Told {
    /// The views of the garrisons at object indices `garrisons` by every player of the game, and
    /// of every garrison by each player of `every` (one whose Taverns the change may count,
    /// TWN-VEW-002), in player order, each player's in object order. Nothing for a sink that keeps
    /// nothing ([`Sink::RECORDS`]), nor when the change names no garrison and no player.
    pub(crate) fn before<S: Sink>(
        state: &GameState,
        rules: &GameRuleset,
        garrisons: &[usize],
        every: &[PlayerId],
    ) -> Self {
        if !S::RECORDS || (garrisons.is_empty() && every.is_empty()) {
            return Self::default();
        }
        let adventure = state.adventure();
        let viewers = (0..state.players()).filter_map(|p| u8::try_from(p).ok().map(PlayerId::new));
        Self(
            viewers
                .map(|viewer| {
                    let all = every.contains(&viewer);
                    let mut seer = Seer::new(state, rules, viewer);
                    let views = self::garrisons(state)
                        .filter(|index| all || garrisons.contains(index))
                        .map(|index| {
                            let explored = |pos| adventure.is_explored(viewer, pos);
                            (index, seer.view(index, explored))
                        })
                        .collect();
                    (viewer, views)
                })
                .collect(),
        )
    }

    /// Tells each viewer of [`Self::before`] its view of a garrison that the change made, where
    /// it differs from the one before and the viewer sees the garrison ([`tell`]), in the same
    /// order.
    pub(crate) fn tell<S: Sink>(self, state: &GameState, rules: &GameRuleset, sink: &mut S) {
        // stub: TWN-STUB-13 (a change of Visions' reach, a hero's step or army or Power, is not
        // told for a map garrison either: only these changes are)
        let adventure = state.adventure();
        for (viewer, views) in self.0 {
            let mut seer = Seer::new(state, rules, viewer);
            for (index, before) in views {
                let explored = |pos| adventure.is_explored(viewer, pos);
                let after = seer.view(index, explored);
                if let Some(view) = after.filter(|view| before.as_ref() != Some(view)) {
                    tell(viewer, &view, sink);
                }
            }
        }
    }
}

/// Tells every player of the game that sees the map garrison at object index `index` its view of
/// it ([`tell`]), in player order: after its capture (ADV-GAR-002, ADV-OWN-001: an empty one's
/// visit, a won battle's flag), which changed its owner, so every viewer's view of it. Nothing for
/// a sink that keeps nothing ([`Sink::RECORDS`]).
// rule: ADV-GAR-006
pub(crate) fn tell_all<S: Sink>(
    state: &GameState,
    rules: &GameRuleset,
    index: usize,
    sink: &mut S,
) {
    let adventure = state.adventure();
    let viewers = (0..state.players())
        .filter(|_| S::RECORDS)
        .filter_map(|p| u8::try_from(p).ok().map(PlayerId::new));
    for viewer in viewers {
        let explored = |pos| adventure.is_explored(viewer, pos);
        if let Some(view) = Seer::new(state, rules, viewer).view(index, explored) {
            tell(viewer, &view, sink);
        }
    }
}

/// Tells `viewer` its view `view` of a map garrison (ADV-GAR-006; E08 T114b): a
/// [`GameEvent::GarrisonChanged`] with its facts, then a [`GameEvent::GarrisonStackSeen`] for each
/// stack its level shows, in slot order (`GameEvent` stays `Copy`, as the towns'). The viewer's
/// view after them is the told one.
fn tell<S: Sink>(viewer: PlayerId, view: &MapGarrisonView, sink: &mut S) {
    let (player, object_type, anchor) = (viewer, view.object_type, view.anchor);
    let army = view.garrison.army.as_deref();
    sink.emit(GameEvent::GarrisonChanged {
        player,
        object_type,
        anchor,
        owner: view.owner,
        level: view.garrison.level,
        slots: army.is_some(),
        removable: view.removable,
    });
    for (index, &stack) in view.garrison.stacks.iter().enumerate() {
        let slot = army
            .and_then(|army| army.get(index))
            .map(|stack| stack.slot);
        sink.emit(GameEvent::GarrisonStackSeen {
            player,
            object_type,
            anchor,
            stack,
            slot,
        });
    }
}
