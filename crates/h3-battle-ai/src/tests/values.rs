//! The spell context's evaluators (0x43B690's table; E23 T4c; BAT-AI-016) by hand on the test
//! ruleset, and the Master Genie's mean. The Genie's choices on the harvested packs:
//! `h3-pack-battle`'s `harvested_computer_casts.rs`.

use h3_battle::{BattleSetup, BattleState, StackId, computer};
use h3_content::Affects;
use h3_contracts::{BattleEffect, BattleSide};
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use super::{
    ARCHER, BALLISTA, GENIE, GIANT, IMP, KNIGHT, LORD, RUNNER, active, creature, hero, play,
    ruleset, setup, spell,
};
use crate::casts::balance;
use crate::spells::boosts::change_value;
use crate::spells::{Candidate, SpellContext};
use crate::walker::Walker;

const ATTACKER: BattleSide = BattleSide::Attacker;

/// Ten runners and ten archers of the attacker, whose hero gives them `luck` and `morale`, against
/// two of `king`.
fn kings(king: usize, (luck, morale): (i8, i8)) -> BattleSetup {
    with_hero(&[(RUNNER, 10), (ARCHER, 10)], king, (luck, morale))
}

/// The attacker's `stacks`, whose hero gives them `luck` and `morale`, against two of `king`.
fn with_hero(stacks: &[(usize, u32)], king: usize, (luck, morale): (i8, i8)) -> BattleSetup {
    let mut setup = setup(stacks, &[(king, 2)]);
    let mut h = hero(0);
    h.effects.push(BattleEffect::Luck {
        amount: luck,
        affects: Affects::Own,
    });
    h.effects.push(BattleEffect::Morale {
        amount: morale,
        affects: Affects::Own,
    });
    setup.heroes[0] = Some(h);
    setup
}

/// The attacker's spell context in `state`.
fn context<'a>(
    state: &'a BattleState,
    r: &'a h3_content_battle::BattleRuleset,
) -> SpellContext<'a> {
    let ctx = computer::context(state, r, ATTACKER);
    SpellContext::new(state, r, (&ctx, ATTACKER), &mut Rng::from_seed(0))
}

const RUNNERS: StackId = StackId(0);
const ARCHERS: StackId = StackId(1);

// covers: BAT-AI-014, BAT-AI-016
#[test]
fn the_attack_bonuses_by_the_strikes_they_raise() {
    // The runners (attack 5) strike the giants (defense 16; the class 3, the strength 3 000 each):
    // a base of 100 does 72, with 3 attack more 80, with 8 more 92; their strike is 21. The
    // archers shoot: 37, 41 with 3 more, 45 with 6, 47 with 8; their shot is 9.
    let r = ruleset();
    let state = BattleState::new(&r, &kings(GIANT, (0, 0)), 1).unwrap();
    let ctx = computer::context(&state, &r, ATTACKER);
    assert_eq!(balance(&state, &r, &ctx, ATTACKER).class, 3);
    let base = |s, b| computer::base_damage(&state, &r, (s, StackId(2)), (100, s == ARCHERS), b);
    assert_eq!([0, 3, 8].map(|b| base(RUNNERS, b)), [72, 80, 92]);
    assert_eq!([0, 3, 6, 8].map(|b| base(ARCHERS, b)), [37, 41, 45, 47]);
    let sc = context(&state, &r);
    let value = |name, mastery, s| sc.value(s, Candidate::cast(spell(&r, name), mastery, 6));
    // Bloodlust (3): 21 × 80 / 72 = 23.3, so trunc((√(80/72) − 1) × 3 000) = 162; nothing for the
    // archers, which shoot.
    assert_eq!(
        (
            value("bloodlust", 0, RUNNERS),
            value("bloodlust", 0, ARCHERS)
        ),
        (162, 0)
    );
    // Precision: nothing for the walkers; at Basic (3) the archers' shot rises to 9.97, not above
    // 9; at Advanced (6) to 10.9: trunc((√(45/37) − 1) × 3 000) = 308.
    assert_eq!(value("precision", 3, RUNNERS), 0);
    assert_eq!(value("precision", 0, ARCHERS), 0);
    assert_eq!(value("precision", 2, ARCHERS), 308);
    // Slayer (8) against a king 1 at any mastery: 391 and 381.
    assert_eq!(value("slayer", 0, RUNNERS), 391);
    assert_eq!(value("slayer", 0, ARCHERS), 381);
    // Against a king 2 from Advanced, a king 3 at Expert.
    for (king, from) in [(KNIGHT, 2), (LORD, 3)] {
        let state = BattleState::new(&r, &kings(king, (0, 0)), 1).unwrap();
        let sc = context(&state, &r);
        for mastery in 0..4 {
            let v = sc.value(RUNNERS, Candidate::cast(spell(&r, "slayer"), mastery, 6));
            assert_eq!(v, if mastery >= from { 391 } else { 0 }, "{king} {mastery}");
        }
    }
    // Nothing beyond a turn: the imps (speed 5) reach the giants in three.
    let state = BattleState::new(&r, &setup(&[(IMP, 10)], &[(GIANT, 2)]), 1).unwrap();
    let sc = context(&state, &r);
    for name in ["bloodlust", "slayer", "bless", "fortune"] {
        assert_eq!(
            sc.value(StackId(0), Candidate::cast(spell(&r, name), 3, 6)),
            0
        );
    }
}

