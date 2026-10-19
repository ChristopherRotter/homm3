//! `h3-view harvest` (E17 M1 T4): generates the asset tables (`{base,ab,sod}.toml`, committed in
//! `clients/assets/`) from the user's `Heroes3.exe` and `H3bitmap.lod` (`cranim.txt`,
//! `crtraits.txt`, `hotraits.txt`), keyed by the committed packs (`packs/<id>/core.toml` and `battle.toml`); by
//! default into `clients/out/harvest` (T8). Afterwards the tables are maintained as source
//! (ADR-0020 point 5); the client never reads the executable. An adapter: it reads files and
//! writes text. The executable's addresses: `docs/exe/map.toml`; what they hold:
//! `docs/epics/E17-design.md` §4 and §5.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::exe::{self, PeImage};
use h3_formats::lod::LodArchive;
use h3_formats::text::TextTable;
use serde::Deserialize;

use crate::gate::find_ci;

/// The tables, one per pack, dependencies first.
const PACKS: [&str; 3] = ["base", "ab", "sod"];

/// The usual obstacle types (`obstacle_types`): 91 records of 0x14 bytes.
const OBSTACLE_TYPES: (u32, usize, usize) = (0x63_C7C8, 91, 0x14);
/// The absolute obstacles (`absolute_obstacles`): 34 records of 0x44 bytes, keys from 100 (VCMI's
/// numbering, which the packs follow).
const ABSOLUTE_OBSTACLES: (u32, usize, usize) = (0x63_BEC0, 34, 0x44);
const ABSOLUTE_BASE: usize = 100;
/// The background tables of 0x4646D0: special fields 1–9, terrains 0–7 by three variants.
const SPECIAL_BACKGROUNDS: u32 = 0x63_D2C8;
const TERRAIN_BACKGROUNDS: u32 = 0x63_D2F0;
const TERRAINS_WITH_BACKGROUNDS: usize = 8;
/// The pictures 0x4646D0 names directly: the shore (special field 0), two heroes in boats, a
/// battle on a ship.
const SHORE: u32 = 0x66_FF40;
const SHIP_TO_SHIP: u32 = 0x66_FF5C;
const SHIP: u32 = 0x66_FF4C;
/// The packs' keys of the special fields 0–9 (manager +0x53C0; `packs/base/battle.toml` and
/// `packs/sod/battle.toml`, E04c T2b; survey `E04c-battlefields.md` §1), whose pictures 0x4646D0
/// names; a battle on a ship (`deck`) and two heroes in boats (`boats`) have pictures of their own.
const SPECIAL_FIELDS: [&str; 10] = [
    "shore",
    "magic_plains",
    "cursed_ground",
    "holy_ground",
    "evil_fog",
    "clover_field",
    "lucid_pools",
    "fiery_fields",
    "rocklands",
    "magic_clouds",
];
/// The shooters' projectile switch in 0x43D9F0: creatures 2–146 index the byte table, whose case
/// selects a handler in the jump table; each handler loads a name (`mov ecx, imm32`).
const MISSILE_INDEX: u32 = 0x43_DE84;
const MISSILE_JUMPS: u32 = 0x43_DE40;
const MISSILE_FIRST: usize = 2;
const MISSILE_COUNT: usize = 145;
/// The shooter's creature flag (the switch runs only for it).
const SHOOTER: u32 = 0x4;
/// The ray shooter's creature flag (`shot_animation` 0x43F51D) and `DrawRay`'s colour tables
/// (R, G, B, from the ray's edge inward) by creature (the switch at 0x43F55B: colour 302 for the
/// Arch Magi, 304 for 74, 75, 120 and 121; any other draws in colour 0, black; E17 design
/// §7.12).
const RAY: u32 = 0x800;
const RAY_COLOURS: [(u32, &[usize]); 2] = [(0x68_833C, &[35]), (0x68_834C, &[74, 75, 120, 121])];
const RAY_ENTRIES: usize = 5;
/// A war machine's flag (`installation.md` §5).
const WAR_MACHINE: u32 = 0x40;
/// The Ballista, whose strike sounds its shot (0x43DA9D).
const BALLISTA: usize = 146;
/// The Master Genies and the Ogre Magi: no shooters, but a shot's sound for their casts
/// (0x43DBCF).
const CASTERS: [usize; 2] = [37, 91];
/// The Devils, Arch Devils, Vampires and Vampire Lords: sounds before and after a move
/// (0x43DC27).
const MOVE_SOUNDS: [usize; 4] = [54, 55, 62, 63];
/// `cranim.txt` as 0x50C810 reads it: from record 2, groups of rows with three records between.
const CRANIM_GROUPS: [usize; 12] = [14, 14, 14, 14, 14, 14, 14, 14, 6, 14, 13, 4];
const CRANIM_CELLS: usize = 24;
/// The DEF groups the battle uses (design §4.2, §4.3).
const GROUPS: &str = "groups = { moving = 0, mouse_over = 1, standing = 2, hit = 3, defend = 4, death = 5, strike = [11, 12, 13], shoot = [14, 15, 16], breath = [17, 18, 19], move_start = 20, move_end = 21, turn_left = [7, 10], turn_right = [9, 8] }";
/// A battle hero's DEF groups (E17 M3 T3a, design §4.5): standing (0, set by `battle_load_heroes`
/// 0x463370), the idle motion (1), defeat (2) and victory (3, `battle_idle_step` 0x4960D0), the
/// cast (4, `cast_spell` 0x5A01D5).
const HERO_GROUPS: &str =
    "hero_groups = { standing = 0, idle = 1, defeat = 2, victory = 3, cast = 4 }";
