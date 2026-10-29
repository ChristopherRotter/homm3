//! Tests of the draws when the game is built (E08 T61b; design §9): the random artifacts' pool and
//! its marks, the chests, the gifts' prices, the Shrines' marks, the Witch Huts (`ADV-QUIRK-21`),
//! the Scholars, the Universities, the finds and the Pyramid, the bans, and the static map's hash.

use std::collections::BTreeSet;
use std::format;
use std::string::{String, ToString};

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactTypeId, ContentKey, ContentPack,
    SpellCategory, SpellDef, SpellId, SpellSchoolDef,
};
use h3_heroes::{PrimarySkill, ScholarGift};
use h3_kernel::hash::hash_of;

use super::*;
use crate::ruleset::{
    AdventurePack, ChestDef, FindContentDef, FindDef, GiftDef, Gives, ObjectKindDef, ObjectTypeDef,
    OnceBonus, PyramidDef, TurnDef, Visit,
};
use crate::setup::{Bans, Holds};
use crate::state::Resolved;
use crate::testing::{
    FLYING, LOGISTICS, PATHFINDING, PEASANT, SCOUTING, WATER_WALKING, hero_at, hero_skills, key,
    movement_rules_with, world,
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
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Neighbour,
                from_above: true,
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
    ];
    let hashes: BTreeSet<u64> = all.iter().map(hash_of).collect();
    assert_eq!(hashes.len(), all.len());
}
