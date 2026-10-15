extern crate std;

use std::vec;
use std::vec::Vec;

use super::*;
use crate::effect::{Affects, CreatureSelector, CreatureStat, HeroEffect, SpellGrant, SpellScope};
use crate::pack::{
    HeroClassDef, HeroParamsDef, HeroTypeDef, ProfileDef, SecondarySkillDef, SpellDef,
    SpellSchoolDef, StartingStackDef,
};
use crate::ruleset::Ruleset;
use crate::ruleset::tests::{creature, faction, key, manifest};

fn school(k: &str) -> SpellSchoolDef {
    SpellSchoolDef {
        key: key(k),
        name: "S".into(),
        protection_rank: 0,
    }
}

fn spell(k: &str, schools: &[&str]) -> SpellDef {
    SpellDef {
        key: key(k),
        name: "Spell".into(),
        category: SpellCategory::Combat,
        level: 1,
        schools: schools.iter().map(|s| key(s)).collect(),
        cost: [5, 4, 4, 4],
        power: 10,
        effect: [10, 10, 20, 30],
        granted_by_name_only: false,
    }
}

fn skill(k: &str, levels: [Vec<HeroEffectDef>; 3]) -> SecondarySkillDef {
    SecondarySkillDef {
        key: key(k),
        name: "Skill".into(),
        levels,
    }
}

fn class(k: &str) -> HeroClassDef {
    HeroClassDef {
        key: key(k),
        name: "Class".into(),
        faction: key("t:town"),
        primary: [2, 2, 1, 1],
        primary_gain: [[35, 45, 10, 10], [30, 30, 20, 20]],
        skill_weights: vec![(key("t:archery"), 5), (key("t:tactics"), 3)],
    }
}

fn hero(k: &str) -> HeroTypeDef {
    HeroTypeDef {
        key: key(k),
        name: "Hero".into(),
        class: key("t:knight"),
        skills: vec![(key("t:tactics"), 1), (key("t:archery"), 2)],
        spell_book: true,
        spells: vec![key("t:bolt")],
        army: vec![
            StartingStackDef {
                creature: key("t:peasant"),
                count: [10, 20],
            },
            StartingStackDef {
                creature: key("t:peasant"),
                count: [1, 1],
            },
        ],
        war_machines: vec![key("t:cart")],
        specialty: vec![HeroEffect::SkillSpecialty {
            skill: key("t:archery"),
            bp_per_level: 500,
        }],
    }
}

fn levels(effect: impl Fn(u16) -> HeroEffectDef) -> [Vec<HeroEffectDef>; 3] {
    [vec![effect(1000)], vec![effect(2500)], vec![effect(5000)]]
}

/// A pack with every hero-content kind: two schools, a hero spell and a creature ability, two
/// skills, a class and a hero, plus the creatures they refer to.
fn pack() -> ContentPack {
    let mut cart = creature("t:cart");
    cart.speed = 0;
    cart.info.flags = vec!["war_machine".into()];
    let mut peasant = creature("t:peasant");
    // A forward reference: `t:farmer` is defined after `t:peasant`.
    peasant.info.upgrades = vec![key("t:farmer")];
    let mut ability = spell("t:gaze", &[]);
    ability.category = SpellCategory::Ability;
    ability.level = 0;
    ContentPack {
        factions: vec![faction("t:town")],
        creatures: vec![peasant, creature("t:farmer"), cart],
        spell_schools: vec![school("t:air"), school("t:fire")],
        spells: vec![spell("t:bolt", &["t:air", "t:fire"]), ability],
        skills: vec![
            skill("t:archery", levels(|bp| HeroEffect::RangedDamage { bp })),
            skill(
                "t:tactics",
                [1, 2, 3].map(|level| vec![HeroEffect::Tactics { level }]),
            ),
        ],
        hero_classes: vec![class("t:knight")],
        heroes: vec![hero("t:orrin")],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    }
}

/// A change to the test pack and the error it causes.
type Case = (fn(&mut ContentPack), CompileError);

fn compile(packs: &[ContentPack]) -> Result<Ruleset, CompileError> {
    Ruleset::compile(packs, "t:faithful")
}

fn fails(p: ContentPack) -> CompileError {
    compile(&[p]).unwrap_err()
}

fn invalid(kind: &'static str, k: &str, reason: &'static str) -> CompileError {
    CompileError::InvalidContent {
        kind,
        key: k.into(),
        reason,
    }
}

#[test]
fn names_round_trip() {
    for c in SpellCategory::ALL {
        assert_eq!(SpellCategory::from_name(c.name()), Some(c));
    }
    assert_eq!(SpellCategory::from_name("hero"), None);
}

