//! `h3-view situation` (E17 M2 T1, *provisional*; `clients/capture/README.md`): the original's
//! map for a scenario's battle, so that the capture harness can show the original that battle and
//! compare its frames with ours. Red's hero (a human) stands just west of Blue's (the computer) on
//! the scenario's terrain, each with the scenario's stacks in their slots and its formation, no
//! secondary skills, artifacts or war machines (nothing the still does not draw, nothing that moves
//! on its own: no morale from Leadership, no Tactics phase); stepping Red's hero east starts the
//! battle (`h3drive.py`). With `--hot-seat` (E17 M3 T1a) Blue may be a human too, so that the
//! harness plays both sides of a hot-seat battle (`hot_seat` in `sequences.toml`). A special
//! battlefield (E17 M3 T2) is set up as the original makes it, on the defender's tile: water
//! beside Blue for the shore, a special ground under both heroes for the others. The creatures'
//! original ids come from `crtraits.txt` (`H3bitmap.lod`) by the packs' names, as `harvest` keys
//! the asset tables; the grounds' pictures from its `Objects.txt`. An adapter: it reads files and
//! writes one.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_formats::text::TextTable;
use h3_map::writer::{
    ARMY_SLOTS, ARTIFACT_SLOTS, Ground, Hero, HeroArtifacts, Map, MapObject, ObjectTemplate,
    PLAYERS, Patch, Player, Terrain, gzip, write,
};

use crate::gate::find_ci;
use crate::harvest::creature_names;
use crate::scenario::{Sides, sides};

/// Red's hero: Orrin, a Knight (hero type 0, class 0); Blue's: Crag Hack, a Barbarian (102, 12).
/// Neither starts with a spell book; their pictures are masked until the heroes are drawn.
const RED: (u8, u8) = (0, 0);
const BLUE: (u8, u8) = (102, 12);
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
    let (ids, objects) = original_tables(&options.install)?;
    let id = options.scenario.display().to_string();
    let mut map = map(&id, &battle, &ids, options.tile, options.hot_seat)?;
    let (patches, ground) = special(battle.battlefield.as_deref(), &objects, options.tile)?;
    map.patches = patches;
    map.objects.extend(ground);
    let bytes = write(&map).map_err(|e| format!("the map: {e}"))?;
    if let Some(dir) = options.out.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&options.out, gzip(&bytes)).map_err(|e| format!("{}: {e}", options.out.display()))?;
    println!("situation: {}", options.out.display());
    Ok(())
}

/// From the installation's `H3bitmap.lod`: `crtraits.txt`'s creature ids by singular name, and
/// `Objects.txt` (the map objects' templates).
fn original_tables(install: &Path) -> Result<(BTreeMap<String, usize>, String), String> {
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
    // Latin-1, one byte a character.
    let objects = entry("OBJECTS.TXT")?.into_iter().map(char::from).collect();
    Ok((ids, objects))
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

/// The map of the battle `battle` (`id` names it in the scenario list), Blue on `tile`; Blue may
/// be a human with `hot_seat`.
fn map(
    id: &str,
    battle: &Sides,
    ids: &BTreeMap<String, usize>,
    tile: (u8, u8),
    hot_seat: bool,
) -> Result<Map, String> {
    let ground = terrain(
        battle
            .terrain
            .as_deref()
            .ok_or("the scenario has no terrain")?,
    )?;
    let hero = |(hero_type, class): (u8, u8), tile, owner: usize| -> Result<MapObject, String> {
        Ok(MapObject::Hero(Hero {
            tile,
            owner: u8::try_from(owner).unwrap_or(u8::MAX),
            hero_type,
            class,
            name: None,
            experience: Some(0),
            skills: Some(Vec::new()),
            army: Some(army(&battle.stacks[owner], ids)?),
            tight_formation: battle.tight[owner],
            artifacts: Some(HeroArtifacts {
                worn: [None; ARTIFACT_SLOTS],
                backpack: Vec::new(),
            }),
            spells: None,
            primary: Some([1, 1, 1, 1]),
        }))
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
        objects: vec![hero(RED, (tile.0 - 1, tile.1), 0)?, hero(BLUE, tile, 1)?],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    fn ids() -> BTreeMap<String, usize> {
        [("Pikeman", 0), ("Archer", 2), ("Goblin", 84)]
            .into_iter()
            .map(|(n, i)| (n.to_owned(), i))
            .collect()
    }

    fn sides_of(terrain: Option<&str>, attacker: &[(u8, &str, u32)]) -> Sides {
        let stacks = |list: &[(u8, &str, u32)]| {
            list.iter()
                .map(|&(s, n, c)| (s, n.to_owned(), c))
                .collect::<Vec<_>>()
        };
        Sides {
            terrain: terrain.map(str::to_owned),
            battlefield: None,
            stacks: [stacks(attacker), stacks(&[(0, "Goblin", 30)])],
            tight: [false, true],
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
        assert!(
            red.artifacts
                .as_ref()
                .unwrap()
                .worn
                .iter()
                .all(Option::is_none)
        );
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
}
