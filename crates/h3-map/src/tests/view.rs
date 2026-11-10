//! What a viewer may learn on imported maps (E08 T8b2b; `docs/05-testing.md`, "Property tests";
//! `docs/06-security.md`, "Information hiding"), on T8b2a's playable maps and driver
//! ([`super::play`]): observation hygiene (what a viewer is shown or told does not depend on what
//! it cannot see) and the observation's consistency with the events it is told.

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, GuardsView, ObjectKind, ObjectView,
    OfferKind, Pos as TilePos, TileMask, TileView, Viewers, allied,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_game::{
    Actor, ArmyRef, DecisionView, GameCommand, GameEvent, GameObservation, GameRuleset, GameState,
    GarrisonStackView, GarrisonView, HeroInfo, LevelUpView, OWNER_LEVEL, TAVERN_LEVELS,
    TownBuildingsView, TownOwnerView, TownView, Viewer, apply, legal_commands, legal_moves,
    observe, project,
};
use h3_heroes::HeroDetails;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use super::import::object;
use super::play::{
    BLOCKERS, DWELLINGS, FAITHFUL, HERO_TEMPLATE, KINDS, LEVEL_1, MONSTER_OBJECTS, SHIPYARD_OBJECT,
    STRICT, Slots, Spot, Step, TOWN_BLOCKED, TOWN_BUILDINGS, TOWN_ENTRANCE, TownPlan, add_town,
    all_objects, army, at_sea, experience, experienced, lay_out, masks, object_index, object_reach,
    options, plan, player_of, resolve, slots, start_on, steps, tile,
};
use crate::model::{Body, Buildings, Object, Pos, RawMap, Stack};

/// The plans whose towns draw nothing a viewer may see, for the hygiene property (E09 T9b2b;
/// decision 34; [`hygienic_town`]). It builds its other game with another seed, and the towns'
/// draws come first in the setup stream (the town epic's design note §14: a random town's faction,
/// a town's buildings and garrison), so that a change of a town the viewer never sees ([`vary`])
/// moves every later draw: a seen random town's faction or a neutral garrison would show another
/// draw in the other game. So made, the towns draw nothing a view shows (an owned standard set's
/// level-2 dwelling with a Fort, TWN-SET-003, is its owner's to see since E09 T18b2a,
/// TWN-VEW-004, so no owned town has that set), and no main town generates a hero, so that the
/// viewer's heroes keep the armies and experience the property gives them. (E08 T106b1) Without
/// the teleporters until [`vary`] knows them (T106b2, decision 291): an unseen exit or gate removed
/// or moved changes another player's jump, so that the viewer may see a hero arrive in one game
/// alone.
fn hygienic() -> impl Strategy<Value = super::play::Plan> {
    plan().prop_map(|mut plan| {
        plan.towns.iter_mut().for_each(hygienic_town);
        plan.teleporters.clear();
        plan
    })
}

/// A plan's town that draws nothing a viewer may see ([`hygienic`]): a random town made a town of
/// a fixed faction (its hero's index as the code; its alignment then unused), a neutral town's
/// editor garrison (drawn, TWN-SET-004) given as ten of a level-1 creature in its first slot, an
/// owned town's standard set with a Fort (its level-2 dwelling drawn, TWN-SET-003; its buildings
/// its owner's, TWN-VEW-004, and its captor's after a capture) made the one without.
fn hygienic_town(town: &mut TownPlan) {
    if town.owner.is_some()
        && TOWN_BUILDINGS[town.buildings] == (Buildings::Standard { fort: true })
    {
        town.buildings = 0;
    }
    let faction = *town.faction.get_or_insert(town.at.1);
    if town.owner.is_none() && !town.visited && town.garrison.is_none() {
        let mut slots = vec![None; 7];
        slots[0] = Some((LEVEL_1[usize::from(faction) % LEVEL_1.len()], 10));
        town.garrison = Some(slots);
    }
}

/// What the hygiene test changes of a game where the viewer cannot see it, drawn as the plan is.
#[derive(Debug, Clone)]
struct Unseen {
    /// Another seed (none: the same).
    seed: Option<u64>,
    /// The army each of the other players' heroes gets instead, in the map's order (none: kept).
    armies: Vec<Option<Slots>>,
    /// Terrain, road and river codes for the tiles the viewer never sees, as many as the largest
    /// map has (none: kept).
    tiles: Vec<Option<(u8, u8, u8)>>,
    /// Which of the blockers the viewer never sees are removed (modulo their number).
    removed: Vec<bool>,
    /// Blockers added where the viewer never sees them: anchor (modulo the map) and template.
    added: Vec<(Spot, usize)>,
    /// Another tile (modulo the map) for each of the other players' heroes, in the map's order
    /// (none: kept): taken where the viewer sees the hero on neither tile, the new one land and
    /// free of heroes and blockers.
    moved: Vec<Option<Spot>>,
    /// Another custom experience for each of the other players' heroes that has one, in the
    /// map's order (none: kept; E08 T31): their levels at load, drawn from their level seeds or
    /// the setup stream after every other draw, and so their level-ups in play. A hero without
    /// one (SoD's plans) keeps its default, drawn from the seed before the objects' contents.
    experience: Vec<Option<u32>>,
    /// (E08 T86b1) Another owner at load for each ownable object the viewer never sees, in the
    /// map's order (cyclically; empty: kept; one the viewer owns is kept, E08 T96b1, [`vary`]): a
    /// playing colour but the viewer's by its index (modulo their number), or none (neutral). Its
    /// owner shows only with the object (ADV-OWN-003), and its reveal at the start is its owner's
    /// (ADV-OWN-001). (E08 T96b1) A dwelling's too (increment 10's design note §10: its owner
    /// shows with it), the random dwellings after every other ownable object; (E09 T9b2b) a
    /// town's after them, one the viewer owns or whose entrance a hero stands on kept.
    owners: Vec<Option<u8>>,
    /// (E09 T9b2b) For each town the viewer never sees and does not own, in the map's order
    /// (cyclically; empty: kept): another garrison (none: kept; the editor's or seven slots) and
    /// other buildings ([`TOWN_BUILDINGS`] by index; none: kept).
    towns: Vec<(Option<Slots>, Option<usize>)>,
    /// (E09 T9b2b) Towns added where the viewer never sees them, after every other object: anchor
    /// (modulo the map) and town (its faction, owner, garrison, formation and buildings as a
    /// plan's, made [`hygienic_town`]; its owner a playing colour but the viewer's by its index).
    added_towns: Vec<(Spot, TownPlan)>,
}

/// Each kind of change in about half the cases, so that some cases keep the other players'
/// commands' outcomes the same for long (another army or terrain changes a hero's points).
fn unseen() -> impl Strategy<Value = Unseen> {
    let tiles = 2 * 16 * 16;
    (
        proptest::option::of(any::<u64>()),
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::weighted(0.5, army()), 1..=6)
        ],
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::weighted(0.5, tile()), tiles)
        ],
        prop_oneof![Just(Vec::new()), vec(any::<bool>(), 1..=8)],
        vec((any::<Spot>(), 0..BLOCKERS.len()), 0..=8),
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::of(any::<Spot>()), 1..=6)
        ],
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::weighted(0.5, experience()), 1..=6)
        ],
        // E08 T86b1, drawn last so that the draws before stay as they were.
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::of(any::<u8>()), 1..=4)
        ],
        // E09 T9b2b, after them for the same reason.
        prop_oneof![
            Just(Vec::new()),
            vec(
                (
                    proptest::option::weighted(0.5, army()),
                    proptest::option::weighted(0.5, 0..TOWN_BUILDINGS.len()),
                ),
                1..=4
            )
        ],
        vec((any::<Spot>(), super::play::town()), 0..=2),
    )
        .prop_map(
            |(
                seed,
                armies,
                tiles,
                removed,
                added,
                moved,
                experience,
                owners,
                towns,
                added_towns,
            )| {
                Unseen {
                    seed,
                    armies,
                    tiles,
                    removed,
                    added,
                    moved,
                    experience,
                    owners,
                    towns,
                    added_towns,
                }
            },
        )
}

/// An army of its own for the viewer's heroes: at least one stack (a hero whose slots are all
/// empty gets its type's starting army, drawn from the seed).
fn own_army() -> impl Strategy<Value = Vec<Option<(u16, u16)>>> {
    slots().prop_filter("a stack at least", |slots| {
        slots.iter().any(Option::is_some)
    })
}

/// The tiles a template anchored at `anchor` covers on the map (blocked and visitable; a special
/// ground's, which neither blocks nor is visited, the anchor it lies on: E08 T20b3b1).
fn covers(anchor: Spot, template: usize, side: u8) -> impl Iterator<Item = Spot> {
    let (ax, ay, az) = anchor;
    let (blocked, visitable) = masks(template);
    let lies: &[(u8, u8)] = if blocked.is_empty() && visitable.is_empty() {
        &[(0, 0)]
    } else {
        &[]
    };
    blocked
        .iter()
        .chain(visitable)
        .chain(lies)
        .filter_map(move |&(column, row)| {
            let x = ax.checked_sub(column).filter(|&x| x < side)?;
            let y = ay.checked_sub(row).filter(|&y| y < side)?;
            Some((x, y, az))
        })
}

/// The tiles of the map a plan's object other than a hero covers ([`covers`]; E09 T9b2b: a town's
/// [`town_covers`], since its template is no `KINDS` entry, the town lane's `TRAPS.md`).
fn footprint(o: &Object, side: u8) -> Vec<Spot> {
    let anchor = (o.pos.x, o.pos.y, o.pos.z);
    if matches!(o.body, Body::Town(_)) {
        town_covers(anchor, side).collect()
    } else {
        covers(anchor, usize::try_from(o.template - 1).unwrap(), side).collect()
    }
}

/// The tiles a plan's town anchored at `anchor` covers on the map (E09 T9b2b; `play.rs`'s
/// [`TOWN_BLOCKED`], its entrance among them).
fn town_covers((ax, ay, az): Spot, side: u8) -> impl Iterator<Item = Spot> {
    TOWN_BLOCKED.iter().filter_map(move |&(column, row)| {
        let x = ax.checked_sub(column).filter(|&x| x < side)?;
        Some((x, ay.checked_sub(row)?, az))
    })
}

/// The entrance of a plan's town anchored at `anchor` (E09 T9b2b; `play.rs`'s
/// [`TOWN_ENTRANCE`]), if it lies on the map.
fn town_entrance((ax, ay, az): Spot) -> Option<Spot> {
    Some((
        ax.checked_sub(TOWN_ENTRANCE.0)?,
        ay.checked_sub(TOWN_ENTRANCE.1)?,
        az,
    ))
}

/// A Shipyard's offsets from its entrance where its boat tile may lie (ADV-SHY-001), the packs'
/// (every Shipyard's kind has the original's twelve).
fn shipyard_offsets() -> &'static [(i8, i8)] {
    let types = FAITHFUL.adventure().object_types();
    types
        .iter()
        .find_map(|t| match &t.kind {
            ObjectKind::Shipyard { offsets, .. } => Some(offsets.as_slice()),
            _ => None,
        })
        .unwrap()
}

/// The class of a plan's object by its template (`KINDS`), none for a hero or (E09 T9b2b) a town.
fn kind(o: &Object) -> Option<u32> {
    (o.template != HERO_TEMPLATE && !matches!(o.body, Body::Town(_)))
        .then(|| KINDS[usize::try_from(o.template - 1).unwrap() / BLOCKERS.len()].0)
}

/// The heroes of `m`, in the map's order: each one's owner (a colour) and army.
fn heroes(m: &mut RawMap) -> impl Iterator<Item = (u8, &mut Option<Vec<Option<Stack>>>)> {
    m.objects.iter_mut().filter_map(|o| match &mut o.body {
        Body::Hero(hero) => Some((hero.owner, &mut hero.army)),
        _ => None,
    })
}

/// The tile a hero object stands on: one column left of its anchor.
fn standing_on(o: &Object) -> Spot {
    (o.pos.x - 1, o.pos.y, o.pos.z)
}

fn stacks(slots: &[Option<(u16, u16)>]) -> Vec<Option<Stack>> {
    slots
        .iter()
        .map(|slot| slot.map(|(creature, count)| Stack { creature, count }))
        .collect()
}