#[test]
fn compiles_every_kind_with_resolved_references() {
    let r = compile(&[pack()]).unwrap();
    assert_eq!(r.spell_schools().len(), 2);
    assert_eq!(r.spell_schools()[1].key.as_str(), "t:fire");

    let bolt = r.spell_by_key("t:bolt").unwrap();
    let s = r.spell(bolt).unwrap();
    assert_eq!(s.schools.len(), 2);
    assert_eq!(
        (s.cost, s.power, s.effect),
        ([5, 4, 4, 4], 10, [10, 10, 20, 30])
    );
    assert_eq!(r.spells().len(), 2);

    let archery = r.skill_by_key("t:archery").unwrap();
    let a = r.skill(archery).unwrap();
    assert_eq!(a.effects(2), &[HeroEffect::RangedDamage { bp: 2500 }]);
    assert!(a.effects(0).is_empty() && a.effects(4).is_empty());
    assert_eq!(r.skills().len(), 2);

    let class = r.hero_class(Handle::from_raw(0)).unwrap();
    assert_eq!(r.hero_classes().len(), 1);
    assert_eq!(class.skill_weights.len(), 2);
    assert_eq!(class.skill_weights[0], (archery, 5));

    let orrin = r.hero_type_by_key("t:orrin").unwrap();
    let h = r.hero_type(orrin).unwrap();
    assert_eq!(r.hero_types().len(), 1);
    assert_eq!(h.skills[1], (archery, 2));
    assert_eq!(h.spells, vec![bolt]);
    let peasant = r.creature_by_key("t:peasant").unwrap();
    assert_eq!(h.army[1].creature, peasant);
    assert_eq!(h.army[0].count, [10, 20]);
    assert_eq!(h.war_machines, vec![r.creature_by_key("t:cart").unwrap()]);
    assert_eq!(
        h.specialty,
        vec![HeroEffect::SkillSpecialty {
            skill: archery,
            bp_per_level: 500
        }]
    );
    assert_eq!(
        r.upgrades(peasant),
        &[r.creature_by_key("t:farmer").unwrap()]
    );
    assert!(r.upgrades(Handle::from_raw(99)).is_empty());
    assert!(r.hero_type(Handle::from_raw(9)).is_none());
}

