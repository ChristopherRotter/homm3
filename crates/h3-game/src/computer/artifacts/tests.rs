//! Tests of the computer's artifact values (E31 T2f): the table against the executable's list,
//! each effect code on a hero whose reads are set by hand ([`Valued`]'s fields), the hero's own
//! morale and luck, the special artifacts and the combinations, the value to a hero with and
//! without a free position, to a player and the turn start's mean; the town codes on the town
//! fixtures. On the crate's test core with AI values (the imp 10 and living, the gog 40, the cart
//! 50 and a shooter), the original's 28 skill places (Necromancy's 9 000, 10 000 and 5 000 basis
//! points, First Aid's +100 %), the undead by their base keys, 141 artifacts (from 5 in the ring
//! slot), a Fire school by its base key and three of its spells. The numbers with doubles come
//! from an IEEE double model of the reading (`E31-log.md` T2f); the spell codes compare with the
//! caster record's own values (CPU-SPL-002, tested at T2e) with the power the code sets.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureRuleset, HeroOnMap, Pos};
use h3_content::{
    ArtifactClass, ArtifactDef, ContentPack, CreatureDef, CreatureTypeId, HeroEffectDef, Ruleset,
    SecondarySkillDef, SpellCategory, SpellDef, SpellSchoolDef,
};
use h3_content_battle::{BattleCreatureDef, BattleSpellDef, SpellAi, SpellAiKind, SpellAiKinds};
use h3_contracts::{ArmyStack, TownId};
use h3_heroes::HeroDetails;
use h3_kernel::StateHasher;
use h3_kernel::arith::DOUBLE_BITS;
use h3_towns::{
    BuildingDef, BuildingRoles, BuildingsSetup, Dwelling, SlotSet, TownCommand, TownHeroChange,
};

use super::super::start;
use super::*;
use crate::tests::{adventure_pack, battle_part_with, core_pack, key, setup};
use crate::{Actor, GameCommand, GameEvent, GameSetup, apply};

const RED: PlayerId = PlayerId::new(0);

/// The army's value the hero codes are valued with.
const A: i32 = 10_000;

/// An artifact of the test packs: `k` in `slot` (none: the backpack's), `effects` its effects.
fn artifact(k: &str, slot: Option<&str>, effects: Vec<HeroEffectDef>) -> ArtifactDef {
    ArtifactDef {
        key: key(k),
        name: "Artifact".into(),
        class: ArtifactClass::Treasure,
        cost: vec![],
        slot: slot.map(key),
        war_machine: None,
        needs_skill: None,
        effects,
        parts: vec![],
    }
}

