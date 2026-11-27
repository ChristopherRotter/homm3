//! `generate` over the packs' tables (E14 T2c, T3c, T3d, T3e): the oracle's corpus against
//! `phases.txt`, `layout.txt`, `zones.txt` and `water.txt` (the comparison seam, ADR-0029 point
//! 8), the run's adjustments, steps 1 to 3, the flags and the errors.

use std::collections::BTreeMap;

use h3_rmg::{
    Choices, CrtRand, Fit, GenerateError, GeneratedMap, MapFormat, MapSize, MemoryZone, Monsters,
    Objects, PartitionFault, PositionError, SITE_TEMPLATE, SITE_TOWN_CENTRAL_CELL,
    SITE_TOWN_TYPE_ANY, SITE_WATER, SITE_ZONE_POSITION, SITE_ZONE_TERRAIN_TOWN,
    SITE_ZONE_TOWN_TYPE, Settings, SettingsError, StepState, TerrainFault, Trace, Water,
    ZoneGenerator, ZoneMemory, generate, kept_templates, resolve_random,
};
use proptest::prelude::*;
use std::sync::LazyLock;

use super::digest::digest12;
use super::*;

/// The draws and the steps' ends, as the phase oracle's log has them, with the draws made before
/// each end.
#[derive(Debug, Default)]
struct Log {
    draws: Vec<(u32, u32)>,
    steps: Vec<u8>,
    ends: Vec<usize>,
}

impl Log {
    /// The frame's draws, before step 1.
    fn frame(&self) -> &[(u32, u32)] {
        &self.draws[..self.ends[0]]
    }
}

impl Trace for Log {
    fn draw(&mut self, site: u32, value: u32) {
        self.draws.push((site, value));
    }

    fn step_end(&mut self, step: u8, _state: StepState<'_>) {
        self.steps.push(step);
        self.ends.push(self.draws.len());
    }
}

/// The phase oracle's view of a run (`phases.txt`'s `steps=` and `cells=`, `layout.txt`): per
/// phase its draws and their lines' digest, per step end its dump's digest, the zone generators
/// after steps 1 and 2.
#[derive(Debug, Default)]
struct Phases {
    log: Log,
    phases: BTreeMap<String, (usize, String)>,
    cells: BTreeMap<u8, String>,
    layout: Vec<ZoneGenerator>,
    /// The zone generators after step 2 (step 3 moves their centres).
    partitioned: Vec<ZoneGenerator>,
    /// Step 2's zones' bytes as measured (`zones.txt`), in the call order.
    memory: Vec<ZoneMemory>,
}

/// The log's lines of `draws` (`<site> <value>`, the site in upper-case hex) and their digest.
fn lines_digest(draws: &[(u32, u32)]) -> (usize, String) {
    let text: String = draws.iter().map(|(s, v)| format!("{s:X} {v}\n")).collect();
    (draws.len(), digest12(text.as_bytes()))
}

impl Trace for Phases {
    fn draw(&mut self, site: u32, value: u32) {
        self.log.draw(site, value);
    }

    fn step_end(&mut self, step: u8, state: StepState<'_>) {
        let from = self.log.ends.last().copied().unwrap_or(0);
        let draws = &self.log.draws[from..];
        if step == 0 {
            // The constructor draws water's Random (the setup), `rmg_generate_map` the template.
            let (setup, template): (Vec<_>, Vec<_>) =
                draws.iter().partition(|&&(site, _)| site == SITE_WATER);
            self.phases.insert("setup".into(), lines_digest(&setup));
            self.phases
                .insert("template".into(), lines_digest(&template));
        } else {
            self.phases.insert(step.to_string(), lines_digest(draws));
        }
        self.cells.insert(step, digest12(&state.dump(step)));
        match step {
            1 => self.layout = state.zones.to_vec(),
            2 => self.partitioned = state.zones.to_vec(),
            _ => {}
        }
        self.log.step_end(step, state);
    }

    fn zone_memory(&mut self, _kind: MemoryZone, index: usize) -> Option<ZoneMemory> {
        self.memory.get(index).copied()
    }
}

/// `zones.txt`'s step 2 zones by case in the call order: the bytes RMG-QUIRK-1 reads.
fn kept_zone_memory() -> BTreeMap<String, Vec<ZoneMemory>> {
    let mut out: BTreeMap<String, Vec<ZoneMemory>> = BTreeMap::new();
    for line in include_str!("../../../../clients/capture/rmg/zones.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let words: Vec<&str> = line.split(' ').collect();
        let field = |name: &str| {
            words
                .iter()
                .find_map(|w| w.strip_prefix(name))
                .unwrap()
                .to_string()
        };
        out.entry(words[0].to_string())
            .or_default()
            .push(ZoneMemory {
                town_types: u16::from_str_radix(&field("mask="), 16).unwrap(),
                same_type: field("same=") == "1",
                terrain_match: field("m84=") == "1",
                monster_match: field("m94=") == "1",
            });
    }
    out
}

/// `phases.txt`'s lines by case: per phase its draws and digest, per step its dump's digest.
type Kept = BTreeMap<String, (BTreeMap<String, (usize, String)>, BTreeMap<u8, String>)>;

fn kept_phases() -> Kept {
    include_str!("../../../../clients/capture/rmg/phases.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let mut words = line.split(' ');
            let id = words.next().unwrap().to_string();
            let field = |name: &str| {
                line.split(' ')
                    .find_map(|w| w.strip_prefix(name))
                    .unwrap()
                    .split(',')
                    .map(|p| p.split(':').collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            };
            let phases = field("steps=")
                .into_iter()
                .map(|p| (p[0].to_string(), (p[1].parse().unwrap(), p[2].to_string())))
                .collect();
            let cells = field("cells=")
                .into_iter()
                .map(|p| (p[0].parse().unwrap(), p[1].to_string()))
                .collect();
            (id, (phases, cells))
        })
        .collect()
}

/// The C runtime's first `rand()` after `srand(seed)`.
fn first_rand(seed: u32) -> u32 {
    (seed.wrapping_mul(214_013).wrapping_add(2_531_011) >> 16) & 0x7FFF
}

/// The original packs' core and part in `faithful` and `strict-fixed`, compiled once.
static ORIGINAL: LazyLock<[(Ruleset, RmgRuleset); 2]> = LazyLock::new(|| {
    let cores: Vec<&str> = ORIGINAL_PACKS.iter().map(|(core, _)| *core).collect();
    ["base:faithful", "base:strict-fixed"].map(|profile| {
        (
            core(&cores, profile),
            compile(&ORIGINAL_PACKS, profile).unwrap(),
        )
    })
});

fn original(profile: &str) -> &'static (Ruleset, RmgRuleset) {
    &ORIGINAL[usize::from(profile == "base:strict-fixed")]
}

fn lone(profile: &str, rmg: &str) -> (Ruleset, RmgRuleset) {
    (
        core(&[LONE_CORE], profile),
        compile(&[(LONE_CORE, rmg)], profile).unwrap(),
    )
}

fn choices(size: MapSize, players: u8, computers: u8) -> Choices {
    let mut humans = [false; 8];
    humans[0] = true;
    Choices {
        size,
        two_levels: false,
        humans,
        towns: [None; 8],
        players: Some(players),
        teams: Some(0),
        computers: Some(computers),
        computer_teams: Some(0),
        water: Water::Normal,
        monsters: Some(Monsters::Normal),
        format: MapFormat::SoD,
    }
}

fn settings(c: &Choices) -> Settings {
    resolve_random(c, &mut CrtRand::new(0)).unwrap()
}

fn run(
    (core, part): &(Ruleset, RmgRuleset),
    s: &Settings,
    seed: u32,
) -> (Result<GeneratedMap, GenerateError>, Log) {
    let mut log = Log::default();
    (generate(core, part, s, seed, &mut log), log)
}

