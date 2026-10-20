//! E05 T3c: `h3 ai bench` on the synthetic scenarios.

use std::path::PathBuf;

use super::bench::{BenchOptions, run};
use super::main;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn options(scenario: &str, threads: u32) -> BenchOptions {
    BenchOptions {
        scenario: root().join("packs/synthetic/scenarios").join(scenario),
        packs_dir: root().join("packs"),
        envs: 5,
        threads,
        calls: 60,
        seed: 3,
        max_steps: 0,
        encounters: None,
    }
}

#[test]
fn every_chosen_action_applies_whatever_the_thread_count() {
    for scenario in ["full.toml", "siege.toml"] {
        let one = run(&options(scenario, 0)).unwrap();
        assert_eq!(one.steps, 5 * 60, "{scenario}");
        assert_eq!(one.refused, 0);
        for threads in [1, 3, 8] {
            let many = run(&options(scenario, threads)).unwrap();
            assert_eq!(
                (many.steps, many.episodes),
                (one.steps, one.episodes),
                "{scenario}, {threads} threads"
            );
        }
    }
    // A step limit ends episodes; the batch resets them in the same call.
    let limited = run(&BenchOptions {
        max_steps: 7,
        ..options("full.toml", 2)
    })
    .unwrap();
    assert!(limited.episodes >= 5 * (60 / 7), "{limited:?}");
}

#[test]
fn the_command_line() {
    let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let scenario = root().join("packs/synthetic/scenarios/duel.toml");
    let packs = root().join("packs");
    let ok = args(&[
        "bench",
        "--scenario",
        scenario.to_str().unwrap(),
        "--packs-dir",
        packs.to_str().unwrap(),
        "--envs",
        "3",
        "--threads",
        "2",
        "--calls",
        "5",
    ]);
    main(&ok).unwrap();
    assert!(main(&[]).unwrap_err().starts_with("usage"));
    assert!(
        main(&args(&["fly"]))
            .unwrap_err()
            .contains("unknown ai command")
    );
    assert!(
        main(&args(&["bench", "--envs"]))
            .unwrap_err()
            .contains("needs a value")
    );
    assert!(
        main(&args(&["bench", "--envs", "x"]))
            .unwrap_err()
            .contains("not a valid number")
    );
    assert!(
        main(&args(&["bench", "--fast", "1"]))
            .unwrap_err()
            .contains("unknown option --fast")
    );
    assert!(
        main(&args(&["bench", "now"]))
            .unwrap_err()
            .contains("unexpected argument")
    );
    let missing = args(&["bench", "--scenario", "nowhere.toml"]);
    assert!(main(&missing).is_err());
    let no_envs = [ok.clone(), args(&["--envs", "0"])].concat();
    assert!(main(&no_envs).unwrap_err().contains("at least one env"));
    // E05 T5b2: the encounter distribution, at its own weights or a curriculum's.
    let encounters = |w: &str| args(&["bench", "--encounters", w, "--envs", "2", "--calls", "5"]);
    main(&encounters("default")).unwrap();
    main(&encounters("0, 0, 1, 1, 0, 0")).unwrap();
    for bad in ["1,2", "a,b,c,d,e,f"] {
        assert!(
            main(&encounters(bad))
                .unwrap_err()
                .contains("not six weights")
        );
    }
    assert!(
        main(&encounters("0,0,0,0,0,0"))
            .unwrap_err()
            .contains("weights")
    );
}

/// E05 T5b2: the batch plays the encounter distribution, the neutral kinds' defender inside the
/// call, its episodes ending and restarting.
#[test]
fn the_bench_plays_the_encounters() {
    let report = run(&BenchOptions {
        encounters: Some([1, 1, 1, 1, 1, 1]),
        calls: 300,
        ..options("nowhere.toml", 2)
    })
    .unwrap();
    assert!(report.steps >= 5 * 300 && report.episodes > 0, "{report:?}");
}

