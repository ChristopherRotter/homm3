//! A search agent's worlds (E06 T8, ADR-0025 point 12a): `BattleState::determinized` reads
//! nothing the viewer may not know, and its viewer sees the battle in it as it is.

use std::vec::Vec;

use h3_content_battle::{BattleRuleset, ObstacleKind};
use h3_contracts::{BattleEffect, BattleHero, BattleSide};
use h3_kernel::{FixedVec, Rng, StreamId};

use super::spells::{arena, mage, spell};
use super::support::*;
use crate::{BattleState, Command, Hex, SetupError, SpellSet, castable_spells, observe};

/// Places `side`'s Quicksand pits on `cells` as its cast would (its hexes are drawn there).
fn pits(s: &mut BattleState, r: &BattleRuleset, side: BattleSide, cells: &[(i32, i32)]) {
    let id = spell(r, "quicksand");
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let hexes: Vec<Hex> = cells.iter().map(|&(x, y)| at(r, x, y)).collect();
    crate::obstacles::place(
        s,
        (id, ObstacleKind::Pit, &shape),
        side,
        (&hexes, 0),
        &mut Vec::new(),
    );
}

/// A battle of two mages, the defender's pits on `hidden` (which the attacker does not see),
/// then the attacker's own; the defender knows `book`.
fn battle(r: &BattleRuleset, hidden: &[(i32, i32)], book: &[&str]) -> BattleState {
    let mut s = arena(
        r,
        [Some(mage(r, &["quicksand"])), Some(mage(r, book))],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        7,
    );
    pits(&mut s, r, BattleSide::Defender, hidden);
    pits(&mut s, r, BattleSide::Attacker, &[(6, 3)]);
    s
}

/// The world of `s` for the attacker from `guess` and `seed`.
fn world(s: &BattleState, r: &BattleRuleset, guess: &BattleHero, seed: u64) -> BattleState {
    s.determinized(r, BattleSide::Attacker, Some(guess), seed)
        .unwrap()
}

// covers: BAT-MAG-031
#[test]
fn a_world_reads_nothing_the_viewer_may_not_know() {
    let r = ruleset();
    let guess = mage(&r, &["bolt"]);
    let s = battle(&r, &[(9, 2), (9, 8)], &["quicksand", "sprites"]);
    // A twin that differs in what the attacker may not know: the stream, the defender's hidden
    // pits' hexes, its hero's private part and the templates its book gives.
    let mut twin = battle(&r, &[(10, 1), (8, 9)], &["haste"]);
    twin.rng = Rng::for_stream(99, StreamId::BATTLE);
    let h = twin.heroes[1].as_mut().unwrap();
    (h.tactics, h.funds, h.surrender_discount_bp) = (3, 5_000, 2_000);
    h.magic.book = SpellSet::default();
    twin.created
        .set_templates(BattleSide::Defender, FixedVec::new());
    assert_ne!(s.state_hash(), twin.state_hash());
    for seed in [1, 2] {
        let a = world(&s, &r, &guess, seed);
        assert_eq!(a.state_hash(), world(&twin, &r, &guess, seed).state_hash());
        // The attacker sees the battle in the world as it is.
        assert_eq!(
            observe(&a, &r, BattleSide::Attacker),
            observe(&s, &r, BattleSide::Attacker)
        );
        // The defender's book is the guess's, its pits as many, elsewhere for another seed.
        let own = observe(&a, &r, BattleSide::Defender).own_hero.unwrap();
        let mut bolt = SpellSet::default();
        bolt.insert(spell(&r, "bolt"));
        assert_eq!(own.spell_book, bolt);
        assert_eq!(a.spell_obstacles().len(), 3);
    }
    let (one, two) = (world(&s, &r, &guess, 1), world(&s, &r, &guess, 2));
    assert_ne!(one.state_hash(), two.state_hash());
    // The attacker's own pit stays where it is.
    for w in [&one, &two] {
        let own: Vec<_> = w
            .spell_obstacles()
            .iter()
            .filter(|o| o.side == BattleSide::Attacker)
            .map(|o| o.hexes)
            .collect();
        assert_eq!(own.len(), 1);
        assert!(own[0].contains(at(&r, 6, 3)));
    }
}

