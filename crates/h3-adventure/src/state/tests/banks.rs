//! Tests of the creature banks' visits, battles and rewards (E08 T124b2; increment 13's design note
//! §6 to §8, §12, §14, §15): an attack kind's marks at once and its question unless looted, a
//! search kind's question first and its marks after a yes, `Fight` and `Search` routed by the
//! kind, the battle against the guards as they stand at the entrance (a deck kind's on the deck),
//! the end's survivors, a draw's empty guards, the reward after the winner's levels (the creature
//! or the army dialog's group, the artifacts in the draw's order, the resources), the loot, a
//! looted search's once-bonus by its mark, and `Revisit`. The draw itself is `draws.rs`'. Since
//! E08 T125a (§9, §10, §16) the hint by the team's mark and its events, and the Shipwreck's water
//! entrance visited from the coast and crossed by a flight.

use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{ArtifactTypeId, HeroParamsDef, SkillId};
use h3_heroes::{HeroDetails, VisitBonus};

use super::draws::{object, rules_with};
use super::rewards::{
    AT, BLUE, GOLD, HERO, RED, RING, Rules, add, draw, hero, map_of, setup_of, store, type_of,
};
use super::*;
use crate::command::{Defender, HeroGains, NoRoom};
use crate::observe::{GuardsView, Known, KnownObject};
use crate::ruleset::{
    BankDef, BankLevelDef, BankQuestion, ObjectKindDef, ObjectTypeDef, OnceBonus, Visit,
};
use crate::setup::{Holds, ObjectSetup};
use crate::state::Resolved;
use crate::testing::{
    FLYING, GNOLL, PEASANT, PIKEMAN, WATER_WALKING, WOLF, coasted, hero_at, hero_details, key,
    started, stream, world,
};

const CROWN: ArtifactTypeId = ArtifactTypeId::from_raw(5);

/// The one level of these tests' banks: 10 peasants (five stacks of 2 after the split), 500 gold,
/// 3 wolves, a treasure and a relic.
fn level() -> BankLevelDef {
    BankLevelDef {
        chance: 100,
        guards: vec![(key("t:peasant"), 10)],
        upgrade: 0,
        resources: vec![(key("t:gold"), 500)],
        creature: Some((key("t:wolf"), 3)),
        artifacts: [1, 0, 0, 1],
    }
}

/// [`rules_with`] with hero parameters (a backpack of two) and four entered banks: the attack kind
/// `t:utopia`, the search kinds `t:crypt` (looted −1 morale, mark 15) and `t:wreck` (mark 14,
/// fought on the deck), and the attack kind `t:hive` without a reward's creature; and (E08 T125a)
/// the search kind `t:shipwreck`, visited from a neighbour and from above, fought on the deck, its
/// water entrance a coast's (design note §9).
fn rules() -> Rules {
    rules_with(
        true,
        |core| {
            core.hero_params = Some(HeroParamsDef {
                primary_min: [0, 0, 1, 1],
                primary_max: 99,
                backpack: 2,
                learn_level: 2,
                experience: vec![0, 1000, 2000],
                max_level: 74,
                wisdom: key("t:logistics"),
                schools: vec![],
                mana_ceiling: 999,
                base_mana_regain: 1,
                regain_specialist_bonus: 1,
            });
        },
        |types| {
            let bank = |k: &str, question, mark: Option<u8>, deck, level| ObjectTypeDef {
                key: key(k),
                name: "Bank".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: false,
                kind: ObjectKindDef::Bank(BankDef {
                    question,
                    levels: vec![level],
                    looted: mark.map(|mark| OnceBonus {
                        morale: -1,
                        luck: 0,
                        mark,
                    }),
                    deck,
                    coast: false,
                }),
            };
            let hive = BankLevelDef {
                creature: None,
                ..level()
            };
            let shipwreck = ObjectTypeDef {
                key: key("t:shipwreck"),
                visit: Visit::Neighbour,
                from_above: true,
                kind: ObjectKindDef::Bank(BankDef {
                    question: BankQuestion::Search,
                    levels: vec![level()],
                    looted: None,
                    deck: true,
                    coast: true,
                }),
                ..bank("t:utopia", BankQuestion::Attack, None, false, level())
            };
            types.extend([
                bank("t:utopia", BankQuestion::Attack, None, false, level()),
                bank("t:crypt", BankQuestion::Search, Some(15), false, level()),
                bank("t:wreck", BankQuestion::Search, Some(14), true, level()),
                bank("t:hive", BankQuestion::Attack, None, false, hive),
                shipwreck,
            ]);
        },
    )
}

/// A started map with a bank of type `k` at [`AT`], drawn at its one level with the crown and
/// the ring (the relic first), its guards five stacks of 2 peasants.
fn bank(rules: &Rules, k: &str) -> AdventureState {
    bank_in(rules, k, |setup| setup)
}

/// [`bank`] on `edit`'s setup ([`allies`]: blue on red's team).
fn bank_in(rules: &Rules, k: &str, edit: fn(AdventureSetup) -> AdventureSetup) -> AdventureState {
    let setup = edit(setup_of(vec![object(rules, AT.x, k, Holds::Nothing)]));
    let mut s = started(rules, &setup).unwrap();
    let artifacts = vec![CROWN, RING];
    draw(
        &mut s,
        0,
        Resolved::Bank {
            level: 0,
            artifacts,
        },
        &[],
    );
    s
}

