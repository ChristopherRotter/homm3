//! `h3-view situation` (E17 M2 T1, *provisional*; `clients/capture/README.md`): the original's
//! map for a scenario's battle, so that the capture harness can show the original that battle and
//! compare its frames with ours. Red's hero (a human) stands just west of Blue's (the computer) on
//! the scenario's terrain, the scenario's heroes (E17 M3 T3a: each side names one, with its
//! primary skills), each with the scenario's stacks in their slots and its formation and its war
//! machines (E17 M3 T3b: their artifacts) and its spell book (M3 T7a-2a: the book in its slot,
//! its spells, and the school skills behind its masteries, M3 T7a-2b), the secondary skills
//! behind its battle effects (M3 T7b-2b-2a: Leadership for its morale, Luck for its luck) and its
//! tactics level (E17 M5 T4c2a: Tactics, the phase with it), no other skills or artifacts;
//! stepping Red's hero east starts the battle
//! (`h3drive.py`). With `--hot-seat` (E17 M3 T1a) Blue may be a human too, so that the
//! harness plays both sides of a hot-seat battle (`hot_seat` in `sequences.toml`). A special
//! battlefield (E17 M3 T2) is set up as the original makes it, on the defender's tile: water
//! beside Blue for the shore, a special ground under both heroes for the others. The creatures'
//! original ids come from `crtraits.txt` (`H3bitmap.lod`) by the packs' names, as `harvest` keys
//! the asset tables, the heroes' from the ruleset checked against its `hotraits.txt`; the grounds'
//! pictures from its `Objects.txt`. An adapter: it reads files and
//! writes one.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_formats::text::TextTable;
use h3_map::writer::{
    ARMY_SLOTS, ARTIFACT_SLOTS, Ground, Hero, HeroArtifacts, Map, MapObject, ObjectTemplate,
    PLAYERS, Patch, Player, TOWN_OBJECT, Terrain, Town, TownBuildings, gzip, write,
};

use crate::gate::find_ci;
use crate::harvest::{creature_names, hero_names, spell_numbers};
use crate::scenario::{Book, Sides, sides};
/// Where Blue stands on the 36 × 36 map by default; Red stands one tile west. The original's
/// obstacles depend on the battle's tile (`--tile`: the capture set's other battlefields).
const BLUE_TILE: (u8, u8) = (11, 10);

/// The special grounds by their battlefield's key (the packs' `ground` object types,
/// `packs/base/adventure.toml` and `packs/sod/adventure.toml`, ADV-GRD-001): the object class
/// whose first picture in `Objects.txt` the map places (h3-map's import reads the same classes).
const GROUNDS: [(&str, u32); 9] = [
    ("cursed_ground", 21),
    ("magic_plains", 46),
    ("clover_field", 222),
    ("evil_fog", 224),
    ("fiery_fields", 226),
    ("holy_ground", 227),
    ("lucid_pools", 228),
    ("magic_clouds", 229),
    ("rocklands", 231),
];

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    scenario: PathBuf,
    packs: PathBuf,
    install: PathBuf,
    out: PathBuf,
    tile: (u8, u8),
    hot_seat: bool,
}

impl Options {
    /// Parses `<scenario.toml> [--packs <dir>] [--install <dir>] [--out <file.h3m>] [--tile x,y]
    /// [--hot-seat]`; the packs and the installation default as `render`'s, the map to
    /// `clients/out/situation/<stem>.h3m`, Blue's tile to (11, 10).
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (mut scenario, mut packs, mut install, mut out) = (None, None, None, None);
        let mut tile = BLUE_TILE;
        let mut hot_seat = false;
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let slot = match arg.as_str() {
                "--tile" => {
                    let value = rest.next().map_or("", String::as_str);
                    tile = parse_tile(value)
                        .ok_or_else(|| format!("situation: bad tile `{value}` (x,y: 1 to 34)"))?;
                    continue;
                }
                "--hot-seat" => {
                    hot_seat = true;
                    continue;
                }
                "--packs" => &mut packs,
                "--install" => &mut install,
                "--out" => &mut out,
                flag if flag.starts_with("--") => {
                    return Err(format!("situation: unknown option `{flag}`"));
                }
                path => {
                    if scenario.replace(PathBuf::from(path)).is_some() {
                        return Err("situation: one scenario only".to_owned());
                    }
                    continue;
                }
            };
            let value = rest
                .next()
                .ok_or_else(|| format!("situation: `{arg}` needs a value"))?;
            *slot = Some(PathBuf::from(value));
        }
        let scenario = scenario.ok_or("situation: which scenario?")?;
        let stem = scenario
            .file_stem()
            .map_or_else(|| "situation".into(), |s| s.to_string_lossy().into_owned());
        Ok(Self {
            tile,
            hot_seat,
            out: out.unwrap_or_else(|| manifest.join(format!("../../out/situation/{stem}.h3m"))),
            scenario,
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
        })
    }
}

