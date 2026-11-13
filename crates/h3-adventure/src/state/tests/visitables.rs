//! The bonus visitables' visits (E08 T115b; increment 12's design note §8 to §10; ADV-BON-002 to
//! -009): the `bonus` kinds by the day of the week, the Fountain of Fortune's value and its draws,
//! the Magic Well's day and the Magic Spring's entrances, the Stables' week and upgrade, the
//! Sirens' losses and their mark after a battle under `ADV-QUIRK-46`, the kind marks.

use std::string::String;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use super::teleporters::{HERO, ONTO, RED, Rules, p, stack};
use super::*;
use crate::command::{Drowned, HeroGains, NothingWhy};
use crate::observe::{Known, KnownObject};
use crate::ruleset::{BONUS_DAYS, ObjectKindDef, ObjectTypeDef, Visit};
use crate::state::visitables::sirens_keep;
use crate::testing::{GNOLL, PIKEMAN, hero_at, hero_details, key, movement_rules_with, started};
use crate::testing::{stream, world};
use h3_heroes::{HeroChange, VisitBonus};

/// [`movement_rules_with`] (`faithful` or `strict-fixed` by `faithful`: the Sirens' flag on or
/// off, `ADV-QUIRK-46`) with one type of each of the fifteen's six kinds, all entered but the
/// Sirens: the Idol `t:idol` (mark 4: odd days luck, even days morale, day 7 both), the Oasis
/// `t:oasis` (mark 7, +1 morale and 400 points), the Swan Pond `t:swan` (mark 3, +2 luck, the
/// points drained), the Fountain `t:fountain` (mark 5), the Well `t:well`, the Spring `t:spring`,
/// the Stables `t:stables` (400 points, Pikemen to Gnolls) and the Sirens `t:sirens` (mark 13,
/// 7 / 10, visited from a neighbour). The kind marks as the base pack's: the bonus kinds, the Well
/// and the Spring.
fn visitable_rules(faithful: bool) -> Rules {
    movement_rules_with(
        faithful,
        32,
        |_| {},
        |part| {
            let ty = |k: &str, visit, kind_mark, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit,
                from_above: false,
                kind_mark,
                kind,
            };
            let bonus = |mark, days, points, drain| ObjectKindDef::Bonus {
                mark,
                days,
                points,
                drain,
            };
            let idol = [(0, 1), (1, 0), (0, 1), (1, 0), (0, 1), (1, 0), (1, 1)];
            part.object_types.extend([
                ty("t:idol", Visit::Enter, true, bonus(4, idol, 0, false)),
                ty(
                    "t:oasis",
                    Visit::Enter,
                    true,
                    bonus(7, [(1, 0); BONUS_DAYS], 400, false),
                ),
                ty(
                    "t:swan",
                    Visit::Enter,
                    true,
                    bonus(3, [(0, 2); BONUS_DAYS], 0, true),
                ),
                ty(
                    "t:fountain",
                    Visit::Enter,
                    false,
                    ObjectKindDef::FountainOfFortune {
                        mark: 5,
                        values: vec![-1, 1, 2, 3],
                    },
                ),
                ty("t:well", Visit::Enter, true, ObjectKindDef::MagicWell),
                ty("t:spring", Visit::Enter, true, ObjectKindDef::MagicSpring),
                ty(
                    "t:stables",
                    Visit::Enter,
                    false,
                    ObjectKindDef::Stables {
                        points: 400,
                        from: key("t:pikeman"),
                        to: key("t:gnoll"),
                    },
                ),
                ty(
                    "t:sirens",
                    Visit::Neighbour,
                    false,
                    ObjectKindDef::Sirens {
                        mark: 13,
                        keep: (7, 10),
                    },
                ),
            ]);
        },
    )
}

/// Red's hero [`HERO`] on (0, 1) of a 30 × 3 grass map with Pikemen, and the objects of type `k`
/// anchored on each `anchor`, visited from the tiles of `visitable` (blocked as visited).
fn state(rules: &Rules, objects: &[(&str, Pos, u64)]) -> AdventureState {
    let rows = vec!["g".repeat(30); 3];
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut setup = world(&rows, 1, vec![hero_at(1, 0, p(0, 1), &[PIKEMAN])]);
    for &(k, anchor, visitable) in objects {
        setup.map.objects.push(ObjectSetup {
            scenery: Scenery::default(),
            anchor,
            blocked: TileMask::from_bits(visitable),
            visitable: TileMask::from_bits(visitable),
            object_type: Some(rules.1.object_type_by_key(k).unwrap()),
            amount: None,
            drawn: TileMask::default(),
            monster: None,
            guards: None,
            holds: Holds::Nothing,
            owner: None,
        });
    }
    started(rules, &setup).unwrap()
}