/// `m` with `unseen`'s changes where the viewer (`colour`) never sees them: `ground` says whether
/// it never sees a tile's terrain, road and river, `objects` whether it never sees what objects
/// cover there and the heroes standing there; a monster it never sees where it never sees the
/// ground (E08 decision 40: a monster shows on the fringe in either profile). A hero's tile stays
/// land, and no blocker is added over it; (E08 T75b) a boat's tile stays water, and no blocker is
/// added over it (the setup refuses both, ADV-SEA-001), while a boat the viewer never sees may be
/// removed as a blocker is. (E08 T96b1) An object the viewer owns at load stays, and stays its
/// own, wherever it lies: what it owns is its own, and so its income (GAM-RES-003), which the
/// start's reveal does not always show (ADV-OWN-002: an object visited from no tile of the map, as
/// a mine anchored on the map's first column, has no entrance and reveals nothing). A random
/// dwelling (increment 10's objects in the plans) shows as the dwelling the build resolves it to,
/// whose footprint may lie past its template's at a moved anchor: it is removed, or its owner at
/// load changed, only where the viewer sees none of the tiles it may cover once resolved
/// ([`object_reach`]), and no hero is moved onto one of those tiles. (E09 T9b2b) A town the viewer
/// never sees ([`footprint`]) may take another owner (not where a hero stands on its entrance),
/// garrison and buildings, unless it is the viewer's; it is removed only after every town the
/// viewer sees (a seen town's `TownId` counts the towns before it), and towns are added after
/// every other object; its tiles take no hero moved and no object added, its entrance stays land.
/// A Shipyard's offsets where its boat tile may lie (ADV-SHY-001) keep their terrain and objects.
fn vary(
    m: &RawMap,
    colour: u8,
    unseen: &Unseen,
    ground: impl Fn(Spot) -> bool,
    objects: impl Fn(Spot) -> bool,
) -> RawMap {
    let mut m = m.clone();
    let side = u8::try_from(m.terrain.size).unwrap();
    let levels = u8::try_from(m.terrain.levels).unwrap();
    let mut standing: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| o.template == HERO_TEMPLATE)
        .map(standing_on)
        .collect();
    let afloat: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| kind(o) == Some(8))
        .map(|o| (o.pos.x, o.pos.y, o.pos.z))
        .collect();
    // E08 T86b1: an ownable object's entrance stays land, as a hero's tile (the import refuses one
    // on water: `OwnedOnWater`); (E08 T96b1) an abandoned mine's too, which is ownable (`lay_out`'s
    // `OWNABLE`) though neutral at load, with no owner in its body. (E08 T97b1) Not a dwelling's
    // (whose body is `Owned`) nor a random dwelling's: the import builds one entered from water
    // and `lay_out` lays it out there (survey `E08-dwellings.md` §17). (E09 T9b2b) A town's
    // entrance too, as the plans lay them out (`play.rs`'s `towns_laid_out`).
    let entrances: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| {
            matches!(
                o.body,
                Body::Mine { .. } | Body::AbandonedMine(_) | Body::Owned { .. }
            ) && !(DWELLINGS.0..DWELLINGS.0 + DWELLINGS.1)
                .contains(&(usize::try_from(o.template - 1).unwrap() / BLOCKERS.len()))
        })
        .flat_map(|o| {
            let (_, visitable) = masks(usize::try_from(o.template - 1).unwrap());
            visitable.iter().filter_map(|&(column, row)| {
                Some((
                    o.pos.x.checked_sub(column)?,
                    o.pos.y.checked_sub(row)?,
                    o.pos.z,
                ))
            })
        })
        .chain(
            m.objects
                .iter()
                .filter(|o| matches!(o.body, Body::Town(_)))
                .filter_map(|o| town_entrance((o.pos.x, o.pos.y, o.pos.z))),
        )
        .collect();
    // A Shipyard's boat tile is fixed when the game is built: the first of its kind's offsets from
    // its entrance on water that no object blocks or is visited from (ADV-SHY-001), and a boat
    // built there shows it to its builder and to the players that see the tile (ADV-SHY-002), so
    // that these tiles keep their terrain and what objects cover them, from each tile a Shipyard is
    // visited from (a case of `master`'s found by E09 T9b2b's stress: another player's boat built
    // on a tile the viewer sees, as an earlier offset it never saw became land).
    let harbour: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| kind(o) == Some(87))
        .flat_map(|o| {
            let (_, visitable) = masks(usize::try_from(o.template - 1).unwrap());
            let (ax, ay, az) = (o.pos.x, o.pos.y, o.pos.z);
            visitable.iter().flat_map(move |&(column, row)| {
                let entrance = ax.checked_sub(column).zip(ay.checked_sub(row));
                shipyard_offsets().iter().filter_map(move |&(dx, dy)| {
                    let (ex, ey) = entrance?;
                    let x = ex.checked_add_signed(dx).filter(|&x| x < side)?;
                    let y = ey.checked_add_signed(dy).filter(|&y| y < side)?;
                    Some((x, y, az))
                })
            })
        })
        .collect();
    let spots =
        (0..levels).flat_map(|z| (0..side).flat_map(move |y| (0..side).map(move |x| (x, y, z))));
    for ((spot, tile), new) in spots.zip(&mut m.terrain.tiles).zip(&unseen.tiles) {
        if let Some((terrain, road, river)) =
            new.filter(|_| ground(spot) && !harbour.contains(&spot))
        {
            tile.terrain =
                if terrain >= 8 && (standing.contains(&spot) || entrances.contains(&spot)) {
                    2
                } else if afloat.contains(&spot) {
                    8
                } else {
                    terrain
                };
            tile.road = road;
            tile.river = river;
        }
    }
    // The tiles of the monsters the viewer sees (where it sees the ground, decision 40), which no
    // object added or removed may cover (E08 T31: random cases found one): the last object on a
    // tile takes its visits (ADV-OBJ-003), so the monster would stop guarding (ADV-MON-005), and
    // the viewer's paths would show it.
    let seen_monsters: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| kind(o) == Some(54))
        .flat_map(|o| footprint(o, side))
        .filter(|&spot| !ground(spot))
        .collect();
    let unseen_blocker = |anchor: Spot, template: usize| {
        let monster = KINDS[template / BLOCKERS.len()].0 == 54;
        covers(anchor, template, side).all(|spot| {
            objects(spot)
                && (!monster || ground(spot))
                && !seen_monsters.contains(&spot)
                && !harbour.contains(&spot)
        })
    };
    // E08 T96b1: a random dwelling the viewer owns at load stays too (its body holds its owner);
    // (E09 T9b2b) and a town.
    let own = |o: &Object| match o.body {
        Body::Mine { owner } | Body::Owned { owner } => owner == u32::from(colour),
        Body::RandomDwelling(ref d) => d.owner == u32::from(colour),
        Body::Town(ref t) => t.owner == colour,
        _ => false,
    };
    // E08 T96b1: the tiles a random dwelling may cover once resolved past its template's
    // ([`object_reach`]: its outcomes' footprints at the moved anchor), which the viewer sees of it
    // as of any object (its view is the resolved dwelling's): it is removed only when it sees none
    // of them either, nor a monster there (as [`unseen_blocker`]'s).
    let unseen_reach = |o: &Object| {
        object_reach(o, side)
            .into_iter()
            .all(|spot| objects(spot) && !seen_monsters.contains(&spot) && !harbour.contains(&spot))
    };
    // E09 T9b2b: a town (whose tiles no other object's of a plan cover) is removed only where
    // the viewer sees none of its tiles nor a monster there, as a blocker, and only after every
    // town it sees: a seen town's `TownId`, which its view and its object's show, counts the towns
    // before it (`docs/06-security.md`, "What a viewer may know: a town").
    let unseen_town = |o: &Object| {
        footprint(o, side)
            .into_iter()
            .all(|spot| objects(spot) && !seen_monsters.contains(&spot) && !harbour.contains(&spot))
    };
    let seen: Vec<bool> = m
        .objects
        .iter()
        .filter(|o| matches!(o.body, Body::Town(_)))
        .map(|o| !footprint(o, side).into_iter().all(&objects))
        .collect();
    let last_seen = seen.iter().rposition(|&seen| seen);
    let (mut blocker, mut town) = (0, 0);
    m.objects.retain(|o| {
        if o.template == HERO_TEMPLATE {
            return true;
        }
        let index = blocker;
        blocker += 1;
        let removed = unseen.removed.get(index % unseen.removed.len().max(1)) == Some(&true);
        if matches!(o.body, Body::Town(_)) {
            let ordinal = town;
            town += 1;
            let after = last_seen.is_none_or(|seen| ordinal > seen);
            return own(o) || !(removed && after && unseen_town(o));
        }
        let template = usize::try_from(o.template - 1).unwrap();
        own(o)
            || !(removed
                && unseen_blocker((o.pos.x, o.pos.y, o.pos.z), template)
                && unseen_reach(o))
    });
    // E09 T9b2b: the towns' tiles, which no blocker added covers.
    let towns: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| matches!(o.body, Body::Town(_)))
        .flat_map(|o| footprint(o, side))
        .collect();
    for &((x, y, z), template) in &unseen.added {
        let anchor = (x % side, y % side, z % levels);
        if unseen_blocker(anchor, template)
            && covers(anchor, template, side).all(|spot| {
                !standing.contains(&spot) && !afloat.contains(&spot) && !towns.contains(&spot)
            })
        {
            let mut o = object(
                anchor.0,
                anchor.1,
                1 + u32::try_from(template).unwrap(),
                Body::None,
            );
            o.pos.z = anchor.2;
            m.objects.push(o);
        }
    }
    // E08 T86b1: the owners of the ownable objects the viewer never sees.
    let playing: Vec<u8> = (0u8..)
        .zip(&m.players)
        .filter(|&(c, p)| (p.human || p.computer) && c != colour)
        .map(|(c, _)| c)
        .collect();
    // (E08 T96b1) A placed dwelling's owner is among them (its body is `Owned`); a random
    // dwelling's comes after every other's, so that the others keep the owners they drew before,
    // and only when the viewer sees none of the tiles it may cover once resolved
    // ([`object_reach`]).
    let mut ownable = 0;
    let unseen_owner = |o: &Object| {
        o.template != HERO_TEMPLATE
            && footprint(o, side).into_iter().all(&objects)
            && object_reach(o, side).into_iter().all(&objects)
    };
    for random in [false, true] {
        for o in &mut m.objects {
            let unseen_object = unseen_owner(o);
            let owner = match &mut o.body {
                Body::Mine { owner } | Body::Owned { owner } if !random => owner,
                Body::RandomDwelling(d) if random => &mut d.owner,
                _ => continue,
            };
            if !unseen_object {
                continue;
            }
            let index = ownable;
            ownable += 1;
            if *owner == u32::from(colour) {
                continue;
            }
            match unseen.owners.get(index % unseen.owners.len().max(1)) {
                Some(Some(n)) if !playing.is_empty() => {
                    *owner = u32::from(playing[usize::from(*n) % playing.len()]);
                }
                Some(None) => *owner = 0xFF,
                _ => {}
            }
        }
    }
    // E09 T9b2b: the towns the viewer never sees, after every other ownable object: their owners
    // (one a hero stands on the entrance of keeps it: its visitor at load, TWN-VIS-002, whose
    // owner's team the town is), garrisons and buildings. One the viewer owns keeps all three: its
    // view shows them to it (TWN-VEW-003's owner's facts), its Tavern the level of every other
    // town's view (TWN-VEW-002), its garrison its own army.
    let mut changed = 0;
    for o in &mut m.objects {
        if !matches!(o.body, Body::Town(_)) || !unseen_owner(o) {
            continue;
        }
        let visited = town_entrance((o.pos.x, o.pos.y, o.pos.z))
            .is_some_and(|entrance| standing.contains(&entrance));
        let Body::Town(body) = &mut o.body else {
            continue;
        };
        let index = ownable;
        ownable += 1;
        if body.owner == colour {
            continue;
        }
        match unseen.owners.get(index % unseen.owners.len().max(1)) {
            Some(Some(n)) if !playing.is_empty() && !visited => {
                body.owner = playing[usize::from(*n) % playing.len()];
            }
            Some(None) if !visited => body.owner = u8::MAX,
            _ => {}
        }
        let change = unseen.towns.get(changed % unseen.towns.len().max(1));
        changed += 1;
        let Some((garrison, buildings)) = change else {
            continue;
        };
        if let Some(slots) = garrison {
            body.garrison = slots.as_deref().map(stacks);
        }
        if let Some(buildings) = *buildings {
            body.buildings = TOWN_BUILDINGS[buildings].clone();
        }
    }
    // E09 T9b2b: towns added where the viewer never sees them, after every other object (so
    // after every town, whose `TownId`s stay), as a plan's are laid out: on no tile of a hero, a
    // boat or another object (nor one a random dwelling may cover once resolved), nor a monster
    // the viewer sees, the entrance on the map's land.
    for ((x, y, z), town) in &unseen.added_towns {
        let anchor = (x % side, y % side, z % levels);
        let taken: Vec<Spot> = m
            .objects
            .iter()
            .filter(|o| o.template != HERO_TEMPLATE)
            .flat_map(|o| footprint(o, side).into_iter().chain(object_reach(o, side)))
            .collect();
        let land = town_entrance(anchor).is_some_and(|(ex, ey, ez)| {
            let at = (usize::from(ez) * usize::from(side) + usize::from(ey)) * usize::from(side)
                + usize::from(ex);
            m.terrain.tiles[at].terrain < 8
        });
        let free = town_covers(anchor, side).all(|spot| {
            objects(spot)
                && !seen_monsters.contains(&spot)
                && !standing.contains(&spot)
                && !afloat.contains(&spot)
                && !taken.contains(&spot)
                && !harbour.contains(&spot)
        });
        if land && free {
            let mut town = TownPlan {
                visited: false,
                ..town.clone()
            };
            hygienic_town(&mut town);
            let owner = town
                .owner
                .filter(|_| !playing.is_empty())
                .map_or(u8::MAX, |n| playing[usize::from(n) % playing.len()]);
            add_town(&mut m, anchor, &town, owner, None);
        }
    }
    // E08 T96b1: and the tiles a random dwelling may cover once resolved, whatever it draws (the
    // setup refuses a hero there, `SetupError::RandomDwellingTile`).
    let blocked: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| o.template != HERO_TEMPLATE)
        .flat_map(|o| footprint(o, side).into_iter().chain(object_reach(o, side)))
        .collect();
    let mut other = 0;
    for o in &mut m.objects {
        let Body::Hero(hero) = &mut o.body else {
            continue;
        };
        if hero.owner == colour {
            continue;
        }
        let index = other;
        other += 1;
        if let Some(Some(slots)) = unseen.armies.get(index % unseen.armies.len().max(1)) {
            hero.army = slots.as_deref().map(stacks);
        }
        let experience = unseen
            .experience
            .get(index % unseen.experience.len().max(1));
        if let (Some(Some(amount)), Some(own)) = (experience, &mut hero.experience) {
            *own = *amount;
        }
        if let Some(Some((x, y, z))) = unseen.moved.get(index % unseen.moved.len().max(1)) {
            let (from, to) = (standing_on(o), (x % side, y % side, z % levels));
            let at = (usize::from(to.2) * usize::from(side) + usize::from(to.1))
                * usize::from(side)
                + usize::from(to.0);
            if objects(from)
                && objects(to)
                && m.terrain.tiles[at].terrain < 8
                && !blocked.contains(&to)
                && !standing.contains(&to)
            {
                o.pos = Pos {
                    x: to.0 + 1,
                    y: to.1,
                    z: to.2,
                };
                standing.retain(|&spot| spot != from);
                standing.push(to);
            }
        }
    }
    m
}

/// The viewer sees the same of `a` and `b`: its observation, its legal commands and the legal
/// moves of every hero it sees.
fn check_same(
    a: &GameState,
    b: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
) -> Result<(), TestCaseError> {
    let seen = observe(a, rules, Viewer::Player(viewer));
    prop_assert_eq!(&seen, &observe(b, rules, Viewer::Player(viewer)));
    let actor = Actor::Player(viewer);
    prop_assert_eq!(
        legal_commands(a, rules, actor),
        legal_commands(b, rules, actor)
    );
    for hero in &seen.adventure.heroes {
        prop_assert_eq!(
            legal_moves(a, rules, actor, hero.id),
            legal_moves(b, rules, actor, hero.id)
        );
    }
    Ok(())
}

/// How many objects `viewer` knows what it learned of (E08 T64b; ADV-VIS-006: its `known`).
fn known(state: &GameState, viewer: PlayerId) -> usize {
    let visits = state.adventure().visits();
    let objects = 0..state.adventure().map().objects().len();
    objects
        .filter(|&i| visits.is_some_and(|v| v.knows(viewer, i)))
        .count()
}

/// The events as `viewer` is told them.
fn told(events: &[GameEvent], viewer: PlayerId) -> Vec<GameEvent> {
    events
        .iter()
        .filter_map(|event| project(event, Viewer::Player(viewer)))
        .collect()
}

/// Whether the viewer sees what stands on the tile at `pos`: objects and heroes (ADV-FOG-004,
/// ADV-FOG-005).
fn sees(view: &GameObservation, pos: TilePos) -> bool {
    matches!(view.adventure.tile(pos), Some(TileView::Seen(_)))
}

/// How much the viewer sees of a tile: nothing, its ground, everything.
fn depth(view: &TileView) -> u8 {
    match view {
        TileView::Hidden => 0,
        TileView::Ground { .. } => 1,
        TileView::Seen(_) => 2,
    }
}

/// The observation's consistency with the events: `after` is `before` with what `viewer` was `told`
/// applied as a client would. The day, the acting players and the turn group follow the game's
/// events; the heroes it sees move as the projected steps say (a step names the viewer alone and
/// tells at least one tile; a step's tile it leaves is where the viewer last saw the hero, a step
/// it is not told the tile entered takes the hero out of its sight); a hero shows without a step
/// only on a tile the viewer did not see before, by its own command or a new day's reveal (E08
/// T58b). The viewer forgets nothing: a tile's view only deepens, and keeps what it showed; an
/// object it saw leaves its view only as an object removal it is told (E08 T13b), and a monster's
/// word of its count changes only as it is told (E08 T20b3b1); its treasury changes only by the
/// gains and payments it is told, which are its own. Its open decision and its purchases change
/// only as its offers' events say: an offer it is told opened is its decision, an answer closes it,
/// an accepted Cartographer's adds the type to its purchases (E08 T14b); a level-up it is told
/// opened is its decision, and the level it is told it gained closes it (E08 T30); a reveal it is
/// told explores at least that many more tiles; its keys change only by the keys it is told it
/// gained (T15a); its heroes' details only by the changes it is told, each of a hero it saw as its
/// own (T19a2a). Its planes change only by its own command (`own`), and its explored tiles grow at
/// a new day's start too (E08 T40: in `strict-fixed`, `ADV-QUIRK-18` off, each hero reveals around
/// it to its owner then, so another player's last end of turn of the day can widen a radius grown
/// since the hero's last step), and at a capture told to it as the town's new owner, whose reveal
/// a siege's last command brings, another player's or the system's (E09 T9b2a: as at a new day,
/// they only grow, and heroes and boats may show in them). (E09 T9b2a) A town's capture is told
/// to its new owner and to a viewer that saw its object, whose owner it changes; a Capitol's
/// change only after the town's capture; a town's view stays, its anchor, faction, name and fort
/// as they were, its owner and Capitol as it is told, its garrison by slot (the owner's) as the
/// merge's slots it is told say but after a battle of its own or the town's capture, and a town
/// shows only where its explored tiles grew (the garrison by its level and the top level's facts:
/// [`check_town_views`]).
/// An open decision as [`check_told`] follows it: an offer by its hero and object, a level-up,
/// a meeting's two heroes, or an army dialog's hero and group (E08 T51b).
#[derive(Debug, PartialEq)]
enum Open {
    Offer(HeroId, h3_adventure::ObjectTypeId, TilePos),
    LevelUp(LevelUpView),
    Meeting([HeroId; 2]),
    Dialog(HeroId, Vec<ArmyStack>),
    // E08 T63a: a choice and a University, named by the hero and the object.
    Choice(HeroId, h3_adventure::ObjectTypeId, TilePos),
    University(HeroId, h3_adventure::ObjectTypeId, TilePos),
    // E08 T95b: a hire dialog, named by its army and its source.
    Hire(h3_game::HireTarget, h3_adventure::ObjectTypeId, TilePos),
}

