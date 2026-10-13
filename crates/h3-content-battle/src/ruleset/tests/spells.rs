//! The spells' battle rows and the abilities that name spells (moved from the core's hero
//! content tests with the content split, R01).

use std::format;
use std::vec;

use h3_content::{
    HeroClassDef, HeroEffect, HeroOverride, HeroPatch, HeroTypeDef, ProfileDef, SecondarySkillDef,
    SpellDef, SpellSchoolDef, StartingStackDef,
};

use super::*;

fn school(k: &str) -> SpellSchoolDef {
    SpellSchoolDef {
        key: key(k),
        name: "S".into(),
        protection_rank: 0,
    }
}

fn spell(k: &str, schools: &[&str]) -> Spell {
    Spell {
        core: SpellDef {
            key: key(k),
            name: "Spell".into(),
            category: SpellCategory::Combat,
            level: 1,
            schools: schools.iter().map(|s| key(s)).collect(),
            cost: [5, 4, 4, 4],
            power: 10,
            effect: [10, 10, 20, 30],
            granted_by_name_only: false,
        },
        battle: BattleSpellDef {
            key: key(k),
            polarity: SpellPolarity::Negative,
            magic: SpellMagic {
                target: SpellTarget::Stack,
                effects: vec![SpellEffect::Damage],
                ..SpellMagic::default()
            },
            counters: vec![],
            dispels: vec![],
            spares: vec![],
            tests_as: None,
            summons: None,
        },
    }
}

/// A pack with the hero content the spells and abilities need: two schools, a hero spell and a
/// creature ability, a skill, a class and a hero with a starting army, and the creatures.
fn pack() -> Pack {
    let mut cart = creature("t:cart");
    cart.battle.damage_min = 0;
    cart.battle.damage_max = 0;
    cart.core.speed = 0;
    cart.core.info.flags = vec!["war_machine".into()];
    let mut p = Pack::new(manifest("t", &[])).creatures([
        creature("t:peasant"),
        creature("t:farmer"),
        cart,
    ]);
    let mut ability = spell("t:gaze", &[]);
    ability.core.category = SpellCategory::Ability;
    ability.core.level = 0;
    ability.battle.magic = SpellMagic::default();
    p.add_spells([spell("t:bolt", &["t:air", "t:fire"]), ability]);
    p.core.factions = vec![FactionDef {
        key: key("t:town"),
        name: "F".into(),
        native_terrain: None,
    }];
    p.core.spell_schools = vec![school("t:air"), school("t:fire")];
    p.core.skills = vec![SecondarySkillDef {
        key: key("t:tactics"),
        name: "Skill".into(),
        levels: [1, 2, 3].map(|level| vec![HeroEffect::Tactics { level }]),
    }];
    p.core.hero_classes = vec![HeroClassDef {
        key: key("t:knight"),
        name: "Class".into(),
        faction: key("t:town"),
        primary: [2, 2, 1, 1],
        primary_gain: [[35, 45, 10, 10], [30, 30, 20, 20]],
        skill_weights: vec![(key("t:tactics"), 3)],
    }];
    p.core.heroes = vec![HeroTypeDef {
        key: key("t:orrin"),
        name: "Hero".into(),
        class: key("t:knight"),
        skills: vec![(key("t:tactics"), 1)],
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
        specialty: vec![],
    }];
    p.core.profiles = vec![ProfileDef {
        key: key("t:faithful"),
        quirks: vec![],
    }];
    p.battle.battle = Some(battle_def());
    p
}

/// A change to the test pack and the error it causes.
type Case = (fn(&mut Pack), CompileError);

fn compile(packs: &[Pack]) -> Result<BattleRuleset, BattleCompileError> {
    super::compile(packs, "t:faithful")
}

fn fails(p: Pack) -> CompileError {
    content(compile(&[p]).unwrap_err())
}

fn invalid(kind: &'static str, k: &str, reason: &'static str) -> CompileError {
    CompileError::InvalidContent {
        kind,
        key: k.into(),
        reason,
    }
}

fn invalid_spell(k: &str, reason: &'static str) -> CompileError {
    invalid("spell", k, reason)
}

#[test]
fn polarity_names_round_trip() {
    for p in SpellPolarity::ALL {
        assert_eq!(SpellPolarity::from_name(p.name()), Some(p));
    }
    assert_eq!(SpellPolarity::from_name("evil"), None);
}

