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
//! creatures and counts (checked); since E08 T96b2 each resolved random dwelling's entrance on the
//! map (checked) and the most dwellings, random dwellings and Refugee Camps on one map; since E08
//! T97b1 the typed dwellings entered from water.
//! Since E09 T7a1 the towns, the players' starts, the random heroes, the heroes moved onto a
//! town's entrance (TWN-SET-010) and the pool's inputs of the maps that import; since E09 T7b1 the
//! random heroes are drawn. Since E09 T8a1 the towns are typed (their owners at load and
//! formations counted), and the static layer's game gives one town record per town (a random
//! town of no alignment). Since E09 T7b2b the main towns' heroes are built (TWN-SET-009; until
//! then a setup with one was counted as refused, `TWN-STUB-3`): counted by pick and draw, with a
//! spell book in a town with a Mage Guild built (the guild's spells at the start, `TWN-STUB-6`),
//! and on the maps of 36 tiles a side. Since E09 T9c each built town's siege facts (its fort
//! level, a moat; TWN-SIE-003) and hall level, and the owned towns' day's income (TWN-INC-001,
//! -002); since E09 T18a their spots and Shipyards (TWN-SET-011: a town without a spot has its
//! Shipyard forbidden and unbuilt, checked). Since E08 T103b the teleporters (43 to 45, 103, 111) by class, subclass and level, and
//! the maps with each class; each checked to import as its class and subclass's type (a monolith
//! of its class's way with its subclass as the channel: the import's tables against the shipped
//! maps, increment 11's design note §3 and §4). Since E08 T104 the channels and the gates' pairs
//! the game's build gives (ADV-TEL-002, design note §5; no shipped map refused for a teleporter
//! off its surface, A-42): the one-way channels by entrances and exits, the two-way ones by size,
//! the Whirlpools' tiles, every visitable tile listed once, every gate paired, and the pairs and
//! the gates not paired with their nearest checked against the survey's census (§1: 301 pairs, 18
//! gates on 8 maps). Since E08 T106c the rest of that census checked too (§1): the teleporters by
//! class, subclass and level, the maps with each class, the one-way channels by entrances and
//! exits, the two-way ones by size, the Whirlpools' tiles and the maps by their Whirlpools; and
//! each class's most on one map and the most channels of each way on one map (the teleporters XL
//! map's numbers, `e08-teleporters-144.h3m`). Since E09 T19c2 every game that builds is played by
//! `EndTurn` alone (exit criterion 6 of the town epic's increment 2, design §32): a week, or on to
//! the last first day of its owned towns' events that fire for their owner (a repeating one to its
//! second firing); the town events fired, by day, town and place, and the buildings they built
//! checked against the census's own walk of TWN-EVT-001 and -003 from the towns' records before
//! their events, the events counted by why they fire or not and their buildings by what became of
//! each; and the fixed towns' events in the survey's order (`E09-construction.md` §10) checked
//! against its census (57 built, 30 skipped as standing, 27 without a requirement).
//!
//! `cargo run -p h3-map --example original_data -- original-data/install`
//!
//! The installation copy is only read. Without a `Maps` folder the check is skipped with a note.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

use std::sync::Arc;

use h3_adventure::{
    DWELLING_LEVELS, Holds, MapSetup, MonolithWay, ObjectKind, ObjectTypeId, Pos, SetupError,
    land_table_points,
};
use h3_content::{CreatureTypeId, Formation};
use h3_contracts::TownId;
use h3_game::{
    Actor, BuildingsSetup, DEFAULT_DIFFICULTY, FactionChoice, GameCommand, GameEvent, GameRuleset,
    GameSetup, GameSetupError, GameState, GarrisonCreature, PlayerSetup, TimedEventSetup,
    TownFaction, TownSetup,
};
use h3_map::model::{Artifacts, Body, Object, Player, RawMap, SeerTask, Text, Version};
use h3_map::{ImportError, Limits, gunzip, import, import_map};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};

/// The largest value of each bounded quantity over all maps, with the map it came from; how many
/// objects of each body kind the maps hold; how many tiles objects are visited from; each map
/// with stacked visitable tiles, with the classes on each such tile in map order; each map with
/// typed objects that share their type and anchor with an earlier one, with how many; the
/// monsters' and grounds' counts (E08 T20a2a); the dwellings the game's build gives (E08 T94a);
/// the dwellings' and the Refugee Camps' weeks it gives (E08 T94b); the teleporters (E08 T103b);
/// the channels and the gates' pairs the game's build gives (E08 T104).
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
    Teleporters,
    ChannelCensus,
    TownEvents,
    Increment12,
);

/// The census of the channels the game's build gives (E08 T104, [`built_channels`]; ADV-TEL-002,
/// increment 11's design note §5, survey `docs/rules/surveys/E08-teleporters.md` §1): its counts,
/// the one-way channels by their entrances and exits, the two-way channels by their size, and each
/// map with gates not paired with their nearest gate on the other level, with how many.
#[derive(Default)]
struct ChannelCensus(
    BTreeMap<&'static str, usize>,
    BTreeMap<(usize, usize), usize>,
    BTreeMap<usize, usize>,
    BTreeMap<String, usize>,
);

/// The survey's census of the shipped maps' gates (§1, by the original's rule, §3.2): the pairs,
/// the gates not paired with their nearest and the maps holding them. [`main`] checks the build's
/// against them.
const SURVEY_GATES: (usize, usize, usize) = (301, 18, 8);

/// The survey's census of the shipped maps' teleporters (§1; E08 T106c): the objects by class and
/// subclass, on the surface and underground. [`teleporter_differences`] checks the census against
/// it and the other `SURVEY_*` tables.
const SURVEY_TELEPORTERS: [((u32, u32), [usize; 2]); 26] = [
    ((43, 0), [39, 9]),
    ((43, 1), [25, 19]),
    ((43, 2), [20, 6]),
    ((43, 3), [5, 0]),
    ((43, 4), [34, 0]),
    ((43, 5), [19, 0]),
    ((43, 6), [4, 6]),
    ((43, 7), [7, 0]),
    ((44, 0), [30, 6]),
    ((44, 1), [30, 19]),
    ((44, 2), [17, 7]),
    ((44, 3), [3, 0]),
    ((44, 4), [18, 2]),
    ((44, 5), [13, 0]),
    ((44, 6), [4, 2]),
    ((44, 7), [5, 1]),
    ((45, 0), [128, 44]),
    ((45, 1), [71, 37]),
    ((45, 2), [69, 31]),
    ((45, 3), [18, 2]),
    ((45, 4), [31, 3]),
    ((45, 5), [17, 3]),
    ((45, 6), [20, 6]),
    ((45, 7), [10, 5]),
    ((103, 0), [301, 301]),
    ((111, 0), [215, 9]),
];
/// The survey's maps holding each class (§1).
const SURVEY_MAPS: [(u32, usize); 5] = [(43, 59), (44, 59), (45, 98), (103, 98), (111, 62)];
/// The survey's one-way channels (a map's entrances and exits of one channel) by their entrances
/// and exits (§1: 111 channels).
const SURVEY_ONE_WAY: [((usize, usize), usize); 14] = [
    ((1, 1), 71),
    ((1, 2), 1),
    ((1, 5), 1),
    ((1, 7), 1),
    ((2, 1), 8),
    ((2, 2), 4),
    ((3, 1), 7),
    ((3, 2), 2),
    ((3, 3), 8),
    ((4, 1), 1),
    ((4, 4), 3),
    ((5, 1), 1),
    ((7, 1), 1),
    ((8, 3), 2),
];
/// The survey's two-way channels by size (§1: 219 channels).
const SURVEY_TWO_WAY: [(usize, usize); 6] = [(2, 185), (3, 19), (4, 11), (5, 2), (6, 1), (8, 1)];
/// The survey's Whirlpools' tiles (§1: every Whirlpool's six on water).
const SURVEY_WHIRLPOOL_TILES: usize = 1_344;
/// The survey's maps by how many Whirlpools they hold (§1: 62 maps).
const SURVEY_WHIRLPOOLS_PER_MAP: [(usize, usize); 8] = [
    (2, 35),
    (3, 8),
    (4, 5),
    (5, 5),
    (6, 4),
    (7, 1),
    (8, 2),
    (19, 2),
];

/// The census of the teleporters (E08 T103b, [`teleporters`]; survey
/// `docs/rules/surveys/E08-teleporters.md` §1): the objects by class and subclass, on the surface
/// and underground, the maps holding each class, and (E08 T106c) the maps by how many Whirlpools
/// they hold.
#[derive(Default)]
struct Teleporters(
    BTreeMap<(u32, u32), [usize; 2]>,
    BTreeMap<u32, usize>,
    BTreeMap<usize, usize>,
);

