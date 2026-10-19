//! E05 T5b1: the encounter distribution's files, the sampler's determinism and validity, and the
//! sampled-scenario check that every dimension occurs at its weight (design §4.2).

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

use h3_contracts::FortLevel;

use super::*;
use crate::batch::Scenario;
use crate::rollout;

/// The sampler of the shipped distribution on the original packs (`base:faithful`).
pub(crate) static SAMPLER: LazyLock<Sampler> = LazyLock::new(|| {
    let profile = "base:faithful";
    let (_, adventure) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile).unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .unwrap();
    Sampler::new(Arc::new(battle), Arc::new(adventure), &EncounterTable::v1()).unwrap()
});

/// The encounters of seeds `0..n` drawn at A1's shares (the neutral kinds included).
fn samples(n: u64) -> Vec<Encounter> {
    let sampler = SAMPLER.clone().with_shares();
    (0..n).map(|seed| sampler.sample(seed)).collect()
}

/// Whether `observed` of `n` draws fits the probability `p`: within five standard deviations
/// and two draws.
fn fits(observed: usize, n: usize, p: f64) -> bool {
    let expected = n as f64 * p;
    let sd = (expected * (1.0 - p)).sqrt();
    (observed as f64 - expected).abs() <= 5.0 * sd + 2.0
}

/// Checks the counts of `observed` (by category) against `weights` (by the same category):
/// each fits its share, and no category of weight 0 occurs.
fn check<K: Ord + Copy + std::fmt::Debug>(
    what: &str,
    observed: &BTreeMap<K, usize>,
    weights: &BTreeMap<K, u64>,
) {
    let n: usize = observed.values().sum();
    let total: u64 = weights.values().sum();
    assert!(n > 0, "{what}: no draw");
    for (key, &weight) in weights {
        let count = observed.get(key).copied().unwrap_or(0);
        let p = weight as f64 / total as f64;
        assert!(
            fits(count, n, p),
            "{what} {key:?}: {count} of {n}, expected {p:.4}"
        );
    }
    assert!(
        observed
            .keys()
            .all(|key| weights.get(key).is_some_and(|&w| w > 0)),
        "{what}: a category of weight 0 was drawn: {observed:?}"
    );
}

/// The counts of `values`.
fn count<K: Ord>(values: impl IntoIterator<Item = K>) -> BTreeMap<K, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

#[test]
fn the_shipped_files_parse() {
    let table = EncounterTable::v1();
    assert_eq!(table.kinds.iter().map(|k| k.share).sum::<u32>(), 1000);
    assert_eq!(table.kinds.iter().map(|k| k.weight).sum::<u32>(), 175);
    for kind in Kind::ALL {
        assert_eq!(
            table.kinds[kind.index()].weight == 0,
            kind.neutral(),
            "{kind:?}"
        );
    }
    // The census: 13 863 monsters, the 63 of any level spread over the seven.
    assert_eq!(table.level_weight.iter().sum::<u32>(), 13_863 * 7);
    assert_eq!(table.places.len(), 70);
    assert_eq!(table.census_forts, [306, 1088, 26, 45]);
    // Conflux has no named town: its weight is the random towns' ninth.
    let conflux = table.factions.iter().find(|f| f.0 == "ab:conflux").unwrap();
    assert_eq!(conflux.1, 875);
    assert_eq!(table.ratio.0[10], 1800);
    assert_eq!(table.k.0[10], 1000);
    assert_eq!(
        (
            table.ratio.bin(0),
            table.ratio.bin(1800),
            table.ratio.bin(9999)
        ),
        (0, 10, 19)
    );
}

