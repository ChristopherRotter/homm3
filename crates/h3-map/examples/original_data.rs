//! `h3-map`'s check against the original data (`cargo xtask verify --with-original-data`): reads
//! every map in `<install>/Maps`, imports it with the original packs (RoE to SoD) and validates
//! the setup. Fails when the reader rejects a map, when a map's static layer does not import and
//! validate, or when a map that imports fails validation; a map the current increment cannot
//! represent is counted by its reason (`Unsupported`). Prints what the input bounds were set from
//! (`docs/formats/h3m.md`, "Input bounds"), the import's counts ("Import") and the maps with a
//! tile that is the visitable tile of two or more objects (E08 T14a: which object a step there
//! visits, ADV-OBJ-003) and the typed objects of one type on one anchor (E08 T15a: what a player
//! sees names a typed object by its type and anchor, ADR-0016 point 6); since E08 T20a2a the
//! monsters' and the special grounds' counts, since E08 T21a the guarded piles' and since E08 T29
//! the level limits, the banned skills and the levels at load of the maps that import whole
//! (`docs/formats/h3m.md`, "Import"), since E08 T42 the Tutorial's hero's day's points
//! (checked), since E08 T62b2 the events: their flags and the tiles they lie on, and since E08 T72
//! the sea: the boats by type, the sea's pickups by type, the holes and Favorable Winds (a hero on
//! water is counted by its reason, `Unsupported::HeroOnWater`; a boat the setup refuses fails the
//! static layer's validation), and since E08 T83a increment 9's objects (their owners at load, the
//! abandoned mines' sets) and the timed events; the static layer is validated in a game of the
//! map's players (an owner must be one); since E08 T93b the dwellings (17, 20), the Refugee
//! Camps, the random dwellings typed by class and those tied to a town (left blockers), and the
//! RoE maps' dwellings under the base pack alone (checked: every RoE map's static layer imports
//! with it, each of its dwellings typed), and since E08 T94a the dwellings the game's build gives
//! (the random ones resolved, checked) and the maps it refuses for a random dwelling's tiles, and
//! since E08 T94b the weeks it gives: the dwellings' counts and guards and the Refugee Camps'
//! creatures and counts (checked).
//! Since E09 T7a1 the towns, the players' starts, the random heroes, the heroes moved onto a
//! town's entrance (TWN-SET-010) and the pool's inputs of the maps that import, and a setup the
//! build refuses for a hero to draw (`TWN-STUB-3`) is counted as an outcome; since E09 T7b1 the
//! random heroes are drawn and only a main town's hero is refused so. Since E09 T8a1 the towns are
//! typed (their owners at load and formations counted), and the static layer's game gives one
//! town record per town (a random town of no alignment).
//!
//! `cargo run -p h3-map --example original_data -- original-data/install`
//!
//! The installation copy is only read. Without a `Maps` folder the check is skipped with a note.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

use std::sync::Arc;

use h3_adventure::{
    DWELLING_LEVELS, Holds, MapSetup, ObjectKind, ObjectTypeId, Pos, SetupError, land_table_points,
};
use h3_content::{CreatureTypeId, Formation};
use h3_game::{
    BuildingsSetup, DEFAULT_DIFFICULTY, FactionChoice, GameRuleset, GameSetup, GameSetupError,
    GameState, GarrisonCreature, HeroDraw, PlayerSetup, TownFaction, TownSetup,
};
use h3_map::model::{Artifacts, Body, Object, Player, RawMap, SeerTask, Text, Version};
use h3_map::{ImportError, Limits, gunzip, import, import_map};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};

/// The largest value of each bounded quantity over all maps, with the map it came from; how many
/// objects of each body kind the maps hold; how many tiles objects are visited from; each map
/// with stacked visitable tiles, with the classes on each such tile in map order; each map with
/// typed objects that share their type and anchor with an earlier one, with how many; the
/// monsters' and grounds' counts (E08 T20a2a); the dwellings the game's build gives (E08 T94a);
/// the dwellings' and the Refugee Camps' weeks it gives (E08 T94b).
#[derive(Default)]
struct Peaks(
    BTreeMap<&'static str, (usize, String)>,
    BTreeMap<&'static str, usize>,
    usize,
    BTreeMap<String, Vec<Vec<u32>>>,
    BTreeMap<String, usize>,
    BTreeMap<&'static str, usize>,
    Built,
    Weeks,
);

/// The census of the dwellings the game's build gives (E08 T94a, [`built_dwellings`]): its
/// counts, and the resolved random dwellings by faction (the core's index) and level.
#[derive(Default)]
struct Built(
    BTreeMap<&'static str, usize>,
    BTreeMap<usize, [usize; DWELLING_LEVELS]>,
);

