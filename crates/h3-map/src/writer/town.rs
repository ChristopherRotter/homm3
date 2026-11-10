//! The town lane's objects (E09 T9a, design note `docs/epics/E09-design.md` §18): a town of any
//! kind ([`AnyTown`]: of a fixed faction (98) or random (77) with its SoD alignment, a player's
//! main town, a name, a garrison whose stacks may be random, [`random_stack`], and since E09 T19a
//! its events, [`TownEvent`], design note §32) and a random hero ([`RandomHero`], class 70). Their bodies are what the import reads (`import/town.rs`, design
//! note §7); a main town also goes into its owner's header (the position of its entrance, survey
//! `docs/rules/surveys/E09-towns.md` finding 3). [`Town`](super::Town), E04c's town of a fixed
//! faction, is written as an [`AnyTown`] without these.

use super::{
    ARMY_SLOTS, Codes, FACTION_COUNT, MAP_BUILDINGS, Map, MapObject, ObjectTemplate, Out, PLAYERS,
    TOWN_OBJECT, TimedEvent, Town, TownBuildings, WriteError, economy, len32, out_of_range,
    write_army,
};

/// Object class of a random town (subclass 0): its faction is drawn when the game is built, or
/// its alignment's colour's (TWN-SET-002).
pub const RANDOM_TOWN_OBJECT: u32 = 77;
/// Object class of a random hero (subclass 0): its type is drawn when the game is built
/// (TWN-SET-007).
pub const RANDOM_HERO_OBJECT: u32 = 70;
/// The levels a random stack may have.
const RANDOM_LEVELS: u8 = 7;
/// The format's code of an empty slot (SoD).
const NONE: u16 = u16::MAX;

/// The code of a random stack of `level` (1 to 7), upgraded or not, in a town's garrison: one of
/// the 14 codes below the format's "none", level 1's `0xFFFE` (upgraded `0xFFFD`) down to level
/// 7's upgraded `0xFFF1`; the game draws its creature when it is built (design note §5.2).
/// `None` for a level outside 1 to 7.
#[must_use]
pub const fn random_stack(level: u8, upgraded: bool) -> Option<u16> {
    if level == 0 || level > RANDOM_LEVELS {
        return None;
    }
    Some(NONE - 1 - 2 * (level as u16 - 1) - upgraded as u16)
}

/// Whether `code` is a random stack's ([`random_stack`]).
const fn is_random_stack(code: u16) -> bool {
    code < NONE && code >= NONE - 2 * RANDOM_LEVELS as u16
}

/// A player's main town: the header names its entrance, the player's start reads it
/// (TWN-SET-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainTown {
    /// The game generates the player's starting hero at the town (the editor's flag).
    pub generates_hero: bool,
}

/// A town's event (`docs/formats/h3m.md`, classes 77 and 98; E09 T19a): a timed event of the
/// map's kind (its players, its flags for a human and a computer, its first day and interval, its
/// resources) and what it adds to the town when it fires (TWN-EVT-001 to -004): buildings by the
/// map's slots, creatures by dwelling level.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TownEvent {
    /// The timed event: its name and message, players, flags, days and resources.
    pub event: TimedEvent,
    /// The buildings it builds, a bit per map slot as a custom set's (`8i + j`, below
    /// [`MAP_BUILDINGS`]; `docs/formats/h3m.md`, "Town buildings").
    pub buildings: [u8; 6],
    /// The creatures it adds by dwelling level, levels 1 to 7.
    pub creatures: [u16; 7],
}