/// E05 T5a: a map whose census is known, written with `h3-map`'s writer: grass with water along
/// the west edge (its coast the shore) and snow along the east edge; Pikemen (10) on the grass,
/// Griffins on the coast and Angels on a Clover Field, their counts drawn (the writer places
/// fixed creatures only); a Stronghold with a citadel among custom buildings and a Castle
/// without a fort.
fn census_map() -> Vec<u8> {
    use h3_map::writer as w;
    let template = |line: &str| w::ObjectTemplate::from_objects_txt(line).unwrap();
    let monster = |tile, line: &str, count| {
        w::MapObject::Monster(w::Monster {
            tile,
            template: template(line),
            count,
            disposition: 2,
            never_flees: false,
            fixed_count: false,
        })
    };
    let creature = |id: u16| {
        format!(
            "AvWPike.def 011111111111111111111111111111111111111111111111 \
             100000000000000000000000000000000000000000000000 111111111 000000100 54 {id} 2 0"
        )
    };
    let town = |faction: u32| {
        template(&format!(
            "AVCstrx0.def 000001110000011110001111111111111111111111111111 \
             001000000000000000000000000000000000000000000000 011111111 011111111 98 {faction} 1 0"
        ))
    };
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 0x1FF,
    });
    let map = w::Map {
        name: "Census".to_owned(),
        description: String::new(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![
            w::Patch {
                terrain: w::Terrain::Water,
                from: (0, 0),
                to: (5, 35),
            },
            w::Patch {
                terrain: w::Terrain::Snow,
                from: (30, 0),
                to: (35, 35),
            },
        ],
        roads: Vec::new(),
        players,
        objects: vec![
            w::MapObject::Ground(w::Ground {
                tile: (20, 20),
                template: template(
                    "AVXcf0.def 111111111111111111111111111111111111111111111111 \
                     000000000000000000000000000000000000000000000000 111111111 011111111 222 0 0 1",
                ),
            }),
            monster((15, 15), &creature(0), 10),
            monster((6, 10), &creature(4), 0),
            monster((19, 19), &creature(12), 0),
            w::MapObject::Town(w::Town {
                tile: (14, 30),
                template: town(6),
                owner: None,
                garrison: None,
                tight_formation: false,
                buildings: w::TownBuildings::Custom {
                    built: [0x58, 0, 0, 0, 0, 0],
                    forbidden: [0; 6],
                },
            }),
            w::MapObject::Town(w::Town {
                tile: (26, 30),
                template: town(0),
                owner: None,
                garrison: None,
                tight_formation: false,
                buildings: w::TownBuildings::Standard { fort: false },
            }),
        ],
        underground: None,
    };
    w::gzip(&w::write(&map).unwrap())
}