// covers: BAT-AI-016
#[test]
fn bless_raises_the_damage_to_the_maximum_and_its_bonus() {
    // The runners' damage 2 to 4 (3.0 a creature) becomes 4 + 1 at Advanced: g = 5 / 3, their
    // strike 21 rises to 35; trunc((√(5/3) − 1) × 3 000) = 872. The archers' 2.5 becomes 4: 794.
    // At Basic (0 more) the runners' strike rises to 28 (g = 4 / 3): 464.
    let r = ruleset();
    let state = BattleState::new(&r, &kings(GIANT, (0, 0)), 1).unwrap();
    assert_eq!(
        computer::damage_per_creature(&state, &r, RUNNERS),
        X87::int(3)
    );
    assert_eq!(computer::maximum_damage(&state, &r, RUNNERS), 4);
    let sc = context(&state, &r);
    let bless = |mastery, s| sc.value(s, Candidate::cast(spell(&r, "bless"), mastery, 6));
    assert_eq!((bless(2, RUNNERS), bless(2, ARCHERS)), (872, 794));
    assert_eq!(bless(0, RUNNERS), 464);
}

// covers: BAT-AI-016
#[test]
fn a_morale_or_luck_change_with_and_without_the_slip() {
    let (p, n) = (
        X87::ratio(173, 10_000, 53).unwrap(),
        X87::ratio(833, 10_000, 53).unwrap(),
    );
    let micro = |m, c, slip| {
        let x = change_value(m, c, (p, n), slip);
        x.ftol_times(X87::int(1_000_000))
    };
    // A rise from −3 by 1: with the slip valued as crossing 0, −2 × 0.0173 + 3 × 0.0833; as the
    // rule, 0.0833.
    assert_eq!(micro(-3, 1, true), 215_300);
    assert_eq!(micro(-3, 1, false), 83_300);
    // A rise that crosses 0 (from −1 by 2): 0.0173 + 0.0833 either way.
    assert_eq!(micro(-1, 2, true), 100_600);
    assert_eq!(micro(-1, 2, false), 100_600);
    // Above 0, cut at 3: from 1 by 5 counts 2; nothing at 3.
    assert_eq!(micro(1, 5, false), 34_600);
    assert_eq!(micro(3, 1, false), 0);
    // Falls: below 0 each point costs 0.0833, cut at −3; above 0 0.0173; across 0 both; nothing
    // at −3; a change of 0 is worth nothing.
    assert_eq!(micro(-1, -1, true), -83_300);
    assert_eq!(micro(-2, -3, true), -83_300);
    assert_eq!(micro(2, -1, true), -17_300);
    assert_eq!(micro(1, -3, true), -183_900);
    assert_eq!(micro(-3, -1, true), 0);
    assert!(change_value(0, 0, (p, n), true).is_zero());
}

// covers: BAT-AI-016
#[test]
fn mirth_values_the_morale_it_adds() {
    let r = ruleset();
    // Morale 1 (one alignment) by 1: 0.0173 × 3 000 = 51.
    let state = BattleState::new(&r, &kings(GIANT, (0, 0)), 1).unwrap();
    assert_eq!(state.stacks()[0].morale, 1);
    let sc = context(&state, &r);
    let mirth = Candidate::cast(spell(&r, "mirth"), 0, 6);
    assert_eq!(sc.value(RUNNERS, mirth), 51);
    // A change of 0 (Bless's value at Basic as the change): nothing.
    assert_eq!(
        sc.mirth(RUNNERS, Candidate::cast(spell(&r, "bless"), 0, 6)),
        0
    );
    // At morale 3 nothing more.
    let state = BattleState::new(&r, &kings(GIANT, (0, 2)), 1).unwrap();
    assert_eq!(context(&state, &r).value(RUNNERS, mirth), 0);
    // A war machine has no morale.
    let mut s = kings(GIANT, (0, 0));
    s.war_machines[0] = std::vec![creature(BALLISTA)];
    let state = BattleState::new(&r, &s, 1).unwrap();
    let ballista = StackId(2);
    assert!(state.stacks()[ballista.index()].has(h3_battle::StackTraits::WAR_MACHINE));
    assert_eq!(context(&state, &r).value(ballista, mirth), 0);
}