/// The guards a bank of these tests is drawn with: five stacks of 2 peasants.
fn guards() -> Vec<ArmyStack> {
    (0..5).map(|slot| stack(slot, PEASANT, 2)).collect()
}

/// `count` of `creature` in `slot`.
fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The offer of `kind` the bank of type `k` at [`AT`] makes red's hero.
fn offer(rules: &Rules, k: &str, kind: OfferKind) -> Offer {
    Offer {
        hero: HERO,
        object_type: type_of(rules, k),
        anchor: AT,
        kind,
    }
}

/// Red's hero steps onto the bank at [`AT`]: the outcome and the events.
fn step(
    rules: &Rules,
    s: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![AT],
    };
    run(rules, s, heroes, &command)
}

/// Red runs `command`: the outcome and the events other than the steps'.
fn run(
    rules: &Rules,
    s: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    command: &AdventureCommand,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let outcome = s
        .apply(
            &rules.0,
            &rules.1,
            heroes,
            (RED, Weekday::FIRST, &mut stream()),
            command,
            &mut events,
        )
        .unwrap();
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    (outcome, events)
}

/// Red accepts `offer`: the outcome and the events.
fn accept(
    rules: &Rules,
    s: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    offer: &Offer,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let outcome = s.accept_offer((&rules.0, &rules.1, heroes), RED, offer, &mut events);
    (outcome, events)
}

/// The word of `count` guards (the tests' monster parameters).
fn word(rules: &Rules, count: u32) -> u8 {
    rules.1.params().monsters().unwrap().count_word(count)
}

/// The events telling `seen_by` its mark on the bank of type `k` at `anchor` is set, then the
/// hint `hint` (none: no hint shows).
fn marked(
    rules: &Rules,
    (k, anchor): (&str, Pos),
    hint: Option<(Option<CreatureTypeId>, u8)>,
    seen_by: Viewers,
) -> Vec<AdventureEvent> {
    let object_type = type_of(rules, k);
    let mark = AdventureEvent {
        kind: AdventureEventKind::ObjectMarked {
            object_type,
            anchor,
        },
        seen_by,
    };
    let hint = hint.map(|(creature, word)| AdventureEvent {
        kind: AdventureEventKind::GuardsChanged {
            object_type,
            anchor,
            creature,
            count_word: Some(word),
        },
        seen_by,
    });
    [mark].into_iter().chain(hint).collect()
}

/// The event telling red the bank of type `k` at [`AT`] is looted.
fn looted(rules: &Rules, k: &str) -> AdventureEvent {
    AdventureEvent {
        kind: AdventureEventKind::ObjectEmpty {
            object_type: type_of(rules, k),
            anchor: AT,
        },
        seen_by: Viewers::NONE.with(RED),
    }
}

/// What `player` knows of the bank of type `k` at [`AT`]: that it was visited, nothing more.
fn known(rules: &Rules, s: &AdventureState, player: PlayerId) -> Vec<KnownObject> {
    s.observe(&rules.0, &rules.1, player).known
}

/// The battle of red's hero against these guards at [`AT`], on the tile's grass.
fn battle_against(army: Vec<ArmyStack>, s: &AdventureState, rules: &Rules) -> AdventureOutcome {
    let attacker = s.hero(HERO).unwrap();
    AdventureOutcome {
        battle: Some(s.battle_at(&rules.1, attacker, Defender::Bank { army }, AT)),
        ..AdventureOutcome::default()
    }
}

// covers: ADV-BNK-003, ADV-VIS-006, ADV-BNK-006, ADV-TEL-008
#[test]
fn an_attack_bank_marks_its_team_at_once_and_asks_unless_looted() {
    let rules = rules();
    let heroes = store(hero());
    let mut s = bank(&rules, "t:utopia");
    let fight = offer(&rules, "t:utopia", OfferKind::Fight);
    // The marks at once, told with the hint (E08 T125a: the creature, the word of 10), then the
    // question; blue knows nothing.
    let (outcome, events) = step(&rules, &mut s, &heroes);
    assert_eq!(
        outcome,
        AdventureOutcome {
            offer: Some(fight.clone()),
            ..AdventureOutcome::default()
        }
    );
    let red = Viewers::NONE.with(RED);
    let hint = Some((Some(PEASANT), word(&rules, 10)));
    assert_eq!(events, marked(&rules, ("t:utopia", AT), hint, red));
    assert!(s.knows(RED, 0) && !s.knows(BLUE, 0));
    let visited = KnownObject {
        object_type: fight.object_type,
        anchor: AT,
        known: Known::Visited,
    };
    assert_eq!(known(&rules, &s, RED), std::slice::from_ref(&visited));
    assert!(known(&rules, &s, BLUE).is_empty());
    // Declined: nothing more; the marks stay, the hero stays on the entrance.
    let before = s.clone();
    let mut events = Vec::new();
    let declined = s.decline_offer((&rules.0, &rules.1, &heroes), &fight, &mut events);
    assert_eq!((declined, events.len()), (AdventureOutcome::default(), 0));
    assert_eq!(s, before);
    assert_eq!(s.hero(HERO).unwrap().pos, AT);
    // `Revisit` on the entrance asks again (ADV-TEL-008), the mark held: nothing told.
    let revisit = AdventureCommand::Revisit { hero: HERO };
    let (outcome, events) = run(&rules, &mut s, &heroes, &revisit);
    assert_eq!((outcome.offer, events.len()), (Some(fight), 0));
    // Looted: told so, and no question.
    s.visits_mut().empty(0);
    let (outcome, events) = run(&rules, &mut s, &heroes, &revisit);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events, [looted(&rules, "t:utopia")]);
    // Its view says it was visited, never that it is looted.
    assert_eq!(known(&rules, &s, RED), [visited]);
}