fn check_told(
    viewer: PlayerId,
    (before, after): (&GameObservation, &GameObservation),
    told: &[GameEvent],
    own: bool,
) -> Result<(), TestCaseError> {
    let mut day = before.day;
    // E08 T85b: the month's kind, public, told by its start.
    let mut month = before.month;
    let mut acting: Vec<bool> = before.players.iter().map(|p| p.acting).collect();
    // The turn group: the players whose turns start after the last turn that ended, if any did.
    let mut group = before.turn_group.clone();
    let mut started = false;
    let mut heroes: Vec<(HeroId, TilePos)> = before
        .adventure
        .heroes
        .iter()
        .map(|h| (h.id, h.pos))
        .collect();
    let mut stepped = Vec::new();
    let mut removed = Vec::new();
    let mut words = Vec::new();
    // E08 T75b: the empty boats the viewer sees, and the heroes it is told boarded (their boat's
    // type) or landed or left (none); (E08 T97b2) the heroes told stepping into its sight, whose
    // boat shows as it stands.
    let mut boats = before.adventure.boats.clone();
    let mut aboard: Vec<(HeroId, Option<u8>)> = Vec::new();
    let mut returned: Vec<HeroId> = Vec::new();
    let mut treasury = before.treasury.clone();
    // E08 T85b: the timed events that fired at the viewer's turn start today, told by their
    // firing; a new day starts with none.
    let mut today = before.events_today.clone();
    let named = |d: &DecisionView| match d {
        DecisionView::Offer(o) => Open::Offer(o.hero, o.object_type, o.anchor),
        DecisionView::LevelUp(view) => Open::LevelUp(*view),
        DecisionView::Meeting { heroes } => Open::Meeting(*heroes),
        DecisionView::ArmyDialog { hero, group, .. } => Open::Dialog(*hero, group.clone()),
        DecisionView::Choice(c) => Open::Choice(c.hero, c.object_type, c.anchor),
        DecisionView::University { visit: v, .. } => {
            Open::University(v.hero, v.object_type, v.anchor)
        }
        DecisionView::Hire(view) => Open::Hire(view.target, view.object_type, view.anchor),
    };
    // E08 T51b: the group of the army dialog the viewer is told closed, whose stacks are lost.
    let mut lost: Vec<ArmyStack> = Vec::new();
    // E08 T86b1: the mine of the viewer's open Leave Guards dialog, whose close makes the group
    // its guards (ADV-MIN-002), which the viewer's team sees whole (ADV-OWN-003).
    let mut guarding = match &before.decision {
        Some(DecisionView::ArmyDialog { mine, .. }) => *mine,
        _ => None,
    };
    // E08 T84a: the objects the viewer is told were flagged or whose guards' hint changed.
    let mut flagged = Vec::new();
    // E08 T85a: the monsters a creature month added that the viewer is told of (the game emits
    // them from T85b).
    let mut added = Vec::new();
    let mut decision = before.decision.as_ref().map(named);
    let mut bought = before.adventure.bought.clone();
    let mut keys = before.adventure.keys;
    let mut revealed = 0;
    // E08 T19a: whether the viewer is in a battle, by its start and end.
    let mut fighting = before.battle.is_some();
    // E08 T19a2a: the viewer's heroes whose details it is told changed.
    let mut changed = Vec::new();
    // E09 T9b2a: the towns it is told were captured and by whom, and the Capitols it is told
    // changed (TWN-CAP-001, -002), in order; the garrisons by slot it sees (its own towns',
    // TWN-VEW-003) as the merge's slots it is told change them (TWN-SIE-002); whether a battle of
    // its own ended, whose sides' armies, a siege's garrison among them, it changes.
    let mut captured: Vec<(TownId, PlayerId)> = Vec::new();
    // E09 T18b2a (SRV-Q26): the views of the towns it is told, each the last `TownChanged` of its
    // town with the stacks and income lines that follow it.
    let mut views: Vec<TownView> = Vec::new();
    let mut garrisons: Vec<(TownId, Vec<ArmyStack>)> = before
        .towns
        .iter()
        .filter_map(|t| Some((t.town, t.garrison.army.clone()?)))
        .collect();
    let mut ended = false;
    let alone = Viewers::NONE.with(viewer);
    for event in told {
        match *event {
            GameEvent::HeroChanged { player, hero, .. }
            | GameEvent::CreaturesRaised { player, hero, .. }
            | GameEvent::LevelGained { player, hero, .. } => {
                prop_assert_eq!(player, viewer);
                let own = before
                    .heroes
                    .iter()
                    .any(|h| h.id == hero && h.details.is_some());
                prop_assert!(
                    own,
                    "{:?} of a hero the viewer did not see as its own",
                    event
                );
                changed.push(hero);
                // E08 T30: a level gained closes its open level-up, if it was one.
                if let GameEvent::LevelGained { level, .. } = *event {
                    let answered = |open: &Open| matches!(open, Open::LevelUp(v) if (v.hero, v.level) == (hero, level));
                    if decision.as_ref().is_some_and(answered) {
                        decision = None;
                    }
                }
            }
            GameEvent::LevelUpOpened {
                player,
                hero,
                level,
                primary,
                offers,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                let view = LevelUpView {
                    hero,
                    level,
                    primary,
                    offers,
                };
                decision = Some(Open::LevelUp(view));
            }
            GameEvent::BattleStarted { seen_by, .. } => {
                prop_assert_eq!(seen_by, alone);
                prop_assert!(!fighting);
                fighting = true;
            }
            GameEvent::Battle { seen_by, .. } => {
                prop_assert_eq!(seen_by, alone);
                prop_assert!(fighting);
            }
            GameEvent::BattleEnded { seen_by, .. } => {
                prop_assert_eq!(seen_by, alone);
                prop_assert!(fighting);
                fighting = false;
                ended = true;
                // E08 T86b1: the guards' side of one of its mines is the viewer's (ADV-MIN-002,
                // GAM-BAT-001): their survivors stay as the guards, which it sees whole, told by
                // the battle's events (their hint's change by `GuardsChanged`).
                let own = before
                    .adventure
                    .objects
                    .iter()
                    .filter(|o| o.guards.as_ref().is_some_and(|g| g.army.is_some()));
                flagged.extend(own.map(|o| (o.object_type, o.anchor)));
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::CreaturesJoined { .. },
                seen_by,
            }) => prop_assert_eq!(seen_by, alone),
            // E08 T105a2: a Whirlpool's loss, told to the hero's owner alone, as its army is
            // (ADV-TEL-006).
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::WhirlpoolLoss { hero, .. },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let own = before
                    .heroes
                    .iter()
                    .any(|h| h.id == hero && h.details.is_some());
                prop_assert!(own, "{:?} of a hero not the viewer's", event);
            }
            // E08 T95a1: a dwelling's free creatures refused for want of room, told to the hero's
            // owner alone, as its army is; none of a free creature at its dwelling, told to the
            // dwelling's owner alone (ADV-HIR-001): nothing the viewer sees changes.
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::JoinRefused { hero, .. },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let own = before
                    .heroes
                    .iter()
                    .any(|h| h.id == hero && h.details.is_some());
                prop_assert!(own, "{:?} of a hero not the viewer's", event);
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::NoneToRecruit { .. },
                seen_by,
            }) => prop_assert_eq!(seen_by, alone),
            // E08 T73: a boarding and a landing, told where the viewer sees heroes on the boat's
            // tile, as it sees the empty boat there (ADV-FOG-008); E08 T74b: a boat leaving with
            // its hero likewise (ADV-SEA-006). E08 T75b follows the boats: a boarding takes the
            // empty boat the viewer saw, a landing leaves one, facing the landing's way.
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::Boarded { hero, boat, at },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let i = boats.iter().position(|b| b.tile == at && b.kind == boat);
                prop_assert!(i.is_some(), "{:?} boards no boat the viewer saw", event);
                boats.remove(i.unwrap());
                aboard.push((hero, Some(boat)));
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::Landed {
                        hero,
                        boat,
                        at,
                        facing,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                boats.push(h3_adventure::BoatView {
                    tile: at,
                    kind: boat,
                    facing,
                });
                aboard.push((hero, None));
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::BoatRemoved { hero, .. },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                aboard.push((hero, None));
            }
            // E08 T62b2: an event's firing, told to its visitor's owner alone (ADV-FOG-007); E08
            // T74a: a message read likewise (ADV-PIK-017; the plans' bottles since T75b); E08
            // T105a1: a teleporter that leads nowhere likewise (ADV-TEL-004, ADV-TEL-005).
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::EventFired { hero, .. }
                    | AdventureEventKind::MessageRead { hero, .. }
                    | AdventureEventKind::LeadsNowhere { hero, .. },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let own = before
                    .heroes
                    .iter()
                    .any(|h| h.id == hero && h.details.is_some());
                prop_assert!(own, "{:?} of a hero not the viewer's", event);
            }
            // E08 T84a: a flag and a mine's guards' hint, told to the players that see the object
            // (and the flag to its new owner): its owner, production and guards may change. E08
            // T84b1: a weekly object's mark, told to the players whose mark it set: its visited
            // mark changes.
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectFlagged {
                        object_type,
                        anchor,
                        ..
                    }
                    | AdventureEventKind::GuardsChanged {
                        object_type,
                        anchor,
                        ..
                    }
                    | AdventureEventKind::ObjectMarked {
                        object_type,
                        anchor,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                flagged.push((object_type, anchor));
            }
            // E09 T8a2: a town's capture, told to the players that see the town and to its new
            // owner (TWN-CAP-001; the town epic's design note §11, §12): its object's owner
            // changes.
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::TownCaptured {
                        town,
                        anchor,
                        owner,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let seen = before
                    .adventure
                    .objects
                    .iter()
                    .find(|o| o.town == Some(town));
                prop_assert!(
                    owner == viewer || seen.is_some_and(|o| o.anchor == anchor),
                    "{:?} of a town the viewer did not see",
                    event
                );
                flagged.extend(seen.map(|o| (o.object_type, o.anchor)));
                captured.push((town, owner));
            }
            // E09 T18b2a (SRV-Q26): the viewer's view of a town it sees, told whole by its facts,
            // then a stack its level shows each, then an income line of the top level each.
            GameEvent::TownChanged {
                player,
                town,
                owner,
                fort,
                capitol,
                built_today,
                level,
                slots,
                hall,
                buildings,
            } => {
                prop_assert_eq!(player, viewer);
                let seen = before
                    .towns
                    .iter()
                    .chain(&after.towns)
                    .find(|t| t.town == town);
                prop_assert!(
                    seen.is_some(),
                    "{:?} of a town the viewer does not see",
                    event
                );
                let seen = seen.unwrap();
                views.retain(|t| t.town != town);
                views.push(TownView {
                    town,
                    anchor: seen.anchor,
                    owner,
                    faction: seen.faction,
                    name: seen.name.clone(),
                    fort,
                    capitol,
                    built_today,
                    garrison: GarrisonView {
                        level,
                        stacks: Vec::new(),
                        army: slots.then(Vec::new),
                    },
                    owner_facts: hall.map(|hall| TownOwnerView {
                        hall,
                        income: Vec::new(),
                    }),
                    buildings,
                });
            }
            GameEvent::TownStackSeen {
                player,
                town,
                stack,
                slot,
            } => {
                prop_assert_eq!(player, viewer);
                let view = views.last_mut().filter(|t| t.town == town);
                prop_assert!(view.is_some(), "{:?} without its town's change", event);
                let garrison = &mut view.unwrap().garrison;
                garrison.stacks.push(stack);
                prop_assert_eq!(slot.is_some(), garrison.army.is_some(), "{:?}", event);
                if let (Some(army), Some(slot), Some(count)) =
                    (&mut garrison.army, slot, stack.count)
                {
                    army.push(ArmyStack {
                        slot,
                        creature: stack.creature,
                        count,
                    });
                }
            }
            GameEvent::TownIncomeSeen {
                player,
                town,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, viewer);
                let view = views.last_mut().filter(|t| t.town == town);
                let facts = view.and_then(|t| t.owner_facts.as_mut());
                prop_assert!(facts.is_some(), "{:?} without its town's top level", event);
                facts.unwrap().income.push((resource, amount));
            }
            // E08 T84b1: a weekly object found empty, told to its visitor's owner alone; nothing
            // it sees changes. E08 T84b2: a Shipyard that makes no boat offer likewise.
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectEmpty { .. } | AdventureEventKind::ShipyardBlocked { .. },
                seen_by,
            }) => prop_assert_eq!(seen_by, alone),
            // E08 T84b2: a boat built, told where the viewer sees heroes on its tile and to its
            // builder (ADV-SHY-002, ADV-FOG-008): an empty boat facing 2, as the map's.
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::BoatBuilt { tile, boat_type },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                boats.push(h3_adventure::BoatView {
                    tile,
                    kind: boat_type,
                    facing: 2,
                });
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::MonsterChanged {
                        object_type,
                        anchor,
                        count_word,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                words.push(((object_type, anchor), count_word));
            }
            // E08 T85a: a creature month's monster, told to the players that see it as a placed
            // one: it shows with its creature and word, its one tile blocked and visited from.
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::MonsterAdded {
                        object_type,
                        anchor,
                        creature,
                        count_word,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                added.push(((object_type, anchor), creature, count_word));
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::HeroRemoved { hero, pos },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let at = heroes.iter().position(|&(id, _)| id == hero);
                prop_assert_eq!(at.map(|i| heroes[i].1), Some(pos), "{:?}", event);
                heroes.remove(at.unwrap());
            }
            GameEvent::OfferOpened {
                player,
                hero,
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some(Open::Offer(hero, object_type, anchor));
            }
            GameEvent::OfferRefused { player, .. } => prop_assert_eq!(player, viewer),
            // E09 T18b1: a town's build, its owner's; no plan builds before E09 T18b2, which
            // folds the views it changes (the mark, the owner's buildings, the Tavern's levels).
            GameEvent::TownBuilt { player, .. } => prop_assert_eq!(player, viewer),
            // E09 T18c: a town event, its owner's; no plan's town has one before E09 T19 (the
            // writer's events), what it gives and builds told around it as above.
            GameEvent::TownEventFired { player, .. } => prop_assert_eq!(player, viewer),
            // E08 T63a: a choice and a University open as the viewer's decision.
            GameEvent::ChoiceOpened {
                player,
                hero,
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some(Open::Choice(hero, object_type, anchor));
            }
            GameEvent::UniversityOpened {
                player,
                hero,
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some(Open::University(hero, object_type, anchor));
            }
            GameEvent::HireOpened {
                player,
                target,
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some(Open::Hire(target, object_type, anchor));
            }
            // E08 T51a: the management's events are the owner's (the plans manage since T52); a
            // slot of an army dialog's group changes its group (T51b).
            GameEvent::StackChanged { player, at, stack } => {
                prop_assert_eq!(player, viewer);
                if let (ArmyRef::Group, Some(Open::Dialog(_, group))) = (at.army, &mut decision) {
                    group.retain(|s| s.slot != at.slot);
                    group.extend(stack);
                    group.sort_by_key(|s| s.slot);
                }
                // E09 T8b2b: a garrison's slot at a siege's merge (TWN-SIE-002), told to the
                // garrison's owner, which sees it by slot (TWN-VEW-003).
                if let ArmyRef::Garrison(town) = at.army {
                    let army = garrisons.iter_mut().find(|(t, _)| *t == town);
                    prop_assert!(
                        army.is_some(),
                        "{:?} of a garrison the viewer did not see as its own",
                        event
                    );
                    let (_, army) = army.unwrap();
                    army.retain(|s| s.slot != at.slot);
                    army.extend(stack);
                    army.sort_by_key(|s| s.slot);
                }
            }
            GameEvent::FormationSet { player, .. } | GameEvent::TacticsSet { player, .. } => {
                prop_assert_eq!(player, viewer);
            }
            // E08 T51b: a meeting and an army dialog open and close as the viewer's decision;
            // what an army dialog's group held when it closed is lost.
            GameEvent::MeetingOpened { player, heroes } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some(Open::Meeting(heroes));
            }
            // E08 T62b1: one event per stack of the group at its opening, in slot order.
            GameEvent::ArmyDialogOpened {
                player,
                hero,
                stack,
            } => {
                prop_assert_eq!(player, viewer);
                match &mut decision {
                    Some(Open::Dialog(open, group))
                        if *open == hero && group.last().is_none_or(|s| s.slot < stack.slot) =>
                    {
                        group.push(stack);
                    }
                    _ => {
                        prop_assert!(decision.is_none());
                        decision = Some(Open::Dialog(hero, vec![stack]));
                    }
                }
            }
            // E08 T84a: an own mine's Leave Guards dialog opens as the viewer's army dialog, its
            // guards following as its stacks (none for a mine without guards).
            GameEvent::LeaveGuardsOpened {
                player,
                hero,
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some(Open::Dialog(hero, Vec::new()));
                guarding = Some((object_type, anchor));
            }
            // E08 T86b1: a Leave Guards dialog's close loses nothing: its group, which the viewer
            // was told stack by stack, is the mine's guards now.
            GameEvent::DecisionClosed { player } => {
                prop_assert_eq!(player, viewer);
                match decision.take() {
                    Some(Open::Dialog(..)) if guarding.is_some() => {
                        flagged.extend(guarding.take());
                    }
                    Some(Open::Dialog(_, group)) => lost = group,
                    open => prop_assert!(
                        matches!(
                            open,
                            Some(
                                Open::Meeting(_)
                                    | Open::Choice(..)
                                    | Open::University(..)
                                    | Open::Hire(..)
                            )
                        ),
                        "{:?}",
                        open
                    ),
                }
            }
            GameEvent::GroupLost { player, stack, .. } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(lost.contains(&stack), "{:?} was not in the group", stack);
            }
            GameEvent::OfferAnswered {
                player,
                object_type,
                anchor,
                accepted,
            } => {
                prop_assert_eq!(player, viewer);
                let open = decision.take();
                prop_assert!(
                    matches!(open, Some(Open::Offer(_, ty, at)) if (ty, at) == (object_type, anchor))
                );
                // The offer was open before the answer: the observation showed its kind.
                let kind = before
                    .decision
                    .as_ref()
                    .and_then(DecisionView::offer)
                    .map(|o| &o.kind);
                if accepted && matches!(kind, Some(OfferKind::Reveal { .. })) {
                    bought.push(object_type);
                    bought.sort();
                }
            }
            GameEvent::ResourcePaid {
                player,
                resource,
                amount,
            }
            | GameEvent::ResourceLost {
                player,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, viewer);
                treasury[usize::from(resource.raw())] -= amount;
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::TilesRevealed { count, .. },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(viewer));
                revealed += count;
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::KeyGained { key },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(viewer));
                keys |= 1 << key;
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectRemoved {
                        object_type,
                        anchor,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(viewer));
                removed.push((object_type, anchor));
            }
            GameEvent::ResourceGained {
                player,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, viewer);
                treasury[usize::from(resource.raw())] += amount;
            }
            GameEvent::DayStarted { day: started } => {
                day = started;
                today.clear();
            }
            // E08 T85b: a timed event is told to its player alone, at its turn's start.
            GameEvent::TimedEventFired { player, event } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(group.contains(&viewer) && started, "{:?}", event);
                today.push(event);
            }
            // E08 T85b: a week and a month start only on a new day, told to every viewer.
            GameEvent::WeekStarted { .. } => prop_assert!(day != before.day),
            GameEvent::MonthStarted { kind, .. } => {
                prop_assert!(day != before.day);
                month = kind;
            }
            GameEvent::TurnEnded { player } => {
                acting[usize::from(player.raw())] = false;
                started = false;
            }
            GameEvent::TurnStarted { player } => {
                acting[usize::from(player.raw())] = true;
                if !started {
                    group.clear();
                    started = true;
                }
                group.push(player);
            }
            // E08 T105a1: a jump is told as a step's two ends (ADV-FOG-004, A-41 not reproduced):
            // a viewer is told an end only where it sees the hero.
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::HeroStepped {
                        hero,
                        from,
                        to,
                        seen_from,
                        seen_to,
                    }
                    | AdventureEventKind::HeroTeleported {
                        hero,
                        from,
                        to,
                        seen_from,
                        seen_to,
                    },
                seen_by,
            }) => {
                let alone = Viewers::NONE.with(viewer);
                prop_assert_eq!(seen_by, alone);
                prop_assert_eq!(
                    (seen_from, seen_to),
                    (
                        if from.is_some() { alone } else { Viewers::NONE },
                        if to.is_some() { alone } else { Viewers::NONE },
                    )
                );
                prop_assert!(from.is_some() || to.is_some(), "{:?} tells no tile", event);
                let at = heroes.iter().position(|&(id, _)| id == hero);
                prop_assert_eq!(at.map(|i| heroes[i].1), from, "{:?}", event);
                match (at, to) {
                    (Some(i), Some(to)) => heroes[i].1 = to,
                    (Some(i), None) => {
                        heroes.remove(i);
                    }
                    (None, to) => heroes.push((hero, to.unwrap())),
                }
                // E08 T97b2: a hero stepping into the viewer's sight shows its boat as it stands
                // (ADV-FOG-006): a boarding or a landing on a water tile the viewer does not see
                // is told only to those that see that tile (`Landed`, ADV-FOG-008), so what it was
                // told of the hero's boat before stops holding.
                if from.is_none() {
                    aboard.retain(|&(id, _)| id != hero);
                    returned.push(hero);
                }
                stepped.push(hero);
            }
        }
    }
    prop_assert_eq!(after.battle.is_some(), fighting);
    // Its heroes' details change only as it is told (E08 T19a2a), but their spell points at a
    // new day's regain, which no event tells (HER-MAN-002, E08 T85b).
    let regained = |details: Option<&HeroDetails>| {
        details.map(|d| HeroDetails {
            mana: if after.day == before.day { d.mana } else { 0 },
            ..d.clone()
        })
    };
    for hero in &after.heroes {
        let was = before.heroes.iter().find(|h| h.id == hero.id);
        let differ =
            |was: &&HeroInfo| regained(was.details.as_deref()) != regained(hero.details.as_deref());
        if let Some(was) = was.filter(differ) {
            prop_assert!(changed.contains(&hero.id), "{:?} changed untold", was);
        }
    }
    prop_assert_eq!(after.month, month);
    prop_assert_eq!(after.day, day);
    let now: Vec<bool> = after.players.iter().map(|p| p.acting).collect();
    prop_assert_eq!(now, acting);
    prop_assert_eq!(&after.turn_group, &group);
    for &(id, pos) in &heroes {
        prop_assert_eq!(after.adventure.hero(id).map(|h| h.pos), Some(pos));
    }
    // A hero shows without a step only where the viewer's sight grew: by its own command, or by
    // a new day's reveal (E08 T40, below; T58b: a hero standing in the radius a Scouting grown
    // since its owner's last step widened at the day's start), or (E09 T9b2a) by a capture's
    // reveal to its new owner, which a siege's last command, another player's or the system's,
    // may bring (TWN-CAP-001, TWN-CAP-003).
    // (E08 T106b1) Or by a jump of its own hero held after a battle at the exit, which the
    // battle's last command gives it, another player's or the system's (GAM-DEC-007 amended,
    // ADV-TEL-007): the arrival explores around the exit.
    let jumper = told.iter().any(|e| {
        matches!(e, GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroTeleported { hero, .. },
            ..
        }) if after.adventure.heroes.iter().any(|h| h.id == *hero && h.owner == viewer))
    });
    let captor = captured.iter().any(|&(_, owner)| owner == viewer) || jumper;
    let revealing = own || after.day != before.day || captor;
    for hero in &after.adventure.heroes {
        if !heroes.iter().any(|&(id, _)| id == hero.id) {
            prop_assert!(
                revealing && !stepped.contains(&hero.id),
                "{:?} shows untold",
                hero
            );
            prop_assert!(!sees(before, hero.pos) && sees(after, hero.pos));
        }
    }
    // E08 T75b: the empty boats change only as told, but where the viewer's sight grew; a hero's
    // boat only by a boarding, a landing or its leaving (ADV-FOG-006, ADV-FOG-008), or (E08
    // T97b2) as it stands once the hero steps into the viewer's sight (the boat plans found a
    // hero that sailed out of it, landed where the viewer does not see the boat's tile and walked
    // back into it).
    for boat in &boats {
        prop_assert!(
            after.adventure.boats.contains(boat),
            "{:?} left untold",
            boat
        );
    }
    for boat in after.adventure.boats.iter().filter(|b| !boats.contains(b)) {
        let grew = revealing && !sees(before, boat.tile) && sees(after, boat.tile);
        prop_assert!(grew, "{:?} shows untold", boat);
    }
    for hero in &after.adventure.heroes {
        let told = aboard.iter().rev().find(|&&(id, _)| id == hero.id);
        let was = before.adventure.hero(hero.id).map(|h| h.boat);
        if let Some(&(_, boat)) = told {
            prop_assert_eq!(hero.boat, boat, "{:?}", hero);
        } else if let Some(was) = was.filter(|_| !returned.contains(&hero.id)) {
            prop_assert_eq!(hero.boat, was, "{:?} changed boats untold", hero);
        }
    }
    prop_assert_eq!(&after.treasury, &treasury);
    prop_assert_eq!(&after.events_today, &today);
    prop_assert_eq!(after.decision.as_ref().map(named), decision);
    prop_assert_eq!(&after.adventure.bought, &bought);
    prop_assert_eq!(after.adventure.keys, keys);
    let ones = |planes: &[u64]| planes.iter().map(|w| w.count_ones()).sum::<u32>();
    prop_assert!(ones(&after.adventure.explored) >= ones(&before.adventure.explored) + revealed);
    // E09 T9b2a: the towns it sees (TWN-VEW-001; design note §12, §17). A view stays (an explored
    // tile stays explored); since E09 T18b2a (SRV-Q26) a town it is told of shows the told view,
    // and every other town's view is as it was: its owner's capture, its Capitol's limit, its
    // garrison's merge and battle and its mark are told. Visions' reach is not (`TWN-STUB-13`):
    // a view at the top level by Visions (no garrison by slot) before or after keeps its public
    // facts and the owner's buildings only. Its garrison by slot is as the merge's slots it is
    // told say too. What its level shows of the garrison and the top level's facts are
    // [`check_town_views`]' (the state's).
    let visions = |t: &TownView| t.garrison.level == OWNER_LEVEL && t.garrison.army.is_none();
    for was in &before.towns {
        let now = after.towns.iter().find(|t| t.town == was.town);
        prop_assert!(now.is_some(), "{:?} left the view", was);
        let now = now.unwrap();
        if let Some(told) = views.iter().find(|t| t.town == was.town) {
            prop_assert_eq!(now, told, "{:?}'s view is not the told one", was.town);
        } else if visions(was) || visions(now) {
            let public = |t: &TownView| {
                let (owner, fort, capitol, today) = (t.owner, t.fort, t.capitol, t.built_today);
                (
                    t.anchor,
                    t.faction,
                    t.name.clone(),
                    owner,
                    fort,
                    capitol,
                    today,
                    t.buildings,
                )
            };
            prop_assert_eq!(public(now), public(was), "{:?} changed untold", was.town);
        } else {
            prop_assert_eq!(now, was, "{:?} changed untold", was.town);
        }
        let owner = captured.iter().rev().find(|&&(t, _)| t == was.town);
        prop_assert_eq!(
            now.owner,
            owner.map_or(was.owner, |&(_, o)| Some(o)),
            "{:?}",
            was
        );
        let told = garrisons.iter().find(|(t, _)| *t == was.town);
        if let (Some(army), Some((_, told))) = (&now.garrison.army, told)
            && !ended
            && owner.is_none()
        {
            prop_assert_eq!(army, told, "{:?}'s garrison changed untold", was);
        }
    }
    for told in &views {
        let now = after.towns.iter().find(|t| t.town == told.town);
        prop_assert_eq!(now, Some(told), "{:?} told but not seen so", told.town);
    }
    let grew = ones(&after.adventure.explored) > ones(&before.adventure.explored);
    for now in &after.towns {
        let new = before.towns.iter().all(|t| t.town != now.town);
        prop_assert!(grew || !new, "{:?} shows untold", now);
    }
    let named = |o: &ObjectView| (o.object_type, o.anchor);
    for object in &after.adventure.objects {
        prop_assert!(!removed.contains(&named(object)), "{:?} removed", object);
    }
    for object in &before.adventure.objects {
        // A monster it was told changed shows its new word, and nothing else changed.
        let told = words
            .iter()
            .rev()
            .find(|(name, _)| *name == named(object))
            .map(|&(_, word)| {
                let mut now = object.clone();
                if let Some(monster) = &mut now.monster {
                    monster.count_word = word;
                }
                now
            });
        prop_assert!(
            told.as_ref().map_or_else(
                || after.adventure.objects.contains(object),
                |now| after.adventure.objects.contains(now)
            ) || removed.contains(&named(object))
                || flagged.contains(&named(object)),
            "{:?} left untold",
            object
        );
    }
    for &(name, creature, count_word) in &added {
        let shown = after.adventure.objects.iter().any(|o| {
            named(o) == name
                && o.monster
                    == Some(h3_adventure::MonsterView {
                        creature,
                        count_word,
                    })
        });
        // A later event of the same batch may have changed its word or taken it off the map.
        let later = removed.contains(&name) || words.iter().any(|(n, _)| *n == name);
        prop_assert!(shown || later, "{:?} added untold", name);
    }
    // The tiles a removed object the viewer saw covered: what covers there may thin out.
    let (width, height) = (
        usize::from(before.adventure.width),
        usize::from(before.adventure.height),
    );
    let mut freed = vec![false; before.adventure.tiles.len()];
    // An added monster's tile, which it now blocks and is visited from (E08 T85a).
    let mut covered = vec![false; before.adventure.tiles.len()];
    for &((_, a), _, _) in &added {
        covered[(usize::from(a.level) * height + usize::from(a.y)) * width + usize::from(a.x)] =
            true;
    }
    for object in before
        .adventure
        .objects
        .iter()
        .filter(|o| removed.contains(&named(o)))
    {
        let mask = TileMask::from_bits(object.blocked.bits() | object.visitable.bits());
        for (column, row) in mask.offsets() {
            let a = object.anchor;
            if let (Some(x), Some(y)) = (
                usize::from(a.x).checked_sub(usize::from(column)),
                usize::from(a.y).checked_sub(usize::from(row)),
            ) {
                freed[(usize::from(a.level) * height + y) * width + x] = true;
            }
        }
    }
    for (((was, is), freed), covered) in before
        .adventure
        .tiles
        .iter()
        .zip(&after.adventure.tiles)
        .zip(freed)
        .zip(covered)
    {
        prop_assert!(depth(was) <= depth(is));
        match (was, is) {
            (TileView::Seen(was), TileView::Seen(is)) if covered => {
                prop_assert_eq!(
                    (was.terrain, was.road, was.river),
                    (is.terrain, is.road, is.river)
                );
                prop_assert!(is.blocked && is.visitable);
            }
            (TileView::Seen(was), TileView::Seen(is)) if freed => {
                prop_assert_eq!(
                    (was.terrain, was.road, was.river),
                    (is.terrain, is.road, is.river)
                );
                prop_assert!(is.blocked <= was.blocked && is.visitable <= was.visitable);
            }
            _ if depth(was) == depth(is) => prop_assert_eq!(was, is),
            _ => {}
        }
    }
    if !own && after.day == before.day && !captor {
        prop_assert_eq!(&before.adventure.explored, &after.adventure.explored);
        prop_assert_eq!(&before.adventure.fringe, &after.adventure.fringe);
    } else if !own {
        // A new day's reveal (E08 T40), or (E09 T9b2a) a capture's in another's command: the
        // explored tiles only grow, the fringe follows them.
        let kept = |(was, is): (&u64, &u64)| was & !is == 0;
        prop_assert!(
            before
                .adventure
                .explored
                .iter()
                .zip(&after.adventure.explored)
                .all(kept)
        );
    }
    Ok(())
}

