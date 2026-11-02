//! The fair system actor (E26 T2; BAT-AI-025, ADR-0027 point 4 as amended): the original's
//! computer on its side's world of the battle. Twins that differ only in the other side's hero's
//! private part give it the same command while its side's observations of them agree, where the
//! all-seeing computer's differ; with no other hero it is the all-seeing computer. The property
//! over random battles (E26 T3) is below; the other side's hidden pits (the test ruleset has no
//! hidden obstacle), sieges of every faction and the special battlefields: `h3-pack-battle`'s
//! `harvested_computer.rs`.

use std::vec::Vec;

use h3_battle::{
    BattleState, Command, LegalCommands, NoopSink, Status, apply, legal_commands, observe, validate,
};
use h3_content_battle::BattleRuleset;
use h3_contracts::{
    BattleHero, BattleSetup, BattleSide, DamageBonuses, FortLevel, Moat, Rate, RetreatFacts,
};
use h3_kernel::Rng;

use super::{ARMY, BALLISTA, TENT, at_town, caster, creature, hero, ruleset_as, setup};
use crate::{choose, choose_fair, unchecked};

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

/// What [`the_fair_choice_is_legal_and_reads_nothing_hidden_in_random_battles`] compared: the
/// decisions (each checked for a fallback and made twice) and the fallbacks known (BAT-STUB-18),
/// those without another hero (the original's), the twin battles compared while the side's
/// observations of them agreed (with another stream; of another setup, `with_hero` of them with
/// another hero, the all-seeing command differing there `differed` times) and the twins of
/// another setup dropped when they parted.
#[derive(Debug, Default)]
struct Tally {
    decisions: u32,
    fallbacks: u32,
    original: u32,
    streams: u32,
    twins: u32,
    with_hero: u32,
    differed: u32,
    parted: u32,
}

/// Random damage bonuses: up to half again on ranged and melee damage, up to 30 % less taken.
fn damage(rng: &mut Rng) -> DamageBonuses {
    DamageBonuses {
        ranged: Rate::bp(rng.below(5001)),
        melee: Rate::bp(rng.below(5001)),
        reduction: Rate::bp(rng.below(3001)),
    }
}

/// Random retreat facts.
fn retreat(rng: &mut Rng) -> RetreatFacts {
    RetreatFacts {
        experience: rng.below(4000),
        artifacts: rng.below(2000),
        tavern: rng.below(4) != 0,
        wanted: rng.below(8) == 0,
    }
}

/// A random hero: [`hero`] with Tactics 0 to 3 (commanding its Ballista and Tent), half the time
/// with a book of about a quarter of the caster's spells at random masteries, power and mana;
/// random damage bonuses and surrender discount.
fn random_hero(r: &BattleRuleset, rng: &mut Rng) -> BattleHero {
    let mut h = hero(u8::try_from(rng.below(4)).unwrap());
    if rng.below(2) == 0 {
        let book: Vec<_> = caster(r, 1)
            .magic
            .spell_book
            .unwrap()
            .into_iter()
            .filter(|_| rng.below(4) == 0)
            .collect();
        h.magic.masteries = book
            .iter()
            .map(|&s| (s, 1 + u8::try_from(rng.below(3)).unwrap()))
            .collect();
        h.magic.spell_book = Some(book);
        h.power = 1 + u8::try_from(rng.below(10)).unwrap();
        h.magic.mana = u16::try_from(20 + rng.below(180)).unwrap();
        h.magic.max_mana = h.magic.mana;
    }
    h.damage = damage(rng);
    h.surrender_discount_bp = u16::try_from(rng.below(5001)).unwrap();
    h
}

/// `hero` with another private part, what its opponent may not know: every spell of the caster
/// in its book if it carries one (a superset, so that it accepts the hero's casts), each at a
/// random mastery, other damage bonuses and surrender discount. Its public part (type, primary
/// skills, mana, whether it carries a book, its war machines) stays.
fn other_private(r: &BattleRuleset, hero: &BattleHero, rng: &mut Rng) -> BattleHero {
    let mut twin = hero.clone();
    if twin.magic.spell_book.is_some() {
        let book = caster(r, 1).magic.spell_book.unwrap();
        twin.magic.masteries = book
            .iter()
            .map(|&s| (s, u8::try_from(rng.below(4)).unwrap()))
            .filter(|&(_, mastery)| mastery > 0)
            .collect();
        twin.magic.spell_book = Some(book);
    }
    twin.damage = damage(rng);
    twin.surrender_discount_bp = u16::try_from(rng.below(5001)).unwrap();
    twin
}

