//! Tests of the dwellings' and the Refugee Camps' visits (E08 T95a1, T95a2; increment 10's design
//! note §7, §12, §13): an own dwelling's hire question, a neutral or another's unguarded one
//! flagged first, a guarded one's guards' question, their battle (unsplit, unowned, on the hero's
//! tile), its end (the survivors kept, or the flag and the hire question after the levels); a
//! camp's question or its "deserted"; an accepted question's free
//! creatures (into a stack of the type, the first free slot, refused without room, none here,
//! none of a level above `free_level`, none on another's dwelling or a camp); the dwellings'
//! writer. The hire dialog they open and its purchases: `hiring.rs` (T95b).

use super::dwellings::{Rules, mask, placed, rules, setup, type_of};
use super::*;
use crate::command::{BattleStart, Defender};
use crate::testing::{PEASANT, started};

pub(super) const RED: PlayerId = PlayerId::new(0);
pub(super) const BLUE: PlayerId = PlayerId::new(1);
pub(super) const HERO: HeroId = HeroId::new(1);
/// The objects' entrance in these tests, two steps right of red's hero at (0, 7).
pub(super) const ENTRANCE: Pos = Pos::new(2, 7, 0);

pub(super) fn creature(rules: &Rules, k: &str) -> CreatureTypeId {
    rules.0.creature_by_key(k).unwrap()
}

pub(super) fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// An object of type `k` at [`ENTRANCE`], one tile, blocked and entered, neutral or `owner`'s.
pub(super) fn at_entrance(rules: &Rules, k: &str, owner: Option<PlayerId>) -> ObjectSetup {
    ObjectSetup {
        owner,
        ..placed(
            rules,
            k,
            (ENTRANCE.x, ENTRANCE.y),
            mask(&[(0, 0)]),
            mask(&[(0, 0)]),
        )
    }
}

/// The game of [`setup`]'s grass map with `object` at [`ENTRANCE`], red's hero's army `army`.
pub(super) fn game(rules: &Rules, object: ObjectSetup, army: Vec<ArmyStack>) -> AdventureState {
    let mut setup = setup(vec![object]);
    setup.heroes[0].army = army;
    started(rules, &setup).unwrap()
}

/// Red's hero's army as the setup gives it: one peasant in slot 0.
pub(super) fn peasant() -> Vec<ArmyStack> {
    vec![stack(0, PEASANT, 1)]
}

/// Red's hero's walk onto [`ENTRANCE`]: the outcome and the events after the steps.
pub(super) fn visit(
    rules: &Rules,
    state: &mut AdventureState,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let walk = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![Pos::new(1, 7, 0), ENTRANCE],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &walk, &mut events)
        .unwrap();
    assert_eq!(state.hero(HERO).unwrap().pos, ENTRANCE);
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    (outcome, events)
}

/// The hire question of the object of type `k` at [`ENTRANCE`] to red's hero, naming these
/// creatures.
pub(super) fn hire(rules: &Rules, k: &str, creatures: &[&str]) -> Offer {
    Offer {
        hero: HERO,
        object_type: type_of(rules, k),
        anchor: ENTRANCE,
        kind: OfferKind::Hire {
            creatures: creatures.iter().map(|k| creature(rules, k)).collect(),
        },
    }
}

/// `offer` accepted by red: the outcome and the events.
pub(super) fn accept(
    rules: &Rules,
    state: &mut AdventureState,
    offer: &Offer,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let outcome = state.accept_offer((&rules.0, &rules.1, &[]), RED, offer, &mut events);
    (outcome, events)
}

/// `kind` told to red alone.
pub(super) fn to_red(kind: AdventureEventKind) -> AdventureEvent {
    AdventureEvent {
        kind,
        seen_by: Viewers::NONE.with(RED),
    }
}

