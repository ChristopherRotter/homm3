//! The original's battle AI's creature casts on the harvested packs (E23 T4a; BAT-AI-012,
//! BAT-AI-013): the Archangels' and the Pit Lords' raise when it beats the walker's value, the
//! Faerie Dragons' drawn spell wherever it is worth anything (at the targets the engine accepts),
//! its estimate at Expert on its school's field with BAT-QUIRK-25 (survey slip 7), no cast on
//! Cursed Ground and none that costs the side more than the enemy.

use h3_battle::{
    BattleState, CastSpell, CastTarget, Command, Field, LegalCommands, NoopSink, StackId, apply,
    computer, legal_commands, stack_spell, validate,
};
use h3_content::{Formation, HeroTypeId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleHero, BattleSetup, BattleSide};
use h3_heroes::{ArtifactInstance, HeroDetails, derive_battle_hero};
use h3_kernel::Rng;

/// The SoD packs' ruleset in `profile`.
fn sod(profile: &str) -> BattleRuleset {
    h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
        profile,
    )
    .unwrap()
}

/// An army of the harvested creatures `stacks` (keys and counts).
fn harvested(r: &BattleRuleset, stacks: &[(&str, u32)]) -> Army {
    Army {
        stacks: (0u8..)
            .zip(stacks)
            .map(|(slot, &(key, count))| ArmyStack {
                slot,
                creature: r.creature_by_key(key).unwrap(),
                count,
            })
            .collect(),
    }
}

/// A hero of the first hero type with the spells `spells` in its book, 100 mana and spell power
/// `power`.
fn mage(r: &BattleRuleset, spells: &[&str], power: u8) -> BattleHero {
    let d = HeroDetails {
        hero_type: HeroTypeId::from_index(0).unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [0, 0, power, 10],
        skills: vec![],
        worn: vec![ArtifactInstance::from(
            r.artifact_by_key("base:spell_book").unwrap(),
        )],
        backpack: vec![],
        spells: spells.iter().map(|s| r.spell_by_key(s).unwrap()).collect(),
        mana: 100,
    };
    h3_heroes::validate(&d, r).unwrap();
    derive_battle_hero(&d, r).unwrap()
}

/// A battle of `attacker` against `defender` on Normal with the heroes `heroes`, on the special
/// battlefield `battlefield` if any, from `seed`.
fn battle(
    r: &BattleRuleset,
    (attacker, defender): (&[(&str, u32)], &[(&str, u32)]),
    heroes: [Option<BattleHero>; 2],
    battlefield: Option<&str>,
    seed: u64,
) -> BattleState {
    let setup = BattleSetup {
        attacker: harvested(r, attacker),
        defender: harvested(r, defender),
        heroes,
        battlefield: battlefield.map(|b| h3_content::ContentKey::parse(b).unwrap()),
        formations: [Formation::Loose, Formation::Loose],
        difficulty: 1,
        ..BattleSetup::default()
    };
    BattleState::new(r, &setup, seed).unwrap()
}

/// The first stack of the creature `key`.
fn stack_of(r: &BattleRuleset, state: &BattleState, key: &str) -> StackId {
    let creature = r.creature_by_key(key).unwrap();
    let i = state
        .stacks()
        .iter()
        .position(|s| s.creature == creature)
        .unwrap();
    StackId(u8::try_from(i).unwrap())
}

/// Whether the active stack is of the creature `key`.
fn acting(r: &BattleRuleset, state: &BattleState, key: &str) -> bool {
    let id = state.active().unwrap();
    Some(state.stack(id).unwrap().creature) == r.creature_by_key(key)
}

/// Every stack defends until one of `key` acts.
fn until(r: &BattleRuleset, state: &mut BattleState, key: &str) {
    while !acting(r, state, key) {
        let side = state.side_to_act().unwrap();
        apply(state, r, side, Command::Defend, &mut NoopSink).unwrap();
    }
}