/// Whether objects of type `object_type` hire out creatures (E08 T96b1, increment 10's objects):
/// some(false) for a dwelling or a random dwelling (which the build resolves to a dwelling,
/// ADV-DWL-005), some(true) for a Refugee Camp, none for every other type.
fn hire_source(rules: &GameRuleset, object_type: h3_adventure::ObjectTypeId) -> Option<bool> {
    match rules.adventure().object_type(object_type).map(|t| &t.kind) {
        Some(ObjectKind::Dwelling { .. } | ObjectKind::RandomDwelling) => Some(false),
        Some(ObjectKind::RefugeeCamp { .. }) => Some(true),
        _ => None,
    }
}

/// What `viewer` is shown of the dwellings and the Refugee Camps of `state` (E08 T96b1; increment
/// 10's design note §10, ADV-OWN-003 amended; §14's "Properties"): `view` its observation and
/// `told` the events of the command that led there as it is told them. Each one it sees shows its
/// type, anchor and footprint and, a dwelling, its owner as the state holds it, a camp none:
/// nothing else (no monster, production, guards, visited mark or boat tile), so never a
/// dwelling's counts or guards nor a camp's creature or count; and none is among what it knows
/// by its visits. Those show only to the visitor's owner, by its decision and its own events: an
/// open hire or guards' question and a hire dialog (its counts and costs) are of a hero of its own
/// on the source's entrance, as an offer's and a dialog's opening there (`OfferOpened`,
/// `HireOpened`); the free creatures joined or refused and the purchases' joins (`CreaturesJoined`,
/// `JoinRefused`) are of a hero of its own; a free creature missing (`NoneToRecruit`) is at a
/// dwelling of its own on whose entrance a hero of its own stands, and a deserted camp
/// (`ObjectEmpty`) is one a hero of its own stands on.
/// Of a source any viewer that sees it is told only the flag, with the owner the state holds
/// (`ObjectFlagged`, never a camp's), and never a change of guards, of a monster's word or of a
/// mark (`GuardsChanged`, `MonsterChanged`, `ObjectMarked`: §10, "a battle's change of a
/// dwelling's guards needs no event"). The guards' battle shows as any battle (§10, ADR-0016
/// point 6), which [`check_told`] follows. The view properties check every player after every
/// command, so each of those is told to the visitor's owner alone.
fn check_hire_sources(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
    view: &GameObservation,
    told: &[GameEvent],
) -> Result<(), TestCaseError> {
    let adventure = state.adventure();
    let map = adventure.map();
    let public: Vec<ObjectView> = all_objects(state)
        .filter(|&(i, _)| !adventure.is_removed(i))
        .filter_map(|(i, o)| {
            let object_type = o.object_type?;
            let camp = hire_source(rules, object_type)?;
            o.tiles(map).any(|pos| sees(view, pos)).then(|| ObjectView {
                object_type,
                anchor: o.anchor,
                blocked: o.blocked,
                visitable: o.visitable,
                drawn: o.drawn,
                monster: None,
                owner: if camp { None } else { adventure.owner(i) },
                production: None,
                guards: None,
                visited: false,
                boat_tile: None,
                town: None,
            })
        })
        .collect();
    let shown: Vec<ObjectView> = view
        .adventure
        .objects
        .iter()
        .filter(|o| hire_source(rules, o.object_type).is_some())
        .cloned()
        .collect();
    prop_assert_eq!(shown, public);
    let known = &view.adventure.known;
    prop_assert!(
        known
            .iter()
            .all(|k| hire_source(rules, k.object_type).is_none()),
        "a dwelling or a camp known by a visit: {:?}",
        known
    );
    // The source of this type and anchor still on the map, if it is one.
    let source = |object_type, anchor| {
        hire_source(rules, object_type)?;
        let index = object_index(state, object_type, anchor)?;
        let object = adventure
            .object(index)
            .filter(|_| !adventure.is_removed(index));
        object.map(|o| (index, o))
    };
    let own = |hero: HeroId| adventure.hero(hero).filter(|h| h.owner == viewer);
    // Whether the viewer's hero `hero` stands on an entrance of the source.
    let visits = |hero: HeroId, object_type, anchor| {
        own(hero)
            .zip(source(object_type, anchor))
            .is_some_and(|(h, (_, o))| o.covers(h.pos).1)
    };
    // Whether one of the viewer's heroes stands on an entrance of the source.
    let stands = |object_type, anchor| {
        let source = source(object_type, anchor);
        adventure
            .heroes()
            .iter()
            .any(|h| h.owner == viewer && source.is_some_and(|(_, o)| o.covers(h.pos).1))
    };
    let hiring =
        |kind: &OfferKind| matches!(kind, OfferKind::Hire { .. } | OfferKind::Guarded { .. });
    match &view.decision {
        Some(DecisionView::Offer(offer)) if hiring(&offer.kind) => prop_assert!(
            visits(offer.hero, offer.object_type, offer.anchor),
            "{:?} open for the viewer without its visit",
            offer
        ),
        Some(DecisionView::Hire(hire)) => {
            let h3_game::HireTarget::Hero(hero) = hire.target;
            prop_assert!(
                visits(hero, hire.object_type, hire.anchor),
                "{:?} open for the viewer without its visit",
                hire
            );
        }
        _ => {}
    }
    for event in told {
        match *event {
            // A question's content is the decision's, above.
            GameEvent::OfferOpened {
                player,
                hero,
                object_type,
                anchor,
            } if hire_source(rules, object_type).is_some() => {
                prop_assert_eq!(player, viewer);
                prop_assert!(
                    visits(hero, object_type, anchor),
                    "{:?} told without the viewer's visit",
                    event
                );
            }
            GameEvent::HireOpened {
                player,
                target: h3_game::HireTarget::Hero(hero),
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(
                    visits(hero, object_type, anchor),
                    "{:?} told without the viewer's visit",
                    event
                );
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::CreaturesJoined { hero, .. }
                    | AdventureEventKind::JoinRefused { hero, .. },
                ..
            }) => prop_assert!(
                own(hero).is_some(),
                "{:?} of a hero not the viewer's",
                event
            ),
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::NoneToRecruit {
                        object_type,
                        anchor,
                        ..
                    },
                ..
            }) => {
                let owner = source(object_type, anchor).and_then(|(i, _)| adventure.owner(i));
                prop_assert_eq!(owner, Some(viewer), "{:?}", event);
                prop_assert!(
                    stands(object_type, anchor),
                    "{:?} told without the viewer's visit",
                    event
                );
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectEmpty {
                        object_type,
                        anchor,
                    },
                ..
            }) if hire_source(rules, object_type).is_some() => prop_assert!(
                stands(object_type, anchor),
                "{:?} told without the viewer's visit",
                event
            ),
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectFlagged {
                        object_type,
                        anchor,
                        owner,
                    },
                ..
            }) if hire_source(rules, object_type).is_some() => {
                let flagged = source(object_type, anchor).map(|(i, _)| adventure.owner(i));
                prop_assert_eq!(flagged, Some(Some(owner)), "{:?}", event);
                prop_assert_eq!(hire_source(rules, object_type), Some(false), "{:?}", event);
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::GuardsChanged { object_type, .. }
                    | AdventureEventKind::MonsterChanged { object_type, .. }
                    | AdventureEventKind::ObjectMarked { object_type, .. },
                ..
            }) => prop_assert!(
                hire_source(rules, object_type).is_none(),
                "{:?} of a dwelling or a camp",
                event
            ),
            _ => {}
        }
    }
    Ok(())
}