// covers: ADV-BNK-003, ADV-VIS-006, ADV-BNK-004, ADV-BNK-006, ADV-TEL-008
#[test]
fn a_search_bank_asks_first_and_marks_after_a_yes() {
    let rules = rules();
    let heroes = store(hero());
    let mut s = bank(&rules, "t:crypt");
    let search = offer(&rules, "t:crypt", OfferKind::Search);
    // The question first, no mark; declined, still none.
    let (outcome, events) = step(&rules, &mut s, &heroes);
    assert_eq!(outcome.offer, Some(search.clone()));
    assert!(events.is_empty());
    assert!(!s.knows(RED, 0));
    let mut events = Vec::new();
    s.decline_offer((&rules.0, &rules.1, &heroes), &search, &mut events);
    assert!(!s.knows(RED, 0));
    // Accepted: the marks with the hint, then the battle against its guards at once.
    let (outcome, events) = accept(&rules, &mut s, &heroes, &search);
    assert!(s.knows(RED, 0) && !s.knows(BLUE, 0));
    assert_eq!(outcome, battle_against(guards(), &s, &rules));
    let hint = Some((Some(PEASANT), word(&rules, 10)));
    let red = Viewers::NONE.with(RED);
    assert_eq!(events, marked(&rules, ("t:crypt", AT), hint, red));
    // Looted: still asked first.
    s.visits_mut().empty(0);
    let revisit = AdventureCommand::Revisit { hero: HERO };
    let (outcome, events) = run(&rules, &mut s, &heroes, &revisit);
    assert_eq!((outcome.offer, events.len()), (Some(search), 0));
}

// covers: ADV-BNK-003, ADV-VIS-006, ADV-BNK-006
#[test]
fn a_bank_s_marks_go_to_every_player_of_the_visitor_s_team() {
    // Blue on red's team (E10 T3a; E10 design note §4): an attack kind marks both at once, told
    // with the hint to both, and both see it visited with its hint.
    let rules = rules();
    let heroes = store(hero());
    let both = Viewers::NONE.with(RED).with(BLUE);
    let ten = Some((Some(PEASANT), word(&rules, 10)));
    let mut s = bank_in(&rules, "t:utopia", allies);
    let fight = offer(&rules, "t:utopia", OfferKind::Fight);
    let (outcome, events) = step(&rules, &mut s, &heroes);
    assert_eq!(outcome.offer, Some(fight));
    assert_eq!(events, marked(&rules, ("t:utopia", AT), ten, both));
    assert!(s.knows(RED, 0) && s.knows(BLUE, 0));
    assert_eq!(known(&rules, &s, BLUE), known(&rules, &s, RED));
    let peasants = hint(Some(PEASANT), word(&rules, 10));
    assert_eq!(view(&rules, &s, BLUE), (true, peasants));
    // A search kind: no mark at the question, both marked after red's yes.
    let mut crypt = bank_in(&rules, "t:crypt", allies);
    let search = offer(&rules, "t:crypt", OfferKind::Search);
    step(&rules, &mut crypt, &heroes);
    assert!(!crypt.knows(RED, 0) && !crypt.knows(BLUE, 0));
    let (_, events) = accept(&rules, &mut crypt, &heroes, &search);
    assert!(crypt.knows(RED, 0) && crypt.knows(BLUE, 0));
    assert_eq!(events, marked(&rules, ("t:crypt", AT), ten, both));
}

