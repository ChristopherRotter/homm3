//! Tests of the draws when the game is built (E08 T61b; increment 7's design note §9): the random
//! artifacts' pool and its marks, the chests, the gifts' prices, the Shrines' marks, the Witch Huts
//! (`ADV-QUIRK-21`), the Scholars, the Universities, the finds and the Pyramid, the bans, and the
//! static map's hash. Since E08 T124b1 the creature banks' (ADV-BNK-002; increment 13's design
//! note §5): the level, the split, the upgrade and the artifacts; since T133a1 the Black Markets'
//! stocks (ADV-MKT-006; increment 14's design note §9).

use std::collections::BTreeSet;
use std::format;
use std::string::{String, ToString};

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactTypeId, ContentKey, ContentPack,
    SpellCategory, SpellDef, SpellId, SpellSchoolDef,
};
use h3_contracts::ArmyStack;
use h3_heroes::{PrimarySkill, ScholarGift};
use h3_kernel::hash::hash_of;

use super::*;
use crate::ruleset::{
    AdventurePack, BankDef, BankLevelDef, BankQuestion, ChestDef, FindContentDef, FindDef, GiftDef,
    Gives, Market, MarketParamsDef, MarketScreen, ObjectKindDef, ObjectTypeDef, OnceBonus,
    PyramidDef, TurnDef, Visit,
};
use crate::setup::{Bans, Holds};
use crate::state::Resolved;
use crate::testing::{
    FLYING, GNOLL, LOGISTICS, PATHFINDING, PEASANT, PIKEMAN, SCOUTING, WATER_WALKING, WOLF,
    hero_at, hero_skills, key, movement_rules_with, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RING: ArtifactTypeId = ArtifactTypeId::from_raw(0);
const CHARM: ArtifactTypeId = ArtifactTypeId::from_raw(1);
const AMULET: ArtifactTypeId = ArtifactTypeId::from_raw(2);
const CLOAK: ArtifactTypeId = ArtifactTypeId::from_raw(3);
const HELM: ArtifactTypeId = ArtifactTypeId::from_raw(4);
const CROWN: ArtifactTypeId = ArtifactTypeId::from_raw(5);
const BOOK: ArtifactTypeId = ArtifactTypeId::from_raw(6);

const GUST: SpellId = SpellId::from_raw(0);
const BREEZE: SpellId = SpellId::from_raw(1);
const CALM: SpellId = SpellId::from_raw(2);
const STORM: SpellId = SpellId::from_raw(4);
const DOOM: SpellId = SpellId::from_raw(6);

const WOOD: ResourceId = ResourceId::from_raw(0);
const GOLD: ResourceId = ResourceId::from_raw(1);

/// [`crate::testing::movement_rules`] (`t:faithful` with `ADV-QUIRK-21` on, or `t:strict`) with
/// the artifacts ring, charm, amulet (treasure), cloak (minor), helm (major), crown (relic) and
/// book (special); the level-1 spells gust, breeze, calm and spit (an ability), the level-2
/// storm, the level-5 bolt (granted by name only) and doom;
/// [`hero_skills`]; and one object type of each kind that draws.
fn rules(faithful: bool) -> Rules {
    rules_with(faithful, |_| {}, |_| {})
}

/// [`rules`] with the core changed by `edit_core` and `more` object types after its own.
pub(super) fn rules_with(
    faithful: bool,
    edit_core: impl FnOnce(&mut ContentPack),
    more: impl FnOnce(&mut Vec<ObjectTypeDef>),
) -> Rules {
    movement_rules_with(
        faithful,
        16,
        |core: &mut ContentPack| {
            core.profiles[0]
                .quirks
                .push(("quirk.adventure.roe_witch_hut_mask".to_string(), true));
            core.artifact_slots = vec![ArtifactSlotDef {
                key: key("t:finger"),
                name: "Finger".into(),
                count: 1,
                backpack: true,
            }];
            let artifact = |k: &str, class| ArtifactDef {
                key: key(k),
                name: "Artifact".into(),
                class,
                cost: vec![],
                slot: Some(key("t:finger")),
                war_machine: None,
                needs_skill: None,
                effects: vec![],
                parts: vec![],
            };
            core.artifacts = vec![
                artifact("t:ring", ArtifactClass::Treasure),
                artifact("t:charm", ArtifactClass::Treasure),
                artifact("t:amulet", ArtifactClass::Treasure),
                artifact("t:cloak", ArtifactClass::Minor),
                artifact("t:helm", ArtifactClass::Major),
                artifact("t:crown", ArtifactClass::Relic),
                artifact("t:book", ArtifactClass::Special),
            ];
            core.spell_schools = vec![SpellSchoolDef {
                key: key("t:air"),
                name: "Air".into(),
                protection_rank: 0,
            }];
            let spell = |k: &str, category, level, school: bool, by_name| SpellDef {
                key: key(k),
                name: "Spell".into(),
                category,
                level,
                schools: if school { vec![key("t:air")] } else { vec![] },
                cost: [5, 4, 4, 4],
                power: 10,
                effect: [10, 10, 20, 30],
                granted_by_name_only: by_name,
            };
            core.spells = vec![
                spell("t:gust", SpellCategory::Combat, 1, true, false),
                spell("t:breeze", SpellCategory::Combat, 1, true, false),
                spell("t:calm", SpellCategory::Adventure, 1, true, false),
                spell("t:spit", SpellCategory::Ability, 1, false, false),
                spell("t:storm", SpellCategory::Combat, 2, true, false),
                spell("t:bolt", SpellCategory::Combat, 5, true, true),
                spell("t:doom", SpellCategory::Combat, 5, true, false),
            ];
            core.skills = hero_skills();
            edit_core(core);
        },
        |part: &mut AdventurePack| {
            // E08 T115b: the Shrine, the Witch Hut, the University and the Tree mark their kind
            // (the base pack's, ADV-BON-009).
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind_mark: ["t:shrine", "t:hut", "t:university", "t:tree"].contains(&k),
                kind,
            };
            let random = |classes: &[ArtifactClass]| ObjectKindDef::Artifact {
                classes: Some(classes.to_vec()),
            };
            let gift = |prices: Vec<Vec<(ContentKey, u32)>>| {
                ObjectKindDef::Gift(GiftDef {
                    gives: Gives::Level,
                    prices,
                    admits: None,
                    leave: false,
                })
            };
            let find = |draws, full_backpack| {
                ObjectKindDef::Find(FindDef {
                    draws,
                    full_backpack,
                    question: false,
                    bonus: None,
                    leaves: false,
                })
            };
            part.object_types.extend([
                ty("t:artifact", ObjectKindDef::Artifact { classes: None }),
                ty("t:random_treasure", random(&[ArtifactClass::Treasure])),
                ty(
                    "t:random_minor_major",
                    random(&[ArtifactClass::Minor, ArtifactClass::Major]),
                ),
                ty(
                    "t:chest",
                    ObjectKindDef::Chest(ChestDef {
                        resource: key("t:gold"),
                        gold: vec![(30, 1000), (30, 1500)],
                        artifact: vec![ArtifactClass::Treasure],
                        full_backpack: 1000,
                        experience_offset: 500,
                    }),
                ),
                ty(
                    "t:tree",
                    gift(vec![
                        vec![],
                        vec![(key("t:gold"), 2000)],
                        vec![(key("t:wood"), 10)],
                    ]),
                ),
                ty("t:stone", gift(vec![vec![(key("t:gold"), 10)]])),
                ty("t:shrine", ObjectKindDef::Shrine { level: 1 }),
                ty("t:shrine_3", ObjectKindDef::Shrine { level: 3 }),
                ty(
                    "t:hut",
                    ObjectKindDef::WitchHut {
                        roe: vec![key("t:logistics"), key("t:pathfinding")],
                        roe_fixed: vec![key("t:scouting")],
                    },
                ),
                ty("t:scholar", ObjectKindDef::Scholar),
                ty(
                    "t:university",
                    ObjectKindDef::University {
                        offers: 4,
                        price: vec![(key("t:gold"), 2000)],
                    },
                ),
                ty(
                    "t:wagon",
                    find(
                        vec![
                            (10, FindContentDef::Nothing),
                            (
                                40,
                                FindContentDef::Artifact(vec![
                                    ArtifactClass::Treasure,
                                    ArtifactClass::Minor,
                                ]),
                            ),
                            (
                                50,
                                FindContentDef::Resources {
                                    amount: [2, 5],
                                    resources: vec![key("t:wood")],
                                },
                            ),
                        ],
                        Some(TurnDef::Drawn),
                    ),
                ),
                ty(
                    "t:corpse",
                    find(
                        vec![
                            (20, FindContentDef::Artifact(vec![ArtifactClass::Treasure])),
                            (80, FindContentDef::Nothing),
                        ],
                        Some(TurnDef::Give(vec![(key("t:gold"), 1000)])),
                    ),
                ),
                // E08 T72: the Sea Chest's table (a find that leaves).
                ty(
                    "t:sea_chest",
                    ObjectKindDef::Find(FindDef {
                        draws: vec![
                            (20, FindContentDef::Nothing),
                            (
                                70,
                                FindContentDef::Fixed {
                                    resources: vec![(key("t:gold"), 1500)],
                                    artifact: None,
                                },
                            ),
                            (
                                10,
                                FindContentDef::Fixed {
                                    resources: vec![(key("t:gold"), 1000)],
                                    artifact: Some(vec![ArtifactClass::Treasure]),
                                },
                            ),
                        ],
                        full_backpack: Some(TurnDef::Give(vec![(key("t:gold"), 1500)])),
                        question: false,
                        bonus: None,
                        leaves: true,
                    }),
                ),
                ty(
                    "t:pyramid",
                    ObjectKindDef::Pyramid(PyramidDef {
                        split: (key("t:pikeman"), 40),
                        fixed: (key("t:peasant"), vec![10, 10]),
                        level: 5,
                        looted: OnceBonus {
                            morale: 0,
                            luck: -2,
                            mark: 1,
                        },
                    }),
                ),
            ]);
            more(&mut part.object_types);
            // E08 T133a1: a market needs the market's parameters (ADV-MKT-002).
            let market = |t: &ObjectTypeDef| matches!(t.kind, ObjectKindDef::Market(_));
            if part.object_types.iter().any(market) {
                part.params.as_mut().unwrap().market = Some(MarketParamsDef {
                    values: vec![(key("t:wood"), 250), (key("t:gold"), 1)],
                    artifact_values: vec![(key("t:wood"), 125), (key("t:gold"), 1)],
                    resources: vec![10, 15, 20, 25, 30],
                    artifacts: vec![20, 30, 40, 50, 60],
                    creatures: vec![30, 45, 60, 75, 90],
                });
            }
        },
    )
}

