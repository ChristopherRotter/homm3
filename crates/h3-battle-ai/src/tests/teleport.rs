//! The computer hero's Teleport (0x43AA60 with the destination map 0x4B2FF0; E23 T5b2a;
//! BAT-AI-020) on the test ruleset: each stack's two walker turns and their gain, the stacks it
//! asks, "now", a disabled stack's turns, the destination map's stops with survey slip 30 and its
//! run-up with slip 31; a caster's cast entry in those turns from its own record (E23 T5b2b).

use h3_battle::computer::{self, Planned};
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
    ANGEL, ARCHER, Armies, CAVALIER, CHAMPION, FAERIE, GENIE, GOG, HARPY, IMP, RUNNER, active,
    at_town, cast_at, caster, decide, hero_with, play, ruleset, ruleset_as, spell,
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

// covers: BAT-AI-020
#[test]
fn the_search_stops_are_the_searchs_own() {
    // `computer::search_stops` is `Search::run`'s own stops, which the search keeps in a copy of
    // its own (a call costs it): every stack's, a flyer's (none) and the moat's at a town.
    let r = ruleset();
    let field = Field::new(r.battle().field);
    for state in [
        besieged(&r, 10, &[(HARPY, 5)], 1),
        BattleState::new(
            &r,
            &led(&[(GOG, 3)], &[(IMP, 2)], hero_with(&r, &[], 1, 0)),
            1,
        )
        .unwrap(),
    ] {
        for (i, s) in (0u8..).zip(state.stacks()) {
            let id = StackId(i);
            if !s.is_alive() || s.is_turret() {
                continue;
            }
            let stops = computer::search_stops(&state, &r, id);
            let map = computer::map(&state, &r, id, s.side);
            for h in (0u8..).map(Hex).take(field.hex_count()) {
                assert_eq!(stops.contains(h), map.is_stop(h), "{id:?} {h:?}");
            }
        }
    }
}

/// The attacker's `own` led by a hero with Basic Teleport against `enemies`, at the first turn of
/// the attacker's gogs once `caster` (another stack of the attacker) has waited if it acts first.
fn gogs_turn(r: &BattleRuleset, own: &[(usize, u32)], enemies: &[(usize, u32)]) -> BattleState {
    let hero = hero_with(r, &["teleport"], 1, 99);
    let mut state = BattleState::new(r, &led(own, enemies, hero), 1).unwrap();
    until_side(&mut state, r, ATTACKER);
    if active(&state) != GOG {
        play(&mut state, r, Command::Wait);
        until_side(&mut state, r, ATTACKER);
    }
    assert_eq!(active(&state), GOG);
    state
}

// covers: BAT-AI-020, BAT-AI-016
#[test]
fn a_caster_that_is_not_the_active_stack_asks_its_own_cast_entry() {
    // The Master Genie waited; at the gogs' turn Teleport asks its turns too, and its first turn
    // asks its cast entry from its own record, at the targets the engine would take from it
    // (§7.59): a spell on the 50 gogs beats striking the lone imp. Worth more than its teleported
    // attack, it leaves Teleport to the gogs, due now.
    let r = ruleset();
    let state = gogs_turn(&r, &[(GOG, 50), (GENIE, 5)], &[(IMP, 1)]);
    let genie = StackId(1);
    let dest = destinations(&state, &r, ATTACKER, genie);
    for seed in 0..4 {
        let real = Walker::new(&state, &r, (genie, ATTACKER)).turn(&mut Rng::from_seed(seed));
        assert_eq!(
            real.0,
            Command::CreatureCast {
                target: CastTarget::Stack(StackId(0)),
            }
        );
        let turns = Walker::teleport_turns(
            &state,
            &r,
            (genie, ATTACKER),
            dest,
            &mut Rng::from_seed(seed),
        );
        let (before, (_, after)) = (turns.before, turns.after);
        assert_eq!(before, real.1);
        assert!(after < before, "{before} {after}");
    }
    let v = teleport_value(&state, &r, ATTACKER, 9);
    assert_eq!((v.target, v.now), (CastTarget::Stack(StackId(0)), true));
}