/// The rules: the crate's test core as the module's doc says, with `artifacts` artifacts in all
/// (at least 5) and `edit` made to the pack.
fn rules_of(artifacts: usize, edit: impl FnOnce(&mut ContentPack)) -> GameRuleset {
    let mut pack: ContentPack = core_pack(false);
    for (creature, ai) in pack.creatures.iter_mut().zip([10, 40, 50]) {
        creature.info.ai_value = ai;
    }
    pack.creatures[0].info.flags.push("alive".into());
    pack.creatures[2].info.flags.push("shooter".into());
    // The creatures to the Lich's id (64), the undead worth 60, 90, 150 and 300.
    for i in pack.creatures.len()..65 {
        let mut creature: CreatureDef = pack.creatures[1].clone();
        // The battle part's fourth row is the elf's.
        let k = if i == 3 {
            "t:elf".into()
        } else {
            alloc::format!("t:c{i}")
        };
        creature.key = key(&k);
        creature.info.ai_value = 0;
        pack.creatures.push(creature);
    }
    for (raw, ai) in UNDEAD.into_iter().zip([60, 90, 150, 300]) {
        pack.creatures[usize::from(raw)].info.ai_value = ai;
    }
    let necromancy = |bp| {
        vec![
            HeroEffect::Necromancy {
                bp,
                unless_skill: None,
            },
            HeroEffect::Raises {
                priority: 0,
                creature: h3_content::RaisedCreature::Creature(key("t:imp")),
            },
        ]
    };
    for i in 4..28 {
        let levels = match i {
            12 => [necromancy(9000), necromancy(10_000), necromancy(5000)],
            27 => [
                vec![HeroEffect::FirstAid { heal_bp: 10_000 }],
                vec![],
                vec![],
            ],
            _ => [vec![], vec![], vec![]],
        };
        pack.skills.push(SecondarySkillDef {
            key: key(&alloc::format!("t:s{i}")),
            name: "Skill".into(),
            levels,
        });
    }
    for i in pack.artifacts.len()..artifacts {
        let k = alloc::format!("t:a{i}");
        pack.artifacts
            .push(artifact(&k, Some("t:ring_slot"), vec![]));
    }
    // Titan's Thunder grants the bolt.
    if let Some(thunder) = pack.artifacts.get_mut(135) {
        thunder.effects = vec![HeroEffect::Spells {
            grant: SpellGrant::Spell(key("t:bolt")),
        }];
    }
    // Fire at the base pack's place of Fire (2).
    for k in ["t:water", "t:fire"] {
        pack.spell_schools.push(SpellSchoolDef {
            key: key(k),
            name: "School".into(),
            protection_rank: 1,
        });
    }
    let gust = pack.spells[0].clone();
    for (k, category, granted) in [
        ("t:blaze", SpellCategory::Combat, false),
        ("t:spark", SpellCategory::Combat, false),
        ("t:ember", SpellCategory::Combat, false),
        ("t:bolt", SpellCategory::Combat, true),
    ] {
        pack.spells.push(SpellDef {
            key: key(k),
            category,
            schools: vec![key("t:fire")],
            granted_by_name_only: granted,
            ..gust.clone()
        });
    }
    edit(&mut pack);
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let part = AdventureRuleset::compile(&core, &[adventure_pack(vec![88, 25], 2)]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        let row: BattleCreatureDef = pack.creatures[1].clone();
        for i in pack.creatures.len()..65 {
            pack.creatures.push(BattleCreatureDef {
                key: key(&alloc::format!("t:c{i}")),
                ..row.clone()
            });
        }
        let ai = |kind, value| SpellAi {
            kinds: SpellAiKinds::default().with(kind),
            values: [value; 4],
        };
        pack.spells[0].ai = ai(SpellAiKind::Damage, 5);
        let gust: BattleSpellDef = pack.spells[0].clone();
        for (k, kind, value) in [
            ("t:blaze", SpellAiKind::Damage, 5),
            ("t:spark", SpellAiKind::DamageOnce, 7),
            ("t:ember", SpellAiKind::Special, 9),
            ("t:bolt", SpellAiKind::DamageOnce, 13),
        ] {
            pack.spells.push(BattleSpellDef {
                key: key(k),
                ai: ai(kind, value),
                ..gust.clone()
            });
        }
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(h3_towns::TownRuleset::empty(&core)),
    )
    .unwrap()
}

/// [`rules_of`] with 141 artifacts and the pack as it is.
fn rules() -> GameRuleset {
    rules_of(141, |_| {})
}

/// The crate's test game on `rules`.
fn game(rules: &GameRuleset) -> GameState {
    GameState::new(rules, &setup(), 3).unwrap()
}

/// Red's hero with 250 gogs (worth [`A`]), the spell book worn, the gust and the blaze learned,
/// Spell Power 3, Knowledge 2, `skills`.
fn hero(state: &GameState, rules: &GameRuleset, skills: &[(u16, u8)]) -> (HeroOnMap, HeroDetails) {
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
        count: 250,
    }];
    let mut details = state.hero(on_map.id).unwrap().clone();
    details.worn[0] = Some(core.artifact_by_key("t:book").unwrap().into());
    details.spells = vec![spell(rules, "t:gust"), spell(rules, "t:blaze")];
    details.primary = [1, 1, 3, 2];
    details.mana = 8;
    details.skills = skills
        .iter()
        .map(|&(raw, level)| (SkillId::from_raw(raw), level))
        .collect();
    (on_map, details)
}

fn spell(rules: &GameRuleset, k: &str) -> SpellId {
    rules.core().spell_by_key(k).unwrap()
}

/// The reads of a hero (no record: its gains 0, no weights).
fn valued<'a>(
    state: &'a GameState,
    rules: &'a GameRuleset,
    (on_map, details): &'a (HeroOnMap, HeroDetails),
) -> Valued<'a> {
    let hero = Fighter { on_map, details };
    Valued::new(state, rules, (&Record::default(), &[]), hero)
}

/// The id of artifact `raw` as an instance without a spell.
fn item(raw: u16) -> ArtifactInstance {
    ArtifactTypeId::from_raw(raw).into()
}

/// The value of `effect` (owner artifact 0) as a candidate and worn, not counted, then counted.
fn values(v: &Valued<'_>, effect: Effect) -> [i32; 3] {
    of(v, effect, 0)
}

/// [`values`] with the owner artifact `owner`.
fn of(v: &Valued<'_>, effect: Effect, owner: u16) -> [i32; 3] {
    let id = ArtifactTypeId::from_raw(owner);
    [
        effect.value(v, id, false, false),
        effect.value(v, id, true, false),
        effect.value(v, id, false, true),
    ]
}