/// An object of type `k` at `(x, 0)`, blocking and visited there, holding `holds`.
pub(super) fn object(rules: &Rules, x: u16, k: &str, holds: Holds) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(x, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(rules.1.object_type_by_key(k).unwrap()),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds,
        owner: None,
    }
}

/// `n` objects of type `k` holding `holds` in a row from column `from`.
fn row(rules: &Rules, from: u16, n: u16, k: &str, holds: &Holds) -> Vec<ObjectSetup> {
    (from..from + n)
        .map(|x| object(rules, x, k, holds.clone()))
        .collect()
}

/// The state of a grass row of 16 tiles with these objects under `bans`, red's hero 1 below at
/// (1, 1), built from the setup stream of `seed`.
fn built(rules: &Rules, objects: Vec<ObjectSetup>, bans: &Bans, seed: u64) -> AdventureState {
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT])],
    );
    setup.map.objects.extend(objects);
    setup.bans = bans.clone();
    let mut draws = Rng::for_stream(seed, StreamId::SETUP);
    AdventureState::new(&rules.0, &rules.1, &setup, &mut draws).unwrap()
}

/// What the build drew for each object, in object order.
fn resolved(state: &AdventureState) -> Vec<Resolved> {
    state
        .map()
        .objects()
        .iter()
        .map(|o| o.resolved.clone())
        .collect()
}

/// The distinct draws among `items`, by their names (a set to compare).
fn names(items: impl IntoIterator<Item = Resolved>) -> BTreeSet<String> {
    items.into_iter().map(|r| format!("{r:?}")).collect()
}

/// The artifact a draw gave, if one.
fn artifact(resolved: &Resolved) -> Option<ArtifactTypeId> {
    match resolved {
        Resolved::Artifact(artifact) => Some(*artifact),
        _ => None,
    }
}