#[test]
fn every_effect_kind_resolves_and_the_hash_sees_all_content() {
    let mut p = pack();
    p.skills.push(skill(
        "t:all",
        [
            vec![
                HeroEffect::Morale {
                    amount: 1,
                    affects: Affects::Own,
                },
                HeroEffect::Luck {
                    amount: -1,
                    affects: Affects::Enemy,
                },
                HeroEffect::MeleeDamage { bp: 1000 },
                HeroEffect::DamageReduction { bp: 500 },
                HeroEffect::SurrenderDiscount { bp: 2000 },
            ],
            vec![
                HeroEffect::Artillery {
                    level: 2,
                    shots: 2,
                    double_damage_chance_bp: 7500,
                },
                HeroEffect::FirstAid { heal_bp: 10_000 },
                HeroEffect::SchoolMastery {
                    school: key("t:fire"),
                    level: 2,
                },
                HeroEffect::SpellDamage {
                    scope: SpellScope::School(key("t:air")),
                    bp: 5000,
                },
                HeroEffect::SpellDamage {
                    scope: SpellScope::All,
                    bp: 500,
                },
                HeroEffect::SpellBook,
            ],
            vec![
                HeroEffect::CreatureStat {
                    creatures: CreatureSelector::All,
                    stat: CreatureStat::Speed,
                    amount: 2,
                },
                HeroEffect::CreatureStat {
                    creatures: CreatureSelector::Flag(CreatureFlag::Dragon),
                    stat: CreatureStat::Attack,
                    amount: 5,
                },
                // E04b T8a: spells given and a mastery raise (a scroll belongs to an artifact).
                HeroEffect::Spells {
                    grant: SpellGrant::School(key("t:air")),
                },
                HeroEffect::Spells {
                    grant: SpellGrant::Level(1),
                },
                HeroEffect::Spells {
                    grant: SpellGrant::Spell(key("t:bolt")),
                },
                HeroEffect::SpellMastery {
                    spell: key("t:bolt"),
                    level: 3,
                },
            ],
        ],
    ));
    p.heroes[0].specialty = vec![
        HeroEffect::CreatureSpecialty {
            creature: key("t:peasant"),
            bp_per_step: 500,
            speed: 1,
        },
        HeroEffect::SpellSpecialty {
            spell: key("t:bolt"),
            bp_per_level: 300,
        },
        HeroEffect::SpellDamage {
            scope: SpellScope::Spell(key("t:bolt")),
            bp: 5000,
        },
        HeroEffect::CreatureStat {
            creatures: CreatureSelector::Creature(key("t:farmer")),
            stat: CreatureStat::MaxDamage,
            amount: 10,
        },
    ];
    let base = compile(&[pack()]).unwrap().content_hash();
    let r = compile(&[p.clone()]).unwrap();
    assert_ne!(r.content_hash(), base);
    assert_eq!(r.skills()[2].effects(3).len(), 6);
    assert_eq!(r.hero_types()[0].specialty.len(), 4);
    let fire = r.spell_schools().len() - 1;
    assert_eq!(
        r.skills()[2].effects(2)[2],
        HeroEffect::SchoolMastery {
            school: Handle::from_raw(u16::try_from(fire).unwrap()),
            level: 2
        }
    );

    let air = Handle::from_raw(0);
    assert_eq!(
        r.skills()[2].effects(3)[2],
        HeroEffect::Spells {
            grant: SpellGrant::School(air)
        }
    );
    // A grant by school or level gives the ruleset's spells of that school or level that are
    // neither abilities nor granted by name only; a named grant gives its spell (HER-SPL-002).
    let (bolt, gaze) = (Handle::from_raw(0), Handle::from_raw(1));
    assert!(SpellGrant::School(air).gives(&r, bolt));
    assert!(SpellGrant::Level(1).gives(&r, bolt));
    assert!(!SpellGrant::Level(0).gives(&r, gaze));
    assert!(!SpellGrant::Level(2).gives(&r, bolt));
    assert!(SpellGrant::Spell(gaze).gives(&r, gaze));
    assert!(!SpellGrant::Spell(gaze).gives(&r, bolt));
    assert!(!SpellGrant::Spell(Handle::from_raw(9)).gives(&r, Handle::from_raw(9)));
    let mut named = pack();
    named.spells[0].granted_by_name_only = true;
    let n = compile(&[named]).unwrap();
    assert!(!SpellGrant::School(air).gives(&n, bolt));
    assert!(!SpellGrant::Level(1).gives(&n, bolt));
    assert!(SpellGrant::Spell(bolt).gives(&n, bolt));

    // Every kind of entry contributes to the hash.
    let changes: [fn(&mut ContentPack); 7] = [
        |p| p.spell_schools[0].name = "X".into(),
        |p| p.spells[0].effect[3] = 31,
        |p| p.spells[0].granted_by_name_only = true,
        |p| p.skills[0].levels[0] = vec![HeroEffect::RangedDamage { bp: 999 }],
        |p| p.hero_classes[0].skill_weights[0].1 = 6,
        |p| p.heroes[0].army[0].count = [10, 21],
        |p| p.creatures[0].info.upgrades.clear(),
    ];
    for change in changes {
        let mut q = pack();
        change(&mut q);
        assert_ne!(compile(&[q]).unwrap().content_hash(), base);
    }
}

#[test]
fn packs_refer_to_their_closure_only() {
    let mut ext = ContentPack::new(manifest("x", &["t"]));
    let mut h = hero("x:edric");
    h.class = key("t:knight");
    ext.heroes.push(h);
    ext.skills.push(skill(
        "x:luck",
        levels(|_| HeroEffect::Luck {
            amount: 1,
            affects: Affects::Own,
        }),
    ));
    let r = compile(&[pack(), ext.clone()]).unwrap();
    assert_eq!(r.hero_types().len(), 2);
    assert_eq!(r.skill_by_key("x:luck").unwrap().raw(), 2);

    // A pack that does not depend on `x` cannot use its content.
    let mut other = ContentPack::new(manifest("y", &["t"]));
    let mut h = hero("y:gem");
    h.skills = vec![(key("x:luck"), 1)];
    other.heroes.push(h);
    assert_eq!(
        compile(&[pack(), ext, other]).unwrap_err(),
        CompileError::ReferenceOutsideDependencies {
            key: "x:luck".into(),
            pack: "y".into()
        }
    );
}