/// The census of increment 12's objects (E08 T113b, [`increment_12`]; survey
/// `docs/rules/surveys/E08-garrisons-bonuses.md` §1): the objects by class and subclass, on the
/// surface and underground; the maps holding each class and subclass (the key `(class, u32::MAX)`
/// for a class of the fifteen, `(0, 0)` for any garrison, `(0, 1)` for any of the fifteen); the
/// garrisons' counts by owner, army and removable flag; each class's most on one map.
#[derive(Default)]
struct Increment12(
    BTreeMap<(u32, u32), [usize; 2]>,
    BTreeMap<(u32, u32), usize>,
    BTreeMap<&'static str, usize>,
    BTreeMap<u32, usize>,
);

/// The survey's garrisons by class and subclass, neutral and owned (§1.1: 238 on 61 maps), with
/// the maps holding each.
const SURVEY_GARRISONS: [((u32, u32), [usize; 2], usize); 4] = [
    ((33, 0), [109, 31], 42),
    ((33, 1), [23, 5], 21),
    ((219, 0), [50, 3], 23),
    ((219, 1), [17, 0], 4),
];
/// The survey's garrisons underground, by class (§1.1).
const SURVEY_GARRISONS_UNDERGROUND: [(u32, usize); 2] = [(33, 40), (219, 5)];
/// The survey's garrisons' armies and flags (§1.1): the census's keys ([`increment_12`]) with the
/// counts.
const SURVEY_GARRISON_COUNTS: [(&str, usize); 6] = [
    ("neutral garrisons empty", 82),
    ("neutral garrisons armed", 117),
    ("owned garrisons empty", 12),
    ("owned garrisons armed", 27),
    ("neutral garrisons not removable", 7),
    ("owned garrisons not removable", 4),
];
/// The survey's largest garrison stack (§1.1).
const SURVEY_LARGEST_GARRISON_STACK: usize = 200;
/// The survey's bonus visitables by class (§1.2: 4 085 on 159 maps), each its type's key: the
/// objects on the surface and underground, the maps holding them and the most on one map.
const SURVEY_BONUSES: [(u32, &str, [usize; 2], usize, usize); 15] = [
    (11, "base:buoy", [199, 10], 56, 17),
    (14, "base:swan_pond", [174, 9], 73, 8),
    (28, "base:faerie_ring", [237, 18], 102, 12),
    (30, "base:fountain_of_fortune", [354, 31], 115, 11),
    (31, "base:fountain_of_youth", [253, 19], 101, 13),
    (38, "base:idol_of_fortune", [446, 117], 133, 15),
    (48, "base:magic_spring", [66, 4], 39, 8),
    (49, "base:magic_well", [768, 147], 145, 32),
    (52, "base:mermaids", [129, 4], 59, 9),
    (56, "base:oasis", [51, 4], 37, 4),
    (64, "base:rally_flag", [271, 60], 91, 19),
    (92, "base:sirens", [103, 4], 57, 9),
    (94, "base:stables", [128, 15], 65, 10),
    (96, "base:temple", [354, 55], 117, 22),
    (110, "base:watering_hole", [48, 7], 43, 4),
];
/// The survey's most garrisons of each class on one map (§1.1).
const SURVEY_GARRISONS_MOST: [(u32, usize); 2] = [(33, 16), (219, 11)];
/// The survey's maps holding a garrison, and a bonus visitable (§1.1, §1.2).
const SURVEY_INCREMENT_12_MAPS: (usize, usize) = (61, 159);

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
    let teleporters = &peaks.8;
    println!("  teleporters, each typed as its class and subclass (checked):");
    for (class, maps) in &teleporters.1 {
        let of_class = || teleporters.0.range((*class, 0)..=(*class, u32::MAX));
        let subclasses: Vec<String> = of_class()
            .map(|((_, subclass), [above, below])| format!("{subclass}: {above} + {below}"))
            .collect();
        let objects: usize = of_class().map(|(_, levels)| levels[0] + levels[1]).sum();
        println!(
            "    class {class}: {objects} ({maps} maps); by subclass, surface + underground: {}",
            subclasses.join(", ")
        );
    }
    let per_map: Vec<String> = (teleporters.2.iter())
        .map(|(whirlpools, maps)| format!("{whirlpools} on {maps}"))
        .collect();
    println!("    maps by their Whirlpools: {}", per_map.join(", "));
    let channels = &peaks.9;
    println!("  the channels and the gates' pairs the game's build gives (checked):");
    for (what, n) in &channels.0 {
        println!("    {what}: {n}");
    }
    let one_way: Vec<String> = channels
        .1
        .iter()
        .map(|((entrances, exits), n)| format!("{entrances} → {exits}: {n}"))
        .collect();
    println!(
        "    one-way channels by entrances → exits: {}",
        one_way.join(", ")
    );
    let two_way: Vec<String> = channels
        .2
        .iter()
        .map(|(size, n)| format!("{size}: {n}"))
        .collect();
    println!("    two-way channels by size: {}", two_way.join(", "));
    let greedy: usize = channels.3.values().sum();
    println!(
        "    gates not paired with their nearest: {greedy} on {} maps",
        channels.3.len()
    );
    for (map, n) in &channels.3 {
        println!("      {map}: {n}");
    }
    let pairs = channels.0.get(PAIRS).copied().unwrap_or(0);
    if (pairs, greedy, channels.3.len()) != SURVEY_GATES {
        println!(
            "h3-map: the gates' pairs ({pairs}; {greedy} not with their nearest on {} maps) \
             are not the survey's census {SURVEY_GATES:?}",
            channels.3.len()
        );
        failures += 1;
    }
    // E08 T106c: the rest of the survey's census (§1).
    failures += teleporter_differences(&peaks);
    failures += increment_12_differences(&peaks.11);
    failures += town_event_differences(&peaks.10);
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
    // since E09 T8a1 one town record per town (`GameSetupError::TownCount`), since E09 T7b2a of
    // the first faction with a town record, which no player's start reads (a random town its
    // owner's start would: `GameSetupError::Choice`).
    let faction = rules
        .towns()
        .towns()
        .first()
        .map_or(TownFaction::Random { alignment: None }, |record| {
            TownFaction::Fixed(record.faction)
        });
    let town_records = static_map
        .objects
        .iter()
        .filter(|object| {
            object
                .object_type
                .and_then(|id| rules.adventure().object_type(id))
                .is_some_and(|ty| matches!(ty.kind, ObjectKind::Town { .. }))
        })
        .map(|_| TownSetup {
            faction,
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
    // E08 T104: the channels and the gates' pairs it builds (ADV-TEL-002); a map it refuses for
    // a teleporter off its surface (A-42) fails above.
    let (one_way, two_way) = built_channels(&built, rules, name, &mut peaks.9)?;
    peaks.note(ONE_WAY_CHANNELS, one_way, name);
    peaks.note(TWO_WAY_CHANNELS, two_way, name);
    most_dwellings(&map, &setup.map, peaks, name);
    let (visitable, stacked) = stacked(&map, &setup.map);
    peaks.2 += visitable;
    if !stacked.is_empty() {
        peaks.3.insert(name.to_owned(), stacked);
    }
    let (twins, unlike, other) = twins(&setup.map, rules);
    if twins > 0 {
        peaks.4.insert(name.to_owned(), twins);
    }
    // E08 T113b (decision 320, the server lane's question): a twin unlike the first, or of a kind
    // something names (not a ground, a ground of the sea or a hole), the setup refuses.
    *peaks.5.entry(TWINS_UNLIKE).or_default() += unlike;
    *peaks.5.entry(TWINS_NAMED).or_default() += other;
    census(&map, &setup.map, rules, &mut peaks.5);
    for (class, n) in teleporters(&map, &setup.map, rules, &mut peaks.8)? {
        peaks.note(most_of(class), n, name);
    }
    increment_12(&map, &setup.map, rules, &mut peaks.11)?;
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
            built_towns(&setup, &game, rules, &mut peaks.5).map_err(|e| format!("{name}: {e}"))?;
            built_linked(&setup, &game, rules, name, &mut peaks.5)?;
            built_heroes(&setup, &game, rules, map.header.size, &mut peaks.5);
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
            // E09 T19c2: the town events played by `EndTurn` alone (exit criterion 6).
            played_town_events(&setup, game, rules, &mut peaks.10)
                .map_err(|e| format!("played: {e}"))?;
            IMPORTED.to_owned()
        }
        Err(ImportError::Unsupported(what)) => format!("not supported: {}", what.reason()),
        Err(e) => return Err(format!("import: {e}")),
    };
    Ok((map.header.version, outcome))
}

