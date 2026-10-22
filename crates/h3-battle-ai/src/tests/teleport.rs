//! The computer hero's Teleport (0x43AA60 with the destination map 0x4B2FF0; E23 T5b2a;
//! BAT-AI-020) on the test ruleset: each stack's two walker turns and their gain, the stacks it
//! asks, "now", a disabled stack's turns, the destination map's stops with survey slip 30 and its
//! run-up with slip 31.

use h3_battle::computer;
use h3_battle::{
    BattleState, CastTarget, CastTargets, Command, Field, Hex, HexSet, StackId, second_targets,
    validate,
};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleSide, FortLevel, Moat};
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use super::hero::{cast, led, until_side};
use super::{
    ARCHER, CAVALIER, CHAMPION, GOG, HARPY, IMP, RUNNER, active, at_town, cast_at, caster, decide,
    hero_with, play, ruleset, ruleset_as, spell,
};
use crate::hero::{self, Hero, Valued};
use crate::spells::{Candidate, SpellContext};
use crate::walker::Walker;

const ATTACKER: BattleSide = BattleSide::Attacker;
const DEFENDER: BattleSide = BattleSide::Defender;

/// The hexes the engine lets `side`'s hero teleport stack `r` to now.
fn destinations(state: &BattleState, rs: &BattleRuleset, side: BattleSide, r: StackId) -> HexSet {
    let mut list = CastTargets::new();
    let t = (spell(rs, "teleport"), CastTarget::Stack(r));
    second_targets(state, rs, side, t, &mut list);
    list.as_slice().iter().fold(HexSet::new(), |mut set, t| {
        if let CastTarget::Hex(h) = *t {
            set.insert(h);
        }
        set
    })
}

/// Teleport's value for `side`'s hero in `state`, its walker turns from a stream seeded `seed`.
fn teleport_value(state: &BattleState, r: &BattleRuleset, side: BattleSide, seed: u64) -> Valued {
    let ctx = computer::context(state, r, side);
    let sc = SpellContext::new(state, r, (&ctx, side), &mut Rng::from_seed(5));
    let h = Hero::new(&sc, (state, r), (state.active().unwrap(), side, ctx));
    let id = spell(r, "teleport");
    let hc = computer::hero_cast(state, r, side, (id, r.spell(id).unwrap()));
    let c = Candidate {
        spell: id,
        mastery: hc.mastery,
        power: u32::from(hc.power),
        rounds: u32::from(hc.rounds),
        cast: true,
    };
    hero::value(&h, (id, r.spell(id).unwrap()), c, &mut Rng::from_seed(seed))
}

/// Stack `r`'s two turns for Teleport from a stream seeded `seed`: the real turn's command and
/// value, then the command and value on the destination map.
fn turns(
    state: &BattleState,
    rs: &BattleRuleset,
    (r, side): (StackId, BattleSide),
    seed: u64,
) -> ((Command, i32), (Command, i32)) {
    let mut rng = Rng::from_seed(seed);
    let w = Walker::new(state, rs, (r, side));
    let real = w.turn(&mut rng);
    let dest = destinations(state, rs, side, r);
    (real, w.teleported(dest).turn(&mut rng))
}

/// The town's moat hex right of hex 60 (row 3) and the drawbridge's hex (95), a stop for the
/// attacker's walkers only while the gate is closed.
const MOAT: Hex = Hex(61);
const BRIDGE: Hex = Hex(95);

/// A Citadel with a harmless moat: the attacker's `runners` runners and 30 archers against the
/// defender's 10 gogs and `extra`, whose hero has Expert Teleport; the runners have walked to hex
/// 60 beside the moat and the gogs act (every stack between defends), on `difficulty`.
fn besieged(
    r: &BattleRuleset,
    runners: u32,
    extra: &[(usize, u32)],
    difficulty: u8,
) -> BattleState {
    let defenders: std::vec::Vec<(usize, u32)> = [(GOG, 10)].iter().chain(extra).copied().collect();
    let mut s = at_town(
        r,
        (&[(RUNNER, runners), (ARCHER, 30)], &defenders),
        (FortLevel::Citadel, Moat::Harmless, 0),
    );
    s.heroes[1] = Some(hero_with(r, &["teleport"], 3, 99));
    s.difficulty = difficulty;
    let mut state = BattleState::new(r, &s, 1).unwrap();
    until_side(&mut state, r, ATTACKER);
    assert_eq!(active(&state), RUNNER);
    play(&mut state, r, Command::Move { to: Hex(60) });
    // Every other stack defends until the gogs act.
    while active(&state) != GOG {
        play(&mut state, r, Command::Defend);
    }
    state
}

