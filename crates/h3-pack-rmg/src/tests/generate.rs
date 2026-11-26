//! `generate` over the packs' tables (E14 T2c, T3c): the oracle's corpus against `phases.txt`
//! and `layout.txt` (the comparison seam, ADR-0029 point 8), the run's adjustments, step 1, the
//! flags and the errors.

use std::collections::BTreeMap;

use h3_rmg::{
    Choices, CrtRand, Fit, GenerateError, GeneratedMap, MapFormat, MapSize, Monsters,
    PositionError, SITE_TEMPLATE, SITE_WATER, SITE_ZONE_POSITION, SITE_ZONE_TERRAIN_TOWN,
    SITE_ZONE_TOWN_TYPE, Settings, SettingsError, StepState, Trace, Water, ZoneGenerator, generate,
    kept_templates, resolve_random,
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
/// after step 1.
#[derive(Debug, Default)]
struct Phases {
    log: Log,
    phases: BTreeMap<String, (usize, String)>,
    cells: BTreeMap<u8, String>,
    layout: Vec<ZoneGenerator>,
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
        self.cells.insert(step, digest12(&state.cells.dump(step)));
        if step == 1 {
            self.layout = state.zones.to_vec();
        }
        self.log.step_end(step, state);
    }
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
        assert_eq!(log.steps, [0, 1], "{id}");
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
    assert_eq!(log.steps, [0, 1]);
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

    /// Arbitrary choices, seeds and either profile run the frame over the original tables without
    /// a panic (exit criterion (d)); a map's template is kept, its positions' colours distinct,
    /// one per player and computer (or the error says why not).
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
            prop_assert_eq!(log.steps, vec![0, 1]);
            prop_assert!(map.settings.water != Water::Random);
            // Step 1: a zone generator per kept zone, on the map's levels, a land terrain on
            // level 0 and subterranean or lava on level 1, town types of the nine or none.
            prop_assert_eq!(map.zones.len(), map.template.zones.len());
            for g in &map.zones {
                let z = g.position[2];
                prop_assert!(z == 0 || (z == 1 && run.two_levels));
                prop_assert!((0..=7).contains(&g.terrain));
                prop_assert!(z == 0 || g.terrain == 6 || g.terrain == 7);
                prop_assert!((-1..=8).contains(&g.town_type));
                prop_assert!((-1..=8).contains(&g.town_type_2));
            }
        }
    }
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
// covers: RMG-ZON-006, RMG-ZON-007, RMG-ZON-008
/// The comparison seam (ADR-0029 point 8) over the oracle's corpus: the port's draws before step
/// 1 and in it, and its dumps after the frame and after step 1, have the phase oracle's digests
/// (`clients/capture/rmg/phases.txt`).
#[test]
fn the_corpus_draws_and_dumps_as_the_phase_oracle_to_step_1() {
    let packs = original("base:faithful");
    let kept = kept_phases();
    assert_eq!(kept.len(), CORPUS.len());
    for case in 0..CORPUS.len() {
        let (id, s, seed, _) = corpus_settings(case);
        let mut t = Phases::default();
        generate(&packs.0, &packs.1, &s, seed, &mut t).unwrap_or_else(|e| panic!("{id}: {e}"));
        let (phases, cells) = &kept[id];
        for phase in ["setup", "template", "1"] {
            assert_eq!(t.phases.get(phase), phases.get(phase), "{id} {phase}");
        }
        for step in [0, 1] {
            assert_eq!(t.cells.get(&step), cells.get(&step), "{id} cells {step}");
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
        let map = run(packs, &s, seed).0.unwrap();
        let port: Vec<[i32; 7]> = map
            .zones
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
/// pack's generator tables.
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
        "[[template]]\nname = \"Test\"\nsizes = [1, 32]\nzones = [{}]\nconnections = [{}]\n\n{}",
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
            let terrains: Vec<i32> = map.zones.iter().map(|g| g.terrain).collect();
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
    let towns = log
        .draws
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
        // Both drawn among their nine types; the first replaced.
        let drawn: Vec<u32> = log
            .draws
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
