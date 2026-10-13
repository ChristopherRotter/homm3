//! The opening spells (E04b T8c, BAT-MAG-068): at the first activation of a stack a side controls
//! outside the tactics phase, the side's hero casts its opening spells for free (Angelic
//! Alliance's Prayer, Armor of the Damned's four), after the original's code
//! (`docs/formats/installation.md` §7.24).

use std::borrow::ToOwned;
use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSetup, BattleSide};

use super::spells::spell;
use super::support::*;
use crate::{
    BattleState, Command, Event, EventKind, HeroError, SetupError, StackId, Status, castable_spells,
};

/// The opening spell `name` at Expert with `power`.
fn opening(r: &BattleRuleset, name: &str, power: u16) -> BattleEffect {
    BattleEffect::OpeningSpell {
        spell: spell(r, name),
        mastery: 3,
        power,
    }
}

/// The test hero with `effects`, a spell book with the bolt and mana 15.
fn wearer(r: &BattleRuleset, effects: Vec<BattleEffect>) -> BattleHero {
    let mut h = hero(r);
    h.effects = effects;
    h.magic.spell_book = Some(vec![spell(r, "bolt")]);
    h.magic.masteries = masteries(r, [1, 0, 0, 0]);
    h
}

/// Angelic Alliance's opening (Prayer, power 10) with a duration bonus of 2 for the attacker,
/// Armor of the Damned's (Slow and Curse, power 50) for the defender.
fn heroes(r: &BattleRuleset) -> [Option<BattleHero>; 2] {
    [
        Some(wearer(
            r,
            vec![
                opening(r, "prayer", 10),
                BattleEffect::SpellDuration { rounds: 2 },
            ],
        )),
        Some(wearer(
            r,
            vec![opening(r, "sloth", 50), opening(r, "curse", 50)],
        )),
    ]
}

/// The attacker's fast and slow stacks against the defender's dragon and flyer: the turn order
/// is the attacker's 0 (speed 9), the defender's 2 (8) and 3 (7), then the attacker's 1 (2, 6
/// with Expert Prayer).
fn armies(r: &BattleRuleset, heroes: [Option<BattleHero>; 2]) -> BattleSetup {
    BattleSetup {
        heroes,
        ..setup(
            r,
            &[(0, "fast", 10), (1, "slow", 10)],
            &[(0, "dragon", 1), (1, "flyer", 10)],
        )
    }
}

fn start(r: &BattleRuleset, setup: &BattleSetup) -> (BattleState, Vec<Event>) {
    let mut events = Vec::new();
    let s = BattleState::start(r, setup, 1, &mut events).unwrap();
    (s, events)
}

/// The opening spells and the effects they gave, as `(side or stack, spell name, rounds)`, in
/// order: `OpeningSpell` as `(side index + 100, spell, 0)`.
fn openings(r: &BattleRuleset, events: &[Event]) -> Vec<(u8, String, u16)> {
    let name = |id| {
        r.spell(id)
            .unwrap()
            .key
            .as_str()
            .trim_start_matches("t:")
            .to_owned()
    };
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::OpeningSpell { side, spell } => {
                Some((100 + u8::try_from(side.index()).unwrap(), name(spell), 0))
            }
            EventKind::EffectApplied {
                target,
                spell,
                rounds,
            } => Some((target.0, name(spell), rounds.unwrap_or(0))),
            _ => None,
        })
        .collect()
}

fn effect(n: u8, name: &str, rounds: u16) -> (u8, String, u16) {
    (n, name.to_owned(), rounds)
}

