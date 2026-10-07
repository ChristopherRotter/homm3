extern crate std;

use std::vec;
use std::vec::Vec;

use super::*;
use crate::effect::{Affects, CreatureSelector, CreatureStat, HeroEffect, SpellScope};
use crate::pack::{
    HeroClassDef, HeroParamsDef, HeroTypeDef, ProfileDef, SecondarySkillDef, SpellDef,
    SpellSchoolDef, StartingStackDef,
};
use crate::ruleset::Ruleset;
use crate::ruleset::tests::{battle_def, creature, faction, key, manifest};
use crate::spell::{SpellEffect, SpellTarget, TimedChange};

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
        polarity: SpellPolarity::Negative,
        magic: SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Damage],
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
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
    cart.damage_min = 0;
    cart.damage_max = 0;
    cart.speed = 0;
    cart.info.flags = vec!["war_machine".into()];
    let mut peasant = creature("t:peasant");
    // A forward reference: `t:farmer` is defined after `t:peasant`.
    peasant.info.upgrades = vec![key("t:farmer")];
    let mut ability = spell("t:gaze", &[]);
    ability.category = SpellCategory::Ability;
    ability.level = 0;
    ability.magic = SpellMagic::default();
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
        battle: Some(battle_def()),
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
    for p in SpellPolarity::ALL {
        assert_eq!(SpellPolarity::from_name(p.name()), Some(p));
    }
    assert_eq!(SpellCategory::from_name("hero"), None);
    assert_eq!(SpellPolarity::from_name("evil"), None);
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
    assert_eq!(s.polarity, SpellPolarity::Negative);
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

/// A timed spell `k` of the test pack's air school.
fn timed(k: &str) -> SpellDef {
    let mut s = spell(k, &["t:air"]);
    s.magic.effects = vec![SpellEffect::Timed(TimedChange::Speed)];
    s
}