// covers: ADV-BNK-004, GAM-BAT-001, ADV-BNK-006
#[test]
fn a_yes_fights_the_guards_as_they_stand_at_the_entrance() {
    let rules = rules();
    let heroes = store(hero());
    let mut s = bank(&rules, "t:utopia");
    let fight = offer(&rules, "t:utopia", OfferKind::Fight);
    step(&rules, &mut s, &heroes);
    // Unowned, without a hero, unsplit by slot, at the entrance where the hero stands, on the
    // tile's ground.
    let (outcome, _) = accept(&rules, &mut s, &heroes, &fight);
    let start = outcome.battle.clone().unwrap();
    assert_eq!(start.defender, Defender::Bank { army: guards() });
    assert_eq!(start.defender.owner(), None);
    assert_eq!((start.tile, start.town), (AT, None));
    assert_eq!(
        (start.terrain, start.battlefield.clone()),
        s.battle_ground(&rules.1, AT)
    );
    // Lost or retreated: the survivors by slot stay, not looted; the next yes fights them. Red's
    // team visited it: told the hint's change (E08 T125a: two creatures, the word of 3).
    let context = (&rules.0, &rules.1, &heroes[..]);
    let survivors = [stack(1, PEASANT, 2), stack(4, PIKEMAN, 1)];
    let mut events = Vec::new();
    let lost = s.end_unowned_battle(context, (HERO, AT), (&survivors, false), &mut events);
    assert_eq!(lost, AdventureOutcome::default());
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::GuardsChanged {
                object_type: fight.object_type,
                anchor: AT,
                creature: None,
                count_word: Some(word(&rules, 3)),
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    assert_eq!(s.guards_left(0), survivors);
    assert!(!s.is_emptied(0));
    let (again, _) = accept(&rules, &mut s, &heroes, &fight);
    assert_eq!(again, battle_against(survivors.to_vec(), &s, &rules));
    // No hero on the map: no battle.
    let mut gone = s.clone();
    gone.heroes.clear();
    assert_eq!(
        accept(&rules, &mut gone, &heroes, &fight).0,
        AdventureOutcome::default()
    );
}

// covers: ADV-BNK-004, GAM-BAT-001
#[test]
fn a_deck_banks_battle_is_on_the_deck_from_land_and_from_a_boat() {
    let rules = rules();
    let heroes = store(hero());
    let battle = rules.1.params().battle_terrains().unwrap();
    let deck = (battle.sea, Some(battle.deck.clone()));
    let search = offer(&rules, "t:wreck", OfferKind::Search);
    // A hero on land (here grass beside the bank): on the sea's terrain and its deck, the bank
    // counted as a boat.
    let mut s = bank(&rules, "t:wreck");
    let start = accept(&rules, &mut s, &heroes, &search).0.battle.unwrap();
    assert_eq!((start.terrain, start.battlefield), deck.clone());
    assert_eq!(start.defender, Defender::Bank { army: guards() });
    // A hero in a boat: the deck too, not the two boats.
    let mut afloat = bank(&rules, "t:wreck");
    afloat.heroes[0].boat = Some(0);
    let start = accept(&rules, &mut afloat, &heroes, &search)
        .0
        .battle
        .unwrap();
    assert_eq!((start.terrain, start.battlefield), deck);
    // A land kind beside it stays on its tile's ground.
    let mut land = bank(&rules, "t:crypt");
    let crypt = offer(&rules, "t:crypt", OfferKind::Search);
    let start = accept(&rules, &mut land, &heroes, &crypt).0.battle.unwrap();
    assert_eq!(
        (start.terrain, start.battlefield),
        land.battle_ground(&rules.1, AT)
    );
}

// covers: ADV-BNK-005, ADV-RWD-001, GAM-DEC-007, ADV-ARM-007, ADV-BNK-006
#[test]
fn a_won_bank_gives_its_reward_after_the_levels_and_stays_looted() {
    let rules = rules();
    let heroes = store(hero());
    let context = (&rules.0, &rules.1, &heroes[..]);
    let mut s = bank(&rules, "t:utopia");
    let mut events = Vec::new();
    // Won: none stand, and the reward waits for the winner's levels.
    let won = s.end_unowned_battle(context, (HERO, AT), (&[], true), &mut events);
    assert_eq!(
        won,
        AdventureOutcome {
            reward_after: Some((HERO, AT)),
            ..AdventureOutcome::default()
        }
    );
    assert!(s.guards_left(0).is_empty() && !s.is_emptied(0));
    // Given: the artifacts in the draw's order (the relic first), the gold, the wolves onto the
    // hero's first free slot, told to red; then looted, on the map.
    let reward = s.give_won(context, (HERO, AT), &mut events);
    assert_eq!(
        reward,
        AdventureOutcome {
            gains: vec![(GOLD, 500)],
            hero: Some(HeroGains {
                hero: HERO,
                changes: vec![add(CROWN), add(RING)],
                experience: 0,
            }),
            ..AdventureOutcome::default()
        }
    );
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::CreaturesJoined {
                hero: HERO,
                creature: WOLF,
                count: 3,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, PEASANT, 1), stack(1, WOLF, 3)]
    );
    assert!(s.is_emptied(0) && !s.is_removed(0));
    // A second `give_won` finds it looted; a visit tells so and asks nothing.
    let mut events = Vec::new();
    let again = s.give_won(context, (HERO, AT), &mut events);
    assert_eq!((again, events.len()), (AdventureOutcome::default(), 0));
    // Its mark set anew with no hint (looted), then told so.
    let (outcome, events) = step(&rules, &mut s, &heroes);
    assert_eq!(outcome, AdventureOutcome::default());
    let mut expected = marked(&rules, ("t:utopia", AT), None, Viewers::NONE.with(RED));
    expected.push(looted(&rules, "t:utopia"));
    assert_eq!(events, expected);
}