// covers: BAT-AI-016
#[test]
fn fortune_values_the_luck_points_it_adds_and_removes() {
    // The runners' strike 21 on the giants: a doubled strike gains trunc((√2 − 1) × 3 000) =
    // 1 242, a halved one (10) doubled the same; each point is one chance in 24.
    let r = ruleset();
    let fortune = |luck, mastery| {
        let state = BattleState::new(&r, &kings(GIANT, (luck, 0)), 1).unwrap();
        assert_eq!(computer::luck(&state, &r, RUNNERS), i32::from(luck));
        context(&state, &r).value(RUNNERS, Candidate::cast(spell(&r, "fortune"), mastery, 6))
    };
    // Luck 0 by 1: a good point, 1 242 / 24 = 51; luck 2 by 2 counts one point.
    assert_eq!(fortune(0, 0), 51);
    assert_eq!(fortune(2, 2), 51);
    // Luck −2 by 1 or 2: bad points removed, 51 and 103.
    assert_eq!(fortune(-2, 0), 51);
    assert_eq!(fortune(-2, 2), 103);
    // Luck −1 by 2: one bad point removed and one good point, 51 + 51.
    assert_eq!(fortune(-1, 2), 102);
    // Nothing at luck 3 or when the luck stays at −3 or below.
    assert_eq!(fortune(3, 2), 0);
    assert_eq!(fortune(-4, 0), 0);
    // A strike that, doubled, would take more than the giants' 300 hit points: 160 runners strike
    // for 348 uncapped, cut to 300 (g = 300 / 348): no gain.
    let mut setup = kings(GIANT, (0, 0));
    setup.attacker.stacks[0].count = 160;
    let state = BattleState::new(&r, &setup, 1).unwrap();
    let count = u64::from(state.stacks()[0].count);
    assert_eq!(
        computer::uncapped_damage(&state, &r, (RUNNERS, StackId(2)), count, false, 0),
        348
    );
    assert_eq!(
        context(&state, &r).value(RUNNERS, Candidate::cast(spell(&r, "fortune"), 0, 6)),
        0
    );
}

// covers: BAT-AI-016
#[test]
fn the_genies_value_is_the_mean_of_the_spells_it_could_draw() {
    // A genie beside the runners and the archers (the class 3), its turn after the runners': on
    // both it could draw Fire Shield, Slayer, Bless, Mirth, Fortune, Shield (the giants strike in
    // melee), Stone Skin, Counterstrike, Frenzy and Haste at Advanced, Bloodlust on the runners,
    // Precision and Prayer on the archers (which have not acted); not Air Shield (no enemy
    // shoots) or the protections (the giants have no hero).
    let r = ruleset();
    let setup = with_hero(&[(RUNNER, 10), (ARCHER, 10), (GENIE, 1)], GIANT, (0, 0));
    let mut state = BattleState::new(&r, &setup, 1).unwrap();
    while active(&state) != GENIE {
        play(&mut state, &r, h3_battle::Command::Defend);
    }
    let genie = StackId(2);
    let w = Walker::new(&state, &r, (genie, ATTACKER));
    let sc = SpellContext::new(&state, &r, (&w.ctx, ATTACKER), &mut Rng::from_seed(0));
    let mean = |s: StackId, names: &[&str]| {
        let sum: i32 = names
            .iter()
            .map(|n| sc.value(s, Candidate::cast(spell(&r, n), 2, 6)))
            .sum();
        sum / i32::try_from(names.len()).unwrap()
    };
    let both = [
        "fireshield",
        "slayer",
        "bless",
        "mirth",
        "fortune",
        "shield",
    ];
    let more = ["stoneskin", "counterstrike", "frenzy", "haste"];
    let runners = mean(RUNNERS, &[&both[..], &["bloodlust"], &more[..]].concat());
    let archers = mean(
        ARCHERS,
        &[&both[..], &["precision", "prayer"], &more[..]].concat(),
    );
    assert!(runners > 0 && archers > 0);
    assert_eq!(crate::casts::genie(&w, &sc, (2, 6), RUNNERS), runners);
    assert_eq!(crate::casts::genie(&w, &sc, (2, 6), ARCHERS), archers);
    // Nothing it could draw for a giant: 0.
    assert_eq!(crate::casts::genie(&w, &sc, (2, 6), StackId(3)), 0);
}