/// Every reference site rejects unknown keys.
#[test]
fn unknown_references_are_rejected() {
    let unknown = |kind: &'static str, k: &str| CompileError::UnknownReference {
        kind,
        key: k.into(),
    };
    let cases: Vec<Case> = vec![
        (
            |p| p.creatures[0].info.upgrades = vec![key("t:nope")],
            unknown("creature", "t:nope"),
        ),
        (
            |p| p.spells[0].schools = vec![key("t:earth")],
            unknown("spell school", "t:earth"),
        ),
        (
            |p| p.skills[0].levels[0] = vec![creature_stat("t:nope")],
            unknown("creature", "t:nope"),
        ),
        (
            |p| p.skills[0].levels[1] = vec![creature_stat("t:nope")],
            unknown("creature", "t:nope"),
        ),
        (
            |p| p.skills[0].levels[2] = vec![creature_stat("t:nope")],
            unknown("creature", "t:nope"),
        ),
        (
            |p| p.hero_classes[0].faction = key("t:city"),
            unknown("faction", "t:city"),
        ),
        (
            |p| p.hero_classes[0].skill_weights[1].0 = key("t:magic"),
            unknown("secondary skill", "t:magic"),
        ),
        (
            |p| p.heroes[0].army[1].creature = key("t:imp"),
            unknown("creature", "t:imp"),
        ),
        (
            |p| p.heroes[0].spells = vec![key("t:nova")],
            unknown("spell", "t:nova"),
        ),
        (
            |p| p.heroes[0].war_machines = vec![key("t:tent")],
            unknown("creature", "t:tent"),
        ),
        (
            |p| p.heroes[0].class = key("t:wizard"),
            unknown("hero class", "t:wizard"),
        ),
        (
            |p| p.heroes[0].skills = vec![(key("t:magic"), 1)],
            unknown("secondary skill", "t:magic"),
        ),
        (
            |p| {
                p.heroes[0].specialty = vec![HeroEffect::SkillSpecialty {
                    skill: key("t:magic"),
                    bp_per_level: 1,
                }];
            },
            unknown("secondary skill", "t:magic"),
        ),
        (
            |p| {
                p.heroes[0].specialty = vec![HeroEffect::SpellSpecialty {
                    spell: key("t:nova"),
                    bp_per_level: 300,
                }];
            },
            unknown("spell", "t:nova"),
        ),
        (
            |p| {
                p.skills[0].levels[0] = vec![HeroEffect::SchoolMastery {
                    school: key("t:earth"),
                    level: 1,
                }];
            },
            unknown("spell school", "t:earth"),
        ),
        (
            |p| {
                p.skills[0].levels[0] = vec![HeroEffect::SpellDamage {
                    scope: SpellScope::School(key("t:earth")),
                    bp: 1,
                }];
            },
            unknown("spell school", "t:earth"),
        ),
        (
            |p| {
                p.skills[0].levels[0] = vec![HeroEffect::SpellDamage {
                    scope: SpellScope::Spell(key("t:nova")),
                    bp: 1,
                }];
            },
            unknown("spell", "t:nova"),
        ),
        (
            |p| {
                p.heroes[0].specialty = vec![HeroEffect::CreatureSpecialty {
                    creature: key("t:imp"),
                    bp_per_step: 1,
                    speed: 0,
                }];
            },
            unknown("creature", "t:imp"),
        ),
    ];
    for (change, err) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(fails(p), err);
    }
}

fn creature_stat(c: &str) -> HeroEffectDef {
    HeroEffect::CreatureStat {
        creatures: CreatureSelector::Creature(key(c)),
        stat: CreatureStat::Defense,
        amount: 1,
    }
}

#[test]
fn keys_are_unique_and_in_the_pack_namespace() {
    let outside = |k: &str| CompileError::KeyOutsidePack {
        key: k.into(),
        pack: "t".into(),
    };
    let cases: Vec<Case> = vec![
        (
            |p| p.spell_schools.push(school("o:water")),
            outside("o:water"),
        ),
        (
            |p| p.spells.push(spell("o:nova", &["t:air"])),
            outside("o:nova"),
        ),
        (
            |p| p.skills.push(skill("o:luck", Default::default())),
            outside("o:luck"),
        ),
        (
            |p| p.hero_classes.push(class("o:cleric")),
            outside("o:cleric"),
        ),
        (|p| p.heroes.push(hero("o:gem")), outside("o:gem")),
        (
            |p| p.heroes.push(hero("t:orrin")),
            CompileError::DuplicateKey("t:orrin".into()),
        ),
        (
            |p| p.spells[0].schools.push(key("t:air")),
            CompileError::DuplicateKey("t:air".into()),
        ),
        (
            |p| p.heroes[0].spells.push(key("t:bolt")),
            CompileError::DuplicateKey("t:bolt".into()),
        ),
        (
            |p| p.heroes[0].war_machines.push(key("t:cart")),
            CompileError::DuplicateKey("t:cart".into()),
        ),
        (
            |p| p.heroes[0].skills.push((key("t:tactics"), 3)),
            CompileError::DuplicateKey("t:tactics".into()),
        ),
    ];
    for (change, err) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(fails(p), err);
    }
}