/// For each of the seeds 0 to 199, what the build drew for one object of type `k` holding
/// `holds` under `bans`, with its contents.
fn outcomes(
    rules: &Rules,
    k: &str,
    holds: &Holds,
    bans: &Bans,
) -> Vec<(Resolved, Vec<(ResourceId, u32)>)> {
    (0..200)
        .map(|seed| {
            let state = built(rules, vec![object(rules, 0, k, holds.clone())], bans, seed);
            let object = &state.map().objects()[0];
            (object.resolved.clone(), object.contents.clone())
        })
        .collect()
}

// covers: ADV-OBJ-006, GAM-SET-003
#[test]
fn random_artifacts_draw_from_their_classes_without_repeats_until_the_pool_is_out() {
    let rules = rules(true);
    let none = Bans::default();
    let placed = object(&rules, 0, "t:artifact", Holds::Artifact(RING.into()));
    let mut widened = BTreeSet::new();
    for seed in 0..20 {
        // The placed ring marks itself: two random treasures give the charm and the amulet,
        // one each; a third finds the pool out, restarts it from the bans and may give any.
        let mut objects = vec![placed.clone()];
        objects.extend(row(&rules, 1, 3, "t:random_treasure", &Holds::Nothing));
        let drawn: Vec<_> = resolved(&built(&rules, objects, &none, seed))
            .iter()
            .map(artifact)
            .collect();
        assert_eq!(drawn[0], None, "a placed artifact draws nothing");
        let first: BTreeSet<_> = drawn[1..3].iter().flatten().copied().collect();
        assert_eq!(first, BTreeSet::from([CHARM, AMULET]));
        assert!(drawn[3].is_some_and(|a| [RING, CHARM, AMULET].contains(&a)));
        // A banned amulet never comes, not even after the restart.
        let bans = Bans {
            artifacts: vec![AMULET],
            ..Bans::default()
        };
        let drawn = resolved(&built(
            &rules,
            row(&rules, 0, 4, "t:random_treasure", &Holds::Nothing),
            &bans,
            seed,
        ));
        assert!(
            drawn
                .iter()
                .all(|r| artifact(r).is_some_and(|a| [RING, CHARM].contains(&a)))
        );
        // Every treasure banned: the draw widens to every class but Special, each time after
        // restarting the marks from the bans (so the widened draws may repeat).
        let bans = Bans {
            artifacts: vec![RING, CHARM, AMULET],
            ..Bans::default()
        };
        let drawn = resolved(&built(
            &rules,
            row(&rules, 0, 4, "t:random_treasure", &Holds::Nothing),
            &bans,
            seed,
        ));
        let wide: Vec<_> = drawn.iter().filter_map(artifact).collect();
        assert_eq!(wide.len(), 4);
        assert!(wide.iter().all(|a| [CLOAK, HELM, CROWN].contains(a)));
        widened.extend(wide);
        // Two classes as one pool.
        let drawn = resolved(&built(
            &rules,
            row(&rules, 0, 2, "t:random_minor_major", &Holds::Nothing),
            &none,
            seed,
        ));
        let both: BTreeSet<_> = drawn.iter().filter_map(artifact).collect();
        assert_eq!(both, BTreeSet::from([CLOAK, HELM]));
    }
    assert_eq!(widened, BTreeSet::from([CLOAK, HELM, CROWN]));
    assert!(!widened.contains(&BOOK), "never the special book");
    // Every artifact but the special one banned: nothing to draw.
    let bans = Bans {
        artifacts: vec![RING, CHARM, AMULET, CLOAK, HELM, CROWN],
        ..Bans::default()
    };
    let state = built(
        &rules,
        vec![object(&rules, 0, "t:random_treasure", Holds::Nothing)],
        &bans,
        0,
    );
    assert_eq!(resolved(&state), [Resolved::Nothing]);
}

// covers: ADV-OBJ-006, ADV-PIK-005
#[test]
fn the_random_artifacts_draw_before_the_chests() {
    // A chest before a random treasure in object order, the charm the only treasure left: the
    // random artifact takes it whatever the chest drew, and a chest's artifact comes after it.
    let rules = rules(true);
    let bans = Bans {
        artifacts: vec![AMULET],
        ..Bans::default()
    };
    let objects = vec![
        object(&rules, 0, "t:chest", Holds::Nothing),
        object(&rules, 1, "t:artifact", Holds::Artifact(RING.into())),
        object(&rules, 2, "t:random_treasure", Holds::Nothing),
    ];
    let mut chest_artifacts = 0;
    for seed in 0..100 {
        let drawn = resolved(&built(&rules, objects.clone(), &bans, seed));
        assert_eq!(drawn[2], Resolved::Artifact(CHARM));
        if let Some(artifact) = artifact(&drawn[0]) {
            // The pool was out (the ring placed, the charm drawn): it restarted from the bans.
            assert!([RING, CHARM].contains(&artifact));
            chest_artifacts += 1;
        }
    }
    assert!(chest_artifacts > 0);
}

// covers: ADV-PIK-005
#[test]
fn a_chest_holds_gold_by_its_chances_or_an_artifact() {
    let rules = rules(true);
    let bans = Bans {
        artifacts: vec![RING],
        ..Bans::default()
    };
    let outcomes = outcomes(&rules, "t:chest", &Holds::Nothing, &bans);
    let mut seen = BTreeSet::new();
    for (resolved, contents) in &outcomes {
        match (resolved, contents.as_slice()) {
            (Resolved::Nothing, &[(resource, gold)]) if resource == GOLD => {
                seen.insert(gold);
            }
            (&Resolved::Artifact(artifact), []) => {
                assert!([CHARM, AMULET].contains(&artifact));
                seen.insert(0);
            }
            other => panic!("not a chest's contents: {other:?}"),
        }
    }
    assert_eq!(seen, BTreeSet::from([0, 1000, 1500]));
    // About 30, 30 and 40 in 100.
    let gold = outcomes.iter().filter(|(_, c)| !c.is_empty()).count();
    assert!((100..140).contains(&gold), "{gold} of 200 hold gold");
}

