//! Eagle Eye in battle (E04b T8d, BAT-MAG-069): at each cast of the enemy hero the hero with
//! Eagle Eye may pick the spell, after the original's code (`docs/formats/installation.md`
//! §7.24); the outcome reports the picks (who learns them is the orchestrator's, BAT-Q24).

use std::vec;
use std::vec::Vec;

use core::num::NonZeroU32;

use h3_content::SpellId;
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleHero, BattleResult, BattleSide, EagleEye, Rate};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::magic::HeroMagic;
use crate::{BattleState, HeroError, SetupError};

/// A mage with the bolt and an Eagle Eye of `chance_bp` for `spells`.
fn watcher(r: &BattleRuleset, chance_bp: u32, spells: &[&str]) -> BattleHero {
    let mut h = mage(r, &["bolt"]);
    h.magic.eagle_eye = Some(EagleEye {
        chance: Rate::bp(chance_bp),
        spells: spells.iter().map(|s| spell(r, s)).collect(),
    });
    h
}

/// A battle of two mages, the attacker's stack active: `heroes`, one stack each.
fn duel(r: &BattleRuleset, heroes: [Option<BattleHero>; 2], seed: u64) -> BattleState {
    arena(
        r,
        heroes,
        &[("fixed", 10, 5, 5)],
        &[("fixed", 10, 9, 5)],
        seed,
    )
}

/// The spells the hero of `side` picked so far.
fn picked(s: &BattleState, side: BattleSide) -> Vec<SpellId> {
    s.heroes[side.index()]
        .as_ref()
        .unwrap()
        .magic
        .learned
        .iter()
        .collect()
}

// covers: BAT-MAG-069
#[test]
fn eagle_eye_picks_the_enemy_heros_spell_by_its_chance() {
    let r = ruleset();
    let bolt = spell(&r, "bolt");
    // Certain: the defender picks the attacker's bolt; the attacker's own Eagle Eye never picks
    // its own hero's spells.
    let mut s = duel(
        &r,
        [
            Some(watcher(&r, 10_000, &["bolt"])),
            Some(watcher(&r, 10_000, &["bolt"])),
        ],
        1,
    );
    act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    assert_eq!(picked(&s, BattleSide::Defender), [bolt]);
    assert!(picked(&s, BattleSide::Attacker).is_empty());
    // Never at 0 %, nor for a spell it may not learn (learned, or above its level).
    for eye in [watcher(&r, 0, &["bolt"]), watcher(&r, 10_000, &["sloth"])] {
        let mut s = duel(&r, [Some(mage(&r, &["bolt"])), Some(eye)], 1);
        act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
        assert!(picked(&s, BattleSide::Defender).is_empty());
    }
    // At 50 % one draw from 1 to 100, the cast's first: picked if it is at most 50.
    let mut outcomes = [0; 2];
    for seed in 0..40 {
        let mut s = duel(
            &r,
            [
                Some(mage(&r, &["bolt"])),
                Some(watcher(&r, 5000, &["bolt"])),
            ],
            seed,
        );
        let mut rng = s.rng.clone();
        let expected = rng.below(100) < 50;
        act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
        assert_eq!(
            picked(&s, BattleSide::Defender) == [bolt],
            expected,
            "{seed}"
        );
        outcomes[usize::from(expected)] += 1;
    }
    assert!(outcomes[0] > 0 && outcomes[1] > 0, "{outcomes:?}");
}

// covers: BAT-MAG-069
#[test]
fn opening_spells_give_eagle_eye_nothing() {
    let r = ruleset();
    let mut opener = mage(&r, &["bolt"]);
    opener.effects = vec![BattleEffect::OpeningSpell {
        spell: spell(&r, "prayer"),
        mastery: 3,
        power: 10,
    }];
    let s = duel(
        &r,
        [Some(opener), Some(watcher(&r, 10_000, &["prayer"]))],
        1,
    );
    assert!(s.effects().iter().any(|e| e.spell == spell(&r, "prayer")));
    assert!(picked(&s, BattleSide::Defender).is_empty());
}