/// A random battle of the test armies on a random difficulty, one in three at the town (every
/// fort level, moat and turrets), each side's `human`, funds and retreat facts drawn and two in
/// three with a [`random_hero`] and its war machines; and for each side its twin: the setup with
/// that side's private part changed ([`other_private`], its funds and retreat facts).
fn game(r: &BattleRuleset, rng: &mut Rng) -> (BattleSetup, [BattleSetup; 2]) {
    let army = |rng: &mut Rng| -> Vec<(usize, u32)> {
        (0..=rng.below(4))
            .map(|_| (ARMY[rng.below(14) as usize], 1 + rng.below(40)))
            .collect()
    };
    let (attacker, defender) = (army(rng), army(rng));
    let mut setup = setup(&attacker, &defender);
    if rng.below(3) == 0 {
        let fort = [FortLevel::Fort, FortLevel::Citadel, FortLevel::Castle][rng.below(3) as usize];
        let moat = if fort == FortLevel::Fort {
            Moat::Absent
        } else {
            [Moat::Absent, Moat::Harmless, Moat::Damaging][rng.below(3) as usize]
        };
        let structures = u8::try_from(rng.below(4)).unwrap();
        setup.siege = at_town(r, (&attacker, &defender), (fort, moat, structures)).siege;
    }
    for side in 0..2 {
        if rng.below(3) != 0 {
            setup.heroes[side] = Some(random_hero(r, rng));
            setup.war_machines[side] = [BALLISTA, TENT]
                .into_iter()
                .filter(|_| rng.below(2) == 0)
                .map(creature)
                .collect();
        }
        setup.funds[side] = rng.below(6000);
        setup.retreat[side] = retreat(rng);
        setup.human[side] = rng.below(2) == 0;
    }
    setup.difficulty = u8::try_from(rng.below(5)).unwrap();
    let twins = [0, 1].map(|side| {
        let mut twin = setup.clone();
        twin.heroes[side] = setup.heroes[side]
            .as_ref()
            .map(|h| other_private(r, h, rng));
        twin.funds[side] = rng.below(6000);
        twin.retreat[side] = retreat(rng);
        twin
    });
    (setup, twins)
}

/// Whether `command` is the active stack's shot at itself, a hypnotized shooter's: the computer
/// takes every stack of the other side as a target, whoever controls it, which the engine refuses
/// (BAT-STUB-18, the shooter's BAT-STUB-10).
fn shoots_itself(state: &BattleState, command: Command) -> bool {
    let active = state.active();
    matches!(command, Command::Shoot { target, .. } if Some(target) == active)
        && active
            .and_then(|id| state.stack(id))
            .is_some_and(|s| s.spells.status.contains(Status::HYPNOTIZED))
}

/// Asserts that the fair command for `side` on `twin`, which `side` observes as the battle, from
/// `seat` is `fair` and leaves the stream at `after`, as on the battle.
fn fair_on_twin(
    r: &BattleRuleset,
    twin: &BattleState,
    (side, seat): (BattleSide, &Rng),
    (fair, after): (Command, &Rng),
    game: u64,
) {
    let mut on = seat.clone();
    assert_eq!(choose_fair(twin, r, side, &mut on), fair, "game {game}");
    assert_eq!(&on, after, "game {game}");
}