#[test]
fn spell_rows_compile_and_are_checked() {
    let r = compile(&[pack()]).unwrap();
    let bolt = r.spell_by_key("t:bolt").unwrap();
    let s = r.spell(bolt).unwrap();
    assert_eq!(s.polarity, SpellPolarity::Negative);
    assert_eq!((s.level, s.schools.len()), (1, 2));
    assert_eq!(r.spells().len(), 2);
    let cases: [Case; 3] = [
        (
            |p| p.core.spells[0].category = SpellCategory::Adventure,
            invalid_spell("t:bolt", "adventure spells have no battle effects"),
        ),
        (
            |p| p.battle.spells[1].magic.target = SpellTarget::Hex,
            invalid_spell("t:gaze", "targeting without effects"),
        ),
        (
            |p| {
                let stack = p.core.heroes[0].army[0].clone();
                p.core.heroes[0].army = vec![stack; 4];
            },
            invalid(
                "hero type",
                "t:orrin",
                "more starting stacks than army slots",
            ),
        ),
    ];
    for (change, err) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(fails(p), err);
    }
    // The battle magic is part of the part's hash.
    let mut q = pack();
    q.battle.spells[0].magic.target = SpellTarget::None;
    assert_ne!(compile(&[q]).unwrap().content_hash(), r.content_hash());
}
/// A timed spell `k` of the test pack's air school.
fn timed(k: &str) -> Spell {
    let mut s = spell(k, &["t:air"]);
    s.battle.magic.effects = vec![SpellEffect::Timed(TimedChange::Speed)];
    s
}

#[test]
fn counters_resolve_within_and_across_packs() {
    let mut p = pack();
    // `t:haste` names `t:slow`, which comes after it.
    let mut haste = timed("t:haste");
    haste.battle.counters = vec![key("t:slow")];
    let mut slow = timed("t:slow");
    slow.battle.counters = vec![key("t:haste")];
    p.add_spells([haste, slow]);
    let r = compile(&[p.clone()]).unwrap();
    let id = |k: &str| r.spell_by_key(k).unwrap();
    assert_eq!(r.spell(id("t:haste")).unwrap().counters, vec![id("t:slow")]);
    assert_eq!(r.spell(id("t:slow")).unwrap().counters, vec![id("t:haste")]);
    assert!(r.spell(id("t:bolt")).unwrap().counters.is_empty());
    let mut later = timed("u:rush");
    later.battle.counters = vec![key("t:slow")];
    let mut ext = Pack::new(manifest("u", &["t"]));
    ext.add_spells([later]);
    let r2 = compile(&[p.clone(), ext]).unwrap();
    let rush = r2.spell_by_key("u:rush").unwrap();
    assert_eq!(r2.spell(rush).unwrap().counters, vec![id("t:slow")]);
    let mut other = p.clone();
    other.battle.spells[3].counters.clear();
    assert_ne!(compile(&[other]).unwrap().content_hash(), r.content_hash());
}

#[test]
fn counters_are_checked() {
    let reason = "counters need a timed effect and may not name the spell itself";
    let mut p = pack();
    let mut haste = timed("t:haste");
    haste.battle.counters = vec![key("t:haste")];
    p.add_spells([haste]);
    assert_eq!(fails(p), invalid("spell", "t:haste", reason));
    let mut p = pack();
    p.battle.spells[0].counters = vec![key("t:gaze")];
    assert_eq!(fails(p), invalid("spell", "t:bolt", reason));
    let mut p = pack();
    p.battle.spells[0].counters = vec![key("t:nope")];
    assert_eq!(
        fails(p),
        CompileError::UnknownReference {
            kind: "spell",
            key: "t:nope".into()
        }
    );
    let mut p = pack();
    let mut haste = timed("t:haste");
    haste.battle.counters = vec![key("t:bolt"), key("t:bolt")];
    p.add_spells([haste]);
    assert_eq!(fails(p), CompileError::DuplicateKey("t:bolt".into()));
}

/// A dispel spell `k` of the test pack's air school.
fn dispel(k: &str) -> Spell {
    let mut s = spell(k, &["t:air"]);
    s.battle.magic.effects = vec![SpellEffect::Dispel];
    s
}