#[test]
fn too_many_entries() {
    let mut p = pack();
    p.spell_schools = (0..=usize::from(u16::MAX) + 1)
        .map(|i| school(&std::format!("t:s{i}")))
        .collect();
    assert_eq!(fails(p), CompileError::TooManyEntries("spell school"));
}

#[test]
fn at_most_128_spells() {
    let mut p = pack();
    p.spells = (0..=MAX_SPELLS)
        .map(|i| spell(&std::format!("t:s{i}"), &["t:air"]))
        .collect();
    p.heroes[0].spells.clear();
    p.heroes[0].specialty.clear();
    assert_eq!(fails(p.clone()), CompileError::TooManyEntries("spell"));
    p.spells.pop();
    assert!(compile(&[p]).is_ok());
}

#[test]
fn at_most_four_spell_schools() {
    let mut p = pack();
    p.spell_schools = (0..5).map(|i| school(&std::format!("t:s{i}"))).collect();
    p.spells[0].schools = vec![key("t:s0")];
    assert_eq!(
        fails(p.clone()),
        CompileError::TooManyEntries("spell school")
    );
    p.spell_schools.pop();
    assert!(compile(&[p]).is_ok());
}

#[test]
fn invalid_entries_are_rejected() {
    let cases: Vec<Case> = vec![
        // The spell effects (E04b T8a): a scroll belongs to an artifact; a grant or a raise
        // names a hero spell, never a creature ability (on a skill, a specialty, an artifact).
        (
            |p| p.skills[0].levels[0] = vec![HeroEffect::Scroll],
            invalid(
                "secondary skill",
                "t:archery",
                "scroll is allowed on artifacts only",
            ),
        ),
        (
            |p| p.heroes[0].specialty = vec![HeroEffect::Scroll],
            invalid(
                "hero type",
                "t:orrin",
                "scroll is allowed on artifacts only",
            ),
        ),
        (
            |p| {
                p.skills[0].levels[0] = vec![HeroEffect::SpellMastery {
                    spell: key("t:gaze"),
                    level: 1,
                }];
            },
            invalid(
                "secondary skill",
                "t:archery",
                "spells and spell_mastery name hero spells, not creature abilities",
            ),
        ),
        (
            |p| {
                p.heroes[0].specialty = vec![HeroEffect::Spells {
                    grant: SpellGrant::Spell(key("t:gaze")),
                }];
            },
            invalid(
                "hero type",
                "t:orrin",
                "spells and spell_mastery name hero spells, not creature abilities",
            ),
        ),
        (
            |p| {
                p.artifact_slots.push(crate::pack::ArtifactSlotDef {
                    key: key("t:misc"),
                    name: "Misc".into(),
                    count: 1,
                    backpack: true,
                });
                p.artifacts.push(crate::pack::ArtifactDef {
                    key: key("t:wand"),
                    name: "Wand".into(),
                    class: crate::ArtifactClass::Treasure,
                    cost: vec![],
                    slot: Some(key("t:misc")),
                    war_machine: None,
                    needs_skill: None,
                    effects: vec![HeroEffect::Spells {
                        grant: SpellGrant::Spell(key("t:gaze")),
                    }],
                    parts: vec![],
                });
            },
            invalid(
                "artifact",
                "t:wand",
                "spells and spell_mastery name hero spells, not creature abilities",
            ),
        ),
        (
            |p| p.spells[0].level = 6,
            invalid(
                "spell",
                "t:bolt",
                "level must be 1..=5 (creature abilities: 0..=5)",
            ),
        ),
        (
            |p| p.spells[0].level = 0,
            invalid(
                "spell",
                "t:bolt",
                "level must be 1..=5 (creature abilities: 0..=5)",
            ),
        ),
        (
            |p| p.spells[0].schools.clear(),
            invalid(
                "spell",
                "t:bolt",
                "hero spells need a school, creature abilities have none",
            ),
        ),
        (
            |p| p.spells[1].schools = vec![key("t:air")],
            invalid(
                "spell",
                "t:gaze",
                "hero spells need a school, creature abilities have none",
            ),
        ),
        (
            |p| p.skills[1].levels[2] = vec![HeroEffect::Tactics { level: 4 }],
            invalid(
                "secondary skill",
                "t:tactics",
                "tactics level must be 1..=3",
            ),
        ),
        (
            |p| p.hero_classes[0].primary_gain[1][0] = 31,
            invalid(
                "hero class",
                "t:knight",
                "each primary_gain row must sum to 100",
            ),
        ),
        (
            |p| {
                p.heroes[0].skills = (0..9).map(|i| (key(&std::format!("t:s{i}")), 1)).collect();
                for i in 0..9 {
                    p.skills
                        .push(skill(&std::format!("t:s{i}"), Default::default()));
                }
            },
            invalid("hero type", "t:orrin", "at most 8 secondary skills"),
        ),
        (
            |p| p.heroes[0].skills[0].1 = 0,
            invalid("hero type", "t:orrin", "skill levels must be 1..=3"),
        ),
        (
            |p| p.heroes[0].spell_book = false,
            invalid("hero type", "t:orrin", "starting spells need a spell book"),
        ),
        (
            |p| p.heroes[0].army[1].count = [0, 0],
            invalid(
                "hero type",
                "t:orrin",
                "starting stacks need 1 <= low <= high",
            ),
        ),
        (
            |p| p.heroes[0].army[1].count = [5, 4],
            invalid(
                "hero type",
                "t:orrin",
                "starting stacks need 1 <= low <= high",
            ),
        ),
        (
            |p| p.heroes[0].army[1].creature = key("t:cart"),
            invalid(
                "hero type",
                "t:orrin",
                "war machines belong in war_machines, not in the army",
            ),
        ),
        (
            |p| p.heroes[0].war_machines = vec![key("t:farmer")],
            invalid(
                "hero type",
                "t:orrin",
                "war_machines must have flag war_machine",
            ),
        ),
        (
            |p| {
                p.heroes[0].specialty = vec![HeroEffect::SkillSpecialty {
                    skill: key("t:archery"),
                    bp_per_level: 0,
                }];
            },
            invalid(
                "hero type",
                "t:orrin",
                "skill_specialty bp_per_level must be at least 1",
            ),
        ),
        (
            |p| {
                p.skills[0].levels[0] = vec![HeroEffect::SpellSpecialty {
                    spell: key("t:bolt"),
                    bp_per_level: 300,
                }];
            },
            invalid(
                "secondary skill",
                "t:archery",
                "specialty effects are allowed in a hero's specialty only",
            ),
        ),
    ];
    for (change, err) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(fails(p), err);
    }
}

