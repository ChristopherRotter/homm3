//! The computer's hero cast (0x422DA0, 0x43C800, 0x43BB20; E23 T5a; BAT-AI-017, BAT-AI-018) on the
//! test ruleset: the entry's gates, the choice's scaling, draw and "now", the retreat's damage
//! spell, the raises-only mode, and each value case.

use h3_battle::computer::{self, HeroCast};
use h3_battle::{
    BattleSetup, BattleState, CastTarget, CastTargets, Command, StackId, StackTraits, cast_targets,
    validate,
};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSide};
use h3_kernel::Rng;

use super::{
    ARCHER, BALLISTA, GOG, IMP, RUNNER, ZOMBIE, active, cast_at, hero_with, play, ruleset,
    ruleset_as, setup, spell,
};
use crate::hero::{self, Hero, Valued};
use crate::spells::{Candidate, SpellContext};

const ATTACKER: BattleSide = BattleSide::Attacker;
const DEFENDER: BattleSide = BattleSide::Defender;

/// `own` led by `hero` against `enemies` without a hero, on Normal.
pub(super) fn led(own: &[(usize, u32)], enemies: &[(usize, u32)], hero: BattleHero) -> BattleSetup {
    let mut s = setup(own, enemies);
    s.heroes[0] = Some(hero);
    s
}

/// Every stack defends until a stack of `side` but a war machine acts.
pub(super) fn until_side(state: &mut BattleState, r: &BattleRuleset, side: BattleSide) {
    for _ in 0..40 {
        let id = state.active().unwrap();
        let machine = state.stacks()[id.index()].has(StackTraits::WAR_MACHINE);
        if state.side_to_act() == Some(side) && !machine {
            return;
        }
        play(state, r, Command::Defend);
    }
    panic!("{side:?} never acted");
}

/// The hero's cast for the side to act, from a fixed stream.
pub(super) fn cast(state: &BattleState, r: &BattleRuleset) -> Option<Command> {
    let side = state.side_to_act().unwrap();
    hero::cast(
        state,
        r,
        (state.active().unwrap(), side),
        &mut Rng::from_seed(5),
    )
}

/// The hero's choice for the side to act with the retreat test's answer `retreat`.
fn choice(state: &BattleState, r: &BattleRuleset, retreat: bool) -> Option<Command> {
    let side = state.side_to_act().unwrap();
    let ctx = computer::context(state, r, side);
    let mut rng = Rng::from_seed(5);
    let mut sc = SpellContext::new(state, r, (&ctx, side), &mut rng);
    let id = state.active().unwrap();
    hero::choose(&mut sc, (state, r), (id, side, ctx), retreat, &mut rng)
}

/// The candidate of the attacker's hero's cast of `name` in `state`.
pub(super) fn candidate(state: &BattleState, r: &BattleRuleset, name: &str) -> Candidate {
    let id = spell(r, name);
    let hc = computer::hero_cast(state, r, ATTACKER, (id, r.spell(id).unwrap()));
    Candidate {
        spell: id,
        mastery: hc.mastery,
        power: u32::from(hc.power),
        rounds: u32::from(hc.rounds),
        cast: true,
    }
}

