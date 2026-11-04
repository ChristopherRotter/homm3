//! The import of the towns, the players' starts, the heroes moved onto a town's entrance and the
//! hero pool's inputs (E09 T7a1; design note `docs/epics/E09-design.md` §7, survey
//! `docs/rules/surveys/E09-towns.md` §1 to §5). The towns' static layer stays the blockers
//! [`super::import_map`] makes of them (their kind: E09 T8a); the game builds none of it yet
//! (E09 T7b).

use h3_adventure::{ObjectSetup, Pos};
use h3_content::HeroTypeId;
use h3_contracts::{PlayerId, TownId};
use h3_game::{
    BuildingsSetup, FactionChoice, FactionSet, GameRuleset, GarrisonCreature, GarrisonStack,
    MainTown, PlayerSetup, SlotSet, TownEventSetup, TownFaction, TownSetup,
};

use super::{ImportError, blocks, codes, creature, entrances, random_stack, resource};
use crate::model::{Army, Body, Buildings, RawMap, Template, TownBody, TownEvent, Version};
use crate::model::{Object, Text};

/// The factions by the format's town codes, `TOWNTYPE.TXT`'s order (a class 98 town's subclass,
/// the bits of a player's allowed towns): Castle 0 to Fortress 7 (RoE) and Conflux 8 (AB).
const FACTIONS: [&str; 9] = [
    "base:castle",
    "base:rampart",
    "base:tower",
    "base:inferno",
    "base:necropolis",
    "base:dungeon",
    "base:stronghold",
    "base:fortress",
    "ab:conflux",
];

/// An RoE-format map's eight factions' bits (decision 14: all set also allow Conflux).
const ROE_FACTIONS: u16 = 0xFF;

/// The hero type codes an RoE-format map's allowed set gives besides its 128 bits when the
/// ruleset has them: the 16 Conflux heroes (survey §4.2, `map_header_read` 0x4C48EF); never the 12
/// campaign heroes after them.
const CONFLUX_HEROES: core::ops::Range<usize> = 128..144;

/// The format's hero types (`codes::HERO_TYPES`), 0 to 155.
const HERO_TYPE_CODES: usize = codes::HERO_TYPES.len();

/// The format's town alignment byte for none (SoD's "same as owner or random"; `0xFF` before SoD).
const NO_ALIGNMENT: u8 = 0xFF;

/// The colours an alignment byte may name.
const COLOURS: u8 = 8;

/// Each playing colour's start (TWN-SET-001's import; design note §7): its factions the map's
/// allowed towns the ruleset has as factions with a town record (an RoE-format map's player
/// whose eight bits are all set also Conflux when the ruleset has it, E09 decision 14), its
/// choice `Fixed` with exactly one, else `Random`; no pick; its main town the town whose entrance
/// is the header's position (survey finding 3), generating a hero by the map's flag (an RoE map's
/// always); its random-hero flag. `towns` are the towns' footprints in `TownId` order.
///
/// # Errors
///
/// [`ImportError::MainTown`]: the first playing colour whose main town's position is no town's
/// entrance (T-4, `TWN-QUIRK-4`).
// rule: TWN-SET-001
pub(super) fn starts(
    map: &RawMap,
    rules: &GameRuleset,
    players: &mut [PlayerSetup],
    towns: &[ObjectSetup],
) -> Result<(), ImportError> {
    for player in players {
        let Some(details) = map
            .players
            .get(usize::from(player.colour))
            .and_then(|p| p.details.as_ref())
        else {
            continue;
        };
        let mut allowed = details.allowed_towns;
        if map.header.version == Version::RoE && allowed & ROE_FACTIONS == ROE_FACTIONS {
            allowed |= 1 << (FACTIONS.len() - 1);
        }
        let factions = FACTIONS
            .iter()
            .zip(0u8..)
            .filter(|&(_, bit)| allowed >> bit & 1 == 1)
            .filter_map(|(&key, _)| rules.core().faction_by_key(key))
            .filter(|&faction| rules.towns().town(faction).is_some())
            .fold(FactionSet::EMPTY, FactionSet::with);
        let mut only = factions.iter();
        let choice = match (only.next(), only.next()) {
            (Some(faction), None) => FactionChoice::Fixed(faction),
            _ => FactionChoice::Random,
        };
        let main_town = details
            .main_town
            .map(|main| {
                let pos = Pos::new(u16::from(main.pos.x), u16::from(main.pos.y), main.pos.z);
                towns
                    .iter()
                    .zip(0..=u16::MAX)
                    .find(|(town, _)| entrances(town).any(|entrance| entrance == pos))
                    .map(|(_, id)| MainTown {
                        town: TownId::new(id),
                        generates_hero: main.generate_hero,
                    })
                    .ok_or(ImportError::MainTown(player.colour))
            })
            .transpose()?;
        player.start.factions = factions;
        player.start.choice = Some(choice);
        player.start.main_town = main_town;
        player.start.random_hero = details.random_hero;
    }
    Ok(())
}