/// Red's hero walks `path` on the day `day` of the week, its details' spell points `mana` (the
/// details' maximum 10: Knowledge 1): the outcome and the events.
fn go(
    s: &mut AdventureState,
    rules: &Rules,
    path: &[Pos],
    (day, mana): (u8, u16),
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    command_at(s, rules, &command, (day, mana))
}

/// Red's hero visits its tile's object again (ADV-TEL-008), as [`go`].
fn again(
    s: &mut AdventureState,
    rules: &Rules,
    (day, mana): (u8, u16),
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let command = AdventureCommand::Revisit { hero: HERO };
    command_at(s, rules, &command, (day, mana))
}

fn command_at(
    s: &mut AdventureState,
    rules: &Rules,
    command: &AdventureCommand,
    (day, mana): (u8, u16),
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut details = hero_details(&[]);
    details.mana = mana;
    let store = [Arc::new(details.clone()), Arc::new(details)];
    let mut events = Vec::new();
    let weekday = Weekday::new(day).unwrap();
    let outcome = s
        .apply(
            &rules.0,
            &rules.1,
            &store,
            (RED, weekday, &mut stream()),
            command,
            &mut events,
        )
        .unwrap();
    (outcome, events)
}

/// The visits' own events among `events`: the nothing gained, the losses, the upgrades and the
/// kind marks learned (the step's and the reveal's dropped).
fn told(events: &[AdventureEvent]) -> Vec<AdventureEventKind> {
    events
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                AdventureEventKind::NothingGained { .. }
                    | AdventureEventKind::SirensLoss { .. }
                    | AdventureEventKind::CreaturesUpgraded { .. }
                    | AdventureEventKind::KindKnown { .. }
            )
        })
        .inspect(|e| assert_eq!(e.seen_by, Viewers::NONE.with(RED)))
        .map(|e| e.kind)
        .collect()
}

/// `hero`'s visit of the object of type `k` on `anchor` gave nothing, for `why`.
fn nothing(rules: &Rules, k: &str, anchor: Pos, why: NothingWhy) -> AdventureEventKind {
    AdventureEventKind::NothingGained {
        hero: HERO,
        object_type: rules.1.object_type_by_key(k).unwrap(),
        anchor,
        why,
    }
}

fn kind_known(rules: &Rules, k: &str) -> AdventureEventKind {
    AdventureEventKind::KindKnown {
        object_type: rules.1.object_type_by_key(k).unwrap(),
    }
}

fn once(mark: u8, morale: i8, luck: i8) -> VisitBonus {
    VisitBonus { mark, morale, luck }
}

/// The hero's points left and day's maximum.
fn points(s: &AdventureState) -> (u32, u32) {
    let hero = s.hero(HERO).unwrap();
    (hero.movement, hero.max_movement)
}