/// What `viewer` is shown of the towns of `state` (E09 T9b2a; the town epic's design note §12,
/// §17; TWN-VEW-001 to -003; `docs/06-security.md`, "What a viewer may know: a town"), `view` its
/// observation, read apart from the game's views, from the state, the store and the town part:
/// each object it sees names its town if it is one; each town's object it sees, as any object (an
/// explored tile, or under ADV-QUIRK-1 its fringe), shows its type, anchor, footprint, owner and
/// `TownId` and nothing else (no monster, production, guards, visited mark or boat tile); and each
/// town a tile of which it has explored has a view, in `TownId` order, with the town's anchor,
/// owner, faction, name, fort level, Capitol and day's mark as they stand (E09 T18b2a: the mark
/// public, the record's sets to the owner alone, TWN-VEW-004) and its garrison by the viewer's
/// level. The level is the top one ([`OWNER_LEVEL`]) for a town of the viewer's team, or whose
/// entrance one of its own heroes with a Visions creature in its army sees (on the entrance's
/// level, nearer than the part's factor times the hero's Power, at least the part's least
/// radius); else the number of its own towns with a standing Tavern, at most [`TAVERN_LEVELS`].
/// By the level: no stack (0), each stack's creature in slot order (1), its count's word (2), its
/// count (the top); the garrison by slot to the owner's team alone, the hall and the day's income
/// at the top alone. A town only on the fringe has its object but no view.
fn check_town_views(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
    view: &GameObservation,
) -> Result<(), TestCaseError> {
    let adventure = state.adventure();
    let map = adventure.map();
    let (towns, part) = (state.towns(), rules.towns());
    for o in &view.adventure.objects {
        let index = object_index(state, o.object_type, o.anchor);
        prop_assert_eq!(o.town, index.and_then(|i| map.town_id(i)), "{:?}", o);
    }
    let ids = (0..map.towns().len()).map(|t| TownId::new(u16::try_from(t).unwrap()));
    let own = |town: TownId| map.town_object(town).and_then(|i| adventure.owner(i)) == Some(viewer);
    let taverns = ids
        .clone()
        .filter(|&town| own(town))
        .filter(|&town| {
            towns
                .get(town)
                .is_some_and(|t| h3_towns::has_tavern(t, part))
        })
        .count();
    let taverns = u8::try_from(taverns.min(usize::from(TAVERN_LEVELS))).unwrap();
    // Its own heroes that see towns by Visions, each with its radius (TWN-VEW-003).
    let seers: Vec<(TilePos, u32)> = part
        .params()
        .into_iter()
        .flat_map(|params| {
            let heroes = adventure.heroes().iter().filter(move |h| {
                h.owner == viewer
                    && h.army
                        .iter()
                        .any(|s| params.visions_creatures.contains(&s.creature))
            });
            heroes.map(move |h| {
                let power = state
                    .hero(h.id)
                    .map_or(0, |d| h3_heroes::power(d, rules.core()));
                let radius = u32::from(params.visions_factor) * u32::from(power);
                (h.pos, radius.max(u32::from(params.visions_min_radius)))
            })
        })
        .collect();
    let sighted = |entrance: TilePos| {
        seers.iter().any(|&(pos, radius)| {
            let (dx, dy) = (
                u32::from(pos.x.abs_diff(entrance.x)),
                u32::from(pos.y.abs_diff(entrance.y)),
            );
            pos.level == entrance.level && dx * dx + dy * dy < radius * radius
        })
    };
    let words = rules.adventure().params().monsters();
    let mut expected = Vec::new();
    for town in ids {
        let index = map.town_object(town);
        prop_assert!(index.is_some(), "{:?} has no object", town);
        let index = index.unwrap();
        let object = &map.objects()[index];
        let owner = adventure.owner(index);
        let public = ObjectView {
            object_type: object.object_type.unwrap(),
            anchor: object.anchor,
            blocked: object.blocked,
            visitable: object.visitable,
            drawn: object.drawn,
            monster: None,
            owner,
            production: None,
            guards: None,
            visited: false,
            boat_tile: None,
            town: Some(town),
        };
        let shown = object.tiles(map).any(|pos| sees(view, pos));
        let listed = view.adventure.objects.iter().find(|o| o.town == Some(town));
        prop_assert_eq!(listed, shown.then_some(&public), "{:?}", town);
        if !object
            .tiles(map)
            .any(|pos| adventure.is_explored(viewer, pos))
        {
            continue;
        }
        let record = towns.get(town);
        prop_assert!(record.is_some(), "{:?} has no record", town);
        let record = record.unwrap();
        let ours = owner.is_some_and(|o| allied(o, viewer));
        let top = ours || object.entrance(map).is_some_and(sighted);
        let level = if top { OWNER_LEVEL } else { taverns };
        let army = adventure.town_garrison(index).unwrap_or_default();
        let stacks = army
            .iter()
            .filter(|_| level > 0)
            .map(|s| GarrisonStackView {
                creature: s.creature,
                count_word: (level >= 2).then(|| words.map_or(0, |w| w.count_word(s.count))),
                count: (level == OWNER_LEVEL).then_some(s.count),
            })
            .collect();
        expected.push(TownView {
            town,
            anchor: object.anchor,
            owner,
            faction: record.faction(),
            name: towns.town_static(town).and_then(|s| s.name.clone()),
            fort: h3_towns::fort_level(record, part),
            capitol: towns.capitol(town, part),
            built_today: record.built_today(),
            garrison: GarrisonView {
                level,
                stacks,
                army: ours.then(|| army.to_vec()),
            },
            owner_facts: top.then(|| TownOwnerView {
                hall: h3_towns::hall_level(record, part),
                income: h3_towns::daily_income(record, part),
            }),
            buildings: (owner == Some(viewer)).then(|| TownBuildingsView {
                built: record.built(),
                forbidden: record.forbidden(),
            }),
        });
    }
    prop_assert_eq!(&view.towns, &expected);
    Ok(())
}

/// The tiles of `player`'s heroes, by id.
fn tiles(game: &GameState, player: PlayerId) -> Vec<(HeroId, TilePos)> {
    let heroes = game.adventure().heroes().iter();
    heroes
        .filter(|h| h.owner == player)
        .map(|h| (h.id, h.pos))
        .collect()
}

/// Whether a command was accepted, each hero's move (id, columns, rows, levels), the steps told,
/// the offers opened (0), refused (1), declined (2) or accepted (3) and (E09 T18c's inbox) the
/// other decisions opened (4 to 10, by kind) or closed (11), (E08 T19a) every
/// battle event, start and end, and the heroes removed, (E08 T31) whom the game waits for,
/// (E09 T9b2a) the guards' hints it changed and (E09 T9b2b) the towns' Capitols it changed.
type Outcome = (
    bool,
    Vec<(HeroId, i32, i32, i32)>,
    usize,
    Vec<u8>,
    Vec<GameEvent>,
    Option<PlayerId>,
    Vec<AdventureEventKind>,
    Vec<(TownId, bool)>,
);