/// The computer's command for the side to act, which the engine accepts.
fn decide(r: &BattleRuleset, state: &BattleState) -> Command {
    let side = state.side_to_act().unwrap();
    let command = h3_battle_ai::choose(state, r, side, &mut Rng::from_seed(1));
    assert!(validate(state, r, side, command).is_ok(), "{command:?}");
    command
}

/// A bolt of the attacker's hero, cast at stack `t` during its stack's turn.
fn bolt(r: &BattleRuleset, state: &mut BattleState, spell: &str, t: StackId) {
    let cast = Command::Cast {
        spell: r.spell_by_key(spell).unwrap(),
        target: CastTarget::Stack(t),
        second: CastTarget::None,
    };
    apply(state, r, BattleSide::Attacker, cast, &mut NoopSink).unwrap();
}

// covers: BAT-AI-012
#[test]
fn the_archangels_raise_a_stack_that_lost_many_rather_than_strike_a_peasant() {
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        let mut state = battle(
            &r,
            (
                &[("ab:peasant", 1)],
                &[("base:archangel", 5), ("base:pikeman", 60)],
            ),
            [Some(mage(&r, &["base:lightning_bolt"], 10)), None],
            None,
            3,
        );
        // Nothing to raise at their first turn: no cast.
        assert!(acting(&r, &state, "base:archangel"));
        assert!(!matches!(decide(&r, &state), Command::CreatureCast { .. }));
        // The hero's Lightning Bolt (25 × 10 + 10) kills 26 of the 60 pikemen; the archangels
        // raise them all (min(26, 500 / 10)), worth more than a peasant, the value doubled in a
        // battle so one-sided.
        until(&r, &mut state, "ab:peasant");
        let pikemen = stack_of(&r, &state, "base:pikeman");
        bolt(&r, &mut state, "base:lightning_bolt", pikemen);
        assert_eq!(state.stacks()[pikemen.index()].count, 34, "{profile}");
        until(&r, &mut state, "base:archangel");
        let raise = decide(&r, &state);
        assert_eq!(
            raise,
            Command::CreatureCast {
                target: CastTarget::Stack(pikemen)
            },
            "{profile}"
        );
        apply(&mut state, &r, BattleSide::Defender, raise, &mut NoopSink).unwrap();
        assert_eq!(state.stacks()[pikemen.index()].count, 60, "{profile}");
        // Their one cast is used: no raise at their next turn.
        until(&r, &mut state, "ab:peasant");
        bolt(&r, &mut state, "base:lightning_bolt", pikemen);
        until(&r, &mut state, "base:archangel");
        assert!(!matches!(decide(&r, &state), Command::CreatureCast { .. }));
    }
}

// covers: BAT-AI-012
#[test]
fn the_raise_must_beat_the_walkers_value() {
    let r = sod("base:faithful");
    let mut state = battle(
        &r,
        (
            &[("base:marksman", 20), ("ab:peasant", 1)],
            &[("base:archangel", 1), ("base:pikeman", 60)],
        ),
        [Some(mage(&r, &["base:magic_arrow"], 1)), None],
        None,
        3,
    );
    // A Magic Arrow kills two pikemen: raising them is worth less than striking the marksmen.
    until(&r, &mut state, "base:marksman");
    let pikemen = stack_of(&r, &state, "base:pikeman");
    bolt(&r, &mut state, "base:magic_arrow", pikemen);
    assert_eq!(state.stacks()[pikemen.index()].count, 58);
    until(&r, &mut state, "base:archangel");
    let marksmen = stack_of(&r, &state, "base:marksman");
    assert!(matches!(
        decide(&r, &state),
        Command::Attack { target, .. } if target == marksmen
    ));
}