// covers: BAT-MAG-068
#[test]
fn each_side_casts_its_opening_spells_at_its_first_activation() {
    let r = ruleset();
    let (mut s, events) = start(&r, &armies(&r, heroes(&r)));
    // The attacker's fast stack is the battle's first: its hero's Prayer reaches both of its
    // stacks for 10 rounds (no duration bonus), the defender's spells wait.
    assert_eq!(
        openings(&r, &events),
        [
            effect(100, "prayer", 0),
            effect(0, "prayer", 10),
            effect(1, "prayer", 10)
        ]
    );
    let first = events
        .iter()
        .position(|e| e.kind == EventKind::TurnStarted { stack: StackId(0) })
        .unwrap();
    assert!(
        events
            .iter()
            .position(|e| matches!(e.kind, EventKind::OpeningSpell { .. }))
            .unwrap()
            > first
    );
    // Free and not the round's cast: the hero keeps its mana and may still cast.
    let view = s.hero(BattleSide::Attacker).unwrap();
    assert_eq!((view.mana, view.casts), (15, 0));
    assert!(castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "bolt")));
    // The defender's first stack: Slow, then Curse, each on both attacking stacks for 50 rounds.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(
        openings(&r, &events),
        [
            effect(101, "sloth", 0),
            effect(0, "sloth", 50),
            effect(1, "sloth", 50),
            effect(101, "curse", 0),
            effect(0, "curse", 50),
            effect(1, "curse", 50),
        ]
    );
    assert_eq!(s.hero(BattleSide::Defender).unwrap().casts, 0);
    // Once a battle: no side casts them again, in this round or the next.
    for _ in 0..6 {
        let events = act(&mut s, &r, Command::Defend).unwrap();
        assert!(openings(&r, &events).is_empty());
    }
}

// covers: BAT-MAG-068
#[test]
fn the_orb_of_inhibition_does_not_stop_the_opening_spells() {
    let r = ruleset();
    let mut with_orb = heroes(&r);
    with_orb[1]
        .as_mut()
        .unwrap()
        .effects
        .push(BattleEffect::NoCasting);
    let (mut s, events) = start(&r, &armies(&r, with_orb));
    assert_eq!(openings(&r, &events).len(), 3);
    // The heroes may not cast, but the defender's armor still casts its two.
    assert!(castable_spells(&s, &r, BattleSide::Attacker).is_empty());
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(openings(&r, &events).len(), 6);
}

// covers: BAT-MAG-068
#[test]
fn an_opening_spell_without_a_stack_it_affects_is_not_cast() {
    let r = ruleset();
    // The attacker's stacks are immune to Slow (a Pendant-like defence of their hero): the
    // defender casts only Curse.
    let mut immune = heroes(&r);
    immune[0]
        .as_mut()
        .unwrap()
        .effects
        .push(BattleEffect::SpellImmunity {
            spell: spell(&r, "sloth"),
        });
    let (mut s, _) = start(&r, &armies(&r, immune));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(
        openings(&r, &events),
        [
            effect(101, "curse", 0),
            effect(0, "curse", 50),
            effect(1, "curse", 50),
        ]
    );
    // A hypnotized stack counts for the test, though the mass cast then passes over it
    // (0x5A40D0, 0x5A66D0): the defender's Prayer, its own stacks immune to it but the one the
    // attacker hypnotized (whose hero's defences now guard it), is cast and reaches none.
    let mut praying = heroes(&r);
    praying[1] = Some(wearer(
        &r,
        vec![
            opening(&r, "prayer", 10),
            BattleEffect::SpellImmunity {
                spell: spell(&r, "prayer"),
            },
        ],
    ));
    let (mut s, _) = start(&r, &armies(&r, praying.clone()));
    let st = &mut s.stacks.as_mut_slice()[3];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(openings(&r, &events), [effect(101, "prayer", 0)]);
    // Not hypnotized, it is immune too: no cast at all.
    let (mut s, _) = start(&r, &armies(&r, praying));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(openings(&r, &events).is_empty());
}