/// The table's canonical words: per artifact by id, its id, each effect's code and parameters
/// as the list holds them (code 18's spell as 57), then −1.
fn words() -> Vec<i32> {
    let school = |s: School| match s {
        School::Air => 1,
        School::Fire => 2,
        School::Water => 4,
        School::Earth => 8,
    };
    let mut out = Vec::new();
    for (id, effects) in (0..).zip(TABLE) {
        if effects.is_empty() {
            continue;
        }
        out.push(id);
        for &effect in effects {
            let coded: Vec<i32> = match effect {
                AttackDefense(p) => vec![0, p],
                SpellPower(p) => vec![1, p],
                Knowledge(p) => vec![2, p],
                Morale(p) => vec![3, p],
                Luck(p) => vec![4, p],
                Scouting(p) => vec![5, p],
                Necromancy(p) => vec![6, p],
                ArmyShare(p) => vec![7, p],
                Movement(p) => vec![8, p],
                Mana(p) => vec![9, p],
                Duration(p) => vec![10, p],
                SchoolPower(p, s) => vec![11, school(s), p],
                SchoolSpells(s) => vec![12, school(s)],
                SpellBlock(p) => vec![13, p],
                EnemyMorale => vec![14],
                EnemyLuck => vec![15],
                Resource(p, q) => vec![16, p, q.into()],
                Growth(p, q) => vec![17, p.into(), q],
                Spell(p) => vec![18, p.into()],
                Shooters(p) => vec![19, p],
                Alliance(_) => vec![20],
                UndeadKing(_) => vec![21],
                Living => vec![22],
                TownGrowth => vec![23],
            };
            out.extend(coded);
        }
        out.push(-1);
    }
    out
}

// covers: CPU-ART-001
#[test]
fn the_table_is_the_executables_list() {
    // The list 0x63AC7C as `ai_artifact_effects_build` reads it, its artifacts by id: 636 words
    // and their FNV-1a (E31 T2f's reading).
    let words = words();
    assert_eq!(words.len(), 636);
    let mut h = StateHasher::new();
    for w in &words {
        h.write_bytes(&w.to_le_bytes());
    }
    assert_eq!(h.finish(), 0xA71B_1A7A_1076_6448);
    assert_eq!(TABLE.iter().filter(|e| !e.is_empty()).count(), 135);
    // The build's own parameters: the alliance's 8, the cloak's 30.
    assert_eq!(TABLE[129], [Alliance(8)]);
    assert_eq!(TABLE[130], [UndeadKing(30)]);
    assert_eq!(TABLE[135], [Spell(57)]);
}

// covers: CPU-ART-002, CPU-ART-004
#[test]
fn the_stat_and_army_codes() {
    let rules = rules();
    let state = game(&rules);
    let (mut on_map, details) = hero(&state, &rules, &[]);
    on_map.max_movement = 1500;
    let both = (on_map, details);
    let mut v = valued(&state, &rules, &both);
    assert_eq!(v.army, A);
    v.gains = SpellGains {
        power: 72,
        rounds: 7,
        knowledge: 15,
        doubled: 0,
        refilled: 0,
    };
    // The stats: nothing where counted.
    assert_eq!(values(&v, AttackDefense(3)), [750, 750, 0]);
    assert_eq!(values(&v, SpellPower(2)), [144, 144, 0]);
    assert_eq!(values(&v, Knowledge(-3)), [-45, -45, 0]);
    assert_eq!(values(&v, Duration(2)), [14, 14, 0]);
    // The army's shares, counted or not; the scouting on the day's points.
    assert_eq!(values(&v, ArmyShare(5)), [500; 3]);
    assert_eq!(values(&v, Movement(25)), [3125; 3]);
    assert_eq!(values(&v, Scouting(1)), [15; 3]);
    // The mana: none without Wisdom, nor without the power's gain.
    assert_eq!(values(&v, Mana(4)), [0; 3]);
    let wise = hero(&state, &rules, &[(7, 1)]);
    let mut w = valued(&state, &rules, &wise);
    assert_eq!(values(&w, Mana(4)), [0; 3]);
    w.gains.power = 1;
    assert_eq!(values(&w, Mana(4)), [400; 3]);
    // The spell block: a fifth (the orb) or an eighth, a worn one less 50 or 25 a Spell Power
    // point (3) where not counted.
    assert_eq!(values(&v, SpellBlock(0)), [2000, 1850, 2000]);
    assert_eq!(values(&v, SpellBlock(3)), [1250, 1175, 1250]);
    // A division truncates toward 0.
    v.army = -50;
    assert_eq!(values(&v, AttackDefense(1)), [-1, -1, 0]);
}