/// A tile `x,y` with room for Red one tile west, inside the 36 × 36 map's border.
fn parse_tile(text: &str) -> Option<(u8, u8)> {
    let (x, y) = text.split_once(',')?;
    let (x, y): (u8, u8) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
    ((2..=34).contains(&x) && (1..=34).contains(&y)).then_some((x, y))
}

pub fn run(options: &Options) -> ExitCode {
    match situation(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("situation: {message}");
            ExitCode::FAILURE
        }
    }
}

fn situation(options: &Options) -> Result<(), String> {
    let battle = sides(&options.scenario, &options.packs)?;
    let Originals {
        numbers,
        heroes,
        objects,
    } = original_tables(&options.install)?;
    let id = options.scenario.display().to_string();
    for hero in battle.heroes.iter().flatten() {
        if heroes.get(hero.hero_type) != Some(&hero.name) {
            return Err(format!(
                "{id}: {} is not hero type {} of hotraits.txt (the packs in their order: base, ab, sod)",
                hero.name, hero.hero_type
            ));
        }
    }
    // The map's players (below): Red human, Blue a computer or, in hot seat, a human too; the
    // scenario's record says the same, so that the viewer's panel shows what the original's does
    // (E17 M3 T6a).
    let human = [true, options.hot_seat];
    if battle.human != human {
        let names = |h: [bool; 2]| {
            let list: Vec<&str> = ["attacker", "defender"]
                .into_iter()
                .zip(h)
                .filter_map(|(side, on)| on.then_some(side))
                .collect();
            format!("{list:?}")
        };
        return Err(format!(
            "{id}: the scenario's `human` is {}, the original's map has {} (--hot-seat: both)",
            names(battle.human),
            names(human)
        ));
    }
    let mut map = map(&id, &battle, &numbers, options.tile, options.hot_seat)?;
    let (patches, ground) = special(battle.battlefield.as_deref(), &objects, options.tile)?;
    map.patches = patches;
    map.objects.extend(ground);
    if let Some((town, fort)) = battle.siege {
        if battle.battlefield.is_some() {
            return Err(format!("{id}: a siege on a special battlefield"));
        }
        map.objects
            .push(siege_town(town, fort, &objects, options.tile)?);
    }
    let bytes = write(&map).map_err(|e| format!("the map: {e}"))?;
    if let Some(dir) = options.out.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&options.out, gzip(&bytes)).map_err(|e| format!("{}: {e}", options.out.display()))?;
    println!("situation: {}", options.out.display());
    Ok(())
}

/// What the map takes from the installation's `H3bitmap.lod`: `crtraits.txt`'s creature ids and
/// `sptraits.txt`'s spell numbers by name, `hotraits.txt`'s hero names by hero type, and
/// `Objects.txt` (the map objects' templates).
struct Originals {
    numbers: Numbers,
    heroes: Vec<String>,
    objects: String,
}

/// The original's numbers by name: the creatures' (singular names) and the spells' (E17 M3
/// T7a-2a).
struct Numbers {
    creatures: BTreeMap<String, usize>,
    spells: BTreeMap<String, usize>,
}