/// What a command of `player` did in a game, as the hygiene test compares it between two games
/// (E08 T14b's review): whether it was accepted, how far each of the player's heroes moved from
/// its tile `before` (columns, rows, levels), how many steps the game told, and the decisions it
/// opened, refused or closed, every kind (a case of `master` found by CI, 2026-11-10: another
/// player's hero accepted a monster's offer to join beside a mine the viewer saw; its army, which
/// the test changes where the viewer never sees it, had room in one game, where the creatures
/// joined and the mine's visit flagged it at once, and not in the other, where an army dialog
/// opened and held the visit, as the original's does); and whom the game then waits for (E08 T31: a level-up open holds
/// the game, which every player learns, never the level-up, ADR-0017 point 6). Where the heroes
/// stand is left out: the test may put them on other tiles in the other game from the start.
/// (E09 T9b2a) And the guards' hints it changed (`GuardsChanged`'s kind, not who sees it): guards
/// a hero leaves at its mine come from its army, which the test may change where the viewer never
/// sees it, and their hint is every viewer's that sees the mine (ADV-OWN-003), as a battle shows
/// its side the other's stacks (a case of `master` before T9b2a: another player's hero left
/// guards of another creature in the other game at a mine the viewer saw). (E09 T9b2b) And the
/// Capitols it changed (`TownChanged`'s town and Capitol, since E09 T18b2a those told to `viewer`,
/// each viewer's: the events tell every viewer's views): a capture takes the town's Capitol away
/// when its new owner holds another (TWN-CAP-002), which may stand in a town the viewer never
/// sees, whose owner and buildings the test may change, and the change is told to every viewer
/// that sees the town.
fn outcome(
    (player, viewer, accepted): (PlayerId, PlayerId, bool),
    before: &[(HeroId, TilePos)],
    after: &GameState,
    events: &[GameEvent],
) -> Outcome {
    let moved = before
        .iter()
        .zip(tiles(after, player))
        .map(|(&(id, from), (_, to))| {
            let d = |a: u16, b: u16| i32::from(b) - i32::from(a);
            (
                id,
                d(from.x, to.x),
                d(from.y, to.y),
                i32::from(to.level) - i32::from(from.level),
            )
        })
        .collect();
    let steps = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::HeroStepped { .. },
                    ..
                })
            )
        })
        .count();
    let decisions = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::OfferOpened { .. } => Some(0),
            GameEvent::OfferRefused { .. } => Some(1),
            GameEvent::OfferAnswered { accepted, .. } => Some(2 + u8::from(*accepted)),
            GameEvent::LevelUpOpened { .. } => Some(4),
            GameEvent::MeetingOpened { .. } => Some(5),
            GameEvent::ArmyDialogOpened { .. } => Some(6),
            GameEvent::LeaveGuardsOpened { .. } => Some(7),
            GameEvent::ChoiceOpened { .. } => Some(8),
            GameEvent::UniversityOpened { .. } => Some(9),
            GameEvent::HireOpened { .. } => Some(10),
            GameEvent::DecisionClosed { .. } => Some(11),
            _ => None,
        })
        .collect();
    let battles = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::BattleStarted { .. }
                    | GameEvent::Battle { .. }
                    | GameEvent::BattleEnded { .. }
                    | GameEvent::Adventure(AdventureEvent {
                        kind: AdventureEventKind::HeroRemoved { .. },
                        ..
                    })
            )
        })
        .copied()
        .collect();
    let guards = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::Adventure(AdventureEvent {
                kind: kind @ AdventureEventKind::GuardsChanged { .. },
                ..
            }) => Some(*kind),
            _ => None,
        })
        .collect();
    let capitols = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::TownChanged {
                player,
                town,
                capitol,
                ..
            } if player == viewer => Some((town, capitol)),
            _ => None,
        })
        .collect();
    (
        accepted,
        moved,
        steps,
        decisions,
        battles,
        after.waiting_for(),
        guards,
        capitols,
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// Observation hygiene: two games that differ only in what a viewer never sees (the seed,
    /// the other players' armies and so their points, the tiles it never sees and what objects
    /// cover there, the objects' contents) show it the same observation, legal commands and moves
    /// after the same commands, accept or reject its own commands alike and tell it the same
    /// events, as long as the other players' commands have the same outcome in both ([`outcome`]:
    /// accepted in both or in neither, their heroes moved alike, the same steps and decisions; a
    /// move that one hero can pay and the other cannot, or that visits a pile from its neighbour
    /// in one game and walks onto the pile's tile in the other, which lacks it, ends where the
    /// viewer may see it), and until
    /// the viewer picks up an object: a pickup tells its visitor the contents (E08 T13b; survey
    /// `E08-objects.md` §8), so the two games may then tell it other gains, and nothing else; or
    /// until it accepts an offer: a Cartographer reveals tiles by their terrain, hidden ones
    /// included, so the two games may then tell it other counts (E08 T14b). An offer opened,
    /// refused or declined tells it nothing hidden, nor does a key or a guard's passage, and an
    /// observatory's disk (T15a) is the same tiles in both games: the comparison goes on. A battle
    /// of the viewer's shows it the other side's stacks: the comparison ends when one starts (E08
    /// T19a); another player's battle must go alike in both games ([`outcome`]). Since E08 T31 the
    /// other players' heroes may have another custom experience in the other game, so other levels
    /// at load (their level seeds' draws, or the setup stream's) that never show; that the game
    /// waits for a player's level-up does show, so the game must wait alike ([`outcome`]). Random
    /// play seldom gets another player's battle out of sight to its end with the comparison still
    /// running: [`another_players_level_up_shows_nothing_of_its_offers`] plays one whose offers
    /// differ. Since E08 T85b a month's start ends the comparison: its kind and a creature
    /// month's monsters are public draws of the game's stream, which the hidden differences move.
    /// Since E08 T96b1 the other game may hold other owners at load of the dwellings the viewer
    /// never sees, fewer of them and of the camps, other camps' creatures (the seed) and, seldom,
    /// other counts after another player's visit (its free creatures joined in one game and
    /// refused for want of room in the other, its army changed); never other guards (another
    /// player's battle goes alike in both games, [`outcome`]; nothing else sets them). A question
    /// of the viewer's at a camp, or at a dwelling whose counts differ between the two games, ends
    /// the comparison when the two games' differ; at any other dwelling the two are the same. And
    /// every step checks what the viewer is shown of them in both games
    /// ([`check_hire_sources`]). Since E09 T9b2b the plans hold towns that draw nothing the viewer
    /// may see ([`hygienic`]), and the other game may hold other owners, garrisons and buildings
    /// of the towns the viewer never sees, fewer of them after every town it sees, more after
    /// every other ([`vary`]); a capture's Capitol taken away (TWN-CAP-002) must go alike in both
    /// games ([`outcome`]); and at the start and every step, what the viewer is shown of the towns
    /// in both games ([`check_town_views`]).
    #[test]
    fn a_viewer_learns_nothing_it_cannot_see(
        plan in hygienic(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in steps(),
        viewer in any::<u8>(),
        own in own_army(),
        unseen in unseen(),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut m = lay_out(&plan);
        let game = start_on(&m, rules, seed);
        let viewer = PlayerId::new(viewer % u8::try_from(game.players()).unwrap());
        let colour = game.colour(viewer).unwrap();
        // The viewer's heroes get armies and experience of their own: a starting army (also for a
        // hero whose slots are all empty) and a default experience are drawn from the seed (E08
        // T18b), and the viewer sees its heroes' details.
        for o in &mut m.objects {
            let Body::Hero(hero) = &mut o.body else { continue };
            if hero.owner == colour {
                if hero.army.iter().flatten().all(Option::is_none) {
                    hero.army = Some(stacks(&own));
                }
                hero.experience = Some(500);
            }
        }
        // What the viewer never sees, from the tiles it has explored at the end of the game (no
        // rule hides tiles) by the rules, not by the observation under test: a tile neither
        // explored nor next to an explored one shows nothing; one next to one (the fringe) its
        // ground, and what objects cover there only under ADV-QUIRK-1 (ADV-FOG-005).
        let mut end = start_on(&m, rules, seed);
        for step in &steps {
            let legal = options(&end, rules);
            let (actor, command) = resolve(&end, rules, &legal, step);
            let _ = apply(&mut end, rules, actor, command, &mut Vec::new());
        }
        let explored = |x: i16, y: i16, z: u8| {
            u16::try_from(x)
                .ok()
                .zip(u16::try_from(y).ok())
                .is_some_and(|(x, y)| end.adventure().is_explored(viewer, TilePos::new(x, y, z)))
        };
        let near = |(x, y, z): Spot| {
            let (x, y) = (i16::from(x), i16::from(y));
            (-1..=1).any(|dy| (-1..=1).any(|dx| explored(x + dx, y + dy, z)))
        };
        let other = vary(
            &m,
            colour,
            &unseen,
            |spot| !near(spot),
            |(x, y, z)| !explored(x.into(), y.into(), z) && (strict || !near((x, y, z))),
        );
        let mut a = start_on(&m, rules, seed);
        let mut b = start_on(&other, rules, unseen.seed.unwrap_or(seed));
        check_same(&a, &b, rules, viewer)?;
        // E09 T9b2b: what the viewer is shown of the towns, in both games.
        for game in [&a, &b] {
            check_town_views(game, rules, viewer, &observe(game, rules, Viewer::Player(viewer)))?;
        }
        for step in &steps {
            let legal = options(&a, rules);
            let (actor, command) = resolve(&a, rules, &legal, step);
            let (mut ea, mut eb) = (Vec::new(), Vec::new());
            // The system's commands play a monster's battle against `player`'s hero (E08
            // T20b3b1): they count as that player's.
            let player = player_of(&a, actor);
            let (from_a, from_b) = (tiles(&a, player), tiles(&b, player));
            let known_before = (known(&a, viewer), known(&b, viewer));
            let ra = apply(&mut a, rules, actor, command.clone(), &mut ea);
            let visit = !matches!(command, GameCommand::Manage(_));
            let rb = apply(&mut b, rules, actor, command, &mut eb);
            if actor == Actor::Player(viewer) {
                prop_assert_eq!(ra, rb);
            } else if outcome((player, viewer, ra.is_ok()), &from_a, &a, &ea)
                != outcome((player, viewer, rb.is_ok()), &from_b, &b, &eb)
            {
                // The other player's command did not do the same in both games.
                break;
            }
            let (ta, tb) = (told(&ea, viewer), told(&eb, viewer));
            // E08 T96b1: what the viewer is shown of the dwellings and the camps, in both games
            // (the other's owners at load, armies and seed differ); (E09 T9b2b) and of the towns.
            for (game, events) in [(&a, &ta), (&b, &tb)] {
                let seen = observe(game, rules, Viewer::Player(viewer));
                check_hire_sources(game, rules, viewer, &seen, events)?;
                check_town_views(game, rules, viewer, &seen)?;
            }
            // A battle shows its side the other's stacks (survey `E08-battles.md` §9; E08
            // T19a): the comparison ends when the viewer's starts.
            let battle = |events: &[GameEvent]| {
                events
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
            };
            if battle(&ta) || battle(&tb) {
                break;
            }
            // A monster's answer follows from its aggression, drawn from the seed and never
            // shown (ADV-MON-002, ADV-MON-007; E08 T20b3b1): the comparison ends when the viewer
            // is told a monster's offer, as at a battle (a dwelling's or a camp's question: below).
            let answered = |events: &[GameEvent]| {
                events.iter().any(|e| match *e {
                    GameEvent::OfferOpened { object_type, .. } => matches!(
                        rules.adventure().object_type(object_type).map(|t| &t.kind),
                        Some(ObjectKind::Monster { .. })
                    ),
                    _ => false,
                })
            };
            if answered(&ta) || answered(&tb) {
                break;
            }
            // E08 T85b: a month's kind and a creature month's monsters are public draws of the
            // game's stream (GAM-CAL-003, ADV-MON-014), whose seed and position the differences
            // hidden from the viewer change (the seed, the weekly objects' draws on its unseen
            // tiles): the comparison ends at a month's start, as at a battle.
            let month = |events: &[GameEvent]| {
                events
                    .iter()
                    .any(|e| matches!(e, GameEvent::MonthStarted { .. }))
            };
            if month(&ta) || month(&tb) {
                break;
            }
            let bought = ta
                .iter()
                .any(|e| matches!(e, GameEvent::OfferAnswered { accepted: true, .. }));
            // E08 T64b: a visit of increment 7's objects shows the visitor what the build drew
            // (a chest's gold, a Shrine's spell, a Tree's price, a University's skills, a random
            // artifact; increment 7's design note §13): its gains, a choice or University opened, a
            // price refused, or what its owner now knows.
            let shows = |events: &[GameEvent]| {
                visit
                    && events.iter().any(|e| {
                        matches!(
                            e,
                            GameEvent::HeroChanged { .. }
                                | GameEvent::ChoiceOpened { .. }
                                | GameEvent::UniversityOpened { .. }
                                | GameEvent::OfferRefused { .. }
                                | GameEvent::LevelUpOpened { .. }
                        )
                    })
            };
            let learned =
                known(&a, viewer) != known_before.0 || known(&b, viewer) != known_before.1;
            if actor == Actor::Player(viewer)
                && (ta != tb || bought || shows(&ta) || shows(&tb) || learned)
            {
                // The viewer learned a pickup's contents or bought a reveal: only what it gained
                // and how many tiles it explored may differ; (E08 T64b) or a visit's contents:
                // what its hero gained and what the visit asked, opened (a chest's choice of
                // gold, not of an artifact) or refused for its price, so also how many of those
                // it was told.
                let blind = |events: Vec<GameEvent>| -> Vec<GameEvent> {
                    events
                        .into_iter()
                        .filter(|e| {
                            !matches!(
                                e,
                                GameEvent::ResourceGained { .. }
                                    | GameEvent::ResourcePaid { .. }
                                    | GameEvent::ResourceLost { .. }
                                    | GameEvent::HeroChanged { .. }
                                    | GameEvent::LevelUpOpened { .. }
                                    | GameEvent::LevelGained { .. }
                                    | GameEvent::OfferOpened { .. }
                                    | GameEvent::OfferRefused { .. }
                                    | GameEvent::ChoiceOpened { .. }
                                    | GameEvent::Adventure(AdventureEvent {
                                        kind: AdventureEventKind::TilesRevealed { .. },
                                        ..
                                    })
                            )
                        })
                        .collect()
                };
                prop_assert_eq!(blind(ta), blind(tb));
                break;
            }
            // E08 T96b1: a dwelling's or a Refugee Camp's question shows its visitor's owner, in
            // its decision, what the source holds: a camp's week's creature, a draw of the game's
            // stream (ADV-RFG-001) that the hidden differences move, or a dwelling's guards'
            // creature and word, the same in both games by construction (above). When the
            // viewer's own at a camp, or at a dwelling whose counts differ between the two games
            // (another player's visit, above), names another in the other game the comparison
            // ends, as at a visit's contents (its events, the same, name the object only); at any
            // other dwelling the two games' decisions are the same (a difference is a leak). One
            // that names the same goes on (a decline keeps the rest hidden, an acceptance ends
            // it, `bought`). Another player's is told to that player alone (`OfferOpened`): the
            // comparison goes on past it, its answer compared by [`outcome`] (a purchase of a
            // creature the other game's camp lacks is refused there, which ends it).
            let question = ta.iter().find_map(|e| match *e {
                GameEvent::OfferOpened {
                    object_type,
                    anchor,
                    ..
                } => hire_source(rules, object_type).map(|camp| (camp, object_type, anchor)),
                _ => None,
            });
            if let Some((camp, object_type, anchor)) = question {
                let decision =
                    |game: &GameState| observe(game, rules, Viewer::Player(viewer)).decision;
                let counts = |game: &GameState| {
                    let index = object_index(game, object_type, anchor)?;
                    game.adventure().dwelling_counts(index).map(<[u32]>::to_vec)
                };
                if camp || counts(&a) != counts(&b) {
                    if decision(&a) != decision(&b) {
                        prop_assert_eq!(ta, tb);
                        break;
                    }
                } else {
                    prop_assert_eq!(decision(&a), decision(&b), "{:?}", question);
                }
            }
            prop_assert_eq!(ta, tb);
            check_same(&a, &b, rules, viewer)?;
        }
    }

    /// The observation's consistency with the events, for every player at every step of random
    /// play on imported maps, in both profiles ([`check_told`]); (E08 T96b1) and what each is
    /// shown of the dwellings and the camps ([`check_hire_sources`]), at the start too; (E09
    /// T9b2a) on the plans' towns, and what each is shown of them ([`check_town_views`]).
    #[test]
    fn views_move_as_the_events_say(
        plan in plan(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in steps(),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start_on(&lay_out(&plan), rules, seed);
        let players: Vec<PlayerId> =
            (0..u8::try_from(state.players()).unwrap()).map(PlayerId::new).collect();
        let mut views: Vec<GameObservation> = players
            .iter()
            .map(|&p| observe(&state, rules, Viewer::Player(p)))
            .collect();
        for (&player, view) in players.iter().zip(&views) {
            check_hire_sources(&state, rules, player, view, &[])?;
            check_town_views(&state, rules, player, view)?;
        }
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command, &mut events);
            for (&player, view) in players.iter().zip(&mut views) {
                let after = observe(&state, rules, Viewer::Player(player));
                let own = Actor::Player(player) == actor && result.is_ok();
                let told = told(&events, player);
                check_told(player, (view, &after), &told, own)?;
                // E08 T96b1: the dwellings and the camps, to every player.
                check_hire_sources(&state, rules, player, &after, &told)?;
                // E09 T9b2a: the towns, to every player.
                check_town_views(&state, rules, player, &after)?;
                *view = after;
            }
        }
    }
}

/// What the town views' checks met (E09 T9b2a; [`town_views_move_as_the_events_say`]), over
/// every player's views after every command.
#[derive(Debug, Default)]
struct TownsMet {
    /// Views by the garrison's level, 0 to [`OWNER_LEVEL`].
    levels: [usize; 4],
    /// Views of a neutral town with its stacks (a level from the viewer's taverns).
    neutral: usize,
    /// Captures told to their captor, and to another viewer that saw the town.
    captures: [usize; 2],
    /// Capitols' changes told (TWN-CAP-002).
    capitols: usize,
    /// Changes of another team's or a neutral town's garrison told (E09 T18b2a, SRV-Q26: a
    /// siege's merge or battle the viewer does not fight).
    garrisons: usize,
    /// A merge's garrison slots told to the garrison's owner (TWN-SIE-002).
    merges: usize,
    /// Towns coming into a view.
    new: usize,
}