// covers: ADV-BON-002, ADV-BON-004, ADV-BON-009
#[test]
fn a_bonus_gives_the_days_pair_once_until_a_battle_then_its_movement_and_the_kind_mark() {
    let rules = visitable_rules(true);
    // The Idol by the day of the week (its week of pairs; the original's date +0x1F63E): day 2
    // morale, day 7 both, day 1 luck.
    for (day, bonus) in [(2, once(4, 1, 0)), (7, once(4, 1, 1)), (1, once(4, 0, 1))] {
        let mut s = state(&rules, &[("t:idol", ONTO[0], 1)]);
        let start = points(&s);
        let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (day, 0));
        assert_eq!(outcome, AdventureOutcome::default());
        assert_eq!(s.visit_marks(HERO).bonuses().once, [bonus], "day {day}");
        // No points: the step's price alone; the kind mark learned, told once.
        assert_eq!(points(&s).1, start.1);
        assert_eq!(told(&events), [kind_known(&rules, "t:idol")]);
        assert!(s.knows_kind(RED, rules.1.object_type_by_key("t:idol").unwrap()));
        // Again the same day, or another (its mark held until a battle): nothing, told.
        for day in [day, 3] {
            let (outcome, events) = again(&mut s, &rules, (day, 0));
            assert_eq!(outcome, AdventureOutcome::default());
            let idol = nothing(&rules, "t:idol", ONTO[0], NothingWhy::Visited);
            assert_eq!(told(&events), [idol]);
            assert_eq!(s.visit_marks(HERO).bonuses().once, [bonus]);
        }
        // After a battle's end it gives again, by that day; the kind mark is not told twice.
        s.clear_battle_bonuses(&rules.0, &rules.1, &[HERO]);
        let (_, events) = again(&mut s, &rules, (3, 0));
        assert_eq!(s.visit_marks(HERO).bonuses().once, [once(4, 0, 1)]);
        assert_eq!(told(&events), []);
    }
    // The Oasis: +1 morale and 400 points to the points left and the day's maximum.
    let mut s = state(&rules, &[("t:oasis", ONTO[0], 1)]);
    let (_, max) = points(&s);
    go(&mut s, &rules, &ONTO[..1], (1, 0));
    let left = points(&s).0;
    assert_eq!(points(&s), (left, max + 400));
    assert!(left >= 400);
    s.clear_battle_bonuses(&rules.0, &rules.1, &[HERO]);
    again(&mut s, &rules, (1, 0));
    assert_eq!(points(&s), (left + 400, max + 800));
    // Saturating (the original's 32 bits unbounded).
    s.hero_mut(HERO).unwrap().max_movement = u32::MAX - 1;
    s.clear_battle_bonuses(&rules.0, &rules.1, &[HERO]);
    again(&mut s, &rules, (1, 0));
    assert_eq!(points(&s).1, u32::MAX);
    // The Swan Pond: +2 luck, the points left emptied, the maximum kept.
    let mut s = state(&rules, &[("t:swan", ONTO[0], 1)]);
    let (_, max) = points(&s);
    go(&mut s, &rules, &ONTO[..1], (1, 0));
    assert_eq!(points(&s), (0, max));
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(3, 0, 2)]);
}

// covers: ADV-BON-003, ADV-VIS-006, ADV-WEK-001
#[test]
fn a_fountain_of_fortune_gives_its_weeks_value_and_its_team_knows_it() {
    let rules = visitable_rules(true);
    let objects = [("t:fountain", ONTO[0], 1), ("t:fountain", p(5, 0), 1)];
    let mut s = state(&rules, &objects);
    // Each fountain's value is drawn when the game is built, among the kind's values.
    let values = s.fortunes.as_deref().unwrap().to_vec();
    assert_eq!(values.len(), 2);
    assert!(values.iter().all(|v| [-1, 1, 2, 3].contains(v)));
    let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (1, 0));
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(5, 0, values[0])]);
    // No kind mark (the type has none), but the team knows the fountain: visited, never its
    // value (ADV-VIS-006).
    assert_eq!(told(&events), []);
    let fountain = rules.1.object_type_by_key("t:fountain").unwrap();
    let known = s.observe(&rules.0, &rules.1, RED).known;
    assert_eq!(
        known,
        [KnownObject {
            object_type: fountain,
            anchor: ONTO[0],
            known: Known::Visited,
        }]
    );
    assert!(
        s.observe(&rules.0, &rules.1, PlayerId::new(1))
            .known
            .is_empty()
    );
    // Held: nothing, told; still known.
    let (_, events) = again(&mut s, &rules, (1, 0));
    let held = nothing(&rules, "t:fountain", ONTO[0], NothingWhy::Visited);
    assert_eq!(told(&events), [held]);
    // The week's step draws each value anew from the game's stream, in object order.
    let mut draws = stream();
    let hash = s.state_hash();
    let mut weeks = Vec::new();
    for _ in 0..12 {
        s.start_week(&rules.0, &rules.1, &mut draws, &mut Vec::new());
        weeks.push(s.fortunes.as_deref().unwrap().to_vec());
    }
    assert!(weeks.iter().flatten().all(|v| [-1, 1, 2, 3].contains(v)));
    assert!(
        weeks
            .iter()
            .any(|week| week[0] != values[0] || week[1] != values[1])
    );
    // The values are hashed (v24).
    assert_ne!(s.state_hash(), hash);
}

