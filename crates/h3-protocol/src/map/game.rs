//! The whole game's mapping (E16 M5 part 1, SRV-Q25; ADR-0018 points 5 and 7 as amended): the
//! engine's observation of a game on the wire ([`crate::wire::game`]). Every field of the
//! engine's types is named in a pattern, without `..`, and every enum is matched without a
//! wildcard: a new field or variant fails this crate's build until it is mapped. What leaves is
//! projected by construction: [`adventure`] takes the viewer and calls the engine's `observe`.
//! M5 T2a: the adventure map; T2b the rest of the observation.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureObservation, BoatView, GuardsView, HeroView, Known as EngineKnown, KnownObject,
    MonsterView, ObjectTypeId, ObjectView, OwnHeroView, StaticTile, TileMask, TileView,
};
use h3_content::{Formation as EngineFormation, ResourceId};
use h3_contracts::ArmyStack as EngineStack;
use h3_game::{GameRuleset, GameState, Viewer};
use h3_kernel::handle::Handle;

use crate::wire::WireU64;
use crate::wire::game::{
    Adventure, Amount, ArmyStack, Boat, Formation, Guards, Known, KnownObject as WireKnownObject,
    MapHero, Monster, Object, ObjectRef, OwnMapHero, Pos, Tile,
};

/// The adventure map as `viewer` sees it, on the wire (ADV-FOG-004 to ADV-FOG-006).
pub fn adventure(state: &GameState, rules: &GameRuleset, viewer: Viewer) -> Adventure {
    let Viewer::Player(player) = viewer;
    adventure_view(
        &state
            .adventure()
            .observe(rules.core(), rules.adventure(), player),
    )
}

/// An adventure observation on the wire.
pub(crate) fn adventure_view(o: &AdventureObservation) -> Adventure {
    let AdventureObservation {
        viewer,
        width,
        height,
        levels,
        stride,
        explored,
        fringe,
        tiles,
        heroes,
        objects,
        bought,
        keys,
        known,
        boats,
    } = o;
    let (palette, tiles) = packed(tiles);
    Adventure {
        viewer: viewer.raw(),
        width: *width,
        height: *height,
        levels: *levels,
        // At most 1 024: a width is a `u16`.
        stride: u16::try_from(*stride).unwrap_or(u16::MAX),
        explored: words(explored),
        fringe: words(fringe),
        palette,
        tiles,
        heroes: heroes.iter().map(map_hero).collect(),
        objects: objects.iter().map(object).collect(),
        bought: bought.iter().map(|&t| t.raw()).collect(),
        keys: *keys,
        known: known.iter().map(known_object).collect(),
        boats: boats.iter().map(boat).collect(),
    }
}

/// The tiles as their distinct views, in the order of their first use, and each tile's index
/// among them.
fn packed(tiles: &[TileView]) -> (Vec<Tile>, Vec<u32>) {
    let mut index = BTreeMap::new();
    let mut palette = Vec::new();
    let packed = tiles
        .iter()
        .map(|&view| {
            let view = tile(view);
            *index.entry(view).or_insert_with(|| {
                // Below the tile count, which a map's `u16` sides and `u8` levels bound.
                let next = u32::try_from(palette.len()).unwrap_or(u32::MAX);
                palette.push(view);
                next
            })
        })
        .collect();
    (palette, packed)
}

/// A plane's words on the wire.
fn words(plane: &[u64]) -> Vec<WireU64> {
    plane.iter().map(|&w| WireU64(w)).collect()
}

/// A tile's view on the wire.
fn tile(view: TileView) -> Tile {
    match view {
        TileView::Hidden => Tile::Hidden {},
        TileView::Ground {
            terrain,
            road,
            river,
        } => Tile::Ground {
            terrain: terrain.raw(),
            road: road.map(Handle::raw),
            river: river.map(Handle::raw),
        },
        TileView::Seen(StaticTile {
            terrain,
            road,
            river,
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        }) => Tile::Seen {
            terrain: terrain.raw(),
            road: road.map(Handle::raw),
            river: river.map(Handle::raw),
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        },
    }
}