// covers: ADV-BNK-005, GAM-DEC-004, ADV-RWD-001
#[test]
fn a_rewards_creature_without_room_is_the_army_dialogs_group_and_its_artifacts_may_be_lost() {
    let rules = rules();
    // Seven slots of other creatures, a full backpack and no free finger: the wolves find no
    // room, both artifacts are lost; the gold still comes.
    let mut s = bank(&rules, "t:utopia");
    let creatures = [PEASANT, PIKEMAN, GNOLL];
    s.heroes[0].army = (0..7)
        .map(|slot| stack(slot, creatures[usize::from(slot) % 3], 1))
        .collect();
    let details = HeroDetails {
        worn: vec![Some(RING.into())],
        backpack: vec![RING.into(), RING.into()],
        ..hero()
    };
    let heroes = store(details);
    let context = (&rules.0, &rules.1, &heroes[..]);
    s.end_unowned_battle(context, (HERO, AT), (&[], true), &mut Vec::new());
    let mut events = Vec::new();
    let reward = s.give_won(context, (HERO, AT), &mut events);
    assert_eq!(
        reward,
        AdventureOutcome {
            gains: vec![(GOLD, 500)],
            no_room: Some(NoRoom {
                hero: HERO,
                group: vec![stack(0, WOLF, 3)],
            }),
            ..AdventureOutcome::default()
        }
    );
    assert!(events.is_empty());
    assert!(s.is_emptied(0));
    // A level without a creature gives none.
    let mut hive = bank(&rules, "t:hive");
    let heroes = store(hero());
    let context = (&rules.0, &rules.1, &heroes[..]);
    hive.end_unowned_battle(context, (HERO, AT), (&[], true), &mut Vec::new());
    let reward = hive.give_won(context, (HERO, AT), &mut events);
    assert_eq!((reward.no_room, reward.gains), (None, vec![(GOLD, 500)]));
    assert_eq!(hive.hero(HERO).unwrap().army, [stack(0, PEASANT, 1)]);
}

// covers: ADV-BNK-004, ADV-BNK-005
#[test]
fn a_draw_leaves_no_guards_and_the_next_yes_gives_the_reward_at_once() {
    let rules = rules();
    let heroes = store(hero());
    let context = (&rules.0, &rules.1, &heroes[..]);
    let mut s = bank(&rules, "t:crypt");
    // A draw: none stand, not looted, no reward.
    let drawn = s.end_unowned_battle(context, (HERO, AT), (&[], false), &mut Vec::new());
    assert_eq!(drawn, AdventureOutcome::default());
    assert!(s.guards_left(0).is_empty() && !s.is_emptied(0));
    // The next yes: the reward at once, no battle, no levels waited for; then looted.
    let search = offer(&rules, "t:crypt", OfferKind::Search);
    let (outcome, events) = accept(&rules, &mut s, &heroes, &search);
    assert_eq!(outcome.battle, None);
    assert_eq!(outcome.reward_after, None);
    assert_eq!(outcome.gains, [(GOLD, 500)]);
    assert_eq!(
        outcome.hero.map(|gains| gains.changes),
        Some(vec![add(CROWN), add(RING)])
    );
    assert_eq!(
        events.len(),
        2,
        "the mark (no hint: none stand), the wolves joined"
    );
    assert!(s.is_emptied(0) && s.knows(RED, 0));
    // A won battle's reward still waiting finds it looted meanwhile: nothing.
    let mut events = Vec::new();
    let late = s.give_won(context, (HERO, AT), &mut events);
    assert_eq!((late, events.len()), (AdventureOutcome::default(), 0));
}

// covers: HER-ADV-008, ADV-BNK-003
#[test]
fn a_looted_search_gives_its_once_bonus_once_per_kind_until_the_next_battle() {
    let rules = rules();
    let heroes = store(hero());
    let looted_at = |k: &str, x: u16| object(&rules, x, k, Holds::Nothing);
    let mut s = map_of(
        &rules,
        vec![
            looted_at("t:crypt", AT.x),
            looted_at("t:wreck", 3),
            looted_at("t:utopia", 4),
        ],
    );
    for index in 0..3 {
        s.visits_mut().empty(index);
    }
    let at = |k: &str, x: u16| Offer {
        anchor: Pos::new(x, 0, 0),
        ..offer(&rules, k, OfferKind::Search)
    };
    let once = |mark| VisitBonus {
        mark,
        morale: -1,
        luck: 0,
    };
    // A looted Crypt's yes: its mark (no hint), told so, −1 morale by mark 15; a second gives
    // nothing more.
    let (outcome, events) = accept(&rules, &mut s, &heroes, &at("t:crypt", AT.x));
    assert_eq!(outcome, AdventureOutcome::default());
    let mut expected = marked(&rules, ("t:crypt", AT), None, Viewers::NONE.with(RED));
    expected.push(looted(&rules, "t:crypt"));
    assert_eq!(events, expected);
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(15)]);
    accept(&rules, &mut s, &heroes, &at("t:crypt", AT.x));
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(15)]);
    // A looted Shipwreck's beside it: −2 in all, by marks 14 and 15.
    accept(&rules, &mut s, &heroes, &at("t:wreck", 3));
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(14), once(15)]);
    let morale: i8 = s
        .visit_marks(HERO)
        .bonuses()
        .once
        .iter()
        .map(|b| b.morale)
        .sum();
    assert_eq!(morale, -2);
    // An attack kind looted while its question was open: told so, no bonus.
    let fight = Offer {
        kind: OfferKind::Fight,
        ..at("t:utopia", 4)
    };
    let (_, events) = accept(&rules, &mut s, &heroes, &fight);
    assert_eq!(events.len(), 1);
    assert_eq!(s.visit_marks(HERO).bonuses().once.len(), 2);
    // The next battle's end clears them (GAM-BAT-014); a looted Crypt gives its −1 again.
    s.clear_battle_bonuses(&rules.0, &rules.1, &[HERO]);
    assert!(s.visit_marks(HERO).bonuses().once.is_empty());
    accept(&rules, &mut s, &heroes, &at("t:crypt", AT.x));
    assert_eq!(s.visit_marks(HERO).bonuses().once, [once(15)]);
}