// covers: ADV-BON-003
#[test]
fn a_map_without_a_fountain_has_no_values_to_draw() {
    let rules = visitable_rules(true);
    let none = state(&rules, &[("t:idol", ONTO[0], 1)]);
    assert!(none.fortunes.is_none());
    let mut s = none.clone();
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut Vec::new());
    assert!(s.fortunes.is_none());
    assert_eq!(s.state_hash(), none.state_hash());
}

// covers: ADV-BON-006, ADV-BON-009, HER-MAN-001
#[test]
fn a_magic_well_fills_the_spell_points_once_a_day_and_marks_no_full_hero() {
    let rules = visitable_rules(true);
    let well = rules.1.object_type_by_key("t:well").unwrap();
    let mut s = state(&rules, &[("t:well", ONTO[0], 1)]);
    // Full (the maximum 10): nothing, told, no day mark; the kind mark learned anyway.
    let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (1, 10));
    assert_eq!(outcome, AdventureOutcome::default());
    let full = nothing(&rules, "t:well", ONTO[0], NothingWhy::Full);
    assert_eq!(told(&events), [kind_known(&rules, "t:well"), full]);
    assert!(s.visit_marks(HERO).timed().is_empty());
    // Below it: the points set to the maximum, the day mark.
    let (outcome, events) = again(&mut s, &rules, (1, 3));
    let set = HeroGains::changes(HERO, vec![HeroChange::SetSpellPoints(10)]);
    assert_eq!(outcome.hero, Some(set));
    assert_eq!(told(&events), []);
    assert_eq!(s.visit_marks(HERO).timed(), [well]);
    // The same day: nothing, told.
    let (outcome, events) = again(&mut s, &rules, (1, 0));
    assert_eq!(outcome, AdventureOutcome::default());
    let visited = nothing(&rules, "t:well", ONTO[0], NothingWhy::Visited);
    assert_eq!(told(&events), [visited]);
    // The next day's start drops the mark: it fills again.
    s.start_day(&rules.0, &rules.1, &[]);
    assert!(s.visit_marks(HERO).timed().is_empty());
    assert!(again(&mut s, &rules, (2, 0)).0.hero.is_some());
}

// covers: ADV-BON-007, ADV-VIS-006, ADV-BON-009
#[test]
fn a_magic_springs_entrances_each_fill_twice_the_maximum_once_a_week() {
    let rules = visitable_rules(true);
    // One Spring with two entrances, (2, 1) and (1, 1) (D-62).
    let mut s = state(&rules, &[("t:spring", ONTO[1], 0b11)]);
    let built = s.clone();
    // Full at twice the maximum (20): nothing, told, the entrance stays full.
    let (_, events) = go(&mut s, &rules, &ONTO[..1], (1, 20));
    let full = nothing(&rules, "t:spring", ONTO[1], NothingWhy::Full);
    assert_eq!(told(&events), [kind_known(&rules, "t:spring"), full]);
    assert!(s.springs.is_none());
    // Below: twice the maximum, the entrance emptied.
    let (outcome, _) = again(&mut s, &rules, (1, 19));
    let set = HeroGains::changes(HERO, vec![HeroChange::SetSpellPoints(20)]);
    assert_eq!(outcome.hero, Some(set.clone()));
    assert_eq!(s.springs.as_deref(), Some(&vec![ONTO[0]]));
    let (outcome, events) = again(&mut s, &rules, (1, 0));
    assert_eq!(outcome, AdventureOutcome::default());
    let emptied = nothing(&rules, "t:spring", ONTO[1], NothingWhy::Emptied);
    assert_eq!(told(&events), [emptied]);
    // The other entrance keeps its own state.
    let (outcome, _) = go(&mut s, &rules, &ONTO[1..], (1, 0));
    assert_eq!(outcome.hero, Some(set));
    assert_eq!(s.springs.as_deref(), Some(&vec![ONTO[0], ONTO[1]]));
    // The team knows the Spring: visited, never its entrances' state.
    let known = s.observe(&rules.0, &rules.1, RED).known;
    assert_eq!(
        known.iter().map(|k| &k.known).collect::<Vec<_>>(),
        [&Known::Visited]
    );
    // The week's step refills every entrance: no block, hashed as none.
    let hash = s.state_hash();
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut Vec::new());
    assert!(s.springs.is_none());
    assert_ne!(s.state_hash(), hash);
    assert!(again(&mut s, &rules, (1, 0)).0.hero.is_some());
    // Built: none emptied.
    assert!(built.springs.is_none());
}