/// The oracle's corpus (`clients/capture/rmg/corpus.toml`; `kept.txt`'s templates, the
/// original's under Wine, `docs/lanes/rmg/ORACLE.md`): id, size, levels, players, teams,
/// computers, computer teams (the tab's: 0 none), water, monsters, seed, the template.
type Case = (
    &'static str,
    MapSize,
    u8,
    u8,
    u8,
    u8,
    u8,
    Water,
    Monsters,
    u32,
    &'static str,
);

#[rustfmt::skip]
const CORPUS: [Case; 24] = {
    use MapSize::{ExtraLarge as XL, Large as L, Medium as M, Small as S};
    use Monsters::{Normal as Nm, Strong as St, Weak as Wk};
    use Water::{Islands as I, None as D, Normal as N};
    [
        ("s1-2p", S, 1, 2, 0, 0, 0, N, Nm, 1_000_000_000, "3SB0b"),
        ("s1-2p-dry", S, 1, 2, 0, 0, 0, D, Wk, 1_100_000_000, "2SM2i"),
        ("s1-2p-c1-islands", S, 1, 2, 0, 1, 0, I, St, 1_200_000_000, "5SB0b"),
        ("s2-2p", S, 2, 2, 0, 0, 0, N, Nm, 1_300_000_000, "2SM4d"),
        ("m1-4p", M, 1, 4, 0, 0, 0, N, Nm, 1_400_000_000, "4MM2h"),
        ("m1-4p-t2", M, 1, 4, 2, 0, 0, D, St, 1_500_000_000, "4SM0g"),
        ("m2-2p", M, 2, 2, 0, 0, 0, N, Nm, 1_000_000_000, "Dragons"),
        ("m2-4p", M, 2, 4, 0, 0, 0, N, Wk, 1_600_000_000, "2SM4d(2)"),
        ("m2-3p-c2-islands", M, 2, 3, 0, 2, 1, I, Nm, 1_700_000_000, "8SM0c"),
        ("l1-4p-a", L, 1, 4, 0, 0, 0, N, Nm, 1_763_900_000, "6LM10"),
        ("l1-4p-b", L, 1, 4, 0, 0, 0, N, Nm, 1_763_900_001, "7SB0c"),
        ("l1-4p-c", L, 1, 4, 0, 0, 0, N, Nm, 1_763_900_002, "Ring"),
        ("l1-6p-t3", L, 1, 6, 3, 0, 0, N, Nm, 1_800_000_000, "Meeting \nin Muzgob"),
        ("l1-8p-dry", L, 1, 8, 0, 0, 0, D, Nm, 1_900_000_000, "8MM0b"),
        ("l2-4p", L, 2, 4, 0, 0, 0, N, Nm, 2_000_000_000, "Dwarven Tunnels"),
        ("l2-2p-c4-islands", L, 2, 2, 0, 4, 1, I, St, 2_100_000_000, "Dragons"),
        ("xl1-4p-seed1", XL, 1, 4, 0, 0, 0, N, Nm, 1, "Dwarven Tunnels"),
        ("xl1-6p-c2", XL, 1, 6, 2, 2, 0, N, Wk, 2_147_483_647, "8XM8"),
        ("xl2-4p-seed0", XL, 2, 4, 0, 0, 0, N, Nm, 0, "8XM12"),
        ("xl2-8p-t4-islands", XL, 2, 8, 4, 0, 0, I, St, 123_456_789, "8XM8"),
        ("m1-2p-t1", M, 1, 2, 1, 0, 0, N, Nm, 1_400_000_001, "3SB0b"),
        ("s1-3p-2sm2i2", S, 1, 3, 0, 0, 0, N, Nm, 1_350_000_003, "2SM2i(2)"),
        ("s1-2p-2sm0k", S, 1, 2, 0, 0, 0, N, Nm, 1_350_000_000, "2SM0k"),
        ("s1-1p-c1-2sm0k", S, 1, 1, 0, 1, 0, N, Nm, 1_450_000_002, "2SM0k"),
    ]
};

/// The settings of a corpus case.
fn corpus_settings(case: usize) -> (&'static str, Settings, u32, &'static str) {
    let (id, size, levels, players, teams, computers, computer_teams, water, monsters, seed, name) =
        CORPUS[case];
    let c = Choices {
        two_levels: levels == 2,
        teams: Some(teams),
        computer_teams: Some(computer_teams),
        water,
        monsters: Some(monsters),
        ..choices(size, players, computers)
    };
    (id, settings(&c), seed, name)
}

// covers: RMG-SET-006, RMG-TPL-005, RMG-TPL-007
/// The cases of the oracle's corpus draw the original's templates: the first `rand()` after
/// `srand(seed)` (no water Random draws nothing before it) over the templates kept, in SoD's
/// table, the only draw before step 1.
#[test]
fn the_corpus_draws_the_originals_templates() {
    let packs = original("base:faithful");
    for case in 0..CORPUS.len() {
        let (id, s, seed, name) = corpus_settings(case);
        let (map, log) = run(packs, &s, seed);
        let map = map.unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(map.template_name, name, "{id}");
        assert_eq!(log.frame(), [(SITE_TEMPLATE, first_rand(seed))], "{id}");
        assert_eq!(log.steps, [0, 1, 2, 3, 4, 5], "{id}");
        assert_eq!((map.seed, map.settings), (seed, s), "{id}");
        assert_eq!(packs.1.templates()[map.template.index].name, name, "{id}");
    }
}

// covers: RMG-SET-007, RMG-TPL-005
/// Water's Random is the generation's first draw, `rand() % 3`, before the templates are kept:
/// islands halve the size index.
#[test]
fn waters_random_is_drawn_first() {
    let packs = lone("lone:on", SYNTHETIC_RMG);
    let random = Settings {
        water: Water::Random,
        ..settings(&choices(MapSize::Medium, 1, 1))
    };
    for (k, water) in [Water::None, Water::Normal, Water::Islands]
        .into_iter()
        .enumerate()
    {
        let seed = (0u32..).find(|&s| first_rand(s) % 3 == k as u32).unwrap();
        let (map, log) = run(&packs, &random, seed);
        let map = map.unwrap();
        assert_eq!(map.settings.water, water);
        assert_eq!(log.draws[0], (SITE_WATER, first_rand(seed)));
        assert_eq!(log.frame().len(), 2);
        assert_eq!(log.draws[1].0, SITE_TEMPLATE);
    }
    // L with islands: index 9 halved to 4, Twin Vales' range [1, 8].
    let large = Settings {
        water: Water::Islands,
        ..settings(&choices(MapSize::Large, 1, 1))
    };
    assert!(run(&packs, &large, 0).0.is_ok());
    let normal = Settings {
        water: Water::Normal,
        ..large
    };
    assert_eq!(run(&packs, &normal, 0).0, Err(GenerateError::NoTemplate));
}

// covers: RMG-SET-005, RMG-TPL-008
/// One player without computers is run as one player and one computer: synthetic Twin Vales
/// (its human start for 1 to 2 humans, its computer start for 2 players in all) is kept, the
/// computer on its computer start.
#[test]
fn under_two_players_in_all_the_run_takes_one_and_one() {
    let packs = lone("lone:on", SYNTHETIC_RMG);
    let alone = settings(&choices(MapSize::Medium, 1, 0));
    let (map, log) = run(&packs, &alone, 7);
    let map = map.unwrap();
    assert_eq!((map.settings.players, map.settings.computers), (1, 1));
    assert_eq!(map.template_name, "Twin Vales");
    assert_eq!(
        map.positions,
        [Some(0), Some(1), None, None, None, None, None, None]
    );
    assert_eq!(map.template.zones.len(), 3);
    assert_eq!(log.steps, [0, 1, 2, 3, 4, 5]);
    // Two players: Twin Vales has one human start, so nothing is kept.
    let two = settings(&choices(MapSize::Medium, 2, 0));
    assert_eq!(run(&packs, &two, 7).0, Err(GenerateError::NoTemplate));
}