/// E09 T11: the random dwellings tied to a town in a game that builds (ADV-DWL-005): each the
/// table's dwelling of its town's faction as the build resolved it (a random town's drawn, or its
/// owner's), at a level of its range, counted by what gave the town its faction. Checked (an error
/// fails the map).
fn built_linked(
    setup: &GameSetup,
    game: &GameState,
    rules: &GameRuleset,
    name: &str,
    counts: &mut BTreeMap<&'static str, usize>,
) -> Result<(), String> {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    let adventure = game.adventure();
    for (index, object) in setup.map.objects.iter().enumerate() {
        let Holds::LinkedDwelling {
            town,
            levels: (low, high),
        } = object.holds
        else {
            continue;
        };
        let faction = game.towns().get(town).map(h3_towns::Town::faction);
        let row = faction.and_then(|faction| rules.adventure().random_dwelling(faction));
        let resolved = adventure
            .map()
            .objects()
            .get(index)
            .and_then(|o| o.object_type);
        let level = row.and_then(|row| {
            (low..=high).find(|&level| row.get(usize::from(level) - 1).copied() == resolved)
        });
        if level.is_none() {
            return Err(format!(
                "{name}: the random dwelling {index} is not its town {}'s table's at a level of \
                 its range",
                town.raw()
            ));
        }
        add("E09 built: random dwellings tied to a town resolved", 1);
        let owned = || {
            adventure
                .owner(adventure.map().towns()[town.index()])
                .is_some()
        };
        let by = match setup.towns.get(town.index()).map(|t| t.faction) {
            Some(TownFaction::Random { alignment: Some(_) }) => {
                "E09 built: tied to a random town by its alignment"
            }
            Some(TownFaction::Random { alignment: None }) if owned() => {
                "E09 built: tied to a random town by its owner"
            }
            Some(TownFaction::Random { alignment: None }) => {
                "E09 built: tied to a random town drawn uniformly"
            }
            _ => "E09 built: tied to a fixed town",
        };
        add(by, 1);
    }
    Ok(())
}

/// E09 T7b2a: the towns of a game that builds as the build resolved them (TWN-SET-002 to -004,
/// TWN-CAP-001): each random town by what gave its faction, the neutral towns' drawn garrisons,
/// the owned towns whose start reveals more than the part's radius; since E09 T18a their spots
/// and Shipyards ([`built_town_water`]).
fn built_towns(
    setup: &GameSetup,
    game: &GameState,
    rules: &GameRuleset,
    counts: &mut BTreeMap<&'static str, usize>,
) -> Result<(), String> {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    let adventure = game.adventure();
    let objects = adventure.map().towns();
    for ((town, &object), index) in setup.towns.iter().zip(objects).zip(0u16..) {
        let owner = adventure.owner(object);
        add("E09 built: towns", 1);
        if let TownFaction::Random { alignment } = town.faction {
            let by = match (alignment, owner) {
                (Some(_), _) => "E09 built: random towns by their alignment",
                (None, Some(_)) => "E09 built: random towns by their owner",
                (None, None) => "E09 built: random towns drawn uniformly",
            };
            add(by, 1);
        }
        if owner.is_none() && town.garrison.is_none() {
            add("E09 built: neutral towns with a drawn garrison", 1);
            let empty = adventure.town_garrison(object).is_some_and(<[_]>::is_empty);
            add(
                "E09 built: neutral towns with a drawn garrison, empty",
                usize::from(empty),
            );
        }
        if let Some(stored) = game.towns().get(TownId::new(index)) {
            built_town_facts(stored, owner.is_some(), rules.towns(), &mut add);
            built_town_water(town, stored, game, index, rules, &mut add)?;
        }
        let built = game
            .towns()
            .get(TownId::new(index))
            .and_then(|t| rules.towns().town(t.faction()).map(|r| (t.built(), r)));
        if let (Some(_), Some((built, record))) = (owner, built) {
            let grails: Vec<bool> = built
                .iter()
                .filter_map(|b| record.building(b))
                .filter(|b| b.roles.reveal.is_some())
                .map(|b| b.roles.grail)
                .collect();
            let all = grails.contains(&true);
            add(
                "E09 built: owned towns revealing a larger radius at the start",
                usize::from(!grails.is_empty() && !all),
            );
            add(
                "E09 built: owned towns revealing every tile at the start",
                usize::from(all),
            );
            add(
                "E09 built: owned towns revealing every tile, two levels",
                usize::from(all && setup.map.levels > 1),
            );
        }
    }
    Ok(())
}