/// The towns (classes 77 and 98) in the map's order, each as the map gives it (design note §5 and
/// §7): a town's faction by its subclass ([`FACTIONS`]), a random town's by its SoD alignment;
/// the buildings' slots as stored; the custom garrison, a random stack's code read as its level
/// and upgrade, a stack of count 0 an empty slot; the name (Windows-1252); the events. An object
/// of these classes without a town's body (never the reader's) is a neutral town's defaults.
///
/// # Errors
///
/// The first [`ImportError`] in the map's order: a class 98 town's subclass past the factions, a
/// faction the ruleset lacks, a random town's alignment from 8 to 254 (T-1, `TWN-QUIRK-1`), a
/// garrison's or an event's code.
pub(super) fn towns(
    map: &RawMap,
    rules: &GameRuleset,
    colours: &[Option<PlayerId>; 8],
    objects: &[(usize, &Object, &Template)],
) -> Result<Vec<TownSetup>, ImportError> {
    let neutral = TownBody {
        owner: NO_ALIGNMENT,
        alignment: NO_ALIGNMENT,
        ..TownBody::default()
    };
    objects
        .iter()
        .map(|&(index, object, template)| {
            let body = match &object.body {
                Body::Town(body) => body,
                _ => &neutral,
            };
            let faction = if template.class == super::TOWN {
                let key = usize::try_from(template.subclass)
                    .ok()
                    .and_then(|subclass| FACTIONS.get(subclass))
                    .ok_or(ImportError::Subclass(index))?;
                TownFaction::Fixed(
                    rules
                        .core()
                        .faction_by_key(key)
                        .ok_or(ImportError::MissingContent(key))?,
                )
            } else {
                TownFaction::Random {
                    alignment: alignment(index, body.alignment)?,
                }
            };
            Ok(TownSetup {
                faction,
                buildings: match body.buildings {
                    Buildings::Standard { fort } => BuildingsSetup::Standard { fort },
                    Buildings::Custom { built, forbidden } => BuildingsSetup::Custom {
                        built: slots(built),
                        forbidden: slots(forbidden),
                    },
                },
                garrison: body
                    .garrison
                    .as_ref()
                    .map(|army| garrison(map.header.version, rules, army))
                    .transpose()?,
                name: body.name.as_ref().map(windows_1252),
                events: body
                    .events
                    .iter()
                    .map(|event| town_event(rules, colours, event))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect()
}

/// A random town's alignment byte (survey §2.3): `0xFF` none, 0 to 7 a colour; 8 to 254 read
/// another table entry in the original (T-1) and are refused.
fn alignment(index: usize, byte: u8) -> Result<Option<u8>, ImportError> {
    match byte {
        NO_ALIGNMENT => Ok(None),
        colour if colour < COLOURS => Ok(Some(colour)),
        _ => Err(ImportError::Alignment(index)),
    }
}

/// A set of building slots as stored: bit `j` of byte `i` is slot `8i + j`.
fn slots(bytes: [u8; 6]) -> SlotSet {
    let mut bits = [0; 8];
    bits[..6].copy_from_slice(&bytes);
    SlotSet::from_bits(u64::from_le_bytes(bits))
}

/// A town's custom garrison in slot order (design note §5.2): each stack with a count, a random
/// stack's code (one of the 14 codes below the format's "none", from −2 down) the
/// level `(−2 − code) / 2 + 1`, upgraded when the code is odd; a stack of count 0 is an empty slot
/// (the original ignores its code).
fn garrison(
    version: Version,
    rules: &GameRuleset,
    army: &Army,
) -> Result<Vec<GarrisonStack>, ImportError> {
    let none = if version == Version::RoE {
        u16::from(u8::MAX)
    } else {
        u16::MAX
    };
    army.iter()
        .zip(0u8..)
        .filter_map(|(stack, slot)| stack.filter(|s| s.count != 0).map(|s| (s, slot)))
        .map(|(stack, slot)| {
            let creature = if random_stack(version, stack.creature) {
                // From 0 for code −2 (`none` − 1) to 13 for code −15.
                let step = none - 1 - stack.creature;
                GarrisonCreature::Random {
                    level: u8::try_from(step / 2 + 1).unwrap_or(u8::MAX),
                    upgraded: step % 2 == 1,
                }
            } else {
                GarrisonCreature::Creature(creature(rules, stack.creature)?)
            };
            Ok(GarrisonStack {
                slot,
                creature,
                count: u32::from(stack.count),
            })
        })
        .collect()
}

/// A town's event as the map gives it (the map's timed events' reading, GAM-CAL-004): the players
/// of its colours that play, its flags, the file's first day + 1, its interval, its resources but
/// the zeros, its buildings' slots and its creatures by level.
fn town_event(
    rules: &GameRuleset,
    colours: &[Option<PlayerId>; 8],
    event: &TownEvent,
) -> Result<TownEventSetup, ImportError> {
    let timed = &event.event;
    Ok(TownEventSetup {
        players: colours
            .iter()
            .zip(0u8..)
            .filter(|&(_, colour)| timed.players >> colour & 1 == 1)
            .filter_map(|(id, _)| *id)
            .collect(),
        human: timed.human,
        computer: timed.computer,
        first_day: timed.first_day.saturating_add(1),
        interval: timed.interval,
        resources: codes::RESOURCES
            .iter()
            .zip(timed.resources)
            .filter(|&(_, amount)| amount != 0)
            .map(|(&key, amount)| Ok((resource(rules, key)?, amount)))
            .collect::<Result<_, ImportError>>()?,
        buildings: slots(event.buildings),
        creatures: event.creatures,
    })
}

/// Windows-1252 code points for bytes 0x80–0x9F (undefined bytes map to the C1 control of the
/// same value): `h3-formats`' `decode_windows_1252`'s table, which this crate does not depend on.
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// A stored text decoded as Windows-1252, the code page of the game's own maps (every byte one
/// character).
fn windows_1252(text: &Text) -> String {
    text.as_bytes()
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => CP1252_HIGH[usize::from(b - 0x80)],
            _ => char::from(b),
        })
        .collect()
}

/// The tile a placed hero (not a prisoner) starts on (TWN-SET-010; survey §4.6,
/// `placed_heroes_setup` 0x4CAE10): the tile left of its own when that is a town's entrance, its
/// own otherwise.
// rule: TWN-SET-010
pub(super) fn hero_tile(pos: Pos, towns: &[ObjectSetup]) -> Pos {
    pos.x
        .checked_sub(1)
        .map(|x| Pos::new(x, pos.y, pos.level))
        .filter(|left| towns.iter().any(|town| entrances(town).any(|e| e == *left)))
        .unwrap_or(pos)
}

/// Whether a hero on `pos` stands in `town`: on a tile it blocks that is not an entrance (the
/// import refuses it, `Unsupported::HeroInTown`).
pub(super) fn in_town(town: &ObjectSetup, pos: Pos) -> bool {
    blocks(town, pos) && !entrances(town).any(|entrance| entrance == pos)
}

/// The hero pool's inputs as a game's setup holds them: the allowed heroes, the colour masks, the
/// prisoners.
type Pool = (Vec<HeroTypeId>, Vec<(HeroTypeId, u8)>, Vec<HeroTypeId>);

/// The hero pool's inputs (design note §7; survey §4.2): the hero types the map allows (an
/// RoE-format map's 128 bits and the 16 Conflux heroes besides; from AB on the 156 bits), SoD's
/// disposed heroes' types with their colour masks and the prisoners' types (class 62), each in
/// the map's order. A type the ruleset lacks (a hero of an expansion it does not enable) is left
/// out: no draw can take it.
///
/// # Errors
///
/// [`ImportError::UnknownCode`]: a disposed hero's or a prisoner's type past the format's, the
/// disposed heroes first.
pub(super) fn pool(
    map: &RawMap,
    rules: &GameRuleset,
    prisoners: &[&Body],
) -> Result<Pool, ImportError> {
    let roe = map.header.version == Version::RoE;
    let allowed = (0..HERO_TYPE_CODES)
        .filter(|&code| map.allowed_heroes.get(code) || roe && CONFLUX_HEROES.contains(&code))
        .filter_map(|code| rules.core().hero_type_by_key(codes::HERO_TYPES[code]))
        .collect();
    let masks = map
        .disposed_heroes
        .iter()
        .map(|hero| Ok(pool_type(rules, hero.hero_type)?.map(|t| (t, hero.players))))
        .filter_map(Result::transpose)
        .collect::<Result<_, ImportError>>()?;
    let prisoners = prisoners
        .iter()
        .filter_map(|body| match body {
            Body::Hero(hero) => Some(hero.hero_type),
            _ => None,
        })
        .map(|code| pool_type(rules, code))
        .filter_map(Result::transpose)
        .collect::<Result<_, ImportError>>()?;
    Ok((allowed, masks, prisoners))
}

/// The hero type of a format's code, `None` when the ruleset lacks it.
fn pool_type(rules: &GameRuleset, code: u8) -> Result<Option<HeroTypeId>, ImportError> {
    let key = codes::HERO_TYPES
        .get(usize::from(code))
        .ok_or(ImportError::UnknownCode {
            what: "hero type",
            code: u16::from(code),
        })?;
    Ok(rules.core().hero_type_by_key(key))
}