// covers: BAT-AI-020
#[test]
fn teleport_is_worth_what_the_walker_gains_on_the_destination_map() {
    // The gogs (speed 3) only walk towards the imps on their own turn; teleported beside them they
    // attack now. Teleport's value is the second turn's value less the first's, both turns from
    // one stream; the destination is the plan's hex, due now for the active stack.
    let r = ruleset();
    let own = [(GOG, 10)];
    let enemies = [(IMP, 5)];
    let hero = hero_with(&r, &["teleport"], 1, 99);
    let mut state = BattleState::new(&r, &led(&own, &enemies, hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    let gogs = StackId(0);
    assert_eq!(state.active(), Some(gogs));
    let ((real, before), (after, value)) = turns(&state, &r, (gogs, ATTACKER), 9);
    assert!(matches!(real, Command::Move { .. }), "{real:?}");
    let Command::Attack { target, from } = after else {
        panic!("{after:?}");
    };
    assert_eq!(target, StackId(1));
    assert!(value > before, "{before} {value}");
    let want = Valued {
        value: value - before,
        target: CastTarget::Stack(gogs),
        second: CastTarget::Hex(from),
        now: true,
    };
    assert_eq!(teleport_value(&state, &r, ATTACKER, 9), want);
    // The hero casts it (its own draws: one enemy, the same plan's hex), and the gogs attack.
    let c = cast(&state, &r).unwrap();
    assert_eq!(
        c,
        Command::Cast {
            spell: spell(&r, "teleport"),
            target: CastTarget::Stack(gogs),
            second: CastTarget::Hex(from),
        }
    );
    play(&mut state, &r, c);
    assert_eq!(state.stacks()[0].hex, from);
    assert_eq!(state.active(), Some(gogs));
    assert!(matches!(
        decide(&state, &r),
        Command::Attack {
            target: StackId(1),
            ..
        }
    ));
}

// covers: BAT-AI-020
#[test]
fn a_stack_waits_for_its_own_turn_and_shooters_or_stacks_that_acted_are_not_asked() {
    // At the imps' turn the gogs are the best stack to move but not the active one: not due now,
    // so the hero casts nothing. The archers shoot: never asked.
    let r = ruleset();
    let own = [(IMP, 1), (GOG, 10), (ARCHER, 10)];
    let enemies = [(IMP, 5)];
    let hero = hero_with(&r, &["teleport"], 1, 99);
    let mut state = BattleState::new(&r, &led(&own, &enemies, hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(state.active(), Some(StackId(0)));
    let v = teleport_value(&state, &r, ATTACKER, 9);
    assert_eq!((v.target, v.now), (CastTarget::Stack(StackId(1)), false));
    assert_eq!(cast(&state, &r), None);
    // The imps wait, the archers and the gogs defend; at the imps' second turn the gogs have
    // acted: only the imps are asked.
    play(&mut state, &r, Command::Wait);
    while state.active() != Some(StackId(0)) {
        play(&mut state, &r, Command::Defend);
    }
    let v = teleport_value(&state, &r, ATTACKER, 9);
    assert!(
        matches!(v.target, CastTarget::None | CastTarget::Stack(StackId(0))),
        "{v:?}"
    );
}

// covers: BAT-AI-020
#[test]
fn a_blind_stack_is_moved_now_by_its_plan_alone() {
    // The enemy's hero blinds the gogs. At the archers' turn (a shooter, never asked) Teleport
    // weighs the blind gogs: their map reaches nothing (range 0), and on the destination map they
    // add no melee value, only the plan's: beside the enemy's archers they block them. Due now.
    let r = ruleset();
    let own = [(GOG, 10), (ARCHER, 5)];
    let enemies = [(ARCHER, 10)];
    let mut s = led(&own, &enemies, hero_with(&r, &["teleport"], 1, 99));
    s.heroes[1] = Some(caster(&r, 1));
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    let gogs = StackId(0);
    cast_at(&mut state, &r, DEFENDER, "blind", gogs);
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(state.active(), Some(StackId(1)));
    let ((_, before), (after, value)) = turns(&state, &r, (gogs, ATTACKER), 9);
    assert_eq!(before, 0);
    let Command::Attack { target, from } = after else {
        panic!("{after:?}");
    };
    assert_eq!(target, StackId(2));
    // The plan's value: the blocked archers, max(1, hit points × (w₁ − w₀) / hit points per
    // creature) as doubles, drawn at 75 to 100 %; no melee value.
    let ctx = computer::context(&state, &r, ATTACKER);
    let w = |shot| computer::worth(&state, &r, &ctx, StackId(2), shot);
    let gain = X87::int(100)
        .times(w(true).minus(w(false)))
        .over(X87::int(10))
        .unwrap();
    let blocked = computer::ftol(gain).max(1);
    assert!(
        (blocked * 75 / 100..=blocked).contains(&value),
        "{value} {blocked}"
    );
    let v = teleport_value(&state, &r, ATTACKER, 9);
    assert_eq!(
        v,
        Valued {
            value,
            target: CastTarget::Stack(gogs),
            second: CastTarget::Hex(from),
            now: true,
        }
    );
    let c = cast(&state, &r).unwrap();
    assert!(validate(&state, &r, ATTACKER, c).is_ok(), "{c:?}");
}

// covers: BAT-AI-020
#[test]
fn the_flag_1_turn_defends_when_its_best_is_worth_less_than_nothing() {
    // Twenty runners outweigh the ten gogs anywhere: their teleported turn defends, so Teleport
    // is worth nothing there.
    let r = ruleset();
    let state = besieged(&r, 20, &[], 1);
    let gogs = state.active().unwrap();
    let ((_, _), (after, value)) = turns(&state, &r, (gogs, DEFENDER), 9);
    assert_eq!(after, Command::Defend);
    assert!(value < 0, "{value}");
    assert_eq!(
        teleport_value(&state, &r, DEFENDER, 9).target,
        CastTarget::None
    );
}

// covers: BAT-AI-020
#[test]
fn the_destination_map_reaches_every_destination_at_one_turn_and_marks_the_enemies_beside_them() {
    let r = ruleset();
    let own = [(GOG, 10)];
    let enemies = [(IMP, 5), (CAVALIER, 2)];
    let hero = hero_with(&r, &["teleport"], 1, 99);
    let mut state = BattleState::new(&r, &led(&own, &enemies, hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    let gogs = StackId(0);
    let start = state.stacks()[0].hex;
    // Only the hexes beside the imps: the imps are marked at 1, the cavaliers not.
    let field = Field::new(r.battle().field);
    let imps = state.stacks()[1].hex;
    let dest = destinations(&state, &r, ATTACKER, gogs);
    let beside = field
        .neighbors(imps)
        .iter()
        .flatten()
        .filter(|h| dest.contains(**h))
        .fold(HexSet::new(), |mut set, &h| {
            set.insert(h);
            set
        });
    let map = computer::destination_map(&state, &r, (gogs, ATTACKER), beside, HexSet::new());
    for h in beside.iter() {
        assert_eq!(map.position(h), Some(1), "{h:?}");
        assert_eq!(map.mark(h), Some(1), "{h:?}");
        assert!(!map.is_stop(h));
    }
    assert_eq!(map.position(start), None);
    assert_eq!(map.mark(imps), Some(1));
    assert_eq!(map.mark(state.stacks()[2].hex), None);
    assert_eq!(map.cost(state.stacks()[2].hex), 0);
    // A two-hex enemy's position is marked, its other hex never (survey slip 5).
    let cav = state.stacks()[2].footprint(field);
    let around = cav.hexes().fold(HexSet::new(), |mut set, h| {
        for n in field.neighbors(h).iter().flatten() {
            if dest.contains(*n) {
                set.insert(*n);
            }
        }
        set
    });
    let map = computer::destination_map(&state, &r, (gogs, ATTACKER), around, HexSet::new());
    let mut hexes = cav.hexes();
    let (pos, other) = (hexes.next().unwrap(), hexes.next().unwrap());
    let (pos, other) = if pos == state.stacks()[2].hex {
        (pos, other)
    } else {
        (other, pos)
    };
    assert_eq!(map.mark(pos), Some(1));
    assert_eq!(map.mark(other), None);
}

// covers: BAT-AI-020
#[test]
fn the_destination_map_keeps_the_last_searchs_stops_with_the_slip() {
    // On Normal the gogs' scores search the enemies' reach last: the runners' (archers shoot).
    // Their stops, the moat and the closed gate's bridge, stay the destination map's with
    // BAT-QUIRK-25 (survey slip 30); without it Teleport stops nowhere.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let slip = profile == "t:faithful";
        let state = besieged(&r, 10, &[], 1);
        let gogs = state.active().unwrap();
        let dest = destinations(&state, &r, DEFENDER, gogs);
        assert!(dest.contains(MOAT));
        let w = Walker::new(&state, &r, (gogs, DEFENDER)).teleported(dest);
        assert_eq!(w.map.is_stop(MOAT), slip, "{profile}");
        assert_eq!(w.map.is_stop(BRIDGE), slip, "{profile}");
        // The runners' plan from the moat hex beside them takes two turns with the slip: the
        // archers win; without it the runners, from the moat.
        let ((_, before), (after, value)) = turns(&state, &r, (gogs, DEFENDER), 9);
        let want = if slip {
            Command::Attack {
                target: StackId(1),
                from: Hex(69),
            }
        } else {
            Command::Attack {
                target: StackId(0),
                from: MOAT,
            }
        };
        assert_eq!(after, want, "{profile}");
        let v = teleport_value(&state, &r, DEFENDER, 9);
        assert_eq!(v.value, value - before, "{profile}");
        let c = cast(&state, &r).unwrap();
        assert!(
            validate(&state, &r, DEFENDER, c).is_ok(),
            "{profile}: {c:?}"
        );
    }
}

// covers: BAT-AI-020
#[test]
fn without_the_reach_s_searches_the_stops_are_the_side_s_last_recorded_or_the_stack_s_own() {
    // On Easy the scores search no reach. With the harpies (a flyer, recorded after the gogs) the
    // threat records search them last: no stops. Alone, the gogs' own: the moat but the bridge,
    // which is none for the defender.
    let r = ruleset();
    let state = besieged(&r, 10, &[(HARPY, 5)], 0);
    let gogs = state.active().unwrap();
    let dest = destinations(&state, &r, DEFENDER, gogs);
    let w = Walker::new(&state, &r, (gogs, DEFENDER)).teleported(dest);
    assert!(!w.map.is_stop(MOAT));
    let state = besieged(&r, 10, &[], 0);
    let gogs = state.active().unwrap();
    let dest = destinations(&state, &r, DEFENDER, gogs);
    let w = Walker::new(&state, &r, (gogs, DEFENDER)).teleported(dest);
    assert!(w.map.is_stop(MOAT));
    assert!(!w.map.is_stop(BRIDGE));
}

// covers: BAT-AI-020
#[test]
fn a_teleported_jouster_counts_a_run_up_of_one_hex_only_with_the_slip() {
    // On the destination map the plan's hex costs 1, which the original passes as the jousting
    // run-up (survey slip 31); a teleported stack strikes from where it stands: none without the
    // slip. The same stream, the same destination, a larger value with the slip.
    let mut values = [0; 2];
    for (i, profile) in ["t:faithful", "t:fixed"].into_iter().enumerate() {
        let r = ruleset_as(profile);
        let own = [(CHAMPION, 5)];
        let enemies = [(GOG, 20)];
        let hero = hero_with(&r, &["teleport"], 1, 99);
        let mut state = BattleState::new(&r, &led(&own, &enemies, hero), 1).unwrap();
        until_side(&mut state, &r, ATTACKER);
        let ((_, _), (after, value)) = turns(&state, &r, (StackId(0), ATTACKER), 9);
        assert!(
            matches!(
                after,
                Command::Attack {
                    target: StackId(1),
                    ..
                }
            ),
            "{after:?}"
        );
        values[i] = value;
    }
    assert!(values[0] > values[1], "{values:?}");
}