#[test]
fn the_census_counts_a_known_map() {
    use super::census::{Census, original_rules, run, to_toml};

    let dir = std::env::temp_dir().join(format!("h3-ai-census-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (one, twin, broken) = (dir.join("a.h3m"), dir.join("b.h3m"), dir.join("c.h3m"));
    std::fs::write(&one, census_map()).unwrap();
    std::fs::write(&twin, census_map()).unwrap();
    std::fs::write(&broken, b"not a map").unwrap();
    std::fs::write(dir.join("notes.txt"), b"not read").unwrap();
    let rules = original_rules().unwrap();
    let files = super::census::map_files(std::slice::from_ref(&dir)).unwrap();
    assert_eq!(files, [one.clone(), twin.clone(), broken.clone()]);
    let (census, skipped): (Census, _) = run(&files, &rules).unwrap();
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].path, broken);
    assert!(skipped[0].reason.starts_with("read:"), "{skipped:?}");

    assert_eq!(census.census.maps, 2);
    assert_eq!(
        (census.census.id.as_str(), census.census.version),
        ("census", 1)
    );
    let maps = &census.maps;
    assert_eq!((maps.roe, maps.ab, maps.sod, maps.two_levels), (0, 0, 2, 0));
    assert_eq!(maps.terrain_twins, 1);
    assert_eq!(
        (maps.size.s, maps.size.m + maps.size.l + maps.size.xl),
        (2, 0)
    );
    let tiles = &census.tiles;
    assert_eq!(tiles.total, 2 * 36 * 36);
    assert_eq!(tiles.water, 2 * 6 * 36);
    assert_eq!((tiles.land, tiles.impassable), (2 * 30 * 36, 0));
    assert_eq!((tiles.coast, tiles.underground_land), (2 * 36, 0));
    assert!(tiles.open_land < tiles.land && tiles.open_water == tiles.water);
    let open: u64 = census.place.iter().map(|p| p.open_land).sum();
    assert_eq!(open, tiles.open_land);

    let monsters = &census.monsters;
    assert_eq!((monsters.placements, monsters.fixed_count), (6, 2));
    let level = |l: &str| monsters.level.get(l).map(|c| (c.fixed, c.random));
    assert_eq!(
        (level("1"), level("3"), level("7"), level("2")),
        (Some((2, 0)), Some((2, 0)), Some((2, 0)), None)
    );
    let place = |terrain: &str, field: Option<&str>| {
        census
            .place
            .iter()
            .find(|p| p.terrain == terrain && p.battlefield.as_deref() == field)
            .map(|p| p.monsters)
    };
    assert_eq!(place("base:grass", None), Some(2));
    assert_eq!(place("base:desert", Some("base:shore")), Some(2));
    assert_eq!(place("base:grass", Some("sod:clover_field")), Some(2));
    assert_eq!(place("base:snow", None), Some(0));
    assert_eq!(census.objects.special_grounds, 2);
    assert_eq!(census.objects.guarded_piles, 0);
    let forts = |f: &str| {
        census
            .towns
            .get(f)
            .map(|t| (t.none, t.fort, t.citadel, t.castle))
    };
    assert_eq!(forts("base:stronghold"), Some((0, 0, 2, 0)));
    assert_eq!(forts("base:castle"), Some((2, 0, 0, 0)));

    // The file reads back as the census it was written from.
    let text = to_toml(&census).unwrap();
    assert!(text.starts_with("# The census"));
    let back: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(back["census"]["maps"].as_integer(), Some(2));
    assert_eq!(
        back["place"].as_array().map(Vec::len),
        Some(census.place.len())
    );

    // The command line: a folder, a file, `--out`; the errors.
    let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let out = dir.join("census.toml");
    main(&args(&[
        "census",
        one.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]))
    .unwrap();
    let written: toml::Value = toml::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(written["census"]["maps"].as_integer(), Some(1));
    main(&args(&["census", one.to_str().unwrap()])).unwrap();
    assert!(
        main(&args(&["census"]))
            .unwrap_err()
            .contains("needs a maps folder")
    );
    assert!(
        main(&args(&["census", "x", "--fast", "1"]))
            .unwrap_err()
            .contains("unknown option --fast")
    );
    assert!(
        main(&args(&["census", "nowhere"]))
            .unwrap_err()
            .contains("no such file")
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_towns_fort_is_its_highest_fort_building() {
    use super::census::fort_level;
    use h3_map::model::Buildings;
    let custom = |first: u8| Buildings::Custom {
        built: [first, 0, 0, 0, 0, 0],
        forbidden: [0; 6],
    };
    assert_eq!(fort_level(&Buildings::Standard { fort: false }), 0);
    assert_eq!(fort_level(&Buildings::Standard { fort: true }), 1);
    assert_eq!(fort_level(&custom(0x47)), 0); // a town hall, a city hall, a capitol, a tavern
    assert_eq!(fort_level(&custom(0x08)), 1);
    assert_eq!(fort_level(&custom(0x18)), 2);
    assert_eq!(fort_level(&custom(0x38)), 3);
}

/// E05 T9b, E06 T3a: `h3 ai eval`'s held-out sets are the Python evaluation's (the ladder's row
/// stands beside its table): the seeds and the report set's size are one.
#[test]
fn the_eval_seeds_are_the_python_evaluations() {
    let python = include_str!("../../../../python/h3env/train/evaluate.py");
    let constant = |name: &str| {
        let prefix = format!("{name} = ");
        let value = python
            .lines()
            .find_map(|l| l.strip_prefix(prefix.as_str()))
            .unwrap()
            .replace('_', "");
        value.strip_prefix("0x").map_or_else(
            || value.parse().unwrap(),
            |hex| u64::from_str_radix(hex, 16).unwrap(),
        )
    };
    assert_eq!(constant("EVAL_SEED"), super::eval::EVAL_SEED);
    assert_eq!(constant("REPORT_V1_SEED"), super::eval::REPORT_V1_SEED);
    assert_eq!(
        constant("REPORT_V1_ENVS"),
        u64::from(super::eval::REPORT_V1_ENVS)
    );
}

/// E05 T7: `h3 ai eval` plays every ordered pair on the same episodes, the same way twice; the
/// bots win most games against Random; the ratings follow; its arguments are checked.
#[test]
fn the_eval_runner_rates_the_policies() {
    use super::eval::{self, EVAL_SEED, EvalOptions, Stratum};
    let options = EvalOptions {
        set: "v0".to_owned(),
        policies: eval::policies("random, greedy,aggressor").unwrap(),
        strata: Stratum::v0(12, 4, EVAL_SEED, [0, 0, 1, 1, 1, 0]),
        max_steps: 2000,
    };
    let sampler = super::bench::sampler().unwrap();
    let report = eval::run(&sampler, &options).unwrap();
    assert_eq!(report, eval::run(&sampler, &options).unwrap());
    assert_eq!(report.names, ["random", "greedy", "aggressor"]);
    for a in 0..3 {
        for b in (0..3).filter(|&b| b != a) {
            assert_eq!(report.pairs[a][b][0].episodes, 12);
            let both = report.win_rate(a, b) + report.win_rate(b, a);
            assert!(both <= 1.0 + 1e-9, "{a} {b}: {both}");
            assert!(
                (report.value(a, b) + report.value(b, a)).abs() < 1e-6,
                "zero-sum"
            );
            assert!((0.0..0.5).contains(&report.interval(a, b)));
        }
    }
    assert!(report.win_rate(1, 0) > 0.5 && report.win_rate(2, 0) > 0.5);
    assert!(report.ratings[0].rating < report.ratings[1].rating.min(report.ratings[2].rating));
    let text = report.to_text();
    assert!(
        text.contains("win rate") && text.contains("Glicko-2") && !text.contains("win rate in"),
        "{text}"
    );
    assert!(eval::policies("random,minimax").is_err());
    let one = EvalOptions {
        policies: eval::policies("greedy").unwrap(),
        ..options
    };
    assert!(eval::run(&sampler, &one).is_err());
    let words = |w: &[&str]| w.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert!(main(&words(&["eval", "--policies", "nobody"])).is_err());
    assert!(main(&words(&["eval", "--set", "v0", "--seed", "0xZZ"])).is_err());
    assert!(main(&words(&["eval", "--colour", "red"])).is_err());
    assert!(main(&words(&["eval", "--max-steps", "0"])).is_err());
    let v0 = ["eval", "--set", "v0", "--policies", "random,greedy"];
    main(&words(
        &[&v0[..], &["--episodes", "2", "--envs", "2", "--seed", "7"]].concat(),
    ))
    .unwrap();
}

/// E06 T3a: the report set v1, one stratum per kind with a player on both sides, the first
/// episode of each env; the overall rate weighs the kinds as the distribution does (4 : 2 : 1),
/// its interval is the stratified one of design §5; the JSON table and the command line.
#[test]
fn the_report_set_is_stratified_by_kind() {
    use super::eval::{self, EvalOptions, EvalReport, PairResult, REPORT_V1_SEED, Stratum};
    let strata = Stratum::report_v1(2).unwrap();
    let described: Vec<_> = strata
        .iter()
        .map(|s| (s.name.as_str(), s.weight, s.seed, s.envs, s.per_env))
        .collect();
    assert_eq!(
        described,
        [
            ("hero", 100, REPORT_V1_SEED + 2, 2, 1),
            ("siege_hero", 50, REPORT_V1_SEED + 3, 2, 1),
            ("siege_garrison", 25, REPORT_V1_SEED + 4, 2, 1),
        ]
    );
    assert_eq!(strata[1].kinds, [0, 0, 0, 1, 0, 0]);
    assert!(Stratum::report_v1(0).is_err() && Stratum::report_v1(501).is_err());
    let options = EvalOptions {
        set: "report-v1".to_owned(),
        policies: eval::policies("random,aggressor").unwrap(),
        strata,
        max_steps: 2000,
    };
    let report = eval::run(&super::bench::sampler().unwrap(), &options).unwrap();
    assert_eq!(report.episodes_per_pair(), 6);
    assert!((0..3).all(|s| report.pairs[0][1][s].episodes == 2));
    let by_kind: f64 = [4.0, 2.0, 1.0]
        .iter()
        .enumerate()
        .map(|(s, w)| w * report.stratum_rate(1, 0, s))
        .sum();
    assert!((report.win_rate(1, 0) - by_kind / 7.0).abs() < 1e-12);
    assert!(report.to_text().contains("win rate in siege_garrison"));
    let json = report.to_json();
    for key in [
        "\"set\": \"report-v1\"",
        "\"stratum\": \"siege_garrison\"",
        "\"seed\": \"0x5eede7a100010003\"",
        "\"engine\": {\"encounters\": 1",
        "\"by_stratum\": {\"hero\"",
    ] {
        assert!(json.contains(key), "{key}: {json}");
    }
    assert_eq!(json.matches('{').count(), json.matches('}').count());
    // Design §5: 0.75 on every kind and side, 500 games each, is ±1.8 points.
    let pair = |wins| PairResult {
        episodes: 500,
        wins,
        ..PairResult::default()
    };
    let known = EvalReport {
        strata: Stratum::report_v1(500).unwrap(),
        pairs: vec![
            vec![Vec::new(), vec![pair([375, 0]); 3]],
            vec![vec![pair([0, 375]); 3], Vec::new()],
        ],
        ..report
    };
    assert!((known.win_rate(0, 1) - 0.75).abs() < 1e-12);
    assert!(
        (known.interval(0, 1) - 0.0176).abs() < 1e-4,
        "{}",
        known.interval(0, 1)
    );
    // The command line: the set fixes its episodes; the table goes to a file.
    let words = |w: &[&str]| w.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    for (bad, message) in [
        (&["--set", "v9"][..], "unknown set"),
        (&["--envs", "3"], "fixes its episodes"),
        (&["--encounters", "default"], "fixes its episodes"),
        (&["--episodes", "501"], "per kind"),
    ] {
        let error = main(&words(&[&["eval"][..], bad].concat())).unwrap_err();
        assert!(error.contains(message), "{error}");
    }
    let out = std::env::temp_dir().join(format!("h3-eval-{}.json", std::process::id()));
    let path = out.to_str().unwrap();
    let run = ["eval", "--episodes", "1", "--policies", "aggressor,greedy"];
    main(&words(&[&run[..], &["--json", path]].concat())).unwrap();
    let written = std::fs::read_to_string(&out).unwrap();
    std::fs::remove_file(&out).unwrap();
    assert!(written.starts_with("{\n  \"schema\": 1,") && written.contains("\"greedy\""));
    let nowhere = [&run[..], &["--json", "/nonexistent/dir/table.json"]].concat();
    assert!(main(&words(&nowhere)).is_err());
}