/// A town of any kind placed on the map (E09 T9a): of a fixed faction or random, perhaps its
/// owner's main town, with a name, a garrison whose stacks may be random and its events (E09
/// T19a). No required or banned spells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyTown {
    /// The object's position: the bottom-right tile of its picture; its entrance is the
    /// template's visitable tile (two columns left of it on its row, the towns' templates in
    /// `Objects.txt`).
    pub tile: (u8, u8),
    /// Template: class [`TOWN_OBJECT`] with the faction as subclass, or
    /// [`RANDOM_TOWN_OBJECT`] with subclass 0 (`Objects.txt`'s lines).
    pub template: ObjectTemplate,
    /// Owning player (0 = red … 7 = pink), `None` for a neutral town.
    pub owner: Option<u8>,
    /// A random town's alignment (SoD's byte): the colour whose faction it takes; `None` the
    /// byte `0xFF` (the owner's, else drawn). Only a random town has one.
    pub alignment: Option<u8>,
    /// Whether the town is its owner's main town; at most one per player.
    pub main: Option<MainTown>,
    /// Custom name.
    pub name: Option<String>,
    /// Garrison (creature code or a [`random_stack`]'s, count) per slot; `None` = the editor's
    /// default.
    pub garrison: Option<[Option<(u16, u16)>; ARMY_SLOTS]>,
    /// The garrison's tight formation.
    pub tight_formation: bool,
    /// Buildings.
    pub buildings: TownBuildings,
    /// Its events, in the town's order (TWN-EVT-001: each named by its place there).
    pub events: Vec<TownEvent>,
}

impl From<&Town> for AnyTown {
    fn from(town: &Town) -> Self {
        Self {
            tile: town.tile,
            template: town.template.clone(),
            owner: town.owner,
            alignment: None,
            main: None,
            name: None,
            garrison: town.garrison,
            tight_formation: town.tight_formation,
            buildings: town.buildings,
            events: Vec::new(),
        }
    }
}

/// A random hero placed on the map (E09 T9a): its type is drawn when the game is built
/// (TWN-SET-007); the rest of its body a placed hero's, the hero type's own but the experience
/// and the army.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomHero {
    /// Tile the hero stands on (its object's position one column right, as a hero's).
    pub tile: (u8, u8),
    /// Owning player (0 = red … 7 = pink).
    pub owner: u8,
    /// Experience; `None` the hero type's own.
    pub experience: Option<u32>,
    /// Army (creature id, count) per slot; `None` the hero type's own.
    pub army: Option<[Option<(u16, u16)>; ARMY_SLOTS]>,
    /// Tight formation.
    pub tight_formation: bool,
}

impl ObjectTemplate {
    /// The map editor's template of a random hero (as in the game's own SoD maps: 36 of them,
    /// E09 T9a): picture `ahrandom.def`, a hero's masks.
    #[must_use]
    pub fn random_hero() -> Self {
        Self {
            def: "ahrandom.def".to_owned(),
            class: RANDOM_HERO_OBJECT,
            subclass: 0,
            ..Self::hero(0)
        }
    }
}

/// Whether `owner`, a colour, plays on `map`.
fn plays(map: &Map, colour: u8) -> bool {
    map.players
        .get(usize::from(colour))
        .and_then(Option::as_ref)
        .is_some()
}

/// The first tile `template` is visited from, as (columns left, rows above) of its position (the
/// masks' order, [`mask_from_text`](super::mask_from_text)'s).
fn entrance(template: &ObjectTemplate) -> Option<(u8, u8)> {
    (0u8..48)
        .find(|&k| {
            let bit = 47 - k;
            template.visitable[usize::from(bit / 8)] >> (bit % 8) & 1 == 1
        })
        .map(|k| (k % 8, k / 8))
}