#[test]
fn a_table_out_of_shape_is_refused() {
    let with = |from: &str, to: &str| {
        EncounterTable::parse(&ENCOUNTERS_V1.replacen(from, to, 1), CENSUS_V1)
    };
    let table_error = |result: Result<EncounterTable, EncounterError>, name: &str| matches!(result, Err(EncounterError::Table { field, .. }) if field == name);
    assert_eq!(
        with("census = \"census-v1\"", "census = \"census-v2\""),
        Err(EncounterError::Census("census-v1".to_owned()))
    );
    assert!(table_error(
        with("version = 1", "version = 2"),
        "distribution"
    ));
    assert!(table_error(with("[kind.guards]", "[kind.banks]"), "kind"));
    assert!(table_error(
        with("week = [1000, 800,", "week = [800,"),
        "kind.week"
    ));
    assert!(table_error(
        with("ratio = [500, 978,", "ratio = [1500, 978,"),
        "neutral.ratio"
    ));
    assert!(table_error(
        with("stacks = 3", "stacks = 8"),
        "guards.stacks"
    ));
    assert!(table_error(
        with("levels_per_artifact = 2", "levels_per_artifact = 0"),
        "hero.levels_per_artifact"
    ));
    assert!(table_error(
        with("level_from_week = [1,", "level_from_week = [2,"),
        "army.level_from_week"
    ));
    assert!(table_error(
        with("[150, 650, 120, 80]", "[0, 0, 0, 0]"),
        "siege.fort"
    ));
    assert!(matches!(
        with("why = \"A11", "whence = \"A11"),
        Err(EncounterError::Toml(_))
    ));
    let zero = [
        "share = 780",
        "share = 20\n",
        "share = 100",
        "share = 50",
        "share = 25",
    ]
    .iter()
    .fold(ENCOUNTERS_V1.to_owned(), |text, share| {
        text.replace(
            share,
            &format!("share = 0{}", if share.ends_with('\n') { "\n" } else { "" }),
        )
    });
    assert!(table_error(
        EncounterTable::parse(&zero, CENSUS_V1),
        "kind.share"
    ));
    // The kinds' weights too (E05 T9b: all 0 would leave the default sampler nothing to draw).
    let zero = ["weight = 100", "weight = 50", "weight = 25"]
        .iter()
        .fold(ENCOUNTERS_V1.to_owned(), |text, w| {
            text.replace(w, "weight = 0")
        });
    assert!(table_error(
        EncounterTable::parse(&zero, CENSUS_V1),
        "kind.weight"
    ));
    assert!(matches!(
        EncounterTable::parse(ENCOUNTERS_V1, "census = 1"),
        Err(EncounterError::Toml(_))
    ));
    for error in [
        EncounterError::Toml("x".to_owned()),
        EncounterError::Census("census-v1".to_owned()),
        EncounterError::Rules("x"),
    ] {
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn unknown_keys_are_refused() {
    let table = |from: &str, to: &str| {
        EncounterTable::parse(&ENCOUNTERS_V1.replacen(from, to, 1), CENSUS_V1).unwrap()
    };
    let sampler = |table: &EncounterTable| {
        Sampler::new(
            Arc::clone(SAMPLER.battle()),
            Arc::clone(SAMPLER.adventure()),
            table,
        )
    };
    let unknown = |result: Result<Sampler, EncounterError>, kind: &str| matches!(result, Err(EncounterError::Unknown { what, .. }) if what == kind);
    assert!(unknown(
        sampler(&table("\"base:deck\"", "\"base:nowhere\"")),
        "battlefield"
    ));
    assert!(unknown(
        sampler(&table("\"base:spell_book\"", "\"base:no_book\"")),
        "artifact"
    ));
    assert!(unknown(
        sampler(&table("\"ab:conflux\"]", "\"ab:atlantis\"]")),
        "faction"
    ));
    assert!(unknown(
        sampler(&table(
            "terrain = \"base:grass\"",
            "terrain = \"base:clouds\""
        )),
        "terrain"
    ));
    assert!(matches!(
        sampler(&table("[\"base:ballista\", ", "[\"base:spell_book\", ")),
        Err(EncounterError::Rules(_))
    ));
    assert!(matches!(
        sampler(&table("\"base:castle\" = 4", "\"base:atlantis\" = 4")),
        Err(EncounterError::Rules(_))
    ));
    assert!(SAMPLER.clone().with_weights([0; 6]).is_err());
    let hero_only = SAMPLER.clone().with_weights([0, 0, 1, 0, 0, 0]).unwrap();
    assert_eq!(hero_only.weights(), [0, 0, 1, 0, 0, 0]);
    assert!((0..20).all(|seed| hero_only.sample(seed).kind == Kind::Hero));
}

#[test]
fn a_seed_gives_one_encounter() {
    let sampler = SAMPLER.clone().with_shares();
    for seed in [0, 1, 7, u64::MAX] {
        assert_eq!(sampler.sample(seed), sampler.sample(seed));
    }
    assert_ne!(sampler.sample(1).setup, sampler.sample(2).setup);
}

#[test]
fn the_default_leaves_the_neutral_kinds_out() {
    let kinds = count((0..2_000).map(|seed| SAMPLER.sample(seed).kind));
    let weights = Kind::ALL
        .iter()
        .map(|&k| (k, u64::from(SAMPLER.weights()[k.index()])))
        .collect();
    check("kind", &kinds, &weights);
    assert!(kinds.keys().all(|kind| !kind.neutral()), "{kinds:?}");
}

#[test]
fn every_sample_is_a_battle_the_engine_plays() {
    let sampler = SAMPLER.clone().with_shares();
    let battle = sampler.battle();
    for seed in 0..1_500 {
        let encounter = sampler.sample(seed);
        let setup = &encounter.setup;
        let (attacker, defender) = (&setup.attacker.stacks, &setup.defender.stacks);
        assert!(!attacker.is_empty() && !defender.is_empty(), "seed {seed}");
        assert!(attacker.len() <= 7 && defender.len() <= 7, "seed {seed}");
        assert!(setup.heroes[0].is_some());
        let defended = matches!(encounter.kind, Kind::Hero | Kind::SiegeHero);
        assert_eq!(setup.heroes[1].is_some(), defended);
        assert_eq!(setup.siege.is_some(), encounter.kind >= Kind::SiegeHero);
        assert_eq!(
            encounter.seats[1] == Seat::External,
            !encounter.kind.neutral()
        );
        let scenario = Scenario::new(Arc::clone(battle), setup, true, 1, encounter.seats);
        assert!(
            scenario.is_ok(),
            "seed {seed}: {encounter:?}: {:?}",
            scenario.err()
        );
        if seed % 50 == 0 {
            let mut env = scenario.unwrap().env(seed, 0).unwrap();
            let played = rollout::random_rollout(&mut env, seed, 3_000);
            assert!(played.steps > 0, "seed {seed}");
        }
    }
}

#[test]
fn every_hero_is_valid() {
    let sampler = &*SAMPLER;
    let core = sampler.core();
    let mut rng = Rng::from_seed(5);
    for faction in 0..sampler.factions.len() {
        for week in 1..=WEEKS as u8 {
            for _ in 0..8 {
                let details = sampler.hero.details(sampler, &mut rng, faction, week);
                assert_eq!(h3_heroes::validate(&details, core), Ok(()), "{details:?}");
                assert_eq!(details.mana, h3_heroes::max_mana(&details, core));
                let (hero, _) = sampler.hero.battle(core, &details);
                assert!(hero.magic.spell_book.is_some(), "{details:?}");
                assert_eq!(hero.attack, h3_heroes::attack_defense(&details, core).0);
            }
        }
    }
}

#[test]
fn every_dimension_occurs_at_its_weight() {
    let all = samples(30_000);
    let table = SAMPLER.table();
    let kinds = |ks: &'static [Kind]| all.iter().filter(move |e| ks.contains(&e.kind));
    const NEUTRAL: &[Kind] = &[Kind::Wandering, Kind::Guards];

    // A1: the kinds.
    let shares = Kind::ALL
        .iter()
        .map(|&k| (k, u64::from(table.kinds[k.index()].share)))
        .collect();
    check("kind", &count(all.iter().map(|e| e.kind)), &shares);

    // A2: the weeks of each kind.
    for kind in Kind::ALL {
        let weeks = (1..=WEEKS as u8)
            .map(|w| {
                (
                    w,
                    u64::from(table.kinds[kind.index()].week[usize::from(w) - 1]),
                )
            })
            .collect();
        let drawn = count(all.iter().filter(|e| e.kind == kind).map(|e| e.week));
        check(kind.key(), &drawn, &weeks);
    }

    // A3: the neutral levels from week 7, where every level weighs its census count and no
    // stack is drawn again (A4's cap is far above them).
    let late = kinds(NEUTRAL).filter(|e| e.week >= 7);
    assert!(late.clone().all(|e| e.facts.redraws == 0));
    let levels = (1..=LEVELS as u8)
        .map(|l| (l, u64::from(table.level_weight[usize::from(l) - 1])))
        .collect();
    check(
        "level",
        &count(late.map(|e| e.facts.level.unwrap())),
        &levels,
    );
    // Earlier: no level 7 before week 4, no level 5 in week 1 (the ramp).
    for e in kinds(NEUTRAL) {
        let level = e.facts.level.unwrap();
        assert!(level < 7 || e.week >= 4, "{e:?}");
        assert!(level < 5 || e.week >= 2, "{e:?}");
    }

    // A10: the neutral places by their monsters, the hero battles' by open land.
    let by = |weight: fn(&Place) -> u32| {
        table
            .places
            .iter()
            .enumerate()
            .map(|(i, p)| (i, u64::from(weight(p))))
            .collect::<BTreeMap<_, _>>()
    };
    let neutral_places = count(kinds(NEUTRAL).map(|e| e.facts.place.unwrap()));
    check("neutral place", &neutral_places, &by(|p| p.monsters));
    let hero_places = count(kinds(&[Kind::Hero]).map(|e| e.facts.place.unwrap()));
    check("hero place", &hero_places, &by(|p| p.open_land));

    // A4: the ratio's bins are even (5 % each), and at least 0.5.
    let ratios = kinds(NEUTRAL).map(|e| e.facts.ratio);
    assert!(ratios.clone().all(|r| r >= 500));
    let even: BTreeMap<usize, u64> = (0..20).map(|b| (b, 1)).collect();
    check("ratio", &count(ratios.map(|r| table.ratio.bin(r))), &even);
    // A5: k and the attacker's ratio likewise.
    let heroes = kinds(&[Kind::Hero, Kind::SiegeHero]);
    check(
        "k",
        &count(heroes.clone().map(|e| table.k.bin(e.facts.k))),
        &even,
    );
    let attacker = count(heroes.map(|e| table.attacker_ratio.bin(e.facts.ratio)));
    check("attacker ratio", &attacker, &even);

    // A7, A8: the factions of heroes and towns.
    let factions: BTreeMap<FactionId, u64> = table
        .factions
        .iter()
        .map(|(key, w)| (SAMPLER.core().faction_by_key(key).unwrap(), u64::from(*w)))
        .collect();
    let defended = &[
        Kind::Hero,
        Kind::SiegeHero,
        Kind::SiegeGarrison,
        Kind::SiegeNeutral,
    ];
    let drawn = count(kinds(defended).map(|e| e.facts.factions[1].unwrap()));
    check("faction", &drawn, &factions);

    // A7: a hero's level by week, about the table's (± the spread, evenly).
    for week in 1..=WEEKS as u8 {
        let levels: Vec<u32> = all
            .iter()
            .filter(|e| e.week == week)
            .map(|e| u32::from(e.facts.hero_levels[0].unwrap()))
            .collect();
        let mean = f64::from(levels.iter().sum::<u32>()) / levels.len() as f64;
        let expected = f64::from(table.hero_level[usize::from(week) - 1]);
        assert!(
            (mean - expected).abs() < 0.3,
            "week {week}: mean level {mean}"
        );
    }

    // A8: the forts from week 7 by the last row; a neutral town's by the census.
    let rows = |weights: [u32; 4]| (0..4).map(|i| (i, u64::from(weights[i]))).collect();
    let fort = |e: &Encounter| e.facts.fort.unwrap() as usize;
    let late_forts = count(
        kinds(&[Kind::SiegeHero, Kind::SiegeGarrison])
            .filter(|e| e.week >= 7)
            .map(fort),
    );
    check("fort", &late_forts, &rows(*table.forts.last().unwrap()));
    let neutral_forts = count(kinds(&[Kind::SiegeNeutral]).map(fort));
    check("neutral fort", &neutral_forts, &rows(table.census_forts));
    assert!(kinds(&[Kind::SiegeHero]).any(|e| e.facts.fort == Some(FortLevel::Castle)));
}

#[test]
fn armies_follow_the_build_order() {
    let sampler = &*SAMPLER;
    let core = sampler.core();
    // A5: Castle's summed growth value by week (levels 1-4, then 5, 6, 7 from weeks 2 to 4).
    let castle = sampler
        .factions
        .iter()
        .position(|f| core.factions()[f.faction.index()].key.as_str() == "base:castle")
        .unwrap();
    let production: Vec<u64> = [1, 4, 8, 16]
        .map(|w| sampler.production(castle, w))
        .to_vec();
    assert_eq!(production, [6_491, 43_132, 110_560, 245_416]);
    // A6: one stack a built level; week 1 has levels 1 to 4 only, week 16 all upgraded.
    let mut rng = Rng::from_seed(3);
    assert_eq!(sampler.army(&mut rng, castle, 1, 6_491).len(), 4);
    let late = sampler.army(&mut rng, castle, 16, 245_416);
    assert_eq!(late.len(), 7);
    assert!(
        late.iter().all(|s| core.upgrades(s.creature).is_empty()),
        "{late:?}"
    );
    // What a level's share cannot buy goes to the level below: at most a level-1 creature lost.
    for value in [100, 2_000, 6_730, 50_000] {
        let army = sampler.army(&mut rng, castle, 4, value);
        let worth = sampler.value(&army);
        assert!(
            worth <= value && worth + 300 > value,
            "{value}: {worth} {army:?}"
        );
    }
    // A worthless army still has one creature.
    assert_eq!(sampler.army(&mut rng, castle, 1, 0).len(), 1);
    // A garrison is one week's growth of the built levels.
    let garrison = sampler.garrison(&mut rng, castle, 1);
    let counts: Vec<u32> = garrison.iter().map(|s| s.count).collect();
    assert_eq!(counts, [14, 9, 7, 4]);
}

#[test]
fn guards_divide_evenly() {
    let creature = SAMPLER.levels[0][0];
    let counts = |count, stacks| {
        divide(creature, count, stacks)
            .iter()
            .map(|s| s.count)
            .collect::<Vec<_>>()
    };
    assert_eq!(counts(10, 3), [4, 3, 3]);
    assert_eq!(counts(2, 3), [1, 1]);
}