// covers: ADV-TCH-003
#[test]
fn a_gift_of_several_prices_draws_one() {
    let rules = rules(true);
    let none = Bans::default();
    let prices = names(
        outcomes(&rules, "t:tree", &Holds::Nothing, &none)
            .into_iter()
            .map(|(resolved, _)| resolved),
    );
    assert_eq!(
        prices,
        names([Resolved::Price(0), Resolved::Price(1), Resolved::Price(2)])
    );
    // One price: no draw.
    let state = built(
        &rules,
        vec![object(&rules, 0, "t:stone", Holds::Nothing)],
        &none,
        0,
    );
    assert_eq!(resolved(&state), [Resolved::Nothing]);
}

// covers: ADV-TCH-004, GAM-SET-003
#[test]
fn random_shrines_repeat_no_spell_until_their_level_is_out() {
    let rules = rules(true);
    // The level's spells: gust and calm (breeze banned; spit an ability). The map's gust marks
    // nothing.
    let bans = Bans {
        spells: vec![BREEZE],
        ..Bans::default()
    };
    for seed in 0..20 {
        let mut objects = vec![object(&rules, 0, "t:shrine", Holds::Spell(Some(GUST)))];
        objects.extend(row(&rules, 1, 4, "t:shrine", &Holds::Spell(None)));
        objects.push(object(&rules, 5, "t:shrine_3", Holds::Spell(None)));
        let drawn = resolved(&built(&rules, objects, &bans, seed));
        assert_eq!(drawn[0], Resolved::Spell(GUST));
        let spells = |r: &[Resolved]| names(r.iter().cloned());
        let both = names([Resolved::Spell(GUST), Resolved::Spell(CALM)]);
        assert_eq!(spells(&drawn[1..3]), both);
        assert_eq!(spells(&drawn[3..5]), both);
        // A level without a spell: none (A-18 fixed).
        assert_eq!(drawn[5], Resolved::Nothing);
    }
}

// covers: ADV-TCH-005, GAM-SET-003
#[test]
fn a_witch_hut_draws_from_its_mask_less_the_bans_and_an_roe_hut_by_the_profile() {
    let faithful = rules(true);
    let bans = Bans {
        skills: vec![FLYING],
        ..Bans::default()
    };
    let skills = |rules: &Rules, holds: Holds, bans: &Bans| {
        names(
            outcomes(rules, "t:hut", &holds, bans)
                .into_iter()
                .map(|(resolved, _)| resolved),
        )
    };
    let mask = Holds::Skills(Some(vec![SCOUTING, FLYING, LOGISTICS]));
    assert_eq!(
        skills(&faithful, mask.clone(), &bans),
        names([Resolved::Skill(LOGISTICS), Resolved::Skill(SCOUTING)])
    );
    let every = Bans {
        skills: vec![SCOUTING, FLYING, LOGISTICS],
        ..Bans::default()
    };
    assert_eq!(skills(&faithful, mask, &every), names([Resolved::Nothing]));
    // ADV-QUIRK-21: an RoE map's hut, without a mask, draws from the original's fixed mask in
    // `faithful` (here logistics and pathfinding), from every skill but Leadership and
    // Necromancy in `strict-fixed` (here scouting).
    assert_eq!(
        skills(&faithful, Holds::Skills(None), &bans),
        names([Resolved::Skill(LOGISTICS), Resolved::Skill(PATHFINDING)])
    );
    assert_eq!(
        skills(&rules(false), Holds::Skills(None), &bans),
        names([Resolved::Skill(SCOUTING)])
    );
}

// covers: ADV-TCH-006, GAM-SET-003
#[test]
fn a_scholar_holds_its_gift_and_the_primary_skill_it_gives_instead() {
    let rules = rules(true);
    let none = Bans::default();
    let drawn = |gift: Option<ScholarGift>, bans: &Bans| {
        outcomes(&rules, "t:scholar", &Holds::Gift(gift), bans)
            .into_iter()
            .map(|(resolved, _)| match resolved {
                Resolved::Scholar(gift, fallback) => (gift, fallback),
                other => panic!("not a Scholar's: {other:?}"),
            })
            .collect::<Vec<_>>()
    };
    // A primary skill is its own fallback; the map's skill or spell draws one of the four.
    let power = ScholarGift::Primary(PrimarySkill::Power);
    assert!(
        drawn(Some(power), &none)
            .iter()
            .all(|&d| d == (power, PrimarySkill::Power))
    );
    let skill = drawn(Some(ScholarGift::Skill(LOGISTICS)), &none);
    assert!(
        skill
            .iter()
            .all(|&(g, _)| g == ScholarGift::Skill(LOGISTICS))
    );
    let fallbacks: BTreeSet<_> = skill.iter().map(|&(_, f)| f as u8).collect();
    assert_eq!(fallbacks, BTreeSet::from([0, 1, 2, 3]));
    // A random one: each kind; its skill and spell unbanned, its spell any level's hero spell
    // that is not granted by name only.
    let bans = Bans {
        skills: vec![LOGISTICS, PATHFINDING],
        spells: vec![GUST],
        ..Bans::default()
    };
    let mut kinds = BTreeSet::new();
    for (gift, fallback) in drawn(None, &bans) {
        match gift {
            ScholarGift::Primary(skill) => {
                assert_eq!(skill, fallback);
                kinds.insert(0);
            }
            ScholarGift::Skill(skill) => {
                assert!([SCOUTING, FLYING, WATER_WALKING].contains(&skill));
                kinds.insert(1);
            }
            ScholarGift::Spell(spell) => {
                assert!([BREEZE, CALM, STORM, DOOM].contains(&spell), "{spell:?}");
                kinds.insert(2);
            }
        }
    }
    assert_eq!(kinds, BTreeSet::from([0, 1, 2]));
    // Every skill and spell banned: a primary skill whatever the kind drawn.
    let every = Bans {
        skills: vec![LOGISTICS, PATHFINDING, SCOUTING, FLYING, WATER_WALKING],
        spells: vec![GUST, BREEZE, CALM, STORM, DOOM],
        ..Bans::default()
    };
    assert!(
        drawn(None, &every)
            .iter()
            .all(|&(gift, fallback)| gift == ScholarGift::Primary(fallback))
    );
}