/// Checks a town of any kind: its class and subclass, a town in a SoD map, a playable owner, an
/// alignment only for a random town and of a colour that plays, a main town owned with its
/// entrance on the map, the garrison's codes (a creature the map may name or a random stack's)
/// and counts, the buildings' bits and each event's.
pub(super) fn validate(map: &Map, town: &AnyTown, codes: &Codes) -> Result<(), WriteError> {
    let ObjectTemplate {
        class, subclass, ..
    } = town.template;
    match class {
        TOWN_OBJECT if subclass < FACTION_COUNT => {}
        TOWN_OBJECT => return Err(out_of_range("town faction", subclass)),
        RANDOM_TOWN_OBJECT if subclass == 0 => {}
        RANDOM_TOWN_OBJECT => return Err(out_of_range("random town subclass", subclass)),
        _ => return Err(out_of_range("town object class", class)),
    }
    codes.refuse("town in an RoE map", subclass)?;
    if let Some(owner) = town.owner
        && !plays(map, owner)
    {
        return Err(out_of_range("town owner (no playable player)", owner));
    }
    if let Some(colour) = town.alignment {
        if class != RANDOM_TOWN_OBJECT {
            return Err(out_of_range(
                "alignment of a town of a fixed faction",
                colour,
            ));
        }
        if !plays(map, colour) {
            return Err(out_of_range("town alignment (no playable player)", colour));
        }
    }
    if town.main.is_some() {
        let Some(owner) = town.owner else {
            return Err(out_of_range("main town without an owner", town.tile.0));
        };
        let on_map = entrance(&town.template)
            .is_some_and(|(left, above)| town.tile.0 >= left && town.tile.1 >= above);
        if !on_map {
            return Err(out_of_range("main town's entrance (off the map)", owner));
        }
    }
    if let Some(garrison) = &town.garrison {
        for &(code, count) in garrison.iter().flatten() {
            if !is_random_stack(code) {
                codes.stack(code, count)?;
            } else if count == 0 {
                return Err(out_of_range("stack size", count));
            }
        }
    }
    if let TownBuildings::Custom { built, forbidden } = &town.buildings
        && (past_slots(built) || past_slots(forbidden))
    {
        return Err(out_of_range("town building (bit)", MAP_BUILDINGS));
    }
    if town.events.iter().any(|event| past_slots(&event.buildings)) {
        return Err(out_of_range("town event building (bit)", MAP_BUILDINGS));
    }
    Ok(())
}

/// Whether a set of building bits has one past the map's [`MAP_BUILDINGS`] slots (the original's
/// event loop reads bits 41 to 43 from the next faction's row, TWN-QUIRK-7: the writer writes
/// none).
const fn past_slots(bits: &[u8; 6]) -> bool {
    bits[5] >> (MAP_BUILDINGS % 8) != 0
}

/// Checks a random hero: a playable owner, its object's position on the map, its army.
pub(super) fn validate_hero(map: &Map, hero: &RandomHero, codes: &Codes) -> Result<(), WriteError> {
    codes.refuse("random hero in an RoE map", hero.owner)?;
    if !plays(map, hero.owner) {
        return Err(out_of_range("hero owner (no playable player)", hero.owner));
    }
    // The hero's object position is one column right of its tile.
    if hero.tile.0.checked_add(1).is_none_or(|x| x >= map.size) {
        return Err(out_of_range("hero tile (last column)", hero.tile.0));
    }
    if let Some(army) = &hero.army {
        codes.army(army)?;
    }
    Ok(())
}

/// Checks the map's main towns: at most one per player.
pub(super) fn validate_main_towns(map: &Map) -> Result<(), WriteError> {
    for colour in (0u8..).take(PLAYERS) {
        if main_towns(map, colour).nth(1).is_some() {
            return Err(out_of_range(
                "main towns of a player (more than one)",
                colour,
            ));
        }
    }
    Ok(())
}

/// The main towns of `colour` with their levels, in the map's order.
fn main_towns(map: &Map, colour: u8) -> impl Iterator<Item = (u8, &AnyTown)> {
    map.placed()
        .filter_map(move |(level, object)| match object {
            MapObject::AnyTown(town) if town.main.is_some() && town.owner == Some(colour) => {
                Some((level, town))
            }
            _ => None,
        })
}

/// The header's main town of `colour` (`docs/formats/h3m.md`, "Players"): none, or the flag, SoD's
/// generate-hero flag and town-type byte (the faction's subclass, `0xFF` for a random town), the
/// position of its entrance.
pub(super) fn write_main_town(out: &mut Out, map: &Map, colour: u8) {
    let Some((level, town)) = main_towns(map, colour).next() else {
        out.bool(false);
        return;
    };
    out.bool(true);
    if out.1 == super::Format::Sod {
        out.bool(town.main.is_some_and(|main| main.generates_hero));
        let town_type = match town.template.class {
            TOWN_OBJECT => u8::try_from(town.template.subclass).unwrap_or(u8::MAX),
            _ => u8::MAX,
        };
        out.u8(town_type);
    }
    // Checked: the entrance is on the map.
    let (left, above) = entrance(&town.template).unwrap_or((0, 0));
    out.bytes(&[
        town.tile.0.saturating_sub(left),
        town.tile.1.saturating_sub(above),
        level,
    ]);
}

