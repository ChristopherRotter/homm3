//! The heroes' spell defences (E04b T8b2): the spells the stacks a hero controls are immune to
//! (BAT-MAG-063: the Pendants, the Sphere of Permanence, Armageddon's Blade), the levels
//! (BAT-MAG-064: Power of the Dragon Father), mind immunity (BAT-MAG-065: Badge of Courage) and
//! the negation of immunities and resistance (BAT-MAG-066: the Orb of Vulnerability), after the
//! original's code (`docs/formats/installation.md` §7.23).

use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSide};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::magic::{SpellBonus, affect_chance};
use crate::{
    BattleState, CastTargets, Command, CommandError, Event, EventKind, StackId, cast_targets,
};

/// A mage with `spells` and the effects `effects`.
fn wearing(r: &Ruleset, spells: &[&str], effects: &[BattleEffect]) -> Option<BattleHero> {
    let mut h = mage(r, spells);
    h.magic.mana = 100;
    h.effects = effects.to_vec();
    Some(h)
}

/// The stacks `side`'s hero may cast `name` at now, by id.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide, name: &str) -> Vec<u8> {
    let mut out = CastTargets::new();
    cast_targets(s, r, side, spell(r, name), &mut out);
    out.iter()
        .filter_map(|t| t.stack().map(|id| id.0))
        .collect()
}

/// Gives stack `id` the timed effect of `name` at Basic for five rounds.
fn give(s: &mut BattleState, r: &Ruleset, id: u8, name: &str) {
    let spell_id = spell(r, name);
    let cast = TimedCast {
        id: spell_id,
        spell: r.spell(spell_id).unwrap(),
        mastery: 1,
        rounds: 5,
        own: SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The spells whose effects stack `id` has.
fn on(s: &BattleState, id: u8) -> Vec<h3_content::SpellId> {
    s.effects()
        .iter()
        .filter(|e| e.stack == StackId(id))
        .map(|e| e.spell)
        .collect()
}

/// The strike spells `events` report, by spell.
fn strike_spells(events: &[Event]) -> Vec<h3_content::SpellId> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::StrikeSpell { spell, .. } => Some(spell),
            _ => None,
        })
        .collect()
}

/// The active stack attacks `target` from `(x, y)`.
fn hit(s: &mut BattleState, r: &Ruleset, target: u8, (x, y): (i32, i32)) -> Vec<Event> {
    act(
        s,
        r,
        Command::Attack {
            target: StackId(target),
            from: at(r, x, y),
        },
    )
    .unwrap()
}

// ---------------------------------------------------------------- BAT-MAG-063

// covers: BAT-MAG-063
#[test]
fn a_heros_spell_immunity_protects_the_stacks_it_controls() {
    let r = ruleset();
    let bolt = spell(&r, "bolt");
    let battle = |defender: Option<BattleHero>| {
        arena(
            &r,
            [Some(mage(&r, &["bolt"])), defender],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 12, 3), ("fixed", 5, 12, 7)],
            1,
        )
    };
    assert_eq!(
        targets(&battle(None), &r, BattleSide::Attacker, "bolt"),
        [1, 2]
    );
    let immune = || wearing(&r, &[], &[BattleEffect::SpellImmunity { spell: bolt }]);
    let mut s = battle(immune());
    assert!(targets(&s, &r, BattleSide::Attacker, "bolt").is_empty());
    let before = s.clone();
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", stack(1))),
        Err(CommandError::InvalidSpellTarget)
    );
    assert_eq!(s, before);
    // A hypnotized stack is its controller's: the immunity of its own hero no longer protects it.
    give(&mut s, &r, 2, "hypnotize");
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "bolt"), [2]);
}