// covers: CPU-ART-004
#[test]
fn the_shooters_and_the_living() {
    let rules = rules();
    let state = game(&rules);
    let (mut on_map, details) = hero(&state, &rules, &[]);
    // 250 gogs, 3 carts (shooters, 150) and 5 imps (alive, 50).
    on_map.army.extend([
        ArmyStack {
            slot: 1,
            creature: CreatureTypeId::from_raw(2),
            count: 3,
        },
        ArmyStack {
            slot: 2,
            creature: CreatureTypeId::from_raw(0),
            count: 5,
        },
    ]);
    let both = (on_map, details);
    let v = valued(&state, &rules, &both);
    assert_eq!(v.army, 10_200);
    assert_eq!(values(&v, Shooters(5)), [7; 3]);
    assert_eq!(values(&v, Living), [6; 3]);
    // The Ammo Cart: a fortieth of each shooter stack.
    assert_eq!(hero_value(&v, item(AMMO_CART), false, false), 3);
}

// covers: CPU-ART-003
#[test]
fn the_morale_and_luck_codes_value_the_change() {
    let rules = rules();
    let state = game(&rules);
    let both = hero(&state, &rules, &[]);
    let mut v = valued(&state, &rules, &both);
    // From morale 2 by 1: 0.0173 × 10 000; worn the hero's own less 1, the same; nothing at 3.
    v.morale = 2;
    assert_eq!(values(&v, Morale(1)), [173, 173, 0]);
    v.morale = 3;
    assert_eq!(values(&v, Morale(1)), [0, 173, 0]);
    // From −1 by 1: across 0 (the slip): 0.0833; worn from −2 (the slip: −0.0173 + 2 × 0.0833).
    v.morale = -1;
    assert_eq!(values(&v, Morale(1)), [833, 1492, 0]);
    // Luck in singles: from 0, 173 (worn from −1: f32 0.0122, 121); from −2 by 1 (the slip:
    // −0.0173 + 2 × 0.0122), worn from −3 (−2 × 0.0173 + 3 × 0.0122).
    assert_eq!(values(&v, Luck(1)), [173, 121, 0]);
    v.luck = -2;
    assert_eq!(values(&v, Luck(1)), [70, 19, 0]);
    // The enemy's: the base 2 × 0.0173 × 10 000 = 346; the hero's own lost where not counted
    // and above 0, held to 3.
    v.morale = 0;
    assert_eq!(values(&v, EnemyMorale), [346; 3]);
    v.morale = 2;
    assert_eq!(values(&v, EnemyMorale), [0, 0, 346]);
    v.morale = 5;
    assert_eq!(values(&v, EnemyMorale), [-173, -173, 346]);
    v.luck = 1;
    assert_eq!(values(&v, EnemyLuck), [172, 172, 346]);
}

// covers: CPU-ART-003
#[test]
fn the_heros_own_morale_and_luck_are_its_own_sides() {
    let rules = rules_of(141, |pack| {
        pack.artifacts[5].effects = vec![
            HeroEffect::Morale {
                amount: 2,
                affects: Affects::Own,
            },
            HeroEffect::Luck {
                amount: 1,
                affects: Affects::Both,
            },
        ];
        pack.artifacts[6].effects = vec![
            HeroEffect::Morale {
                amount: -1,
                affects: Affects::Enemy,
            },
            HeroEffect::LuckAtMost {
                value: 0,
                affects: Affects::Enemy,
            },
        ];
        pack.artifacts[7].effects = vec![HeroEffect::LuckAtMost {
            value: 0,
            affects: Affects::Both,
        }];
    });
    let state = game(&rules);
    let wearing = |raws: &[u16]| {
        let (on_map, mut details) = hero(&state, &rules, &[]);
        for (at, &raw) in (2..).zip(raws) {
            details.worn[at] = Some(item(raw));
        }
        let both = (on_map, details);
        let v = valued(&state, &rules, &both);
        (v.morale, v.luck)
    };
    assert_eq!(wearing(&[]), (0, 0));
    // Its own +2 and its side's +1; the enemy's −1 and cap not read.
    assert_eq!(wearing(&[5, 6]), (2, 1));
    // A cap on its side (the Hourglass): its luck 0.
    assert_eq!(wearing(&[5, 7]), (2, 0));
}