/// A town's body (`docs/formats/h3m.md`, classes 77 and 98): the name, the garrison, the
/// buildings, no required or banned spells, the events, the alignment.
pub(super) fn write_body(out: &mut Out, town: &AnyTown, identifier: u32) -> Result<(), WriteError> {
    out.u32(identifier);
    out.u8(town.owner.unwrap_or(0xff));
    out.bool(town.name.is_some());
    if let Some(name) = &town.name {
        out.string(name)?;
    }
    write_rest(out, town)
}

/// A town of a fixed faction's body ([`Town`]): an [`AnyTown`]'s without a name or events, so
/// nothing fails.
pub(super) fn write_unnamed(out: &mut Out, town: &Town, identifier: u32) {
    out.u32(identifier);
    out.u8(town.owner.unwrap_or(0xff));
    out.bool(false); // no custom name
    write_settings(out, &AnyTown::from(town));
    out.u32(0); // no events
    write_alignment(out, None);
}

/// A town's body after its name: its settings, its events (E09 T19a: each a timed event, the
/// buildings' bits, the creatures by level, four unused bytes), the alignment.
fn write_rest(out: &mut Out, town: &AnyTown) -> Result<(), WriteError> {
    write_settings(out, town);
    out.u32(len32(town.events.len()));
    for event in &town.events {
        economy::write_event(out, &event.event)?;
        out.bytes(&event.buildings);
        for count in event.creatures {
            out.u16(count);
        }
        out.bytes(&[0; 4]);
    }
    write_alignment(out, town.alignment);
    Ok(())
}

/// A town's garrison, formation, buildings and spells (none required or banned).
fn write_settings(out: &mut Out, town: &AnyTown) {
    out.bool(town.garrison.is_some());
    if let Some(garrison) = &town.garrison {
        write_army(out, garrison);
    }
    out.bool(town.tight_formation);
    match &town.buildings {
        TownBuildings::Standard { fort } => {
            out.bool(false);
            out.bool(*fort);
        }
        TownBuildings::Custom { built, forbidden } => {
            out.bool(true);
            out.bytes(built);
            out.bytes(forbidden);
        }
    }
    out.bytes(&[0; 9]); // no required spells
    out.bytes(&[0; 9]); // no banned spells
}

/// A town's alignment (SoD's byte, `0xFF` none) and its three unused bytes.
fn write_alignment(out: &mut Out, alignment: Option<u8>) {
    out.u8(alignment.unwrap_or(0xff));
    out.bytes(&[0; 3]);
}

/// A random hero's body: a placed hero's with the type `0xFF`.
pub(super) fn write_hero(
    out: &mut Out,
    hero: &RandomHero,
    identifier: u32,
) -> Result<(), WriteError> {
    let placed = super::Hero {
        tile: hero.tile,
        owner: hero.owner,
        hero_type: u8::MAX,
        class: 0,
        name: None,
        experience: hero.experience,
        skills: None,
        army: hero.army,
        tight_formation: hero.tight_formation,
        artifacts: None,
        spells: None,
        primary: None,
    };
    super::write_hero(out, &placed, identifier)
}

#[cfg(test)]
mod tests {
    use super::super::tests::smoke_like_map;
    use super::super::{Format, write, write_in};
    use super::*;
    use crate::model::{
        Body, MainTown as ReadMainTown, Pos, Stack, Text, TimedEvent as ReadTimedEvent,
        TownEvent as ReadTownEvent,
    };
    use crate::{Limits, parse};