// covers: ADV-DWL-004
#[test]
fn an_own_dwelling_asks_the_hire_question_without_a_flag_or_its_guards() {
    let rules = rules();
    // A level-1 dwelling: its two creatures in its kind's order, nothing told.
    let mut s = game(&rules, at_entrance(&rules, "t:d1", Some(RED)), peasant());
    let before = s.clone();
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(
        outcome,
        AdventureOutcome {
            offer: Some(hire(&rules, "t:d1", &["t:c1", "t:w1"])),
            ..AdventureOutcome::default()
        }
    );
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(s.owner(0), Some(RED));
    assert_eq!(s.dwelling_counts(0), before.dwelling_counts(0));
    // A guarded level-5 one of red's: no guards' question, the question at once, and the whole
    // outcome else default: no Leave Guards (E08 T97a's audit, criterion 4).
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), peasant());
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(
        outcome,
        AdventureOutcome {
            offer: Some(hire(&rules, "t:d5", &["t:c5", "t:w5"])),
            ..AdventureOutcome::default()
        }
    );
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(s.dwelling_guards(0).len(), 2);
}

// covers: ADV-DWL-004, ADV-OWN-001
#[test]
fn a_neutral_or_another_s_unguarded_dwelling_is_flagged_then_asks() {
    let rules = rules();
    let d1 = type_of(&rules, "t:d1");
    for owner in [None, Some(BLUE)] {
        let mut s = game(&rules, at_entrance(&rules, "t:d1", owner), peasant());
        let before = s.clone();
        let (outcome, events) = visit(&rules, &mut s);
        assert_eq!(outcome.offer, Some(hire(&rules, "t:d1", &["t:c1", "t:w1"])));
        // Flagged for red, told to red and, for its own, to blue; then the reveal around the
        // entrance, told to red alone.
        assert_eq!(s.owner(0), Some(RED));
        let flagged = AdventureEventKind::ObjectFlagged {
            object_type: d1,
            anchor: ENTRANCE,
            owner: RED,
        };
        let seen = owner.map_or(Viewers::NONE, |_| Viewers::NONE.with(BLUE));
        assert_eq!(events[0].kind, flagged, "{owner:?}");
        assert_eq!(events[0].seen_by, seen.with(RED), "{owner:?}");
        assert!(matches!(
            events[1].kind,
            AdventureEventKind::TilesRevealed { object_type, .. } if object_type == d1
        ));
        assert_eq!(
            (events[1].seen_by, events.len()),
            (Viewers::NONE.with(RED), 2)
        );
        assert!(s.check_owners(&rules.1));
        assert_ne!(s.state_hash(), before.state_hash());
        // Declined, the visit ends and the flag stays.
        let mut events = Vec::new();
        let offer = outcome.offer.unwrap();
        let declined = s.decline_offer((&rules.0, &rules.1, &[]), &offer, &mut events);
        assert_eq!(declined, AdventureOutcome::default());
        assert!(events.is_empty());
        assert_eq!(s.owner(0), Some(RED));
    }
}

/// The guards' question of the object of type `k` at [`ENTRANCE`] to red's hero, naming
/// `creature` and the word `count_word`.
fn guarded(rules: &Rules, k: &str, creature: &str, count_word: u8) -> Offer {
    Offer {
        hero: HERO,
        object_type: type_of(rules, k),
        anchor: ENTRANCE,
        kind: OfferKind::Guarded {
            creature: self::creature(rules, creature),
            count_word,
        },
    }
}

// covers: ADV-DWL-004
#[test]
fn a_guarded_dwelling_not_of_the_visitor_s_team_asks_the_guards_question() {
    let rules = rules();
    let parts = (&rules.0, &rules.1, &[][..]);
    for owner in [None, Some(BLUE)] {
        // Its full guards, 6 `t:c5` in slot 0 and 6 `t:w5` in slot 1: the first slot's creature
        // and the word of the total, 12 (the third word; the first stack's 6 is the second).
        let mut s = game(&rules, at_entrance(&rules, "t:d5", owner), peasant());
        let before = s.clone();
        let (outcome, events) = visit(&rules, &mut s);
        let ask = guarded(&rules, "t:d5", "t:c5", 2);
        assert_eq!(
            outcome,
            AdventureOutcome {
                offer: Some(ask.clone()),
                ..AdventureOutcome::default()
            },
            "{owner:?}"
        );
        assert!(events.is_empty(), "{events:?}");
        assert_eq!(s.owner(0), owner);
        assert!(s.offer_stands(&ask));
        let asked = s.clone();
        // Declined: nothing, the hero stays on the entrance.
        let mut events = Vec::new();
        let declined = s.decline_offer(parts, &ask, &mut events);
        assert_eq!(declined, AdventureOutcome::default());
        assert!(events.is_empty());
        assert_eq!(s.hero(HERO).unwrap().pos, ENTRANCE);
        assert_eq!(s.owner(0), owner);
        assert_eq!(s.dwelling_guards(0), before.dwelling_guards(0));
        // E08 T97a's audit: the whole state as the question left it.
        assert!(s == asked);
    }
    // Slot 0 empty: the first occupied slot's creature, the word of its 4 (the first word).
    let mut s = game(&rules, at_entrance(&rules, "t:d5", None), peasant());
    let w5 = creature(&rules, "t:w5");
    s.set_dwelling(0, &[2, 2], &[stack(1, w5, 4)]);
    let (outcome, _) = visit(&rules, &mut s);
    assert_eq!(outcome.offer, Some(guarded(&rules, "t:d5", "t:w5", 0)));
    // Its guards beaten (none stand): flagged, then the hire question, no guards' question.
    let mut s = game(&rules, at_entrance(&rules, "t:d5", None), peasant());
    s.set_dwelling(0, &[2, 2], &[]);
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(outcome.offer, Some(hire(&rules, "t:d5", &["t:c5", "t:w5"])));
    assert_eq!(s.owner(0), Some(RED));
    assert_eq!(events.len(), 2);
}