/// The flags' DEFs `battle_load_heroes` loads (0x4634AB): the left side's, the right side's.
const FLAGS: [u32; 2] = [0x66_FF04, 0x66_FEF8];
/// The heroes' battle pictures (E17 M3 T3a, design §4.5): `battle_load_heroes` takes the hero's
/// record of the static hero table (H3HeroInfo 0x679DD0, 156 records of 0x5C bytes: +0 female,
/// +8 class), its class's town (the class table 0x67D868, records of 0x40 bytes, +0), and loads
/// the DEF named first in record town × 2 + female of 18 records of 0x10 bytes at 0x63BD40.
const HERO_INFO: (u32, usize, usize) = (0x67_9DD0, 156, 0x5C);
const HERO_CLASSES: (u32, usize, usize) = (0x67_D868, 18, 0x40);
const HERO_DEFS: (u32, usize, usize) = (0x63_BD40, 18, 0x10);
/// The magic animations (E17 M3 T3b, design §7.15): `PlayMagicAnimation` 0x496840's table, 83
/// records of 12 bytes: the DEF's name (+0), a force-feedback effect's name (+4, IFC20's; not
/// harvested) and the kind (+8: the low byte where it is drawn, bit 8 the translucent drawer).
const MAGIC: (u32, usize, usize) = (0x64_1E08, 83, 12);
/// The heal's sound `first_aid_tent_action` plays (0x478C46, `Regener.wav`).
const HEAL_SOUND: u32 = 0x66_0A94;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    install: PathBuf,
    packs: PathBuf,
    out: PathBuf,
}

impl Options {
    /// Parses `[--install <dir>] [--packs <dir>] [--out <dir>]`; the defaults are `$H3_INSTALL`,
    /// then the repository's `original-data/install`, the repository's `packs`, `clients/out/harvest`
    /// (never `clients/assets`, whose tables are source edited by hand: a fresh harvest is diffed
    /// against them, *provisional*, E17 M1 T8).
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (mut install, mut packs, mut out) = (None, None, None);
        let mut rest = args.iter();
        while let Some(flag) = rest.next() {
            let slot = match flag.as_str() {
                "--install" => &mut install,
                "--packs" => &mut packs,
                "--out" => &mut out,
                other => return Err(format!("harvest: unknown option `{other}`")),
            };
            let value = rest
                .next()
                .ok_or_else(|| format!("harvest: `{flag}` needs a directory"))?;
            *slot = Some(PathBuf::from(value));
        }
        Ok(Self {
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            out: out.unwrap_or_else(|| manifest.join("../../out/harvest")),
        })
    }
}