// covers: BAT-AI-020
#[test]
fn with_the_slip_a_casters_context_in_teleports_turns_leaves_its_planned_damage() {
    // Survey slip 33: the Master Genie's cast entry in its Teleport turns builds a spell context;
    // with BAT-QUIRK-25 the damage its predictions planned stays on the stacks, and the spells the
    // hero values after Teleport read it. The gogs' turns build none, nor do the genie's without
    // the flag.
    for (profile, slips) in [("t:faithful", true), ("t:fixed", false)] {
        let r = ruleset_as(profile);
        let state = gogs_turn(&r, &[(GOG, 50), (GENIE, 5)], &[(IMP, 1)]);
        let mut rng = Rng::from_seed(3);
        let mut last = None;
        for (id, caster) in [(StackId(0), false), (StackId(1), true)] {
            let dest = destinations(&state, &r, ATTACKER, id);
            let turns = Walker::teleport_turns(&state, &r, (id, ATTACKER), dest, &mut rng);
            assert_eq!(turns.planned.is_some(), slips && caster, "{profile} {id:?}");
            last = turns.planned.or(last);
        }
        // Teleport's valuation asks the same turns from the same stream.
        let ctx = computer::context(&state, &r, ATTACKER);
        let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
        let own = sc.planned();
        let h = Hero::new(&sc, (&state, &r), (state.active().unwrap(), ATTACKER, ctx));
        let id = spell(&r, "teleport");
        let hc = computer::hero_cast(&state, &r, ATTACKER, (id, r.spell(id).unwrap()));
        let c = Candidate {
            spell: id,
            mastery: hc.mastery,
            power: u32::from(hc.power),
            rounds: u32::from(hc.rounds),
            cast: true,
        };
        let _ = hero::value(&h, (id, r.spell(id).unwrap()), c, &mut Rng::from_seed(3));
        assert_eq!(*sc.planned_now(), last.unwrap_or(own), "{profile}");
    }
}

// covers: BAT-AI-020
#[test]
fn the_first_teleports_planned_damage_holds_for_the_choice() {
    // `SpellContext::after_teleport`: the planned damage replaces the context's, the retreat
    // test's too, and the at-risk test reads it; Teleport is valued once per choice, so a second
    // call changes nothing.
    let r = ruleset();
    let state = gogs_turn(&r, &[(GOG, 50), (GENIE, 5)], &[(IMP, 1)]);
    let ctx = computer::context(&state, &r, ATTACKER);
    let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    assert!(!sc.at_risk());
    let mut planned = Planned::new();
    planned.add(&state, StackId(0), 1_000_000);
    sc.after_teleport(planned);
    assert!(sc.at_risk());
    sc.after_teleport(Planned::new());
    assert_eq!(*sc.planned_now(), planned);
}

// covers: BAT-AI-020, BAT-AI-013, BAT-MAG-049
#[test]
fn a_faerie_dragon_that_is_not_active_casts_the_spell_of_its_last_activation() {
    // The faerie acts after the gogs. In the first round it has drawn nothing: its first turn
    // walks, Teleport's best is the faerie, not due now, and the hero casts nothing. It draws the
    // bolt at its turn; at the gogs' next turn it still holds it (the original's +0x4E0): its
    // first turn casts it, worth more than its teleported attack, and the hero teleports the
    // gogs now.
    let r = ruleset();
    let mut state = gogs_turn(&r, &[(GOG, 50), (FAERIE, 5)], &[(IMP, 10)]);
    let faerie = StackId(1);
    assert_eq!(computer::drawn_spell(&state, faerie), None);
    let (real, _) = turns(&state, &r, (faerie, ATTACKER), 3);
    assert!(matches!(real.0, Command::Move { .. }), "{real:?}");
    let v = teleport_value(&state, &r, ATTACKER, 9);
    assert_eq!((v.target, v.now), (CastTarget::Stack(faerie), false));
    assert_eq!(cast(&state, &r), None);
    // Every stack defends until the gogs' next turn.
    play(&mut state, &r, Command::Defend);
    while active(&state) != GOG {
        play(&mut state, &r, Command::Defend);
    }
    assert_eq!(state.round(), 2);
    assert_eq!(
        computer::drawn_spell(&state, faerie),
        Some(spell(&r, "bolt"))
    );
    let (real, _) = turns(&state, &r, (faerie, ATTACKER), 3);
    assert_eq!(
        real.0,
        Command::CreatureCast {
            target: CastTarget::Stack(StackId(2)),
        }
    );
    let v = teleport_value(&state, &r, ATTACKER, 9);
    assert_eq!((v.target, v.now), (CastTarget::Stack(StackId(0)), true));
    let Some(Command::Cast {
        spell: s, target, ..
    }) = cast(&state, &r)
    else {
        panic!("no cast");
    };
    assert_eq!(
        (s, target),
        (spell(&r, "teleport"), CastTarget::Stack(StackId(0)))
    );
}

/// A Citadel with a harmless moat on Easy: the attacker's `own`, led by a hero with Expert
/// Teleport, against `enemies`, at the first turn of the attacker's archers once `ready` holds.
fn archers_turn(
    r: &BattleRuleset,
    (own, enemies): Armies<'_>,
    ready: impl Fn(&BattleState) -> bool,
) -> BattleState {
    let mut s = at_town(r, (own, enemies), (FortLevel::Citadel, Moat::Harmless, 0));
    s.heroes[0] = Some(hero_with(r, &["teleport"], 3, 99));
    s.difficulty = 0;
    let mut state = BattleState::new(r, &s, 1).unwrap();
    for _ in 0..20 {
        if state.side_to_act() == Some(ATTACKER) && active(&state) == ARCHER && ready(&state) {
            return state;
        }
        let wait = active(&state) != ARCHER
            && state.side_to_act() == Some(ATTACKER)
            && !computer::wait_phase(&state);
        play(
            &mut state,
            r,
            if wait { Command::Wait } else { Command::Defend },
        );
    }
    panic!("the archers never acted");
}