#[test]
fn upgrades_are_checked_and_overridable() {
    let creature_err = |reason| CompileError::InvalidCreature {
        key: "t:peasant".into(),
        reason,
    };
    let cases: Vec<Case> = vec![
        (
            |p| p.creatures[0].info.upgrades = vec![key("t:farmer"); 4],
            creature_err("at most 3 upgrades"),
        ),
        (
            |p| p.creatures[0].info.upgrades = vec![key("t:peasant")],
            creature_err("a creature cannot upgrade into itself"),
        ),
        (
            |p| p.creatures[0].info.upgrades = vec![key("t:farmer"), key("t:farmer")],
            creature_err("an upgrade is listed twice"),
        ),
    ];
    for (change, err) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(fails(p), err);
    }
    // An override replaces the list; unknown targets are errors.
    let mut ext = ContentPack::new(manifest("x", &["t"]));
    ext.creature_overrides.push(crate::CreatureOverride {
        fixes: None,
        target: key("t:farmer"),
        patch: crate::CreaturePatch {
            upgrades: Some(vec![key("t:peasant")]),
            ..Default::default()
        },
    });
    let r = compile(&[pack(), ext.clone()]).unwrap();
    let farmer = r.creature_by_key("t:farmer").unwrap();
    assert_eq!(
        r.upgrades(farmer),
        &[r.creature_by_key("t:peasant").unwrap()]
    );
    ext.creature_overrides[0].patch.upgrades = Some(vec![key("t:nope")]);
    assert_eq!(
        compile(&[pack(), ext]).unwrap_err(),
        CompileError::UnknownReference {
            kind: "creature",
            key: "t:nope".into()
        }
    );
}

#[test]
fn a_later_override_can_invalidate_a_hero() {
    // `t:cart` stops being a war machine: the hero's war machine is no longer valid.
    let mut ext = ContentPack::new(manifest("x", &["t"]));
    ext.creature_overrides.push(crate::CreatureOverride {
        fixes: None,
        target: key("t:cart"),
        patch: crate::CreaturePatch {
            flags: Some(vec![]),
            speed: Some(1),
            ..Default::default()
        },
    });
    assert_eq!(
        compile(&[pack(), ext]).unwrap_err(),
        invalid(
            "hero type",
            "t:orrin",
            "war_machines must have flag war_machine"
        )
    );
}