/// The census of the dwellings' and the Refugee Camps' weeks the game's build gives (E08 T94b,
/// [`built_weeks`]): its counts, and the camps' draws by the drawn creature's level.
#[derive(Default)]
struct Weeks(BTreeMap<&'static str, usize>, [usize; DWELLING_LEVELS]);

/// A dwelling's guards of a creature of the part's guard level or more are this times its growth
/// (ADV-DWL-002; E08 T94b's census, [`built_weeks`]): the original's 3, checked against the pack.
const GUARD_FACTOR: u32 = 3;

/// The seed every game of the check is built with: the census of what the build draws (E08
/// T94a, [`built_dwellings`]) is that seed's.
const SEED: u64 = 0;

/// The maps refused for a hero on a tile a random dwelling could cover (E08 T94a,
/// `SetupError::RandomDwellingTile`).
const REFUSED_HERO: &str = "maps refused for a hero on a random dwelling's tile";

/// The maps refused for a boat on a tile a random dwelling could cover (E08 T94a,
/// `SetupError::Boat`, which the setup without its random dwellings does not give).
const REFUSED_BOAT: &str = "maps refused for a boat on a random dwelling's tile";

impl Peaks {
    fn note(&mut self, what: &'static str, value: usize, map: &str) {
        let entry = self.0.entry(what).or_insert((0, String::new()));
        if value > entry.0 {
            *entry = (value, map.to_owned());
        }
    }
}

fn main() -> ExitCode {
    let Some(install) = std::env::args().nth(1) else {
        eprintln!("usage: original_data <installation copy>");
        return ExitCode::FAILURE;
    };
    let dir = Path::new(&install).join("Maps");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        println!("h3-map: skipped ({} is not there)", dir.display());
        return ExitCode::SUCCESS;
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("h3m") || e.eq_ignore_ascii_case("tut"))
        })
        .collect();
    files.sort();
    let (rules, base) = match (ruleset(ORIGINAL_PACKS.len()), ruleset(1)) {
        (Ok(rules), Ok(base)) => (rules, base),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("h3-map: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut imports = BTreeMap::<String, Vec<String>>::new();
    let limits = Limits::default();
    let mut versions = BTreeMap::<Version, usize>::new();
    let mut peaks = Peaks::default();
    let mut failures = 0;
    for path in &files {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        match check(path, &limits, &mut peaks, &name, (&rules, &base)) {
            Ok((version, outcome)) => {
                *versions.entry(version).or_default() += 1;
                imports.entry(outcome).or_default().push(name);
            }
            Err(e) => {
                println!("h3-map: {name}: {e}");
                failures += 1;
            }
        }
    }
    println!(
        "h3-map: {} of {} maps read ({versions:?})",
        files.len() - failures,
        files.len()
    );
    for (what, (value, map)) in &peaks.0 {
        println!("  largest {what}: {value} ({map})");
    }
    let kinds: Vec<String> = peaks.1.iter().map(|(k, n)| format!("{k} {n}")).collect();
    println!("  objects by body: {}", kinds.join(", "));
    let tiles: usize = peaks.3.values().map(Vec::len).sum();
    println!(
        "  maps with a tile visited from two or more objects: {} ({tiles} of {} visitable tiles)",
        peaks.3.len(),
        peaks.2
    );
    let mut stacks = BTreeMap::<&[u32], usize>::new();
    for classes in peaks.3.values().flatten() {
        *stacks.entry(classes).or_default() += 1;
    }
    for (classes, n) in &stacks {
        println!("    classes {classes:?} (map order): {n}");
    }
    for (map, tiles) in &peaks.3 {
        println!("    {map}: {}", tiles.len());
    }
    println!(
        "  typed objects of one type on one anchor (each after the first): {} in {} maps",
        peaks.4.values().sum::<usize>(),
        peaks.4.len()
    );
    for (map, twins) in &peaks.4 {
        println!("    {map}: {twins}");
    }
    for (what, n) in &peaks.5 {
        println!("  {what}: {n}");
    }
    let built = &mut peaks.6;
    for what in [REFUSED_HERO, REFUSED_BOAT] {
        built.0.entry(what).or_default();
    }
    println!("  dwellings built (seed {SEED}), checked:");
    for (what, n) in &built.0 {
        println!("    {what}: {n}");
    }
    let spread: Vec<String> = built
        .1
        .iter()
        .map(|(&faction, levels)| {
            let key = rules
                .core()
                .factions()
                .get(faction)
                .map_or("?", |f| f.key.as_str());
            format!("{key} {levels:?}")
        })
        .collect();
    println!(
        "    resolved by faction drawn, levels 1 to {DWELLING_LEVELS} (seed {SEED}): {}",
        spread.join(", ")
    );
    let weeks = &peaks.7;
    println!("  the dwellings' and the Refugee Camps' weeks built (seed {SEED}), checked:");
    for (what, n) in &weeks.0 {
        println!("    {what}: {n}");
    }
    println!(
        "    the camps' draws by creature level, 1 to {DWELLING_LEVELS} (seed {SEED}): {:?}",
        weeks.1
    );
    println!("h3-map: every static layer imports and validates; the import by outcome:");
    for (outcome, maps) in &imports {
        println!("  {outcome}: {}", maps.len());
        if outcome == IMPORTED {
            println!("    {}", maps.join(", "));
        }
    }
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The outcome of a map that imports and validates.
const IMPORTED: &str = "imported and valid";

/// The game's ruleset of the first `n` original packs, RoE to SoD: all three for the import, the
/// base pack alone for the RoE maps' check (E08 T93b, [`roe`]).
fn ruleset(n: usize) -> Result<GameRuleset, String> {
    let (core, part) = compile(&ORIGINAL_PACKS[..n], "base:faithful")
        .map_err(|e| format!("the original packs do not compile: {e:?}"))?;
    let packs = [
        h3_pack_battle::BASE_PACK,
        h3_pack_battle::AB_PACK,
        h3_pack_battle::SOD_PACK,
    ];
    let battle = h3_pack_battle::compile(&packs[..n], "base:faithful")
        .map_err(|e| format!("the original battle packs do not compile: {e:?}"))?;
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS[..n], "base:faithful")
        .map_err(|e| format!("the original town packs do not compile: {e:?}"))?;
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .map_err(|_| "the parts belong to another core".to_owned())
}

/// Reads one map and checks that its terrain codes are in range (a misaligned read shows there),
/// then imports it: its static layer must import and validate; the whole map imports and
/// validates, or the import reports what is not supported yet (the outcome). Since E08 T93b an RoE
/// map's static layer must also import with the base pack alone (`base`, [`roe`]).
fn check(
    path: &Path,
    limits: &Limits,
    peaks: &mut Peaks,
    name: &str,
    (rules, base): (&GameRuleset, &GameRuleset),
) -> Result<(Version, String), String> {
    let file = std::fs::read(path).map_err(|e| e.to_string())?;
    peaks.note("file", file.len(), name);
    let inflated = gunzip(&file, limits.max_map_bytes).map_err(|e| e.to_string())?;
    peaks.note("inflated map", inflated.len(), name);
    let map = h3_map::read(&file, limits).map_err(|e| e.to_string())?;
    if let Some((i, t)) = map
        .terrain
        .tiles
        .iter()
        .enumerate()
        .find(|(_, t)| t.terrain > 9 || t.river > 4 || t.road > 3)
    {
        return Err(format!("tile {i} has codes out of range: {t:?}"));
    }
    note(&map, peaks, name);
    let static_map = import_map(&map, rules).map_err(|e| format!("static layer: {e}"))?;
    // The static layer alone, in a game of the map's players without heroes: its objects'
    // contents are drawn as a game's are (E08 T12b); since E08 T83a its owners are those players;
    // since E09 T8a1 one town record per town (`GameSetupError::TownCount`), a random town of no
    // alignment, which no player's start reads.
    let town_records = static_map
        .objects
        .iter()
        .filter(|object| {
            object
                .object_type
                .and_then(|id| rules.adventure().object_type(id))
                .is_some_and(|ty| ty.kind == ObjectKind::Town)
        })
        .map(|_| TownSetup {
            faction: TownFaction::Random { alignment: None },
            buildings: BuildingsSetup::Standard { fort: false },
            garrison: None,
            name: None,
            events: Vec::new(),
        })
        .collect();
    let players = map
        .players
        .iter()
        .zip(0u8..)
        .filter(|(player, _)| player.human || player.computer)
        .map(|(player, colour)| PlayerSetup {
            colour,
            human: player.human,
            start: Default::default(),
        })
        .collect();
    let setup = GameSetup {
        players,
        difficulty: DEFAULT_DIFFICULTY,
        map: static_map,
        heroes: Vec::new(),
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_spells: Vec::new(),
        banned_artifacts: Vec::new(),
        events: Vec::new(),
        towns: town_records,
        allowed_heroes: Vec::new(),
        hero_masks: Vec::new(),
        prisoners: Vec::new(),
    };
    let built = GameState::new(rules, &setup, SEED)
        .map_err(|e| format!("static layer: {}", refusal(rules, &setup, e, &mut peaks.6)))?;
    // E08 T83b1: the state the game builds (ADV-SHY-001, ADV-MIN-003, ADV-OWN-002).
    for shipyard in built_economy(&built, rules, name, &mut peaks.5) {
        println!("h3-map: a Shipyard without a boat tile: {shipyard}");
    }
    // E08 T94a: the dwellings it builds, the random ones resolved (ADV-DWL-005).
    for dwelling in built_dwellings(&built, &map, &setup.map, rules, name, &mut peaks.6)? {
        println!("h3-map: a resolved dwelling's anchor past the right edge: {dwelling}");
    }
    // E08 T94b: the dwellings' and the camps' weeks it builds (ADV-DWL-002, ADV-RFG-001).
    built_weeks(&built, rules, name, &mut peaks.7)?;
    let (visitable, stacked) = stacked(&map, &setup.map);
    peaks.2 += visitable;
    if !stacked.is_empty() {
        peaks.3.insert(name.to_owned(), stacked);
    }
    let twins = twins(&setup.map);
    if twins > 0 {
        peaks.4.insert(name.to_owned(), twins);
    }
    census(&map, &setup.map, rules, &mut peaks.5);
    if map.header.version == Version::RoE {
        let counts = &mut peaks.5;
        roe(&map, base, &mut |what, n| {
            *counts.entry(what).or_default() += n
        })?;
    }
    let outcome = match import(&map, rules) {
        Ok(setup) => {
            towns(&map, &setup, rules, &mut peaks.5);
            let game = match GameState::new(rules, &setup, SEED) {
                // E09 T7a1: the main towns' heroes are T7b2's.
                Err(GameSetupError::HeroDraw(draw)) => {
                    return Ok((map.header.version, refused(draw).to_owned()));
                }
                // E09 T7a2: a map the setup's checks of the towns and the starts refuse, by the
                // check's name.
                Err(error) => match checked(error) {
                    Some(check) => {
                        let outcome = format!("imported; the setup's check {check} refuses it");
                        return Ok((map.header.version, outcome));
                    }
                    None => {
                        let why = refusal(rules, &setup, error, &mut peaks.6);
                        return Err(format!("imported, but invalid: {why}"));
                    }
                },
                Ok(game) => game,
            };
            // E08 T29: the placed heroes' levels at load (GAM-SET-002; survey
            // E08-progression.md §6.5: 9 heroes, 41 level-ups on the 5 maps that import whole).
            let levels: Vec<u8> = game.heroes().iter().map(|h| h.level).collect();
            let counts = &mut peaks.5;
            let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
            add(
                "heroes levelled at load",
                levels.iter().filter(|&&l| l > 1).count(),
            );
            let ups: usize = levels.iter().map(|&l| usize::from(l - 1)).sum();
            add("level-ups at load", ups);
            if name.eq_ignore_ascii_case("Tutorial.tut") {
                tutorial(&game, rules)?;
            }
            IMPORTED.to_owned()
        }
        Err(ImportError::Unsupported(what)) => format!("not supported: {}", what.reason()),
        Err(e) => return Err(format!("import: {e}")),
    };
    Ok((map.header.version, outcome))
}

/// The outcome of a map the build refuses for a hero to draw (`TWN-STUB-3`, until E09 T7b2).
const fn refused(draw: HeroDraw) -> &'static str {
    match draw {
        HeroDraw::MainTown(_) => "imported; the build refuses a main town's hero (TWN-STUB-3)",
    }
}