/// The parts of a pack the tables are keyed by: from `core.toml` the manifest, creatures, heroes
/// and terrains, from `battle.toml` the special battlefields and the obstacles.
#[derive(Debug, Deserialize)]
struct Pack {
    pack: PackHeader,
    #[serde(default)]
    creature: Vec<Keyed>,
    #[serde(default)]
    hero: Vec<Keyed>,
    #[serde(default)]
    terrain: Vec<Keyed>,
    #[serde(skip)]
    battlefield: Vec<Keyed>,
    #[serde(skip)]
    obstacle: Vec<Keyed>,
}

/// The part of a pack's `battle.toml` the tables are keyed by.
#[derive(Debug, Default, Deserialize)]
struct BattlePart {
    #[serde(default)]
    battlefield: Vec<Keyed>,
    #[serde(default)]
    obstacle: Vec<Keyed>,
}

#[derive(Debug, Deserialize)]
struct PackHeader {
    id: String,
    depends: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Keyed {
    id: String,
    #[serde(default)]
    name: Option<String>,
}

/// One creature's `cranim.txt` row as the executable keeps it (0x50CA00).
struct Cranim {
    times: [u32; 4],
    offsets: [[i16; 2]; 3],
    angles: [i16; 12],
    troop_count_offset: i32,
    attack_climax: u32,
}

/// What the executable and the text tables give, by original id or index.
struct Harvest {
    defs: Vec<String>,
    sounds: Vec<String>,
    missiles: BTreeMap<usize, String>,
    rays: BTreeMap<usize, Vec<[u8; 3]>>,
    cranim: Vec<Cranim>,
    names: BTreeMap<String, usize>,
    /// The heroes' battle DEFs and `hotraits.txt`'s names, by hero type.
    hero_defs: Vec<String>,
    hero_names: Vec<String>,
    flags: [String; 2],
    /// The magic animations: DEF, place, translucent.
    magic: Vec<(String, u8, bool)>,
    heal_sound: String,
    obstacles: Vec<(String, u8, u8)>,
    absolute: Vec<(String, i16, i16)>,
    terrains: Vec<[String; 3]>,
    specials: Vec<String>,
    ship_to_ship: String,
    ship: String,
}

pub fn run(options: &Options) -> ExitCode {
    match harvest(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("harvest: {message}");
            ExitCode::FAILURE
        }
    }
}

fn harvest(options: &Options) -> Result<(), String> {
    let exe_path = find_ci(&options.install, "Heroes3.exe")
        .map_err(|e| format!("{}: Heroes3.exe: {e}", options.install.display()))?;
    let exe_bytes = fs::read(&exe_path).map_err(|e| format!("{}: {e}", exe_path.display()))?;
    let data = find_ci(&options.install, "Data").map_err(|e| format!("Data: {e}"))?;
    let lod_path = find_ci(&data, "H3bitmap.lod").map_err(|e| format!("H3bitmap.lod: {e}"))?;
    let lod_bytes = fs::read(&lod_path).map_err(|e| format!("{}: {e}", lod_path.display()))?;
    let lod = LodArchive::parse(&lod_bytes).map_err(|e| format!("H3bitmap.lod: {e:?}"))?;
    let text = |name: &str| -> Result<TextTable, String> {
        let bytes = lod
            .read_by_name(name)
            .map_err(|e| format!("{name}: {e:?}"))?
            .ok_or_else(|| format!("H3bitmap.lod has no {name}"))?;
        TextTable::parse(&bytes).map_err(|e| format!("{name}: {e:?}"))
    };
    let h = read_harvest(
        &exe_bytes,
        &text("CRANIM.TXT")?,
        &text("CRTRAITS.TXT")?,
        &text("HOTRAITS.TXT")?,
    )?;
    let mut gaps = Vec::new();
    fs::create_dir_all(&options.out).map_err(|e| format!("{}: {e}", options.out.display()))?;
    let mut packs = Vec::new();
    for id in PACKS {
        let path = options.packs.join(id).join("core.toml");
        let source = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut pack: Pack =
            toml::from_str(&source).map_err(|e| format!("{}: {e}", path.display()))?;
        // A pack folder without battle.toml reads as an empty one (packs/README.md).
        let battle_path = options.packs.join(id).join("battle.toml");
        if battle_path.exists() {
            let source = fs::read_to_string(&battle_path)
                .map_err(|e| format!("{}: {e}", battle_path.display()))?;
            let battle: BattlePart =
                toml::from_str(&source).map_err(|e| format!("{}: {e}", battle_path.display()))?;
            pack.battlefield = battle.battlefield;
            pack.obstacle = battle.obstacle;
        }
        if pack.pack.id != id {
            return Err(format!(
                "{}: the pack's id is `{}`",
                path.display(),
                pack.pack.id
            ));
        }
        packs.push(pack);
    }
    // The obstacle types some pack defines; base keeps the others (none since E04c T2b).
    let mut defined = Vec::new();
    for pack in &packs {
        for o in &pack.obstacle {
            defined.push(obstacle_number(&pack.pack.id, &o.id)?);
        }
    }
    // The packs' heroes in order are the hero types (as `h3-harvest` writes them), each checked
    // against hotraits.txt's name.
    let mut hero_type = 0;
    for pack in &packs {
        let id = &pack.pack.id;
        let table = write_table(pack, &h, &defined, &mut hero_type, &mut gaps)?;
        let out = options.out.join(format!("{id}.toml"));
        fs::write(&out, table).map_err(|e| format!("{}: {e}", out.display()))?;
        println!("harvest: {}", out.display());
    }
    for gap in &gaps {
        println!("harvest: gap: {gap}");
    }
    Ok(())
}