fn params() -> HeroParamsDef {
    HeroParamsDef {
        primary_min: [0, 0, 1, 1],
        primary_max: 99,
        backpack: 64,
        learn_level: 2,
    }
}

#[test]
fn hero_params_are_optional_unique_and_checked() {
    let without = compile(&[pack()]).unwrap();
    assert_eq!(without.hero_params(), None);
    let mut p = pack();
    p.hero_params = Some(params());
    let with = compile(&[p.clone()]).unwrap();
    assert_eq!(
        with.hero_params(),
        Some(&HeroParams {
            primary_min: [0, 0, 1, 1],
            primary_max: 99,
            backpack: 64,
            learn_level: 2,
        })
    );
    // The hash sees presence and every field.
    let changes: [fn(&mut HeroParamsDef); 4] = [
        |d| d.primary_min[3] = 2,
        |d| d.primary_max = 98,
        |d| d.backpack = 0,
        |d| d.learn_level = 3,
    ];
    let hash = with.content_hash();
    assert_ne!(without.content_hash(), hash);
    for change in changes {
        let mut q = pack();
        let mut def = params();
        change(&mut def);
        q.hero_params = Some(def);
        assert_ne!(compile(&[q]).unwrap().content_hash(), hash);
    }
    // A minimum above the maximum.
    let mut q = pack();
    q.hero_params = Some(HeroParamsDef {
        primary_max: 0,
        ..params()
    });
    assert_eq!(
        fails(q),
        CompileError::InvalidHeroParams("primary_min must not exceed primary_max")
    );
    // A learnable level above the highest spell level; 5 is the highest allowed.
    let mut q = pack();
    q.hero_params = Some(HeroParamsDef {
        learn_level: 6,
        ..params()
    });
    assert_eq!(
        fails(q),
        CompileError::InvalidHeroParams("learn_level must be at most 5")
    );
    let mut q = pack();
    q.hero_params = Some(HeroParamsDef {
        learn_level: 5,
        ..params()
    });
    assert!(compile(&[q]).is_ok());
    // Provided twice in the closure.
    let mut ext = ContentPack::new(manifest("x", &["t"]));
    ext.hero_params = Some(params());
    assert_eq!(
        compile(&[p, ext]).unwrap_err(),
        CompileError::DuplicateHeroParams("x".into())
    );
}

// ---- E04b T8e: data quirks through gated overrides (ADR-0007, amendment E04b T8e) ----

const DATA_REGISTRY: crate::QuirkRegistry = crate::QuirkRegistry::new(&[
    "quirk.battle.x",
    "quirk.content.levels",
    "quirk.content.specialty",
]);

/// The test pack with a `faithful` profile that keeps both data quirks and a `fixed` one that
/// fixes them, and a pack `x` whose overrides fix them.
fn data_quirk_packs() -> [ContentPack; 2] {
    let mut t = pack();
    t.profiles = vec![
        ProfileDef {
            key: key("t:faithful"),
            quirks: vec![
                ("quirk.content.levels".into(), true),
                ("quirk.content.specialty".into(), true),
            ],
        },
        ProfileDef {
            key: key("t:fixed"),
            quirks: vec![("quirk.content.levels".into(), false)],
        },
    ];
    let mut x = ContentPack::new(manifest("x", &["t"]));
    x.creature_overrides.push(crate::CreatureOverride {
        target: key("t:farmer"),
        fixes: Some("quirk.content.levels".into()),
        patch: crate::CreaturePatch {
            level: Some(5),
            ..Default::default()
        },
    });
    x.hero_overrides.push(crate::HeroOverride {
        target: key("t:orrin"),
        fixes: Some("quirk.content.specialty".into()),
        patch: crate::HeroPatch {
            specialty: Some(vec![creature_stat("t:farmer")]),
        },
    });
    [t, x]
}

fn compile_data(packs: &[ContentPack], profile: &str) -> Result<Ruleset, CompileError> {
    Ruleset::compile_with(packs, profile, &DATA_REGISTRY)
}