// covers: ADV-TCH-007, GAM-SET-003
#[test]
fn a_university_draws_distinct_unbanned_skills_and_all_when_fewer_are_left() {
    let rules = rules(true);
    let skills = |bans: &Bans| {
        outcomes(&rules, "t:university", &Holds::Nothing, bans)
            .into_iter()
            .map(|(resolved, _)| match resolved {
                Resolved::Skills(skills) => skills,
                other => panic!("not a University's: {other:?}"),
            })
            .collect::<Vec<_>>()
    };
    let one = Bans {
        skills: vec![FLYING],
        ..Bans::default()
    };
    let drawn = skills(&one);
    for offered in &drawn {
        let set: BTreeSet<_> = offered.iter().copied().collect();
        assert_eq!(
            set,
            BTreeSet::from([LOGISTICS, PATHFINDING, SCOUTING, WATER_WALKING])
        );
    }
    // The order is the draw's.
    let orders: BTreeSet<_> = drawn.into_iter().collect();
    assert!(orders.len() > 1);
    // Three left for four offers: all three (A-22 fixed).
    let two = Bans {
        skills: vec![FLYING, LOGISTICS],
        ..Bans::default()
    };
    assert!(skills(&two).iter().all(|offered| {
        offered.iter().copied().collect::<BTreeSet<_>>()
            == BTreeSet::from([PATHFINDING, SCOUTING, WATER_WALKING])
    }));
}

// covers: ADV-PIK-009, ADV-PIK-010, ADV-PIK-011, ADV-PIK-012
#[test]
fn a_find_holds_what_its_table_draws() {
    let rules = rules(true);
    let none = Bans::default();
    // The wagon: nothing (10 %), an artifact of the treasure and minor classes with its drawn
    // resources beside it (40 %), or 2 to 5 wood (50 %).
    let mut seen = [0; 3];
    for (resolved, contents) in outcomes(&rules, "t:wagon", &Holds::Nothing, &none) {
        let wood = |contents: &[(ResourceId, u32)]| matches!(contents, &[(resource, amount)] if resource == WOOD && (2..=5).contains(&amount));
        match resolved {
            Resolved::Nothing if contents.is_empty() => seen[0] += 1,
            Resolved::Artifact(artifact) => {
                assert!([RING, CHARM, AMULET, CLOAK].contains(&artifact));
                assert!(wood(&contents), "{contents:?}");
                seen[1] += 1;
            }
            Resolved::Nothing => {
                assert!(wood(&contents), "{contents:?}");
                seen[2] += 1;
            }
            other => panic!("not a wagon's: {other:?}"),
        }
    }
    assert!(seen.iter().all(|&n| n > 0), "{seen:?}");
    assert!((60..100).contains(&seen[1]), "{seen:?}");
    // The corpse: a treasure artifact (20 %) without resources, or nothing.
    let corpse = outcomes(&rules, "t:corpse", &Holds::Nothing, &none);
    assert!(corpse.iter().all(|(resolved, contents)| {
        contents.is_empty() && matches!(resolved, Resolved::Nothing | Resolved::Artifact(_))
    }));
    let held = corpse
        .iter()
        .filter(|(resolved, _)| *resolved != Resolved::Nothing)
        .count();
    assert!((20..70).contains(&held), "{held} of 200 hold an artifact");
}

// covers: ADV-PIK-015
#[test]
fn a_sea_find_holds_its_fixed_entry_with_the_artifact_beside_it() {
    // E08 T72 (increment 8's design note §11): the Sea Chest holds nothing (20 %), 1 500 gold (70
    // %), or 1 000 gold and a treasure artifact (10 %), one draw of 0 to 99 and the artifact from
    // the pool.
    let rules = rules(true);
    let mut seen = [0; 3];
    for (resolved, contents) in outcomes(&rules, "t:sea_chest", &Holds::Nothing, &Bans::default()) {
        let gold = |amount| contents == [(GOLD, amount)];
        match resolved {
            Resolved::Nothing if contents.is_empty() => seen[0] += 1,
            Resolved::Nothing if gold(1500) => seen[1] += 1,
            Resolved::Artifact(artifact) if gold(1000) => {
                assert!([RING, CHARM, AMULET].contains(&artifact), "{artifact:?}");
                seen[2] += 1;
            }
            other => panic!("not a sea chest's: {other:?} with {contents:?}"),
        }
    }
    assert!(seen.iter().all(|&n| n > 0), "{seen:?}");
    assert!((100..180).contains(&seen[1]), "{seen:?}");
}

// covers: ADV-TCH-008, GAM-SET-003
#[test]
fn the_pyramid_holds_an_unbanned_spell_of_its_level() {
    let rules = rules(true);
    // Level 5: doom; bolt is granted by name only.
    let drawn = outcomes(&rules, "t:pyramid", &Holds::Nothing, &Bans::default());
    assert!(drawn.iter().all(|(r, _)| *r == Resolved::Spell(DOOM)));
    // Doom banned: none (A-20 fixed).
    let bans = Bans {
        spells: vec![DOOM],
        ..Bans::default()
    };
    let state = built(
        &rules,
        vec![object(&rules, 0, "t:pyramid", Holds::Nothing)],
        &bans,
        0,
    );
    assert_eq!(resolved(&state), [Resolved::Nothing]);
}

#[test]
fn the_static_maps_hash_sees_what_the_build_drew() {
    let rules = rules(true);
    let none = Bans::default();
    let chest = vec![object(&rules, 0, "t:chest", Holds::Nothing)];
    let hashes: BTreeSet<u64> = (0..20)
        .map(|seed| {
            built(&rules, chest.clone(), &none, seed)
                .map()
                .content_hash()
        })
        .collect();
    assert!(hashes.len() > 1);
    // Each kind of draw hashes apart from the others.
    let all = [
        Resolved::Nothing,
        Resolved::Artifact(RING),
        Resolved::Price(0),
        Resolved::Spell(GUST),
        Resolved::Skill(LOGISTICS),
        Resolved::Scholar(
            ScholarGift::Primary(PrimarySkill::Attack),
            PrimarySkill::Attack,
        ),
        Resolved::Scholar(ScholarGift::Skill(LOGISTICS), PrimarySkill::Attack),
        Resolved::Scholar(ScholarGift::Spell(GUST), PrimarySkill::Attack),
        Resolved::Scholar(ScholarGift::Spell(GUST), PrimarySkill::Knowledge),
        Resolved::Skills(vec![LOGISTICS]),
        Resolved::Skills(vec![]),
        Resolved::Bank {
            level: 0,
            artifacts: vec![],
        },
        Resolved::Bank {
            level: 1,
            artifacts: vec![],
        },
        Resolved::Bank {
            level: 0,
            artifacts: vec![RING],
        },
        Resolved::Stock(vec![]),
        Resolved::Stock(vec![RING]),
        Resolved::Stock(vec![RING, CHARM]),
        Resolved::Stock(vec![CHARM, RING]),
    ];
    let hashes: BTreeSet<u64> = all.iter().map(hash_of).collect();
    assert_eq!(hashes.len(), all.len());
}