impl TownsMet {
    fn count(
        &mut self,
        viewer: PlayerId,
        (before, after): (&GameObservation, &GameObservation),
        told: &[GameEvent],
    ) {
        for now in &after.towns {
            self.levels[usize::from(now.garrison.level)] += 1;
            self.neutral += usize::from(now.owner.is_none() && !now.garrison.stacks.is_empty());
            self.new += usize::from(before.towns.iter().all(|t| t.town != now.town));
        }
        for event in told {
            match *event {
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::TownCaptured { owner, .. },
                    ..
                }) => self.captures[usize::from(owner != viewer)] += 1,
                GameEvent::TownChanged {
                    town,
                    capitol,
                    slots,
                    ..
                } => {
                    let was = before.towns.iter().find(|t| t.town == town);
                    let now = after.towns.iter().find(|t| t.town == town);
                    self.capitols += usize::from(was.is_some_and(|t| t.capitol != capitol));
                    let garrison = was.zip(now).is_some_and(|(w, n)| w.garrison != n.garrison);
                    self.garrisons += usize::from(garrison && !slots);
                }
                GameEvent::StackChanged { at, .. } => {
                    self.merges += usize::from(matches!(at.army, ArmyRef::Garrison(_)));
                }
                _ => {}
            }
        }
    }
}

/// The town views over plans of three to six towns (E09 T9b2a; the draw of `play.rs`'s
/// `the_plans_capture_and_besiege_towns`): over the first 256 cases of proptest's deterministic
/// runner, each player's view moves as the events say ([`check_told`]) and shows the towns as they
/// stand ([`check_town_views`]), and the cases meet each of what those follow ([`TownsMet`]):
/// views at every level, a neutral garrison seen with its stacks, captures told to their captor
/// and to another viewer, a Capitol's change, a merge's garrison slots and a town coming into
/// view. (A siege's effects on a garrison a third viewer sees by its taverns, which show untold,
/// `docs/06-security.md`, were met in none of these cases: a lost siege at a town such a viewer
/// sees is rare in random play.)
#[test]
fn town_views_move_as_the_events_say() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    // The first two towns the first player's, the first of the standard set (with its Tavern), so
    // that the player often sees others' towns at the taverns' second level, or holds a Capitol.
    let towns = (plan(), vec(super::play::town(), 3..=6)).prop_map(|(plan, mut towns)| {
        for town in towns.iter_mut().take(2) {
            town.owner = Some(0);
            town.visited = false;
        }
        towns[0].buildings %= 2;
        super::play::Plan { towns, ..plan }
    });
    let strategy = (towns, any::<bool>(), any::<u64>(), steps());
    let mut met = TownsMet::default();
    for _ in 0..512 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start_on(&lay_out(&plan), rules, seed);
        let players: Vec<PlayerId> = (0..u8::try_from(state.players()).unwrap())
            .map(PlayerId::new)
            .collect();
        let mut views: Vec<GameObservation> = players
            .iter()
            .map(|&p| observe(&state, rules, Viewer::Player(p)))
            .collect();
        for (&player, view) in players.iter().zip(&views) {
            check_town_views(&state, rules, player, view).unwrap();
        }
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command, &mut events);
            for (&player, view) in players.iter().zip(&mut views) {
                let after = observe(&state, rules, Viewer::Player(player));
                let own = Actor::Player(player) == actor && result.is_ok();
                let told = told(&events, player);
                check_told(player, (view, &after), &told, own).unwrap();
                check_town_views(&state, rules, player, &after).unwrap();
                met.count(player, (view, &after), &told);
                *view = after;
            }
        }
    }
    eprintln!("{met:?}");
    let m = &met;
    assert!(m.levels.iter().all(|&n| n > 0) && m.neutral > 0, "{m:?}");
    assert!(m.captures.iter().all(|&n| n > 0) && m.capitols > 0, "{m:?}");
    assert!(m.garrisons > 0, "{m:?}");
    assert!(m.merges > 0 && m.new > 0, "{m:?}");
}

/// [`vary`] changes what it may and nothing else: with every tile unseen, each kind of change
/// shows on a known plan; with none, only the other players' armies change, and (E08 T31) their
/// heroes' custom experience, where they have one (not on SoD's plans).
#[test]
fn vary_changes_what_the_viewer_never_sees_only() {
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 8,
        two_levels: false,
        tiles: vec![(0, 0, 0); 64],
        players: vec![(0, true), (1, true)],
        heroes: vec![((1, 1, 0), None), ((6, 6, 0), None)],
        hero_types: 0,
        backpacks: vec![],
        skills: vec![],
        kits: vec![],
        nomads: None,
        blockers: vec![((4, 1, 0), 0), ((1, 5, 0), 1)],
        difficulty: 1,
        exchange: None,
        events: Vec::new(),
        owners: Vec::new(),
        teleporters: Vec::new(),
        towns: Vec::new(),
    };
    let m = lay_out(&plan);
    let unseen = Unseen {
        seed: None,
        armies: vec![Some(Some(vec![Some((10, 3))]))],
        tiles: vec![Some((4, 1, 2)); 64],
        removed: vec![true],
        added: vec![((3, 6, 0), 0)],
        moved: vec![Some((6, 2, 0))],
        experience: vec![Some(3_000)],
        owners: Vec::new(),
        towns: Vec::new(),
        added_towns: Vec::new(),
    };
    let other = vary(&m, 0, &unseen, |_| true, |_| true);
    // Every tile is swamp with a road and a river but the heroes', which stay land as swamp is.
    assert!(
        other
            .terrain
            .tiles
            .iter()
            .all(|t| (t.terrain, t.road, t.river) == (4, 1, 2))
    );
    let blockers = |m: &RawMap| -> Vec<(u8, u8, u32)> {
        m.objects
            .iter()
            .filter(|o| o.template != HERO_TEMPLATE)
            .map(|o| (o.pos.x, o.pos.y, o.template))
            .collect()
    };
    assert_eq!(blockers(&m), [(4, 1, 1), (1, 5, 2)]);
    assert_eq!(blockers(&other), [(3, 6, 1)]);
    let mut seen = other.clone();
    let mine: Vec<_> = heroes(&mut seen)
        .map(|(owner, army)| (owner, army.clone()))
        .collect();
    assert_eq!(mine[0], (0, None));
    assert_eq!(mine[1], (1, Some(stacks(&[Some((10, 3))]))));
    let tiles: Vec<Spot> = other
        .objects
        .iter()
        .filter(|o| o.template == HERO_TEMPLATE)
        .map(standing_on)
        .collect();
    assert_eq!(tiles, [(1, 1, 0), (6, 2, 0)]);
    // Nothing unseen: only the other player's army changes (armies never show).
    let same = vary(&m, 0, &unseen, |_| false, |_| false);
    assert_eq!(same.terrain.tiles, m.terrain.tiles);
    assert_eq!(blockers(&same), blockers(&m));
    let mut kept = same.clone();
    let armies: Vec<_> = heroes(&mut kept).map(|(_, army)| army.clone()).collect();
    assert_eq!(armies, [None, Some(stacks(&[Some((10, 3))]))]);
    // The heroes' experience: none of their own on SoD's plans, kept; RoE's write 0 for every
    // hero, and the other player's gets its new one.
    let experience = |m: &RawMap| -> Vec<Option<u32>> {
        m.objects
            .iter()
            .filter_map(|o| match &o.body {
                Body::Hero(hero) => Some(hero.experience),
                _ => None,
            })
            .collect()
    };
    assert_eq!(experience(&other), [None, None]);
    let roe = lay_out(&super::play::Plan {
        version: crate::model::Version::RoE,
        ..plan
    });
    let varied = vary(&roe, 0, &unseen, |_| false, |_| false);
    assert_eq!(experience(&varied), [Some(0), Some(3_000)]);
}

/// [`vary`] and a random dwelling (E08 T96b1): the build resolves it to a dwelling whose tiles may
/// lie past its template's, at a moved anchor ([`object_reach`]), and the viewer sees it wherever
/// it sees one of those. Red's at load: with every tile unseen by Blue it is removed or made
/// neutral; with one tile past its template's seen it stays, Red's; Red's own stays its own.
#[test]
fn vary_keeps_a_random_dwelling_whose_resolved_tiles_may_show() {
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 8,
        two_levels: false,
        tiles: vec![(0, 0, 0); 64],
        players: vec![(0, true), (1, true)],
        heroes: vec![((1, 1, 0), None), ((6, 6, 0), None)],
        hero_types: 0,
        backpacks: vec![],
        skills: vec![],
        kits: vec![],
        nomads: None,
        // A random dwelling of 216 (one faction, one level), visited on the left.
        blockers: vec![((5, 3, 0), BLOCKERS.len() * 92 + 1)],
        difficulty: 1,
        exchange: None,
        events: Vec::new(),
        owners: vec![Some(0)],
        teleporters: Vec::new(),
        towns: Vec::new(),
    };
    let m = lay_out(&plan);
    let random = |m: &RawMap| -> Vec<(Spot, u32)> {
        m.objects
            .iter()
            .filter_map(|o| match &o.body {
                Body::RandomDwelling(d) => Some(((o.pos.x, o.pos.y, o.pos.z), d.owner)),
                _ => None,
            })
            .collect()
    };
    assert_eq!(random(&m), [((5, 3, 0), 0)]);
    let o = m
        .objects
        .iter()
        .find(|o| matches!(o.body, Body::RandomDwelling(_)));
    let o = o.unwrap();
    let template = usize::try_from(o.template - 1).unwrap();
    let past: Vec<Spot> = object_reach(o, 8)
        .into_iter()
        .filter(|&spot| covers((5, 3, 0), template, 8).all(|c| c != spot))
        .collect();
    assert!(!past.is_empty(), "{:?}", object_reach(o, 8));
    let unseen = |removed: bool| Unseen {
        seed: None,
        armies: Vec::new(),
        tiles: Vec::new(),
        removed: vec![removed],
        added: Vec::new(),
        moved: Vec::new(),
        experience: Vec::new(),
        owners: vec![None],
        towns: Vec::new(),
        added_towns: Vec::new(),
    };
    let blue = |removed: bool, objects: &dyn Fn(Spot) -> bool| {
        random(&vary(&m, 1, &unseen(removed), |_| true, objects))
    };
    assert_eq!(blue(true, &|_| true), []);
    assert_eq!(blue(false, &|_| true), [((5, 3, 0), 0xFF)]);
    for removed in [true, false] {
        assert_eq!(blue(removed, &|spot| spot != past[0]), [((5, 3, 0), 0)]);
    }
    let red = vary(&m, 0, &unseen(true), |_| true, |_| true);
    assert_eq!(random(&red), [((5, 3, 0), 0)]);
}

/// [`vary`] keeps the tiles where a Shipyard's boat tile may lie (ADV-SHY-001; a case of
/// `master`'s found by E09 T9b2b's stress): with every tile unseen, the twelve offsets from the
/// entrance of a Shipyard anchored at (4, 4), visited from (3, 4), keep their dirt while the other
/// tiles turn to water (the heroes' and the entrance's to grass), a blocker on one of them stays
/// and none is added over one, while elsewhere a blocker is removed and another added.
#[test]
fn vary_keeps_a_shipyard_s_boat_tiles() {
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 8,
        two_levels: false,
        tiles: vec![(0, 0, 0); 64],
        players: vec![(0, true), (1, true)],
        heroes: vec![((1, 1, 0), None), ((6, 6, 0), None)],
        hero_types: 0,
        backpacks: vec![],
        skills: vec![],
        kits: vec![],
        nomads: None,
        blockers: vec![((4, 4, 0), SHIPYARD_OBJECT), ((2, 3, 0), 0), ((7, 0, 0), 0)],
        difficulty: 1,
        exchange: None,
        events: Vec::new(),
        owners: Vec::new(),
        teleporters: Vec::new(),
        towns: Vec::new(),
    };
    let m = lay_out(&plan);
    let unseen = Unseen {
        seed: None,
        armies: Vec::new(),
        tiles: vec![Some((8, 0, 0)); 64],
        removed: vec![true],
        added: vec![((1, 4, 0), 0), ((0, 7, 0), 0)],
        moved: Vec::new(),
        experience: Vec::new(),
        owners: Vec::new(),
        towns: Vec::new(),
        added_towns: Vec::new(),
    };
    let other = vary(&m, 0, &unseen, |_| true, |_| true);
    let dirt: Vec<(usize, usize)> = (0..64)
        .filter(|&i| other.terrain.tiles[i].terrain == 0)
        .map(|i| (i % 8, i / 8))
        .collect();
    let mut offsets: Vec<(usize, usize)> = shipyard_offsets()
        .iter()
        .map(|&(dx, dy)| {
            (
                3usize.wrapping_add_signed(dx.into()),
                4usize.wrapping_add_signed(dy.into()),
            )
        })
        .collect();
    offsets.sort_by_key(|&(x, y)| (y, x));
    assert_eq!(dirt, offsets);
    let grass = (0..64).filter(|&i| other.terrain.tiles[i].terrain == 2);
    assert_eq!(
        grass.map(|i| (i % 8, i / 8)).collect::<Vec<_>>(),
        [(1, 1), (3, 4), (6, 6)]
    );
    let blockers = |m: &RawMap| -> Vec<(u8, u8)> {
        let objects = m.objects.iter().filter(|o| o.template == 1);
        objects.map(|o| (o.pos.x, o.pos.y)).collect()
    };
    assert_eq!(blockers(&m), [(2, 3), (7, 0)]);
    assert_eq!(blockers(&other), [(2, 3), (0, 7)]);
}