/// A tile's position on the wire.
pub(crate) const fn pos(p: h3_adventure::Pos) -> Pos {
    let h3_adventure::Pos { x, y, level } = p;
    Pos { x, y, level }
}

/// An object by type and anchor on the wire.
pub(crate) const fn object_ref(
    (object_type, anchor): (ObjectTypeId, h3_adventure::Pos),
) -> ObjectRef {
    ObjectRef {
        object_type: object_type.raw(),
        anchor: pos(anchor),
    }
}

/// An amount of a resource on the wire.
pub(crate) const fn amount((resource, amount): (ResourceId, u32)) -> Amount {
    Amount {
        resource: resource.raw(),
        amount,
    }
}

/// An army's stacks on the wire.
pub(crate) fn army(stacks: &[EngineStack]) -> Vec<ArmyStack> {
    stacks
        .iter()
        .map(
            |&EngineStack {
                 slot,
                 creature,
                 count,
             }| ArmyStack {
                slot,
                creature: creature.raw(),
                count,
            },
        )
        .collect()
}

/// A tile mask on the wire: its bits.
const fn mask(m: TileMask) -> WireU64 {
    WireU64(m.bits())
}

/// A hero on the map on the wire.
fn map_hero(h: &HeroView) -> MapHero {
    let HeroView {
        id,
        owner,
        pos: at,
        boat,
        own,
    } = h;
    MapHero {
        id: id.raw(),
        owner: owner.raw(),
        pos: pos(*at),
        boat: *boat,
        own: own.as_ref().map(own_map_hero),
    }
}

/// An own hero's details on the map on the wire.
fn own_map_hero(h: &OwnHeroView) -> OwnMapHero {
    let OwnHeroView {
        army: stacks,
        movement,
        max_movement,
        formation,
        tactics,
        marks,
    } = h;
    OwnMapHero {
        army: army(stacks),
        movement: *movement,
        max_movement: *max_movement,
        formation: match formation {
            EngineFormation::Loose => Formation::Loose,
            EngineFormation::Tight => Formation::Tight,
        },
        tactics: *tactics,
        marks: marks.iter().copied().map(object_ref).collect(),
    }
}

/// An object on the wire.
fn object(o: &ObjectView) -> Object {
    let ObjectView {
        object_type,
        anchor,
        blocked,
        visitable,
        drawn,
        monster,
        owner,
        production,
        guards,
        visited,
        boat_tile,
        town,
    } = o;
    Object {
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        blocked: mask(*blocked),
        visitable: mask(*visitable),
        drawn: mask(*drawn),
        monster: monster.map(
            |MonsterView {
                 creature,
                 count_word,
             }| Monster {
                creature: creature.raw(),
                count_word,
            },
        ),
        owner: owner.map(h3_contracts::PlayerId::raw),
        production: production.map(amount),
        guards: guards.as_ref().map(
            |GuardsView {
                 creature,
                 count_word,
                 army: stacks,
             }| Guards {
                creature: creature.map(Handle::raw),
                count_word: *count_word,
                army: stacks.as_deref().map(army),
            },
        ),
        visited: *visited,
        boat_tile: boat_tile.map(pos),
        town: town.map(h3_contracts::TownId::raw),
    }
}

/// What a player knows of an object on the wire.
fn known_object(k: &KnownObject) -> WireKnownObject {
    let KnownObject {
        object_type,
        anchor,
        known,
    } = k;
    WireKnownObject {
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        known: match known {
            EngineKnown::Spell(spell) => Known::Spell {
                spell: spell.map(Handle::raw),
            },
            EngineKnown::Skill(skill) => Known::Skill {
                skill: skill.map(Handle::raw),
            },
            EngineKnown::Price(price) => Known::Price {
                price: price.iter().copied().map(amount).collect(),
            },
            EngineKnown::Skills(skills) => Known::Skills {
                skills: skills.iter().map(|&s| s.raw()).collect(),
            },
            EngineKnown::Emptied(emptied) => Known::Emptied { emptied: *emptied },
        },
    }
}

/// An empty boat on the wire.
const fn boat(b: &BoatView) -> Boat {
    let BoatView { tile, kind, facing } = *b;
    Boat {
        tile: pos(tile),
        kind,
        facing,
    }
}