/// The installation's [`Originals`].
fn original_tables(install: &Path) -> Result<Originals, String> {
    let data = find_ci(install, "Data").map_err(|e| format!("{}: Data: {e}", install.display()))?;
    let path = find_ci(&data, "H3bitmap.lod").map_err(|e| format!("H3bitmap.lod: {e}"))?;
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let lod = LodArchive::parse(&bytes).map_err(|e| format!("H3bitmap.lod: {e:?}"))?;
    let entry = |name: &str| -> Result<Vec<u8>, String> {
        lod.read_by_name(name)
            .map_err(|e| format!("{name}: {e:?}"))?
            .ok_or(format!("H3bitmap.lod has no {name}"))
    };
    let table = entry("CRTRAITS.TXT")?;
    let ids =
        creature_names(&TextTable::parse(&table).map_err(|e| format!("CRTRAITS.TXT: {e:?}"))?)?;
    let table = entry("SPTRAITS.TXT")?;
    let spells =
        spell_numbers(&TextTable::parse(&table).map_err(|e| format!("SPTRAITS.TXT: {e:?}"))?);
    let table = entry("HOTRAITS.TXT")?;
    let heroes = hero_names(&TextTable::parse(&table).map_err(|e| format!("HOTRAITS.TXT: {e:?}"))?);
    // Latin-1, one byte a character.
    let objects = entry("OBJECTS.TXT")?.into_iter().map(char::from).collect();
    Ok(Originals {
        numbers: Numbers {
            creatures: ids,
            spells,
        },
        heroes,
        objects,
    })
}

/// How the original fights the special battlefield `key` at Blue's `tile` (the battle's tile is
/// the defender's; survey `E04c-battlefields.md` §1, §4): the shore, a land tile beside water
/// (the writer gives it the coast flag 0x40), so water east of Blue; a special ground's field,
/// the ground under the tile: its class's first picture in `objects` (`Objects.txt`), its
/// bottom-right tile one east and one south of Blue's, so that the picture (3 × 3 or 6 × 4
/// tiles) lies under both heroes. A ship's deck or two boats need boats, which h3-map's writer
/// lacks (CLT-Q5).
fn special(
    key: Option<&str>,
    objects: &str,
    tile: (u8, u8),
) -> Result<(Vec<Patch>, Option<MapObject>), String> {
    let Some(key) = key else {
        return Ok((Vec::new(), None));
    };
    let name = key.rsplit(':').next().unwrap_or(key);
    if name == "shore" {
        let water = Patch {
            terrain: Terrain::Water,
            from: (tile.0 + 1, tile.1 - 1),
            to: ((tile.0 + 3).min(35), tile.1 + 1),
        };
        return Ok((vec![water], None));
    }
    let class = GROUNDS
        .iter()
        .find(|(k, _)| *k == name)
        .map(|&(_, class)| class)
        .ok_or_else(|| {
            format!("no map for the battlefield `{key}` (a ship's deck, two boats: CLT-Q5)")
        })?;
    let line = objects
        .lines()
        .find(|l| l.split_whitespace().nth(5) == Some(class.to_string().as_str()))
        .ok_or(format!("Objects.txt has no picture of class {class}"))?;
    let template = ObjectTemplate::from_objects_txt(line).map_err(|e| e.to_string())?;
    let ground = Ground {
        tile: (tile.0 + 1, tile.1 + 1),
        template,
    };
    Ok((Vec::new(), Some(MapObject::Ground(ground))))
}

/// The map's building bits of the fort levels (`docs/formats/h3m.md`, "Town buildings"): the
/// Fort's 3, the Citadel's 4, the Castle's 5 (the Village Hall, which every town has, has none).
/// A level is built with those below it, as the experiments' sieges build them (E04c T5b).
const FORT_BITS: [u8; 3] = [3, 4, 5];

/// The town of a siege (E17 M4 T1b): Blue's, of town type `town` with the fort level `fort` (0
/// to 3) and no other building but the Village Hall, an empty garrison, its entrance on Blue's
/// `tile`, so that Blue's hero is its visiting hero; its picture is `objects`' (`Objects.txt`)
/// first of class 98 with the town type as subclass, its anchor two tiles east of the entrance
/// (h3-map's `Town`). Red stands south of the entrance ([`map`]) and steps north into the town
/// (the situation's `key = "Up"`, `h3drive.py`).
fn siege_town(town: usize, fort: u8, objects: &str, tile: (u8, u8)) -> Result<MapObject, String> {
    let anchor = (tile.0 + 2, tile.1);
    if tile.0 < 5 || tile.1 < 5 || anchor.0 > 35 || tile.1 > 34 {
        return Err(format!(
            "a town with its entrance on {tile:?} does not fit the map (5 columns west, 2 east, 5 \
             rows north and Red's one south)"
        ));
    }
    let (class, subclass) = (TOWN_OBJECT.to_string(), town.to_string());
    let line = objects
        .lines()
        .find(|l| {
            let mut fields = l.split_whitespace().skip(5);
            fields.next() == Some(class.as_str()) && fields.next() == Some(subclass.as_str())
        })
        .ok_or(format!("Objects.txt has no town of type {town}"))?;
    let template = ObjectTemplate::from_objects_txt(line).map_err(|e| e.to_string())?;
    let mut built = [0u8; 6];
    for &bit in FORT_BITS.iter().take(usize::from(fort)) {
        built[usize::from(bit / 8)] |= 1 << (bit % 8);
    }
    Ok(MapObject::Town(Town {
        tile: anchor,
        template,
        owner: Some(1),
        garrison: Some([None; ARMY_SLOTS]),
        tight_formation: false,
        buildings: TownBuildings::Custom {
            built,
            forbidden: [0; 6],
        },
    }))
}