fn read_harvest(
    exe: &[u8],
    cranim: &TextTable,
    crtraits: &TextTable,
    hotraits: &TextTable,
) -> Result<Harvest, String> {
    let (_, image) = exe::identify(exe).map_err(|e| format!("Heroes3.exe: {e:?}"))?;
    let creatures = exe::creature_table(exe).map_err(|e| format!("creatures: {e:?}"))?;
    let string = |va: u32| image.c_string(va).map_err(|e| format!("0x{va:X}: {e:?}"));
    let mut missiles = BTreeMap::new();
    for c in creatures.iter().filter(|c| c.flags & SHOOTER != 0) {
        let id = usize::from(c.id);
        if let Some(name) = missile(&image, exe, id)? {
            missiles.insert(id, name);
        }
    }
    let mut rays = BTreeMap::new();
    for c in creatures.iter().filter(|c| c.flags & RAY != 0) {
        let id = usize::from(c.id);
        let colours = match RAY_COLOURS.iter().find(|(_, ids)| ids.contains(&id)) {
            Some(&(table, _)) => bytes(&image, exe, table, 3 * RAY_ENTRIES)?
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect(),
            None => vec![[0; 3]],
        };
        rays.insert(id, colours);
    }
    let mut obstacles = Vec::new();
    for i in 0..OBSTACLE_TYPES.1 {
        let at = OBSTACLE_TYPES.0 + to_u32(i * OBSTACLE_TYPES.2);
        let record = bytes(&image, exe, at, OBSTACLE_TYPES.2)?;
        obstacles.push((string(u32_at(&record, 0x10))?, record[4], record[7]));
    }
    let mut absolute = Vec::new();
    for j in 0..ABSOLUTE_OBSTACLES.1 {
        let at = ABSOLUTE_OBSTACLES.0 + to_u32(j * ABSOLUTE_OBSTACLES.2);
        let record = bytes(&image, exe, at, ABSOLUTE_OBSTACLES.2)?;
        let x = i16::try_from(i32::from_le_bytes([
            record[4], record[5], record[6], record[7],
        ]));
        let y = i16::try_from(i32::from_le_bytes([
            record[8], record[9], record[10], record[11],
        ]));
        let (x, y) = x
            .ok()
            .zip(y.ok())
            .ok_or(format!("absolute obstacle {j}: position"))?;
        absolute.push((string(u32_at(&record, 0x40))?, x, y));
    }
    let mut terrains = Vec::new();
    for t in 0..TERRAINS_WITH_BACKGROUNDS {
        let row = bytes(&image, exe, TERRAIN_BACKGROUNDS + to_u32(t * 12), 12)?;
        terrains.push([
            string(u32_at(&row, 0))?,
            string(u32_at(&row, 4))?,
            string(u32_at(&row, 8))?,
        ]);
    }
    let mut battle_defs = Vec::new();
    for k in 0..HERO_DEFS.1 {
        let record = bytes(&image, exe, HERO_DEFS.0 + to_u32(k * HERO_DEFS.2), 4)?;
        battle_defs.push(string(u32_at(&record, 0))?);
    }
    let mut hero_defs = Vec::new();
    for id in 0..HERO_INFO.1 {
        let info = bytes(&image, exe, HERO_INFO.0 + to_u32(id * HERO_INFO.2), 12)?;
        let (female, class) = (u32_at(&info, 0), u32_at(&info, 8) as usize);
        if female > 1 || class >= HERO_CLASSES.1 {
            return Err(format!("hero {id}: female {female}, class {class}"));
        }
        let town = u32_at(
            &bytes(
                &image,
                exe,
                HERO_CLASSES.0 + to_u32(class * HERO_CLASSES.2),
                4,
            )?,
            0,
        ) as usize;
        let def = battle_defs
            .get(town * 2 + female as usize)
            .ok_or(format!("hero {id}: class {class}'s town {town}"))?;
        hero_defs.push(def.clone());
    }
    let mut magic = Vec::new();
    for k in 0..MAGIC.1 {
        let record = bytes(&image, exe, MAGIC.0 + to_u32(k * MAGIC.2), MAGIC.2)?;
        let kind = u32_at(&record, 8);
        magic.push((string(u32_at(&record, 0))?, kind as u8, kind & 0x100 != 0));
    }
    let heal_sound = string(HEAL_SOUND)?;
    let heal_sound = heal_sound
        .split_once('.')
        .map_or(heal_sound.as_str(), |(stem, _)| stem)
        .to_owned();
    let mut specials = vec![string(SHORE)?];
    let row = bytes(&image, exe, SPECIAL_BACKGROUNDS + 4, 4 * 9)?;
    for k in 0..9 {
        specials.push(string(u32_at(&row, 4 * k))?);
    }
    Ok(Harvest {
        defs: creatures.iter().map(|c| c.def_name.clone()).collect(),
        sounds: creatures
            .iter()
            .map(|c| sounds(usize::from(c.id), &c.sound, c.flags))
            .collect(),
        missiles,
        rays,
        cranim: cranim_rows(cranim)?,
        names: creature_names(crtraits)?,
        hero_defs,
        hero_names: hero_names(hotraits),
        flags: [string(FLAGS[0])?, string(FLAGS[1])?],
        magic,
        heal_sound,
        obstacles,
        absolute,
        terrains,
        specials,
        ship_to_ship: string(SHIP_TO_SHIP)?,
        ship: string(SHIP)?,
    })
}