#[test]
fn dispel_lists_resolve_and_are_checked() {
    let mut p = pack();
    // `t:purge` spares `t:haste`, which comes after it; `t:mend` ends only `t:haste`.
    let mut purge = dispel("t:purge");
    purge.battle.spares = vec![key("t:haste")];
    let mut mend = dispel("t:mend");
    mend.battle.dispels = vec![key("t:haste")];
    p.add_spells([purge, mend, timed("t:haste")]);
    let r = compile(&[p.clone()]).unwrap();
    let id = |k: &str| r.spell_by_key(k).unwrap();
    assert_eq!(r.spell(id("t:purge")).unwrap().spares, vec![id("t:haste")]);
    assert_eq!(r.spell(id("t:mend")).unwrap().dispels, vec![id("t:haste")]);
    assert!(r.spell(id("t:purge")).unwrap().dispels.is_empty());
    for (i, field) in [(2, "spares"), (3, "dispels")] {
        let mut other = p.clone();
        if field == "spares" {
            other.battle.spells[i].spares.clear();
        } else {
            other.battle.spells[i].dispels.clear();
        }
        assert_ne!(compile(&[other]).unwrap().content_hash(), r.content_hash());
    }
    let reason = "dispels or spares (not both) need a dispel effect";
    let mut both = p.clone();
    both.battle.spells[2].dispels = vec![key("t:haste")];
    assert_eq!(fails(both), invalid("spell", "t:purge", reason));
    let mut timed_only = p.clone();
    timed_only.battle.spells[4].spares = vec![key("t:purge")];
    assert_eq!(fails(timed_only), invalid("spell", "t:haste", reason));
    let mut timed_only = p.clone();
    timed_only.battle.spells[4].dispels = vec![key("t:purge")];
    assert_eq!(fails(timed_only), invalid("spell", "t:haste", reason));
    let unknown = CompileError::UnknownReference {
        kind: "spell",
        key: "t:nope".into(),
    };
    let mut bad = p.clone();
    bad.battle.spells[2].spares = vec![key("t:nope")];
    assert_eq!(fails(bad), unknown);
    let mut bad = p;
    bad.battle.spells[3].dispels = vec![key("t:nope")];
    assert_eq!(fails(bad), unknown);
}

/// A stack spell of `effect` (v17: raise, sacrifice).
fn raising(k: &str, effect: SpellEffect) -> Spell {
    let mut s = spell(k, &["t:air"]);
    s.battle.magic.target = SpellTarget::Stack;
    s.battle.magic.effects = vec![effect];
    s
}

#[test]
fn a_sacrifice_names_the_raise_spell_it_is_tested_as() {
    let mut p = pack();
    // `t:offer` names `t:rise`, which comes after it.
    let mut offer = raising("t:offer", SpellEffect::Sacrifice);
    offer.battle.tests_as = Some(key("t:rise"));
    p.add_spells([offer, raising("t:rise", SpellEffect::Raise)]);
    let r = compile(&[p.clone()]).unwrap();
    let id = |k: &str| r.spell_by_key(k).unwrap();
    assert_eq!(r.spell(id("t:offer")).unwrap().tests_as, Some(id("t:rise")));
    assert_eq!(r.spell(id("t:rise")).unwrap().tests_as, None);
    let reason = "a sacrifice effect needs tests_as, a spell with a raise effect";
    let first = p.battle.spells.len() - 2;
    let mut missing = p.clone();
    missing.battle.spells[first].tests_as = None;
    assert_eq!(fails(missing), invalid("spell", "t:offer", reason));
    let mut not_a_raise = p.clone();
    not_a_raise.battle.spells[first].tests_as = Some(key("t:offer"));
    assert_eq!(fails(not_a_raise), invalid("spell", "t:offer", reason));
    let mut no_sacrifice = p.clone();
    no_sacrifice.battle.spells[first + 1].tests_as = Some(key("t:rise"));
    assert_eq!(fails(no_sacrifice), invalid("spell", "t:rise", reason));
    let mut unknown = p.clone();
    unknown.battle.spells[first].tests_as = Some(key("t:nope"));
    assert_eq!(
        fails(unknown),
        CompileError::UnknownReference {
            kind: "spell",
            key: "t:nope".into(),
        }
    );
    let mut other = p;
    other.battle.spells[first].tests_as = None;
    other.battle.spells[first].magic.effects = vec![SpellEffect::Raise];
    assert_ne!(compile(&[other]).unwrap().content_hash(), r.content_hash());
}