/// The battle `outcome` opens.
fn battle(outcome: &AdventureOutcome) -> &BattleStart {
    outcome.battle.as_ref().unwrap()
}

// covers: ADV-DWL-004, GAM-BAT-001
#[test]
fn an_accepted_guards_question_fights_them_unsplit_and_unowned_on_the_hero_s_tile() {
    let rules = rules();
    let (c5, w5) = (creature(&rules, "t:c5"), creature(&rules, "t:w5"));
    // A neutral dwelling's two stacks: unsplit, unowned, on the tile's own ground.
    let mut s = game(&rules, at_entrance(&rules, "t:d5", None), peasant());
    let (outcome, _) = visit(&rules, &mut s);
    let before = s.clone();
    let (accepted, events) = accept(&rules, &mut s, &outcome.offer.unwrap());
    let start = battle(&accepted);
    let army = vec![stack(0, c5, 6), stack(1, w5, 6)];
    assert_eq!(start.defender, Defender::Guards { army });
    assert_eq!(start.defender.owner(), None);
    assert_eq!((start.tile, start.attacker.hero), (ENTRANCE, HERO));
    assert_eq!(
        (start.terrain, start.battlefield.clone()),
        s.battle_ground(&rules.1, ENTRANCE)
    );
    assert!(events.is_empty());
    assert_eq!(s, before);
    // Blue's dwelling with a single stack: unsplit and unowned too (no monster, no owner's side).
    let mut blue = game(&rules, at_entrance(&rules, "t:e5", Some(BLUE)), peasant());
    let (outcome, _) = visit(&rules, &mut blue);
    let ask = guarded(&rules, "t:e5", "t:f5", 1);
    assert_eq!(outcome.offer, Some(ask.clone()));
    let (accepted, _) = accept(&rules, &mut blue, &ask);
    let start = battle(&accepted);
    let f5 = creature(&rules, "t:f5");
    assert_eq!(
        start.defender,
        Defender::Guards {
            army: vec![stack(0, f5, 6)]
        }
    );
    assert_eq!(start.defender.owner(), None);
    assert_eq!(blue.owner(0), Some(BLUE));
    // A hero no longer on the map: nothing.
    let gone = Offer {
        hero: HeroId::new(9),
        ..ask
    };
    let before = blue.clone();
    let (accepted, events) = accept(&rules, &mut blue, &gone);
    assert_eq!(accepted, AdventureOutcome::default());
    assert!(events.is_empty());
    assert_eq!(blue, before);
}