// covers: ADV-BNK-005
#[test]
fn a_bank_without_its_draw_is_looted_and_gives_nothing() {
    // None in a valid game (the build draws every bank, `draws.rs`): the reward reads the draw.
    let rules = rules();
    let heroes = store(hero());
    let mut s = bank(&rules, "t:utopia");
    draw(&mut s, 0, Resolved::Nothing, &[]);
    assert_eq!(Resolved::Nothing.bank(), None);
    let context = (&rules.0, &rules.1, &heroes[..]);
    s.end_unowned_battle(context, (HERO, AT), (&[], true), &mut Vec::new());
    let reward = s.give_won(context, (HERO, AT), &mut Vec::new());
    assert_eq!(reward, AdventureOutcome::default());
    assert!(s.is_emptied(0));
}

/// `player`'s view of the bank at [`AT`]: its mark and its guards.
fn view(rules: &Rules, s: &AdventureState, player: PlayerId) -> (bool, Option<GuardsView>) {
    let seen = s.observe(&rules.0, &rules.1, player).objects;
    let bank = seen.into_iter().find(|o| o.anchor == AT).unwrap();
    (bank.visited, bank.guards)
}

/// The hint as a view shows it: no army.
fn hint(creature: Option<CreatureTypeId>, count_word: u8) -> Option<GuardsView> {
    Some(GuardsView {
        creature,
        count_word,
        army: None,
    })
}

// covers: ADV-BNK-006, ADV-VIS-006, ADV-OWN-003
#[test]
fn a_bank_shows_the_team_that_visited_it_its_hint_and_nothing_more() {
    let rules = rules();
    let heroes = store(hero());
    let context = (&rules.0, &rules.1, &heroes[..]);
    let mut s = bank(&rules, "t:utopia");
    let object_type = type_of(&rules, "t:utopia");
    // Not visited: its type to every viewer, no mark, no hint (blue seeing its tile too).
    reveal(&mut s.explored[usize::from(BLUE.raw())], AT, 1);
    assert_eq!(view(&rules, &s, RED), (false, None));
    assert_eq!(view(&rules, &s, BLUE), (false, None));
    // Red's attack visit marks red's team: the word of 10 and the creature, never the counts.
    step(&rules, &mut s, &heroes);
    let peasants = hint(Some(PEASANT), word(&rules, 10));
    assert_eq!(view(&rules, &s, RED), (true, peasants.clone()));
    assert_eq!(view(&rules, &s, BLUE), (false, None));
    // Blue's team visited it too: a battle's end tells both the hint's change (two creatures: the
    // generic text; the word of 3).
    s.visits_mut().learn(BLUE, 0);
    assert_eq!(view(&rules, &s, BLUE), (true, peasants));
    let both = Viewers::NONE.with(RED).with(BLUE);
    let changed = |creature, count_word| AdventureEvent {
        kind: AdventureEventKind::GuardsChanged {
            object_type,
            anchor: AT,
            creature,
            count_word,
        },
        seen_by: both,
    };
    let survivors = [stack(1, PEASANT, 2), stack(4, PIKEMAN, 1)];
    let mut events = Vec::new();
    s.end_unowned_battle(context, (HERO, AT), (&survivors, false), &mut events);
    assert_eq!(events, [changed(None, Some(word(&rules, 3)))]);
    assert_eq!(view(&rules, &s, BLUE), (true, hint(None, word(&rules, 3))));
    // The same hint after another battle (one creature left of each, the word unchanged): nothing.
    let mut events = Vec::new();
    let fewer = [stack(1, PEASANT, 1), stack(4, PIKEMAN, 1)];
    s.end_unowned_battle(context, (HERO, AT), (&fewer, false), &mut events);
    assert!(events.is_empty(), "{events:?}");
    // Won: none stand, told as none; visited, no hint, before and after the loot.
    let mut events = Vec::new();
    s.end_unowned_battle(context, (HERO, AT), (&[], true), &mut events);
    assert_eq!(events, [changed(None, None)]);
    assert_eq!(view(&rules, &s, RED), (true, None));
    let mut events = Vec::new();
    s.give_won(context, (HERO, AT), &mut events);
    assert!(s.is_emptied(0));
    assert!(events.iter().all(|e| !matches!(
        e.kind,
        AdventureEventKind::GuardsChanged { .. } | AdventureEventKind::ObjectMarked { .. }
    )));
    assert_eq!(view(&rules, &s, RED), (true, None));
    assert_eq!(view(&rules, &s, BLUE), (true, None));
}