// covers: BAT-AI-012
#[test]
fn the_pit_lords_raise_demons_from_a_corpse() {
    let r = sod("base:faithful");
    let mut state = battle(
        &r,
        (
            &[("base:titan", 10)],
            &[("base:pit_lord", 10), ("base:imp", 100)],
        ),
        [None, None],
        None,
        3,
    );
    // The titans shoot all the imps first; the pit lords raise 11 demons from their corpse
    // (min(100, min(500, 400) div 35)) rather than walk at the far, stronger titans.
    let imps = stack_of(&r, &state, "base:imp");
    assert!(acting(&r, &state, "base:titan"));
    let shot = Command::Shoot {
        target: imps,
        at: state.stacks()[imps.index()].hex,
    };
    apply(&mut state, &r, BattleSide::Attacker, shot, &mut NoopSink).unwrap();
    assert!(!state.stacks()[imps.index()].is_alive());
    until(&r, &mut state, "base:pit_lord");
    let lords = stack_of(&r, &state, "base:pit_lord");
    assert_eq!(computer::raise_count(&state, &r, lords, imps), 11);
    assert_eq!(
        decide(&r, &state),
        Command::CreatureCast {
            target: CastTarget::Stack(imps)
        }
    );
}

/// The drawn spell's key of the active Faerie Dragons.
fn drawn(r: &BattleRuleset, state: &BattleState) -> String {
    let dragon = state.active().unwrap();
    let spell = stack_spell(state, r, dragon)
        .and_then(CastSpell::named)
        .unwrap();
    r.spell(spell).unwrap().key.to_string()
}

// covers: BAT-AI-013
#[test]
fn the_faerie_dragons_cast_their_drawn_spell_wherever_it_is_worth_anything() {
    let r = sod("base:faithful");
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..64 {
        // The dragons (speed 15) act first, before any enemy is in reach; their side's hero
        // adds its bonuses to the chain and the area spells.
        let state = battle(
            &r,
            (
                &[
                    ("base:pikeman", 50),
                    ("base:archer", 30),
                    ("ab:peasant", 20),
                ],
                &[("ab:faerie_dragon", 10)],
            ),
            [None, Some(mage(&r, &[], 0))],
            None,
            seed,
        );
        assert!(acting(&r, &state, "ab:faerie_dragon"));
        let spell = drawn(&r, &state);
        let command = decide(&r, &state);
        let enemy = |t: StackId| state.stacks()[t.index()].side == BattleSide::Attacker;
        match spell.as_str() {
            // A bolt or the chain at an enemy stack.
            "base:magic_arrow"
            | "base:ice_bolt"
            | "base:lightning_bolt"
            | "base:chain_lightning" => {
                assert!(
                    matches!(command, Command::CreatureCast { target: CastTarget::Stack(t) } if enemy(t)),
                    "{seed} {spell}: {command:?}"
                );
            }
            // An area at an enemy's hex.
            "base:fireball" | "base:meteor_shower" | "base:inferno" => {
                assert!(
                    matches!(
                        command,
                        Command::CreatureCast {
                            target: CastTarget::Hex(_)
                        }
                    ),
                    "{seed} {spell}: {command:?}"
                );
            }
            // Frost Ring spares its centre: at a stack's hex, the only targets the engine
            // accepts (BAT-STUB-8), it hits no other stack of these far apart; no cast.
            _ => {
                assert_eq!(spell, "base:frost_ring");
                assert!(!matches!(command, Command::CreatureCast { .. }), "{seed}");
            }
        }
        seen.insert(spell);
    }
    assert_eq!(seen.len(), 8, "{seen:?}");
}

// covers: BAT-AI-013, BAT-TER-004
#[test]
fn no_creature_casts_on_cursed_ground() {
    let r = sod("base:faithful");
    for seed in 0..8 {
        let state = battle(
            &r,
            (&[("base:pikeman", 50)], &[("ab:faerie_dragon", 10)]),
            [None, None],
            Some("base:cursed_ground"),
            seed,
        );
        assert!(acting(&r, &state, "ab:faerie_dragon"));
        assert!(!matches!(decide(&r, &state), Command::CreatureCast { .. }));
    }
}