/// E09 T18a (TWN-SET-011, design §28; survey `E09-towns.md` §5.3): a built town's spot and its
/// Shipyard: the towns with a spot, those of a faction with a Shipyard without one, the custom
/// sets that build a Shipyard and those that lose it for want of water. Checked (an error fails
/// the map): a town without a spot has its faction's Shipyard forbidden and never built, one with
/// a spot keeps a custom set's.
fn built_town_water(
    setup: &TownSetup,
    town: &h3_towns::Town,
    game: &GameState,
    index: u16,
    rules: &GameRuleset,
    add: &mut impl FnMut(&'static str, usize),
) -> Result<(), String> {
    let id = TownId::new(index);
    let spot = game.adventure().map().town_boat_spot(id);
    add("E09 built: towns with a spot", usize::from(spot.is_some()));
    let Some(record) = rules.towns().town(town.faction()) else {
        return Ok(());
    };
    let shipyards = (0..h3_towns::MAX_BUILDINGS)
        .filter_map(h3_towns::BuildingId::from_index)
        .filter(|&b| record.building(b).is_some_and(|b| b.roles.shipyard))
        .fold(h3_towns::BuildingSet::EMPTY, |set, b| set.with(b));
    if shipyards.is_empty() {
        return Ok(());
    }
    add("E09 built: towns of a faction with a Shipyard", 1);
    // Whether `set` holds a Shipyard.
    let meets = |set: h3_towns::BuildingSet| set.difference(shipyards) != set;
    let picked = match setup.buildings {
        BuildingsSetup::Custom { built, forbidden } => {
            meets(built.buildings(record)) && !meets(forbidden.buildings(record))
        }
        BuildingsSetup::Standard { .. } => false,
    };
    add(
        "E09 built: custom sets building a Shipyard",
        usize::from(picked),
    );
    let kept = meets(town.built());
    let forbidden = shipyards.difference(town.forbidden()).is_empty();
    if spot.is_none() {
        add("E09 built: towns of a faction with a Shipyard, no spot", 1);
        add(
            "E09 built: custom sets losing a built Shipyard, no spot",
            usize::from(picked),
        );
        if kept || !forbidden {
            return Err(format!(
                "town {index} has no spot but its Shipyard is not forbidden or still built"
            ));
        }
    } else if picked && !kept {
        return Err(format!(
            "town {index} has a spot {spot:?} but lost its custom set's Shipyard"
        ));
    }
    Ok(())
}

/// E09 T9c: a built town's facts for a battle at it (TWN-SIE-003: its fort level, a moat) and its
/// hall level (TWN-INC-001); for an owned one the day's income it pays from day 2 (TWN-INC-001,
/// -002): its gold, summed, and whether it pays another resource (a Resource Silo).
fn built_town_facts(
    town: &h3_towns::Town,
    owned: bool,
    part: &h3_towns::TownRuleset,
    add: &mut impl FnMut(&'static str, usize),
) {
    let siege = h3_towns::siege(town, part);
    add(
        match siege.fort {
            h3_contracts::FortLevel::None => "E09 built: towns by fort level: none",
            h3_contracts::FortLevel::Fort => "E09 built: towns by fort level: Fort",
            h3_contracts::FortLevel::Citadel => "E09 built: towns by fort level: Citadel",
            h3_contracts::FortLevel::Castle => "E09 built: towns by fort level: Castle",
        },
        1,
    );
    add(
        "E09 built: towns with a moat",
        usize::from(siege.moat != h3_towns::Moat::Absent),
    );
    add(
        match h3_towns::hall_level(town, part) {
            1 => "E09 built: towns by hall level: 1 (Village Hall)",
            2 => "E09 built: towns by hall level: 2 (Town Hall)",
            3 => "E09 built: towns by hall level: 3 (City Hall)",
            _ => "E09 built: towns by hall level: 4 (Capitol)",
        },
        1,
    );
    if owned {
        let income = h3_towns::daily_income(town, part);
        let gold = part.params().map(|p| p.gold);
        let paid = |id| {
            income
                .iter()
                .find(|&&(r, _)| Some(r) == id)
                .map_or(0, |&(_, n)| n)
        };
        add(
            "E09 built: owned towns' day's income, gold summed",
            usize::try_from(paid(gold)).unwrap_or(usize::MAX),
        );
        add(
            "E09 built: owned towns paying a resource other than gold",
            usize::from(income.iter().any(|&(r, _)| Some(r) != gold)),
        );
    }
}

/// E09 T7b2b: the main towns' heroes of a game that builds (TWN-SET-009), the heroes after the
/// setup's: by a human player's pick or drawn, those whose type starts with a spell book in a
/// main town with a Mage Guild built (the original teaches them the guild's spells at the start,
/// `TWN-STUB-6`), and those on a map of 36 tiles a side.
fn built_heroes(
    setup: &GameSetup,
    game: &GameState,
    rules: &GameRuleset,
    size: u32,
    counts: &mut BTreeMap<&'static str, usize>,
) {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    let generated = game.heroes().iter().skip(setup.heroes.len());
    for (details, index) in generated.zip(setup.heroes.len()..) {
        let id = h3_contracts::HeroId::new(u16::try_from(index).unwrap_or(u16::MAX));
        let Some(owner) = game.adventure().hero(id).map(|hero| hero.owner) else {
            continue;
        };
        let player = &setup.players[owner.index()];
        add("E09 built: main towns' heroes", 1);
        let picked = player.human && player.start.pick == Some(details.hero_type);
        add(
            "E09 built: main towns' heroes by a human pick",
            usize::from(picked),
        );
        add("E09 built: main towns' heroes drawn", usize::from(!picked));
        add(
            "E09 built: main towns' heroes on a map of 36 tiles a side",
            usize::from(size == 36),
        );
        let book = rules
            .core()
            .hero_type(details.hero_type)
            .is_some_and(|ty| ty.spell_book);
        let guild = player.start.main_town.and_then(|main| {
            let town = game.towns().get(main.town)?;
            let record = rules.towns().town(town.faction())?;
            Some(
                town.built()
                    .iter()
                    .filter_map(|b| record.building(b))
                    .any(|b| b.key.starts_with("mage_guild")),
            )
        });
        add(
            "E09 built: main towns' heroes with a spell book in a town with a Mage Guild",
            usize::from(book && guild == Some(true)),
        );
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
/// by the terrain alone: TWN-SET-011's case, the built game's census in [`built_town_water`]),
/// custom garrisons (with random stacks), names and events; the players'
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
    // The towns' owners at the start (E09 T7b2a: a random town takes its owner's faction).
    let owners: Vec<_> = setup
        .map
        .objects
        .iter()
        .filter(|object| {
            object
                .object_type
                .and_then(|id| rules.adventure().object_type(id))
                .is_some_and(|ty| matches!(ty.kind, ObjectKind::Town { .. }))
        })
        .map(|object| object.owner)
        .collect();
    add("E09 maps that import", 1);
    for ((town, &(x, y, level)), owner) in setup.towns.iter().zip(&entrances).zip(&owners) {
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
                add(
                    "E09 towns: class 77 without an alignment, owned at the start",
                    usize::from(alignment.is_none() && owner.is_some()),
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
            // E09 T7b2a: a building that reveals at the start (TWN-CAP-001), the Grail's every
            // tile.
            let reveals = |grail: bool| {
                candidates.iter().any(|&f| {
                    records.town(f).is_some_and(|record| {
                        record.buildings.iter().any(|b| {
                            b.roles.reveal.is_some()
                                && b.roles.grail == grail
                                && b.slot.is_some_and(|slot| built.contains(slot))
                        })
                    })
                })
            };
            add(
                "E09 towns: a custom set with a building revealing a radius built",
                usize::from(reveals(false)),
            );
            add(
                "E09 towns: a custom set with a building revealing every tile built",
                usize::from(reveals(true)),
            );
            let wet = water(x.wrapping_sub(1), y + 2, level) || water(x + 1, y + 2, level);
            add(
                "E09 towns: a built Shipyard without water by the terrain (TWN-SET-011)",
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
    dwellings(map, setup, rules, &mut add);
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

/// Counts the teleporters (E08 T103b; ADV-TEL-001, ADV-OBJ-002, increment 11's design note §3 and
/// §4) into the census. Checked (an error fails the map): each object of classes 43 to 45, 103 and
/// 111 imports as its class and subclass's type, a monolith of its class's way (43 an entrance, 44
/// an exit, 45 two-way) with its subclass as the channel, 103 the Subterranean Gate, 111 the
/// Whirlpool. Returns the map's objects by class (E08 T106c: each class's most on one map, and the
/// maps by their Whirlpools in the census).
fn teleporters(
    raw: &RawMap,
    setup: &MapSetup,
    rules: &GameRuleset,
    census: &mut Teleporters,
) -> Result<BTreeMap<u32, usize>, String> {
    let mut classes = BTreeMap::<u32, usize>::new();
    for ((placed, class), object) in placed(raw).zip(&setup.objects) {
        let way = match class {
            43 => Some(MonolithWay::Entrance),
            44 => Some(MonolithWay::Exit),
            45 => Some(MonolithWay::TwoWay),
            103 | 111 => None,
            _ => continue,
        };
        let subclass = usize::try_from(placed.template)
            .ok()
            .and_then(|i| raw.templates.get(i))
            .map_or(u32::MAX, |t| t.subclass);
        let kind = object
            .object_type
            .and_then(|id| rules.adventure().object_type(id))
            .map(|ty| &ty.kind);
        let typed = match kind {
            Some(ObjectKind::Monolith { channel, way: w }) => {
                way == Some(*w) && u32::from(*channel) == subclass
            }
            Some(ObjectKind::SubterraneanGate) => class == 103,
            Some(ObjectKind::Whirlpool { .. }) => class == 111,
            _ => false,
        };
        if !typed {
            return Err(format!(
                "a teleporter of class {class} subclass {subclass} at {:?} imports as {kind:?}",
                object.anchor
            ));
        }
        census.0.entry((class, subclass)).or_default()[usize::from(object.anchor.level > 0)] += 1;
        *classes.entry(class).or_default() += 1;
    }
    for &class in classes.keys() {
        *census.1.entry(class).or_default() += 1;
    }
    if let Some(&whirlpools) = classes.get(&111) {
        *census.2.entry(whirlpools).or_default() += 1;
    }
    Ok(classes)
}

/// Counts increment 12's objects (E08 T113b; ADV-GAR-001, ADV-BON-001, increment 12's design note
/// §4) into the census. Checked (an error fails the map): each object of classes 33 and 219
/// imports as its class and subclass's garrison (anti-magic for subclass 1, 219's the AB pack's
/// vertical ones) holding its troops and removable flag, an owned one its owner; each of the
/// fifteen bonus classes as its type, holding nothing.
fn increment_12(
    raw: &RawMap,
    setup: &MapSetup,
    rules: &GameRuleset,
    census: &mut Increment12,
) -> Result<(), String> {
    let part = rules.adventure();
    let mut classes = BTreeMap::<(u32, u32), usize>::new();
    for ((placed, class), object) in placed(raw).zip(&setup.objects) {
        let bonus = SURVEY_BONUSES.iter().find(|b| b.0 == class);
        if !matches!(class, 33 | 219) && bonus.is_none() {
            continue;
        }
        let subclass = usize::try_from(placed.template)
            .ok()
            .and_then(|i| raw.templates.get(i))
            .map_or(u32::MAX, |t| t.subclass);
        let ty = object.object_type.and_then(|id| part.object_type(id));
        let key = ty.map_or("", |ty| ty.key.as_str());
        let expected = match (class, subclass, bonus) {
            (33, 0, _) => "base:garrison",
            (33, 1, _) => "base:anti_magic_garrison",
            (219, 0, _) => "ab:garrison_vertical",
            (219, 1, _) => "ab:anti_magic_garrison_vertical",
            (_, 0, Some(bonus)) => bonus.1,
            _ => "",
        };
        let fits = match (ty.map(|ty| &ty.kind), &object.holds) {
            (Some(ObjectKind::Garrison { anti_magic }), Holds::Garrison { .. }) => {
                *anti_magic == (subclass == 1)
            }
            (Some(_), Holds::Nothing) => bonus.is_some() && object.owner.is_none(),
            _ => false,
        };
        if key != expected || !fits {
            return Err(format!(
                "an object of class {class} subclass {subclass} at {:?} imports as {key:?} \
                 holding {:?}",
                object.anchor, object.holds
            ));
        }
        let level = usize::from(object.anchor.level > 0);
        census.0.entry((class, subclass)).or_default()[level] += 1;
        *classes.entry((class, subclass)).or_default() += 1;
        if let Holds::Garrison { army, removable } = &object.holds {
            let owner = if object.owner.is_some() { "owned" } else { "neutral" };
            let mut add = |what: &str| {
                let key = match (owner, what) {
                    ("owned", "empty") => "owned garrisons empty",
                    ("owned", "armed") => "owned garrisons armed",
                    ("owned", _) => "owned garrisons not removable",
                    (_, "empty") => "neutral garrisons empty",
                    (_, "armed") => "neutral garrisons armed",
                    _ => "neutral garrisons not removable",
                };
                *census.2.entry(key).or_default() += 1;
            };
            add(if army.is_empty() { "empty" } else { "armed" });
            if !removable {
                add("fixed");
            }
            let owned = usize::from(object.owner.is_some());
            *census.2.entry(GARRISON_OWNERS[owned]).or_default() += 1;
            let largest = army.iter().map(|s| s.count).max().unwrap_or(0);
            let peak = census.2.entry(LARGEST_GARRISON_STACK).or_default();
            *peak = (*peak).max(usize::try_from(largest).unwrap_or(usize::MAX));
            let by = match (class, subclass, owned) {
                (33, 0, 0) => "neutral 33.0",
                (33, 0, _) => "owned 33.0",
                (33, _, 0) => "neutral 33.1",
                (33, _, _) => "owned 33.1",
                (_, 0, 0) => "neutral 219.0",
                (_, 0, _) => "owned 219.0",
                (_, _, 0) => "neutral 219.1",
                _ => "owned 219.1",
            };
            *census.2.entry(by).or_default() += 1;
        }
    }
    let mut most = BTreeMap::<u32, usize>::new();
    for (&(class, subclass), &n) in &classes {
        *census.1.entry((class, subclass)).or_default() += 1;
        *most.entry(class).or_default() += n;
    }
    for (&class, &n) in &most {
        let peak = census.3.entry(class).or_default();
        *peak = (*peak).max(n);
    }
    if classes.keys().any(|&(class, _)| matches!(class, 33 | 219)) {
        *census.1.entry((0, 0)).or_default() += 1;
    }
    if classes.keys().any(|&(class, _)| !matches!(class, 33 | 219)) {
        *census.1.entry((0, 1)).or_default() += 1;
    }
    Ok(())
}

/// The census's keys of the garrisons by owner, neutral then owned ([`increment_12`]).
const GARRISON_OWNERS: [&str; 2] = ["neutral garrisons", "owned garrisons"];
/// The census's key of the largest garrison stack ([`increment_12`]).
const LARGEST_GARRISON_STACK: &str = "largest garrison stack";

/// Increment 12's census against the survey's (§1; E08 T113b): the garrisons by class, subclass,
/// owner and level, their maps, armies and flags; the bonus visitables by class and level, their
/// maps and each class's most on one map. Prints each difference and the census; returns how many
/// differences there are.
fn increment_12_differences(census: &Increment12) -> usize {
    let mut differences = 0;
    let mut differ = |what: &str, found: usize, survey: usize| {
        if found != survey {
            println!("h3-map: the census's {what} ({found}) is not the survey's ({survey})");
            differences += 1;
        }
    };
    let get = |key: &str| census.2.get(key).copied().unwrap_or(0);
    for ((class, subclass), [neutral, owned], maps) in SURVEY_GARRISONS {
        let both = census.0.get(&(class, subclass)).copied().unwrap_or([0; 2]);
        let what = format!("{class}.{subclass}");
        differ(
            &format!("{what} garrisons"),
            both[0] + both[1],
            neutral + owned,
        );
        differ(
            &format!("neutral {what} garrisons"),
            get(&format!("neutral {what}")),
            neutral,
        );
        differ(
            &format!("owned {what} garrisons"),
            get(&format!("owned {what}")),
            owned,
        );
        let found = census.1.get(&(class, subclass)).copied().unwrap_or(0);
        differ(&format!("maps with {what} garrisons"), found, maps);
    }
    for (class, underground) in SURVEY_GARRISONS_UNDERGROUND {
        let found = census
            .0
            .range((class, 0)..=(class, u32::MAX))
            .map(|(_, n)| n[1])
            .sum();
        differ(&format!("class {class} underground"), found, underground);
    }
    for (what, n) in SURVEY_GARRISON_COUNTS {
        differ(what, get(what), n);
    }
    differ(
        LARGEST_GARRISON_STACK,
        get(LARGEST_GARRISON_STACK),
        SURVEY_LARGEST_GARRISON_STACK,
    );
    for (class, most) in SURVEY_GARRISONS_MOST {
        let found = census.3.get(&class).copied().unwrap_or(0);
        differ(&format!("most of class {class} on one map"), found, most);
    }
    for (class, key, levels, maps, most) in SURVEY_BONUSES {
        let found = census.0.get(&(class, 0)).copied().unwrap_or([0; 2]);
        differ(&format!("{key} on the surface"), found[0], levels[0]);
        differ(&format!("{key} underground"), found[1], levels[1]);
        let on = census.1.get(&(class, 0)).copied().unwrap_or(0);
        differ(&format!("maps with {key}"), on, maps);
        let peak = census.3.get(&class).copied().unwrap_or(0);
        differ(&format!("most {key} on one map"), peak, most);
    }
    let maps = |key| census.1.get(&key).copied().unwrap_or(0);
    differ(
        "maps with a garrison",
        maps((0, 0)),
        SURVEY_INCREMENT_12_MAPS.0,
    );
    differ(
        "maps with a bonus visitable",
        maps((0, 1)),
        SURVEY_INCREMENT_12_MAPS.1,
    );
    let objects = |garrisons: bool| -> usize {
        census
            .0
            .iter()
            .filter(|((class, _), _)| matches!(class, 33 | 219) == garrisons)
            .map(|(_, n)| n[0] + n[1])
            .sum()
    };
    println!(
        "  increment 12 (checked): {} garrisons on {} maps, {} bonus visitables on {} maps; \
         garrisons {} neutral, {} owned, largest stack {}",
        objects(true),
        maps((0, 0)),
        objects(false),
        maps((0, 1)),
        get(GARRISON_OWNERS[0]),
        get(GARRISON_OWNERS[1]),
        get(LARGEST_GARRISON_STACK),
    );
    differences
}

/// The census's key for a teleporter class's most on one map (E08 T106c, [`teleporters`]).
fn most_of(class: u32) -> &'static str {
    match class {
        43 => "one-way entrances on one map",
        44 => "one-way exits on one map",
        45 => "two-way monoliths on one map",
        103 => "Subterranean Gates on one map",
        _ => "Whirlpools on one map",
    }
}

/// The census's keys for the most channels of each way on one map (E08 T106c,
/// [`built_channels`]).
const ONE_WAY_CHANNELS: &str = "one-way channels on one map";
const TWO_WAY_CHANNELS: &str = "two-way channels on one map";

/// The survey's most of each teleporter class on one map, and the most channels of each way on one
/// map (§1 as amended by E08 T106c; the teleporters XL map's numbers): the census's keys
/// ([`most_of`], [`ONE_WAY_CHANNELS`], [`TWO_WAY_CHANNELS`]) with the counts.
const SURVEY_MOST: [(&str, usize); 7] = [
    ("one-way entrances on one map", 8),
    ("one-way exits on one map", 8),
    ("two-way monoliths on one map", 17),
    ("Subterranean Gates on one map", 26),
    ("Whirlpools on one map", 19),
    (ONE_WAY_CHANNELS, 4),
    (TWO_WAY_CHANNELS, 6),
];

/// The teleporters' census against the survey's (§1; E08 T106c): the objects by class, subclass
/// and level ([`SURVEY_TELEPORTERS`]), the maps with each class, the one-way channels by entrances
/// and exits, the two-way ones by size, the Whirlpools' tiles, the maps by their Whirlpools and
/// each class's and way's most on one map. Prints each difference; returns how many there are.
fn teleporter_differences(peaks: &Peaks) -> usize {
    let mut differences = 0;
    let mut differ = |what: &str, census: String, survey: String| {
        if census != survey {
            println!("h3-map: the census's {what} ({census}) are not the survey's ({survey})");
            differences += 1;
        }
    };
    let (teleporters, channels) = (&peaks.8, &peaks.9);
    let listed = |pairs: Vec<String>| pairs.join(", ");
    differ(
        "teleporters by class and subclass",
        listed(
            teleporters
                .0
                .iter()
                .map(|(k, v)| format!("{k:?} {v:?}"))
                .collect(),
        ),
        listed(
            SURVEY_TELEPORTERS
                .iter()
                .map(|(k, v)| format!("{k:?} {v:?}"))
                .collect(),
        ),
    );
    differ(
        "maps by class",
        listed(
            teleporters
                .1
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect(),
        ),
        listed(
            SURVEY_MAPS
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect(),
        ),
    );
    differ(
        "one-way channels by entrances and exits",
        listed(
            channels
                .1
                .iter()
                .map(|(k, v)| format!("{k:?} {v}"))
                .collect(),
        ),
        listed(
            SURVEY_ONE_WAY
                .iter()
                .map(|(k, v)| format!("{k:?} {v}"))
                .collect(),
        ),
    );
    differ(
        "two-way channels by size",
        listed(channels.2.iter().map(|(k, v)| format!("{k} {v}")).collect()),
        listed(
            SURVEY_TWO_WAY
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect(),
        ),
    );
    differ(
        "Whirlpools' tiles",
        channels
            .0
            .get("Whirlpools' tiles listed")
            .copied()
            .unwrap_or(0)
            .to_string(),
        SURVEY_WHIRLPOOL_TILES.to_string(),
    );
    differ(
        "maps by their Whirlpools",
        listed(
            teleporters
                .2
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect(),
        ),
        listed(
            SURVEY_WHIRLPOOLS_PER_MAP
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect(),
        ),
    );
    let most = |what: &str| peaks.0.get(what).map_or(0, |(n, _)| *n);
    differ(
        "most on one map",
        listed(
            SURVEY_MOST
                .iter()
                .map(|(k, _)| format!("{k} {}", most(k)))
                .collect(),
        ),
        listed(
            SURVEY_MOST
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect(),
        ),
    );
    differences
}

/// The census's key for the gates' pairs ([`built_channels`]).
const PAIRS: &str = "gates' pairs";

/// Counts the channels and the gates' pairs the game's build gives (E08 T104; ADV-TEL-002,
/// increment 11's design note §5) into the census: the one-way channels by their entrances (from
/// the objects: an entrance is in no list) and exits, the two-way channels by size, the Whirlpools'
/// tiles, the gates' entries, pairs and those not paired with their nearest gate on another level
/// by the truncated distance (the greedy order, survey §1). Checked (an error fails the map): every
/// visitable tile of an exit, a two-way monolith, a gate and a Whirlpool is listed once (no
/// shipped entrance is another object's, none lies off the map: survey §1), every pair is mutual,
/// and every gate is paired (the shipped maps' gates are as many on each level as on the other).
/// Returns the map's one-way and two-way channels (E08 T106c: the most of each on one map).
fn built_channels(
    built: &GameState,
    rules: &GameRuleset,
    name: &str,
    census: &mut ChannelCensus,
) -> Result<(usize, usize), String> {
    let map = built.adventure().map();
    let channels = map.channels();
    let mut entrances = BTreeMap::<u8, usize>::new();
    let mut visitable = 0;
    for object in map.objects() {
        let kind = object
            .object_type
            .and_then(|id| rules.adventure().object_type(id))
            .map(|ty| &ty.kind);
        match kind {
            Some(ObjectKind::Monolith {
                channel,
                way: MonolithWay::Entrance,
            }) => *entrances.entry(*channel).or_default() += 1,
            Some(
                ObjectKind::Monolith { .. }
                | ObjectKind::SubterraneanGate
                | ObjectKind::Whirlpool { .. },
            ) => visitable += object.visitable.offsets().count(),
            _ => {}
        }
    }
    let exits: usize = channels.one_way_channels().map(|(_, e)| e.len()).sum();
    let members: usize = channels.two_way_channels().map(|(_, m)| m.len()).sum();
    let (whirlpools, gates) = (channels.whirlpools().len(), channels.gates().len());
    if exits + members + whirlpools + gates != visitable {
        return Err(format!(
            "the channels list {} entries of {visitable} visitable tiles",
            exits + members + whirlpools + gates
        ));
    }
    let numbers: BTreeSet<u8> = entrances
        .keys()
        .copied()
        .chain(channels.one_way_channels().map(|(channel, _)| channel))
        .collect();
    let one_way = numbers.len();
    for channel in numbers {
        let ends = (
            entrances.get(&channel).copied().unwrap_or(0),
            channels.exits(channel).len(),
        );
        *census.1.entry(ends).or_default() += 1;
    }
    for (_, members) in channels.two_way_channels() {
        *census.2.entry(members.len()).or_default() += 1;
    }
    let distance = |a: Pos, b: Pos| {
        let (dx, dy) = (u64::from(a.x.abs_diff(b.x)), u64::from(a.y.abs_diff(b.y)));
        (dx * dx + dy * dy).isqrt()
    };
    let mut greedy = 0;
    for gate in channels.gates() {
        let Some(pair) = gate.pair else {
            return Err(format!("the gate at {:?} has no pair", gate.tile));
        };
        if channels.gate_pair(pair) != Some(gate.tile) {
            return Err(format!("the gate at {:?}'s pair is not mutual", gate.tile));
        }
        let nearest = channels
            .gates()
            .iter()
            .filter(|other| other.tile.level != gate.tile.level)
            .map(|other| distance(gate.tile, other.tile))
            .min();
        greedy += usize::from(nearest < Some(distance(gate.tile, pair)));
    }
    if greedy > 0 {
        census.3.insert(name.to_owned(), greedy);
    }
    let counts = &mut census.0;
    let mut add = |what, n| *counts.entry(what).or_default() += n;
    add("one-way exits listed", exits);
    add("two-way members listed", members);
    add("Whirlpools' tiles listed", whirlpools);
    add("gates' entries listed", gates);
    add(PAIRS, gates / 2);
    add("maps with channels", usize::from(!channels.is_empty()));
    Ok((one_way, channels.two_way_channels().count()))
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
/// type's shift, its entrance (the first tile it is visited from) on the map (E08 T96b2).
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
        // E09 T11: one tied to a town takes the town's faction, here the static layer's (the
        // first faction with a town record).
        let town_faction = |town| {
            game.towns()
                .get(town)
                .map(h3_towns::Town::faction)
                .into_iter()
                .collect()
        };
        let (factions, low, high): (Vec<_>, _, _) = match &object.holds {
            Holds::RandomDwelling {
                factions,
                levels: (low, high),
            } => (factions.clone(), low, high),
            Holds::LinkedDwelling {
                town,
                levels: (low, high),
            } => {
                add("random dwellings tied to a town resolved", 1);
                (town_faction(*town), low, high)
            }
            _ => {
                if (216..=218).contains(&class) && object.object_type.is_some() {
                    return Err(at("holds no set and range"));
                }
                add(
                    "dwellings placed by the map",
                    usize::from(map.dwelling_index(index).is_some()),
                );
                continue;
            }
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
        // E08 T96b2 (increment 10's design note §14): visited from a tile of the map.
        if resolved.entrance(map).is_none() {
            return Err(at("has no entrance on the map"));
        }
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

/// Notes the map's typed objects of increment 10's classes as peaks (E08 T96b2, the most on one
/// map that the dwellings XL holds, increment 10's design note §14): the dwellings of class 17 and
/// of class 20, the random dwellings of 216 to 218 (since E09 T11 those tied to a town typed too)
/// and the Refugee Camps.
fn most_dwellings(raw: &RawMap, setup: &MapSetup, peaks: &mut Peaks, name: &str) {
    let mut counts = [0; 4];
    for ((_, class), object) in placed(raw).zip(&setup.objects) {
        let slot = match class {
            17 => 0,
            20 => 1,
            216..=218 => 2,
            78 => 3,
            _ => continue,
        };
        counts[slot] += usize::from(object.object_type.is_some());
    }
    let what = [
        "dwellings of class 17 on a map",
        "dwellings of class 20 on a map",
        "random dwellings typed on a map",
        "Refugee Camps on a map",
    ];
    for (what, n) in what.into_iter().zip(counts) {
        peaks.note(what, n, name);
    }
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
            ObjectKind::Town { .. } => {
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
/// tied to a town, holding it since E09 T11 (map import v20; ADV-STUB-13 before), with those tied
/// to a random town (the survey: about 1 397, all to random towns), and the identifiers more than
/// one town has (the import takes the last town's); since E08 T97b1 the typed ones entered from
/// water (imported since map import v19, ADV-OWN-002).
fn dwellings(
    map: &RawMap,
    setup: &MapSetup,
    rules: &GameRuleset,
    add: &mut impl FnMut(&'static str, usize),
) {
    let random_towns: BTreeSet<u32> = placed(map)
        .filter(|&(_, class)| class == 77)
        .filter_map(|(object, _)| match &object.body {
            Body::Town(town) => town.identifier,
            _ => None,
        })
        .collect();
    let mut shared: BTreeMap<u32, usize> = BTreeMap::new();
    for (object, _) in placed(map).filter(|&(_, class)| matches!(class, 77 | 98)) {
        if let Body::Town(town) = &object.body
            && let Some(identifier) = town.identifier
        {
            *shared.entry(identifier).or_default() += 1;
        }
    }
    add(
        "towns: identifiers more than one town has (E09 T11)",
        shared.values().filter(|&&n| n > 1).count(),
    );
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
        let afloat = entry.visitable.offsets().any(|(column, row)| {
            let (Some(x), Some(y)) = (
                entry.anchor.x.checked_sub(u16::from(column)),
                entry.anchor.y.checked_sub(u16::from(row)),
            ) else {
                return false;
            };
            let index = (usize::from(entry.anchor.level) * usize::from(setup.height)
                + usize::from(y))
                * usize::from(setup.width)
                + usize::from(x);
            setup.tiles.get(index).is_some_and(|t| {
                matches!(
                    rules.adventure().passage(t.terrain),
                    Some(h3_adventure::Passage::Water(_))
                )
            })
        });
        add(
            "dwellings: typed ones entered from water (E08 T97b1)",
            usize::from(typed && afloat),
        );
        let Body::RandomDwelling(body) = &object.body else {
            continue;
        };
        match (&body.faction, &entry.holds) {
            (Some((0, _)), _) => add("dwellings: random dwellings with a faction set", 1),
            (Some((town, _)), Holds::LinkedDwelling { .. }) => {
                add(
                    "dwellings: random dwellings tied to a town (holding it, E09 T11)",
                    1,
                );
                add(
                    "dwellings: random dwellings tied to a random town",
                    usize::from(random_towns.contains(town)),
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

/// The census's keys of the twins unlike the first of their type and anchor, and of those of a
/// kind something names (E08 T113b, [`twins`]).
const TWINS_UNLIKE: &str = "typed objects of one type on one anchor unlike the first";
const TWINS_NAMED: &str = "typed objects of one type on one anchor not a ground, sea ground or hole";

/// How many typed objects of the static layer share their type and anchor with an earlier one (a
/// random resource counted by its random type, before the draw); since E08 T113b (decision 320)
/// also how many of them differ from the first in any field (masks, drawn tiles, holds), and how
/// many are of a kind other than the grounds, the grounds of the sea and the holes (which nothing
/// names: `SetupError::SharedAnchor` refuses the rest).
fn twins(setup: &MapSetup, rules: &GameRuleset) -> (usize, usize, usize) {
    let mut first = BTreeMap::<_, &h3_adventure::ObjectSetup>::new();
    let (mut twins, mut unlike, mut named) = (0, 0, 0);
    for object in &setup.objects {
        if let Some(object_type) = object.object_type {
            let anchor = object.anchor;
            let key = (object_type, anchor.level, anchor.y, anchor.x);
            match first.get(&key) {
                None => {
                    first.insert(key, object);
                }
                Some(earlier) => {
                    twins += 1;
                    unlike += usize::from(*earlier != object);
                    let ground = rules.adventure().object_type(object_type).is_some_and(|ty| {
                        matches!(
                            ty.kind,
                            ObjectKind::Ground { .. } | ObjectKind::SeaGround | ObjectKind::Hole
                        )
                    });
                    named += usize::from(!ground);
                }
            }
        }
    }
    (twins, unlike, named)
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

/// The census of the town events played (E09 T19c2, [`played_town_events`]; exit criterion 6,
/// design §32): its counts, the maps by the days played, and the survey's order's counts
/// ([`survey_order`]).
#[derive(Default)]
struct TownEvents(
    BTreeMap<&'static str, usize>,
    BTreeMap<u32, usize>,
    BTreeMap<&'static str, usize>,
);

/// The survey's census of the town events' buildings in the fixed towns (survey
/// `E09-construction.md` §10: the reading's rules, the events in day order, no build of the
/// owner's between): built, skipped as standing, built without a requirement. [`main`] checks
/// [`survey_order`]'s against it.
const SURVEY_TOWN_EVENTS: (usize, usize, usize) = (57, 30, 27);

/// The days every map is played by `EndTurn` alone (E09 T19c2, [`played_town_events`]): its first
/// week and the week's step; a map whose owned towns' events fire later plays on to their last
/// first day, a repeating one's to its second firing.
const PLAYED_DAYS: u32 = 8;

/// What one firing of a town event does to its town's record (E09 T19c2, [`fire`]): the
/// event's buildings of the faction's record, each counted once by what became of it.
#[derive(Default)]
struct Fired {
    /// Its buildings built, in index order, each with whether a requirement was not ever built.
    built: Vec<(h3_towns::BuildingId, bool)>,
    /// Its buildings ever built before it fired, or built by an earlier one of its own.
    standing: usize,
    /// Its buildings another of its own supersedes (the Fort beside the Castle).
    superseded: usize,
    /// Its Shipyards in a town without water (TWN-SET-011).
    shipyards: usize,
}

/// Fires `event` on `town` as the game's turn start does (TWN-EVT-003: `h3_towns::event_buildings`
/// with the town's water, each building built through `h3_towns::built_with` unless standing by
/// its turn), the census's own walk of the rule, and returns the town after it with what each of
/// the event's buildings came to.
fn fire(
    town: &h3_towns::Town,
    part: &h3_towns::TownRuleset,
    event: &h3_towns::TownEvent,
    water: bool,
) -> Result<(h3_towns::Town, Fired), String> {
    let record = part
        .town(town.faction())
        .ok_or("a town of a faction without a record")?;
    let builds = h3_towns::event_buildings(town, part, event, water);
    let mut fired = Fired::default();
    let mut now = town.clone();
    for id in event.buildings.iter() {
        let building = record
            .building(id)
            .ok_or("an event's building off its record")?;
        if town.built().contains(id) {
            fired.standing += 1;
        } else if !builds.contains(id) {
            if building.roles.shipyard && !water {
                fired.shipyards += 1;
            } else {
                fired.superseded += 1;
            }
        } else if now.built().contains(id) {
            fired.standing += 1;
        } else {
            let missing = !building.requires.difference(now.built()).is_empty();
            let built = h3_towns::built_with(&now, part, id).ok_or("a build off the record")?;
            now = h3_towns::Town::new(now.faction(), built, now.forbidden());
            fired.built.push((id, missing));
        }
    }
    Ok((now, fired))
}

/// Adds a firing's counts to the census under `keys` (built, standing, superseded, Shipyards
/// without water, built without a requirement).
fn count_fired(fired: &Fired, keys: [&'static str; 5], add: &mut impl FnMut(&'static str, usize)) {
    add(keys[0], fired.built.len());
    add(keys[1], fired.standing);
    add(keys[2], fired.superseded);
    add(keys[3], fired.shipyards);
    add(
        keys[4],
        fired.built.iter().filter(|&&(_, missing)| missing).count(),
    );
}

/// The survey's order (survey `E09-construction.md` §10, T12b's census): every event of every
/// fixed town (a town of a fixed faction), owned or not and for whatever player, once each in day
/// order (its first day, then the town's list), from the town's record at the start, no build of
/// the owner's between; each of its buildings of the faction's record in index order (the
/// original's id order, `town_events_apply` 0x5BFEB0) built one by one unless ever built by its
/// turn, a building another of its own supersedes too (the Fort before the Castle), and counted
/// built without a requirement when one of its direct requirements is not ever built then (the
/// executable's lists: the part's closed `requires` less what those requirements require). The
/// counts that [`main`] checks against [`SURVEY_TOWN_EVENTS`]. Checked (an error fails the map):
/// after each event the town's ever-built set is the game's walk's ([`fire`], which builds the
/// superseding building alone and tests the closed requirements).
fn survey_order(
    town: &h3_towns::Town,
    events: &[h3_towns::TownEvent],
    part: &h3_towns::TownRuleset,
    water: bool,
    counts: &mut BTreeMap<&'static str, usize>,
) -> Result<(), String> {
    let record = part
        .town(town.faction())
        .ok_or("a town of a faction without a record")?;
    // A building's direct requirements: its closed ones less what those require.
    let direct = |building: &h3_towns::Building| {
        let implied = (building.requires.iter())
            .filter_map(|id| record.building(id))
            .fold(h3_towns::BuildingSet::EMPTY, |set, r| set.union(r.requires));
        building.requires.difference(implied)
    };
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    let mut order: Vec<(u16, usize)> = (events.iter().enumerate())
        .map(|(place, event)| (event.first_day, place))
        .collect();
    order.sort_unstable();
    let mut now = town.clone();
    for (_, place) in order {
        let event = &events[place];
        let (walked, _) = fire(&now, part, event, water)?;
        for id in event.buildings.iter() {
            let building = record
                .building(id)
                .ok_or("an event's building off its record")?;
            if now.built().contains(id) {
                add("skipped as standing", 1);
                continue;
            }
            add("built", 1);
            let missing = !direct(building).difference(now.built()).is_empty();
            add("built without a requirement", usize::from(missing));
            let built = h3_towns::built_with(&now, part, id).ok_or("a build off the record")?;
            now = h3_towns::Town::new(now.faction(), built, now.forbidden());
        }
        if now.built() != walked.built() {
            return Err(format!(
                "a town event's buildings one by one differ from the game's walk: {:?}, {:?}",
                now.built(),
                walked.built()
            ));
        }
    }
    Ok(())
}

/// The town events a game fired by `EndTurn` alone (E09 T19c2; TWN-EVT-001 to -004, exit
/// criterion 6, design §32): the map played to [`PLAYED_DAYS`], or on to the last first day of
/// the events of its owned towns that name their owner and its kind (a repeating one to its
/// second firing); checked (an error fails the map) against the census's own walk of the rules
/// ([`fire`]) from the towns' records before their events (the game built again without them):
/// the events fired by day, town and place (day 1's of the first group inside the build,
/// `GameState::town_events_at_start`), the buildings built after the build by day, town, event
/// and building (`TownBuilt`), and the first group's towns' records after the build. Counts the
/// events by why they fire or not, the firings by what they carry, and their buildings by what
/// became of each; with [`survey_order`]'s counts of the fixed towns.
fn played_town_events(
    setup: &GameSetup,
    mut game: GameState,
    rules: &GameRuleset,
    census: &mut TownEvents,
) -> Result<(), String> {
    let part = rules.towns();
    let first = game.turn_group().to_vec();
    let towns = u16::try_from(game.towns().len()).map_err(|_| "too many towns")?;
    let has_events = setup.towns.iter().any(|town| !town.events.is_empty());
    // The towns before their day-1 events: the same game built without the towns' events (the
    // events move no draw, so every town takes the same faction; checked).
    let quiet = if has_events {
        let mut quiet = setup.clone();
        for town in &mut quiet.towns {
            town.events.clear();
        }
        Some(GameState::new(rules, &quiet, SEED).map_err(|e| format!("without events: {e:?}"))?)
    } else {
        None
    };
    // Each town with events: its record before them, its water, its owner and kind at the start.
    let mut starts = Vec::new();
    for (town, place) in (0..towns).map(TownId::new).zip(&setup.towns) {
        let (Some(quiet), Some(statics)) = (&quiet, game.towns().town_static(town)) else {
            continue;
        };
        if statics.events.is_empty() {
            continue;
        }
        let before = (quiet.towns().get(town).cloned()).ok_or("a town without a record")?;
        if Some(before.faction()) != game.towns().get(town).map(h3_towns::Town::faction) {
            return Err(format!(
                "town {}: another faction without events",
                town.raw()
            ));
        }
        let map = game.adventure().map();
        let water = map.town_boat_spot(town).is_some();
        let owner = map
            .town_object(town)
            .and_then(|o| game.adventure().owner(o));
        let fixed = matches!(place.faction, TownFaction::Fixed(_));
        if fixed {
            survey_order(&before, &statics.events, part, water, &mut census.2)?;
        }
        starts.push((town, before, statics.events.clone(), water, owner, fixed));
    }
    // The days played: the week, or on to the last first day of an event that fires.
    let mut days = PLAYED_DAYS;
    let counts = &mut census.0;
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    for (_, _, events, _, owner, _) in &starts {
        for event in events {
            add("town events", 1);
            let Some(owner) = *owner else {
                add(
                    "town events of a neutral town (none fires without a capture)",
                    1,
                );
                continue;
            };
            let human = game.human(owner).ok_or("an owner not in the game")?;
            let timing = timing(event);
            if !event.players.contains(&owner) {
                add("town events not naming their town's owner", 1);
            } else if !timing.fires(owner, human, u32::from(event.first_day)) {
                add("town events naming the owner, not its kind", 1);
            } else {
                add("town events that fire", 1);
                days = days.max(u32::from(event.first_day) + u32::from(event.interval));
            }
        }
    }
    // The census's walk of the rules over those days.
    let mut expected_fired = Vec::new();
    let mut expected_built = Vec::new();
    for (town, before, events, water, owner, fixed) in starts {
        let Some(owner) = owner else { continue };
        let human = game.human(owner).ok_or("an owner not in the game")?;
        let mut now = before;
        for day in 1..=days {
            for (event, place) in events.iter().zip(0u16..) {
                if !timing(event).fires(owner, human, day) {
                    continue;
                }
                expected_fired.push((day, town, place));
                add("firings", 1);
                add(
                    "firings giving or taking resources",
                    usize::from(!event.resources.is_empty()),
                );
                add(
                    "firings with buildings",
                    usize::from(!event.buildings.is_empty()),
                );
                let creatures = event.creatures.iter().any(|&n| n > 0);
                add(
                    "firings with creatures (not added, TWN-STUB-10)",
                    usize::from(creatures),
                );
                add(
                    "firings with a message alone",
                    usize::from(
                        event.resources.is_empty() && event.buildings.is_empty() && !creatures,
                    ),
                );
                let (after, fired) = fire(&now, part, event, water)?;
                let keys = if fixed {
                    [
                        "fixed towns: built",
                        "fixed towns: skipped as standing",
                        "fixed towns: superseded by another of the event's",
                        "fixed towns: Shipyards without water",
                        "fixed towns: built without a requirement",
                    ]
                } else {
                    [
                        "random towns: built",
                        "random towns: skipped as standing",
                        "random towns: superseded by another of the event's",
                        "random towns: Shipyards without water",
                        "random towns: built without a requirement",
                    ]
                };
                count_fired(&fired, keys, &mut add);
                let inside = day == 1 && first.contains(&owner);
                add(
                    "built inside the build (day 1, the first group)",
                    fired.built.len() * usize::from(inside),
                );
                if !inside {
                    let built = fired.built.iter().map(|&(id, _)| (day, town, place, id));
                    expected_built.extend(built);
                }
                now = after;
            }
            if day == 1 && first.contains(&owner) {
                let built = game.towns().get(town).map(h3_towns::Town::built);
                if built != Some(now.built()) {
                    return Err(format!(
                        "town {}: the build's day-1 events built otherwise",
                        town.raw()
                    ));
                }
            }
        }
    }
    // The play: `EndTurn` alone until the day after the last, the first group's events inside
    // the build, every other's told.
    let mut fired: Vec<(u32, TownId, u16)> = (first.iter())
        .flat_map(|&player| game.town_events_at_start(player))
        .map(|(town, place)| (1, town, place))
        .collect();
    let mut built = Vec::new();
    let mut sink = Vec::new();
    while game.day() <= days {
        let day = game.day();
        let player = (game.turn_group().iter().copied())
            .find(|&player| game.is_acting(player))
            .ok_or_else(|| format!("day {day}: no player acting"))?;
        let end = h3_game::apply(
            &mut game,
            rules,
            Actor::Player(player),
            GameCommand::EndTurn,
            &mut sink,
        );
        end.map_err(|e| format!("day {day}: EndTurn refused: {e:?}"))?;
        let day = game.day();
        for event in sink.drain(..) {
            match event {
                GameEvent::TownEventFired { town, event, .. } if day <= days => {
                    fired.push((day, town, event));
                }
                GameEvent::TownBuilt {
                    town,
                    building,
                    event: Some(place),
                    ..
                } if day <= days => {
                    built.push((day, town, place, building));
                }
                _ => {}
            }
        }
    }
    fired.sort_unstable();
    expected_fired.sort_unstable();
    built.sort_unstable();
    expected_built.sort_unstable();
    if fired != expected_fired {
        return Err(format!(
            "the town events fired {fired:?}, the rules' walk {expected_fired:?}"
        ));
    }
    if built != expected_built {
        return Err(format!(
            "the town events built {built:?}, the rules' walk {expected_built:?}"
        ));
    }
    add("maps played", 1);
    add("maps played with town events", usize::from(has_events));
    *census.1.entry(days).or_default() += 1;
    Ok(())
}

/// A town event's day test, the map's timed events' (GAM-CAL-004, TWN-EVT-001): the game's own
/// [`TimedEventSetup::fires`].
fn timing(event: &h3_towns::TownEvent) -> TimedEventSetup {
    TimedEventSetup {
        players: event.players.clone(),
        human: event.human,
        computer: event.computer,
        first_day: event.first_day,
        interval: event.interval,
        resources: Vec::new(),
        message: 0,
    }
}

/// Prints the census of the town events played (E09 T19c2, [`played_town_events`]) and checks
/// [`survey_order`]'s counts against the survey's ([`SURVEY_TOWN_EVENTS`]): the differences.
fn town_event_differences(census: &TownEvents) -> usize {
    println!(
        "  the town events played by EndTurn alone (seed {SEED}), checked against the rules' walk:"
    );
    for (what, n) in &census.0 {
        println!("    {what}: {n}");
    }
    let days: Vec<String> = (census.1.iter())
        .map(|(days, maps)| format!("{days} days {maps}"))
        .collect();
    println!("    maps by the days played: {}", days.join(", "));
    println!(
        "  the fixed towns' town events in the survey's order (every event once, day order, each \
         building one by one), checked:"
    );
    for (what, n) in &census.2 {
        println!("    {what}: {n}");
    }
    let get = |what| census.2.get(what).copied().unwrap_or(0);
    let order = (
        get("built"),
        get("skipped as standing"),
        get("built without a requirement"),
    );
    if order == SURVEY_TOWN_EVENTS {
        0
    } else {
        println!(
            "h3-map: the town events in the survey's order (built, skipped as standing, without a \
             requirement: {order:?}) are not the survey's census {SURVEY_TOWN_EVENTS:?}"
        );
        1
    }
}