/// The map terrain of a pack's terrain key (`base:grass`): the packs' land terrains in the map
/// format's order (`packs/base/core.toml`; `desert` is the map's sand).
fn terrain(key: &str) -> Result<Terrain, String> {
    let name = key.rsplit(':').next().unwrap_or(key);
    Ok(match name {
        "dirt" => Terrain::Dirt,
        "desert" => Terrain::Sand,
        "grass" => Terrain::Grass,
        "snow" => Terrain::Snow,
        "swamp" => Terrain::Swamp,
        "rough" => Terrain::Rough,
        "subterranean" => Terrain::Subterranean,
        "lava" => Terrain::Lava,
        _ => {
            return Err(format!(
                "no land battle on `{key}` (water and rock: a boat, or none)"
            ));
        }
    })
}

/// One side's army: (original id, count) by slot.
fn army(
    stacks: &[(u8, String, u32)],
    ids: &BTreeMap<String, usize>,
) -> Result<[Option<(u16, u16)>; ARMY_SLOTS], String> {
    let mut army = [None; ARMY_SLOTS];
    for (slot, name, count) in stacks {
        let id = ids
            .get(name)
            .and_then(|&id| u16::try_from(id).ok())
            .ok_or_else(|| format!("`{name}` is not in crtraits.txt"))?;
        let count = u16::try_from(*count).map_err(|_| format!("{name}: {count} is too many"))?;
        let cell = army
            .get_mut(usize::from(*slot))
            .ok_or_else(|| format!("{name}: no slot {slot}"))?;
        *cell = Some((id, count));
    }
    Ok(army)
}

/// A hero's worn artifacts: its war machines only (E17 M3 T3b), each in its slot.
fn worn(
    machines: &[(usize, u16)],
    book: Option<&Book>,
) -> Result<[Option<u16>; ARTIFACT_SLOTS], String> {
    let mut worn = [None; ARTIFACT_SLOTS];
    let book = book.map(|b| (BOOK_SLOT, b.artifact));
    for &(slot, artifact) in machines.iter().chain(&book) {
        *worn
            .get_mut(slot)
            .ok_or_else(|| format!("artifact {artifact}: no slot {slot}"))? = Some(artifact);
    }
    Ok(worn)
}

/// The map's worn slot of the spell book (`ArtSlots.txt`'s order; E17 M3 T7a-2a).
const BOOK_SLOT: usize = 17;

/// A spell book's 9 bytes, its spells numbered by name by `numbers` (`sptraits.txt`): spell `i`
/// at bit `i mod 8` of byte `i / 8` (docs/formats/h3m.md); only the first 70 fit a book.
fn spell_bits(spells: &[String], numbers: &BTreeMap<String, usize>) -> Result<[u8; 9], String> {
    let mut bits = [0; 9];
    for name in spells {
        let spell = numbers
            .get(name)
            .copied()
            .filter(|&n| n < 8 * bits.len() - 2)
            .ok_or_else(|| format!("spell `{name}`: no book's spell of sptraits.txt"))?;
        bits[spell / 8] |= 1 << (spell % 8);
    }
    Ok(bits)
}