// covers: ADV-DWL-004, ADV-OWN-001
#[test]
fn an_accepted_guards_question_with_none_standing_goes_on_with_the_visit() {
    // Unreachable under sequential play (nothing changes the dwelling while its question is
    // open): the state built directly.
    let rules = rules();
    let d5 = type_of(&rules, "t:d5");
    let ask = guarded(&rules, "t:d5", "t:c5", 2);
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(BLUE)), peasant());
    visit(&rules, &mut s);
    s.set_dwelling(0, &[2, 2], &[]);
    let (accepted, events) = accept(&rules, &mut s, &ask);
    assert_eq!(
        accepted,
        AdventureOutcome {
            offer: Some(hire(&rules, "t:d5", &["t:c5", "t:w5"])),
            ..AdventureOutcome::default()
        }
    );
    assert_eq!(s.owner(0), Some(RED));
    assert_eq!(
        events[0].kind,
        AdventureEventKind::ObjectFlagged {
            object_type: d5,
            anchor: ENTRANCE,
            owner: RED,
        }
    );
    assert_eq!(events.len(), 2);
    // Red's own (its guards standing): no battle, no flag, the hire question.
    let mut own = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), peasant());
    let (accepted, events) = accept(&rules, &mut own, &ask);
    assert_eq!(
        accepted.offer,
        Some(hire(&rules, "t:d5", &["t:c5", "t:w5"]))
    );
    assert_eq!(accepted.battle, None);
    assert!(events.is_empty());
}

// covers: ADV-DWL-004, GAM-DEC-007
#[test]
fn a_dwelling_battle_s_end_keeps_the_survivors_or_goes_on_after_the_levels() {
    let rules = rules();
    let parts = (&rules.0, &rules.1, &[][..]);
    let d5 = type_of(&rules, "t:d5");
    let w5 = creature(&rules, "t:w5");
    // Lost or retreated at blue's dwelling: the survivors by slot as they stand, its owner
    // unchanged, nothing told; the next visit asks of them.
    let start = game(&rules, at_entrance(&rules, "t:d5", Some(BLUE)), peasant());
    let mut s = start.clone();
    visit(&rules, &mut s);
    let fought = s.clone();
    let survivors = [stack(1, w5, 2)];
    let mut events = Vec::new();
    let ended = s.end_unowned_battle(parts, (HERO, ENTRANCE), (&survivors, false), &mut events);
    assert_eq!(ended, AdventureOutcome::default());
    assert_eq!(s.dwelling_guards(0), survivors);
    assert_eq!(s.owner(0), Some(BLUE));
    assert!(events.is_empty());
    assert!(s.check_dwellings());
    assert_ne!(s.state_hash(), fought.state_hash());
    let mut again = start.clone();
    again.end_unowned_battle(parts, (HERO, ENTRANCE), (&survivors, false), &mut events);
    let (outcome, _) = visit(&rules, &mut again);
    assert_eq!(outcome.offer, Some(guarded(&rules, "t:d5", "t:w5", 0)));
    // A draw (no survivor, no win): none stand; the next visit flags it at once.
    let mut draw = start.clone();
    let ended = draw.end_unowned_battle(parts, (HERO, ENTRANCE), (&[], false), &mut events);
    assert_eq!(ended, AdventureOutcome::default());
    assert!(draw.dwelling_guards(0).is_empty());
    assert!(draw.check_dwellings());
    assert_eq!(draw.owner(0), Some(BLUE));
    let (outcome, events) = visit(&rules, &mut draw);
    assert_eq!(outcome.offer, Some(hire(&rules, "t:d5", &["t:c5", "t:w5"])));
    assert_eq!(draw.owner(0), Some(RED));
    assert_eq!(events.len(), 2);
    // Won: the guards gone, its owner unchanged, nothing told; the visit's rest after the
    // winner's levels (`give_won`): the flag, then the hire question.
    let mut won = fought.clone();
    let mut events = Vec::new();
    let ended = won.end_unowned_battle(parts, (HERO, ENTRANCE), (&survivors, true), &mut events);
    assert_eq!(
        ended,
        AdventureOutcome {
            reward_after: Some((HERO, ENTRANCE)),
            ..AdventureOutcome::default()
        }
    );
    assert!(won.dwelling_guards(0).is_empty());
    assert_eq!(won.owner(0), Some(BLUE));
    assert!(events.is_empty());
    assert!(won.check_dwellings());
    let mut gone = won.clone();
    let given = won.give_won(parts, (HERO, ENTRANCE), &mut events);
    assert_eq!(
        given,
        AdventureOutcome {
            offer: Some(hire(&rules, "t:d5", &["t:c5", "t:w5"])),
            ..AdventureOutcome::default()
        }
    );
    assert_eq!(won.owner(0), Some(RED));
    assert_eq!(
        events[0],
        AdventureEvent {
            kind: AdventureEventKind::ObjectFlagged {
                object_type: d5,
                anchor: ENTRANCE,
                owner: RED,
            },
            seen_by: Viewers::NONE.with(RED).with(BLUE),
        }
    );
    assert!(won.check_owners(&rules.1));
    // A hero gone from the map: nothing.
    let before = gone.clone();
    let mut events = Vec::new();
    let given = gone.give_won(parts, (HeroId::new(9), ENTRANCE), &mut events);
    assert_eq!(given, AdventureOutcome::default());
    assert!(events.is_empty());
    assert_eq!(gone, before);
}