// covers: BAT-MAG-068
#[test]
fn a_side_whose_first_stack_is_passed_over_casts_at_its_next_stack() {
    let r = ruleset();
    let (mut s, _) = start(&r, &armies(&r, heroes(&r)));
    // The defender's first stack (2) is blind: its turn passes, the opening waits for stack 3.
    {
        let st = &mut s.stacks.as_mut_slice()[2];
        st.spells.status = st.spells.status.with(Status::NO_TURNS, true);
    }
    let events = act(&mut s, &r, Command::Defend).unwrap();
    let passed = events
        .iter()
        .position(|e| e.kind == EventKind::TurnPassed { stack: StackId(2) })
        .unwrap();
    let started = events
        .iter()
        .position(|e| e.kind == EventKind::TurnStarted { stack: StackId(3) })
        .unwrap();
    let cast = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::OpeningSpell { .. }))
        .unwrap();
    assert!(passed < started && started < cast);
    assert_eq!(s.active(), Some(StackId(3)));
}

// covers: BAT-MAG-068, BAT-MOR-004, BAT-MAG-060
#[test]
fn a_side_whose_first_stack_freezes_casts_at_the_first_stack_that_takes_its_turn() {
    use h3_content::ContentKey;
    use h3_content_battle::AbilityDef;
    // E04b T11b review (r6 F-6): bad morale and Fear pass a stack over before the activation
    // that casts its side's opening spells, as the original activates only after its pick's
    // checks. Over 40 battles each (only the defender has a hero, so the attacker's `fast` is
    // the battle's first stack and the defender's two follow): when the defender's first stack
    // freezes after the attacker's Defend and its second does not, the cast follows the second
    // one's turn start; when both freeze, nothing is cast in that command.
    let waited = |r: &BattleRuleset, setup: &BattleSetup, (first, second): (u8, u8)| {
        let mut waited = 0;
        for seed in 0..40 {
            let mut s = BattleState::start(r, setup, seed, &mut Vec::<Event>::new()).unwrap();
            let events = act(&mut s, r, Command::Defend).unwrap();
            let frozen = |id: u8| {
                let stack = StackId(id);
                events.iter().position(|e| {
                    e.kind == EventKind::BadMorale { stack }
                        || e.kind == EventKind::Frightened { stack }
                })
            };
            let started = |id: u8| {
                let turn = EventKind::TurnStarted { stack: StackId(id) };
                events.iter().position(|e| e.kind == turn).unwrap()
            };
            let cast = events.iter().position(|e| {
                let defender = BattleSide::Defender;
                matches!(e.kind, EventKind::OpeningSpell { side, .. } if side == defender)
            });
            match (frozen(first), frozen(second)) {
                (None, _) => assert!(started(first) < cast.unwrap(), "{seed}"),
                (Some(freeze), None) => {
                    assert!(
                        freeze < started(second) && started(second) < cast.unwrap(),
                        "{seed}"
                    );
                    assert_eq!(s.active(), Some(StackId(second)), "{seed}");
                    waited += 1;
                }
                (Some(_), Some(_)) => assert_eq!(cast, None, "{seed}"),
            }
        }
        waited
    };
    // Bad morale: the defender's hero takes 3 morale from its stacks; a third of the picks of a
    // stack with bad morale freeze it.
    let r = ruleset_with("t:faithful", |b| b.bad_morale_chances = vec![8, 8, 8]);
    let mut sad = [None, heroes(&r)[1].clone()];
    sad[1].as_mut().unwrap().effects.push(BattleEffect::Morale {
        amount: -3,
        affects: h3_content::Affects::Own,
    });
    assert!(waited(&r, &armies(&r, sad), (2, 3)) > 0);
    // Fear: a spook of the attacker's (stack 2, speed 4) frightens half of the picks of the
    // defender's stacks (3 and 4).
    let key = |name: &str| ContentKey::new("t", name).unwrap();
    let r = ruleset_from(
        "t:faithful",
        |_| {},
        |_, battle| {
            let spook = battle.creatures.iter_mut().find(|c| c.key == key("spook"));
            spook.unwrap().abilities = vec![AbilityDef::Fear { percent: 50 }];
        },
    );
    let scared = BattleSetup {
        heroes: [None, heroes(&r)[1].clone()],
        ..setup(
            &r,
            &[(0, "fast", 10), (1, "slow", 10), (2, "spook", 1)],
            &[(0, "dragon", 1), (1, "flyer", 10)],
        )
    };
    assert!(waited(&r, &scared, (3, 4)) > 0);
}