/// A creature bank's level of `chance` guarded by `guards` (core keys and counts), upgraded at
/// `upgrade` %, giving `artifacts` by class (treasures, minors, majors, relics) and nothing else.
fn bank_level(chance: u8, guards: &[(&str, u32)], upgrade: u8, artifacts: [u8; 4]) -> BankLevelDef {
    BankLevelDef {
        chance,
        guards: guards.iter().map(|&(k, count)| (key(k), count)).collect(),
        upgrade,
        resources: vec![],
        creature: None,
        artifacts,
    }
}

/// [`rules`] with peasants upgrading to pikemen and an entered attack bank of each key and its
/// levels.
fn bank_rules(banks: Vec<(&str, Vec<BankLevelDef>)>) -> Rules {
    rules_with(
        true,
        |core| core.creatures[2].info.upgrades = vec![key("t:pikeman")],
        |types| {
            types.extend(banks.into_iter().map(|(k, levels)| ObjectTypeDef {
                key: key(k),
                name: "Bank".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: false,
                kind: ObjectKindDef::Bank(BankDef {
                    question: BankQuestion::Attack,
                    levels,
                    looted: None,
                    deck: false,
                    coast: false,
                }),
            }));
        },
    )
}

/// The bank at object index `index`: its guards as drawn (they ask), its level and artifacts.
fn bank(state: &AdventureState, index: usize) -> (Vec<ArmyStack>, u8, Vec<ArtifactTypeId>) {
    let object = &state.map().objects()[index];
    let Resolved::Bank { level, artifacts } = &object.resolved else {
        panic!("{:?}", object.resolved);
    };
    let guards = object.guards.as_ref().unwrap();
    assert!(guards.question);
    (guards.army.clone(), *level, artifacts.clone())
}

/// The level index the original's subtraction gives r over these chances (an oracle restated:
/// the first level but the last whose running sum reaches r, else the last).
fn level_for(r: u32, chances: &[u8]) -> u8 {
    let mut sum = 0;
    for (index, &chance) in chances[..chances.len() - 1].iter().enumerate() {
        sum += u32::from(chance);
        if r <= sum {
            return u8::try_from(index).unwrap();
        }
    }
    u8::try_from(chances.len() - 1).unwrap()
}

/// `n` of the setup stream of `seed`'s draws in 1 to 100.
fn percents(seed: u64, n: usize) -> Vec<u32> {
    let mut stream = Rng::for_stream(seed, StreamId::SETUP);
    (0..n).map(|_| stream.range_inclusive(1, 100)).collect()
}

/// The original's four chances (survey `E08-banks.md` §3 step 1), each level guarded by 5, 10,
/// 15 or 20 peasants.
fn four_levels() -> Vec<BankLevelDef> {
    [30, 30, 30, 10]
        .into_iter()
        .zip(1..)
        .map(|(chance, n)| bank_level(chance, &[("t:peasant", 5 * n)], 0, [0; 4]))
        .collect()
}

// covers: ADV-BNK-002
#[test]
fn a_bank_draws_its_level_by_the_running_sum_of_its_chances() {
    let rules = bank_rules(vec![("t:bank", four_levels())]);
    let mut seen = [0; 4];
    for seed in 0..400 {
        let objects = vec![object(&rules, 0, "t:bank", Holds::Nothing)];
        let (army, level, artifacts) = bank(&built(&rules, objects, &Bans::default(), seed), 0);
        // The setup stream's first draw r: 1 to 30 the first level, 31 to 60 the second, 61 to
        // 90 the third, 91 to 100 the fourth.
        let r = percents(seed, 1)[0];
        assert_eq!(level, level_for(r, &[30, 30, 30, 10]), "seed {seed}: r {r}");
        // Its guards are that level's, and it gives no artifact.
        let total: u32 = army.iter().map(|stack| stack.count).sum();
        assert_eq!(total, 5 * (u32::from(level) + 1));
        assert!(artifacts.is_empty());
        seen[usize::from(level)] += 1;
    }
    assert!(seen.iter().all(|&n| n > 0), "{seen:?}");
    assert!(seen[3] < seen[0], "{seen:?}");
}

// covers: ADV-BNK-002
#[test]
fn a_banks_guards_split_by_their_number_into_the_originals_slots() {
    // The guards of a one-level bank of these, by slot.
    let split = |guards: &[(&str, u32)]| {
        let rules = bank_rules(vec![("t:bank", vec![bank_level(100, guards, 0, [0; 4])])]);
        let objects = vec![object(&rules, 0, "t:bank", Holds::Nothing)];
        let (army, _, _) = bank(&built(&rules, objects, &Bans::default(), 0), 0);
        army.iter()
            .map(|stack| (stack.slot, stack.creature, stack.count))
            .collect::<Vec<_>>()
    };
    let (p, w, g, k) = (PEASANT, WOLF, GNOLL, PIKEMAN);
    // One creature: slots 1 to 4 take h₁ = ⌊c / 5⌋, h₂ = ⌊(c − h₁) / 4⌋, h₃, h₄; slot 0 the rest.
    assert_eq!(
        split(&[("t:peasant", 50)]),
        [(0, p, 10), (1, p, 10), (2, p, 10), (3, p, 10), (4, p, 10)]
    );
    assert_eq!(
        split(&[("t:peasant", 13)]),
        [(0, p, 3), (1, p, 2), (2, p, 2), (3, p, 3), (4, p, 3)]
    );
    // A quotient of 0 leaves its slot empty.
    assert_eq!(
        split(&[("t:peasant", 3)]),
        [(0, p, 1), (3, p, 1), (4, p, 1)]
    );
    // Two (the Crypt's level 1): the second's half into slot 2, the first's third into slot 3 and
    // half of what is left into slot 4.
    assert_eq!(
        split(&[("t:peasant", 30), ("t:wolf", 20)]),
        [(0, p, 10), (1, w, 10), (2, w, 10), (3, p, 10), (4, p, 10)]
    );
    assert_eq!(
        split(&[("t:peasant", 31), ("t:wolf", 21)]),
        [(0, p, 11), (1, w, 11), (2, w, 10), (3, p, 10), (4, p, 10)]
    );
    // Three (the Crypt's level 2): the second's half into slot 3, the first's into slot 4.
    assert_eq!(
        split(&[("t:peasant", 25), ("t:wolf", 20), ("t:gnoll", 5)]),
        [(0, p, 13), (1, w, 10), (2, g, 5), (3, w, 10), (4, p, 12)]
    );
    // Four: no split.
    assert_eq!(
        split(&[
            ("t:peasant", 20),
            ("t:wolf", 20),
            ("t:gnoll", 10),
            ("t:pikeman", 5)
        ]),
        [(0, p, 20), (1, w, 20), (2, g, 10), (3, k, 5)]
    );
}