// covers: BAT-AI-013, BAT-QUIRK-25
#[test]
fn the_dragons_estimate_takes_expert_on_its_schools_field_with_the_slip() {
    // One dragon's Ice Bolt (power 5) on Lucid Pools: a naga queen (110 hit points, capped
    // either way) is worth more than a behemoth (160, never capped) at Advanced, less at Expert.
    let ice = |r: &BattleRuleset| r.spell_by_key("base:ice_bolt").unwrap();
    let pick = |profile: &str| {
        let r = sod(profile);
        let id = ice(&r);
        let mut seed = 0;
        let state = loop {
            let state = battle(
                &r,
                (
                    &[("base:naga_queen", 1), ("base:behemoth", 1)],
                    &[("ab:faerie_dragon", 1)],
                ),
                [None, None],
                Some("sod:lucid_pools"),
                seed,
            );
            if acting(&r, &state, "ab:faerie_dragon") && drawn(&r, &state) == "base:ice_bolt" {
                break state;
            }
            seed += 1;
        };
        let ctx = computer::context(&state, &r, BattleSide::Defender);
        let spell = r.spell(id).unwrap();
        let value = |mastery: usize, t: u8| {
            let damage = u64::from(spell.power) * 5 + u64::from(spell.effect[mastery]);
            computer::spell_damage_value(&state, &r, &ctx, (id, damage), false, StackId(t))
        };
        assert!(value(2, 0) > value(2, 1) && value(3, 0) < value(3, 1));
        decide(&r, &state)
    };
    let naga = Command::CreatureCast {
        target: CastTarget::Stack(StackId(0)),
    };
    let behemoth = Command::CreatureCast {
        target: CastTarget::Stack(StackId(1)),
    };
    assert_eq!(pick("base:faithful"), behemoth);
    assert_eq!(pick("base:strict-fixed"), naga);
}

// covers: BAT-AI-013
#[test]
fn no_spell_that_costs_the_side_more_than_the_enemy() {
    // The defender's archangels fly next to the attacker's lone peasant: every hex the dragon's
    // chain or area spell may take hurts the archangels more than the peasant (or only them); no
    // cast. A bolt spares its own side and goes at the peasant.
    let r = sod("base:faithful");
    let field = Field::new(r.battle().field);
    let (mut bolts, mut chains, mut areas) = (0, 0, 0);
    for seed in 0..48 {
        let mut state = battle(
            &r,
            (
                &[("ab:peasant", 1)],
                &[("base:archangel", 20), ("ab:faerie_dragon", 1)],
            ),
            [None, None],
            None,
            seed,
        );
        let archangels = stack_of(&r, &state, "base:archangel");
        let peasant = stack_of(&r, &state, "ab:peasant");
        assert!(acting(&r, &state, "base:archangel"));
        // The first move the engine offers that ends beside the peasant.
        let mut legal = LegalCommands::new();
        legal_commands(&state, &r, BattleSide::Defender, &mut legal);
        let target = state.stacks()[peasant.index()].hex;
        let me = state.stacks()[archangels.index()];
        let beside = |to| {
            field
                .footprint(to, me.is_two_hex(), me.side)
                .is_some_and(|f| f.hexes().any(|h| field.are_adjacent(h, target)))
        };
        let to = legal
            .iter()
            .find_map(|c| match *c {
                Command::Move { to } if beside(to) => Some(to),
                _ => None,
            })
            .unwrap();
        apply(
            &mut state,
            &r,
            BattleSide::Defender,
            Command::Move { to },
            &mut NoopSink,
        )
        .unwrap();
        // (A good morale turn of the archangels defends.)
        until(&r, &mut state, "ab:faerie_dragon");
        let spell = drawn(&r, &state);
        let command = decide(&r, &state);
        match spell.as_str() {
            "base:magic_arrow" | "base:ice_bolt" | "base:lightning_bolt" => {
                bolts += 1;
                let at_peasant = Command::CreatureCast {
                    target: CastTarget::Stack(peasant),
                };
                assert_eq!(command, at_peasant, "{seed}");
            }
            "base:chain_lightning" => {
                chains += 1;
                assert!(!matches!(command, Command::CreatureCast { .. }), "{seed}");
            }
            _ => {
                areas += 1;
                assert!(
                    !matches!(command, Command::CreatureCast { .. }),
                    "{seed} {spell}"
                );
            }
        }
    }
    assert!(
        bolts > 0 && chains > 0 && areas > 0,
        "{bolts} {chains} {areas}"
    );
}