// covers: CPU-ART-005
#[test]
fn necromancy_values_the_room_left_of_the_share() {
    let rules = rules();
    let state = game(&rules);
    let value = |skills: &[(u16, u8)], effect: Effect| {
        let both = hero(&state, &rules, skills);
        values(&valued(&state, &rules, &both), effect)
    };
    // Without the skill: nothing.
    assert_eq!(value(&[], Necromancy(15)), [0; 3]);
    // At Basic (f32 0.9): h = 10; a candidate's min(15, 10), a worn one's 15: A × n / 250.
    assert_eq!(value(&[(12, 1)], Necromancy(15)), [400, 600, 400]);
    // At Advanced (1.0): no room for a candidate.
    assert_eq!(value(&[(12, 2)], Necromancy(15)), [0, 600, 0]);
    // The cloak without the skill: code 6's value of 30 (no share: h = 100).
    assert_eq!(value(&[], UndeadKing(30)), [1200; 3]);
    // With it, times r of its level's creature: (90 − 60) / 60 = 0.5 at Basic (a candidate's n
    // 10: 400, a worn one's 30: 1 200); 1.5 at Advanced (worn, n 30); 4 at Expert (0.5: h = 50,
    // n 30).
    assert_eq!(value(&[(12, 1)], UndeadKing(30)), [200, 600, 200]);
    assert_eq!(value(&[(12, 2)], UndeadKing(30)), [0, 1800, 0]);
    assert_eq!(value(&[(12, 3)], UndeadKing(30)), [4800; 3]);
    // A Skeleton worth nothing: the FPU's indefinite.
    let worthless = rules_of(141, |pack| {
        pack.creatures[56].info.ai_value = 0;
    });
    let both = hero(&state, &worthless, &[(12, 1)]);
    assert_eq!(
        values(&valued(&state, &worthless, &both), UndeadKing(30)),
        [i32::MIN; 3]
    );
}

/// The value of `spell` on `v`'s caster record with the power `power`.
fn at_power(v: &Valued<'_>, spell: SpellId, power: i32) -> i32 {
    let mut record = v.caster.clone();
    record.power = power;
    record.value(v.rules.battle(), spell)
}

// covers: CPU-ART-006, CPU-ART-009
#[test]
fn the_spell_codes_and_the_scroll() {
    let rules = rules();
    let state = game(&rules);
    let (on_map, mut details) = hero(&state, &rules, &[]);
    // The ember (fire, no damage kind) learned too: the orb passes it.
    details.spells.push(spell(&rules, "t:ember"));
    let both = (on_map, details);
    let v = valued(&state, &rules, &both);
    let [blaze, spark, ember, bolt] =
        ["t:blaze", "t:spark", "t:ember", "t:bolt"].map(|k| spell(&rules, k));
    assert_eq!(v.caster.power, 3);
    // The orb: the blaze's gain from power 3 to 6 (a candidate), from 1 (300 / 200) to 3 (worn).
    let orb = SchoolPower(100, School::Fire);
    let candidate = at_power(&v, blaze, 6) - at_power(&v, blaze, 3);
    let worn = at_power(&v, blaze, 3) - at_power(&v, blaze, 1);
    assert!(candidate > 0 && worn > 0);
    assert_eq!(values(&v, orb), [candidate, worn, 0]);
    // No spell of the school available: 0; the Air orb on this core (no `base:air`): 0.
    assert_eq!(values(&v, SchoolPower(100, School::Air))[0], 0);
    // The tome: the best of the fire spells not learned (the spark, the bolt); a candidate's not
    // available either.
    let tome = SchoolSpells(School::Fire);
    let best = v
        .caster
        .value(rules.battle(), spark)
        .max(v.caster.value(rules.battle(), bolt));
    assert!(best > 0);
    assert_eq!(values(&v, tome), [best, best, 0]);
    // Titan's Thunder's bolt (the spell its artifact grants): its value, 0 where counted; an
    // artifact granting none, 0.
    let thunder = Spell(57);
    let value = v.caster.value(rules.battle(), bolt);
    assert!(value > 0);
    assert_eq!(of(&v, thunder, 135), [value, value, 0]);
    assert_eq!(values(&v, thunder), [0; 3]);
    // The scroll: its spell's gain (CPU-SPL-007); 0 for a learned spell, or a candidate's spell
    // the hero has available.
    let scroll = |s, worn| {
        let item = ArtifactInstance {
            artifact: ArtifactTypeId::from_raw(SCROLL),
            spell: Some(s),
        };
        hero_value(&v, item, worn, false)
    };
    assert_eq!(scroll(spark, false), spells::gain(&rules, &v.caster, spark));
    assert_eq!(scroll(blaze, false), 0);
    assert_eq!(hero_value(&v, item(SCROLL), false, false), 0);
    let ember_v = ember;
    let mut learned = valued(&state, &rules, &both);
    learned.available.push(spark);
    assert_eq!(scroll(ember_v, true), 0);
    let mut spark_scroll = ArtifactInstance::from(ArtifactTypeId::from_raw(SCROLL));
    spark_scroll.spell = Some(spark);
    assert_eq!(hero_value(&learned, spark_scroll, false, false), 0);
    assert_eq!(
        hero_value(&learned, spark_scroll, true, false),
        spells::gain(&rules, &learned.caster, spark)
    );
    // With the spark available, the tome's candidate passes it, a worn tome counts it; the bolt
    // available: the thunder's candidate 0, worn its value.
    learned.available.push(bolt);
    assert_eq!(values(&learned, tome)[1], best);
    assert_eq!(of(&learned, thunder, 135), [0, value, 0]);
    // Without power (no book): every spell code 0.
    let (on_map, mut details) = hero(&state, &rules, &[]);
    details.worn[0] = None;
    let bookless = (on_map, details);
    let none = valued(&state, &rules, &bookless);
    assert_eq!(none.caster.power, 0);
    for effect in [orb, tome, thunder] {
        assert_eq!(of(&none, effect, 135), [0; 3]);
    }
}