// covers: ADV-RFG-001
#[test]
fn a_camp_asks_the_hire_question_or_tells_its_visitor_it_is_deserted() {
    let rules = rules();
    let camp = type_of(&rules, "t:camp");
    let mut s = game(&rules, at_entrance(&rules, "t:camp", None), peasant());
    let (drawn, count) = s.camp(0).unwrap();
    assert!(count > 0);
    let before = s.clone();
    let (outcome, events) = visit(&rules, &mut s);
    // Its week's creature named; no flag, no event.
    assert_eq!(
        outcome.offer,
        Some(Offer {
            hero: HERO,
            object_type: camp,
            anchor: ENTRANCE,
            kind: OfferKind::Hire {
                creatures: vec![drawn],
            },
        })
    );
    assert!(events.is_empty());
    assert_eq!(s.owner(0), None);
    // Accepted: no free creatures, whatever the creature's level; the hire dialog (T95b,
    // `hiring.rs`).
    let mut accepted = s.clone();
    let (outcome, events) = accept(&rules, &mut accepted, &outcome.offer.unwrap());
    assert!(outcome.hire.is_some());
    assert!(events.is_empty());
    assert_eq!(accepted, s);
    // None left: deserted, told to red alone, no question.
    let mut empty = before;
    empty.set_camp(0, drawn, 0);
    let (outcome, events) = visit(&rules, &mut empty);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(
        events,
        [to_red(AdventureEventKind::ObjectEmpty {
            object_type: camp,
            anchor: ENTRANCE,
        })]
    );
}

// covers: ADV-HIR-001, ADV-ARM-007
#[test]
fn an_accepted_question_at_an_own_dwelling_gives_its_free_creatures() {
    let rules = rules();
    let (c1, w1) = (creature(&rules, "t:c1"), creature(&rules, "t:w1"));
    let offer = hire(&rules, "t:d1", &["t:c1", "t:w1"]);
    // Into the first free slots: the whole count of each, the counts set to 0.
    let mut s = game(&rules, at_entrance(&rules, "t:d1", Some(RED)), peasant());
    visit(&rules, &mut s);
    let visited = s.clone();
    let (outcome, events) = accept(&rules, &mut s, &offer);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(
        events,
        [
            to_red(AdventureEventKind::CreaturesJoined {
                hero: HERO,
                creature: c1,
                count: 2,
            }),
            to_red(AdventureEventKind::CreaturesJoined {
                hero: HERO,
                creature: w1,
                count: 2,
            }),
        ]
    );
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, PEASANT, 1), stack(1, c1, 2), stack(2, w1, 2)]
    );
    assert_eq!(s.dwelling_counts(0), Some(&[0, 0][..]));
    assert!(s.check_dwellings());
    assert_ne!(s.state_hash(), visited.state_hash());
    // The fork made before keeps the dwelling full: the write copied the block.
    assert!(visited.dwellings.is_none());
    assert_eq!(visited.dwelling_counts(0), Some(&[2, 2][..]));
    // Onto a stack of the type: the castle's creature joins red's stack of it in slot 3.
    let mut s = game(
        &rules,
        at_entrance(&rules, "t:d1", Some(RED)),
        vec![stack(0, PEASANT, 1), stack(3, c1, 5)],
    );
    visit(&rules, &mut s);
    accept(&rules, &mut s, &offer);
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, PEASANT, 1), stack(1, w1, 2), stack(3, c1, 7)]
    );
}