// covers: BAT-MAG-069, BAT-END-003
#[test]
fn the_outcome_reports_the_spells_eagle_eye_picked() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [
            Some(mage(&r, &["bolt"])),
            Some(watcher(&r, 10_000, &["bolt"])),
        ],
        &[("fixed", 10, 5, 5)],
        &[("fixed", 1, 9, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    let [attacker, defender] = outcome.heroes.map(Option::unwrap);
    assert!(attacker.learned.is_empty());
    assert_eq!(defender.learned, [spell(&r, "bolt")]);
}

// covers: BAT-MAG-069
#[test]
fn the_chance_is_truncated_to_whole_percent() {
    let r = ruleset();
    let chance = |rate: Rate, single: bool| {
        let mut h = watcher(&r, 0, &["bolt"]);
        h.magic.eagle_eye.as_mut().unwrap().chance = rate;
        HeroMagic::of(&h, single).unwrap().eagle_chance
    };
    // 44 % is 0.4399999976… in single precision (a Basic Eagle Eye specialist at level 2 in the
    // original, `installation.md` §7.24): 43 % with quirk Q-0005; a value one unit above it 44 %.
    assert_eq!(chance(Rate::bp(4400), false), 44);
    assert_eq!(chance(Rate::bp(4400), true), 43);
    let above = Rate {
        bp: 4400,
        single: NonZeroU32::new(0x3ee1_47af),
    };
    assert_eq!(chance(above, true), 44);
    assert_eq!(chance(Rate::bp(10_000), true), 100);
    assert_eq!(chance(Rate::bp(4999), false), 49);
}

// covers: BAT-HER-001, BAT-MAG-069
#[test]
fn the_battle_checks_eagle_eye() {
    let r = ruleset();
    let refused = |change: &dyn Fn(&mut EagleEye)| {
        let mut h = watcher(&r, 4000, &["bolt"]);
        change(h.magic.eagle_eye.as_mut().unwrap());
        let setup = h3_contracts::BattleSetup {
            heroes: [None, Some(h)],
            ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        BattleState::new(&r, &setup, 1).err().map(|e| match e {
            SetupError::InvalidHero { error, .. } => error,
            other => panic!("{other:?}"),
        })
    };
    assert_eq!(refused(&|_| {}), None);
    assert_eq!(
        refused(&|e| e.chance = Rate::bp(10_001)),
        Some(HeroError::EagleEye)
    );
    assert_eq!(
        refused(&|e| e.chance.single = NonZeroU32::new(0x3f80_0001)),
        Some(HeroError::EagleEye)
    );
    assert_eq!(
        refused(&|e| e.spells.push(SpellId::from_raw(999))),
        Some(HeroError::UnknownSpell)
    );
    assert_eq!(
        refused(&|e| e.spells.push(e.spells[0])),
        Some(HeroError::DuplicateSpell)
    );
    let gaze = spell(&r, "gaze");
    assert_eq!(refused(&|e| e.spells.push(gaze)), Some(HeroError::EagleEye));
}

// covers: BAT-MAG-069
#[test]
fn creature_casts_give_eagle_eye_nothing() {
    // The attacker's Enchanter casts at its first activation (BAT-MAG-053): the defender's Eagle
    // Eye, certain for every spell it may cast, picks none.
    let r = ruleset();
    let spells = [
        "haste",
        "sloth",
        "skin",
        "bless",
        "weaken",
        "lust",
        "airshield",
        "cure",
    ];
    let setup = h3_contracts::BattleSetup {
        heroes: [None, Some(watcher(&r, 10_000, &spells))],
        ..setup(
            &r,
            &[(0, "chanter", 5), (1, "fixed", 3)],
            &[(0, "fixed", 4)],
        )
    };
    let mut events = Vec::new();
    let s = BattleState::start(&r, &setup, 3, &mut events).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, crate::EventKind::PeriodicCast { .. }))
    );
    assert!(picked(&s, BattleSide::Defender).is_empty());
}