// covers: CPU-ART-006
#[test]
fn the_schools_are_the_base_packs_handles() {
    let ids = [School::Earth, School::Water, School::Fire, School::Air].map(School::id);
    assert_eq!(ids, [0, 1, 2, 3].map(SchoolId::from_raw));
}

// covers: CPU-ART-007
#[test]
fn a_resource_is_its_weight_three_times() {
    let rules = rules();
    let state = game(&rules);
    let both = hero(&state, &rules, &[]);
    let hero = Fighter {
        on_map: &both.0,
        details: &both.1,
    };
    let weights = [SignedX87::from(X87::ratio(1, 10, DOUBLE_BITS).unwrap())];
    let v = Valued::new(&state, &rules, (&Record::default(), &weights), hero);
    // 1 000 × 0.1 × 3.0; a resource without a weight 0.
    assert_eq!(values(&v, Resource(1000, 0)), [300; 3]);
    assert_eq!(values(&v, Resource(1000, 6)), [0; 3]);
}

// covers: CPU-ART-009
#[test]
fn the_machines_and_the_combinations() {
    let rules = rules_of(141, |pack| {
        pack.artifacts[8].parts = vec![key("t:a7"), key("t:a9")];
    });
    let state = game(&rules);
    let both = hero(&state, &rules, &[]);
    let v = valued(&state, &rules, &both);
    // The Ballista: Attack 1 (√2 × 500 = 707), no Artillery; q = 10 000 × 47 / 40 = 11 750.
    assert_eq!(
        hero_value(&v, item(BALLISTA), false, false),
        11_750 * 707 / 12_457
    );
    let gunner = hero(&state, &rules, &[(20, 2)]);
    let g = valued(&state, &rules, &gunner);
    // With Advanced Artillery s = 707 + 707: 11 750 × 1 414 / 13 164.
    assert_eq!(
        hero_value(&g, item(BALLISTA), false, false),
        11_750 * 1414 / 13_164
    );
    // The Tent: 25 hit points without First Aid, 50 with (its +100 %): a gog's 13 heals whole,
    // its value 40.
    assert_eq!(hero_value(&v, item(TENT), false, false), 40);
    let medic = hero(&state, &rules, &[(27, 1)]);
    assert_eq!(
        hero_value(&valued(&state, &rules, &medic), item(TENT), false, false),
        40
    );
    // An undead of 13 hit points worth 300, and a creature of the core past the tent's heal: a
    // share of it. The skeleton row is the gog's (13 hit points); none is past 25.
    let (mut on_map, details) = hero(&state, &rules, &[]);
    on_map.army.push(ArmyStack {
        slot: 1,
        creature: CreatureTypeId::from_raw(64),
        count: 1,
    });
    let liches = (on_map, details);
    assert_eq!(
        hero_value(&valued(&state, &rules, &liches), item(TENT), false, false),
        300
    );
    // Carts of 100 hit points: 25 of them healed (50 × 25 / 100), with First Aid 50.
    let carts = |skills: &[(u16, u8)]| {
        let (mut on_map, details) = hero(&state, &rules, skills);
        on_map.army = vec![ArmyStack {
            slot: 0,
            creature: CreatureTypeId::from_raw(2),
            count: 4,
        }];
        let both = (on_map, details);
        hero_value(&valued(&state, &rules, &both), item(TENT), false, false)
    };
    assert_eq!((carts(&[]), carts(&[(27, 1)])), (12, 25));
    // A combination: its own effects and its parts' (2, 1 and 3 points: 500 + 250 + 750);
    // counted, nothing.
    assert_eq!(hero_value(&v, item(8), false, false), 1500);
    assert_eq!(hero_value(&v, item(8), true, true), 0);
    // No entry (the Catapult) and an id past the table: 0.
    assert_eq!(hero_value(&v, item(3), false, false), 0);
    assert_eq!(hero_value(&v, item(200), false, false), 0);
}