    /// `Objects.txt`'s lines of a Castle (98 0) and a random town (77 0).
    const CASTLE: &str = "AVCcasx0.def 000001110000011110001111111111111111111111111111 \
                          001000000000000000000000000000000000000000000000 011111111 \
                          011111111 98 0 1 0";
    const RANDOM: &str = "AVCranx0.def 000001110000011110001111111111111111111111111111 \
                          001000000000000000000000000000000000000000000000 011111111 \
                          011111111 77 0 1 0";

    /// Red's random main town aligned to blue, named, with a random stack.
    fn random_town() -> AnyTown {
        AnyTown {
            tile: (14, 8),
            template: ObjectTemplate::from_objects_txt(RANDOM).unwrap(),
            owner: Some(0),
            alignment: Some(1),
            main: Some(MainTown {
                generates_hero: true,
            }),
            name: Some("Haven".into()),
            garrison: Some([
                random_stack(3, true).map(|code| (code, 12)),
                Some((1, 25)),
                None,
                None,
                None,
                None,
                random_stack(7, false).map(|code| (code, 1)),
            ]),
            tight_formation: true,
            buildings: TownBuildings::Standard { fort: false },
            events: Vec::new(),
        }
    }

    fn random_hero() -> RandomHero {
        RandomHero {
            tile: (22, 14),
            owner: 1,
            experience: Some(500),
            army: Some([Some((84, 3)), None, None, None, None, None, None]),
            tight_formation: false,
        }
    }

    #[test]
    fn random_stacks_take_the_fourteen_codes_below_none() {
        assert_eq!(random_stack(1, false), Some(0xFFFE));
        assert_eq!(random_stack(1, true), Some(0xFFFD));
        assert_eq!(random_stack(4, false), Some(0xFFF8));
        assert_eq!(random_stack(7, true), Some(0xFFF1));
        assert_eq!(random_stack(0, false), None);
        assert_eq!(random_stack(8, true), None);
        assert!(!is_random_stack(NONE));
        assert!(is_random_stack(0xFFF1));
        assert!(!is_random_stack(0xFFF0));
    }

    #[test]
    fn towns_of_any_kind_and_random_heroes_read_back_as_written() {
        let mut map = smoke_like_map();
        map.objects.push(MapObject::AnyTown(random_town()));
        // Blue's main town, a Castle of a fixed faction that generates no hero.
        map.objects.push(MapObject::AnyTown(AnyTown {
            tile: (30, 30),
            template: ObjectTemplate::from_objects_txt(CASTLE).unwrap(),
            owner: Some(1),
            alignment: None,
            main: Some(MainTown {
                generates_hero: false,
            }),
            name: None,
            garrison: None,
            tight_formation: false,
            buildings: TownBuildings::Standard { fort: true },
            events: Vec::new(),
        }));
        map.objects.push(MapObject::RandomHero(random_hero()));
        let raw = parse(&write(&map).unwrap(), &Limits::default()).unwrap();
        let main = |colour: usize| raw.players[colour].details.as_ref().unwrap().main_town;
        assert_eq!(
            main(0),
            Some(ReadMainTown {
                generate_hero: true,
                town_type: 0xFF,
                pos: Pos { x: 12, y: 8, z: 0 },
            })
        );
        assert_eq!(
            main(1),
            Some(ReadMainTown {
                generate_hero: false,
                town_type: 0,
                pos: Pos { x: 28, y: 30, z: 0 },
            })
        );
        let class = |index: usize| raw.templates[raw.objects[index].template as usize].class;
        assert_eq!((class(2), class(3), class(4)), (77, 98, 70));
        let Body::Town(town) = &raw.objects[2].body else {
            panic!("a town's body");
        };
        assert_eq!(
            (town.identifier, town.owner, town.alignment, town.formation),
            (Some(3), 0, 1, 1)
        );
        assert_eq!(town.name, Some(Text(b"Haven".to_vec())));
        let garrison = town.garrison.as_ref().unwrap();
        assert_eq!(
            garrison[0],
            Some(Stack {
                creature: 0xFFF9,
                count: 12,
            })
        );
        assert_eq!(
            garrison[6],
            Some(Stack {
                creature: 0xFFF2,
                count: 1,
            })
        );
        let Body::Town(castle) = &raw.objects[3].body else {
            panic!("a town's body");
        };
        assert_eq!(
            (castle.owner, castle.alignment, &castle.name),
            (1, 0xFF, &None)
        );
        let Body::Hero(hero) = &raw.objects[4].body else {
            panic!("a hero's body");
        };
        assert_eq!(
            (hero.owner, hero.hero_type, hero.experience),
            (1, 0xFF, Some(500))
        );
        assert_eq!(raw.objects[4].pos, Pos { x: 23, y: 14, z: 0 });
        // A town of a fixed faction is written as a town of any kind without its fields.
        let mut fixed = smoke_like_map();
        fixed.objects.push(MapObject::Town(Town {
            tile: (30, 30),
            template: ObjectTemplate::from_objects_txt(CASTLE).unwrap(),
            owner: Some(1),
            garrison: None,
            tight_formation: false,
            buildings: TownBuildings::Standard { fort: true },
        }));
        let mut any = smoke_like_map();
        any.objects.push(MapObject::AnyTown(AnyTown {
            main: None,
            ..match &map.objects[3] {
                MapObject::AnyTown(town) => town.clone(),
                _ => unreachable!(),
            }
        }));
        assert_eq!(write(&fixed), write(&any));
    }