// covers: BAT-MAG-063, BAT-MAG-016, BAT-MAG-042
#[test]
fn a_heros_immunity_to_dispel_keeps_every_dispel_off_its_stacks() {
    let r = ruleset();
    let dispel = spell(&r, "dispel");
    let unbless = spell(&r, "unbless");
    // Its own hero's Dispel too (the original tests the stack's hero, 0x5A8138).
    let battle = |effects: &[BattleEffect]| {
        let mut s = arena(
            &r,
            [None, wearing(&r, &["dispel"], effects)],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 12, 5)],
            1,
        );
        give(&mut s, &r, 1, "curse");
        activate(&mut s, 1);
        s
    };
    assert_eq!(
        targets(&battle(&[]), &r, BattleSide::Defender, "dispel"),
        [1]
    );
    let sphere = [
        BattleEffect::SpellImmunity { spell: dispel },
        BattleEffect::SpellImmunity { spell: unbless },
    ];
    assert!(targets(&battle(&sphere), &r, BattleSide::Defender, "dispel").is_empty());
    // The strike's dispel of positive spells too: the haste stays, the weakening lands.
    let strike = |effects: &[BattleEffect]| {
        let mut s = arena(
            &r,
            [None, wearing(&r, &[], effects)],
            &[("fly", 5, 7, 5)],
            &[("fixed", 10, 8, 5)],
            1,
        );
        give(&mut s, &r, 1, "haste");
        let events = hit(&mut s, &r, 1, (7, 5));
        (strike_spells(&events), on(&s, 1))
    };
    let (cast, kept) = strike(&[]);
    assert!(cast.contains(&unbless) && !kept.contains(&spell(&r, "haste")));
    let (cast, kept) = strike(&sphere);
    assert_eq!(cast, [spell(&r, "weaken")]);
    assert!(kept.contains(&spell(&r, "haste")));
}

// ---------------------------------------------------------------- BAT-MAG-064

// covers: BAT-MAG-064
#[test]
fn a_heros_level_immunity_spares_its_stacks_from_every_spell_up_to_it_but_dispel() {
    let r = ruleset();
    let father = [BattleEffect::SpellLevelImmunity { max_level: 4 }];
    // The enemy's bolt (1) has no target; its own hero's Bless (1) none either; its Dispel does.
    let mut s = arena(
        &r,
        [
            Some(mage(&r, &["bolt"])),
            wearing(&r, &["bless", "dispel"], &father),
        ],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    assert!(targets(&s, &r, BattleSide::Attacker, "bolt").is_empty());
    give(&mut s, &r, 1, "curse");
    activate(&mut s, 1);
    assert!(targets(&s, &r, BattleSide::Defender, "bless").is_empty());
    assert_eq!(targets(&s, &r, BattleSide::Defender, "dispel"), [1]);
    // The strike's abilities: Disease (2) does nothing, Age (5) lands.
    let struck = |striker: &'static str| {
        let mut s = arena(
            &r,
            [None, wearing(&r, &[], &father)],
            &[(striker, 5, 7, 5)],
            &[("serf", 10, 8, 5)],
            1,
        );
        strike_spells(&hit(&mut s, &r, 1, (7, 5)))
    };
    assert!(struck("rotter").is_empty());
    assert_eq!(struck("ancient"), [spell(&r, "wither")]);
}

// ---------------------------------------------------------------- BAT-MAG-065

// covers: BAT-MAG-065
#[test]
fn a_heros_mind_immunity_spares_its_stacks_from_mind_spells_unless_negated() {
    let r = ruleset();
    let battle = |attacker: &[BattleEffect], defender: &[BattleEffect]| {
        arena(
            &r,
            [
                wearing(&r, &["bolt", "blind", "hypnotize"], attacker),
                wearing(&r, &[], defender),
            ],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 1, 12, 5)],
            1,
        )
    };
    let badge = [BattleEffect::MindImmunity];
    let s = battle(&[], &[]);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "blind"), [1]);
    let s = battle(&[], &badge);
    assert!(targets(&s, &r, BattleSide::Attacker, "blind").is_empty());
    assert!(targets(&s, &r, BattleSide::Attacker, "hypnotize").is_empty());
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "bolt"), [1]);
    let s = battle(&[BattleEffect::NegateImmunities], &badge);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "blind"), [1]);
}