#[test]
fn strike_abilities_name_spells_that_do_what_they_need() {
    let mut p = pack();
    let mut unbless = dispel("t:unbless");
    unbless.battle.magic.dispels_positive = true;
    p.add_spells([timed("t:haste"), unbless]);
    let after = |spell: &str| AbilityDef::SpellAfterStrike {
        spell: key(spell),
        chance_percent: 20,
        mastery: 0,
        rounds: 3,
        living: false,
        unless_active: false,
        defended: false,
    };
    let damage = |spell: &str| AbilityDef::SpellDamageAfterStrike {
        spell: key(spell),
        chance_percent: 20,
        per_creature: 10,
    };
    p.battle.creatures[0].abilities = vec![after("t:haste"), after("t:unbless"), damage("t:bolt")];
    let r = compile(&[p.clone()]).unwrap();
    let unbless = r.spell_by_key("t:unbless").unwrap();
    assert!(r.spell(unbless).unwrap().magic.dispels_positive);
    let invalid = |reason| CompileError::InvalidCreature {
        key: "t:peasant".into(),
        reason,
    };
    for (ability, reason) in [
        (
            after("t:bolt"),
            "spell_after_strike needs a spell with timed or dispel effects only",
        ),
        (
            after("t:gaze"),
            "spell_after_strike needs a spell with timed or dispel effects only",
        ),
        (
            damage("t:haste"),
            "spell_damage_after_strike needs a spell with a damage effect",
        ),
    ] {
        let mut bad = p.clone();
        bad.battle.creatures[0].abilities = vec![ability];
        assert_eq!(fails(bad), invalid(reason));
    }
    let mut no_dispel = p;
    no_dispel.battle.spells[2].magic.dispels_positive = true;
    assert_eq!(
        fails(no_dispel),
        invalid_spell("t:haste", "dispels_positive needs a dispel effect")
    );
}

#[test]
fn a_casters_spell_takes_a_target_stack_at_its_mastery() {
    let mut p = pack();
    // Haste: one stack below Expert, all of them from Expert (its mass version); with quirk
    // Q-0011's earlier mass version from Advanced (`mass_from`, `fixed_mass_from` without it).
    let mut haste = timed("t:haste");
    haste.battle.magic.mass_from = Some(2);
    haste.battle.magic.fixed_mass_from = Some(3);
    let mut charm = timed("t:charm");
    charm.battle.magic.health_limit = true;
    let mut wall = timed("t:wall");
    wall.battle.magic.target = SpellTarget::Hex;
    p.add_spells([haste, charm, wall, raising("t:rise", SpellEffect::Raise)]);
    let caster = |spell: &str, mastery| AbilityDef::SpellCaster {
        spell: key(spell),
        mastery,
        power: 6,
    };
    for ok in [caster("t:bolt", 3), caster("t:haste", 1)] {
        let mut good = p.clone();
        good.battle.creatures[0].abilities = vec![ok];
        let r = compile(&[good]).unwrap();
        let peasant = r.creature_by_key("t:peasant").unwrap();
        assert!(matches!(
            r.abilities(peasant),
            [Ability::SpellCaster { power: 6, .. }]
        ));
    }
    let reason = "spell_caster needs a spell with damage, timed, dispel or heal effects and no \
                  health limit that takes a target stack at its mastery";
    for bad_ability in [
        caster("t:gaze", 0),
        caster("t:haste", 2),
        caster("t:haste", 3),
        caster("t:charm", 0),
        caster("t:wall", 0),
        caster("t:rise", 0),
    ] {
        let mut bad = p.clone();
        bad.battle.creatures[0].abilities = vec![bad_ability];
        assert_eq!(
            fails(bad),
            CompileError::InvalidCreature {
                key: "t:peasant".into(),
                reason,
            }
        );
    }
}

