//! Tests of the computer's spell gains and skill values of a hero (E31 T2e): its caster record on
//! the game, a spell's gain, the hero's spell values per unit (CPU-STA-007), a school's value,
//! what the flag-set skill value reads and a University's value. On the crate's test core with
//! AI values, the original's 28 skill places (Air's masteries at its 15th), the gust a spell of
//! one AI kind worth 5 at every mastery and the market; the numbers come from an IEEE double model
//! of the reading (`E31-log.md` T2e).

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureRuleset, HeroOnMap, MarketParamsDef, StartingResourcesDef};
use h3_content::{
    ContentPack, CreatureTypeId, HeroEffect, Ruleset, SecondarySkillDef, SkillId, SpellId,
};
use h3_content_battle::{AbilityDef, SpellAi, SpellAiKind};
use h3_contracts::{ArmyStack, PlayerId};
use h3_heroes::{ArtifactInstance, HeroDetails, computer_skill_value};
use h3_kernel::arith::SignedX87;
use h3_towns::TownRuleset;

use super::super::skills::{self, school_value, university_value};
use super::super::start;
use super::*;
use crate::tests::{adventure_pack, battle_part_with, core_pack, key, setup};

const RED: PlayerId = PlayerId::new(0);

/// The Air school's skill, at the original's place of Air Magic.
const AIR: SkillId = SkillId::from_raw(15);

/// The rules: the crate's test core with AI values (the imp 10, the gog 40, the cart 50), its four
/// skills followed by `t:s4` to `t:s17` (`t:s15` with Air's masteries Basic to Expert), the
/// market (gold its cost resource, worth 1; the resources' rate 0.1 at one Marketplace), 1 999 gold
/// at the start on difficulty 2, the gust
/// of the AI kind `kind` worth 5 at every mastery; with `machine` the cart a shooter with First Aid.
fn rules(kind: SpellAiKind, machine: bool) -> GameRuleset {
    let mut pack: ContentPack = core_pack(false);
    for (creature, ai) in pack.creatures.iter_mut().zip([10, 40, 50]) {
        creature.info.ai_value = ai;
    }
    if machine {
        pack.creatures[2].info.flags.push("shooter".into());
    }
    for i in 4..18 {
        let levels = if i == 15 {
            [1, 2, 3].map(|level| {
                vec![HeroEffect::SchoolMastery {
                    school: key("t:air"),
                    level,
                }]
            })
        } else {
            [vec![], vec![], vec![]]
        };
        pack.skills.push(SecondarySkillDef {
            key: key(&alloc::format!("t:s{i}")),
            name: "Skill".into(),
            levels,
        });
    }
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 2);
    adventure.params.as_mut().unwrap().market = Some(MarketParamsDef {
        cost: key("t:gold"),
        values: vec![(key("t:gold"), 1)],
        artifact_values: vec![(key("t:gold"), 1)],
        resources: vec![10; 10],
        artifacts: vec![10; 10],
        creatures: vec![10; 10],
    });
    adventure.starting_resources.push(StartingResourcesDef {
        human: vec![(key("t:gold"), 1999)],
        computer: vec![],
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        pack.spells[0].ai = SpellAi {
            kinds: SpellAiKinds::default().with(kind),
            values: [5; 4],
        };
        if machine {
            pack.creatures[2].abilities.push(AbilityDef::FirstAid {
                min_hp: 1,
                max_hp: 25,
            });
        }
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(TownRuleset::empty(&core)),
    )
    .unwrap()
}

/// The crate's test game on `rules`: on difficulty 1 (2 000 gold) or, `poor`, 2 (1 999).
fn game(rules: &GameRuleset, poor: bool) -> GameState {
    let mut setup = setup();
    if poor {
        setup.difficulty = 2;
    }
    GameState::new(rules, &setup, 3).unwrap()
}