// covers: ADV-HIR-001, ADV-ARM-007
#[test]
fn free_creatures_without_room_stay_and_a_count_of_0_is_none_here() {
    let rules = rules();
    let d1 = type_of(&rules, "t:d1");
    let (c1, w1) = (creature(&rules, "t:c1"), creature(&rules, "t:w1"));
    let offer = hire(&rules, "t:d1", &["t:c1", "t:w1"]);
    // Seven slots full, one of them a stack of the castle's creature: it joins there, the wild's
    // finds no room, none of it joins and its count stays (no merge of the hero's stacks, D-52).
    let full: Vec<ArmyStack> = (0..7)
        .map(|slot| {
            let k = if slot == 4 { "t:c1" } else { "t:f2" };
            stack(slot, creature(&rules, k), 1)
        })
        .collect();
    let mut s = game(&rules, at_entrance(&rules, "t:d1", Some(RED)), full);
    visit(&rules, &mut s);
    let (_, events) = accept(&rules, &mut s, &offer);
    assert_eq!(
        events,
        [
            to_red(AdventureEventKind::CreaturesJoined {
                hero: HERO,
                creature: c1,
                count: 2,
            }),
            to_red(AdventureEventKind::JoinRefused {
                hero: HERO,
                creature: w1,
                count: 2,
            }),
        ]
    );
    assert_eq!(s.dwelling_counts(0), Some(&[0, 2][..]));
    assert_eq!(s.hero(HERO).unwrap().army[4], stack(4, c1, 3));
    // A second acceptance: none of the castle's here, the wild's refused again; nothing written,
    // so the block is shared still.
    let fork = s.clone();
    let (_, events) = accept(&rules, &mut s, &offer);
    assert_eq!(
        events,
        [
            to_red(AdventureEventKind::NoneToRecruit {
                object_type: d1,
                anchor: ENTRANCE,
                creature: c1,
            }),
            to_red(AdventureEventKind::JoinRefused {
                hero: HERO,
                creature: w1,
                count: 2,
            }),
        ]
    );
    assert_eq!(s, fork);
    assert!(Arc::ptr_eq(
        s.dwellings.as_ref().unwrap(),
        fork.dwellings.as_ref().unwrap()
    ));
}

// covers: ADV-HIR-001
#[test]
fn no_creature_joins_free_above_free_level_nor_at_another_s_dwelling() {
    let rules = rules();
    // A level-5 dwelling of red's: nothing free (its creatures are the hire dialog's, T95b).
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), peasant());
    visit(&rules, &mut s);
    let visited = s.clone();
    let offer = hire(&rules, "t:d5", &["t:c5", "t:w5"]);
    let (outcome, events) = accept(&rules, &mut s, &offer);
    assert!(outcome.hire.is_some());
    assert!(events.is_empty());
    assert_eq!(s, visited);
    // A dwelling blue owns when red accepts: nothing (the owner is read at acceptance).
    let mut s = game(&rules, at_entrance(&rules, "t:d1", Some(BLUE)), peasant());
    let before = s.clone();
    let (outcome, events) = accept(&rules, &mut s, &hire(&rules, "t:d1", &["t:c1", "t:w1"]));
    assert_eq!(outcome, AdventureOutcome::default());
    assert!(events.is_empty());
    assert_eq!(s, before);
}

// covers: ADV-DWL-002
#[test]
fn the_dwellings_writer_keeps_the_block_as_it_was_when_nothing_changes() {
    let rules = rules();
    let mut s = game(&rules, at_entrance(&rules, "t:d1", None), peasant());
    let built = s.clone();
    // An object that is no dwelling, or a write of the week as it stands: nothing written.
    s.write_dwelling(1, |week| week.counts_mut().fill(0));
    s.write_dwelling(0, |_| {});
    assert!(s.dwellings.is_none());
    assert_eq!(s, built);
    s.write_dwelling(0, |week| week.counts_mut()[1] = 1);
    assert_eq!(s.dwelling_counts(0), Some(&[2, 1][..]));
    let fork = s.clone();
    s.write_dwelling(0, |week| week.counts_mut()[1] = 1);
    assert!(Arc::ptr_eq(
        s.dwellings.as_ref().unwrap(),
        fork.dwellings.as_ref().unwrap()
    ));
    // Back to full: the block dropped, one state with the built one.
    s.write_dwelling(0, |week| week.counts_mut()[1] = 2);
    assert!(s.dwellings.is_none() && s.check_dwellings());
    assert_eq!(s, built);
}