// covers: RMG-SET-008
/// The banned heroes: a campaign hero, or not one of the format's heroes.
#[test]
fn heroes_are_banned_by_the_format() {
    let packs = original("base:faithful");
    let (_, s, seed, _) = corpus_settings(0);
    let banned = |format| {
        let map = run(packs, &Settings { format, ..s }, seed).0.unwrap();
        (0..)
            .zip(map.banned_heroes)
            .filter(|(_, b)| *b)
            .map(|(id, _)| id)
            .collect::<Vec<u32>>()
    };
    // Lord Haart the Knight (4) and the eleven campaign heroes; RoE: every hero past its 128.
    let campaign: Vec<u32> = (145..156).collect();
    assert_eq!(banned(MapFormat::SoD), [&[4][..], &campaign].concat());
    assert_eq!(banned(MapFormat::AB), banned(MapFormat::SoD));
    assert_eq!(banned(MapFormat::RoE), (128..156).collect::<Vec<_>>());
    let limits = packs.1.generator().unwrap().type_limits();
    assert_eq!(
        (limits.map[6], limits.map[8], limits.map[0]),
        (200, 64, 32_000)
    );
    assert_eq!((limits.zone[113], limits.zone[2]), (3, 1));
}

// covers: RMG-TPL-003
/// RMG-QUIRK-11: an RoE map's zones lose Fortress's monster town with the flag, Conflux's
/// without; their town types lose Conflux either way.
#[test]
fn roe_zones_lose_a_monster_town_by_the_flag() {
    let rmg = SYNTHETIC_RMG.replacen(
        "monster_towns = \"000010000\"",
        "monster_towns = \"111111111\"",
        1,
    );
    let roe = Settings {
        format: MapFormat::RoE,
        ..settings(&choices(MapSize::Medium, 1, 1))
    };
    let towns = |profile| {
        let map = run(&lone(profile, &rmg), &roe, 3).0.unwrap();
        let z = &map.template.zones[2];
        (z.town_types, z.monster_towns)
    };
    assert_eq!(towns("lone:on"), (0x010, 0x17F));
    assert_eq!(towns("lone:off"), (0x010, 0x0FF));
}

// covers: RMG-TPL-008
/// RMG-QUIRK-24: AB's 2SM2i(2) with three players puts the third on position 7, which no zone
/// has, with the flag; without it the starts run out; four run out with it too.
#[test]
fn two_sm2i_2_gives_the_third_player_a_seventh_start() {
    let three = settings(&choices(MapSize::Small, 3, 0));
    let seed_for = |packs: &(Ruleset, RmgRuleset), s: &Settings| {
        let fit = Fit::new(36, 1, Water::Normal, i32::from(s.players), 0);
        let kept = kept_templates(packs.1.templates(), fit);
        let at = kept
            .iter()
            .position(|&i| packs.1.templates()[i].name == "2SM2i(2)")
            .unwrap();
        (0u32..)
            .find(|&seed| first_rand(seed) as usize % kept.len() == at)
            .unwrap()
    };
    let faithful = original("base:faithful");
    let seed = seed_for(faithful, &three);
    let map = run(faithful, &three, seed).0.unwrap();
    assert_eq!(map.template_name, "2SM2i(2)");
    assert_eq!(
        map.positions,
        [Some(0), Some(1), None, None, None, None, None, Some(2)]
    );
    let fixed = original("base:strict-fixed");
    let exhausted = |packs: &(Ruleset, RmgRuleset), s: &Settings, seed| {
        matches!(
            run(packs, s, seed).0,
            Err(GenerateError::Positions {
                error: PositionError::Exhausted,
                ..
            })
        )
    };
    assert!(exhausted(fixed, &three, seed_for(fixed, &three)));
    let four = settings(&choices(MapSize::Small, 4, 0));
    assert!(exhausted(faithful, &four, seed_for(faithful, &four)));
}

// covers: RMG-SET-004, RMG-TPL-008
/// The errors: refused settings, a part of another core, no generator's tables, positions
/// outside the original's arrays; each with a message.
#[test]
fn generations_that_cannot_run_are_refused() {
    let packs = lone("lone:on", SYNTHETIC_RMG);
    let ok = settings(&choices(MapSize::Medium, 1, 1));
    let bad = Settings { teams: 0, ..ok };
    assert_eq!(
        run(&packs, &bad, 0).0,
        Err(GenerateError::Settings(SettingsError::Teams {
            teams: 0,
            players: 1
        }))
    );
    let other = (core(&[LONE_CORE], "lone:off"), packs.1.clone());
    assert!(matches!(
        run(&other, &ok, 0).0,
        Err(GenerateError::Core { .. })
    ));
    let start = SYNTHETIC_RMG.find("\n# The generator's constants").unwrap();
    let without = lone("lone:on", &SYNTHETIC_RMG[..start]);
    assert_eq!(run(&without, &ok, 0).0, Err(GenerateError::NoGenerator));
    let ab = ORIGINAL_PACKS[..2].to_vec();
    let cores: Vec<&str> = ab.iter().map(|(core, _)| *core).collect();
    let ab = (
        core(&cores, "base:faithful"),
        compile(&ab, "base:faithful").unwrap(),
    );
    assert_eq!(run(&ab, &ok, 0).0, Err(GenerateError::NoGenerator));
    let far = SYNTHETIC_RMG.replacen("ownership = 2", "ownership = 9", 1);
    assert_eq!(
        run(&lone("lone:on", &far), &ok, 0).0,
        Err(GenerateError::Positions {
            template: 0,
            error: PositionError::Outside {
                zone: 1,
                position: 8
            }
        })
    );
    let errors = [
        GenerateError::Settings(SettingsError::Players(0)),
        GenerateError::Core { part: 1, core: 2 },
        GenerateError::NoGenerator,
        GenerateError::NoTemplate,
        GenerateError::Positions {
            template: 0,
            error: PositionError::Outside {
                zone: 1,
                position: 8,
            },
        },
        GenerateError::Positions {
            template: 0,
            error: PositionError::Exhausted,
        },
    ];
    for e in errors {
        assert!(!e.to_string().is_empty());
    }
    assert_eq!(
        GenerateError::from(SettingsError::Players(9)),
        GenerateError::Settings(SettingsError::Players(9))
    );
}