/// [`vary`] and the towns (E09 T9b2b): Red's own town at (4, 12) keeps everything; where Red sees
/// none of its tiles, Blue's at (15, 12), the neutral one at (4, 2) and Blue's at (15, 3), whose
/// entrance Blue's hero stands on (its visitor at load), take another garrison and buildings and
/// the first two another owner, Blue's visited one keeping its own; a town is removed only after
/// every town Red sees; a town is added after every other where Red sees none of its tiles, on
/// none of another object's; the games build on every map so changed.
#[test]
fn vary_changes_the_towns_the_viewer_never_sees_only() {
    let town = |(at, hero): ((u8, u8), u8), owner, faction, visited| TownPlan {
        at: (at, hero),
        faction: Some(faction),
        owner,
        alignment: None,
        garrison: None,
        buildings: 0,
        tight: false,
        visited,
    };
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 16,
        two_levels: false,
        tiles: vec![(0, 0, 0); 256],
        players: vec![(0, true), (1, true)],
        // Red's, Blue's, Red's and Blue's.
        heroes: vec![
            ((2, 13, 0), None),
            ((13, 13, 0), None),
            ((2, 3, 0), None),
            ((13, 3, 0), None),
        ],
        hero_types: 0,
        backpacks: vec![],
        skills: vec![],
        kits: vec![],
        nomads: None,
        blockers: vec![],
        difficulty: 1,
        exchange: None,
        events: Vec::new(),
        owners: Vec::new(),
        teleporters: Vec::new(),
        // Each entrance a row above its hero, the last one on its hero's tile.
        towns: vec![
            town(((2, 0), 0), Some(0), 0, false),
            town(((2, 0), 1), Some(1), 1, false),
            town(((2, 0), 2), None, 2, false),
            town(((2, 0), 3), None, 3, true),
        ],
    };
    let m = lay_out(&plan);
    // Each town's anchor, owner, garrison and buildings.
    type Town = (Spot, u8, Option<Vec<Option<Stack>>>, Buildings);
    let towns = |m: &RawMap| -> Vec<Town> {
        m.objects
            .iter()
            .filter_map(|o| match &o.body {
                Body::Town(t) => Some((
                    (o.pos.x, o.pos.y, o.pos.z),
                    t.owner,
                    t.garrison.clone(),
                    t.buildings.clone(),
                )),
                _ => None,
            })
            .collect()
    };
    let standard = TOWN_BUILDINGS[0].clone();
    let at = [(4, 12, 0), (15, 12, 0), (4, 2, 0), (15, 3, 0)];
    let owners = [0, 1, u8::MAX, 1];
    let laid: Vec<Town> = at
        .iter()
        .zip(owners)
        .map(|(&at, owner)| (at, owner, None, standard.clone()))
        .collect();
    assert_eq!(towns(&m), laid);
    let pikemen = vec![Some((0, 5)), None, None, None, None, None, None];
    let unseen = |removed: bool, added_towns| Unseen {
        seed: None,
        armies: Vec::new(),
        tiles: Vec::new(),
        removed: vec![removed],
        added: Vec::new(),
        moved: Vec::new(),
        experience: Vec::new(),
        owners: vec![None],
        towns: vec![(Some(Some(pikemen.clone())), Some(2))],
        added_towns,
    };
    let build = |m: &RawMap| assert_eq!(start_on(m, &FAITHFUL, 1).players(), 2);
    // Nothing unseen: every town kept.
    assert_eq!(
        towns(&vary(
            &m,
            0,
            &unseen(true, Vec::new()),
            |_| false,
            |_| false
        )),
        laid
    );
    // Everything unseen: Red's kept, the others changed, Blue's visited one still Blue's.
    let changed = (Some(stacks(&pikemen)), TOWN_BUILDINGS[2].clone());
    let other = vary(&m, 0, &unseen(false, Vec::new()), |_| true, |_| true);
    let mut expected = vec![laid[0].clone()];
    for (&at, owner) in at[1..].iter().zip([u8::MAX, u8::MAX, 1]) {
        expected.push((at, owner, changed.0.clone(), changed.1.clone()));
    }
    assert_eq!(towns(&other), expected);
    build(&other);
    // Removed where unseen: all but Red's; with the neutral town seen, only the town after it.
    let seen = |town: Spot| move |spot: Spot| !town_covers(town, 16).any(|t| t == spot);
    let removed = vary(&m, 0, &unseen(true, Vec::new()), |_| true, |_| true);
    assert_eq!(towns(&removed), [laid[0].clone()]);
    build(&removed);
    let removed = vary(&m, 0, &unseen(true, Vec::new()), |_| true, seen(at[2]));
    assert_eq!(
        towns(&removed),
        [laid[0].clone(), expected[1].clone(), laid[2].clone()]
    );
    build(&removed);
    // Added after every other object, Blue's (the only playing colour but Red's), where Red sees
    // none of its tiles and no other object covers one.
    let added = |at: Spot| {
        let mut town = town(((0, 0), 0), Some(0), 3, true);
        town.buildings = 1;
        vec![(at, town)]
    };
    let other = vary(&m, 0, &unseen(false, added((8, 8, 0))), |_| true, |_| true);
    let mut with = expected.clone();
    // Its standard set with a Fort made the one without (`hygienic_town`: an owned town's
    // level-2 dwelling is drawn, its owner's to see, TWN-VEW-004).
    with.push(((8, 8, 0), 1, None, TOWN_BUILDINGS[0].clone()));
    assert_eq!(towns(&other), with);
    assert!(matches!(other.objects.last().unwrap().body, Body::Town(_)));
    build(&other);
    let other = vary(
        &m,
        0,
        &unseen(false, added((8, 8, 0))),
        |_| true,
        seen((8, 8, 0)),
    );
    assert_eq!(towns(&other), expected);
    let other = vary(
        &m,
        0,
        &unseen(false, added((16 + 15, 11, 0))),
        |_| true,
        |_| true,
    );
    assert_eq!(towns(&other), expected);
}

// covers: ADV-FOG-004, ADV-PIK-001, ADV-PIK-003
#[test]
fn a_typed_object_in_sight_shows_its_type_never_its_drawn_contents() {
    // Red's hero at (1, 1), with an army of its own, sees a wood pile of a drawn amount (object 3:
    // shape 0, kind 1) at (3, 1) and a campfire (object 9) at (1, 3); blue's hero is far away.
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 16,
        two_levels: false,
        tiles: vec![(0, 0, 0); 256],
        players: vec![(0, true), (1, true)],
        heroes: vec![
            (
                (1, 1, 0),
                Some(vec![Some((0, 10)), None, None, None, None, None, None]),
            ),
            ((14, 14, 0), None),
        ],
        hero_types: 0,
        backpacks: vec![],
        skills: vec![],
        kits: vec![],
        nomads: None,
        blockers: vec![((3, 1, 0), 3), ((1, 3, 0), 9)],
        difficulty: 1,
        exchange: None,
        events: Vec::new(),
        owners: Vec::new(),
        teleporters: Vec::new(),
        towns: Vec::new(),
    };
    let m = lay_out(&plan);
    let red = PlayerId::new(0);
    let contents = |seed| -> Vec<Vec<(h3_content::ResourceId, u32)>> {
        let game = start_on(&m, &FAITHFUL, seed);
        let objects = game.adventure().map().objects();
        objects.iter().map(|o| o.contents.clone()).collect()
    };
    let view = |seed| {
        let game = start_on(&m, &FAITHFUL, seed);
        observe(&game, &FAITHFUL, Viewer::Player(red)).adventure
    };
    let base = view(0);
    assert_eq!(base.objects.len(), 2);
    let part = FAITHFUL.adventure();
    assert_eq!(
        base.objects[0].object_type,
        part.object_type_by_key("base:pile_wood").unwrap()
    );
    assert_eq!(
        base.objects[1].object_type,
        part.object_type_by_key("base:campfire").unwrap()
    );
    // Some seeds draw other contents; the observation is the same for each.
    let drawn: Vec<_> = (0..20).map(contents).collect();
    assert!(drawn.iter().any(|c| *c != drawn[0]));
    for seed in 0..20 {
        assert_eq!(view(seed), base, "seed {seed}");
    }
}

/// Another player's level-up shows nothing of its draws (E08 T31, increment 4's design note §14;
/// GAM-DEC-002, ADR-0017 points 5 and 6), in both profiles: red's hero, at 990 experience, beats
/// eight savage wolf riders beside it, out of the sight of blue's hero underground, and levels; in
/// the other game it also knows Scholar, which no battle or move reads, so the same battle opens a
/// level-up with other offers for some seeds (drawn from its level seed in `faithful`, for most of
/// them; from red's stream in `strict-fixed`, for two of these sixteen). Blue is shown the same
/// observation, legal commands and moves and told the same events at every step: that the game
/// waits for red, never what red is offered or takes.
// covers: GAM-DEC-002
#[test]
fn another_players_level_up_shows_nothing_of_its_offers() {
    let plan = |scholar: bool| super::play::Plan {
        version: crate::model::Version::RoE,
        side: 12,
        two_levels: true,
        tiles: vec![(0, 0, 0); 2 * 12 * 12],
        players: vec![(0, true), (1, true)],
        heroes: vec![
            (
                (2, 2, 0),
                Some(vec![Some((13, 20)), None, None, None, None, None, None]),
            ),
            ((9, 9, 1), None),
        ],
        hero_types: 0,
        backpacks: vec![],
        // Estates, and Scholar in the other game.
        skills: vec![Some(if scholar {
            vec![(13, 1), (18, 1)]
        } else {
            vec![(13, 1)]
        })],
        kits: vec![],
        nomads: None,
        blockers: vec![((3, 2, 0), MONSTER_OBJECTS[3])],
        difficulty: 1,
        exchange: None,
        events: Vec::new(),
        owners: Vec::new(),
        teleporters: Vec::new(),
        towns: Vec::new(),
    };
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    let attack = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: vec![TilePos::new(3, 2, 0)],
    });
    let filter = |events: &[GameEvent], keep: fn(&GameEvent) -> bool| -> Vec<GameEvent> {
        events.iter().filter(|e| keep(e)).copied().collect()
    };
    for (rules, profile) in [(&*FAITHFUL, "faithful"), (&*STRICT, "strict-fixed")] {
        let mut differ = 0;
        for seed in 0..16 {
            let [mut a, mut b] = [false, true].map(|scholar| {
                start_on(&experienced(&plan(scholar), Some(&[990, 500])), rules, seed)
            });
            let mut levels = [Vec::new(), Vec::new()];
            check_same(&a, &b, rules, blue).unwrap();
            for turn in 0..40 {
                let (actor, command) = if turn == 0 {
                    (Actor::Player(red), attack.clone())
                } else {
                    resolve(&a, rules, &options(&a, rules), &Step::Fight(0))
                };
                let (mut ea, mut eb) = (Vec::new(), Vec::new());
                let ra = apply(&mut a, rules, actor, command.clone(), &mut ea);
                let rb = apply(&mut b, rules, actor, command, &mut eb);
                assert_eq!(ra, rb, "{profile}, seed {seed}");
                // The battle goes alike in both games.
                let battle = |e: &GameEvent| {
                    matches!(
                        e,
                        GameEvent::BattleStarted { .. }
                            | GameEvent::Battle { .. }
                            | GameEvent::BattleEnded { .. }
                    )
                };
                assert_eq!(filter(&ea, battle), filter(&eb, battle));
                let level = |e: &GameEvent| {
                    matches!(
                        e,
                        GameEvent::LevelUpOpened { .. } | GameEvent::LevelGained { .. }
                    )
                };
                levels[0].extend(filter(&ea, level));
                levels[1].extend(filter(&eb, level));
                assert_eq!(told(&ea, blue), told(&eb, blue), "{profile}, seed {seed}");
                assert_eq!(a.waiting_for(), b.waiting_for());
                check_same(&a, &b, rules, blue).unwrap();
            }
            assert!(!levels[0].is_empty(), "{profile}, seed {seed}: no level");
            differ += usize::from(levels[0] != levels[1]);
        }
        assert!(differ > 0, "{profile}");
    }
}

/// Increment 9's objects on an imported map (E08 T83b2b, ADR-0016 point 6, `docs/06-security.md`):
/// the economy map with Red's hero between the mines, a second Red hero by the Shipyard (moved one
/// column toward the sea, so that it has a boat tile) and Blue's hero below the mines. Every
/// viewer sees the owners and the abandoned mines' guards as the original's hint shows them; a
/// mine's production and a Shipyard's boat tile only their owner (E08 decision 229); nothing of
/// what the build drew for the abandoned mines, whatever the seed.
// covers: ADV-OWN-003, ADV-FOG-006
#[test]
fn increment_9_s_objects_show_their_owners_never_their_draws() {
    use super::economy::{economy_map, hero, read_back};
    let mut map = economy_map();
    map.objects[0] = hero((9, 7), 0, 0);
    map.objects[1] = hero((9, 13), 1, 1);
    map.objects.push(hero((26, 19), 0, 2));
    let crate::writer::MapObject::Owned(shipyard) = &mut map.objects[8] else {
        panic!("the Shipyard is the economy map's ninth object");
    };
    shipyard.tile = (29, 16);
    let m = read_back(&crate::writer::write(&map).unwrap());
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    let part = FAITHFUL.adventure();
    let view = |game: &GameState, player| {
        observe(game, &FAITHFUL, Viewer::Player(player))
            .adventure
            .objects
    };
    let at = |views: &[ObjectView], x, y| {
        views
            .iter()
            .find(|o| o.anchor == TilePos::new(x, y, 0))
            .cloned()
            .unwrap()
    };
    let game = start_on(&m, &FAITHFUL, 0);
    let state = game.adventure();
    let (reds, blues) = (view(&game, red), view(&game, blue));
    // The owners, to every viewer that sees the object: Red's sawmill and Lighthouse, Blue's
    // crystal cavern, the neutral abandoned mines and Shipyard.
    for (views, x, y, owner) in [
        (&reds, 6, 4, Some(red)),
        (&reds, 12, 4, Some(blue)),
        (&blues, 12, 4, Some(blue)),
        (&reds, 29, 10, Some(red)),
        (&reds, 6, 10, None),
        (&blues, 6, 10, None),
        (&reds, 29, 16, None),
    ] {
        assert_eq!(at(views, x, y).owner, owner, "({x}, {y})");
    }
    // A mine's production to its owner alone.
    assert!(state.production(part, 0).is_some() && state.production(part, 1).is_some());
    assert_eq!(at(&reds, 6, 4).production, state.production(part, 0));
    assert_eq!(at(&reds, 12, 4).production, None);
    assert_eq!(at(&blues, 12, 4).production, state.production(part, 1));
    // The abandoned mines' guards: their creature and the word of their count, never the count;
    // no production, the mines being neutral.
    let words = part.params().monsters().unwrap();
    for (index, x) in [(3, 6), (4, 12)] {
        let guards = state.mine_guards(part, index);
        let expected = Some(GuardsView {
            creature: Some(guards[0].creature),
            count_word: words.count_word(guards[0].count),
            army: None,
        });
        for views in [&reds, &blues] {
            let mine = at(views, x, 10);
            assert_eq!((mine.guards, mine.production), (expected.clone(), None));
        }
    }
    // The Shipyard's boat tile, on the sea, to its owner's team alone (E08 decision 229): nobody's
    // while it is neutral, Red's once it is Red's, never Blue's.
    let tile = state.map().objects()[6].resolved.boat_tile();
    assert_eq!(tile, Some(TilePos::new(30, 16, 0)));
    assert_eq!(at(&reds, 29, 16).boat_tile, None);
    let mut reds_yard = map.clone();
    let crate::writer::MapObject::Owned(shipyard) = &mut reds_yard.objects[8] else {
        panic!("the Shipyard is the economy map's ninth object");
    };
    shipyard.body = crate::writer::OwnedBody::Owner(Some(0));
    let reds_yard = start_on(
        &read_back(&crate::writer::write(&reds_yard).unwrap()),
        &FAITHFUL,
        0,
    );
    assert_eq!(at(&view(&reds_yard, red), 29, 16).boat_tile, tile);
    assert!(view(&reds_yard, blue).iter().all(|o| o.boat_tile.is_none()));
    // Other seeds draw other resources and guards; neither player's view changes.
    let mut drawn = std::collections::BTreeSet::new();
    for seed in 0..16 {
        let game = start_on(&m, &FAITHFUL, seed);
        drawn.insert(game.adventure().map().objects()[3].resolved.mine());
        assert_eq!(view(&game, red), reds, "seed {seed}");
        assert_eq!(view(&game, blue), blues, "seed {seed}");
    }
    assert!(drawn.len() > 1);
}

/// What the viewers are told of the boat plans' games (E08 T97b2; survey `E08-dwellings.md`
/// §17): over the first 128 cases of `play.rs`'s `afloat_plans` (proptest's deterministic runner,
/// as `the_plans_visit_dwellings_from_a_boat`'s), each player's view moves as the events it is
/// told say ([`check_told`]) and shows the dwellings and camps as they stand
/// ([`check_hire_sources`]), dwellings visited from a boat among them, and (E09 T9b2a) the towns
/// ([`check_town_views`]). Its 117th case, while the plans' towns were left out (before E09
/// T9b2a), was a hero that sails out of a viewer's sight, lands where the viewer does not see its
/// boat's tile and walks back into it: its boat shows as it stands (`check_told`'s heroes stepping
/// into sight).
#[test]
fn boat_visits_are_told_as_they_happen() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    // E09 T9b2a: with the plans' towns (T9b1's), whose views follow the events too.
    let strategy = super::play::afloat_plans();
    for _ in 0..128 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start_on(&lay_out(&plan), rules, seed);
        let players: Vec<PlayerId> = (0..u8::try_from(state.players()).unwrap())
            .map(PlayerId::new)
            .collect();
        let mut views: Vec<GameObservation> = players
            .iter()
            .map(|&p| observe(&state, rules, Viewer::Player(p)))
            .collect();
        for (&player, view) in players.iter().zip(&views) {
            check_hire_sources(&state, rules, player, view, &[]).unwrap();
            check_town_views(&state, rules, player, view).unwrap();
        }
        for (step, sea) in &steps {
            let mut legal = options(&state, rules);
            if *sea {
                legal = at_sea(&state, legal);
            }
            let (actor, command) = resolve(&state, rules, &legal, step);
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command, &mut events);
            for (&player, view) in players.iter().zip(&mut views) {
                let after = observe(&state, rules, Viewer::Player(player));
                let own = Actor::Player(player) == actor && result.is_ok();
                let told = told(&events, player);
                check_told(player, (view, &after), &told, own).unwrap();
                check_hire_sources(&state, rules, player, &after, &told).unwrap();
                check_town_views(&state, rules, player, &after).unwrap();
                *view = after;
            }
        }
    }
}