#[test]
fn a_random_casters_drawn_spells_take_a_stack_or_a_hex_at_its_mastery() {
    let mut p = pack();
    let mut haste = timed("t:haste");
    haste.battle.magic.mass_from = Some(3);
    let mut charm = timed("t:charm");
    charm.battle.magic.health_limit = true;
    let mut wall = timed("t:wall");
    wall.battle.magic.target = SpellTarget::Hex;
    // An Armageddon: damage without a target.
    let mut doom = spell("t:doom", &["t:air"]);
    doom.battle.magic.target = SpellTarget::None;
    p.add_spells([
        haste,
        charm,
        wall,
        raising("t:rise", SpellEffect::Raise),
        doom,
    ]);
    let random = |mastery| AbilityDef::RandomSpellCaster {
        mastery,
        power_per_creature: 5,
    };
    let drawn = |spell: &str| AbilityDef::DrawnSpell {
        spell: key(spell),
        weight: 10,
    };
    // A stack spell, a hex spell and one whose mass version starts above the caster's mastery,
    // listed before or after their caster (BAT-MAG-049).
    for ok in [
        vec![
            random(2),
            drawn("t:bolt"),
            drawn("t:wall"),
            drawn("t:haste"),
        ],
        vec![drawn("t:haste"), random(2)],
    ] {
        let mut good = p.clone();
        good.battle.creatures[0].abilities = ok;
        let r = compile(&[good]).unwrap();
        let peasant = r.creature_by_key("t:peasant").unwrap();
        assert!(
            r.abilities(peasant)
                .iter()
                .any(|a| matches!(a, Ability::DrawnSpell { weight: 10, .. }))
        );
    }
    let reason = "drawn_spell needs a spell with damage, timed, dispel or heal effects and no \
                  health limit or obstacle that takes a target stack or hex (for a helpful \
                  caster a stack; for a periodic caster no target, and no damage) at its \
                  caster's mastery";
    let helpful = AbilityDef::HelpfulSpellCaster {
        mastery: 2,
        power: 6,
    };
    let mut good = p.clone();
    good.battle.creatures[0].abilities = vec![drawn("t:haste"), helpful.clone()];
    assert!(compile(&[good]).is_ok());
    // A periodic caster's spells take no target at its mastery: a mass version (BAT-MAG-053).
    let periodic = |mastery| AbilityDef::PeriodicSpellCaster {
        rounds: 3,
        mastery,
        power: 3,
    };
    let mut good = p.clone();
    good.battle.creatures[0].abilities = vec![periodic(3), drawn("t:haste")];
    let r = compile(&[good]).unwrap();
    let peasant = r.creature_by_key("t:peasant").unwrap();
    assert!(matches!(
        r.abilities(peasant),
        [Ability::PeriodicSpellCaster { rounds: 3, .. }, ..]
    ));
    for bad_abilities in [
        vec![random(2), drawn("t:gaze")],
        vec![random(2), drawn("t:charm")],
        vec![random(2), drawn("t:rise")],
        vec![random(3), drawn("t:haste")],
        vec![helpful, drawn("t:wall")],
        vec![periodic(2), drawn("t:haste")],
        vec![periodic(3), drawn("t:bolt")],
        vec![periodic(3), drawn("t:doom")],
    ] {
        let mut bad = p.clone();
        bad.battle.creatures[0].abilities = bad_abilities;
        assert_eq!(
            fails(bad),
            CompileError::InvalidCreature {
                key: "t:peasant".into(),
                reason,
            }
        );
    }
}

#[test]
fn a_corpse_raiser_raises_one_one_hex_creature() {
    let raiser = |creature: &str| AbilityDef::CorpseRaiser {
        creature: key(creature),
        hit_points_per_creature: 50,
    };
    let mut p = pack();
    p.battle.creatures[0].abilities = vec![raiser("t:farmer")];
    p.battle.creatures[1].abilities = vec![raiser("t:farmer")];
    let r = compile(&[p.clone()]).unwrap();
    let peasant = r.creature_by_key("t:peasant").unwrap();
    assert!(matches!(
        r.abilities(peasant),
        [Ability::CorpseRaiser {
            hit_points_per_creature: 50,
            ..
        }]
    ));
    // At most one creature raised from corpses in a ruleset (BAT-MAG-052).
    let mut two = p.clone();
    two.battle.creatures[1].abilities = vec![raiser("t:peasant")];
    assert_eq!(
        fails(two),
        CompileError::TooManyEntries("corpse_raiser creature")
    );
    let mut wide = p;
    wide.core.creatures[1].info.flags.push("two_hex".into());
    assert_eq!(
        fails(wide),
        CompileError::InvalidCreature {
            key: "t:peasant".into(),
            reason: "corpse_raiser needs a one-hex creature",
        }
    );
}