fn any_choices() -> impl Strategy<Value = Choices> {
    let size = prop_oneof![
        Just(MapSize::Small),
        Just(MapSize::Medium),
        Just(MapSize::Large),
        Just(MapSize::ExtraLarge)
    ];
    let water = prop_oneof![
        Just(Water::None),
        Just(Water::Normal),
        Just(Water::Islands),
        Just(Water::Random)
    ];
    let monsters = proptest::option::of(prop_oneof![
        Just(Monsters::Weak),
        Just(Monsters::Normal),
        Just(Monsters::Strong)
    ]);
    let format = prop_oneof![
        Just(MapFormat::RoE),
        Just(MapFormat::AB),
        Just(MapFormat::SoD)
    ];
    let count = || proptest::option::of(0u8..10);
    (
        (size, any::<bool>(), any::<[bool; 8]>()),
        (count(), count(), count(), count()),
        (water, monsters, format),
    )
        .prop_map(
            |((size, two_levels, humans), (players, teams, computers, computer_teams), rest)| {
                Choices {
                    size,
                    two_levels,
                    humans,
                    towns: [None, Some(0), Some(8), None, Some(3), None, None, Some(1)],
                    players,
                    teams,
                    computers,
                    computer_teams,
                    water: rest.0,
                    monsters: rest.1,
                    format: rest.2,
                }
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    /// Arbitrary choices, seeds and either profile run the frame and steps 1 to 5 over the
    /// original tables without a panic (exit criterion (d)); a map's template is kept, its
    /// positions' colours distinct, one per player and computer (or the error says why not).
    #[test]
    fn arbitrary_settings_never_panic(
        c in any_choices(),
        draws in any::<u32>(),
        seed in any::<u32>(),
        fixed in any::<bool>(),
    ) {
        let profile = if fixed { "base:strict-fixed" } else { "base:faithful" };
        let packs = original(profile);
        let Ok(s) = resolve_random(&c, &mut CrtRand::new(draws)) else {
            return Ok(());
        };
        if let (Ok(map), log) = run(packs, &s, seed) {
            let source = &packs.1.templates()[map.template.index];
            prop_assert_eq!(&map.template_name, &source.name);
            let mut colours: Vec<u8> = map.positions.iter().flatten().copied().collect();
            colours.sort_unstable();
            colours.dedup();
            let run = map.settings;
            prop_assert_eq!(colours.len(), usize::from(run.players + run.computers));
            prop_assert_eq!(log.steps, vec![0, 1, 2, 3, 4, 5]);
            prop_assert!(map.settings.water != Water::Random);
            // Step 1: a zone generator per kept zone, on the map's levels, a land terrain on
            // level 0 and subterranean or lava on level 1, town types of the nine or none; step
            // 2's water zones (terrain 8) on level 0 with water, each its own template zone.
            prop_assert_eq!(map.zones.len(), map.template.zones.len());
            prop_assert_eq!(map.partition.polygons.len(), map.zones.len());
            for g in &map.zones {
                let z = g.position[2];
                prop_assert!(z == 0 || (z == 1 && run.two_levels));
                prop_assert!((-1..=8).contains(&g.town_type));
                if g.terrain == 8 {
                    prop_assert!(z == 0 && run.water != Water::None);
                    continue;
                }
                prop_assert!((0..=7).contains(&g.terrain));
                prop_assert!(z == 0 || g.terrain == 6 || g.terrain == 7);
                prop_assert!((-1..=8).contains(&g.town_type_2));
            }
            // Step 2's cells: none, or a zone's. Step 3's: a terrain and one of its set's
            // pictures, rock's unmirrored; a zone with cells centred in its box.
            let tables = packs.1.generator().unwrap();
            for c in map.cells.cells() {
                let b = c.zone_byte();
                prop_assert!(b == 0xFF || usize::from(b) < map.zones.len());
                let (terrain, picture) = ((c.tile & 0x3F) as usize, ((c.tile >> 6) & 0xFF) as usize);
                let count = match tables.picture_sets.get(terrain) {
                    Some(set) => set.pictures.len(),
                    None => tables.rock_pictures.len(),
                };
                prop_assert!(terrain <= 9 && picture < count);
                prop_assert!(terrain != 9 || c.tile_data & (3 << 15) == 0);
            }
            for g in &map.zones {
                let [x0, y0, x1, y1] = g.bounds;
                if x0 < x1 {
                    prop_assert!((x0..x1).contains(&g.position[0]));
                    prop_assert!((y0..y1).contains(&g.position[1]));
                }
            }
            // Steps 4 and 5: each town an object of class 98 with its entrance a road target, its
            // serial the next, its owner neutral or a position's colour, its zone with a first
            // town whose entrance is that zone's first road target.
            let placed = &map.placed;
            prop_assert_eq!(placed.towns.len(), placed.road_targets.len());
            prop_assert_eq!(map.objects.serial(), placed.towns.len() as i32 + 1);
            for (k, t) in placed.towns.iter().enumerate() {
                let o = &map.objects.placed()[t.object];
                prop_assert_eq!(o.class, 98);
                prop_assert_eq!(t.serial, k as i32 + 1);
                prop_assert!(
                    t.owner == -1 || map.positions.contains(&u8::try_from(t.owner).ok())
                );
                let first = placed.towns.iter().position(|u| u.zone == t.zone);
                prop_assert_eq!(
                    map.zones[t.zone].town,
                    first.map(|f| placed.road_targets[f])
                );
                let [x, y, z] = placed.road_targets[k];
                let cell = map.cells.get(x, y, z).unwrap();
                prop_assert_eq!(i32::from(cell.zone_byte() as i8), map.template.zones[map.zones[t.zone].zone].id);
            }
        }
    }
}

// covers: RMG-OBJ-009
/// The original tables' prototypes (`rmg_objects_read`): SoD's table files every line of a class
/// below 232, AB's format those below 222 but SoD's monoliths and gates past subclass 2, RoE's
/// below 165 likewise; AB's and SoD's decorations 165 to 205 file among RoE's 114 to 161 (their
/// class group); the monsters' prototypes rise by subclass. A generated map has them, its
/// placed objects steps 4 and 5's towns.
#[test]
fn the_originals_prototypes_are_filed_by_the_format() {
    let (_, part) = original("base:faithful");
    let tables = part.generator().unwrap();
    let lines = part.object_templates();
    let filed = |format| Objects::new(lines, &tables.object_settings, format, 0);
    let wanted = |bound: u32, roe_ab: bool| {
        lines
            .iter()
            .filter(|t| {
                t.class < bound && !(roe_ab && (43..=45).contains(&t.class) && t.subclass > 2)
            })
            .count()
    };
    let (roe, ab, sod) = (
        filed(MapFormat::RoE),
        filed(MapFormat::AB),
        filed(MapFormat::SoD),
    );
    assert_eq!(roe.prototypes().len(), wanted(165, true));
    assert_eq!(ab.prototypes().len(), wanted(222, true));
    assert_eq!(sod.prototypes().len(), wanted(232, false));
    assert!(roe.prototypes().len() < ab.prototypes().len());
    assert!(ab.prototypes().len() < sod.prototypes().len());
    for o in [&roe, &ab, &sod] {
        let group = |class: usize| -> Vec<u32> {
            o.group(class)
                .iter()
                .map(|&p| o.prototypes()[p].shape.class)
                .collect()
        };
        assert!(group(165).is_empty() && group(205).is_empty());
        let monsters: Vec<u32> = o
            .group(54)
            .iter()
            .map(|&p| o.prototypes()[p].shape.subclass)
            .collect();
        assert!(!monsters.is_empty() && monsters.is_sorted());
    }
    assert!((114..=161).any(|g| {
        sod.group(g)
            .iter()
            .any(|&p| sod.prototypes()[p].shape.class >= 165)
    }));
    let (core, part) = original("base:faithful");
    let (_, s, seed, _) = corpus_settings(0);
    let map = generate(core, part, &s, seed, &mut Log::default()).unwrap();
    // Steps 4 and 5 placed towns only.
    assert!(!map.objects.placed().is_empty());
    assert!(map.objects.placed().iter().all(|o| o.class == 98));
    let filed = |o: &Objects| -> Vec<_> {
        o.prototypes()
            .iter()
            .map(|p| (p.template, p.shape))
            .collect()
    };
    assert_eq!(filed(&map.objects), filed(&sod));
    // The towns' prototypes are used (+0x8) once per town.
    let used: i32 = map.objects.prototypes().iter().map(|p| p.used).sum();
    assert_eq!(used as usize, map.placed.towns.len());
    assert_eq!(map.objects.zone_count(map.zones.len() - 1).len(), 232);
}

// covers: RMG-TPL-008
/// RMG-QUIRK-24 against the oracle (E14 T3a, `docs/lanes/rmg/ORACLE.md`): the corpus's 2SM2i(2)
/// case with three players puts the third colour on position 7, which no zone has, and the
/// original's map leaves that colour (tan) out; its two 2SM0k cases put both colours on the two
/// zones with Ownership, and the original's maps give both their main towns.
#[test]
fn the_corpus_starts_without_ownership_place_the_colours_as_the_original() {
    let packs = original("base:faithful");
    let two = [Some(0), Some(1), None, None, None, None, None, None];
    for (id, positions) in [
        (
            "s1-3p-2sm2i2",
            [Some(0), Some(1), None, None, None, None, None, Some(2)],
        ),
        ("s1-2p-2sm0k", two),
        ("s1-1p-c1-2sm0k", two),
    ] {
        let case = CORPUS.iter().position(|c| c.0 == id).unwrap();
        let (_, s, seed, _) = corpus_settings(case);
        let map = run(packs, &s, seed).0.unwrap();
        assert_eq!(map.positions, positions, "{id}");
    }
}

// covers: RMG-SET-010, RMG-ZON-001, RMG-ZON-002, RMG-ZON-003, RMG-ZON-004, RMG-ZON-005
// covers: RMG-ZON-006, RMG-ZON-007, RMG-ZON-008, RMG-ZON-009, RMG-ZON-010, RMG-ZON-011
// covers: RMG-ZON-012, RMG-ZON-013, RMG-ZON-014, RMG-TER-001, RMG-TER-002, RMG-TER-003
// covers: RMG-TER-005, RMG-TER-007, RMG-TER-009, RMG-TER-012, RMG-TER-014, RMG-TER-015
// covers: RMG-OBJ-009, RMG-OBJ-010, RMG-OBJ-011, RMG-TWN-001, RMG-TWN-002, RMG-TWN-003
// covers: RMG-TWN-004, RMG-TWN-005
/// The comparison seam (ADR-0029 point 8) over the oracle's corpus: the port's draws before step
/// 1, in it and in steps 2 to 5, and its dumps after the frame and after steps 1 to 5 (the
/// towns' objects with the cells), have the phase oracle's digests
/// (`clients/capture/rmg/phases.txt`), step 2's unwritten bytes as the memory record measured
/// them (`zones.txt`, RMG-QUIRK-1). Step 2's water connections show in step 6's cells (increment
/// 4). RMG-QUIRK-13 and -25 in `faithful` are among them: `xl1-4p-seed1`'s zone 3 and
/// `xl2-8p-t4-islands`' zone 4, human starts with only their border cells, get no town.
#[test]
fn the_corpus_draws_and_dumps_as_the_phase_oracle_to_step_5() {
    let packs = original("base:faithful");
    let kept = kept_phases();
    let memory = kept_zone_memory();
    assert_eq!(kept.len(), CORPUS.len());
    for case in 0..CORPUS.len() {
        let (id, s, seed, _) = corpus_settings(case);
        let mut t = Phases {
            memory: memory.get(id).cloned().unwrap_or_default(),
            ..Phases::default()
        };
        generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap_or_else(|e| panic!("{id}: {e}"));
        let (phases, cells) = &kept[id];
        for phase in ["setup", "template", "1", "2", "3", "4", "5"] {
            assert_eq!(t.phases.get(phase), phases.get(phase), "{id} {phase}");
        }
        for step in [0, 1, 2, 3, 4, 5] {
            assert_eq!(t.cells.get(&step), cells.get(&step), "{id} cells {step}");
        }
    }
}

/// `plan.txt`'s lines by case: the lanes' draws and their digest, the draws of step 6's plan.
fn kept_plan() -> BTreeMap<String, ((usize, String), usize)> {
    include_str!("../../../../clients/capture/rmg/plan.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let field = |name: &str| line.split(' ').find_map(|w| w.strip_prefix(name)).unwrap();
            let (n, digest) = field("lanes=").split_once(':').unwrap();
            let id = line.split(' ').next().unwrap().to_string();
            let lanes = (n.parse().unwrap(), digest.to_string());
            (id, (lanes, field("draws=").parse().unwrap()))
        })
        .collect()
}

// covers: RMG-CON-001, RMG-CON-002, RMG-CON-003, RMG-CON-004, RMG-CON-005, RMG-CON-006
/// Step 6's lanes over the oracle's corpus (`clients/capture/rmg/plan.txt`, E14 T4c): the port's
/// first draws after step 5's end are the original's lanes (`rmg_free_lanes_draw`'s, their
/// count and digest), step 2's unwritten bytes as measured (`zones.txt`). In the 11 cases whose
/// step 6 draws nothing between the lanes and the first connection (no islet), the port draws
/// exactly the lanes' after step 5: the zone walls and the cells of no zone draw nothing.
#[test]
fn the_corpus_draws_step_6s_lanes_as_the_original() {
    let packs = original("base:faithful");
    let kept = kept_plan();
    let memory = kept_zone_memory();
    assert_eq!(kept.len(), CORPUS.len());
    let mut whole = Vec::new();
    for case in 0..CORPUS.len() {
        let (id, s, seed, _) = corpus_settings(case);
        let mut t = Phases {
            memory: memory.get(id).cloned().unwrap_or_default(),
            ..Phases::default()
        };
        generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap_or_else(|e| panic!("{id}: {e}"));
        let ((lanes, digest), plan) = &kept[id];
        let after = &t.log.draws[t.log.ends[5]..];
        assert!(after.len() >= *lanes, "{id}: {} draws", after.len());
        assert_eq!(
            lines_digest(&after[..*lanes]),
            (*lanes, digest.clone()),
            "{id}"
        );
        if plan == lanes {
            assert_eq!(after.len(), *lanes, "{id}");
            whole.push(id);
        }
    }
    assert_eq!(whole.len(), 11);
}

// covers: RMG-ZON-010, RMG-ZON-011
/// The oracle model's own bytes (RMG-QUIRK-1, provisional: the temporary zone's 0BB, a water
/// zone's nine town types) give the oracle's step 2 on 22 of the corpus's 24: `l1-4p-b`'s and
/// `xl2-4p-seed0`'s first and fourteenth water zones have none set (one draw fewer).
#[test]
fn the_oracle_models_own_bytes_give_step_2_on_all_but_two() {
    let packs = original("base:faithful");
    let kept = kept_phases();
    let mut differ = Vec::new();
    for case in 0..CORPUS.len() {
        let (id, s, seed, _) = corpus_settings(case);
        let mut t = Phases::default();
        generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap_or_else(|e| panic!("{id}: {e}"));
        let (phases, cells) = &kept[id];
        if t.phases.get("2") != phases.get("2") || t.cells.get(&2) != cells.get(&2) {
            differ.push(id);
        }
    }
    assert_eq!(differ, ["l1-4p-b", "xl2-4p-seed0"]);
}

/// `water.txt`'s zone generators by case: per water zone its index, town type, terrain,
/// position and size.
fn kept_water() -> BTreeMap<String, Vec<[i32; 7]>> {
    let mut out: BTreeMap<String, Vec<[i32; 7]>> = BTreeMap::new();
    for line in include_str!("../../../../clients/capture/rmg/water.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let words: Vec<&str> = line.split(' ').collect();
        let mut values = vec![words[1].parse().unwrap()];
        values.extend(
            words[2..]
                .iter()
                .map(|w| w.split_once('=').unwrap().1.parse::<i32>().unwrap()),
        );
        out.entry(words[0].to_string())
            .or_default()
            .push(values.try_into().unwrap());
    }
    out
}

// covers: RMG-ZON-010, RMG-ZON-011
/// Step 2's water zones are the original's (`clients/capture/rmg/water.txt`, the memory
/// record's zone generators after step 2, E14 T3d) on the corpus, with the measured bytes: their
/// town types, water, positions (some off the map) and sizes; no other zone generator changes.
#[test]
fn the_corpus_appends_the_originals_water_zones() {
    let packs = original("base:faithful");
    let kept = kept_water();
    let layout = kept_layout();
    let memory = kept_zone_memory();
    assert_eq!(kept.values().map(Vec::len).sum::<usize>(), 44);
    for case in 0..CORPUS.len() {
        let (id, s, seed, _) = corpus_settings(case);
        let mut t = Phases {
            memory: memory.get(id).cloned().unwrap_or_default(),
            ..Phases::default()
        };
        let map = generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap();
        let before = layout[id].len();
        let water: Vec<[i32; 7]> = t.partitioned[before..]
            .iter()
            .enumerate()
            .map(|(k, g)| {
                let [x, y, z] = g.position;
                [(before + k) as i32, g.town_type, g.terrain, x, y, z, g.size]
            })
            .collect();
        assert_eq!(water, kept.get(id).cloned().unwrap_or_default(), "{id}");
        assert_eq!(t.layout, t.partitioned[..before], "{id}");
        assert_eq!(map.template.zones.len(), map.zones.len(), "{id}");
    }
}

// covers: RMG-ZON-010, RMG-ZON-011
/// In `strict-fixed` step 2's unwritten bytes are zeroed (RMG-QUIRK-1): no zone of the step
/// draws a town type, a water zone has none; `faithful`'s temporary and water zones draw.
#[test]
fn strict_fixed_zeroes_step_2s_unwritten_bytes() {
    let case = CORPUS.iter().position(|c| c.0 == "m2-4p").unwrap();
    let (_, s, seed, _) = corpus_settings(case);
    for (profile, draws) in [("base:faithful", true), ("base:strict-fixed", false)] {
        let mut t = Phases::default();
        let map = generate(&original(profile).0, &original(profile).1, &s, seed, &mut t).unwrap();
        let from = t.log.ends[1];
        let step_2 = &t.log.draws[from..t.log.ends[2]];
        assert_eq!(
            step_2.iter().any(|&(site, _)| site == SITE_ZONE_TOWN_TYPE),
            draws,
            "{profile}"
        );
        let water = &map.template.zones[t.layout.len()..];
        assert!(!water.is_empty(), "{profile}");
        for (zone, g) in water.iter().zip(&map.zones[t.layout.len()..]) {
            assert_eq!(
                (zone.town_types == 0, g.town_type == -1),
                (!draws, !draws),
                "{profile}"
            );
            assert_eq!(zone.same_type, draws, "{profile}");
        }
    }
}

/// `layout.txt`'s zone generators by case: per zone its town types, terrain, position and size.
fn kept_layout() -> BTreeMap<String, Vec<[i32; 7]>> {
    let mut out: BTreeMap<String, Vec<[i32; 7]>> = BTreeMap::new();
    for line in include_str!("../../../../clients/capture/rmg/layout.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let words: Vec<&str> = line.split(' ').collect();
        let values: Vec<i32> = words[2..]
            .iter()
            .map(|w| w.split_once('=').unwrap().1.parse().unwrap())
            .collect();
        out.entry(words[0].to_string())
            .or_default()
            .push(values.try_into().unwrap());
    }
    out
}

// covers: RMG-ZON-001, RMG-ZON-002, RMG-ZON-003, RMG-ZON-004, RMG-ZON-005, RMG-ZON-006
// covers: RMG-ZON-007, RMG-ZON-008
/// The zone generators after step 1 are the original's (`clients/capture/rmg/layout.txt`, the
/// phase oracle's record of `rmg_zones_lay_out`'s result, E14 T3c) on the corpus: town types,
/// terrain, position on the map and size.
#[test]
fn the_corpus_lays_the_zones_out_as_the_original() {
    let packs = original("base:faithful");
    let kept = kept_layout();
    assert_eq!(kept.len(), CORPUS.len());
    for case in 0..CORPUS.len() {
        let (id, s, seed, _) = corpus_settings(case);
        // The zone generators as step 1 left them (step 2 appends water zones).
        let mut t = Phases::default();
        generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap();
        let port: Vec<[i32; 7]> = t
            .layout
            .iter()
            .map(|g| {
                let [x, y, z] = g.position;
                [g.town_type, g.town_type_2, g.terrain, x, y, z, g.size]
            })
            .collect();
        assert_eq!(port, kept[id], "{id}");
    }
}

/// A zone row of a test template: its id, kind, base, Ownership, town types and terrains.
fn zone_row(id: i32, kind: &str, base: i32, ownership: i32, towns: &str, terrains: &str) -> String {
    format!(
        "{{ id = {id}, kind = \"{kind}\", base_size = {base}, humans = [0, 8], players = [0, 8], \
         ownership = {ownership}, player_towns = [0, 0, 0, 0], neutral_towns = [0, 0, 0, 0], \
         same_type = false, town_types = \"{towns}\", mines = [0, 0, 0, 0, 0, 0, 0], mine_density \
         = [0, 0, 0, 0, 0, 0, 0], terrain_match = false, terrains = \"{terrains}\", strength = \
         \"average\", monster_match = false, neutral_monsters = false, monster_towns = \
         \"000000000\", treasure = [[0, 0, 0], [0, 0, 0], [0, 0, 0]] }}"
    )
}

/// An rmg pack with one template of `zones`, its zones connected in a chain, and the synthetic
/// pack's object templates and generator tables.
fn one_template(zones: &[String]) -> String {
    let links: Vec<String> = (1..zones.len())
        .map(|k| {
            format!(
                "{{ zones = [{k}, {}], value = 0, wide = false, border_guard = false, humans = \
                 [0, 0], players = [0, 0] }}",
                k + 1
            )
        })
        .collect();
    format!(
        "{}[[template]]\nname = \"Test\"\nsizes = [1, 32]\nzones = [{}]\nconnections = \
         [{}]\n\n{}",
        &SYNTHETIC_RMG[..SYNTHETIC_RMG.find("[[template]]").unwrap()],
        zones.join(", "),
        links.join(", "),
        &SYNTHETIC_RMG[SYNTHETIC_RMG.find("[generator]").unwrap()..]
    )
}

// covers: RMG-ZON-001, RMG-ZON-008
/// RMG-QUIRK-12: a zone without a town type takes its second one from its terrain's row: with
/// the flag `rand() % 4` of the four entries, the -1 and the padding among them; without, of
/// those before the -1, Conflux not in RoE, and none (-1, no draw) when the row starts with -1.
/// The synthetic rows: rough Stronghold, Conflux, -1, Castle; swamp -1, Fortress, Necropolis,
/// Castle. The zones with a town type copy it.
#[test]
fn a_zone_without_a_town_type_draws_its_second_by_the_flag() {
    let rmg = one_template(&[
        zone_row(1, "human_start", 10, 1, "111111111", "00000000"),
        zone_row(2, "computer_start", 10, 2, "000000010", "00000000"),
        zone_row(3, "treasure", 5, 0, "000000000", "00000100"),
        zone_row(4, "treasure", 5, 0, "000000000", "00001000"),
    ]);
    let swamp = [-1, 7, 4, 0];
    for (profile, format, rough, flag) in [
        ("lone:on", MapFormat::SoD, &[6, 8, -1, 0][..], true),
        ("lone:off", MapFormat::SoD, &[6, 8][..], false),
        ("lone:off", MapFormat::RoE, &[6][..], false),
    ] {
        let packs = lone(profile, &rmg);
        let s = Settings {
            format,
            ..settings(&choices(MapSize::Small, 1, 1))
        };
        for seed in 0..16 {
            let (map, log) = run(&packs, &s, seed);
            let map = map.unwrap();
            assert_eq!(map.zones[1].town_type, 7);
            assert_eq!(map.zones[1].town_type_2, 7);
            assert_eq!(map.zones[0].town_type_2, map.zones[0].town_type);
            // Step 1's four (step 2 appends water zones).
            let terrains: Vec<i32> = map.zones.iter().take(4).map(|g| g.terrain).collect();
            assert_eq!(terrains, [0, 0, 5, 4]);
            let drawn: Vec<u32> = log
                .draws
                .iter()
                .filter(|d| d.0 == SITE_ZONE_TERRAIN_TOWN)
                .map(|d| d.1)
                .collect();
            assert_eq!(drawn.len(), 1 + usize::from(flag), "{profile} {seed}");
            assert_eq!(map.zones[2].town_type, -1);
            assert_eq!(
                map.zones[2].town_type_2,
                rough[drawn[0] as usize % rough.len()],
                "{profile} {seed}"
            );
            let fourth = if flag {
                swamp[drawn[1] as usize % 4]
            } else {
                -1
            };
            assert_eq!(map.zones[3].town_type_2, fourth, "{profile} {seed}");
        }
    }
    // A zone's town type is drawn among its set ones (the computer start's one alone: a draw).
    let (_, log) = run(
        &lone("lone:on", &rmg),
        &settings(&choices(MapSize::Small, 1, 1)),
        3,
    );
    let towns = log.draws[..log.ends[1]]
        .iter()
        .filter(|d| d.0 == SITE_ZONE_TOWN_TYPE)
        .count();
    assert_eq!(towns, 2);
}

// covers: RMG-ZON-001
/// A start zone with colour towns whose position's colour has a town choice takes the choice
/// over its drawn town type (synthetic Twin Vales: red on the human start, Tower chosen); the
/// computer's start keeps its draw.
#[test]
fn a_start_zones_colour_choice_replaces_its_town_type() {
    let packs = lone("lone:on", SYNTHETIC_RMG);
    let mut c = choices(MapSize::Medium, 1, 1);
    c.towns[0] = Some(2);
    let s = settings(&c);
    for seed in 0..8 {
        let (map, log) = run(&packs, &s, seed);
        let map = map.unwrap();
        assert_eq!(map.positions[0], Some(0));
        assert_eq!(map.template.zones[1].town_types, 0x1FF);
        assert_eq!(map.zones[0].town_type, 2, "{seed}");
        // Both drawn among their nine types; the first replaced (step 1's draws).
        let drawn: Vec<u32> = log.draws[..log.ends[1]]
            .iter()
            .filter(|d| d.0 == SITE_ZONE_TOWN_TYPE)
            .map(|d| d.1)
            .collect();
        assert_eq!(drawn.len(), 3, "{seed}");
        assert_eq!(map.zones[1].town_type, (drawn[1] % 9) as i32, "{seed}");
    }
}

// covers: RMG-ZON-002, RMG-ZON-004, RMG-ZON-006
/// RMG-QUIRK-3's step 1 cases, where the original divides by zero, are typed errors: a zone with
/// no position (around a zone of base 50 000 every point's squared distance wraps to a negative
/// number, whose root truncates to 0, so none fits) and a layout spanning nothing (two zones of
/// base -1 end within one cell).
#[test]
fn step_1s_divisions_by_zero_are_refused() {
    let s = settings(&choices(MapSize::Medium, 1, 1));
    let wide = one_template(&[
        zone_row(1, "human_start", 50_000, 1, "000000000", "00000000"),
        zone_row(2, "computer_start", 1, 2, "000000000", "00000000"),
    ]);
    let (map, log) = run(&lone("lone:on", &wide), &s, 1);
    assert_eq!(map, Err(GenerateError::NoPosition { zone: 1 }));
    // Zone 0's placement drew, zone 1's did not.
    assert_eq!(log.draws.last().unwrap().0, SITE_ZONE_POSITION);
    assert_eq!(log.steps, [0]);
    let none = one_template(&[
        zone_row(1, "human_start", -1, 1, "000000000", "00000000"),
        zone_row(2, "computer_start", -1, 2, "000000000", "00000000"),
    ]);
    for seed in 0..8 {
        let (map, log) = run(&lone("lone:on", &none), &s, seed);
        assert_eq!(map, Err(GenerateError::LayoutSpan));
        // Three passes over two zones, then nothing.
        let placements = log
            .draws
            .iter()
            .filter(|d| d.0 == SITE_ZONE_POSITION)
            .count();
        assert_eq!(placements, 6);
    }
    for e in [
        GenerateError::NoPosition { zone: 1 },
        GenerateError::LayoutSpan,
    ] {
        assert!(e.to_string().starts_with("step 1: "));
    }
}

// covers: RMG-TER-002
/// RMG-QUIRK-3's step 3 case is a typed error: with islands a zone of base 0 has a scaled size
/// of 0, so its coast draws `rand() % 0` (step 2's borders on the islands' level are straight).
#[test]
fn step_3s_coast_limit_of_0_is_refused() {
    let s = Settings {
        water: Water::Islands,
        ..settings(&choices(MapSize::Medium, 1, 1))
    };
    let small = one_template(&[
        zone_row(1, "human_start", 0, 1, "000000000", "00000000"),
        zone_row(2, "computer_start", 10, 2, "000000000", "00000000"),
    ]);
    let (map, log) = run(&lone("lone:on", &small), &s, 1);
    let error = GenerateError::Terrain(TerrainFault::CoastLimit);
    assert_eq!(map, Err(error.clone()));
    assert_eq!(log.steps, [0, 1, 2]);
    assert_eq!(error.to_string(), "step 3: CoastLimit");
}

// covers: RMG-ZON-012
/// RMG-QUIRK-3's step 2 case is a typed error: a zone of base 0 has a scaled size of 0, so its
/// jagged borders draw `rand() % 0`.
#[test]
fn step_2s_jagged_limit_of_0_is_refused() {
    let s = Settings {
        water: Water::None,
        ..settings(&choices(MapSize::Medium, 1, 1))
    };
    let small = one_template(&[
        zone_row(1, "human_start", 0, 1, "000000000", "00000000"),
        zone_row(2, "computer_start", 10, 2, "000000000", "00000000"),
    ]);
    let (map, log) = run(&lone("lone:on", &small), &s, 1);
    let error = GenerateError::Partition {
        level: 0,
        fault: PartitionFault::Limit,
    };
    assert_eq!(map, Err(error.clone()));
    assert_eq!(log.steps, [0, 1]);
    assert_eq!(error.to_string(), "step 2, level 0: Limit");
}

/// The towns of zone generator `zone`: owner and castle, in placing order.
fn towns_of(map: &GeneratedMap, zone: usize) -> Vec<(i32, bool)> {
    map.placed
        .towns
        .iter()
        .filter(|t| t.zone == zone)
        .map(|t| (t.owner, t.castle))
        .collect()
}

// covers: RMG-TWN-001, RMG-TWN-003
/// The corpus's start zones without Ownership (position -1; RMG-QUIRK-24's zones) get what their
/// rows ask for, as the phase oracle's steps 4 and 5 (the corpus's comparison): their rows ask
/// for no town of their colour, so the colours' slot -1 is never used; 2SM2i(2)'s human start 3
/// and 2SM0k's computer starts 3 and 4 nothing, 2SM2i(2)'s human start 5 and 2SM0k's computer
/// starts 5 to 8 one neutral town each (their least neutral town, a village of their drawn town
/// type), no castle.
#[test]
fn the_corpus_starts_without_ownership_get_their_rows_neutral_towns() {
    let packs = original("base:faithful");
    let memory = kept_zone_memory();
    for (id, nothing, neutral) in [
        ("s1-3p-2sm2i2", &[2][..], &[4][..]),
        ("s1-2p-2sm0k", &[2, 3][..], &[4, 5, 6, 7][..]),
        ("s1-1p-c1-2sm0k", &[2, 3][..], &[4, 5, 6, 7][..]),
    ] {
        let case = CORPUS.iter().position(|c| c.0 == id).unwrap();
        let (_, s, seed, _) = corpus_settings(case);
        let mut t = Phases {
            memory: memory.get(id).cloned().unwrap_or_default(),
            ..Phases::default()
        };
        let map = generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap();
        for &g in nothing.iter().chain(neutral) {
            let zone = &map.template.zones[map.zones[g].zone];
            assert_eq!(zone.ownership, 0, "{id} {g}");
            assert_eq!(zone.player_towns, [0; 4], "{id} {g}");
        }
        for &g in nothing {
            assert!(towns_of(&map, g).is_empty(), "{id} {g}");
        }
        for &g in neutral {
            assert_eq!(towns_of(&map, g), [(-1, false)], "{id} {g}");
        }
    }
}

// covers: RMG-TWN-003, RMG-TWN-005
/// RMG-QUIRK-13 over the packs: a zone without a town type that asks for one least neutral town
/// gets none in step 4 (type -1 refused); with the flag step 5 skips that town as step 4's, so the
/// zone has none; without it step 5 places it, its type drawn among all nine (no type set), at
/// the cell nearest the centre.
#[test]
fn a_least_town_step_4_never_placed_comes_in_step_5_by_the_flag() {
    let treasure = zone_row(3, "treasure", 10, 0, "000000000", "00000001").replace(
        "neutral_towns = [0, 0, 0, 0]",
        "neutral_towns = [1, 0, 0, 0]",
    );
    let rmg = one_template(&[
        zone_row(1, "human_start", 10, 1, "111111111", "00000001"),
        zone_row(2, "computer_start", 10, 2, "111111111", "00000001"),
        treasure,
    ]);
    for (profile, skipped) in [("lone:on", true), ("lone:off", false)] {
        let packs = lone(profile, &rmg);
        let s = settings(&choices(MapSize::Small, 1, 1));
        for seed in 0..8 {
            let (map, log) = run(&packs, &s, seed);
            let map = map.unwrap();
            assert_eq!(map.zones[2].town_type, -1);
            let step_5 = &log.draws[log.ends[4]..log.ends[5]];
            if skipped {
                assert!(towns_of(&map, 2).is_empty(), "{profile} {seed}");
                assert!(step_5.is_empty(), "{profile} {seed}");
            } else {
                assert_eq!(towns_of(&map, 2), [(-1, false)], "{profile} {seed}");
                let sites: Vec<u32> = step_5.iter().map(|d| d.0).collect();
                assert_eq!(
                    sites,
                    [SITE_TOWN_TYPE_ANY, SITE_TOWN_CENTRAL_CELL],
                    "{seed}"
                );
                let town = map.placed.towns.iter().find(|t| t.zone == 2).unwrap();
                let subclass = map.objects.placed()[town.object].subclass;
                assert_eq!(subclass, step_5[0].1 % 9, "{seed}");
            }
        }
    }
}

// covers: RMG-ZON-013, RMG-TWN-002
/// RMG-QUIRK-25 over the corpus: in `faithful` `xl1-4p-seed1`'s zone 3 and `xl2-8p-t4-islands`'
/// zone 4 (human starts whose centre and the cell left of it lie on borders) keep only their
/// border cells and get no town (the corpus's comparison); in `strict-fixed` their fills start
/// from a free cell and each gets its first town, of its colour (the other flags off move the
/// maps too: the cells are not compared).
#[test]
fn a_zone_whose_start_lies_on_borders_gets_its_region_by_the_flag() {
    for (id, g, faithful_cells) in [("xl1-4p-seed1", 3, 133), ("xl2-8p-t4-islands", 4, 225)] {
        let case = CORPUS.iter().position(|c| c.0 == id).unwrap();
        let (_, s, seed, _) = corpus_settings(case);
        let cells = |map: &GeneratedMap| {
            map.cells
                .cells()
                .iter()
                .filter(|c| usize::from(c.zone_byte()) == g)
                .count()
        };
        let mut t = Phases {
            memory: kept_zone_memory().get(id).cloned().unwrap_or_default(),
            ..Phases::default()
        };
        let packs = original("base:faithful");
        let map = generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap();
        assert_eq!(cells(&map), faithful_cells, "{id}");
        assert_eq!(map.zones[g].town, None, "{id}");
        let (fixed, _) = run(original("base:strict-fixed"), &s, seed);
        let fixed = fixed.unwrap();
        assert!(fixed.zones[g].town.is_some(), "{id}");
        let colour = fixed.positions[g].map(i32::from).unwrap();
        assert_eq!(towns_of(&fixed, g)[0].0, colour, "{id}");
    }
}

// covers: RMG-TWN-001
/// RMG-QUIRK-4's step 4 case over the packs: a zone with an Ownership past the eight positions
/// asking for a town of its colour reads its owner outside the colours, a typed error at step 4's
/// end (no step 4 reported).
#[test]
fn a_towns_colour_outside_the_positions_is_refused() {
    let treasure = zone_row(3, "treasure", 10, 10, "111111111", "00000001")
        .replace("player_towns = [0, 0, 0, 0]", "player_towns = [1, 0, 0, 0]");
    let rmg = one_template(&[
        zone_row(1, "human_start", 10, 1, "111111111", "00000001"),
        zone_row(2, "computer_start", 10, 2, "111111111", "00000001"),
        treasure,
    ]);
    let (map, log) = run(
        &lone("lone:on", &rmg),
        &settings(&choices(MapSize::Small, 1, 1)),
        1,
    );
    let error = GenerateError::Objects {
        step: 4,
        fault: h3_rmg::ObjectFault::Colour,
    };
    assert_eq!(map, Err(error.clone()));
    assert_eq!(log.steps, [0, 1, 2, 3]);
    assert_eq!(error.to_string(), "step 4, an object: Colour");
}

// covers: RMG-TWN-002, RMG-TWN-004
/// RMG-QUIRK-23's towns' half, as the original (the corpus's comparison): Dragons' zone 32,
/// purple's computer start asking only for neutral towns by density, takes a neutral castle as
/// its first town (step 5's densities, the castle first on their tie), whose entrance is purple's
/// main town's; the other starts' first towns are their colours'.
#[test]
fn a_start_zone_without_colour_towns_takes_a_neutral_first_town() {
    let id = "l2-2p-c4-islands";
    let case = CORPUS.iter().position(|c| c.0 == id).unwrap();
    let (_, s, seed, _) = corpus_settings(case);
    let mut t = Phases {
        memory: kept_zone_memory().get(id).cloned().unwrap_or_default(),
        ..Phases::default()
    };
    let packs = original("base:faithful");
    let map = generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap();
    let purple = &map.template.zones[31];
    assert_eq!((purple.ownership, purple.player_towns), (6, [0; 4]));
    assert_eq!(map.positions[5], Some(5));
    assert_eq!(towns_of(&map, 31), [(-1, true)]);
    let first = map.placed.towns.iter().position(|t| t.zone == 31).unwrap();
    assert_eq!(map.zones[31].town, Some(map.placed.road_targets[first]));
    for (g, colour) in [(0, 0), (28, 1), (27, 2), (10, 3), (3, 4)] {
        assert_eq!(towns_of(&map, g)[0].0, colour, "{g}");
    }
}
