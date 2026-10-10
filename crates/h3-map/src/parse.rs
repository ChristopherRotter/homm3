//! Parsing the inflated bytes of a map into the raw model. Field order and sizes follow VCMI
//! 1.7.5's map reader (`lib/mapping/MapFormatH3M.cpp`, `MapReaderH3M.cpp`, `MapFeaturesH3M.cpp`;
//! GPL-2.0-or-later, the order and sizes were ported, no code) and are checked against every map
//! shipped with the game (`examples/original_data.rs`). Notes: `docs/formats/h3m.md`.

use crate::cursor::Cursor;
use crate::model::{
    Artifacts, Bits, CustomHero, DisposedHero, Header, Loss, MainHero, MainTown, Player,
    PlayerDetails, PlayerHero, RawMap, Rumor, Teams, Terrain, Tile, Version, Victory, VictoryGoal,
};
use crate::{Limits, ReadError};

/// "No hero" in a hero type byte.
const NO_HERO: u8 = 0xFF;
/// The standard condition in a victory or loss condition byte.
const STANDARD: u8 = 0xFF;
/// Bytes of a tile.
const TILE_BYTES: usize = 7;

/// Parses the inflated bytes of a map file (RoE, AB or SoD) within `limits`.
///
/// E08 T3a reads up to the end of the terrain and ignores what follows (the object templates,
/// the objects and the timed events, T3b; ADV-STUB-1).
///
/// # Errors
///
/// [`ReadError`]: an unsupported version, data that ends early, a count or string beyond its
/// limit, or a condition kind the version does not have.
pub fn parse(bytes: &[u8], limits: &Limits) -> Result<RawMap, ReadError> {
    let mut c = Cursor::new(bytes, limits);
    let header = header(&mut c)?;
    let v = header.version;
    let mut players: [Player; 8] = Default::default();
    for player in &mut players {
        *player = self::player(&mut c, v)?;
    }
    let victory = victory(&mut c, v)?;
    let loss = loss(&mut c)?;
    let teams = teams(&mut c)?;
    let allowed_heroes = c.bits(v.hero_types().div_ceil(8), "allowed heroes")?;
    let placeholders = if v == Version::RoE {
        Vec::new()
    } else {
        let n = c.count("placeholders", c.limits.max_placeholders, 1)?;
        c.take(n, "placeholders")?.to_vec()
    };
    let disposed_heroes = if v == Version::Sod {
        let n = c.count_u8("disposed heroes", 7)?;
        (0..n)
            .map(|_| disposed_hero(&mut c))
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    c.skip(31, "map options")?;
    let banned_artifacts = match v {
        Version::RoE => Bits::default(),
        Version::Ab | Version::Sod => c.bits(v.artifacts().div_ceil(8), "banned artifacts")?,
    };
    let (banned_spells, banned_skills) = if v == Version::Sod {
        (c.bits(9, "banned spells")?, c.bits(4, "banned skills")?)
    } else {
        (Bits::default(), Bits::default())
    };
    let n = c.count("rumors", c.limits.max_rumors, 8)?;
    let rumors = (0..n)
        .map(|_| {
            Ok(Rumor {
                name: c.text("rumor name")?,
                text: c.text("rumor text")?,
            })
        })
        .collect::<Result<_, ReadError>>()?;
    let mut custom_heroes = Vec::new();
    if v == Version::Sod {
        for hero_type in 0..u8::try_from(v.hero_types()).unwrap_or(u8::MAX) {
            if c.bool("custom hero")? {
                custom_heroes.push(custom_hero(&mut c, v, hero_type)?);
            }
        }
    }
    let terrain = terrain(&mut c, &header)?;
    // stub: ADV-STUB-1: the object templates, the objects and the timed events follow (T3b).
    Ok(RawMap {
        header,
        players,
        victory,
        loss,
        teams,
        allowed_heroes,
        placeholders,
        disposed_heroes,
        banned_artifacts,
        banned_spells,
        banned_skills,
        rumors,
        custom_heroes,
        terrain,
    })
}

fn header(c: &mut Cursor<'_>) -> Result<Header, ReadError> {
    let code = c.u32("version")?;
    let version = Version::from_code(code).ok_or(ReadError::UnsupportedVersion(code))?;
    let any_players = c.bool("any players")?;
    let offset = c.offset();
    let size = c.u32("map size")?;
    if size == 0 {
        return Err(ReadError::InvalidValue {
            field: "map size",
            value: 0,
            offset,
        });
    }
    if size > c.limits.max_size {
        return Err(ReadError::LimitExceeded {
            field: "map size",
            value: u64::from(size),
            max: u64::from(c.limits.max_size),
            offset,
        });
    }
    Ok(Header {
        version,
        any_players,
        size,
        two_levels: c.bool("underground")?,
        name: c.text("map name")?,
        description: c.text("map description")?,
        difficulty: c.u8("difficulty")?,
        level_limit: if version == Version::RoE {
            0
        } else {
            c.u8("level limit")?
        },
    })
}

fn player(c: &mut Cursor<'_>, v: Version) -> Result<Player, ReadError> {
    let human = c.bool("player human")?;
    let computer = c.bool("player computer")?;
    if !(human || computer) {
        // The bytes of a colour nobody plays (VCMI: 6 in RoE, 6 more from AB, 1 more in SoD).
        let unused = match v {
            Version::RoE => 6,
            Version::Ab => 12,
            Version::Sod => 13,
        };
        c.skip(unused, "unplayable player")?;
        return Ok(Player {
            human,
            computer,
            details: None,
        });
    }
    let ai_tactic = c.i8("ai tactic")?;
    let town_choice = if v == Version::Sod {
        c.u8("town choice")?
    } else {
        0
    };
    let allowed_towns = if v.faction_bytes() == 1 {
        u16::from(c.u8("allowed towns")?)
    } else {
        c.u16("allowed towns")?
    };
    let random_town = c.bool("random town")?;
    let main_town = if c.bool("has main town")? {
        let (generate_hero, town_type) = if v == Version::RoE {
            (true, 0)
        } else {
            (c.bool("generate hero")?, c.u8("main town type")?)
        };
        Some(MainTown {
            generate_hero,
            town_type,
            pos: c.pos("main town")?,
        })
    } else {
        None
    };
    let random_hero = c.bool("random hero")?;
    let hero_type = c.u8("main hero")?;
    let main_hero = if hero_type == NO_HERO {
        None
    } else {
        Some(MainHero {
            hero_type,
            portrait: c.u8("main hero portrait")?,
            name: c.text("main hero name")?,
        })
    };
    let (unknown, heroes) = if v == Version::RoE {
        (0, Vec::new())
    } else {
        let unknown = c.u8("player unknown")?;
        let n = c.count("player heroes", c.limits.max_player_heroes, 5)?;
        let heroes = (0..n)
            .map(|_| {
                Ok(PlayerHero {
                    hero_type: c.u8("player hero")?,
                    name: c.text("player hero name")?,
                })
            })
            .collect::<Result<_, ReadError>>()?;
        (unknown, heroes)
    };
    Ok(Player {
        human,
        computer,
        details: Some(PlayerDetails {
            ai_tactic,
            town_choice,
            allowed_towns,
            random_town,
            main_town,
            random_hero,
            main_hero,
            unknown,
            heroes,
        }),
    })
}

/// An artifact or creature code: one byte in RoE, two from AB on.
fn id(c: &mut Cursor<'_>, v: Version, field: &'static str) -> Result<u16, ReadError> {
    if v == Version::RoE {
        c.u8(field).map(u16::from)
    } else {
        c.u16(field)
    }
}

/// An artifact code where the format's "none" (all bits set) means an empty slot.
fn artifact(c: &mut Cursor<'_>, v: Version) -> Result<Option<u16>, ReadError> {
    let none = if v == Version::RoE {
        u16::from(u8::MAX)
    } else {
        u16::MAX
    };
    id(c, v, "artifact").map(|a| (a != none).then_some(a))
}

fn victory(c: &mut Cursor<'_>, v: Version) -> Result<Victory, ReadError> {
    let offset = c.offset();
    let code = c.u8("victory condition")?;
    if code == STANDARD {
        return Ok(Victory::Standard);
    }
    if code > 10 {
        return Err(ReadError::InvalidValue {
            field: "victory condition",
            value: u64::from(code),
            offset,
        });
    }
    let allow_standard = c.bool("victory allows standard")?;
    let applies_to_ai = c.bool("victory applies to ai")?;
    let goal = match code {
        0 => VictoryGoal::AcquireArtifact {
            artifact: id(c, v, "victory artifact")?,
        },
        1 => VictoryGoal::AccumulateCreatures {
            creature: id(c, v, "victory creature")?,
            count: c.i32("victory creature count")?,
        },
        2 => VictoryGoal::AccumulateResources {
            resource: c.u8("victory resource")?,
            amount: c.i32("victory resource amount")?,
        },
        3 => VictoryGoal::UpgradeTown {
            pos: c.pos("victory town")?,
            hall: c.u8("victory hall level")?,
            castle: c.u8("victory castle level")?,
        },
        4 => VictoryGoal::BuildGrail {
            pos: c.pos("victory grail town")?,
        },
        5 => VictoryGoal::DefeatHero {
            pos: c.pos("victory hero")?,
        },
        6 => VictoryGoal::CaptureTown {
            pos: c.pos("victory town")?,
        },
        7 => VictoryGoal::DefeatMonster {
            pos: c.pos("victory monster")?,
        },
        8 => VictoryGoal::FlagDwellings,
        9 => VictoryGoal::FlagMines,
        _ => VictoryGoal::TransportArtifact {
            artifact: c.u8("victory transported artifact")?,
            pos: c.pos("victory destination")?,
        },
    };
    Ok(Victory::Special {
        allow_standard,
        applies_to_ai,
        goal,
    })
}

fn loss(c: &mut Cursor<'_>) -> Result<Loss, ReadError> {
    let offset = c.offset();
    match c.u8("loss condition")? {
        STANDARD => Ok(Loss::Standard),
        0 => Ok(Loss::LoseTown {
            pos: c.pos("loss town")?,
        }),
        1 => Ok(Loss::LoseHero {
            pos: c.pos("loss hero")?,
        }),
        2 => Ok(Loss::TimeExpires {
            days: c.u16("loss days")?,
        }),
        code => Err(ReadError::InvalidValue {
            field: "loss condition",
            value: u64::from(code),
            offset,
        }),
    }
}

fn teams(c: &mut Cursor<'_>) -> Result<Option<Teams>, ReadError> {
    let count = c.u8("team count")?;
    if count == 0 {
        return Ok(None);
    }
    Ok(Some(Teams {
        count,
        of_player: c.array("teams")?,
    }))
}

fn disposed_hero(c: &mut Cursor<'_>) -> Result<DisposedHero, ReadError> {
    Ok(DisposedHero {
        hero_type: c.u8("disposed hero")?,
        portrait: c.u8("disposed hero portrait")?,
        name: c.text("disposed hero name")?,
        players: c.u8("disposed hero players")?,
    })
}

fn custom_hero(c: &mut Cursor<'_>, v: Version, hero_type: u8) -> Result<CustomHero, ReadError> {
    let experience = if c.bool("custom hero has experience")? {
        Some(c.u32("custom hero experience")?)
    } else {
        None
    };
    let secondary_skills = if c.bool("custom hero has skills")? {
        Some(secondary_skills(c)?)
    } else {
        None
    };
    let artifacts = artifacts(c, v)?;
    let biography = if c.bool("custom hero has biography")? {
        Some(c.text("custom hero biography")?)
    } else {
        None
    };
    let sex = c.u8("custom hero sex")?;
    let spells = if c.bool("custom hero has spells")? {
        Some(c.bits(9, "custom hero spells")?)
    } else {
        None
    };
    let primary_skills = if c.bool("custom hero has primary skills")? {
        Some(c.array("custom hero primary skills")?)
    } else {
        None
    };
    Ok(CustomHero {
        hero_type,
        experience,
        secondary_skills,
        artifacts,
        biography,
        sex,
        spells,
        primary_skills,
    })
}

/// A `u32` count of (skill code, level code) pairs.
fn secondary_skills(c: &mut Cursor<'_>) -> Result<Vec<(u8, u8)>, ReadError> {
    let n = c.count("secondary skills", c.limits.max_secondary_skills, 2)?;
    (0..n)
        .map(|_| Ok((c.u8("secondary skill")?, c.u8("secondary skill level")?)))
        .collect()
}

/// A hero's artifacts behind their "customised" flag.
fn artifacts(c: &mut Cursor<'_>, v: Version) -> Result<Option<Artifacts>, ReadError> {
    if !c.bool("has artifacts")? {
        return Ok(None);
    }
    let worn = (0..v.artifact_slots())
        .map(|_| artifact(c, v))
        .collect::<Result<_, _>>()?;
    let n = c.count_u16("backpack", c.limits.max_backpack, v.id_bytes())?;
    let backpack = (0..n).map(|_| artifact(c, v)).collect::<Result<_, _>>()?;
    Ok(Some(Artifacts { worn, backpack }))
}

fn terrain(c: &mut Cursor<'_>, header: &Header) -> Result<Terrain, ReadError> {
    // The size is at most `Limits::max_size`, checked in the header.
    let size = usize::try_from(header.size).unwrap_or(usize::MAX);
    let levels = header.levels();
    let count = size.saturating_mul(size).saturating_mul(levels);
    let bytes = c.take(count.saturating_mul(TILE_BYTES), "terrain")?;
    let tiles = bytes
        .chunks_exact(TILE_BYTES)
        .map(|t| Tile {
            terrain: t[0],
            view: t[1],
            river: t[2],
            river_dir: t[3],
            road: t[4],
            road_dir: t[5],
            flags: t[6],
        })
        .collect();
    Ok(Terrain {
        size,
        levels,
        tiles,
    })
}