// covers: BAT-AI-020
#[test]
fn after_a_cast_entry_s_spell_context_the_destination_map_keeps_its_last_search_s_stops() {
    // On Easy no reach is searched and the archers (shooters) are no threat records: the stops
    // the destination map keeps are those of the last search of the faerie's first turn after its
    // map (§7.59). Its cast entry builds a spell context, whose last search is the defender's gogs
    // (their stops the moat but the bridge), and casts: with BAT-QUIRK-25 the moat is a stop, the
    // bridge not; the faerie's own stops (the bridge too) only when the turn does not record its
    // searches; without the slip none (survey slip 30).
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let slip = profile == "t:faithful";
        let faerie = StackId(1);
        let state = archers_turn(&r, (&[(ARCHER, 20), (FAERIE, 5)], &[(GOG, 10)]), |s| {
            computer::drawn_spell(s, faerie).is_some()
        });
        let dest = destinations(&state, &r, ATTACKER, faerie);
        let w = Walker::tracking(&state, &r, (faerie, ATTACKER));
        let real = w.turn(&mut Rng::from_seed(3));
        assert!(matches!(real.0, Command::CreatureCast { .. }), "{real:?}");
        let t = w.teleported(dest);
        assert_eq!((t.map.is_stop(MOAT), t.map.is_stop(BRIDGE)), (slip, false));
        let own = Walker::new(&state, &r, (faerie, ATTACKER)).teleported(dest);
        assert_eq!(
            (own.map.is_stop(MOAT), own.map.is_stop(BRIDGE)),
            (slip, slip)
        );
    }
    // The genie's context (the gogs' last too) is followed by its walk, which searches for the
    // genie, a flyer: no stops.
    let r = ruleset();
    let genie = StackId(0);
    let state = archers_turn(&r, (&[(GENIE, 5), (ARCHER, 50)], &[(GOG, 1)]), |_| true);
    let dest = destinations(&state, &r, ATTACKER, genie);
    let w = Walker::tracking(&state, &r, (genie, ATTACKER));
    let real = w.turn(&mut Rng::from_seed(3));
    assert!(matches!(real.0, Command::Move { .. }), "{real:?}");
    let t = w.teleported(dest);
    assert!(!t.map.is_stop(MOAT));
}

// covers: BAT-AI-012, BAT-AI-020
#[test]
fn a_stack_raiser_s_test_takes_the_active_stack_s_hit_points_with_the_slip() {
    // The enemy's bolt kills one of the five cavaliers (100 hit points each). On the angels' own
    // turn their raise test has their 2 × 100 hit points: they raise a cavalier. At the lone gog's
    // turn Teleport asks the angels' turn too, and with BAT-QUIRK-25 the original's test reads the
    // active record, one gog at 50 per creature, short of a cavalier (survey slip 32): no raise;
    // without the slip the angels' own.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let slip = profile == "t:faithful";
        let mut s = led(
            &[(GOG, 1), (CAVALIER, 5), (ANGEL, 2)],
            &[(IMP, 1)],
            hero_with(&r, &["teleport"], 1, 99),
        );
        s.heroes[1] = Some(caster(&r, 3));
        let mut state = BattleState::new(&r, &s, 1).unwrap();
        let (cavaliers, angels) = (StackId(1), StackId(2));
        cast_at(&mut state, &r, DEFENDER, "bolt", cavaliers);
        assert_eq!(state.stacks()[1].count, 4);
        let raise = Command::CreatureCast {
            target: CastTarget::Stack(cavaliers),
        };
        let mut own_turn = None;
        for _ in 0..20 {
            let mine = state.side_to_act() == Some(ATTACKER);
            if mine && active(&state) == GOG && state.round() >= 2 {
                break;
            }
            if mine && active(&state) == ANGEL && !computer::wait_phase(&state) {
                own_turn = Some(
                    Walker::new(&state, &r, (angels, ATTACKER))
                        .turn(&mut Rng::from_seed(3))
                        .0,
                );
                play(&mut state, &r, Command::Wait);
            } else {
                play(&mut state, &r, Command::Defend);
            }
        }
        assert_eq!(own_turn, Some(raise), "{profile}");
        assert_eq!(active(&state), GOG);
        let real = Walker::new(&state, &r, (angels, ATTACKER)).turn(&mut Rng::from_seed(3));
        assert_eq!(real.0 == raise, !slip, "{profile}: {real:?}");
    }
}