/// The value of the attacker's hero's cast of `name` in `state` (0x43BB20).
pub(super) fn valued(state: &BattleState, r: &BattleRuleset, name: &str) -> Valued {
    let ctx = computer::context(state, r, ATTACKER);
    let sc = SpellContext::new(state, r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    let h = Hero::new(&sc, (state, r), (state.active().unwrap(), ATTACKER, ctx));
    let id = spell(r, name);
    hero::value(&h, (id, r.spell(id).unwrap()), candidate(state, r, name))
}

/// The cast of `name` at `target`.
fn cast_of(r: &BattleRuleset, name: &str, target: CastTarget) -> Command {
    Command::Cast {
        spell: spell(r, name),
        target,
        second: CastTarget::None,
    }
}

// covers: BAT-AI-017
#[test]
fn the_values_scale_by_the_casts_the_mana_covers() {
    let hc = |mana, cost| HeroCast {
        mana,
        cost,
        ..HeroCast::default()
    };
    // Seven casts and more: 2.5 times; a free spell too.
    assert_eq!(hero::scaled(101, hc(70, 10)), 252);
    assert_eq!(hero::scaled(101, hc(0, 0)), 252);
    // Fewer: × √casts, truncated; none left, nothing.
    assert_eq!(hero::scaled(100, hc(69, 10)), 244);
    assert_eq!(hero::scaled(100, hc(10, 10)), 100);
    assert_eq!(hero::scaled(100, hc(9, 10)), 0);
}

// covers: BAT-AI-017
#[test]
fn a_lone_stack_bolts_the_enemy_worth_most() {
    // The runners are the side's only stack: every cast is due now. The bolt goes to the enemy it
    // is worth most against, which the engine accepts.
    let r = ruleset();
    let hero = hero_with(&r, &["bolt"], 1, 999);
    let mut state =
        BattleState::new(&r, &led(&[(RUNNER, 10)], &[(IMP, 10), (GOG, 5)], hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    let v = valued(&state, &r, "bolt");
    assert!(v.value > 0 && v.now, "{v:?}");
    let c = cast(&state, &r).unwrap();
    assert_eq!(c, cast_of(&r, "bolt", v.target));
    assert!(validate(&state, &r, ATTACKER, c).is_ok());
    // The best target by the bolt's evaluator.
    let ctx = computer::context(&state, &r, ATTACKER);
    let damage = 10 * 5 + 10;
    let on = |s: u8| {
        computer::spell_damage_value(
            &state,
            &r,
            &ctx,
            (spell(&r, "bolt"), damage),
            true,
            StackId(s),
        )
    };
    let best = (0..state.stacks().len())
        .map(|i| u8::try_from(i).unwrap())
        .filter(|&i| state.stacks()[usize::from(i)].side == DEFENDER)
        .fold(
            (0, None),
            |b, i| if on(i) > b.0 { (on(i), Some(i)) } else { b },
        );
    assert_eq!(v.target, CastTarget::Stack(StackId(best.1.unwrap())));
    // The policy casts first; once cast this round, the stack acts.
    assert_eq!(super::decide(&state, &r), c);
    play(&mut state, &r, c);
    assert_eq!(cast(&state, &r), None);
    assert!(!matches!(super::decide(&state, &r), Command::Cast { .. }));
}

// covers: BAT-AI-017
#[test]
fn no_cast_without_a_book_while_casting_is_forbidden_or_for_a_war_machine() {
    let r = ruleset();
    // No book.
    let mut state =
        BattleState::new(&r, &led(&[(RUNNER, 10)], &[(IMP, 10)], super::hero(0)), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(cast(&state, &r), None);
    // The enemy hero's Orb of Inhibition.
    let mut setup = led(
        &[(RUNNER, 10)],
        &[(IMP, 10)],
        hero_with(&r, &["bolt"], 1, 999),
    );
    let mut orb = super::hero(0);
    orb.effects.push(BattleEffect::NoCasting);
    setup.heroes[1] = Some(orb);
    let mut state = BattleState::new(&r, &setup, 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(cast(&state, &r), None);
    // The side's Ballista acts: no cast at its activation.
    let hero = hero_with(&r, &["bolt"], 1, 999);
    let mut setup = led(&[(RUNNER, 10)], &[(IMP, 10)], hero);
    setup.war_machines = [std::vec![super::creature(BALLISTA)], std::vec![]];
    let mut state = BattleState::new(&r, &setup, 1).unwrap();
    for _ in 0..40 {
        if active(&state) == BALLISTA {
            break;
        }
        play(&mut state, &r, Command::Defend);
    }
    assert_eq!(active(&state), BALLISTA);
    assert_eq!(cast(&state, &r), None);
    // Without a hero: nothing.
    let mut state = BattleState::new(&r, &setup_without(), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(cast(&state, &r), None);
}

/// Runners against imps, no heroes.
fn setup_without() -> BattleSetup {
    setup(&[(RUNNER, 10)], &[(IMP, 10)])
}

// covers: BAT-AI-017
#[test]
fn a_cast_not_due_now_waits_and_a_retreating_hero_casts_a_damage_spell() {
    // The archers act and can shoot; the slower gogs of the side can still act; the enemy gogs far
    // away threaten no one this turn: a bolt on them is not due now. Retreating, the hero casts its damage spell at
    // once (Haste, not a damage spell, is left out).
    let r = ruleset();
    let hero = hero_with(&r, &["bolt", "haste"], 1, 999);
    let mut state =
        BattleState::new(&r, &led(&[(ARCHER, 10), (GOG, 10)], &[(GOG, 2)], hero), 1).unwrap();
    for _ in 0..40 {
        if active(&state) == ARCHER {
            break;
        }
        play(&mut state, &r, Command::Defend);
    }
    let v = valued(&state, &r, "bolt");
    assert!(v.value > 0 && !v.now, "{v:?}");
    let calm = choice(&state, &r, false);
    assert!(!matches!(calm, Some(Command::Cast { spell, .. }) if spell == spell_of(&r, "bolt")));
    let fleeing = choice(&state, &r, true).unwrap();
    assert!(matches!(fleeing, Command::Cast { spell, .. } if spell == spell_of(&r, "bolt")));
    assert!(validate(&state, &r, ATTACKER, fleeing).is_ok());
}

/// The test spell `name`.
fn spell_of(r: &BattleRuleset, name: &str) -> h3_content::SpellId {
    spell(r, name)
}

// covers: BAT-AI-017
#[test]
fn with_the_enemy_beaten_only_the_raises_count() {
    // A lone imp the archers' shot kills: the predictions beat the enemy and the side loses
    // nothing, so only the raises are weighed, and the unhurt archers have nothing to raise: no
    // cast.
    let r = ruleset();
    let hero = hero_with(&r, &["bolt", "raise"], 1, 999);
    let mut state = BattleState::new(&r, &led(&[(ARCHER, 30)], &[(IMP, 1)], hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    let ctx = computer::context(&state, &r, ATTACKER);
    let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    assert!(sc.beaten() && !sc.at_risk());
    assert_eq!(cast(&state, &r), None);
    assert_eq!(
        valued(&state, &r, "raise"),
        Valued {
            value: 0,
            target: CastTarget::None,
            second: CastTarget::None,
            now: false,
        }
    );
}

// covers: BAT-AI-018
#[test]
fn the_damage_spells_and_the_summon() {
    let r = ruleset();
    let hero = hero_with(
        &r,
        &["fireball", "ripple", "chain", "summon", "firewall"],
        1,
        999,
    );
    let mut state = BattleState::new(
        &r,
        &led(&[(GOG, 10)], &[(IMP, 20), (IMP, 20), (RUNNER, 5)], hero),
        1,
    )
    .unwrap();
    until_side(&mut state, &r, ATTACKER);
    let targets = |name: &str| {
        let mut t = CastTargets::new();
        cast_targets(&state, &r, ATTACKER, spell(&r, name), &mut t);
        t
    };
    // The area spell: a hex the engine takes, not a side column, due now.
    let v = valued(&state, &r, "fireball");
    assert!(v.value > 0 && v.now, "{v:?}");
    let CastTarget::Hex(h) = v.target else {
        panic!("{v:?}")
    };
    assert!(h.0 % 17 != 0 && h.0 % 17 != 16);
    assert!(targets("fireball").as_slice().contains(&v.target));
    // The mass damage: the enemies lose more than the gogs, no target.
    let v = valued(&state, &r, "ripple");
    assert!(
        v.value > 0 && v.now && v.target == CastTarget::None,
        "{v:?}"
    );
    // The chain: from an enemy stack.
    let v = valued(&state, &r, "chain");
    assert!(v.value > 0 && v.now, "{v:?}");
    let CastTarget::Stack(first) = v.target else {
        panic!("{v:?}")
    };
    assert_eq!(state.stacks()[first.index()].side, DEFENDER);
    // The summon: 2 × power 5 runners at the runner's AI value (126).
    let v = valued(&state, &r, "summon");
    assert_eq!(
        v,
        Valued {
            value: 10 * 126,
            target: CastTarget::None,
            second: CastTarget::None,
            now: true,
        }
    );
    // A wall of fire is never cast.
    assert_eq!(valued(&state, &r, "firewall").value, 0);
    // The choice casts one of them, accepted.
    let c = cast(&state, &r).unwrap();
    assert!(validate(&state, &r, ATTACKER, c).is_ok(), "{c:?}");
}

// covers: BAT-AI-018
#[test]
fn a_summon_is_worth_nothing_once_the_enemy_is_beaten() {
    let r = ruleset();
    let hero = hero_with(&r, &["summon"], 1, 999);
    let mut state = BattleState::new(&r, &led(&[(ARCHER, 30)], &[(IMP, 1)], hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    let ctx = computer::context(&state, &r, ATTACKER);
    let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    let h = Hero::new(&sc, (&state, &r), (state.active().unwrap(), ATTACKER, ctx));
    let id = spell(&r, "summon");
    let c = candidate(&state, &r, "summon");
    assert_eq!(hero::summon(&h, r.spell(id).unwrap(), c).value, 0);
    // In the desperate mode 1000 a creature.
    let mut sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    sc.desperately();
    let v = valued_in(&state, &r, &sc, "summon");
    assert_eq!(v.value, 0, "beaten: {v:?}");
}

/// The value of `name` in the spell context `sc`.
fn valued_in(state: &BattleState, r: &BattleRuleset, sc: &SpellContext<'_>, name: &str) -> Valued {
    let ctx = computer::context(state, r, ATTACKER);
    let h = Hero::new(sc, (state, r), (state.active().unwrap(), ATTACKER, ctx));
    let id = spell(r, name);
    hero::value(&h, (id, r.spell(id).unwrap()), candidate(state, r, name))
}

// covers: BAT-AI-018
#[test]
fn a_desperate_summon_counts_1000_a_creature() {
    let r = ruleset();
    let hero = hero_with(&r, &["summon"], 1, 999);
    let mut state = BattleState::new(&r, &led(&[(RUNNER, 10)], &[(GOG, 10)], hero), 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    let ctx = computer::context(&state, &r, ATTACKER);
    let mut sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    sc.desperately();
    assert_eq!(valued_in(&state, &r, &sc, "summon").value, 10 * 1000);
}

// covers: BAT-AI-017
#[test]
fn dispel_by_its_mastery() {
    // The enemy hero blessed its runners, which reach the side within a turn; the attacker's hero
    // dispels. At Basic only its own side counts (nothing to remove there); at Advanced the
    // enemy's blessed runners; at Expert, the hero also blessed the side's runners (their removal
    // counts against Dispel): the original's stacks' case overwrites the two sums with the enemy's
    // alone (survey slip 27), without the slip both count.
    let battle = |r: &BattleRuleset, mastery: u8, blesses: &[BattleSide]| {
        let mut hero = hero_with(r, &["dispel", "bless"], mastery, 999);
        // Bless at Basic, aimed at one stack.
        hero.magic.masteries[1].1 = 1;
        let mut setup = led(&[(RUNNER, 10)], &[(RUNNER, 20)], hero);
        setup.heroes[1] = Some(hero_with(r, &["bless"], 1, 999));
        let mut state = BattleState::new(r, &setup, 1).unwrap();
        for &side in blesses {
            let i = state.stacks().iter().position(|s| s.side == side).unwrap();
            let t = StackId(u8::try_from(i).unwrap());
            cast_at(&mut state, r, side, "bless", t);
        }
        until_side(&mut state, r, ATTACKER);
        state
    };
    let blessed = [DEFENDER];
    let r = ruleset();
    let state = battle(&r, 1, &blessed);
    assert_eq!(valued(&state, &r, "dispel").value, 0);
    let state = battle(&r, 2, &blessed);
    let advanced = valued(&state, &r, "dispel");
    assert!(advanced.value > 0, "{advanced:?}");
    let runners = state
        .stacks()
        .iter()
        .position(|s| s.side == DEFENDER)
        .unwrap();
    let runners = StackId(u8::try_from(runners).unwrap());
    assert_eq!(advanced.target, CastTarget::Stack(runners));
    // Nothing on either side: nothing at Advanced either.
    assert_eq!(valued(&battle(&r, 2, &[]), &r, "dispel").value, 0);
    // At Expert: each side's sum of the evaluator over its stacks.
    let both = [DEFENDER, ATTACKER];
    let expert = |r: &BattleRuleset| {
        let state = battle(r, 3, &both);
        let ctx = computer::context(&state, r, ATTACKER);
        let sc = SpellContext::new(&state, r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
        let h = Hero::new(&sc, (&state, r), (state.active().unwrap(), ATTACKER, ctx));
        let c = candidate(&state, r, "dispel");
        let v = valued(&state, r, "dispel");
        assert_eq!(v.target, CastTarget::None);
        (
            v.value,
            hero::on_stacks(&h, ATTACKER, c),
            hero::on_stacks(&h, DEFENDER, c),
        )
    };
    let (v, own, theirs) = expert(&r);
    assert!(own < 0 && theirs > 0, "{own} {theirs}");
    assert_eq!(v, theirs);
    let (v, own, theirs) = expert(&ruleset_as("t:fixed"));
    assert!(own < 0, "{own}");
    assert_eq!(v, own + theirs);
}

// covers: BAT-AI-017
#[test]
fn magic_mirror_halves_a_bolt_on_its_stack_and_now_follows_the_order() {
    let r = ruleset();
    let mut setup = led(
        &[(RUNNER, 10), (ARCHER, 10)],
        &[(IMP, 20), (IMP, 20)],
        hero_with(&r, &["bolt"], 1, 999),
    );
    setup.heroes[1] = Some(hero_with(&r, &["mirror"], 1, 999));
    let mut state = BattleState::new(&r, &setup, 1).unwrap();
    let first = StackId(
        u8::try_from(
            state
                .stacks()
                .iter()
                .position(|s| s.side == DEFENDER)
                .unwrap(),
        )
        .unwrap(),
    );
    cast_at(&mut state, &r, DEFENDER, "mirror", first);
    until_side(&mut state, &r, ATTACKER);
    let ctx = computer::context(&state, &r, ATTACKER);
    let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    let h = Hero::new(&sc, (&state, &r), (state.active().unwrap(), ATTACKER, ctx));
    let c = candidate(&state, &r, "bolt");
    let v = hero::one_target(&h, DEFENDER, c, &h.targets(c.spell));
    // The mirrored imps are worth 2 × (50 − 20) % of their value: the other imps are the target.
    assert_ne!(v.target, CastTarget::Stack(first), "{v:?}");
    let chance = i32::from(computer::cast_mirror_chance(&state, &r, first));
    assert!(chance > 0);
    // "Now" on an enemy: the desperate mode always.
    let mut sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(5));
    sc.desperately();
    let h = Hero::new(&sc, (&state, &r), (state.active().unwrap(), ATTACKER, ctx));
    assert!(hero::now_on_enemy(&h, first));
}

// covers: BAT-AI-017
#[test]
fn the_buffs_on_the_side_and_clone() {
    // Haste is always due; a protection is due though other stacks can act; Clone values the
    // active stack's strike on its enemy within a turn.
    let r = ruleset();
    let hero = hero_with(&r, &["haste", "shield", "clone", "slow"], 1, 999);
    let mut state = BattleState::new(
        &r,
        &led(&[(RUNNER, 10), (ZOMBIE, 10)], &[(RUNNER, 10)], hero),
        1,
    )
    .unwrap();
    until_side(&mut state, &r, ATTACKER);
    for name in ["haste", "shield", "clone", "slow"] {
        let v = valued(&state, &r, name);
        assert!(v.value >= 0, "{name} {v:?}");
        if v.value > 0 && name != "slow" {
            assert!(v.now, "{name} {v:?}");
        }
    }
    let c = cast(&state, &r);
    if let Some(c) = c {
        assert!(validate(&state, &r, ATTACKER, c).is_ok(), "{c:?}");
    }
}