    /// A town event for red and blue's humans from the file's day 2 every week: five wood and
    /// three ore given, 500 gold taken; the Town Hall, the Fort and the last slot's building;
    /// creatures of levels 1, 2 and 7.
    fn harvest() -> TownEvent {
        TownEvent {
            event: TimedEvent {
                name: "Harvest".into(),
                message: "The fields yield.".into(),
                resources: [5, 0, 3, 0, 0, 0, -500],
                players: 0b11,
                human: true,
                computer: false,
                first_day: 2,
                interval: 7,
            },
            buildings: [0b1001, 0, 0, 0, 0, 0x01],
            creatures: [10, 5, 0, 0, 0, 0, 1],
        }
    }

    #[test]
    fn a_town_s_events_read_back_as_written() {
        // E09 T19a: each event in the town's order, after the spells and before the alignment.
        let mut map = smoke_like_map();
        let mut town = random_town();
        town.events = vec![harvest(), TownEvent::default()];
        map.objects.push(MapObject::AnyTown(town));
        let raw = parse(&write(&map).unwrap(), &Limits::default()).unwrap();
        let Body::Town(read) = &raw.objects[2].body else {
            panic!("a town's body");
        };
        assert_eq!(
            read.events,
            vec![
                ReadTownEvent {
                    event: ReadTimedEvent {
                        name: Text(b"Harvest".to_vec()),
                        message: Text(b"The fields yield.".to_vec()),
                        resources: [5, 0, 3, 0, 0, 0, -500],
                        players: 0b11,
                        human: true,
                        computer: false,
                        first_day: 2,
                        interval: 7,
                    },
                    buildings: [0b1001, 0, 0, 0, 0, 0x01],
                    creatures: [10, 5, 0, 0, 0, 0, 1],
                },
                ReadTownEvent {
                    event: ReadTimedEvent {
                        name: Text(Vec::new()),
                        message: Text(Vec::new()),
                        resources: [0; 7],
                        players: 0,
                        human: false,
                        computer: false,
                        first_day: 0,
                        interval: 0,
                    },
                    buildings: [0; 6],
                    creatures: [0; 7],
                },
            ]
        );
        assert_eq!(read.alignment, 1);
    }