// covers: ADV-BNK-006
#[test]
fn a_battles_end_at_a_bank_no_team_visited_tells_nothing() {
    let rules = rules();
    let heroes = store(hero());
    let context = (&rules.0, &rules.1, &heroes[..]);
    let mut s = bank(&rules, "t:crypt");
    let mut events = Vec::new();
    let survivors = [stack(1, PEASANT, 2)];
    s.end_unowned_battle(context, (HERO, AT), (&survivors, false), &mut events);
    assert!(events.is_empty());
    assert_eq!(view(&rules, &s, RED), (false, None));
}

/// The store with hero 1's details `skills`, the others' none.
fn movers(skills: &[(SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![
        Arc::new(hero_details(&[])),
        Arc::new(hero_details(skills)),
        Arc::new(hero_details(&[])),
    ]
}

/// A started one-row coast of `row` (`g` grass, `~` sea, the map's coast flags set), red's hero 1
/// on (0, 0), a bank of each type `k` on `x` of the row in order (one tile, blocked and visited
/// on it), drawn when the game was built.
fn coast(rules: &Rules, row: &str, banks: &[(&str, u16)]) -> AdventureState {
    let mut setup = coasted(&[row], vec![hero_at(1, 0, Pos::new(0, 0, 0), &[PEASANT])]);
    setup.map.objects.extend(
        banks
            .iter()
            .map(|&(k, x)| object(rules, x, k, Holds::Nothing)),
    );
    started(rules, &setup).unwrap()
}

/// Red's hero 1 moves along `path`: the outcome and the events but the steps, or the error (a
/// rejected move leaves the state and emits nothing).
fn walk(
    rules: &Rules,
    s: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let before = s.clone();
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let result = s.apply(
        &rules.0,
        &rules.1,
        heroes,
        (RED, Weekday::FIRST, &mut stream()),
        &command,
        &mut events,
    );
    if result.is_err() {
        assert!(events.is_empty());
        assert_eq!(*s, before);
    }
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    result.map(|outcome| (outcome, events))
}

/// Tile `x` of the row.
fn x(x: u16) -> Pos {
    Pos::new(x, 0, 0)
}

/// Tiles `xs` of the row.
fn xs(xs: core::ops::RangeInclusive<u16>) -> Vec<Pos> {
    xs.map(x).collect()
}

/// The points red's hero 1 has left on `pos` as its reachable tiles give them.
fn reach_left(
    rules: &Rules,
    s: &AdventureState,
    heroes: &[Arc<HeroDetails>],
    pos: Pos,
) -> Option<u32> {
    s.reach(&rules.0, &rules.1, heroes, HERO)
        .unwrap()
        .points_left(pos)
}

// covers: ADV-SEA-002, ADV-MOV-009, ADV-MOV-010, ADV-BNK-003, ADV-BNK-004, GAM-BAT-001
#[test]
fn a_shipwrecks_water_entrance_is_visited_from_the_coast_whatever_the_mode() {
    // E13-12's W4 and E13-5's W1 (design note §9): the hero stays on land, its step paid as
    // taken; its question first (a search kind), no mark before the yes; the battle on the deck.
    let rules = rules();
    let search = Offer {
        hero: HERO,
        object_type: type_of(&rules, "t:shipwreck"),
        anchor: x(1),
        kind: OfferKind::Search,
    };
    let battle = rules.1.params().battle_terrains().unwrap();
    for skills in [&[][..], &[(FLYING, 3)], &[(WATER_WALKING, 3)]] {
        let heroes = movers(skills);
        let mut s = coast(&rules, "g~g", &[("t:shipwreck", 1)]);
        let left = reach_left(&rules, &s, &heroes, x(1));
        assert!(left.is_some(), "{skills:?}");
        let (outcome, events) = walk(&rules, &mut s, &heroes, &[x(1)]).unwrap();
        assert_eq!(outcome.offer, Some(search.clone()), "{skills:?}");
        assert!(events.is_empty(), "{events:?}");
        let hero = s.hero(HERO).unwrap();
        assert_eq!((hero.pos, Some(hero.movement)), (x(0), left), "{skills:?}");
        assert!(!s.knows(RED, 0));
        // Nothing past the visit.
        let mut again = coast(&rules, "g~g", &[("t:shipwreck", 1)]);
        let past = walk(&rules, &mut again, &heroes, &xs(1..=2));
        if skills.is_empty() {
            assert_eq!(past, Err(AdventureCommandError::PastVisit { step: 1 }));
        }
        // The yes: its mark with the hint, the battle at its entrance on the deck.
        let (outcome, _) = accept(&rules, &mut s, &heroes, &search);
        let start = outcome.battle.unwrap();
        assert_eq!(start.tile, x(1));
        assert_eq!(
            (start.terrain, start.battlefield),
            (battle.sea, Some(battle.deck.clone()))
        );
        assert!(s.knows(RED, 0));
    }
}

// covers: ADV-MOV-009, ADV-MOV-010, ADV-SEA-002
#[test]
fn a_flight_passes_a_shipwrecks_water_entrance_and_a_water_walk_never_does() {
    // E13-12's P1 and P2 (from land and from water: no visit on the way), P3 (no path across).
    let rules = rules();
    let flyer = movers(&[(FLYING, 3)]);
    let swimmer = movers(&[(WATER_WALKING, 3)]);
    // From land: the flight over it to the shore beyond, visiting nothing; a water walk's step
    // onto it is the visit, which ends the move.
    let fresh = coast(&rules, "g~g", &[("t:shipwreck", 1)]);
    assert!(reach_left(&rules, &fresh, &flyer, x(2)).is_some());
    assert_eq!(reach_left(&rules, &fresh, &swimmer, x(2)), None);
    let mut s = fresh.clone();
    let (outcome, events) = walk(&rules, &mut s, &flyer, &xs(1..=2)).unwrap();
    assert_eq!((outcome, events.len()), (AdventureOutcome::default(), 0));
    assert_eq!(s.hero(HERO).unwrap().pos, x(2));
    let mut s = fresh.clone();
    assert_eq!(
        walk(&rules, &mut s, &swimmer, &xs(1..=2)),
        Err(AdventureCommandError::PastVisit { step: 1 })
    );
    // From water: a flight passes it, ending on it nowhere; a water walk never steps onto it.
    let fresh = coast(&rules, "g~~~g", &[("t:shipwreck", 2)]);
    assert!(reach_left(&rules, &fresh, &flyer, x(4)).is_some());
    assert_eq!(reach_left(&rules, &fresh, &swimmer, x(4)), None);
    let mut s = fresh.clone();
    walk(&rules, &mut s, &flyer, &xs(1..=4)).unwrap();
    assert_eq!(s.hero(HERO).unwrap().pos, x(4));
    let mut s = fresh.clone();
    assert_eq!(
        walk(&rules, &mut s, &flyer, &xs(1..=2)),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    assert_eq!(
        walk(&rules, &mut s, &swimmer, &xs(1..=4)),
        Err(AdventureCommandError::Blocked {
            step: 1,
            error: StandError::Blocked
        })
    );
    // Up off a closed top (a chart's, ADV-OBJ-004) a flight passes it too.
    let mut setup = coasted(
        &["g#~", "gCg", "ggg"],
        vec![hero_at(1, 0, Pos::new(0, 1, 0), &[PEASANT])],
    );
    setup
        .map
        .objects
        .push(object(&rules, 2, "t:shipwreck", Holds::Nothing));
    let mut s = started(&rules, &setup).unwrap();
    let path = [Pos::new(1, 1, 0), Pos::new(2, 0, 0), Pos::new(2, 1, 0)];
    walk(&rules, &mut s, &flyer, &path).unwrap();
    assert_eq!(s.hero(HERO).unwrap().pos, Pos::new(2, 1, 0));
}

// covers: ADV-MOV-009, ADV-SEA-002, ADV-OBJ-003
#[test]
fn the_derelict_ship_and_a_coast_tile_another_object_is_visited_from_stay_closed() {
    // D-46 stays the Derelict Ship's (a deck kind not of the coast): no walker's visit from the
    // coast, no flight over it.
    let rules = rules();
    let walker = movers(&[]);
    let flyer = movers(&[(FLYING, 3)]);
    let terrain = AdventureCommandError::Blocked {
        step: 0,
        error: StandError::Terrain,
    };
    let fresh = coast(&rules, "g~g", &[("t:wreck", 1)]);
    let mut s = fresh.clone();
    assert_eq!(walk(&rules, &mut s, &walker, &[x(1)]), Err(terrain));
    assert_eq!(
        walk(&rules, &mut s, &flyer, &xs(1..=2)),
        Err(AdventureCommandError::Blocked {
            step: 0,
            error: StandError::Blocked
        })
    );
    assert_eq!(reach_left(&rules, &fresh, &walker, x(1)), None);
    assert_eq!(reach_left(&rules, &fresh, &flyer, x(2)), None);
    // A Shipwreck's entrance another object is visited from after it (the last decides,
    // ADV-OBJ-003): closed as that object's.
    let fresh = coast(&rules, "g~g", &[("t:shipwreck", 1), ("t:crypt", 1)]);
    let mut s = fresh.clone();
    assert_eq!(walk(&rules, &mut s, &walker, &[x(1)]), Err(terrain));
    assert_eq!(reach_left(&rules, &fresh, &walker, x(1)), None);
    // A Shipwreck placed on land: its entrance a land neighbour's (ADV-OBJ-003), no coast's.
    let mut s = coast(&rules, "ggg", &[("t:shipwreck", 1)]);
    assert!(s.map().coast_entrances().is_empty());
    let (outcome, _) = walk(&rules, &mut s, &walker, &[x(1)]).unwrap();
    assert_eq!(outcome.offer.map(|o| o.kind), Some(OfferKind::Search));
    assert_eq!(s.hero(HERO).unwrap().pos, x(0));
    // A Shipwreck outside the walker's window (on the other level) changes nothing of its search.
    let mut setup = world(&["ggg", "g~g"], 2, vec![hero_at(1, 0, x(0), &[PEASANT])]);
    let below = Pos::new(1, 0, 1);
    setup.map.objects.push(ObjectSetup {
        anchor: below,
        ..object(&rules, 1, "t:shipwreck", Holds::Nothing)
    });
    let s = started(&rules, &setup).unwrap();
    assert_eq!(s.map().coast_entrances(), [below]);
    assert!(reach_left(&rules, &s, &walker, x(2)).is_some());
    assert_eq!(reach_left(&rules, &s, &walker, below), None);
}