/// A creature's sounds as `stack_load_graphics` loads them (0x43D9F0, design §8.2): its prefix
/// and a suffix each, lower-cased. A war machine has no move sound and sounds its wince when it
/// strikes or defends (the Ballista its shot when it strikes); only shooters and [`CASTERS`] have
/// a shot's; only [`MOVE_SOUNDS`] a move's start and end. The fields of an inline TOML table.
fn sounds(id: usize, prefix: &str, flags: u32) -> String {
    let machine = flags & WAR_MACHINE != 0;
    let attack = if id == BALLISTA {
        "shot"
    } else if machine {
        "wnce"
    } else {
        "attk"
    };
    let shot = flags & SHOOTER != 0 || CASTERS.contains(&id);
    let moves = MOVE_SOUNDS.contains(&id);
    let fields = [
        ("move", (!machine).then_some("move")),
        ("attack", Some(attack)),
        ("wince", Some("wnce")),
        ("shot", shot.then_some("shot")),
        ("death", Some("kill")),
        ("defend", Some(if machine { "wnce" } else { "dfnd" })),
        ("start", moves.then_some("ext1")),
        ("end", moves.then_some("ext2")),
    ];
    fields
        .iter()
        .filter_map(|(field, suffix)| {
            suffix.map(|s| format!("{field} = \"{}{s}\"", prefix.to_ascii_lowercase()))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The projectile a shooter's case loads, none for the default case.
fn missile(image: &PeImage<'_>, exe: &[u8], id: usize) -> Result<Option<String>, String> {
    if !(MISSILE_FIRST..MISSILE_FIRST + MISSILE_COUNT).contains(&id) {
        return Ok(None);
    }
    let case = bytes(image, exe, MISSILE_INDEX + to_u32(id - MISSILE_FIRST), 1)?[0];
    let handler = u32_at(
        &bytes(image, exe, MISSILE_JUMPS + 4 * u32::from(case), 4)?,
        0,
    );
    let code = bytes(image, exe, handler, 5)?;
    if code[0] != 0xB9 {
        return Ok(None);
    }
    let va = u32_at(&code, 1);
    image
        .c_string(va)
        .map(Some)
        .map_err(|e| format!("creature {id}'s projectile 0x{va:X}: {e:?}"))
}

/// `cranim.txt`'s rows by creature id, read and scaled as 0x50CA00 does.
fn cranim_rows(table: &TextTable) -> Result<Vec<Cranim>, String> {
    let mut rows = Vec::new();
    let mut record = 2;
    for group in CRANIM_GROUPS {
        for _ in 0..group {
            let id = rows.len();
            let cells = table
                .records
                .get(record)
                .filter(|r| r.len() > CRANIM_CELLS)
                .ok_or(format!("cranim.txt record {}: too short", record + 1))?;
            let real = |k: usize| -> Result<f64, String> {
                cells[k]
                    .trim()
                    .parse()
                    .map_err(|_| format!("cranim.txt creature {id}, cell {k}"))
            };
            let int = |k: usize| -> Result<i32, String> {
                cells[k]
                    .trim()
                    .parse()
                    .map_err(|_| format!("cranim.txt creature {id}, cell {k}"))
            };
            // atof, a 53-bit x87 product, truncated (0x50CA00; installation.md §7.10).
            let scaled = |k: usize, by: f64| -> Result<u32, String> {
                let v = (real(k)? * by).trunc();
                if (0.0..=f64::from(u32::MAX)).contains(&v) {
                    // In range and whole: the conversion is exact.
                    Ok(v as u32)
                } else {
                    Err(format!("cranim.txt creature {id}, cell {k}: out of range"))
                }
            };
            let small = |k: usize| -> Result<i16, String> {
                i16::try_from(int(k)?).map_err(|_| format!("cranim.txt creature {id}, cell {k}"))
            };
            let mut angles = [0i16; 12];
            for (a, slot) in angles.iter_mut().enumerate() {
                let v = real(10 + a)?;
                if v.fract() != 0.0 || !(-180.0..=180.0).contains(&v) {
                    return Err(format!(
                        "cranim.txt creature {id}: angle `{}`",
                        cells[10 + a]
                    ));
                }
                // Whole and in range: the conversion is exact.
                *slot = v as i16;
            }
            rows.push(Cranim {
                times: [
                    scaled(0, 9000.0)?,
                    scaled(1, 500.0)?,
                    scaled(2, 500.0)?,
                    scaled(3, 115.0)?,
                ],
                offsets: [
                    [small(4)?, small(5)?],
                    [small(6)?, small(7)?],
                    [small(8)?, small(9)?],
                ],
                angles,
                troop_count_offset: int(22)?,
                attack_climax: u32::try_from(int(23)?)
                    .map_err(|_| format!("cranim.txt creature {id}, cell 23"))?,
            });
            record += 1;
        }
        record += 3;
    }
    Ok(rows)
}

/// `crtraits.txt`'s singular names by creature id (from record 2, blank records skipped, as
/// `h3-harvest` reads it for the packs).
pub fn creature_names(table: &TextTable) -> Result<BTreeMap<String, usize>, String> {
    let mut names = BTreeMap::new();
    let rows = table
        .records
        .iter()
        .skip(2)
        .filter(|r| !TextTable::is_blank(r));
    for (id, row) in rows.enumerate() {
        let name = row.first().map_or("", |n| n.trim()).to_owned();
        if names.insert(name.clone(), id).is_some() {
            return Err(format!("crtraits.txt: the name `{name}` twice"));
        }
    }
    Ok(names)
}

/// `hotraits.txt`'s names by hero type (from record 2, blank records skipped, as `h3-harvest`
/// reads it for the packs). Not unique: two heroes are called Lord Haart.
pub fn hero_names(table: &TextTable) -> Vec<String> {
    table
        .records
        .iter()
        .skip(2)
        .filter(|r| !TextTable::is_blank(r))
        .map(|row| row.first().map_or("", |n| n.trim()).to_owned())
        .collect()
}

/// The type number of an obstacle key `obstacle_<n>` of pack `id`.
fn obstacle_number(id: &str, key: &str) -> Result<usize, String> {
    key.strip_prefix("obstacle_")
        .and_then(|n| n.parse().ok())
        .ok_or(format!("{id}:{key}: not an obstacle_<n> key"))
}

fn write_table(
    pack: &Pack,
    h: &Harvest,
    defined: &[usize],
    hero_type: &mut usize,
    gaps: &mut Vec<String>,
) -> Result<String, String> {
    let id = &pack.pack.id;
    let mut t = String::new();
    let _ = writeln!(
        t,
        "# Asset tables of the pack `{id}` (E17 M1 T4; ADR-0020 points 5 and 6; the format:\n\
         # clients/README.md, \"The asset tables\"). GENERATED by `h3-view harvest` from the user's\n\
         # Heroes3.exe and H3bitmap.lod (cranim.txt, crtraits.txt, hotraits.txt), keyed by the pack's\n\
         # core.toml and battle.toml (packs/{id}/); maintained as source afterwards. Identifiers and\n\
         # numbers only (ADR-0009).\n\n\
         [table]\nid = \"{id}\"\ndepends = [{}]",
        list(&pack.pack.depends)
    );
    if id == "base" {
        let _ = writeln!(
            t,
            "\n[battle]\n{GROUPS}\n{HERO_GROUPS}\nflags = [{}]",
            list(&h.flags)
        );
        // The magic animations by number (E17 M3 T3b), one a line.
        let _ = writeln!(t, "magic = [");
        for (n, (def, place, translucent)) in h.magic.iter().enumerate() {
            let shown = if *translucent {
                ", translucent = true"
            } else {
                ""
            };
            let _ = writeln!(
                t,
                "    {{ def = \"{}\", place = {place}{shown} }},  # {n}",
                plain(def)?
            );
        }
        let _ = writeln!(t, "]\nheal_sound = \"{}\"", plain(&h.heal_sound)?);
    }
    for c in &pack.creature {
        let name = c.name.as_deref().unwrap_or_default();
        let Some(&original) = h.names.get(name) else {
            gaps.push(format!("{id}:{}: `{name}` is not in crtraits.txt", c.id));
            continue;
        };
        let (Some(def), Some(anim)) = (h.defs.get(original), h.cranim.get(original)) else {
            gaps.push(format!("{id}:{}: no DEF or cranim.txt row", c.id));
            continue;
        };
        let _ = write!(
            t,
            "\n[[creature]]\nkey = \"{}\"\noriginal = {original}\ndef = \"{}\"\ntimes = [{}]\n",
            plain(&c.id)?,
            plain(def)?,
            numbers(&anim.times)
        );
        if anim.troop_count_offset != 0 {
            let _ = writeln!(t, "troop_count_offset = {}", anim.troop_count_offset);
        }
        if anim.attack_climax != 0 {
            let _ = writeln!(t, "attack_climax = {}", anim.attack_climax);
        }
        if let Some(m) = h.missiles.get(&original) {
            let _ = writeln!(t, "missile = \"{}\"", plain(m)?);
        }
        if anim.offsets != [[0; 2]; 3] {
            let o = anim.offsets;
            let _ = writeln!(
                t,
                "missile_offsets = [[{}], [{}], [{}]]",
                numbers(&o[0]),
                numbers(&o[1]),
                numbers(&o[2])
            );
        }
        if anim.angles != [0; 12] {
            let _ = writeln!(t, "missile_angles = [{}]", numbers(&anim.angles));
        }
        if let Some(colours) = h.rays.get(&original) {
            let colours: Vec<String> = colours
                .iter()
                .map(|c| format!("[{}]", numbers(c)))
                .collect();
            let _ = writeln!(t, "ray = [{}]", colours.join(", "));
        }
        if let Some(sounds) = h.sounds.get(original) {
            let _ = writeln!(t, "sounds = {{ {sounds} }}");
        }
    }
    for hero in &pack.hero {
        let (n, name) = (*hero_type, hero.name.as_deref().unwrap_or_default());
        *hero_type += 1;
        let def = h
            .hero_defs
            .get(n)
            .filter(|_| h.hero_names.get(n).map(String::as_str) == Some(name));
        let Some(def) = def else {
            gaps.push(format!(
                "{id}:{}: `{name}` is not hero type {n} of hotraits.txt",
                hero.id
            ));
            continue;
        };
        let _ = write!(
            t,
            "\n[[hero]]\nkey = \"{}\"  # original id {n}\ndef = \"{}\"\n",
            plain(&hero.id)?,
            plain(def)?
        );
    }
    for (index, terrain) in pack.terrain.iter().enumerate() {
        match h.terrains.get(index) {
            Some(names) => {
                let _ = write!(
                    t,
                    "\n[[battlefield]]\nkey = \"{}\"\nbackgrounds = [{}]\n",
                    plain(&terrain.id)?,
                    list(names)
                );
            }
            None => gaps.push(format!("{id}:{}: no background (0x4646D0)", terrain.id)),
        }
    }
    for field in &pack.battlefield {
        let picture = match field.id.as_str() {
            "deck" => Some(&h.ship),
            "boats" => Some(&h.ship_to_ship),
            key => SPECIAL_FIELDS
                .iter()
                .position(|k| *k == key)
                .and_then(|k| h.specials.get(k)),
        };
        match picture {
            Some(name) => {
                let _ = write!(
                    t,
                    "\n[[battlefield]]\nkey = \"{}\"\nbackgrounds = [\"{}\"]\n",
                    plain(&field.id)?,
                    plain(name)?
                );
            }
            None => gaps.push(format!("{id}:{}: no picture (0x4646D0)", field.id)),
        }
    }
    let mut keys: Vec<(usize, &str)> = Vec::new();
    for o in &pack.obstacle {
        keys.push((obstacle_number(id, &o.id)?, "pack"));
    }
    if id == "base" {
        // The obstacle types no pack defines, kept with base.
        let all = (0..h.obstacles.len()).chain(ABSOLUTE_BASE..ABSOLUTE_BASE + h.absolute.len());
        for n in all.filter(|n| !defined.contains(n)) {
            keys.push((n, "no pack yet"));
        }
        keys.sort_unstable();
    }
    for (n, origin) in keys {
        let comment = if origin == "pack" {
            ""
        } else {
            "  # not in a pack yet"
        };
        if let Some((def, rows, layer)) = h.obstacles.get(n) {
            let _ = write!(
                t,
                "\n[[obstacle]]\nkey = \"obstacle_{n}\"{comment}\ndef = \"{}\"\nrows = {rows}\nlayer = {layer}\n",
                plain(def)?
            );
        } else if let Some((pcx, x, y)) =
            n.checked_sub(ABSOLUTE_BASE).and_then(|j| h.absolute.get(j))
        {
            let _ = write!(
                t,
                "\n[[obstacle]]\nkey = \"obstacle_{n}\"{comment}\npcx = \"{}\"\nat = [{x}, {y}]\n",
                plain(pcx)?
            );
        } else {
            gaps.push(format!(
                "{id}:obstacle_{n}: no obstacle type in the executable"
            ));
        }
    }
    Ok(t)
}

/// A name or key as a TOML basic string's content: printable ASCII without quotes or escapes.
fn plain(s: &str) -> Result<&str, String> {
    if !s.is_empty()
        && s.bytes()
            .all(|b| (0x20..0x7F).contains(&b) && b != b'"' && b != b'\\')
    {
        Ok(s)
    } else {
        Err(format!("`{s}` is not a plain name"))
    }
}

fn list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn numbers<T: std::fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(T::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn bytes(image: &PeImage<'_>, exe: &[u8], va: u32, len: usize) -> Result<Vec<u8>, String> {
    let start = image
        .file_offset(va)
        .ok_or(format!("0x{va:X} is not in the executable's file"))?;
    exe.get(start..start + len)
        .map(<[u8]>::to_vec)
        .ok_or(format!("0x{va:X}: {len} bytes past the end"))
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