/// The name of a setup's check of the towns and the starts that refuses a map (E09 T7a2), or of
/// the random heroes' draw (E09 T7b1), if `error` is one.
const fn checked(error: GameSetupError) -> Option<&'static str> {
    match error {
        GameSetupError::TownFaction(_) => Some("TownFaction (TWN-SET-002)"),
        GameSetupError::TownAlignment(_) => Some("TownAlignment (TWN-SET-002)"),
        GameSetupError::Choice(_) => Some("Choice (TWN-SET-001)"),
        GameSetupError::MainTown(_) => Some("MainTown (TWN-SET-001)"),
        GameSetupError::Pick(_) => Some("Pick (TWN-SET-006)"),
        GameSetupError::PickTaken(_) => Some("PickTaken (TWN-SET-006)"),
        GameSetupError::RandomHeroOwner(_) => Some("RandomHeroOwner (TWN-SET-008)"),
        GameSetupError::NoFreeHero(_) => Some("NoFreeHero (TWN-SET-007)"),
        _ => None,
    }
}

/// The factions' labels in the original packs' order (Castle 0 to Conflux 8).
const FACTIONS: [&str; 9] = [
    "E09 towns: class 98 castle",
    "E09 towns: class 98 rampart",
    "E09 towns: class 98 tower",
    "E09 towns: class 98 inferno",
    "E09 towns: class 98 necropolis",
    "E09 towns: class 98 dungeon",
    "E09 towns: class 98 stronghold",
    "E09 towns: class 98 fortress",
    "E09 towns: class 98 conflux",
];

/// E09 T7a1's census of a map that imports (`docs/formats/h3m.md`, "Import"): its towns by kind,
/// faction and alignment, their custom building sets (with a horde built, a Shipyard built and
/// that Shipyard without water on either tile two rows below the entrance and one column aside,
/// TWN-STUB-2's case), custom garrisons (with random stacks), names and events; the players'
/// starts (main towns, those generating a hero, the random-hero flag, the choices, an RoE map's
/// players with all eight bits); the random heroes (with a custom experience), the placed heroes
/// on a town's entrance (moved there, TWN-SET-010: no map places one there itself, survey §1.4)
/// and the pool's inputs.
fn towns(
    map: &RawMap,
    setup: &GameSetup,
    rules: &GameRuleset,
    counts: &mut BTreeMap<&'static str, usize>,
) {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    let records = rules.towns();
    let hordes = |faction: h3_content::FactionId| {
        records.town(faction).map_or(Vec::new(), |record| {
            record
                .buildings
                .iter()
                .filter_map(|b| b.forbidden_with)
                .filter_map(|horde| record.building(horde).and_then(|b| b.slot))
                .collect::<Vec<u8>>()
        })
    };
    let shipyard = |faction: h3_content::FactionId| {
        records.town(faction).and_then(|record| {
            record
                .buildings
                .iter()
                .find(|b| b.key == "shipyard")
                .and_then(|b| b.slot)
        })
    };
    let factions: Vec<_> = (0..rules.core().factions().len())
        .filter_map(h3_content::FactionId::from_index)
        .collect();
    let side = setup.map.width;
    let water = |x: u16, y: u16, level: u8| {
        let index = (usize::from(level) * usize::from(side) + usize::from(y)) * usize::from(side)
            + usize::from(x);
        x < side
            && y < side
            && setup.map.tiles.get(index).is_some_and(|tile| {
                matches!(
                    rules.adventure().passage(tile.terrain),
                    Some(h3_adventure::Passage::Water(_))
                )
            })
    };
    // The towns' entrances: the anchor less two columns (survey finding 3).
    let entrances: Vec<(u16, u16, u8)> = map
        .objects
        .iter()
        .filter(|o| matches!(o.body, Body::Town(_)))
        .map(|o| {
            (
                u16::from(o.pos.x).saturating_sub(2),
                u16::from(o.pos.y),
                o.pos.z,
            )
        })
        .collect();
    add("E09 maps that import", 1);
    for (town, &(x, y, level)) in setup.towns.iter().zip(&entrances) {
        let candidates = match town.faction {
            TownFaction::Fixed(faction) => {
                add(
                    FACTIONS
                        .get(faction.index())
                        .copied()
                        .unwrap_or("E09 towns: class 98 other"),
                    1,
                );
                vec![faction]
            }
            TownFaction::Random { alignment } => {
                add(
                    if alignment.is_some() {
                        "E09 towns: class 77 aligned to a colour"
                    } else {
                        "E09 towns: class 77 without an alignment"
                    },
                    1,
                );
                factions.clone()
            }
        };
        if let BuildingsSetup::Custom { built, .. } = town.buildings {
            add("E09 towns: a custom building set", 1);
            let horde = candidates
                .iter()
                .any(|&f| hordes(f).iter().any(|&slot| built.contains(slot)));
            add(
                "E09 towns: a custom set with a horde built",
                usize::from(horde),
            );
            let yard = candidates
                .iter()
                .any(|&f| shipyard(f).is_some_and(|slot| built.contains(slot)));
            add(
                "E09 towns: a custom set with a Shipyard built",
                usize::from(yard),
            );
            let wet = water(x.wrapping_sub(1), y + 2, level) || water(x + 1, y + 2, level);
            add(
                "E09 towns: a built Shipyard without water (TWN-STUB-2)",
                usize::from(yard && !wet),
            );
        }
        if let Some(garrison) = &town.garrison {
            add("E09 towns: a custom garrison", 1);
            let random = garrison
                .iter()
                .any(|s| matches!(s.creature, GarrisonCreature::Random { .. }));
            add(
                "E09 towns: a custom garrison with a random stack",
                usize::from(random),
            );
            add(
                "E09 towns: a custom garrison, empty",
                usize::from(garrison.is_empty()),
            );
        }
        add("E09 towns: named", usize::from(town.name.is_some()));
        add(
            "E09 towns: with events",
            usize::from(!town.events.is_empty()),
        );
    }
    for player in &setup.players {
        let start = &player.start;
        add("E09 players", 1);
        add(
            "E09 players: a main town",
            usize::from(start.main_town.is_some()),
        );
        add(
            "E09 players: a main town generating a hero",
            usize::from(start.main_town.is_some_and(|t| t.generates_hero)),
        );
        add(
            "E09 players: the random-hero flag",
            usize::from(start.random_hero),
        );
        add(
            match start.choice {
                Some(FactionChoice::Fixed(_)) => "E09 players: choice Fixed",
                Some(FactionChoice::Random) if start.factions.is_empty() => {
                    "E09 players: choice Random, no faction"
                }
                Some(FactionChoice::Random) => "E09 players: choice Random",
                None => "E09 players: no choice",
            },
            1,
        );
        let eight = map.header.version == Version::RoE
            && map.players[usize::from(player.colour)]
                .details
                .as_ref()
                .is_some_and(|d| d.allowed_towns & 0xFF == 0xFF);
        add(
            "E09 players: RoE format, all eight factions",
            usize::from(eight),
        );
    }
    for hero in &setup.heroes {
        let random = hero.hero_type.is_none();
        add("E09 heroes: random", usize::from(random));
        add(
            "E09 heroes: random, custom experience",
            usize::from(random && hero.details.experience.is_some()),
        );
        let (x, y, level) = (hero.pos.x, hero.pos.y, hero.pos.level);
        add(
            "E09 heroes: on a town's entrance (TWN-SET-010)",
            usize::from(entrances.contains(&(x, y, level))),
        );
    }
    let allowed = setup.allowed_heroes.len();
    add(
        match allowed {
            156 => "E09 pool: maps allowing 156 heroes",
            144..=155 => "E09 pool: maps allowing 144 to 155",
            128..=143 => "E09 pool: maps allowing 128 to 143",
            _ => "E09 pool: maps allowing fewer than 128",
        },
        1,
    );
    add(
        "E09 pool: maps with hero masks",
        usize::from(!setup.hero_masks.is_empty()),
    );
    add("E09 pool: hero masks", setup.hero_masks.len());
    add("E09 pool: prisoners", setup.prisoners.len());
}