// covers: ADV-BNK-002
#[test]
fn a_banks_upgrade_slot_takes_its_upgrade_by_its_chance() {
    // The creatures of a one-level bank of these, upgraded at `upgrade` %, by slot.
    let at = |guards: &[(&str, u32)], upgrade: u8, seed: u64| {
        let rules = bank_rules(vec![(
            "t:bank",
            vec![bank_level(100, guards, upgrade, [0; 4])],
        )]);
        let objects = vec![object(&rules, 0, "t:bank", Holds::Nothing)];
        let (army, _, _) = bank(&built(&rules, objects, &Bans::default(), seed), 0);
        army
    };
    let (p, w, g, k) = (PEASANT, WOLF, GNOLL, PIKEMAN);
    let mut upgraded = 0;
    for seed in 0..200 {
        // One creature: slot 2's stack by the stream's second draw, at most 50 (the first the
        // level's), its count kept.
        let up = percents(seed, 2)[1] <= 50;
        for stack in at(&[("t:peasant", 50)], 50, seed) {
            let creature = if up && stack.slot == 2 { k } else { p };
            assert_eq!((stack.creature, stack.count), (creature, 10), "seed {seed}");
        }
        upgraded += usize::from(up);
    }
    assert!((60..140).contains(&upgraded), "{upgraded}");
    // At 100 %, the upgrade's slot by the number of creatures: 3 for two, 0 for three and four.
    let creatures = |army: Vec<ArmyStack>| {
        army.iter()
            .map(|stack| (stack.slot, stack.creature))
            .collect::<Vec<_>>()
    };
    let two = [("t:peasant", 30), ("t:wolf", 20)];
    assert_eq!(
        creatures(at(&two, 100, 0)),
        [(0, p), (1, w), (2, w), (3, k), (4, p)]
    );
    let three = [("t:peasant", 25), ("t:wolf", 20), ("t:gnoll", 5)];
    assert_eq!(
        creatures(at(&three, 100, 0)),
        [(0, k), (1, w), (2, g), (3, w), (4, p)]
    );
    let four = [
        ("t:peasant", 20),
        ("t:wolf", 20),
        ("t:gnoll", 10),
        ("t:pikeman", 5),
    ];
    assert_eq!(
        creatures(at(&four, 100, 0)),
        [(0, k), (1, w), (2, g), (3, k)]
    );
    // A creature without an upgrade in the slot stays: wolves first, the peasants elsewhere.
    let wolves = [("t:wolf", 30), ("t:peasant", 20)];
    assert_eq!(
        creatures(at(&wolves, 100, 0)),
        [(0, w), (1, p), (2, p), (3, w), (4, w)]
    );
    // Neither a slot without an upgrade nor a chance of 0 draws: a bank after two such banks
    // draws its level from the stream's third draw.
    let rules = bank_rules(vec![
        (
            "t:wolves",
            vec![bank_level(100, &[("t:wolf", 50)], 100, [0; 4])],
        ),
        (
            "t:never",
            vec![bank_level(100, &[("t:peasant", 50)], 0, [0; 4])],
        ),
        ("t:bank", four_levels()),
    ]);
    for seed in 0..50 {
        let objects = vec![
            object(&rules, 0, "t:wolves", Holds::Nothing),
            object(&rules, 1, "t:never", Holds::Nothing),
            object(&rules, 2, "t:bank", Holds::Nothing),
        ];
        let state = built(&rules, objects, &Bans::default(), seed);
        assert!(bank(&state, 0).0.iter().all(|stack| stack.creature == w));
        assert!(bank(&state, 1).0.iter().all(|stack| stack.creature == p));
        let r = percents(seed, 3)[2];
        assert_eq!(
            bank(&state, 2).1,
            level_for(r, &[30, 30, 30, 10]),
            "seed {seed}"
        );
    }
}

// covers: ADV-BNK-002, ADV-OBJ-006
#[test]
fn a_banks_artifacts_come_relics_first_from_the_random_artifacts_pool() {
    // A treasure, a minor, a major and a relic: the relic first, then the major, the minor and
    // the treasure.
    let level = bank_level(100, &[("t:peasant", 5)], 0, [1, 1, 1, 1]);
    let rules = bank_rules(vec![("t:bank", vec![level])]);
    for seed in 0..20 {
        // A placed ring and a random treasure mark theirs: the bank's treasure is the third.
        let objects = vec![
            object(&rules, 0, "t:artifact", Holds::Artifact(RING.into())),
            object(&rules, 1, "t:random_treasure", Holds::Nothing),
            object(&rules, 2, "t:bank", Holds::Nothing),
        ];
        let state = built(&rules, objects, &Bans::default(), seed);
        let random = artifact(&state.map().objects()[1].resolved).unwrap();
        let (_, _, artifacts) = bank(&state, 2);
        let third = if random == CHARM { AMULET } else { CHARM };
        assert_eq!(artifacts, [CROWN, HELM, CLOAK, third], "seed {seed}");
    }
    // Every artifact but the special one banned: the bank gives none.
    let bans = Bans {
        artifacts: vec![RING, CHARM, AMULET, CLOAK, HELM, CROWN],
        ..Bans::default()
    };
    let objects = vec![object(&rules, 0, "t:bank", Holds::Nothing)];
    assert_eq!(bank(&built(&rules, objects, &bans, 0), 0).2, []);
}