// covers: BAT-MAG-061, BAT-MAG-062
#[test]
fn a_world_keeps_what_the_other_hero_forbids_every_caster() {
    // The defender wears an Orb of Inhibition or Recanter's Cloak, the guess neither: the
    // attacker meets either in its own casts, so its world keeps it (AI-STUB-1's magic limits,
    // found by `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`).
    let r = ruleset();
    for effect in [
        BattleEffect::NoCasting,
        BattleEffect::SpellLevelLimit { max_level: 2 },
    ] {
        let mut wearer = mage(&r, &["haste"]);
        wearer.effects.push(effect);
        let s = arena(
            &r,
            [Some(mage(&r, &["bolt"])), Some(wearer)],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 12, 5)],
            7,
        );
        let w = world(&s, &r, &mage(&r, &["bolt"]), 1);
        assert_eq!(w.limits, s.limits);
        assert_eq!(
            castable_spells(&w, &r, BattleSide::Attacker),
            castable_spells(&s, &r, BattleSide::Attacker)
        );
        assert_eq!(
            observe(&w, &r, BattleSide::Attacker),
            observe(&s, &r, BattleSide::Attacker)
        );
    }
    // Whether a stack's guard is looked at follows the guessed hero's private defences.
    let s = battle(&r, &[], &["haste"]);
    let mut guarded = mage(&r, &["bolt"]);
    guarded
        .effects
        .push(BattleEffect::SpellLevelImmunity { max_level: 1 });
    assert!(!s.limits.guards);
    assert!(world(&s, &r, &guarded, 1).limits.guards);
    assert!(!world(&s, &r, &mage(&r, &["bolt"]), 1).limits.guards);
}

// covers: BAT-MAG-031
#[test]
fn a_world_plays_on_and_reseeding_draws_other_rolls() {
    let r = ruleset();
    let s = battle(&r, &[(9, 2)], &["quicksand"]);
    let guess = BattleHero::new(s.heroes[1].unwrap().view.hero_type);
    let mut a = world(&s, &r, &guess, 3);
    // A re-seed replaces the stream alone: the same seed the same state, another another.
    let reseeded = |seed| {
        let mut w = a.clone();
        w.reseed(seed);
        w.state_hash()
    };
    assert_eq!(reseeded(4), reseeded(4));
    assert_ne!(reseeded(4), reseeded(5));
    assert_ne!(reseeded(4), a.state_hash());
    for _ in 0..6 {
        act(&mut a, &r, Command::Defend).unwrap();
    }
    assert!(a.round() > 1);
    // Without a guess the other hero is one of its type alone: no book where it carries one
    // is still a book, empty.
    let bare = s.determinized(&r, BattleSide::Attacker, None, 3).unwrap();
    let own = observe(&bare, &r, BattleSide::Defender).own_hero.unwrap();
    assert_eq!(own.spell_book, SpellSet::default());
    assert!(
        observe(&bare, &r, BattleSide::Defender).heroes[1]
            .unwrap()
            .spell_book
    );
    // A hero without a book has none in the world, whatever the guess's.
    let mut bookless = hero(&r);
    bookless.magic.spell_book = None;
    let plain = arena(
        &r,
        [Some(mage(&r, &["quicksand"])), Some(bookless)],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        7,
    );
    let w = world(&plain, &r, &mage(&r, &["bolt"]), 3);
    let seen = observe(&w, &r, BattleSide::Defender);
    assert!(!seen.heroes[1].unwrap().spell_book);
    assert_eq!(seen.own_hero.unwrap().spell_book, SpellSet::default());
    // A guess the battle refuses is an error (BAT-HER-001).
    let mut bad = guess;
    bad.tactics = 9;
    assert!(matches!(
        s.determinized(&r, BattleSide::Attacker, Some(&bad), 3),
        Err(SetupError::InvalidHero { .. })
    ));
}
