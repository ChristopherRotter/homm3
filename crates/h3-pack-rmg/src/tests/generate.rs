//! `generate`'s frame over the packs' tables (E14 T2c): the oracle's corpus, the run's
//! adjustments, the flags and the errors.

use h3_rmg::{
    Choices, CrtRand, Fit, GenerateError, GeneratedMap, MapFormat, MapSize, Monsters,
    PositionError, SITE_TEMPLATE, SITE_WATER, Settings, SettingsError, Trace, Water, generate,
    kept_templates, resolve_random,
};
use proptest::prelude::*;
use std::sync::LazyLock;

use super::*;

/// The draws and the steps' ends, as the phase oracle's log has them.
#[derive(Debug, Default)]
struct Log {
    draws: Vec<(u32, u32)>,
    steps: Vec<u8>,
}

impl Trace for Log {
    fn draw(&mut self, site: u32, value: u32) {
        self.draws.push((site, value));
    }

    fn step_end(&mut self, step: u8) {
        self.steps.push(step);
    }
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
        assert_eq!(log.draws, [(SITE_TEMPLATE, first_rand(seed))], "{id}");
        assert_eq!(log.steps, [0], "{id}");
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
        assert_eq!(log.draws.len(), 2);
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
    assert_eq!(log.steps, [0]);
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
            prop_assert_eq!(log.steps, vec![0]);
            prop_assert!(map.settings.water != Water::Random);
        }
    }
}