/// The map of the battle `battle` (`id` names it in the scenario list), Blue on `tile`; Blue may
/// be a human with `hot_seat`.
fn map(
    id: &str,
    battle: &Sides,
    numbers: &Numbers,
    tile: (u8, u8),
    hot_seat: bool,
) -> Result<Map, String> {
    let ground = terrain(
        battle
            .terrain
            .as_deref()
            .ok_or("the scenario has no terrain")?,
    )?;
    let hero = |tile, owner: usize| -> Result<MapObject, String> {
        let side = ["attacker", "defender"][owner];
        let h = battle.heroes[owner]
            .as_ref()
            .ok_or_else(|| format!("{id}: no {side}_hero (the original's map needs both)"))?;
        let number = |n: usize| u8::try_from(n).map_err(|_| format!("{}: number {n}", h.name));
        let (hero_type, class) = (number(h.hero_type)?, number(h.class)?);
        Ok(MapObject::Hero(Hero {
            tile,
            owner: u8::try_from(owner).unwrap_or(u8::MAX),
            hero_type,
            class,
            name: None,
            experience: Some(0),
            // The school skills behind the masteries (E17 M3 T7a-2b; none without a book), then
            // those behind the battle effects (T7b-2b-2a).
            skills: Some(
                h.book
                    .iter()
                    .flat_map(|b| b.skills.iter().copied())
                    .chain(h.skills.iter().copied())
                    .collect(),
            ),
            army: Some(army(&battle.stacks[owner], &numbers.creatures)?),
            tight_formation: battle.tight[owner],
            artifacts: Some(HeroArtifacts {
                worn: worn(&battle.machines[owner], h.book.as_ref())?,
                backpack: Vec::new(),
            }),
            // A hero without a book keeps its type's spells (none for the scenarios' heroes).
            spells: h
                .book
                .as_ref()
                .map(|b| spell_bits(&b.spells, &numbers.spells))
                .transpose()?,
            primary: Some(h.primary),
        }))
    };
    let red = if battle.siege.is_some() {
        (tile.0, tile.1 + 1)
    } else {
        (tile.0 - 1, tile.1)
    };
    let mut players: [Option<Player>; PLAYERS] = Default::default();
    players[0] = Some(Player {
        human: true,
        computer: true,
        towns: 1,
    });
    players[1] = Some(Player {
        human: hot_seat,
        computer: true,
        towns: 1 << 6,
    });
    Ok(Map {
        name: "h3-view situation".into(),
        description: format!("Generated by h3-view situation (E17 M2 T1) from {id}."),
        size: 36,
        terrain: ground,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        // Red west of Blue, or south of the town's entrance in a siege (E17 M4 T1b).
        objects: vec![hero(red, 0)?, hero(tile, 1)?],
        underground: None,
        // No level limit (E08 T32a's field; the byte the writer wrote before it).
        level_limit: 0,
        // No banned skills (E08 T36b's field; the bytes the writer wrote before it).
        banned_skills: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::SideHero;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    fn ids() -> Numbers {
        let named = |list: &[(&str, usize)]| {
            list.iter()
                .map(|&(n, i)| (n.to_owned(), i))
                .collect::<BTreeMap<_, _>>()
        };
        Numbers {
            creatures: named(&[("Pikeman", 0), ("Archer", 2), ("Goblin", 84)]),
            spells: named(&[
                ("Summon Boat", 0),
                ("Magic Arrow", 15),
                ("Slow", 54),
                ("Air Elemental", 69),
                ("Stone Gaze", 70),
            ]),
        }
    }

    fn sides_of(terrain: Option<&str>, attacker: &[(u8, &str, u32)]) -> Sides {
        let stacks = |list: &[(u8, &str, u32)]| {
            list.iter()
                .map(|&(s, n, c)| (s, n.to_owned(), c))
                .collect::<Vec<_>>()
        };
        let hero = |hero_type, class, name: &str| {
            Some(SideHero {
                hero_type,
                class,
                name: name.to_owned(),
                primary: [1, 2, 3, 4],
                book: None,
                skills: Vec::new(),
            })
        };
        Sides {
            terrain: terrain.map(str::to_owned),
            battlefield: None,
            stacks: [stacks(attacker), stacks(&[(0, "Goblin", 30)])],
            tight: [false, true],
            heroes: [hero(0, 0, "Orrin"), hero(102, 12, "Crag Hack")],
            human: [true, false],
            machines: [vec![(13, 4), (15, 6)], Vec::new()],
            siege: None,
        }
    }

    #[test]
    fn the_options_default_beside_the_scenario() {
        let o = Options::parse_with(&args(&["scenarios/a.toml"]), None).unwrap();
        assert!(o.out.ends_with("out/situation/a.h3m"));
        assert_eq!((o.tile, o.hot_seat), ((11, 10), false));
        let o = Options::parse_with(
            &args(&[
                "x.toml",
                "--out",
                "m.h3m",
                "--install",
                "i",
                "--packs",
                "p",
                "--tile",
                "20,5",
                "--hot-seat",
            ]),
            Some(PathBuf::from("env")),
        )
        .unwrap();
        assert_eq!(
            (o.out, o.install, o.packs, o.tile, o.hot_seat),
            ("m.h3m".into(), "i".into(), "p".into(), (20, 5), true)
        );
        for bad in ["1,5", "35,5", "5,0", "5", "a,b"] {
            assert!(Options::parse_with(&args(&["x", "--tile", bad]), None).is_err());
        }
        assert!(Options::parse_with(&args(&[]), None).is_err());
        assert!(Options::parse_with(&args(&["a", "b"]), None).is_err());
        assert!(Options::parse_with(&args(&["a", "--out"]), None).is_err());
        assert!(Options::parse_with(&args(&["a", "--bad", "x"]), None).is_err());
    }

    #[test]
    fn a_battle_becomes_two_heroes_side_by_side() {
        let battle = sides_of(
            Some("base:desert"),
            &[(0, "Archer", 20), (6, "Pikeman", 10)],
        );
        let map = map("t", &battle, &ids(), BLUE_TILE, false).unwrap();
        assert_eq!(map.terrain, Terrain::Sand);
        let [MapObject::Hero(red), MapObject::Hero(blue)] = &map.objects[..] else {
            panic!("two heroes");
        };
        assert_eq!(
            (red.owner, red.tile, blue.owner, blue.tile),
            (0, (10, 10), 1, (11, 10))
        );
        let army = red.army.unwrap();
        assert_eq!(
            (army[0], army[6], army[1]),
            (Some((2, 20)), Some((0, 10)), None)
        );
        assert_eq!(blue.army.unwrap()[0], Some((84, 30)));
        assert!(!red.tight_formation && blue.tight_formation);
        assert_eq!(red.skills, Some(Vec::new()));
        assert_eq!(
            (red.hero_type, red.class, blue.hero_type, blue.class),
            (0, 0, 102, 12)
        );
        assert_eq!(red.primary, Some([1, 2, 3, 4]));
        // The skills behind the battle effects after the book's (E17 M3 T7b-2b-2a).
        let mut lucky = sides_of(Some("base:desert"), &[]);
        lucky.heroes[0] = lucky.heroes[0].clone().map(|h| SideHero {
            book: Some(Book {
                artifact: 0,
                spells: vec!["Slow".to_owned()],
                skills: vec![(17, 3)],
            }),
            skills: vec![(6, 1), (9, 3)],
            ..h
        });
        let lucky = super::map("t", &lucky, &ids(), BLUE_TILE, false).unwrap();
        let MapObject::Hero(lucky) = &lucky.objects[0] else {
            panic!("a hero");
        };
        assert_eq!(lucky.skills, Some(vec![(17, 3), (6, 1), (9, 3)]));
        // Each side needs a hero, of a number the map holds.
        let mut alone = sides_of(Some("base:desert"), &[]);
        alone.heroes[1] = None;
        assert!(super::map("t", &alone, &ids(), BLUE_TILE, false).is_err());
        alone.heroes[1] = alone.heroes[0].clone().map(|h| SideHero {
            hero_type: 300,
            ..h
        });
        assert!(super::map("t", &alone, &ids(), BLUE_TILE, false).is_err());
        // Red's war machines in their slots (E17 M3 T3b), Blue without.
        let worn = red.artifacts.as_ref().unwrap().worn;
        assert_eq!((worn[13], worn[14], worn[15]), (Some(4), None, Some(6)));
        assert_eq!(worn.iter().flatten().count(), 2);
        assert!(
            blue.artifacts
                .as_ref()
                .unwrap()
                .worn
                .iter()
                .all(Option::is_none)
        );
        let mut bad = sides_of(Some("base:desert"), &[]);
        bad.machines[1] = vec![(19, 4)];
        assert!(
            super::map("t", &bad, &ids(), BLUE_TILE, false)
                .unwrap_err()
                .contains("slot 19")
        );
        // Without a book the heroes keep their types' spells (E17 M3 T7a-2a).
        assert_eq!((red.spells, blue.spells), (None, None));
        // A book: worn in slot 17, its spells as bits.
        let mut read = sides_of(Some("base:desert"), &[]);
        let book = |spells: &[&str]| {
            Some(Book {
                artifact: 0,
                spells: spells.iter().map(|s| (*s).to_owned()).collect(),
                skills: vec![(17, 3)],
            })
        };
        if let Some(h) = read.heroes[0].as_mut() {
            h.book = book(&["Summon Boat", "Magic Arrow", "Slow", "Air Elemental"]);
        }
        let booked = super::map("t", &read, &ids(), BLUE_TILE, false).unwrap();
        let MapObject::Hero(reader) = &booked.objects[0] else {
            panic!("a hero");
        };
        assert_eq!(reader.artifacts.as_ref().unwrap().worn[17], Some(0));
        assert_eq!(reader.spells, Some([1, 0x80, 0, 0, 0, 0, 0x40, 0, 0x20]));
        // Its school skills (E17 M3 T7a-2b): Expert Earth Magic.
        assert_eq!(reader.skills, Some(vec![(17, 3)]));
        assert!(write(&booked).is_ok());
        // A spell the book cannot hold (70 on: the creatures' abilities), or one not in the table.
        for bad in ["Stone Gaze", "Fly Away"] {
            if let Some(h) = read.heroes[0].as_mut() {
                h.book = book(&[bad]);
            }
            let error = super::map("t", &read, &ids(), BLUE_TILE, false).unwrap_err();
            assert!(error.contains(bad), "{error}");
        }
        assert!(!map.players[1].as_ref().unwrap().human);
        // The writer accepts it.
        assert!(write(&map).is_ok());
        // In hot seat Blue may be a human too.
        let hot = super::map("t", &battle, &ids(), BLUE_TILE, true).unwrap();
        assert!(hot.players[1].as_ref().unwrap().human);
    }

    #[test]
    fn what_the_original_cannot_show_is_an_error() {
        let ok = |t, a: &[(u8, &str, u32)]| map("t", &sides_of(t, a), &ids(), (20, 20), false);
        assert!(ok(None, &[]).unwrap_err().contains("no terrain"));
        assert!(ok(Some("base:water"), &[]).unwrap_err().contains("water"));
        assert!(
            ok(Some("grass"), &[(0, "Angel", 1)])
                .unwrap_err()
                .contains("Angel")
        );
        assert!(
            ok(Some("grass"), &[(0, "Archer", 70_000)])
                .unwrap_err()
                .contains("too many")
        );
        assert!(
            ok(Some("grass"), &[(7, "Archer", 1)])
                .unwrap_err()
                .contains("slot 7")
        );
        for (key, terrain) in [
            ("dirt", Terrain::Dirt),
            ("grass", Terrain::Grass),
            ("snow", Terrain::Snow),
            ("swamp", Terrain::Swamp),
            ("rough", Terrain::Rough),
            ("subterranean", Terrain::Subterranean),
            ("lava", Terrain::Lava),
        ] {
            assert_eq!(super::terrain(key), Ok(terrain));
        }
    }

    #[test]
    fn a_missing_installation_is_an_error() {
        assert!(original_tables(Path::new("/no/such/install")).is_err());
    }

    /// Two lines of `Objects.txt` (numbers and pictures' names only).
    const OBJECTS: &str = "2\n\
        AVXcrsd0.def 111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 011111111 21 0 1 0\n\
        AVXhg0.def 111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 011111111 227 0 1 0\n";

    #[test]
    fn a_special_battlefield_lies_under_the_defender() {
        let blue = BLUE_TILE;
        assert_eq!(special(None, OBJECTS, blue), Ok((Vec::new(), None)));
        // The shore: water east of Blue's tile.
        let (patches, ground) = special(Some("base:shore"), OBJECTS, blue).unwrap();
        assert_eq!(
            (patches[0].terrain, patches[0].from, patches[0].to),
            (Terrain::Water, (12, 9), (14, 11))
        );
        assert!(ground.is_none());
        // At the map's eastern edge the water stops at the last column.
        let (patches, _) = special(Some("base:shore"), OBJECTS, (34, 10)).unwrap();
        assert_eq!(patches[0].to, (35, 11));
        // A special ground's first picture, its bottom-right tile south-east of Blue.
        for (key, def) in [
            ("base:cursed_ground", "AVXcrsd0.def"),
            ("sod:holy_ground", "AVXhg0.def"),
        ] {
            let (patches, ground) = special(Some(key), OBJECTS, blue).unwrap();
            let Some(MapObject::Ground(g)) = ground else {
                panic!("{key}: a ground");
            };
            assert_eq!(
                (patches.len(), g.tile, g.template.def.as_str()),
                (0, (12, 11), def)
            );
        }
        // The water fields need boats; a ground without a picture cannot be placed.
        assert!(
            special(Some("base:deck"), OBJECTS, blue)
                .unwrap_err()
                .contains("CLT-Q5")
        );
        assert!(special(Some("base:boats"), OBJECTS, blue).is_err());
        assert!(
            special(Some("sod:rocklands"), OBJECTS, blue)
                .unwrap_err()
                .contains("class 231")
        );
        let bad = "AVXrk0.def 1 2 3 4 231 0 1 0";
        assert!(special(Some("sod:rocklands"), bad, blue).is_err());
        // The map takes them: a ground under both heroes, water beside them.
        let mut map = map("t", &sides_of(Some("grass"), &[]), &ids(), blue, false).unwrap();
        let (patches, ground) = special(Some("base:cursed_ground"), OBJECTS, blue).unwrap();
        map.patches = patches;
        map.objects.extend(ground);
        assert!(write(&map).is_ok());
    }

    /// Two towns' pictures (`Objects.txt`'s lines of class 98: Castle and Conflux).
    const TOWNS: &str = "2\n\
        AVCcasx0.def 000001110000011110001111111111111111111111111111 \
        001000000000000000000000000000000000000000000000 011111111 011111111 98 0 1 0\n\
        avchforx.def 000001110000011110001111111111111111111111111111 \
        001000000000000000000000000000000000000000000000 011111111 011111111 98 8 1 0\n";

    #[test]
    fn a_siege_puts_blues_hero_into_its_town() {
        let blue = BLUE_TILE;
        let town = |t, fort| siege_town(t, fort, TOWNS, blue);
        let Ok(MapObject::Town(castle)) = town(0, 3) else {
            panic!("a town");
        };
        assert_eq!(
            (castle.tile, castle.owner, castle.template.def.as_str()),
            ((13, 10), Some(1), "AVCcasx0.def")
        );
        assert_eq!(castle.garrison, Some([None; ARMY_SLOTS]));
        // The Fort's, the Citadel's and the Castle's bits, each level with those below.
        let built = |t: &Town| match t.buildings {
            TownBuildings::Custom { built, .. } => built[0],
            TownBuildings::Standard { .. } => 0xFF,
        };
        assert_eq!(built(&castle), 0b11_1000);
        let Ok(MapObject::Town(conflux)) = town(8, 1) else {
            panic!("a town");
        };
        assert_eq!(
            (built(&conflux), conflux.template.def.as_str()),
            (0b1000, "avchforx.def")
        );
        assert!(town(5, 2).unwrap_err().contains("no town of type 5"));
        assert!(
            siege_town(0, 1, TOWNS, (4, 10))
                .unwrap_err()
                .contains("does not fit")
        );
        assert!(siege_town(0, 1, TOWNS, (11, 4)).is_err());
        assert!(siege_town(0, 1, TOWNS, (34, 10)).is_err());
        assert!(siege_town(0, 1, TOWNS, (11, 35)).is_err());
        let bad = "AVCcasx0.def 1 2 3 4 98 0 1 0";
        assert!(siege_town(0, 1, bad, blue).is_err());
        // Red stands south of the entrance; the map with the town is written.
        let mut battle = sides_of(Some("grass"), &[(0, "Archer", 20)]);
        battle.siege = Some((0, 3));
        let mut map = map("t", &battle, &ids(), blue, false).unwrap();
        let [MapObject::Hero(red), MapObject::Hero(b)] = &map.objects[..] else {
            panic!("two heroes");
        };
        assert_eq!((red.tile, b.tile), ((11, 11), blue));
        map.objects.push(MapObject::Town(castle));
        assert!(write(&map).is_ok());
    }
}