// covers: CPU-ART-010
#[test]
fn a_value_to_a_hero_is_less_what_it_replaces() {
    let rules = rules();
    let state = game(&rules);
    let (on_map, mut details) = hero(&state, &rules, &[]);
    // A free ring position: the candidate's whole value (5 points: 1 250), at least 0.
    let both = (on_map.clone(), details.clone());
    let v = valued(&state, &rules, &both);
    assert_eq!(value_to_hero(&v, item(11), false), 1250);
    assert_eq!(value_to_hero(&v, item(14), false), 500);
    assert_eq!(value_to_hero(&v, item(30), false), 0);
    // Both rings worn (4 then 3 points): less the last one it would replace (3: 750).
    let at = rules.core().artifacts().len();
    assert!(at > 11);
    details.worn[2] = Some(item(10));
    details.worn[3] = Some(item(9));
    let full = (on_map.clone(), details.clone());
    let f = valued(&state, &rules, &full);
    assert_eq!(value_to_hero(&f, item(11), false), 1250 - 750);
    assert_eq!(value_to_hero(&f, item(7), false), 0);
    // Where counted, the stats count nothing.
    assert_eq!(value_to_hero(&f, item(11), true), 0);
    // An artifact no position takes and nothing to replace (the backpack's charm): its value.
    assert_eq!(value_to_hero(&f, item(2), false), 0);
}

// covers: CPU-ART-011, CPU-STA-008
#[test]
fn the_mean_of_the_artifacts_values_to_the_player() {
    // Nine artifacts: the mean counts the axe (1 point) and the blackshard (2).
    let rules = rules_of(9, |_| {});
    let state = game(&rules);
    let (on_map, details) = hero(&state, &rules, &[]);
    let both = (on_map, details);
    let heroes = [valued(&state, &rules, &both)];
    // To the player: at least 10.
    assert_eq!(value_to_player(&heroes, item(7)), 250);
    assert_eq!(value_to_player(&[], item(7)), 10);
    // Red's own hero in the state has its own army (no 250 gogs): the mean is over what it
    // values. A record with its gains.
    let record = start::record(&state, &rules, RED);
    let own: Vec<_> = state
        .adventure()
        .heroes()
        .iter()
        .filter(|h| h.owner == RED)
        .map(|h| fight::value_of(&rules, &h.army))
        .collect();
    let to_player = |points: i32| own.iter().fold(10, |best, &a| best.max(a * points / 40));
    let mean = (to_player(1) + to_player(2)) as f32 / 2.0;
    assert_eq!(record.artifacts, mean.to_bits());
    assert_eq!(super::mean(&state, &rules, &record, RED), mean.to_bits());
    // Fewer than eight artifacts: none counted, 0 / 0.
    let few = rules_of(5, |_| {});
    let few_state = game(&few);
    assert_eq!(start::record(&few_state, &few, RED).artifacts, INDEFINITE);
}

/// The town fixtures' rules for the town codes: the imp level 1 (its upgrade the gog), the gog
/// level 2, a Citadel, a Castle and both dwellings in map slots, an upgraded level 1 dwelling.
fn town_rules() -> GameRuleset {
    let mut core = crate::town::tests::towns_core_pack(false);
    core.creatures[0].info.upgrades = vec![key("t:gog")];
    for (creature, ai) in core.creatures.iter_mut().zip([10, 40, 50]) {
        creature.info.ai_value = ai;
    }
    core.artifacts.push(artifact(
        "t:alliance",
        Some("t:ring_slot"),
        vec![
            HeroEffect::AlignmentGroup {
                factions: vec![key("t:town"), key("t:other")],
            },
            HeroEffect::Morale {
                amount: 1,
                affects: Affects::Own,
            },
        ],
    ));
    let mut towns = crate::town::tests::town_pack(false);
    let record = &mut towns.towns[0];
    record.creatures[0].upgrade = Some(key("t:gog"));
    for (at, slot) in [(4, 7), (5, 8), (6, 9), (7, 30), (8, 31)] {
        record.buildings[at].slot = Some(slot);
    }
    record.buildings.push(BuildingDef {
        key: "upgraded".into(),
        name: "Upgraded".into(),
        slot: Some(37),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles: BuildingRoles {
            dwelling: Some(Dwelling {
                level: 1,
                upgraded: true,
            }),
            ..BuildingRoles::default()
        },
        war_machine: None,
    });
    crate::town::tests::rules_with_adventure(core, &towns, |_| {})
}

/// The town fixtures' game: both towns red's, T0 with the Castle and the upgraded level 1, T1
/// with the Citadel (else a Fort alone) and level 2; red's hero on T0's gate, in its garrison
/// with `garrison`.
fn town_game(rules: &GameRuleset, garrison: bool) -> GameState {
    town_game_with(rules, garrison, true)
}

