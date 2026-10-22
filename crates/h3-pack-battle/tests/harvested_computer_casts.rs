//! The original's battle AI's creature casts on the harvested packs (E23 T4a; BAT-AI-012,
//! BAT-AI-013): the Archangels' and the Pit Lords' raise when it beats the walker's value, the
//! Faerie Dragons' drawn spell wherever it is worth anything (at the targets the engine accepts),
//! its estimate at Expert on its school's field with BAT-QUIRK-25 (survey slip 7), no cast on
//! Cursed Ground and none that costs the side more than the enemy; the Ogre Mages' Bloodlust (E23
//! T4b1; BAT-AI-014) when it beats their attack, at a stack as strong as they are, skipped three
//! times in ten, its mass version on Magic Plains, its estimate at Expert on Fiery Fields with
//! BAT-QUIRK-25 (slip 8); the elementals that never cast (BAT-Q57); the Master Genies' cast at their
//! side when the mean of the spells they could draw beats their attack (E23 T4c; BAT-AI-016).

use h3_battle::{
    BattleState, CastSpell, CastTarget, Command, Field, Hex, LegalCommands, NoopSink, StackId,
    apply, computer, creature_cast_targets, legal_commands, stack_spell, validate,
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

/// An army's stacks: the creatures' keys and counts.
type Stacks<'a> = &'a [(&'a str, u32)];

/// A battle of `attacker` against `defender` on Normal with the heroes `heroes`, on the special
/// battlefield `battlefield` if any, from `seed`.
fn battle(
    r: &BattleRuleset,
    (attacker, defender): (Stacks<'_>, Stacks<'_>),
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

// covers: BAT-AI-013
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

/// A battle of `pegasi` Silver Pegasi against `mages` Ogre Mages and `others` on `battlefield` (on
/// Normal, from seed 3): the pegasi fly to hex 98, beside the defender's last stack, every other
/// stack before the Ogre Mages defends, and the Ogre Mages are to act.
fn mages_against_pegasi(
    r: &BattleRuleset,
    pegasi: u32,
    (mages, others): (u32, (&str, u32)),
    battlefield: Option<&str>,
) -> BattleState {
    let mut state = battle(
        r,
        (
            &[("base:silver_pegasus", pegasi)],
            &[("base:ogre_mage", mages), others],
        ),
        [None, None],
        battlefield,
        3,
    );
    assert!(acting(r, &state, "base:silver_pegasus"));
    let fly = Command::Move { to: Hex(98) };
    apply(&mut state, r, BattleSide::Attacker, fly, &mut NoopSink).unwrap();
    until(r, &mut state, "base:ogre_mage");
    state
}

/// The commands of the side to act from the streams of seeds `0..n`.
fn choices(r: &BattleRuleset, state: &BattleState, n: u64) -> Vec<Command> {
    let side = state.side_to_act().unwrap();
    (0..n)
        .map(|seed| h3_battle_ai::choose(state, r, side, &mut Rng::from_seed(seed)))
        .collect()
}

/// The Ogre Mages' Bloodlust at stack `t`.
fn bloodlust_at(t: StackId) -> Command {
    Command::CreatureCast {
        target: CastTarget::Stack(t),
    }
}

// covers: BAT-AI-014
#[test]
fn the_ogre_mages_bless_a_stack_as_strong_as_they_are_when_it_beats_their_attack() {
    // 10 Ogre Mages (strength 6 720) value their attack on 15 Silver Pegasi at about 300 to 400
    // per turn. Nine Dendroid Soldiers beside the pegasi (strength 6 885, not yet acted) gain by
    // Bloodlust's 6 attack g = 125 / 97 on a base of 100: their strike of 105 rises to 135, worth
    // trunc((√g − 1) × 6 885) = 930 over the battle's class 2 (six rounds, all of it). So the
    // mages cast it at them unless the scan's draw skips (three in ten).
    let r = sod("base:faithful");
    let state = mages_against_pegasi(&r, 15, (10, ("base:dendroid_soldier", 9)), None);
    let dendroids = stack_of(&r, &state, "base:dendroid_soldier");
    let mages = stack_of(&r, &state, "base:ogre_mage");
    let ctx = computer::context(&state, &r, BattleSide::Defender);
    assert_eq!(computer::strength(&state, &r, &ctx, mages), 6720);
    assert_eq!(computer::strength(&state, &r, &ctx, dendroids), 6885);
    let commands = choices(&r, &state, 20);
    let casts = commands
        .iter()
        .filter(|&&c| c == bloodlust_at(dendroids))
        .count();
    assert!(casts >= 10, "{commands:?}");
    let mut state = state;
    apply(
        &mut state,
        &r,
        BattleSide::Defender,
        bloodlust_at(dendroids),
        &mut NoopSink,
    )
    .unwrap();
    let blessed = r.spell_by_key("base:bloodlust").unwrap();
    assert!(
        state
            .effects()
            .iter()
            .any(|e| e.stack == dendroids && e.spell == blessed)
    );
    // Eight dendroids (strength 6 120) would be worth about 830, more than the attack, but once
    // there is a value to beat the scan takes only stacks at least as strong as the caster.
    let state = mages_against_pegasi(&r, 15, (10, ("base:dendroid_soldier", 8)), None);
    let commands = choices(&r, &state, 20);
    assert!(
        commands
            .iter()
            .all(|c| !matches!(c, Command::CreatureCast { .. })),
        "{commands:?}"
    );
}

// covers: BAT-AI-014
#[test]
fn three_scans_in_ten_cast_nothing_when_the_attack_is_worth_something() {
    // With an attack worth about 300 to 400 the scan draws 1 to 100 and casts nothing on 30 or
    // less: the mages attack instead.
    let r = sod("base:faithful");
    let state = mages_against_pegasi(&r, 15, (10, ("base:dendroid_soldier", 9)), None);
    let dendroids = stack_of(&r, &state, "base:dendroid_soldier");
    let commands = choices(&r, &state, 200);
    let casts = commands
        .iter()
        .filter(|&&c| c == bloodlust_at(dendroids))
        .count();
    assert!((120..=160).contains(&casts), "{casts}");
    assert!(
        commands
            .iter()
            .all(|&c| c == bloodlust_at(dendroids) || matches!(c, Command::Attack { .. })),
        "{commands:?}"
    );
    // Against 20 pegasi their attack is worth nothing (they would lose more than they kill): no
    // draw, the cast every time; 30 dendroids gain 6 attack for their whole strike (no cap).
    let state = mages_against_pegasi(&r, 20, (10, ("base:dendroid_soldier", 30)), None);
    let dendroids = stack_of(&r, &state, "base:dendroid_soldier");
    assert!(
        choices(&r, &state, 20)
            .iter()
            .all(|&c| c == bloodlust_at(dendroids))
    );
}

// covers: BAT-AI-014
#[test]
fn no_bloodlust_where_the_strike_already_takes_nearly_everything() {
    // 30 dendroids strike 12 pegasi (360 hit points) for 351: Bloodlust can add only the 9 left,
    // g = 360 / 351, worth trunc((√g − 1) × 22 950) = 292, below the mages' attack (about 500 to
    // 600): they attack.
    let r = sod("base:faithful");
    let state = mages_against_pegasi(&r, 12, (10, ("base:dendroid_soldier", 30)), None);
    let pegasi = stack_of(&r, &state, "base:silver_pegasus");
    let dendroids = stack_of(&r, &state, "base:dendroid_soldier");
    let count = u64::from(state.stacks()[dendroids.index()].count);
    assert_eq!(
        computer::damage_estimate(&state, &r, (dendroids, pegasi), count, false, 0),
        351
    );
    assert!(
        choices(&r, &state, 20)
            .iter()
            .all(|c| matches!(c, Command::Attack { target, .. } if *target == pegasi))
    );
}

// covers: BAT-AI-014, BAT-TER-005
#[test]
fn on_magic_plains_the_ogre_mages_cast_the_mass_version() {
    // Magic Plains raise the creatures' casts to Expert: the rules cast Bloodlust's mass version,
    // which takes no target. The estimate is the evaluator's sum over the side: the Ancient
    // Behemoths acted in a battle so one-sided (class 1) that their share is 0, the mages' own,
    // trunc((√(145 / 115) − 1) × 2 016) = 247, beats an attack worth nothing.
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        let mut state = mages_against_pegasi(
            &r,
            10,
            (3, ("base:ancient_behemoth", 2)),
            Some("base:magic_plains"),
        );
        let mass = Command::CreatureCast {
            target: CastTarget::None,
        };
        // The listing offers the cast without a target, and no stack (BAT-MAG-048).
        let mut targets = h3_battle::CastTargets::new();
        creature_cast_targets(&state, &r, BattleSide::Defender, &mut targets);
        assert_eq!(targets.as_slice(), &[CastTarget::None], "{profile}");
        assert!(
            choices(&r, &state, 20).iter().all(|&c| c == mass),
            "{profile}"
        );
        apply(&mut state, &r, BattleSide::Defender, mass, &mut NoopSink).unwrap();
        let blessed = r.spell_by_key("base:bloodlust").unwrap();
        for key in ["base:ogre_mage", "base:ancient_behemoth"] {
            let s = stack_of(&r, &state, key);
            assert!(
                state
                    .effects()
                    .iter()
                    .any(|e| e.stack == s && e.spell == blessed),
                "{profile} {key}"
            );
        }
    }
}

// covers: BAT-AI-014
#[test]
fn on_fiery_fields_the_estimate_takes_expert_with_the_slip() {
    // Fiery Fields raise Bloodlust's school for a hero, not a creature's cast: the rules cast it
    // at Advanced on one stack. The original's estimate takes Expert there (survey slip 8): the
    // mass value, the mages' own 247, makes them bless the behemoths. Without the slip the
    // behemoths' own value is 0 (they acted, class 1): no cast.
    let pick = |profile: &str| {
        let r = sod(profile);
        let state = mages_against_pegasi(
            &r,
            10,
            (3, ("base:ancient_behemoth", 2)),
            Some("sod:fiery_fields"),
        );
        let behemoths = stack_of(&r, &state, "base:ancient_behemoth");
        (choices(&r, &state, 20), behemoths)
    };
    let (faithful, behemoths) = pick("base:faithful");
    assert!(faithful.iter().all(|&c| c == bloodlust_at(behemoths)));
    let (fixed, _) = pick("base:strict-fixed");
    assert!(
        fixed
            .iter()
            .all(|c| !matches!(c, Command::CreatureCast { .. }))
    );
}

// covers: BAT-AI-014
#[test]
fn a_stack_with_bloodlust_is_not_blessed_again() {
    let r = sod("base:faithful");
    let mut state = mages_against_pegasi(&r, 20, (10, ("base:dendroid_soldier", 30)), None);
    let dendroids = stack_of(&r, &state, "base:dendroid_soldier");
    let cast = decide(&r, &state);
    assert_eq!(cast, bloodlust_at(dendroids));
    apply(&mut state, &r, BattleSide::Defender, cast, &mut NoopSink).unwrap();
    // The next round: Bloodlust still lasts on the dendroids, the only other stack is the caster.
    until(&r, &mut state, "base:dendroid_soldier");
    until(&r, &mut state, "base:ogre_mage");
    assert!(state.casts_left(state.active().unwrap()) > 0);
    assert!(
        choices(&r, &state, 20)
            .iter()
            .all(|c| !matches!(c, Command::CreatureCast { .. }))
    );
}

// covers: BAT-AI-012
#[test]
fn the_computers_elementals_never_cast() {
    // The rules let the Storm Elementals protect the dendroids (BAT-MAG-048); the computer's
    // table sends no elemental to a cast routine (BAT-Q57).
    let r = sod("base:faithful");
    let mut state = battle(
        &r,
        (
            &[("base:silver_pegasus", 15)],
            &[("ab:storm_elemental", 10), ("base:dendroid_soldier", 9)],
        ),
        [None, None],
        None,
        3,
    );
    let fly = Command::Move { to: Hex(98) };
    apply(&mut state, &r, BattleSide::Attacker, fly, &mut NoopSink).unwrap();
    until(&r, &mut state, "ab:storm_elemental");
    let mut targets = h3_battle::CastTargets::new();
    creature_cast_targets(&state, &r, BattleSide::Defender, &mut targets);
    assert!(!targets.is_empty());
    assert!(
        choices(&r, &state, 20)
            .iter()
            .all(|c| !matches!(c, Command::CreatureCast { .. }))
    );
}

// covers: BAT-AI-014
#[test]
fn the_spell_contexts_threat_records_and_the_damage_of_a_base() {
    let r = sod("base:faithful");
    let mut state = battle(
        &r,
        (
            &[("base:marksman", 20), ("base:pikeman", 10)],
            &[("ab:peasant", 1), ("base:swordsman", 5)],
        ),
        [None, None],
        None,
        3,
    );
    let marksmen = stack_of(&r, &state, "base:marksman");
    let pikemen = stack_of(&r, &state, "base:pikeman");
    let peasant = stack_of(&r, &state, "ab:peasant");
    let swordsmen = stack_of(&r, &state, "base:swordsman");
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    // A shooter takes every enemy at one turn, the one its shot is worth most against.
    let threat = computer::threat(&state, &r, &ctx, marksmen).unwrap();
    assert_eq!((threat.enemy, threat.turns), (swordsmen, 1));
    // A walker the nearest by its turns: the pikemen (speed 4) need three for 13 hexes or so.
    let threat = computer::threat(&state, &r, &ctx, pikemen).unwrap();
    assert!(threat.turns > 1, "{threat:?}");
    // The damage of a base of 100 by the marksmen (attack 6) on the pikemen (defense 5): 105 for
    // a shot, halved in melee; with 6 attack more 135 and 67.
    let base =
        |shot, bonus| computer::base_damage(&state, &r, (marksmen, pikemen), (100, shot), bonus);
    assert_eq!((base(true, 0), base(false, 0)), (105, 52));
    assert_eq!((base(true, 6), base(false, 6)), (135, 67));
    // A dead stack records nothing.
    let shot = Command::Shoot {
        target: peasant,
        at: state.stacks()[peasant.index()].hex,
    };
    apply(&mut state, &r, BattleSide::Attacker, shot, &mut NoopSink).unwrap();
    assert!(!state.stacks()[peasant.index()].is_alive());
    assert_eq!(computer::threat(&state, &r, &ctx, peasant), None);
}

/// Silver Pegasi that flew to hex 98 against `genies` Master Genies and nine Dendroid Soldiers on
/// the genies' turn.
fn genies_against_pegasi(r: &BattleRuleset, genies: u32) -> BattleState {
    let mut state = battle(
        r,
        (
            &[("base:silver_pegasus", 15)],
            &[("base:master_genie", genies), ("base:dendroid_soldier", 9)],
        ),
        [None, None],
        None,
        3,
    );
    let fly = Command::Move { to: Hex(98) };
    apply(&mut state, r, BattleSide::Attacker, fly, &mut NoopSink).unwrap();
    until(r, &mut state, "base:master_genie");
    state
}

// covers: BAT-AI-016
#[test]
fn the_master_genies_cast_at_their_side_when_it_beats_their_attack() {
    // Five Master Genies cast at the dendroids for every stream, in both profiles, the engine
    // accepting it: the mean of the spells they could draw there beats their attack. Twenty
    // genies' attack on the pegasi is worth more than that mean: they strike.
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        let state = genies_against_pegasi(&r, 5);
        let dendroids = stack_of(&r, &state, "base:dendroid_soldier");
        let cast = Command::CreatureCast {
            target: CastTarget::Stack(dendroids),
        };
        for c in choices(&r, &state, 20) {
            assert_eq!(c, cast, "{profile}");
            assert!(validate(&state, &r, BattleSide::Defender, c).is_ok());
        }
        let state = genies_against_pegasi(&r, 20);
        let pegasi = stack_of(&r, &state, "base:silver_pegasus");
        assert!(
            choices(&r, &state, 20)
                .iter()
                .all(|c| matches!(c, Command::Attack { target, .. } if *target == pegasi)),
            "{profile}"
        );
    }
}