// covers: BAT-MAG-068, BAT-MAG-053
#[test]
fn the_opening_spells_come_before_the_enchanters_own_cast() {
    // E04b T11b review (r6 F-6): the activation casts the side's opening spells before the
    // Enchanters count the spells that are of use. Chanters whose hero opens with Slow never
    // draw Slow, which is on every enemy stack by then; without the hero they draw it in some
    // of 60 battles.
    let r = ruleset();
    let slow = spell(&r, "sloth");
    let drawn = |hero: Option<BattleHero>| {
        let opens = hero.is_some();
        let setup = BattleSetup {
            heroes: [hero, None],
            ..setup(&r, &[(0, "chanter", 5)], &[(0, "fixed", 10)])
        };
        let mut slows = 0;
        for seed in 0..60 {
            let mut events = Vec::new();
            BattleState::start(&r, &setup, seed, &mut events).unwrap();
            let opening = events
                .iter()
                .position(|e| matches!(e.kind, EventKind::OpeningSpell { .. }));
            let (own, cast) = events
                .iter()
                .enumerate()
                .find_map(|(i, e)| match e.kind {
                    EventKind::PeriodicCast { spell, .. } => Some((i, spell)),
                    _ => None,
                })
                .unwrap();
            assert_eq!(opening.is_some(), opens, "{seed}");
            assert!(opening.is_none_or(|o| o < own), "{seed}");
            slows += u32::from(cast == slow);
        }
        slows
    };
    assert_eq!(drawn(Some(wearer(&r, vec![opening(&r, "sloth", 50)]))), 0);
    assert!(drawn(None) > 0);
}

// covers: BAT-MAG-068
#[test]
fn the_opening_spells_wait_for_the_end_of_the_tactics_phase() {
    let r = ruleset();
    let mut both = heroes(&r);
    both[1].as_mut().unwrap().tactics = 1;
    let (mut s, events) = start(&r, &armies(&r, both));
    assert!(s.tactics().is_some());
    assert!(openings(&r, &events).is_empty());
    let events = act(&mut s, &r, Command::EndTactics).unwrap();
    // Round 1's first stack is the attacker's: its Prayer, not yet the defender's spells.
    assert_eq!(
        openings(&r, &events).first(),
        Some(&effect(100, "prayer", 0))
    );
    assert_eq!(openings(&r, &events).len(), 3);
}

// covers: BAT-MAG-068
#[test]
fn the_casting_heros_specialty_applies_to_its_opening_spells() {
    let r = ruleset();
    let prayer = |specialty: bool| {
        let mut h = heroes(&r);
        if specialty {
            h[0].as_mut()
                .unwrap()
                .effects
                .push(BattleEffect::SpellSpecialty {
                    spell: spell(&r, "prayer"),
                    bp_per_level: 300,
                    level: 10,
                });
        }
        let (s, _) = start(&r, &armies(&r, h));
        s.effects()
            .iter()
            .find(|e| e.stack == StackId(0) && e.spell == spell(&r, "prayer"))
            .unwrap()
            .value
    };
    // Expert Prayer's 4, and 3 more on a level-1 creature for a Prayer specialist (BAT-MAG-020).
    assert_eq!((prayer(false), prayer(true)), (4, 7));
}