/// The Tutorial's hero's day's points at the start (E08 T42, increment 5's exit criterion 1;
/// survey `E08-hero-movement.md` §5): red's hero, the map's first, has Expert Logistics and wears
/// Boots of Speed and Equestrian's Gloves, so its army's entry of the land table grows by 30 % and
/// 900 points (HER-ADV-003, ADV-MOV-001; 2 928 from its speed-4 army's 1 560), printed and checked.
fn tutorial(game: &GameState, rules: &GameRuleset) -> Result<(), String> {
    let Some(hero) = game.adventure().heroes().first() else {
        return Err("Tutorial.tut: no placed hero".to_owned());
    };
    let details = game
        .hero(hero.id)
        .ok_or("Tutorial.tut: the hero's details")?;
    let entry = land_table_points(rules.core(), rules.adventure(), Some(details), &hero.army);
    let expected = entry * 13 / 10 + 900;
    println!(
        "  Tutorial.tut's hero: {} day's points (its army's entry {entry} × 1.3 + 900)",
        hero.movement
    );
    if hero.movement == expected {
        Ok(())
    } else {
        Err(format!(
            "Tutorial.tut: the hero's day's points {} are not {expected}",
            hero.movement
        ))
    }
}

/// Counts the monsters' data the import reads or refuses and the special grounds (E08 T20a2a):
/// treasures with a resource below 0 (the import leaves such an amount out), dispositions
/// beyond savage, treasures with an artifact; grounds, those whose picture the import's table
/// lacks (they lie on one tile), and tiles two or more grounds lie on; (E08 T21a) the guarded
/// piles the import keeps, those whose guards ask first, and the piles with a guards record the
/// import reads as plain (no stack) or leaves unguarded (a random stack); since E08 T61b the
/// guarded objects by kind (T61a gave guards to artifacts, scrolls, boxes and events) and the
/// scenario's banned spells and artifacts (GAM-SET-003).
fn census(
    map: &RawMap,
    setup: &MapSetup,
    rules: &GameRuleset,
    counts: &mut BTreeMap<&'static str, usize>,
) {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    // E08 T61b: the banned spells and artifacts, and bits past the original's codes (the
    // import's tables: 70 hero spells before the abilities, 141 artifacts).
    let set = |bits: &h3_map::model::Bits, from: usize| {
        (from..bits.bytes().len() * 8)
            .filter(|&code| bits.get(code))
            .count()
    };
    add(
        "maps banning spells",
        usize::from(set(&map.banned_spells, 0) > 0),
    );
    add("banned spells", set(&map.banned_spells, 0));
    add(
        "banned spells past the 70 hero spells",
        set(&map.banned_spells, 70),
    );
    add(
        "maps banning artifacts",
        usize::from(set(&map.banned_artifacts, 0) > 0),
    );
    add("banned artifacts", set(&map.banned_artifacts, 0));
    add(
        "banned artifacts past the 141 codes",
        set(&map.banned_artifacts, 141),
    );
    // E08 T29: the scenario's level limit and banned skills (survey E08-progression.md §6.2,
    // §6.3: none has a limit, 19 SoD maps ban skills, none past the 28 skills' codes).
    add(
        "maps with a level limit",
        usize::from(map.header.level_limit > 0),
    );
    let banned = map.banned_skills.bytes().iter().any(|&b| b != 0);
    add("maps banning skills", usize::from(banned));
    let past = (28..map.banned_skills.bytes().len() * 8).any(|code| map.banned_skills.get(code));
    add("maps banning a skill past the 28 codes", usize::from(past));
    for object in &map.objects {
        if let Body::Monster(m) = &object.body {
            add("monsters", 1);
            add(
                "monsters of a disposition beyond savage",
                usize::from(m.disposition > 4),
            );
            if let Some(t) = &m.treasure {
                add("monsters with a treasure", 1);
                add(
                    "monsters with a resource below 0 in their treasure",
                    usize::from(t.resources.iter().any(|&r| r < 0)),
                );
                add(
                    "monsters with an artifact in their treasure",
                    usize::from(t.artifact.is_some()),
                );
            }
        }
    }
    let records = map.objects.iter().filter(
        |o| matches!(&o.body, Body::Resource { message: Some(m), .. } if m.guards.is_some()),
    );
    add("piles with a guards record", records.count());
    for object in &setup.objects {
        let Some(guards) = &object.guards else {
            continue;
        };
        let kind = object
            .object_type
            .and_then(|id| rules.adventure().object_type(id))
            .map(|ty| &ty.kind);
        let (all, ask, two) = match kind {
            Some(ObjectKind::Pickup(_)) => (
                "guarded piles",
                "guarded piles that ask first",
                "guarded piles with two or more stacks",
            ),
            Some(ObjectKind::Artifact { .. }) => (
                "guarded artifacts",
                "guarded artifacts that ask first",
                "guarded artifacts with two or more stacks",
            ),
            Some(ObjectKind::Scroll) => (
                "guarded scrolls",
                "guarded scrolls that ask first",
                "guarded scrolls with two or more stacks",
            ),
            Some(ObjectKind::PandoraBox) => (
                "guarded boxes",
                "guarded boxes that ask first",
                "guarded boxes with two or more stacks",
            ),
            _ => (
                "guarded events",
                "guarded events that ask first",
                "guarded events with two or more stacks",
            ),
        };
        add(all, 1);
        add(ask, usize::from(guards.question));
        add(two, usize::from(guards.army.len() > 1));
    }
    events(setup, rules, &mut add);
    sea(setup, rules, &mut add);
    economy(map, setup, rules, &mut add);
    dwellings(map, setup, &mut add);
    let mut lying = BTreeMap::<(u16, u16, u8), usize>::new();
    let ground = |object: &&h3_adventure::ObjectSetup| {
        matches!(
            object
                .object_type
                .and_then(|ty| rules.adventure().object_type(ty))
                .map(|ty| &ty.kind),
            Some(ObjectKind::Ground { .. })
        )
    };
    for object in setup.objects.iter().filter(ground) {
        add("special grounds", 1);
        add(
            "special grounds of a picture the import lacks",
            usize::from(object.drawn.bits() == 1),
        );
        for (column, row) in object.drawn.offsets() {
            if let (Some(x), Some(y)) = (
                object.anchor.x.checked_sub(u16::from(column)),
                object.anchor.y.checked_sub(u16::from(row)),
            ) {
                *lying.entry((x, y, object.anchor.level)).or_default() += 1;
            }
        }
    }
    add(
        "tiles two or more special grounds lie on",
        lying.values().filter(|&&n| n > 1).count(),
    );
}