#[test]
fn a_stack_raiser_needs_a_raise_spell() {
    let mut p = pack();
    let mut offer = raising("t:offer", SpellEffect::Sacrifice);
    offer.battle.tests_as = Some(key("t:rise"));
    p.add_spells([raising("t:rise", SpellEffect::Raise), offer]);
    let raiser = |spell: &str| AbilityDef::StackRaiser {
        spell: key(spell),
        hit_points_per_creature: 100,
    };
    let mut good = p.clone();
    good.battle.creatures[0].abilities = vec![raiser("t:rise")];
    let r = compile(&[good]).unwrap();
    let peasant = r.creature_by_key("t:peasant").unwrap();
    assert!(matches!(
        r.abilities(peasant),
        [Ability::StackRaiser {
            hit_points_per_creature: 100,
            ..
        }]
    ));
    for spell in ["t:offer", "t:bolt"] {
        let mut bad = p.clone();
        bad.battle.creatures[0].abilities = vec![raiser(spell)];
        assert_eq!(
            fails(bad),
            CompileError::InvalidCreature {
                key: "t:peasant".into(),
                reason: "stack_raiser needs a spell with a raise effect",
            }
        );
    }
}

#[test]
fn an_innate_spell_is_a_fire_shield_or_a_magic_mirror() {
    let mut p = pack();
    let with = |k: &str, change| {
        let mut s = timed(k);
        s.battle.magic.effects = vec![SpellEffect::Timed(change)];
        s
    };
    p.add_spells([
        with("t:shield", TimedChange::FireShield),
        with("t:mirror", TimedChange::MagicMirror),
        timed("t:haste"),
    ]);
    let innate = |spell: &str| AbilityDef::InnateSpell {
        spell: key(spell),
        value: 20,
    };
    let mut good = p.clone();
    good.battle.creatures[0].abilities = vec![innate("t:shield"), innate("t:mirror")];
    let r = compile(&[good]).unwrap();
    let peasant = r.creature_by_key("t:peasant").unwrap();
    let (shield, mirror) = (
        r.spell_by_key("t:shield").unwrap(),
        r.spell_by_key("t:mirror").unwrap(),
    );
    assert_eq!(
        r.abilities(peasant),
        [
            Ability::InnateSpell {
                spell: shield,
                value: 20
            },
            Ability::InnateSpell {
                spell: mirror,
                value: 20
            }
        ]
    );
    for spell in ["t:haste", "t:bolt"] {
        let mut bad = p.clone();
        bad.battle.creatures[0].abilities = vec![innate(spell)];
        assert_eq!(
            fails(bad),
            CompileError::InvalidCreature {
                key: "t:peasant".into(),
                reason: "innate_spell needs a spell with a timed fire_shield or magic_mirror \
                         change",
            }
        );
    }
}

#[test]
fn a_summon_spell_names_its_creature() {
    let mut p = pack();
    let mut conjure = spell("t:conjure", &["t:air"]);
    conjure.battle.magic = SpellMagic {
        effects: vec![SpellEffect::Summon],
        ..SpellMagic::default()
    };
    conjure.battle.summons = Some(key("t:farmer"));
    p.add_spells([conjure]);
    let r = compile(&[p.clone()]).unwrap();
    let id = r.spell_by_key("t:conjure").unwrap();
    let farmer = r.creature_by_key("t:farmer").unwrap();
    assert_eq!(r.spell(id).unwrap().summons, Some(farmer));
    let reason = "a summon effect and the creature it summons go together";
    let last = p.battle.spells.len() - 1;
    let mut missing = p.clone();
    missing.battle.spells[last].summons = None;
    assert_eq!(fails(missing), invalid("spell", "t:conjure", reason));
    let mut no_summon = p.clone();
    no_summon.battle.spells[0].summons = Some(key("t:farmer"));
    let first = no_summon.core.spells[0].key.clone();
    assert_eq!(fails(no_summon), invalid("spell", first.as_str(), reason));
    let mut unknown = p.clone();
    unknown.battle.spells[last].summons = Some(key("t:nope"));
    assert_eq!(
        fails(unknown),
        CompileError::UnknownReference {
            kind: "creature",
            key: "t:nope".into(),
        }
    );
    let mut other = p.clone();
    other.battle.spells[last].summons = Some(key("t:peasant"));
    assert_ne!(compile(&[other]).unwrap().content_hash(), r.content_hash());
    // At most `MAX_SUMMON_SPELLS` (4) summon spells.
    let mut many = p;
    for i in 0..MAX_SUMMON_SPELLS {
        let core = many.core.spells[last].clone();
        let battle = many.battle.spells[last].clone();
        let mut extra = Spell { core, battle };
        extra.core.key = key(&format!("t:conjure{i}"));
        extra.battle.key = extra.core.key.clone();
        many.add_spells([extra]);
    }
    assert_eq!(fails(many), CompileError::TooManyEntries("summon spell"));
}