/// E26 T3, exit criteria 1 and 2 on the test ruleset (beside the property
/// `the_choice_is_legal_and_the_same_for_the_same_stream`): random battles ([`game`]) of both
/// sides at every difficulty, with and without heroes, on the field or at the town. At every
/// step the original's choice on the side to act's world is a command the engine accepts on the
/// battle (no fallback but the one known, the all-seeing computer's too: BAT-STUB-18, which gives
/// Defend) and is the fair command; the same state and stream give the same fair command and
/// stream; without another hero both are the original's ([`choose`]). Twins that
/// differ only in what the side may not know give the same fair command and leave the stream
/// the same: the battle with another stream at every step, and the twin battle of [`game`]
/// whose other side's private part differs, played on with the same commands, at every step the
/// side's observations of the two agree (until they part: an outcome the private part changes,
/// or a command the twin refuses); there the all-seeing command differs at times. Two steps in
/// three play the fair command, the third a random legal command that stays in the battle.
// covers: BAT-AI-025
#[test]
fn the_fair_choice_is_legal_and_reads_nothing_hidden_in_random_battles() {
    let mut tally = Tally::default();
    for game_no in 0..48u64 {
        let r = ruleset_as(if game_no % 2 == 0 {
            "t:faithful"
        } else {
            "t:fixed"
        });
        let mut rng = Rng::from_seed(game_no ^ 0x0e26_7303);
        let (setup, twin_setups) = game(&r, &mut rng);
        let mut state = BattleState::new(&r, &setup, game_no).unwrap();
        let mut twins = twin_setups.map(|s| Some(BattleState::new(&r, &s, game_no).unwrap()));
        let mut seat = Rng::from_seed(game_no);
        let mut legal = LegalCommands::new();
        for _ in 0..200 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            let other = side.opponent().index();
            let world = state
                .determinized(&r, side, None, seat.clone().next_u64())
                .unwrap();
            let mut again = seat.clone();
            let proposed = unchecked(&world, &r, side, &mut again);
            let refused = validate(&state, &r, side, proposed).is_err();
            // stub: BAT-STUB-18
            // The one fallback known, the all-seeing computer's too: a hypnotized shooter's shot
            // at itself.
            assert!(
                !refused || shoots_itself(&state, proposed),
                "game {game_no}: the fair {proposed:?} is refused"
            );
            tally.fallbacks += u32::from(refused);
            let mut after = seat.clone();
            let fair = choose_fair(&state, &r, side, &mut after);
            let expected = if refused { Command::Defend } else { proposed };
            assert_eq!(fair, expected, "game {game_no}");
            assert_eq!(after, again, "game {game_no}");
            tally.decisions += 1;
            let hero = setup.heroes[other].is_some();
            if !hero {
                let mut all_seeing = seat.clone();
                assert_eq!(
                    fair,
                    choose(&state, &r, side, &mut all_seeing),
                    "game {game_no}"
                );
                assert_eq!(after, all_seeing, "game {game_no}");
                tally.original += 1;
            }
            let seen = observe(&state, &r, side);
            let mut reseeded = state.clone();
            reseeded.reseed(rng.next_u64());
            assert_eq!(observe(&reseeded, &r, side), seen);
            fair_on_twin(&r, &reseeded, (side, &seat), (fair, &after), game_no);
            tally.streams += 1;
            match twins[other]
                .as_ref()
                .map(|t| (observe(t, &r, side) == seen, t))
            {
                Some((true, twin)) => {
                    fair_on_twin(&r, twin, (side, &seat), (fair, &after), game_no);
                    tally.twins += 1;
                    if hero {
                        tally.with_hero += 1;
                        tally.differed += u32::from(
                            choose(twin, &r, side, &mut seat.clone())
                                != choose(&state, &r, side, &mut seat.clone()),
                        );
                    }
                }
                Some((false, _)) => {
                    twins[other] = None;
                    tally.parted += 1;
                }
                None => {}
            }
            seat = after;
            let played = if rng.below(3) == 0 {
                legal_commands(&state, &r, side, &mut legal);
                let staying: Vec<Command> = legal
                    .iter()
                    .copied()
                    .filter(|c| !c.leaves_battle())
                    .collect();
                staying[rng.below(u32::try_from(staying.len()).unwrap()) as usize]
            } else {
                fair
            };
            apply(&mut state, &r, side, played, &mut NoopSink).unwrap();
            for twin in &mut twins {
                if twin
                    .as_mut()
                    .is_some_and(|t| apply(t, &r, side, played, &mut NoopSink).is_err())
                {
                    *twin = None;
                    tally.parted += 1;
                }
            }
        }
    }
    // 2059 decisions, the all-seeing command differing on 7 twins of 284 with another hero, one
    // fallback (E26-log.md T3).
    assert!(
        tally.decisions > 1800
            && tally.original > 600
            && tally.twins > 800
            && tally.with_hero > 200
            && tally.differed > 5,
        "{tally:?}"
    );
}