/// Counts the sea's objects (E08 T72, increment 8's design note §4): the boats and the maps with
/// boats, the boats by type, the sea's pickups by type, the holes, Favorable Winds and those of a
/// picture the import lacks (they lie on one tile).
fn sea(setup: &MapSetup, rules: &GameRuleset, add: &mut impl FnMut(&'static str, usize)) {
    add("boats", setup.boats.len());
    add("maps with boats", usize::from(!setup.boats.is_empty()));
    for boat in &setup.boats {
        add(
            ["boats of type 0", "boats of type 1", "boats of type 2"][usize::from(boat.kind)],
            1,
        );
    }
    for object in &setup.objects {
        let Some(ty) = object
            .object_type
            .and_then(|ty| rules.adventure().object_type(ty))
        else {
            continue;
        };
        match ty.key.as_str() {
            "base:flotsam" => add("sea pickups: Flotsam", 1),
            "base:sea_chest" => add("sea pickups: Sea Chests", 1),
            "base:shipwreck_survivor" => add("sea pickups: Shipwreck Survivors", 1),
            "base:ocean_bottle" => add("sea pickups: Ocean Bottles", 1),
            "base:hole" => add("holes", 1),
            "sod:favorable_winds" => {
                add("Favorable Winds", 1);
                add(
                    "Favorable Winds of a picture the import lacks",
                    usize::from(object.drawn.bits() == 1),
                );
            }
            _ => {}
        }
    }
}

/// Counts what the game's build gives increment 9's objects (E08 T83b1): the owned objects (each
/// reveals around its entrance at the start), the abandoned mines whose drawn resource is outside
/// their set or whose guards' count is outside 100 to 200 (none), the Shipyards without a boat
/// tile among the offsets, by name and entrance (survey `E08-economy.md` §1: Hoard's at (62, 38,
/// 0)), and the weekly objects whose first week does not fit their kind (none).
fn built_economy(
    game: &GameState,
    rules: &GameRuleset,
    name: &str,
    counts: &mut BTreeMap<&'static str, usize>,
) -> Vec<String> {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    let adventure = game.adventure();
    let map = adventure.map();
    let mut without = Vec::new();
    for &index in map.ownables() {
        let object = &map.objects()[index];
        add(
            "economy: objects owned at the start",
            usize::from(adventure.owner(index).is_some()),
        );
        let kind = object
            .object_type
            .and_then(|ty| rules.adventure().object_type(ty))
            .map(|ty| &ty.kind);
        match (kind, &object.holds, &object.resolved) {
            (_, Holds::Resources(set), resolved) => {
                let fits = resolved.mine().is_some_and(|(resource, guards)| {
                    set.contains(&resource) && (100..=200).contains(&guards)
                });
                add(
                    "economy: abandoned mines drawn outside their set or 100 to 200",
                    usize::from(!fits),
                );
            }
            (Some(ObjectKind::Shipyard { .. }), _, resolved) if resolved.boat_tile().is_none() => {
                add("economy: Shipyards without a boat tile", 1);
                without.push(format!("{name} {:?}", object.entrance(map)));
            }
            _ => {}
        }
    }
    add(
        "economy: weekly objects whose first week does not fit",
        usize::from(!adventure.check_weekly(rules.adventure())),
    );
    add(
        "economy: maps whose owners do not check",
        usize::from(!adventure.check_owners(rules.adventure())),
    );
    add(
        "economy: maps whose fractions and added objects do not check",
        usize::from(!adventure.check_added(rules.adventure())),
    );
    without
}

/// Counts the dwellings the game's build gives (E08 T94a; ADV-DWL-005, increment 10's design note
/// §5), on the static map built with [`SEED`]: the dwellings (`StaticMap::dwellings`), placed by
/// the map or random dwellings resolved (their setup object holds a [`Holds::RandomDwelling`]),
/// the resolved ones by class (216 to 218) and by the faction and level drawn (the first faction
/// of the set whose table has the type at a level of the range), those whose anchor moved (the
/// Angel Portal's shift) and those whose moved anchor lies past the right edge (x the map's width;
/// returned by name and anchor), the Refugee Camps (`StaticMap::camps`) and the dwellings owned at
/// the start. Checked (an error fails the map): the dwellings and the camps are the built objects
/// of their kinds in object order; each typed object of 216 to 218 holds a set and a range and is
/// resolved, its type its set's tables' at a level of its range, its anchor moved right by the
/// type's shift.
fn built_dwellings(
    game: &GameState,
    raw: &RawMap,
    setup: &MapSetup,
    rules: &GameRuleset,
    name: &str,
    built: &mut Built,
) -> Result<Vec<String>, String> {
    let adventure = game.adventure();
    let map = adventure.map();
    let part = rules.adventure();
    let kind = |ty: Option<ObjectTypeId>| ty.and_then(|ty| part.object_type(ty)).map(|ty| &ty.kind);
    let objects = map.objects();
    let of_kind = |dwelling: bool| -> Vec<usize> {
        objects
            .iter()
            .map(|o| kind(o.object_type))
            .enumerate()
            .filter(|(_, kind)| match kind {
                Some(ObjectKind::Dwelling { .. }) => dwelling,
                Some(ObjectKind::RefugeeCamp { .. }) => !dwelling,
                _ => false,
            })
            .map(|(index, _)| index)
            .collect()
    };
    let classes: Vec<u32> = placed(raw).map(|(_, class)| class).collect();
    if objects.len() != setup.objects.len()
        || classes.len() != setup.objects.len()
        || of_kind(true) != map.dwellings()
        || of_kind(false) != map.camps()
    {
        return Err(format!(
            "{name}: the built dwellings or camps are not the objects of their kinds"
        ));
    }
    let (counts, spread) = (&mut built.0, &mut built.1);
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    add("maps built", 1);
    add("dwellings", map.dwellings().len());
    add("Refugee Camps", map.camps().len());
    let owned = |index| usize::from(adventure.owner(index).is_some());
    add(
        "dwellings owned at the start",
        map.dwellings().iter().map(|&index| owned(index)).sum(),
    );
    let mut edge = Vec::new();
    for ((index, object), &class) in setup.objects.iter().enumerate().zip(&classes) {
        let at = |why: &str| {
            format!(
                "{name}: the random dwelling at {:?} (object {index}) {why}",
                object.anchor
            )
        };
        let Holds::RandomDwelling {
            factions,
            levels: (low, high),
        } = &object.holds
        else {
            if (216..=218).contains(&class) && object.object_type.is_some() {
                return Err(at("holds no set and range"));
            }
            add(
                "dwellings placed by the map",
                usize::from(map.dwelling_index(index).is_some()),
            );
            continue;
        };
        let resolved = &objects[index];
        let Some(ObjectKind::Dwelling { shift, .. }) = kind(resolved.object_type) else {
            return Err(at("is not resolved"));
        };
        let drawn = factions.iter().find_map(|&faction| {
            let row = part.random_dwelling(faction)?;
            (*low..=*high)
                .find(|&level| {
                    let place = usize::from(level).checked_sub(1);
                    place.and_then(|place| row.get(place)) == resolved.object_type.as_ref()
                })
                .map(|level| (faction.index(), level))
        });
        let anchor = object.anchor;
        let moved = Pos::new(
            anchor.x.saturating_add(u16::from(*shift)),
            anchor.y,
            anchor.level,
        );
        let Some((faction, level)) = drawn.filter(|_| resolved.anchor == moved) else {
            return Err(at(
                "is not a type of its set's tables at a level of its range, its anchor moved by \
                 the type's shift",
            ));
        };
        let class = match class {
            216 => "random dwellings of class 216 resolved",
            217 => "random dwellings of class 217 resolved",
            218 => "random dwellings of class 218 resolved",
            _ => return Err(at("is of another class")),
        };
        spread.entry(faction).or_default()[usize::from(level - 1)] += 1;
        add(class, 1);
        add("random dwellings resolved", 1);
        add("resolved ones owned at the start", owned(index));
        add(
            "resolved with the anchor moved (the Angel Portal's shift)",
            usize::from(*shift != 0),
        );
        let past = resolved.anchor.x == map.width();
        add(
            "resolved with the moved anchor past the right edge",
            usize::from(past),
        );
        if past {
            edge.push(format!("{name} {:?}", resolved.anchor));
        }
    }
    Ok(edge)
}

/// Counts the dwellings' and the Refugee Camps' weeks the game's build gives (E08 T94b;
/// ADV-DWL-002, ADV-DWL-003, ADV-RFG-001, increment 10's design note §6, §9), on the state built
/// with [`SEED`]: the dwellings with guards, their guard stacks and creatures, the camps drawn and
/// their draws by the drawn creature's level. Checked (an error fails the map): the state's
/// dwellings' and camps' invariants hold (`check_dwellings`, `check_camps`); each dwelling's
/// counts are its kind's creatures' growths in creature order, its guards one stack for each of
/// its creatures of the part's guard level or more, in creature order from slot 0, of
/// [`GUARD_FACTOR`] times its growth, and so are the kind's derived full guards; each camp holds a
/// creature of its levels' random set, its count that creature's growth.
fn built_weeks(
    game: &GameState,
    rules: &GameRuleset,
    name: &str,
    weeks: &mut Weeks,
) -> Result<(), String> {
    let adventure = game.adventure();
    let map = adventure.map();
    let (core, part) = (rules.core(), rules.adventure());
    let objects = map.objects();
    let kind = |index: usize| {
        objects[index]
            .object_type
            .and_then(|ty| part.object_type(ty))
            .map(|ty| &ty.kind)
    };
    let at = |index: usize, what: &str, why: &str| {
        format!(
            "{name}: the {what} at {:?} (object {index}) {why}",
            objects[index].anchor
        )
    };
    let growth = |creature| core.creature(creature).map(|c| u32::from(c.growth));
    let Some(params) = part.params().dwellings() else {
        return Err(format!("{name}: the part has no dwellings' parameters"));
    };
    if !adventure.check_dwellings() || !adventure.check_camps(core, part) {
        return Err(format!(
            "{name}: the built dwellings' or camps' weeks break their invariants"
        ));
    }
    let (counts, by_level) = (&mut weeks.0, &mut weeks.1);
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    for &index in map.dwellings() {
        let Some(ObjectKind::Dwelling {
            creatures, guards, ..
        }) = kind(index)
        else {
            return Err(at(index, "dwelling", "is not of a dwelling's kind"));
        };
        let growths: Option<Vec<u32>> = creatures.iter().map(|&c| growth(c)).collect();
        if growths.as_deref() != adventure.dwelling_counts(index) {
            return Err(at(
                index,
                "dwelling",
                "does not hold its creatures' growths",
            ));
        }
        let full: Vec<(CreatureTypeId, u32)> = creatures
            .iter()
            .filter(|&&c| {
                core.creature(c)
                    .is_some_and(|c| c.level >= params.guard_level)
            })
            .map(|&c| (c, GUARD_FACTOR * growth(c).unwrap_or(0)))
            .collect();
        let stood: Vec<(u8, CreatureTypeId, u32)> = adventure
            .dwelling_guards(index)
            .iter()
            .map(|stack| (stack.slot, stack.creature, stack.count))
            .collect();
        let slotted: Vec<(u8, CreatureTypeId, u32)> = (0..)
            .zip(&full)
            .map(|(slot, &(creature, count))| (slot, creature, count))
            .collect();
        if *guards != full || stood != slotted {
            return Err(at(
                index,
                "dwelling",
                "is not guarded by a stack of 3 times the growth of each of its creatures of the \
                 guard level or more, in creature order",
            ));
        }
        add("dwellings with guards", usize::from(!full.is_empty()));
        add("dwellings' guard stacks", full.len());
        let guarding: u32 = full.iter().map(|&(_, count)| count).sum();
        add("dwellings' guards (creatures)", guarding as usize);
    }
    for &index in map.camps() {
        let Some(ObjectKind::RefugeeCamp { levels }) = kind(index) else {
            return Err(at(index, "Refugee Camp", "is not of a camp's kind"));
        };
        let drawn = adventure.camp(index).and_then(|(creature, count)| {
            let info = core.creature(creature)?;
            let place = usize::from(info.level).checked_sub(1)?;
            part.random_monsters(core, *levels)
                .any(|c| c == creature)
                .then_some((place, count == u32::from(info.growth)))
        });
        let Some((place, true)) = drawn else {
            return Err(at(
                index,
                "Refugee Camp",
                "holds no creature of its levels' random set, its growth the count",
            ));
        };
        let Some(level) = by_level.get_mut(place) else {
            return Err(at(
                index,
                "Refugee Camp",
                "holds a creature past the levels",
            ));
        };
        *level += 1;
        add("Refugee Camps drawn", 1);
    }
    Ok(())
}

/// Why the game refuses `setup` (E08 T94a): `error`, and when a random dwelling's tiles refuse it,
/// counted in `built` by the reason and named so: a hero on one
/// (`SetupError::RandomDwellingTile`), or a boat the same setup without its random dwellings does
/// not refuse (`SetupError::Boat`).
fn refusal(
    rules: &GameRuleset,
    setup: &GameSetup,
    error: GameSetupError,
    built: &mut Built,
) -> String {
    let why = match error {
        GameSetupError::Adventure(SetupError::RandomDwellingTile { .. }) => Some(REFUSED_HERO),
        GameSetupError::Adventure(SetupError::Boat(_)) => {
            let mut without = setup.clone();
            without
                .map
                .objects
                .retain(|o| !matches!(o.holds, Holds::RandomDwelling { .. }));
            (GameState::new(rules, &without, SEED).err() != Some(error)).then_some(REFUSED_BOAT)
        }
        _ => None,
    };
    match why {
        Some(why) => {
            *built.0.entry(why).or_default() += 1;
            format!("{error} ({why})")
        }
        None => error.to_string(),
    }
}

/// Counts increment 9's objects and the timed events (E08 T83a): the mines, those owned at
/// load, the abandoned mines by the size of their set, the Lighthouses and the Shipyards (owned
/// at load), the weekly objects by kind; the timed events, those for no player that plays and
/// those that repeat.
fn economy(
    map: &RawMap,
    setup: &MapSetup,
    rules: &GameRuleset,
    add: &mut impl FnMut(&'static str, usize),
) {
    for object in &setup.objects {
        let Some(ty) = object
            .object_type
            .and_then(|ty| rules.adventure().object_type(ty))
        else {
            continue;
        };
        let owned = usize::from(object.owner.is_some());
        match &ty.kind {
            ObjectKind::Mine { .. } => {
                add("economy: mines", 1);
                add("economy: mines owned at load", owned);
            }
            ObjectKind::AbandonedMine { .. } => {
                add("economy: abandoned mines", 1);
                let size = match &object.holds {
                    Holds::Resources(set) => set.len(),
                    _ => 0,
                };
                let sizes = [
                    "economy: abandoned mines of 1 resource",
                    "economy: abandoned mines of 2 to 6 resources",
                    "economy: abandoned mines of all 7 resources",
                ];
                add(sizes[usize::from(size > 1) + usize::from(size > 6)], 1);
            }
            ObjectKind::Lighthouse { .. } => {
                add("economy: Lighthouses", 1);
                add("economy: Lighthouses owned at load", owned);
            }
            ObjectKind::Shipyard { .. } => {
                add("economy: Shipyards", 1);
                add("economy: Shipyards owned at load", owned);
            }
            ObjectKind::Windmill { .. } => add("economy: Windmills", 1),
            ObjectKind::WaterWheel { .. } => add("economy: Water Wheels", 1),
            ObjectKind::Garden { .. } => add("economy: Mystical Gardens", 1),
            // E09 T8a1: the towns typed, their owners at load and formations.
            ObjectKind::Town => {
                add("E09 towns: typed", 1);
                add("E09 towns: owned at load", owned);
                let tight = Holds::Town {
                    formation: Formation::Tight,
                };
                add(
                    "E09 towns: a tight formation",
                    usize::from(object.holds == tight),
                );
            }
            _ => {}
        }
    }
    let playing = map
        .players
        .iter()
        .zip(0..)
        .filter(|(player, _)| player.human || player.computer)
        .fold(0u8, |bits, (_, colour)| bits | 1 << colour);
    add("timed events", map.events.len());
    add(
        "timed events for no player that plays",
        map.events
            .iter()
            .filter(|e| e.players & playing == 0)
            .count(),
    );
    add(
        "timed events that repeat",
        map.events.iter().filter(|e| e.interval != 0).count(),
    );
}

/// Counts increment 10's dwellings (E08 T93b, increment 10's design note §3, §4): the dwellings
/// (17, 20) and the Refugee Camps (78) typed by class, the random dwellings (216 to 218) typed by
/// class, those with a faction set (the survey: 206), and those
/// tied to a town, which stay blockers (ADV-STUB-13), with those tied to a random town (the
/// survey: about 1 397, all to random towns).
fn dwellings(map: &RawMap, setup: &MapSetup, add: &mut impl FnMut(&'static str, usize)) {
    let random_towns: BTreeSet<u32> = placed(map)
        .filter(|&(_, class)| class == 77)
        .filter_map(|(object, _)| match &object.body {
            Body::Town(town) => town.identifier,
            _ => None,
        })
        .collect();
    for ((object, class), entry) in placed(map).zip(&setup.objects) {
        let typed = entry.object_type.is_some();
        let what = match class {
            17 => "dwellings: class 17 typed",
            20 => "dwellings: class 20 typed",
            78 => "dwellings: Refugee Camps (78) typed",
            216 => "dwellings: random dwellings of class 216 typed",
            217 => "dwellings: random dwellings of class 217 typed",
            218 => "dwellings: random dwellings of class 218 typed",
            _ => continue,
        };
        add(what, usize::from(typed));
        let Body::RandomDwelling(body) = &object.body else {
            continue;
        };
        match body.faction {
            Some((0, _)) => add("dwellings: random dwellings with a faction set", 1),
            Some((town, _)) if !typed => {
                add(
                    "dwellings: random dwellings tied to a town (blockers, ADV-STUB-13)",
                    1,
                );
                add(
                    "dwellings: random dwellings tied to a random town",
                    usize::from(random_towns.contains(&town)),
                );
            }
            _ => {}
        }
    }
}

/// An RoE map's dwellings under the base pack alone (E08 T93b, increment 10's design note §3):
/// its static layer must import with the RoE ruleset (`base`), each of its dwellings (17, 20)
/// typed, so none takes a type AB added; the maps and the dwellings by class counted (the survey:
/// 49 RoE maps; of class 17 695 on 44, of class 20 14 on 10).
fn roe(
    map: &RawMap,
    base: &GameRuleset,
    add: &mut impl FnMut(&'static str, usize),
) -> Result<(), String> {
    let setup = import_map(map, base).map_err(|e| format!("under the base pack alone: {e}"))?;
    let (mut all, mut typed) = (0, 0);
    for ((_, class), entry) in placed(map).zip(&setup.objects) {
        let what = match class {
            17 => "dwellings: RoE maps' class 17 typed with the base pack alone",
            20 => "dwellings: RoE maps' class 20 typed with the base pack alone",
            _ => continue,
        };
        all += 1;
        let one = usize::from(entry.object_type.is_some());
        typed += one;
        add(what, one);
    }
    add("dwellings: RoE maps imported with the base pack alone", 1);
    add(
        "dwellings: RoE maps' dwellings (17, 20) typed with the base pack alone",
        typed,
    );
    if typed == all {
        Ok(())
    } else {
        Err(format!(
            "under the base pack alone: {} of its {all} dwellings (17, 20) untyped",
            all - typed
        ))
    }
}

/// Counts the events (E08 T62b2, ADV-OBJ-007): how many, those that fire for a computer, those
/// removed after they fire, those whose template has other than one visitable tile, and the
/// tiles they lie on that are another object's visitable or blocked tile, another event's, water,
/// or within one step of a monster's visitable tile on its level (a guard may stand there).
fn events(setup: &MapSetup, rules: &GameRuleset, add: &mut impl FnMut(&'static str, usize)) {
    let part = rules.adventure();
    let kind = |o: &h3_adventure::ObjectSetup| {
        o.object_type
            .and_then(|id| part.object_type(id))
            .map(|ty| &ty.kind)
    };
    let tiles = |o: &h3_adventure::ObjectSetup, mask: h3_adventure::TileMask| {
        let anchor = o.anchor;
        mask.offsets()
            .filter_map(move |(column, row)| {
                let x = anchor.x.checked_sub(u16::from(column))?;
                let y = anchor.y.checked_sub(u16::from(row))?;
                Some((anchor.level, y, x))
            })
            .collect::<Vec<_>>()
    };
    let is_event = |o: &h3_adventure::ObjectSetup| matches!(kind(o), Some(ObjectKind::Event));
    let mut visited = BTreeMap::<(u8, u16, u16), usize>::new();
    let mut blocked = BTreeMap::<(u8, u16, u16), usize>::new();
    let mut monsters = Vec::new();
    for o in &setup.objects {
        if is_event(o) {
            add(
                "events whose template blocks a tile",
                usize::from(o.blocked.bits() != 0),
            );
            continue;
        }
        for at in tiles(o, o.blocked) {
            *blocked.entry(at).or_default() += 1;
        }
        {
            for at in tiles(o, o.visitable) {
                *visited.entry(at).or_default() += 1;
                if o.monster.is_some() {
                    monsters.push(at);
                }
            }
        }
    }
    let mut lying = BTreeMap::<(u8, u16, u16), usize>::new();
    for o in setup.objects.iter().filter(|o| is_event(o)) {
        let Holds::Event(event) = &o.holds else {
            continue;
        };
        add("events", 1);
        add(
            "events that fire for a computer",
            usize::from(event.computer),
        );
        add("events removed after they fire", usize::from(event.remove));
        add("events with guards", usize::from(o.guards.is_some()));
        let on = tiles(o, o.visitable);
        add(
            "events with other than one visitable tile",
            usize::from(on.len() != 1),
        );
        for at in on {
            *lying.entry(at).or_default() += 1;
            add(
                "events on another object's visitable tile",
                usize::from(visited.contains_key(&at)),
            );
            add(
                "events on another object's blocked tile",
                usize::from(blocked.contains_key(&at)),
            );
            let (level, y, x) = at;
            let side = setup.width;
            let index = (usize::from(level) * usize::from(setup.height) + usize::from(y))
                * usize::from(side)
                + usize::from(x);
            let water = setup.tiles.get(index).is_some_and(|t| {
                matches!(
                    part.passage(t.terrain),
                    Some(h3_adventure::Passage::Water(_))
                )
            });
            add("events on water", usize::from(water));
            let near = monsters
                .iter()
                .any(|&(l, my, mx)| l == level && my.abs_diff(y) <= 1 && mx.abs_diff(x) <= 1);
            add("events within one step of a monster", usize::from(near));
        }
    }
    add(
        "tiles two or more events lie on",
        lying.values().filter(|&&n| n > 1).count(),
    );
}

/// The object classes the static layer leaves out (hero, random hero, hero placeholder), as
/// `h3_map::import_map` does.
const HERO_CLASSES: [u32; 3] = [34, 70, 214];

/// The boat's class: the setup lists boats apart (E08 T72).
const BOAT: u32 = 8;

/// The map's objects the static layer places, in its order, with their classes: all but the
/// heroes and (since E08 T93b, which needs the order) the boats.
fn placed(map: &RawMap) -> impl Iterator<Item = (&Object, u32)> {
    map.objects
        .iter()
        .filter_map(|o| {
            let template = map.templates.get(usize::try_from(o.template).ok()?)?;
            Some((o, template.class))
        })
        .filter(|(_, class)| !HERO_CLASSES.contains(class) && *class != BOAT)
}

/// How many tiles of the static layer objects are visited from, and each one that is the
/// visitable tile of two or more objects, with the objects' classes in map order.
fn stacked(map: &RawMap, setup: &MapSetup) -> (usize, Vec<Vec<u32>>) {
    let classes = placed(map).map(|(_, class)| class);
    let mut tiles = BTreeMap::<(u8, u16, u16), Vec<u32>>::new();
    for (object, class) in setup.objects.iter().zip(classes) {
        let anchor = object.anchor;
        for (column, row) in object.visitable.offsets() {
            let x = anchor.x.checked_sub(u16::from(column));
            let y = anchor.y.checked_sub(u16::from(row));
            if let (Some(x), Some(y)) = (x, y) {
                tiles.entry((anchor.level, y, x)).or_default().push(class);
            }
        }
    }
    let visitable = tiles.len();
    let stacked = tiles
        .into_values()
        .filter(|classes| classes.len() > 1)
        .collect();
    (visitable, stacked)
}

/// How many typed objects of the static layer share their type and anchor with an earlier one (a
/// random resource counted by its random type, before the draw).
fn twins(setup: &MapSetup) -> usize {
    let mut seen = BTreeMap::<_, usize>::new();
    for object in &setup.objects {
        if let Some(object_type) = object.object_type {
            let anchor = object.anchor;
            *seen
                .entry((object_type, anchor.level, anchor.y, anchor.x))
                .or_default() += 1;
        }
    }
    seen.values().map(|n| n - 1).sum()
}

fn note(map: &RawMap, peaks: &mut Peaks, name: &str) {
    let mut texts: Vec<&Text> = vec![&map.header.name, &map.header.description];
    for r in &map.rumors {
        texts.extend([&r.name, &r.text]);
    }
    for d in map
        .players
        .iter()
        .filter_map(|p: &Player| p.details.as_ref())
    {
        peaks.note("player heroes", d.heroes.len(), name);
        texts.extend(d.heroes.iter().map(|h| &h.name));
    }
    peaks.note("placeholders", map.placeholders.len(), name);
    peaks.note("disposed heroes", map.disposed_heroes.len(), name);
    peaks.note("rumors", map.rumors.len(), name);
    peaks.note("custom heroes", map.custom_heroes.len(), name);
    let hero =
        |peaks: &mut Peaks, skills: Option<&Vec<(u8, u8)>>, artifacts: Option<&Artifacts>| {
            peaks.note("secondary skills", skills.map_or(0, Vec::len), name);
            peaks.note("backpack", artifacts.map_or(0, |a| a.backpack.len()), name);
        };
    for h in &map.custom_heroes {
        hero(peaks, h.secondary_skills.as_ref(), h.artifacts.as_ref());
        texts.extend(h.biography.as_ref());
    }
    for o in &map.objects {
        *peaks.1.entry(kind(&o.body)).or_default() += 1;
        match &o.body {
            Body::Hero(h) => {
                hero(peaks, h.secondary_skills.as_ref(), h.artifacts.as_ref());
                texts.extend(h.name.iter().chain(h.biography.iter()));
            }
            Body::Event(e) => texts.extend(e.message.as_ref().map(|m| &m.text)),
            Body::Pandora { message, .. }
            | Body::Artifact(message)
            | Body::Scroll { message, .. }
            | Body::Resource { message, .. } => texts.extend(message.as_ref().map(|m| &m.text)),
            Body::Monster(m) => texts.extend(m.treasure.as_ref().map(|t| &t.text)),
            Body::Sign(t) => texts.push(t),
            Body::SeerHut(Some(SeerTask { quest, .. })) | Body::QuestGuard(Some(quest)) => {
                texts.extend([&quest.first_visit, &quest.next_visit, &quest.completed]);
            }
            Body::Town(t) => {
                peaks.note("town events", t.events.len(), name);
                texts.extend(t.name.iter());
                for e in &t.events {
                    texts.extend([&e.event.name, &e.event.message]);
                }
            }
            _ => {}
        }
    }
    for e in &map.events {
        texts.extend([&e.name, &e.message]);
    }
    peaks.note("templates", map.templates.len(), name);
    peaks.note("objects", map.objects.len(), name);
    peaks.note("timed events", map.events.len(), name);
    let longest = texts.iter().map(|t| t.as_bytes().len()).max().unwrap_or(0);
    peaks.note("string", longest, name);
}

/// The body's kind, for the summary.
const fn kind(body: &Body) -> &'static str {
    match body {
        Body::None => "none",
        Body::Event(_) => "event",
        Body::Hero(_) => "hero",
        Body::Monster(_) => "monster",
        Body::Sign(_) => "sign",
        Body::SeerHut(_) => "seer hut",
        Body::WitchHut(_) => "witch hut",
        Body::Scholar { .. } => "scholar",
        Body::Garrison { .. } => "garrison",
        Body::Artifact(_) => "artifact",
        Body::Scroll { .. } => "scroll",
        Body::Resource { .. } => "resource",
        Body::Town(_) => "town",
        Body::Mine { .. } => "mine",
        Body::AbandonedMine(_) => "abandoned mine",
        Body::Owned { .. } => "owned",
        Body::RandomDwelling(_) => "random dwelling",
        Body::Shrine { .. } => "shrine",
        Body::Pandora { .. } => "pandora",
        Body::Grail { .. } => "grail",
        Body::QuestGuard(_) => "quest guard",
        Body::HeroPlaceholder { .. } => "hero placeholder",
    }
}