// covers: BAT-HER-001, BAT-MAG-068
#[test]
fn the_battle_casts_only_opening_spells_it_can() {
    let r = ruleset();
    let refused = |effect: BattleEffect| {
        let setup = armies(&r, [Some(wearer(&r, vec![effect])), None]);
        BattleState::new(&r, &setup, 1).err()
    };
    let error = |error| {
        Some(SetupError::InvalidHero {
            side: BattleSide::Attacker,
            error,
        })
    };
    let with = |name, mastery, power| BattleEffect::OpeningSpell {
        spell: spell(&r, name),
        mastery,
        power,
    };
    // A mastery above Expert, power 0, a target at its mastery (Prayer below Expert, the bolt),
    // an unknown spell.
    assert_eq!(
        refused(with("prayer", 4, 10)),
        error(HeroError::OpeningSpell)
    );
    assert_eq!(
        refused(with("prayer", 3, 0)),
        error(HeroError::OpeningSpell)
    );
    assert_eq!(
        refused(with("prayer", 2, 10)),
        error(HeroError::OpeningSpell)
    );
    assert_eq!(refused(with("bolt", 3, 10)), error(HeroError::OpeningSpell));
    // Without a target but damaging (the storm strikes every stack), or a creature ability.
    assert_eq!(
        refused(with("storm", 3, 10)),
        error(HeroError::OpeningSpell)
    );
    assert_eq!(refused(with("gaze", 3, 10)), error(HeroError::OpeningSpell));
    assert_eq!(
        refused(BattleEffect::OpeningSpell {
            spell: h3_content::SpellId::from_raw(999),
            mastery: 3,
            power: 1,
        }),
        error(HeroError::UnknownSpell)
    );
    assert_eq!(refused(with("prayer", 3, 10)), None);
    // At most `MAX_OPENING_SPELLS`.
    let mut many = wearer(&r, vec![]);
    many.effects = vec![with("prayer", 3, 10); crate::MAX_OPENING_SPELLS + 1];
    let setup = armies(&r, [Some(many), None]);
    assert_eq!(
        BattleState::new(&r, &setup, 1).err(),
        error(HeroError::OpeningSpell)
    );
}

// covers: BAT-MAG-068
#[test]
fn each_stack_rolls_against_an_opening_spell() {
    // The defender's Slow at the attacker's Dwarves (20 % resistance): resisted with some seeds,
    // not with others, one roll each (as any of the hero's mass casts, BAT-MAG-008).
    let r = ruleset();
    let mut outcomes = [0; 2];
    for seed in 0..40 {
        let setup = BattleSetup {
            heroes: [None, Some(wearer(&r, vec![opening(&r, "sloth", 50)]))],
            ..setup(&r, &[(0, "dwarf", 10)], &[(0, "slow", 10)])
        };
        let mut events = Vec::new();
        let mut s = BattleState::start(&r, &setup, seed, &mut events).unwrap();
        let events = act(&mut s, &r, Command::Defend).unwrap();
        let resisted = events
            .iter()
            .any(|e| e.kind == EventKind::SpellResisted { target: StackId(0) });
        assert_eq!(
            openings(&r, &events),
            if resisted {
                vec![effect(101, "sloth", 0)]
            } else {
                vec![effect(101, "sloth", 0), effect(0, "sloth", 50)]
            }
        );
        outcomes[usize::from(resisted)] += 1;
    }
    assert!(outcomes[0] > 0 && outcomes[1] > 0, "{outcomes:?}");
}

// covers: BAT-MAG-068
#[test]
fn a_hypnotized_stacks_activation_is_its_controllers() {
    // The defender's dragon is the battle's first stack; its flyer, hypnotized by the attacker,
    // activates next: the attacker's first activation, so the attacker's Prayer goes then, at
    // the attacker's own stack (the flyer is the defender's).
    let r = ruleset();
    let setup = BattleSetup {
        heroes: [Some(wearer(&r, vec![opening(&r, "prayer", 10)])), None],
        ..setup(
            &r,
            &[(0, "slow", 10)],
            &[(0, "dragon", 1), (1, "flyer", 10)],
        )
    };
    let (mut s, events) = start(&r, &setup);
    assert!(openings(&r, &events).is_empty());
    let st = &mut s.stacks.as_mut_slice()[2];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(2)));
    assert_eq!(
        openings(&r, &events),
        [effect(100, "prayer", 0), effect(0, "prayer", 10)]
    );
}