// ---- E04b T8e: the battle half of a gated creature override (ADR-0007, amendment E04b T8e) ----

const DATA_REGISTRY: QuirkRegistry = QuirkRegistry::new(&[
    "quirk.battle.x",
    "quirk.content.levels",
    "quirk.content.specialty",
]);

/// The test pack with a `faithful` profile that keeps the data quirk and a `fixed` one that fixes
/// it, and a pack `x` whose creature override (both halves) and hero override fix them.
fn data_quirk_packs() -> [Pack; 2] {
    let mut t = pack();
    t.core.profiles = vec![
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
    let mut x = Pack::new(manifest("x", &["t"]));
    x.add_override("t:farmer", Some("quirk.content.levels"));
    x.core.creature_overrides[0].patch.level = Some(5);
    x.battle.creature_overrides[0].patch = BattleCreaturePatch {
        attack: Some(9),
        abilities: Some(vec![AbilityDef::Hate {
            target: key("t:peasant"),
            percent: 50,
        }]),
        ..BattleCreaturePatch::default()
    };
    x.core.hero_overrides.push(HeroOverride {
        target: key("t:orrin"),
        fixes: Some("quirk.content.specialty".into()),
        patch: HeroPatch { specialty: None },
    });
    [t, x]
}

fn compile_data(packs: &[Pack], profile: &str) -> Result<BattleRuleset, BattleCompileError> {
    compile_with(packs, profile, &DATA_REGISTRY)
}

#[test]
fn gated_overrides_apply_only_where_the_profile_fixes_the_quirk() {
    let packs = data_quirk_packs();
    let faithful = compile_data(&packs, "t:faithful").unwrap();
    let fixed = compile_data(&packs, "t:fixed").unwrap();
    let plain = compile_data(&packs[..1], "t:faithful").unwrap();
    let farmer = faithful.creature_by_key("t:farmer").unwrap();
    // `faithful` keeps the original's values: the gated overrides change nothing.
    assert_eq!(faithful.creature(farmer), plain.creature(farmer));
    assert_eq!(faithful.abilities(farmer), plain.abilities(farmer));
    // `fixed` turns the flag off: both halves apply.
    assert_eq!(fixed.creature_info(farmer).unwrap().level, 5);
    assert_eq!(fixed.creature(farmer).unwrap().attack, 9);
    assert_eq!(fixed.abilities(farmer).len(), 1);
    assert_ne!(faithful.content_hash(), fixed.content_hash());
}

#[test]
fn gated_overrides_are_checked_in_every_profile() {
    let cases: [Case; 2] = [
        // The gate must name a registered data quirk flag, not a behaviour flag.
        (
            |x| x.battle.creature_overrides[0].fixes = Some("quirk.battle.x".into()),
            CompileError::NotADataQuirk("quirk.battle.x".into()),
        ),
        // A patch that does not apply in `faithful` is still resolved there.
        (
            |x| {
                x.battle.creature_overrides[0].patch.abilities = Some(vec![AbilityDef::Hate {
                    target: key("t:nope"),
                    percent: 50,
                }]);
            },
            CompileError::UnknownReference {
                kind: "creature",
                key: "t:nope".into(),
            },
        ),
    ];
    for (change, err) in cases {
        let [t, mut x] = data_quirk_packs();
        change(&mut x);
        assert_eq!(
            content(compile_data(&[t, x], "t:faithful").unwrap_err()),
            err
        );
    }
}

#[test]
fn a_school_immunity_names_a_school_of_the_closure() {
    let immunity = |school: &str| AbilityDef::SchoolImmunity {
        school: key(school),
    };
    let mut p = pack();
    p.battle.creatures[0].abilities = vec![immunity("t:fire")];
    let r = compile(&[p.clone()]).unwrap();
    let peasant = r.creature_by_key("t:peasant").unwrap();
    assert_eq!(
        r.abilities(peasant),
        [Ability::SchoolImmunity {
            school: SchoolId::from_raw(1)
        }]
    );
    p.battle.creatures[0].abilities = vec![immunity("t:water")];
    assert_eq!(
        fails(p),
        CompileError::UnknownReference {
            kind: "spell school",
            key: "t:water".into()
        }
    );
}
