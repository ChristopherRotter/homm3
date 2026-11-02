//! The fair system actor (E26 T2; BAT-AI-025, ADR-0027 point 4 as amended): the original's
//! computer on its side's world of the battle. Twins that differ only in the other side's hero's
//! private part give it the same command while its side's observations of them agree, where the
//! all-seeing computer's differ; with no other hero it is the all-seeing computer. The other
//! side's hidden pits (the test ruleset has no hidden obstacle): `h3-pack-battle`'s
//! `harvested_computer.rs` (E26 T3).

use std::vec::Vec;

use h3_battle::{BattleState, Command, NoopSink, apply, observe, validate};
use h3_contracts::{BattleHero, BattleSetup, BattleSide, DamageBonuses, Rate};
use h3_kernel::Rng;

use super::{ARMY, caster, hero, ruleset_as, setup};
use crate::{choose, choose_fair};

/// A random field battle of the test armies, on a random difficulty, each side's `human` drawn.
fn battle(rng: &mut Rng) -> BattleSetup {
    let army = |rng: &mut Rng| -> Vec<(usize, u32)> {
        (0..=rng.below(4))
            .map(|_| (ARMY[rng.below(14) as usize], 1 + rng.below(40)))
            .collect()
    };
    let (attacker, defender) = (army(rng), army(rng));
    let mut setup = setup(&attacker, &defender);
    setup.difficulty = u8::try_from(rng.below(5)).unwrap();
    setup.human = [rng.below(2) == 0, rng.below(2) == 0];
    setup
}

/// `hero` without its private magic and skills: an empty book it still carries, no mastery, no
/// damage bonus. Its public part (type, primary skills, mana, sex, war machines) stays.
fn bare(hero: &BattleHero) -> BattleHero {
    let mut bare = hero.clone();
    bare.magic.spell_book = Some(Vec::new());
    bare.magic.masteries = Vec::new();
    bare.damage = DamageBonuses::default();
    bare
}

/// The other side's hero's book, masteries and damage skills change the all-seeing computer's
/// command in some battle; the fair actor's never, at any step its side's observations of the
/// two battles agree (the other side only defends, so they part only by outcomes its skills
/// change), and it leaves the stream as on the other battle.
// covers: BAT-AI-025
#[test]
fn the_other_heros_private_part_changes_nothing_for_the_fair_actor() {
    let mut differed = 0;
    for game in 0..24u64 {
        let r = ruleset_as(if game % 2 == 0 {
            "t:faithful"
        } else {
            "t:fixed"
        });
        let mut rng = Rng::from_seed(game);
        let viewer = if rng.below(2) == 0 {
            BattleSide::Attacker
        } else {
            BattleSide::Defender
        };
        let other = viewer.opponent();
        let mut full = caster(&r, 3);
        full.damage = DamageBonuses {
            ranged: Rate::bp(5000),
            melee: Rate::bp(5000),
            reduction: Rate::bp(3000),
        };
        let mut seen = battle(&mut rng);
        seen.heroes[viewer.index()] = (rng.below(2) == 0).then(|| caster(&r, 2));
        seen.heroes[other.index()] = Some(full.clone());
        let mut hidden = seen.clone();
        hidden.heroes[other.index()] = Some(bare(&full));
        let mut a = BattleState::new(&r, &seen, game).unwrap();
        let mut b = BattleState::new(&r, &hidden, game).unwrap();
        let mut seat = Rng::from_seed(game ^ 0xfa1e);
        for _ in 0..120 {
            let Some(side) = a.side_to_act() else {
                break;
            };
            if observe(&a, &r, viewer) != observe(&b, &r, viewer) {
                break;
            }
            if side == other {
                apply(&mut a, &r, side, Command::Defend, &mut NoopSink).unwrap();
                apply(&mut b, &r, side, Command::Defend, &mut NoopSink).unwrap();
                continue;
            }
            let (mut on_a, mut on_b) = (seat.clone(), seat.clone());
            let fair = choose_fair(&a, &r, viewer, &mut on_a);
            assert_eq!(fair, choose_fair(&b, &r, viewer, &mut on_b), "game {game}");
            assert_eq!(on_a, on_b);
            let all_seeing = choose(&a, &r, viewer, &mut seat.clone());
            differed += usize::from(all_seeing != choose(&b, &r, viewer, &mut seat.clone()));
            seat = on_a;
            apply(&mut a, &r, viewer, fair, &mut NoopSink).unwrap();
            apply(&mut b, &r, viewer, fair, &mut NoopSink).unwrap();
        }
    }
    assert!(
        differed > 0,
        "the hidden part never changed the all-seeing computer's command"
    );
}

/// Where the side to act's opponent has no hero (nothing of the battle is hidden from it on the
/// test ruleset), the fair command is the all-seeing one's and leaves the stream where it does;
/// where it has one, the fair command is one the engine accepts.
// covers: BAT-AI-025
#[test]
fn without_another_hero_the_fair_actor_is_the_original() {
    let mut compared = 0;
    for game in 0..24u64 {
        let r = ruleset_as(if game % 2 == 0 {
            "t:faithful"
        } else {
            "t:fixed"
        });
        let mut rng = Rng::from_seed(game ^ 0x0516);
        let mut setup = battle(&mut rng);
        for side in 0..2 {
            setup.heroes[side] = match rng.below(3) {
                0 => Some(caster(&r, 1 + u8::try_from(rng.below(3)).unwrap())),
                1 => Some(hero(0)),
                _ => None,
            };
        }
        let mut state = BattleState::new(&r, &setup, game).unwrap();
        let mut seat = Rng::from_seed(game);
        for _ in 0..200 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            let mut fair_seat = seat.clone();
            let fair = choose_fair(&state, &r, side, &mut fair_seat);
            let command = choose(&state, &r, side, &mut seat);
            if setup.heroes[side.opponent().index()].is_none() {
                assert_eq!(fair, command, "game {game}");
                assert_eq!(fair_seat, seat);
                compared += 1;
            } else {
                assert!(
                    validate(&state, &r, side, fair).is_ok(),
                    "game {game}: {fair:?}"
                );
            }
            apply(&mut state, &r, side, command, &mut NoopSink).unwrap();
        }
    }
    assert!(compared > 0);
}