// ---------------------------------------------------------------- BAT-MAG-066

// covers: BAT-MAG-066, BAT-MAG-005, BAT-MAG-008
#[test]
fn negated_immunities_lift_the_creatures_immunities_and_all_resistance() {
    let r = ruleset();
    let orb = [BattleEffect::NegateImmunities];
    let ward = [BattleEffect::MagicResistance {
        rate: Rate::bp(2000),
    }];
    let battle = |attacker: &[BattleEffect], defender: &[BattleEffect]| {
        arena(
            &r,
            [
                wearing(&r, &["bolt", "blind"], attacker),
                wearing(&r, &[], defender),
            ],
            &[("fixed", 5, 2, 5)],
            &[
                ("wyrm", 1, 12, 1),
                ("statue", 1, 12, 3),
                ("troglo", 1, 12, 5),
                ("dwarf", 1, 12, 9),
                ("unicorn", 1, 13, 9),
            ],
            1,
        )
    };
    let bolt = r.spell(spell(&r, "bolt")).unwrap();
    let chance =
        |s: &BattleState, id: u8| affect_chance(s, &r, BattleSide::Attacker, bolt, StackId(id));
    // Without: the wyrm (spell levels up to 1) and the troglodyte resist the bolt, the statue
    // (mind) and the troglodyte Blind; the dwarf next to the unicorn keeps 0.8 × 0.8 less the
    // hero's 20 %.
    let s = battle(&[], &ward);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "bolt"), [2, 4, 5]);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "blind"), [1, 4, 5]);
    assert_eq!(chance(&s, 4), 48);
    // Negated by the caster's hero or the stacks' hero: every bolt target, Blind but at the
    // troglodyte (an absolute immunity), the dwarf's share whole but the unicorn's aura.
    for (attacker, defender) in [(&orb[..], &ward[..]), (&[][..], &[ward[0], orb[0]][..])] {
        let s = battle(attacker, defender);
        assert_eq!(
            targets(&s, &r, BattleSide::Attacker, "bolt"),
            [1, 2, 3, 4, 5]
        );
        assert_eq!(targets(&s, &r, BattleSide::Attacker, "blind"), [1, 2, 4, 5]);
        assert_eq!(chance(&s, 4), 80);
        assert_eq!(chance(&s, 5), 100);
    }
}

// covers: BAT-MAG-066, BAT-MAG-063, BAT-MAG-064
#[test]
fn a_negation_asks_the_casters_and_the_controllers_hero_and_spares_the_heroes_defences() {
    let r = ruleset();
    let orb = BattleEffect::NegateImmunities;
    // A positive mind spell at the caster's own statue: the enemy's negation does not count.
    let own = |attacker: &[BattleEffect], defender: &[BattleEffect]| {
        let s = arena(
            &r,
            [
                wearing(&r, &["mirth"], attacker),
                wearing(&r, &[], defender),
            ],
            &[("statue", 1, 2, 5)],
            &[("fixed", 1, 12, 5)],
            1,
        );
        targets(&s, &r, BattleSide::Attacker, "mirth")
    };
    assert!(own(&[], &[orb]).is_empty());
    assert_eq!(own(&[orb], &[]), [0]);
    // The heroes' own defences come first and stay.
    let bolt = spell(&r, "bolt");
    for defence in [
        BattleEffect::SpellImmunity { spell: bolt },
        BattleEffect::SpellLevelImmunity { max_level: 1 },
    ] {
        let s = arena(
            &r,
            [wearing(&r, &["bolt"], &[orb]), wearing(&r, &[], &[defence])],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 12, 5)],
            1,
        );
        assert!(targets(&s, &r, BattleSide::Attacker, "bolt").is_empty());
    }
}