/// Red's hero with 10 gogs and these details: the spell book (with `book`), the gust learned,
/// Spell Power 3, Knowledge 2 (20 spell points at most), 8 left, `skills` at their levels.
fn caster(
    state: &GameState,
    rules: &GameRuleset,
    book: bool,
    skills: &[(SkillId, u8)],
) -> (HeroOnMap, HeroDetails) {
    let core = rules.core();
    let mut on_map = state
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == RED)
        .unwrap()
        .clone();
    on_map.army = vec![ArmyStack {
        slot: 0,
        creature: CreatureTypeId::from_raw(1),
        count: 10,
    }];
    let mut details = state.hero(on_map.id).unwrap().clone();
    if book {
        details.worn[0] = Some(core.artifact_by_key("t:book").unwrap().into());
    }
    details.spells = vec![gust(rules)];
    details.primary = [1, 1, 3, 2];
    details.mana = 8;
    details.skills = skills.to_vec();
    (on_map, details)
}

fn gust(rules: &GameRuleset) -> SpellId {
    rules.core().spell_by_key("t:gust").unwrap()
}

// covers: CPU-SPL-001, CPU-STA-007
#[test]
fn a_heros_spell_values_per_unit_are_its_best_spell_with_one_more_of_each() {
    let rules = rules(SpellAiKind::Damage, false);
    let state = game(&rules, false);
    let (on_map, details) = caster(&state, &rules, true, &[(AIR, 1)]);
    let hero = Fighter {
        on_map: &on_map,
        details: &details,
    };
    let r = record(&state, &rules, hero);
    assert_eq!((r.strength, r.power, r.rounds, r.mana), (400, 3, 3, 20));
    // The gust at Basic costs 4: with 20 points, b = 5 × 4, X = 0.5, row 3, Y = 1.218, five casts
    // Z = 1.495: 400 × Y = 487. One more power: b = 25, X = 0.625, 400 × 1.3995 = 559: 72; one
    // more round: nothing (10 at least for the power); 50 points: still 487: 0 / 3, 10 at least.
    // With its 8 points two casts, Z = 0.765: 306; refilled 487, doubled (ten casts) 487.
    assert_eq!(
        gains(&rules, &r, 8),
        SpellGains {
            power: 72,
            rounds: 0,
            knowledge: 10,
            doubled: 181,
            refilled: 181,
        }
    );
    // With its maximum the refill is nothing; with twice it the doubling too.
    assert_eq!(
        (
            gains(&rules, &r, 20).refilled,
            gains(&rules, &r, 20).doubled
        ),
        (0, 0)
    );
    assert_eq!(gains(&rules, &r, 40).doubled, 0);
}

// covers: CPU-SPL-007
#[test]
fn a_spells_gain_is_its_value_less_the_best_of_its_damage_kinds() {
    let damage = rules(SpellAiKind::Damage, false);
    let state = game(&damage, false);
    let gain_of = |rules: &GameRuleset, book: bool, learned: bool| {
        let (on_map, mut details) = caster(&state, rules, book, &[(AIR, 1)]);
        if !learned {
            details.spells.clear();
        }
        let hero = Fighter {
            on_map: &on_map,
            details: &details,
        };
        gain(rules, &record(&state, rules, hero), gust(rules))
    };
    // Its own best (487) is the gust's value: 1; without it in the book its whole value.
    assert_eq!(gain_of(&damage, true, true), 1);
    assert_eq!(gain_of(&damage, true, false), 487);
    // No spell book: no power, no gain.
    assert_eq!(gain_of(&damage, false, true), 0);
    // An other spell's gain is its value: every stack (the gust aims at none) for 7 rounds, 5 ×
    // 400 × 7 / 700.
    let effect = rules(SpellAiKind::Effect, false);
    assert_eq!(gain_of(&effect, true, true), 20);
}

// covers: CPU-SKL-002
#[test]
fn a_school_is_worth_its_next_level_and_a_new_one_counts_at_expert() {
    let rules = rules(SpellAiKind::Damage, false);
    let state = game(&rules, false);
    let value = |skills: &[(SkillId, u8)], school: SkillId| {
        let (on_map, details) = caster(&state, &rules, true, skills);
        let hero = Fighter {
            on_map: &on_map,
            details: &details,
        };
        school_value(&state, &rules, hero, school)
    };
    // Basic to Advanced: the cost stays 4, b = 5 × 5: 559 - 487.
    assert_eq!(value(&[(AIR, 1)], AIR), 72);
    // None (cost 5, four casts, b = 15: 400 × 0.97125 = 388) to Expert (b = 30: 632).
    assert_eq!(value(&[], AIR), 244);
    // A school none of the spells is of.
    assert_eq!(value(&[(AIR, 1)], SkillId::from_raw(14)), 0);
}