    #[test]
    fn the_writer_refuses_a_town_or_a_random_hero_the_game_cannot_load() {
        let with = |f: &dyn Fn(&mut Map), format: Format| {
            let mut map = smoke_like_map();
            if let MapObject::Hero(hero) = &mut map.objects[0] {
                hero.primary = None;
            }
            map.objects.push(MapObject::AnyTown(random_town()));
            map.objects.push(MapObject::RandomHero(random_hero()));
            f(&mut map);
            write_in(&map, format).map(|_| ())
        };
        let fails = |f: &dyn Fn(&mut Map), format: Format, what: &str| match with(f, format) {
            Err(WriteError::OutOfRange { what: w, .. }) => assert_eq!(w, what),
            other => panic!("{what}: {other:?}"),
        };
        let town = |f: fn(&mut AnyTown)| {
            move |map: &mut Map| {
                if let Some(MapObject::AnyTown(town)) = map.objects.get_mut(2) {
                    f(town);
                }
            }
        };
        let hero = |f: fn(&mut RandomHero)| {
            move |map: &mut Map| {
                if let Some(MapObject::RandomHero(hero)) = map.objects.get_mut(3) {
                    f(hero);
                }
            }
        };
        let sod = Format::Sod;
        assert_eq!(with(&|_| {}, sod), Ok(()));
        fails(&town(|t| t.template.class = 13), sod, "town object class");
        fails(
            &town(|t| t.template.subclass = 1),
            sod,
            "random town subclass",
        );
        fails(
            &town(|t| {
                t.template.class = TOWN_OBJECT;
                t.template.subclass = 9;
            }),
            sod,
            "town faction",
        );
        fails(
            &town(|t| t.template.class = TOWN_OBJECT),
            sod,
            "alignment of a town of a fixed faction",
        );
        fails(
            &town(|t| t.owner = Some(4)),
            sod,
            "town owner (no playable player)",
        );
        fails(
            &town(|t| t.alignment = Some(5)),
            sod,
            "town alignment (no playable player)",
        );
        fails(&town(|t| t.owner = None), sod, "main town without an owner");
        fails(
            &town(|t| t.tile = (1, 8)),
            sod,
            "main town's entrance (off the map)",
        );
        fails(
            &town(|t| t.template.visitable = [0; 6]),
            sod,
            "main town's entrance (off the map)",
        );
        fails(
            &town(|t| t.garrison = Some([random_stack(2, false).map(|c| (c, 0)); ARMY_SLOTS])),
            sod,
            "stack size",
        );
        fails(
            &town(|t| t.garrison = Some([Some((0xFFF0, 1)); ARMY_SLOTS])),
            sod,
            "creature",
        );
        fails(
            &town(|t| {
                t.buildings = TownBuildings::Custom {
                    built: [0, 0, 0, 0, 0, 0x02],
                    forbidden: [0; 6],
                };
            }),
            sod,
            "town building (bit)",
        );
        fails(
            &town(|t| {
                t.buildings = TownBuildings::Custom {
                    built: [0; 6],
                    forbidden: [0, 0, 0, 0, 0, 0x80],
                };
            }),
            sod,
            "town building (bit)",
        );
        fails(
            &town(|t| {
                t.events = vec![TownEvent {
                    buildings: [0, 0, 0, 0, 0, 0x02],
                    ..harvest()
                }];
            }),
            sod,
            "town event building (bit)",
        );
        fails(
            &|map: &mut Map| map.objects.push(MapObject::AnyTown(random_town())),
            sod,
            "main towns of a player (more than one)",
        );
        fails(&|_| {}, Format::RoE, "town in an RoE map");
        let roe_hero = |map: &mut Map| {
            map.objects.remove(2);
        };
        fails(&roe_hero, Format::RoE, "random hero in an RoE map");
        fails(
            &hero(|h| h.owner = 3),
            sod,
            "hero owner (no playable player)",
        );
        fails(&hero(|h| h.tile = (35, 4)), sod, "hero tile (last column)");
        fails(
            &hero(|h| h.army = Some([Some((300, 1)); ARMY_SLOTS])),
            sod,
            "creature",
        );
        // A name the format cannot hold fails as it is written, a town's or its event's.
        fails(
            &town(|t| t.name = Some("\u{2603}".into())),
            sod,
            "character in a map string",
        );
        fails(
            &town(|t| {
                let mut event = harvest();
                event.event.message = "\u{2603}".into();
                t.events = vec![TownEvent::default(), event];
            }),
            sod,
            "character in a map string",
        );
    }
}