// covers: ADV-BON-005, ADV-BON-001
#[test]
fn the_stables_give_the_weeks_movement_once_and_upgrade_at_every_visit() {
    let rules = visitable_rules(true);
    let stables = rules.1.object_type_by_key("t:stables").unwrap();
    let mut s = state(&rules, &[("t:stables", ONTO[0], 1)]);
    s.hero_mut(HERO).unwrap().army = vec![stack(0, PIKEMAN, 5), stack(3, GNOLL, 2)];
    let (_, max) = points(&s);
    let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (1, 0));
    assert_eq!(outcome, AdventureOutcome::default());
    // The week's 400 to the points left and the maximum, the week mark (its land points: 400).
    assert_eq!(points(&s).1, max + 400);
    assert_eq!(s.visit_marks(HERO).timed(), [stables]);
    assert_eq!(s.visit_marks(HERO).week_land_points(&rules.1), 400);
    // The Pikemen become Gnolls, their count kept; no kind mark.
    let upgraded = AdventureEventKind::CreaturesUpgraded {
        hero: HERO,
        slot: 0,
        from: PIKEMAN,
        to: GNOLL,
    };
    assert_eq!(told(&events), [upgraded]);
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, GNOLL, 5), stack(3, GNOLL, 2)]
    );
    // Again with the mark and nothing to upgrade: nothing, told.
    let before = points(&s);
    let (_, events) = again(&mut s, &rules, (1, 0));
    let visited = nothing(&rules, "t:stables", ONTO[0], NothingWhy::Visited);
    assert_eq!(told(&events), [visited]);
    assert_eq!(points(&s), before);
    // With the mark, new Pikemen are upgraded, no movement (D-63).
    s.hero_mut(HERO).unwrap().army.push(stack(6, PIKEMAN, 1));
    let (_, events) = again(&mut s, &rules, (1, 0));
    assert_eq!(told(&events).len(), 1);
    assert_eq!(points(&s), before);
    assert_eq!(s.hero(HERO).unwrap().army[2], stack(6, GNOLL, 1));
    // The week's start drops the mark: the movement again.
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut Vec::new());
    assert!(s.visit_marks(HERO).timed().is_empty());
    again(&mut s, &rules, (1, 0));
    assert_eq!(s.visit_marks(HERO).timed(), [stables]);
}

// covers: ADV-BON-008
#[test]
fn the_sirens_keep_the_double_products_survivors_in_full_width() {
    // D-64: the double product, not the exact tenths (90 keeps 62, 170 keeps 118); A-45 fixed
    // (`ADV-QUIRK-47`): a stack of 46 812 keeps 32 768, not the 16 bits' −32 768.
    for (count, kept) in [
        (2, 1),
        (3, 2),
        (10, 7),
        (90, 62),
        (170, 118),
        (180, 125),
        (330, 230),
        (46_812, 32_768),
        (1_000_000, 700_000),
    ] {
        assert_eq!(sirens_keep(count, (7, 10)), kept, "{count}");
    }
    // A whole keep keeps all; a smaller part than the original's never empties a slot; a lone
    // creature is the visit's (never asked here).
    assert_eq!(sirens_keep(90, (10, 10)), 90);
    assert_eq!(sirens_keep(90, (0, 10)), 1);
    assert_eq!(sirens_keep(9, (1, 10)), 1);
}