/// [`town_game`], T1's Citadel with `citadel`.
fn town_game_with(rules: &GameRuleset, garrison: bool, citadel: bool) -> GameState {
    let mut setup: GameSetup = crate::town::tests::towns_setup(rules);
    setup.map.objects[1].owner = Some(RED);
    let built = |slots: &[u8]| BuildingsSetup::Custom {
        built: slots.iter().fold(SlotSet::EMPTY, |set, &s| set.with(s)),
        forbidden: SlotSet::EMPTY,
    };
    setup.towns[0].buildings = built(&[7, 8, 9, 30, 37]);
    setup.towns[1].buildings = if citadel {
        built(&[7, 8, 31])
    } else {
        built(&[7, 31])
    };
    let entrance = setup.map.objects[0].anchor;
    setup.heroes[0].pos = entrance;
    let mut state = GameState::new(rules, &setup, 3).unwrap();
    if garrison {
        let enter = GameCommand::Town(TownCommand::Heroes {
            town: TownId::new(0),
            change: TownHeroChange::Enter,
        });
        let mut events: Vec<GameEvent> = Vec::new();
        apply(&mut state, rules, Actor::Player(RED), enter, &mut events).unwrap();
    }
    state
}

/// Red's hero of `state` (on the map, else in T0's garrison) with its details.
fn red_hero(state: &GameState) -> (HeroOnMap, HeroDetails) {
    let adventure = state.adventure();
    let on_map = adventure
        .heroes()
        .iter()
        .find(|h| h.owner == RED)
        .or_else(|| adventure.town_hero(TownId::new(0)))
        .unwrap()
        .clone();
    let details = state.hero(on_map.id).unwrap().clone();
    (on_map, details)
}

// covers: CPU-ART-007, CPU-ART-008
#[test]
fn the_town_codes() {
    let rules = town_rules();
    for garrison in [false, true] {
        let state = town_game(&rules, garrison);
        let both = red_hero(&state);
        assert_eq!(both.0.pos, Pos::new(1, 1, 0));
        let v = valued(&state, &rules, &both);
        // Level 0's dwelling: T0's upgraded (the gog, 40) × 5; level 1's: T1's gog × 4; no
        // level 2. Counted on T0's gate: its garrison hero's 200, else 1; level 1 not at T0: 0.
        let at_gate = if garrison { 200 } else { 1 };
        assert_eq!(values(&v, Growth(0, 5)), [200, 200, at_gate]);
        assert_eq!(values(&v, Growth(1, 4)), [160, 160, 0]);
        assert_eq!(values(&v, Growth(2, 1)), [0; 3]);
        // The Legion's statue: T0's gog with its Castle (g + g), T1's gog with its Citadel
        // (g + g / 2), each AI × growth / 2.
        let gog = rules.core().creature(CreatureTypeId::from_raw(1)).unwrap();
        let g = i32::from(gog.growth);
        let expected = 40 * (g + g) / 2 + 40 * (g + g / 2) / 2;
        assert_eq!(values(&v, TownGrowth), [expected; 3]);
        // The alliance: 5 % of the joined factions' creatures in the heroes' and the upper
        // bars' armies, plus A × 8 / 40 where not counted.
        let alliance = rules.core().artifact_by_key("t:alliance").unwrap();
        let joined: i32 = state
            .adventure()
            .heroes()
            .iter()
            .filter(|h| h.owner == RED)
            .map(|h| &h.army[..])
            .chain(state.adventure().upper_bars(RED))
            .flatten()
            .filter_map(|s| {
                let c = rules.core().creature(s.creature)?;
                c.faction.map(|_| word(c.ai_value) * word(s.count))
            })
            .sum();
        let own = v.army * 8 / 40;
        let value = |worn, counted| Alliance(8).value(&v, alliance, worn, counted);
        assert_eq!(value(false, false), joined * 5 / 100 + own);
        assert_eq!(value(true, true), joined * 5 / 100);
        assert!(joined > 0);
    }
    // T1 with a Fort alone: no castle's part there.
    let state = town_game_with(&rules, false, false);
    let both = red_hero(&state);
    let gog = rules.core().creature(CreatureTypeId::from_raw(1)).unwrap();
    let g = i32::from(gog.growth);
    assert_eq!(
        values(&valued(&state, &rules, &both), TownGrowth)[0],
        40 * (g + g) / 2 + 40 * g / 2
    );
    // Off a town's gate: counted, 0.
    let state = town_game(&rules, false);
    let (mut on_map, details) = red_hero(&state);
    on_map.pos = Pos::new(0, 0, 0);
    let away = (on_map, details);
    assert_eq!(values(&valued(&state, &rules, &away), Growth(0, 5))[2], 0);
}