#[test]
fn counters_resolve_within_and_across_packs() {
    let mut p = pack();
    // `t:haste` names `t:slow`, which comes after it.
    let mut haste = timed("t:haste");
    haste.counters = vec![key("t:slow")];
    let mut slow = timed("t:slow");
    slow.counters = vec![key("t:haste")];
    p.spells.extend([haste, slow]);
    let r = compile(&[p.clone()]).unwrap();
    let id = |k: &str| r.spell_by_key(k).unwrap();
    assert_eq!(r.spell(id("t:haste")).unwrap().counters, vec![id("t:slow")]);
    assert_eq!(r.spell(id("t:slow")).unwrap().counters, vec![id("t:haste")]);
    assert!(r.spell(id("t:bolt")).unwrap().counters.is_empty());
    let mut later = timed("u:rush");
    later.counters = vec![key("t:slow")];
    let ext = ContentPack {
        spells: vec![later],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let r2 = compile(&[p.clone(), ext]).unwrap();
    let rush = r2.spell_by_key("u:rush").unwrap();
    assert_eq!(r2.spell(rush).unwrap().counters, vec![id("t:slow")]);
    let mut other = p.clone();
    other.spells[3].counters.clear();
    assert_ne!(compile(&[other]).unwrap().content_hash(), r.content_hash());
}

#[test]
fn counters_are_checked() {
    let reason = "counters need a timed effect and may not name the spell itself";
    let mut p = pack();
    let mut haste = timed("t:haste");
    haste.counters = vec![key("t:haste")];
    p.spells.push(haste);
    assert_eq!(fails(p), invalid("spell", "t:haste", reason));
    let mut p = pack();
    p.spells[0].counters = vec![key("t:gaze")];
    assert_eq!(fails(p), invalid("spell", "t:bolt", reason));
    let mut p = pack();
    p.spells[0].counters = vec![key("t:nope")];
    assert_eq!(
        fails(p),
        CompileError::UnknownReference {
            kind: "spell",
            key: "t:nope".into()
        }
    );
    let mut p = pack();
    let mut haste = timed("t:haste");
    haste.counters = vec![key("t:bolt"), key("t:bolt")];
    p.spells.push(haste);
    assert_eq!(fails(p), CompileError::DuplicateKey("t:bolt".into()));
}

/// A dispel spell `k` of the test pack's air school.
fn dispel(k: &str) -> SpellDef {
    let mut s = spell(k, &["t:air"]);
    s.magic.effects = vec![SpellEffect::Dispel];
    s
}

#[test]
fn dispel_lists_resolve_and_are_checked() {
    let mut p = pack();
    // `t:purge` spares `t:haste`, which comes after it; `t:mend` ends only `t:haste`.
    let mut purge = dispel("t:purge");
    purge.spares = vec![key("t:haste")];
    let mut mend = dispel("t:mend");
    mend.dispels = vec![key("t:haste")];
    p.spells.extend([purge, mend, timed("t:haste")]);
    let r = compile(&[p.clone()]).unwrap();
    let id = |k: &str| r.spell_by_key(k).unwrap();
    assert_eq!(r.spell(id("t:purge")).unwrap().spares, vec![id("t:haste")]);
    assert_eq!(r.spell(id("t:mend")).unwrap().dispels, vec![id("t:haste")]);
    assert!(r.spell(id("t:purge")).unwrap().dispels.is_empty());
    for (i, field) in [(2, "spares"), (3, "dispels")] {
        let mut other = p.clone();
        if field == "spares" {
            other.spells[i].spares.clear();
        } else {
            other.spells[i].dispels.clear();
        }
        assert_ne!(compile(&[other]).unwrap().content_hash(), r.content_hash());
    }
    let reason = "dispels or spares (not both) need a dispel effect";
    let mut both = p.clone();
    both.spells[2].dispels = vec![key("t:haste")];
    assert_eq!(fails(both), invalid("spell", "t:purge", reason));
    let mut timed_only = p.clone();
    timed_only.spells[4].spares = vec![key("t:purge")];
    assert_eq!(fails(timed_only), invalid("spell", "t:haste", reason));
    let mut timed_only = p.clone();
    timed_only.spells[4].dispels = vec![key("t:purge")];
    assert_eq!(fails(timed_only), invalid("spell", "t:haste", reason));
    let unknown = CompileError::UnknownReference {
        kind: "spell",
        key: "t:nope".into(),
    };
    let mut bad = p.clone();
    bad.spells[2].spares = vec![key("t:nope")];
    assert_eq!(fails(bad), unknown);
    let mut bad = p;
    bad.spells[3].dispels = vec![key("t:nope")];
    assert_eq!(fails(bad), unknown);
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
    assert_eq!(r.skills()[2].effects(3).len(), 2);
    assert_eq!(r.hero_types()[0].specialty.len(), 4);
    let fire = r.spell_schools().len() - 1;
    assert_eq!(
        r.skills()[2].effects(2)[2],
        HeroEffect::SchoolMastery {
            school: Handle::from_raw(u16::try_from(fire).unwrap()),
            level: 2
        }
    );

    // Every kind of entry contributes to the hash.
    let changes: [fn(&mut ContentPack); 7] = [
        |p| p.spell_schools[0].name = "X".into(),
        |p| p.spells[0].effect[3] = 31,
        |p| p.spells[0].magic.target = SpellTarget::None,
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
            |p| p.spells[0].category = SpellCategory::Adventure,
            invalid("spell", "t:bolt", "adventure spells have no battle effects"),
        ),
        (
            |p| p.spells[1].magic.target = SpellTarget::Hex,
            invalid("spell", "t:gaze", "targeting without effects"),
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
            |p| {
                let stack = p.heroes[0].army[0].clone();
                p.heroes[0].army = vec![stack; 4];
            },
            invalid(
                "hero type",
                "t:orrin",
                "more starting stacks than army slots",
            ),
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
        target: key("t:cart"),
        patch: crate::CreaturePatch {
            flags: Some(vec![]),
            damage_min: Some(1),
            damage_max: Some(1),
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
        })
    );
    // The hash sees presence and every field.
    let changes: [fn(&mut HeroParamsDef); 3] = [
        |d| d.primary_min[3] = 2,
        |d| d.primary_max = 98,
        |d| d.backpack = 0,
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
    // Provided twice in the closure.
    let mut ext = ContentPack::new(manifest("x", &["t"]));
    ext.hero_params = Some(params());
    assert_eq!(
        compile(&[p, ext]).unwrap_err(),
        CompileError::DuplicateHeroParams("x".into())
    );
}