// covers: ADV-BON-008, GAM-BAT-014
#[test]
fn the_sirens_take_their_part_once_and_again_after_a_battle_only_under_the_quirk() {
    for faithful in [true, false] {
        let rules = visitable_rules(faithful);
        let mut s = state(&rules, &[("t:sirens", ONTO[0], 1)]);
        s.hero_mut(HERO).unwrap().army = vec![
            stack(0, PIKEMAN, 90),
            stack(2, GNOLL, 1),
            stack(4, GNOLL, 170),
        ];
        // From the neighbour (the hero stays on (0, 1)).
        let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (1, 0));
        assert_eq!(s.hero(HERO).unwrap().pos, p(0, 1));
        let loss = |slot, creature, count| AdventureEventKind::SirensLoss {
            hero: HERO,
            slot,
            creature,
            count,
        };
        assert_eq!(told(&events), [loss(0, PIKEMAN, 28), loss(4, GNOLL, 52)]);
        let army = [
            stack(0, PIKEMAN, 62),
            stack(2, GNOLL, 1),
            stack(4, GNOLL, 118),
        ];
        assert_eq!(s.hero(HERO).unwrap().army, army);
        let drowned = Drowned {
            hero: HERO,
            lost: vec![(PIKEMAN, 28), (GNOLL, 52)],
        };
        assert_eq!(
            outcome,
            AdventureOutcome {
                drowned: Some(drowned),
                ..AdventureOutcome::default()
            }
        );
        // The mark, no amounts (no battle bonus); held: nothing, told.
        assert_eq!(s.visit_marks(HERO).bonuses().once, [once(13, 0, 0)]);
        let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (1, 0));
        assert_eq!(outcome, AdventureOutcome::default());
        let visited = nothing(&rules, "t:sirens", ONTO[0], NothingWhy::Visited);
        assert_eq!(told(&events), [visited]);
        // A battle's end: `faithful` clears the mark (the original, A-44); `strict-fixed` keeps
        // it with the other marks cleared (decision 308).
        s.give_once(HERO, once(2, 1, 0));
        s.give_bonus(HERO, 1, 1);
        s.clear_battle_bonuses(&rules.0, &rules.1, &[HERO]);
        let left: &[VisitBonus] = if faithful { &[] } else { &[once(13, 0, 0)] };
        assert_eq!(s.visit_marks(HERO).bonuses().once, left, "{faithful}");
        assert_eq!(s.visit_marks(HERO).bonuses().morale, 0);
        let (outcome, _) = go(&mut s, &rules, &ONTO[..1], (1, 0));
        assert_eq!(outcome.drowned.is_some(), faithful);
        // A second clear with only the kept mark writes nothing.
        let before = s.clone();
        if !faithful {
            s.clear_battle_bonuses(&rules.0, &rules.1, &[HERO]);
            assert_eq!(s, before);
        }
    }
    // Nothing lost (lone creatures): the mark, no loss, no outcome.
    let rules = visitable_rules(true);
    let mut s = state(&rules, &[("t:sirens", ONTO[0], 1)]);
    s.hero_mut(HERO).unwrap().army = vec![stack(0, PIKEMAN, 1)];
    let (outcome, events) = go(&mut s, &rules, &ONTO[..1], (1, 0));
    assert_eq!(
        (outcome, told(&events)),
        (AdventureOutcome::default(), vec![])
    );
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(13, 0, 0)]);
}

// covers: ADV-BON-009, ADV-FOG-006
#[test]
fn a_kind_mark_is_its_teams_and_shows_in_its_observation_alone() {
    let rules = visitable_rules(true);
    let idol = rules.1.object_type_by_key("t:idol").unwrap();
    let oasis = rules.1.object_type_by_key("t:oasis").unwrap();
    let mut s = state(&rules, &[("t:oasis", ONTO[0], 1), ("t:idol", ONTO[1], 1)]);
    let built = s.clone();
    go(&mut s, &rules, &ONTO[..1], (1, 0));
    go(&mut s, &rules, &ONTO[1..], (1, 0));
    let kinds = |s: &AdventureState, player| s.observe(&rules.0, &rules.1, player).kinds;
    let mut both = vec![idol, oasis];
    both.sort();
    assert_eq!(kinds(&s, RED), both);
    assert_eq!(s.kinds_known(RED), both);
    assert!(kinds(&s, PlayerId::new(1)).is_empty());
    assert!(!s.knows_kind(PlayerId::new(1), idol));
    assert!(kinds(&built, RED).is_empty());
    // Hashed (v24): the marks alone move the hash.
    let mut marked = built.clone();
    marked.mark_kind(&rules.1, RED, idol, &mut Vec::new());
    assert_ne!(marked.state_hash(), built.state_hash());
    // A type without the flag marks nothing.
    let fountain = rules.1.object_type_by_key("t:fountain").unwrap();
    let mut events = Vec::new();
    marked.mark_kind(&rules.1, RED, fountain, &mut events);
    assert!(events.is_empty() && !marked.knows_kind(RED, fountain));
}