/// [`rules_with`] with `extra` more treasures, minors and majors after the book (`t:trinket0`,
/// `t:trinket1`, … in turn), a Black Market
/// (`t:black_market`: three treasures, three minors and a major, the base pack's stock) and a
/// Trading Post (`t:trading_post`, no stock), entered as the base pack's.
fn market_rules(extra: u16) -> Rules {
    let market = |k: &str, screens: Vec<MarketScreen>, stock: Vec<ArtifactClass>| ObjectTypeDef {
        key: key(k),
        name: "Market".into(),
        visit: Visit::Enter,
        from_above: false,
        kind_mark: false,
        kind: ObjectKindDef::Market(Market {
            screens,
            count: 5,
            stock,
        }),
    };
    use ArtifactClass::{Major, Minor, Treasure};
    rules_with(
        true,
        |core| {
            let mut trinket = core.artifacts[0].clone();
            for n in 0..extra * 3 {
                trinket.key = key(&format!("t:trinket{n}"));
                trinket.class = [Treasure, Minor, Major][usize::from(n % 3)];
                core.artifacts.push(trinket.clone());
            }
        },
        |types| {
            types.push(market(
                "t:black_market",
                vec![MarketScreen::Buy],
                vec![Treasure, Treasure, Treasure, Minor, Minor, Minor, Major],
            ));
            types.push(market(
                "t:trading_post",
                vec![MarketScreen::Trade, MarketScreen::Transfer],
                vec![],
            ));
        },
    )
}

/// The stock of the Black Market at object index `index`.
fn stock(state: &AdventureState, index: usize) -> Vec<ArtifactTypeId> {
    state.map().objects()[index].resolved.stock().to_vec()
}

// covers: ADV-MKT-006, ADV-OBJ-006
#[test]
fn a_black_market_draws_its_classes_in_order_from_the_random_artifacts_pool() {
    let rules = market_rules(0);
    let mut seen = BTreeSet::new();
    for seed in 0..40 {
        // A placed ring and a random treasure mark theirs: the market's first treasure is the
        // third; then the treasures are out and the marks restart from the bans, so its second
        // and third are two of the three. The one minor is out at each draw after the first, the
        // major is the one.
        let objects = vec![
            object(&rules, 0, "t:artifact", Holds::Artifact(RING.into())),
            object(&rules, 1, "t:random_treasure", Holds::Nothing),
            object(&rules, 2, "t:black_market", Holds::Nothing),
        ];
        let state = built(&rules, objects, &Bans::default(), seed);
        let random = artifact(&state.map().objects()[1].resolved).unwrap();
        let third = if random == CHARM { AMULET } else { CHARM };
        let stock = stock(&state, 2);
        assert_eq!(stock.len(), 7, "seed {seed}");
        assert_eq!(stock[0], third, "seed {seed}");
        assert_ne!(stock[1], stock[2], "seed {seed}");
        assert!([RING, CHARM, AMULET].contains(&stock[1]), "seed {seed}");
        assert!([RING, CHARM, AMULET].contains(&stock[2]), "seed {seed}");
        assert_eq!(stock[3..], [CLOAK, CLOAK, CLOAK, HELM], "seed {seed}");
        seen.insert(stock[1..3].to_vec());
    }
    // Every pair of the three in both orders: the draw is the setup stream's.
    assert_eq!(seen.len(), 6);
    // A Trading Post holds no stock and draws nothing: the next object's draw is the same.
    let post = |with: bool| {
        let mut objects = vec![object(&rules, 0, "t:random_treasure", Holds::Nothing)];
        if with {
            objects.insert(0, object(&rules, 1, "t:trading_post", Holds::Nothing));
        }
        let state = built(&rules, objects, &Bans::default(), 7);
        resolved(&state)
    };
    let (with, without) = (post(true), post(false));
    assert_eq!(with[0], Resolved::Nothing);
    assert!(with[0].stock().is_empty());
    assert_eq!(with[1], without[0]);
}

// covers: ADV-MKT-006, ADV-OBJ-006, GAM-SET-003
#[test]
fn black_markets_repeat_neither_each_other_nor_the_maps_random_artifacts_nor_a_ban() {
    // Nine treasures, seven minors and seven majors: a random treasure and two markets' seven
    // each are fifteen distinct ones, none run out (a class's last taken restarts the marks).
    let rules = market_rules(6);
    for seed in 0..40 {
        let objects = vec![
            object(&rules, 0, "t:black_market", Holds::Nothing),
            object(&rules, 1, "t:random_treasure", Holds::Nothing),
            object(&rules, 2, "t:black_market", Holds::Nothing),
        ];
        let state = built(&rules, objects, &Bans::default(), seed);
        let mut drawn = stock(&state, 0);
        drawn.extend(stock(&state, 2));
        drawn.extend(artifact(&state.map().objects()[1].resolved));
        let distinct: BTreeSet<_> = drawn.iter().collect();
        assert_eq!(distinct.len(), 15, "seed {seed}");
    }
    // The one minor banned: the minors' draws widen to any class but the special one, never
    // the ban.
    let rules = market_rules(0);
    let bans = Bans {
        artifacts: vec![CLOAK],
        ..Bans::default()
    };
    for seed in 0..40 {
        let objects = vec![object(&rules, 0, "t:black_market", Holds::Nothing)];
        let stock = stock(&built(&rules, objects, &bans, seed), 0);
        assert_eq!(stock.len(), 7, "seed {seed}");
        assert!(
            !stock.contains(&CLOAK) && !stock.contains(&BOOK),
            "seed {seed}"
        );
    }
    // Every artifact but the special one banned: every entry is left out.
    let bans = Bans {
        artifacts: vec![RING, CHARM, AMULET, CLOAK, HELM, CROWN],
        ..Bans::default()
    };
    let objects = vec![object(&rules, 0, "t:black_market", Holds::Nothing)];
    let state = built(&rules, objects, &bans, 0);
    assert_eq!(state.map().objects()[0].resolved, Resolved::Stock(vec![]));
}