// covers: CPU-SKL-001, CPU-SKL-003
#[test]
fn the_flag_set_path_reads_the_computers_values_and_a_university_sums_what_it_would_take() {
    let rules = rules(SpellAiKind::Damage, false);
    let state = game(&rules, false);
    let start = start::record(&state, &rules, RED);
    let (on_map, details) = caster(&state, &rules, true, &[(AIR, 1)]);
    let hero = Fighter {
        on_map: &on_map,
        details: &details,
    };
    let gains = gains(&rules, &record(&state, &rules, hero), 8);
    let reads = skills::reads(&state, &rules, hero, &gains, &start);
    assert_eq!(reads.worth, (1400, 500));
    assert_eq!((reads.power_gain, reads.knowledge_gain), (72, 10));
    // The gold weight: no demand, the rate f32(0.1) times gold's value 1.
    assert_eq!(reads.gold_weight, SignedX87::from_f32_bits(0x3DCC_CCCD));
    // Only Air's value is reached and not 0.
    assert_eq!(reads.schools, [0, 72, 0, 0]);
    assert!(!reads.ballista && !reads.tent);
    assert_eq!(reads.shares, state.adventure().map().terrain_shares());
    // Estates (the 14th place): trunc(725 × 0.1000000015) = 72; Air its school value.
    let core = rules.core();
    let value = |skill| computer_skill_value(&details, core, &reads, SkillId::from_raw(skill));
    assert_eq!((value(13), value(15)), (72, 72));
    // A University of Dark (Logistics' place: 1 400 / 10), Lore (Archery's: 16 × 500 / 100), the
    // known Air and `t:s5` (no class weight): the first two, which it would take.
    let offer = [2, 1, 15, 5].map(SkillId::from_raw);
    assert_eq!(
        university_value(&state, &rules, hero, &reads, (offer, true)),
        220
    );
    // 1 999 gold fail the University's gold test; a hero of eight skills takes none.
    let poor = game(&rules, true);
    assert_eq!(
        university_value(&poor, &rules, hero, &reads, (offer, true)),
        0
    );
    assert_eq!(
        university_value(&poor, &rules, hero, &reads, (offer, false)),
        220
    );
    let eight: Vec<(SkillId, u8)> = (4..12).map(|i| (SkillId::from_raw(i), 1)).collect();
    let (on_map, details) = caster(&state, &rules, true, &eight);
    let full = Fighter {
        on_map: &on_map,
        details: &details,
    };
    assert_eq!(
        university_value(&state, &rules, full, &reads, (offer, false)),
        0
    );
    // With eight skills no school is reached; on a ruleset whose cart shoots and heals, a hero
    // wearing it has a Ballista and a Tent.
    assert_eq!(
        skills::reads(&state, &rules, full, &gains, &start).schools,
        [0; 4]
    );
    let machine = rules_machine();
    let state = game(&machine, false);
    let (on_map, mut details) = caster(&state, &machine, true, &[(AIR, 3)]);
    let cart = machine.core().artifact_by_key("t:cart_art").unwrap();
    details.worn[1] = Some(ArtifactInstance::from(cart));
    let armed = Fighter {
        on_map: &on_map,
        details: &details,
    };
    let reads = skills::reads(
        &state,
        &machine,
        armed,
        &gains,
        &start::record(&state, &machine, RED),
    );
    assert!(reads.ballista && reads.tent);
    // Air at Expert is not reached.
    assert_eq!(reads.schools, [0; 4]);
}

/// [`rules`] with the damage kind and the shooting, healing cart.
fn rules_machine() -> GameRuleset {
    rules(SpellAiKind::Damage, true)
}