#[test]
fn gated_overrides_apply_only_where_the_profile_fixes_the_quirk() {
    let packs = data_quirk_packs();
    let faithful = compile_data(&packs, "t:faithful").unwrap();
    let fixed = compile_data(&packs, "t:fixed").unwrap();
    let plain = compile_data(&packs[..1], "t:faithful").unwrap();
    let farmer = faithful.creature_by_key("t:farmer").unwrap();
    let orrin = faithful.hero_type_by_key("t:orrin").unwrap();
    // `faithful` keeps the original's values: the gated overrides change nothing.
    assert_eq!(faithful.creature_info(farmer), plain.creature_info(farmer));
    assert_eq!(faithful.hero_type(orrin), plain.hero_type(orrin));
    // `fixed` turns both flags off (the specialty's by leaving it out): both overrides apply.
    assert_eq!(fixed.creature_info(farmer).unwrap().level, 5);
    assert!(matches!(
        fixed.hero_type(orrin).unwrap().specialty.as_slice(),
        [HeroEffect::CreatureStat { .. }]
    ));
    assert_ne!(faithful.content_hash(), fixed.content_hash());
}

#[test]
fn a_hero_override_without_a_gate_applies_in_every_profile() {
    let [t, mut x] = data_quirk_packs();
    x.creature_overrides.clear();
    x.hero_overrides[0].fixes = None;
    let r = compile_data(&[t.clone(), x.clone()], "t:faithful").unwrap();
    let orrin = r.hero_type_by_key("t:orrin").unwrap();
    assert!(matches!(
        r.hero_type(orrin).unwrap().specialty.as_slice(),
        [HeroEffect::CreatureStat { .. }]
    ));
    // A patch without a specialty keeps the hero's.
    x.hero_overrides[0].patch.specialty = None;
    let r = compile_data(&[t.clone(), x], "t:faithful").unwrap();
    let plain = compile_data(&[t], "t:faithful").unwrap();
    assert_eq!(r.hero_type(orrin), plain.hero_type(orrin));
}

#[test]
fn gated_overrides_are_checked_in_every_profile() {
    let not_data = |flag: &str| CompileError::NotADataQuirk(flag.into());
    let cases: Vec<Case> = vec![
        // The gate must name a registered data quirk flag, not a behaviour flag.
        (
            |x| x.creature_overrides[0].fixes = Some("quirk.content.nope".into()),
            not_data("quirk.content.nope"),
        ),
        (
            |x| x.hero_overrides[0].fixes = Some("quirk.battle.x".into()),
            not_data("quirk.battle.x"),
        ),
        // A patch that does not apply in `faithful` is still resolved there.
        (
            |x| x.creature_overrides[0].patch.upgrades = Some(vec![key("t:nope")]),
            CompileError::UnknownReference {
                kind: "creature",
                key: "t:nope".into(),
            },
        ),
        (
            |x| x.hero_overrides[0].patch.specialty = Some(vec![creature_stat("t:nope")]),
            CompileError::UnknownReference {
                kind: "creature",
                key: "t:nope".into(),
            },
        ),
        (
            |x| x.hero_overrides[0].target = key("t:nobody"),
            CompileError::UnknownReference {
                kind: "hero type",
                key: "t:nobody".into(),
            },
        ),
        (
            |x| x.hero_overrides[0].target = key("y:orrin"),
            CompileError::ReferenceOutsideDependencies {
                key: "y:orrin".into(),
                pack: "x".into(),
            },
        ),
    ];
    for (change, err) in cases {
        let [t, mut x] = data_quirk_packs();
        change(&mut x);
        assert_eq!(compile_data(&[t, x], "t:faithful").unwrap_err(), err);
    }
}

#[test]
fn gated_and_ungated_overrides_apply_in_load_order() {
    let level = |packs: &[ContentPack], profile: &str| {
        let r = compile_data(packs, profile).unwrap();
        r.creature_info(r.creature_by_key("t:farmer").unwrap())
            .unwrap()
            .level
    };
    let later = |fixes: Option<&str>, level: u8| {
        let mut y = ContentPack::new(manifest("y", &["x"]));
        y.creature_overrides.push(crate::CreatureOverride {
            target: key("t:farmer"),
            fixes: fixes.map(Into::into),
            patch: crate::CreaturePatch {
                level: Some(level),
                ..Default::default()
            },
        });
        y
    };
    // x's gated level 5, then y's ungated 6: y's wins in both profiles.
    let [t, x] = data_quirk_packs();
    let packs = [t, x, later(None, 6)];
    assert_eq!(
        (level(&packs, "t:faithful"), level(&packs, "t:fixed")),
        (6, 6)
    );
    // x's ungated level 6, then y's gated 5: y's only where it applies.
    let [t, mut x] = data_quirk_packs();
    x.creature_overrides[0].fixes = None;
    x.creature_overrides[0].patch.level = Some(6);
    let packs = [t, x, later(Some("quirk.content.levels"), 5)];
    assert_eq!(
        (level(&packs, "t:faithful"), level(&packs, "t:fixed")),
        (6, 5)
    );
}
