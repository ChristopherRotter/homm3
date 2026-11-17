//! Tests of the creature banks' visits, battles and rewards (E08 T124b2; increment 13's design note
//! §6 to §8, §12, §14, §15): an attack kind's marks at once and its question unless looted, a
//! search kind's question first and its marks after a yes, `Fight` and `Search` routed by the
//! kind, the battle against the guards as they stand at the entrance (a deck kind's on the deck),
//! the end's survivors, a draw's empty guards, the reward after the winner's levels (the creature
//! or the army dialog's group, the artifacts in the draw's order, the resources), the loot, a
//! looted search's once-bonus by its mark, and `Revisit`. The draw itself is `draws.rs`'.

use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{ArtifactTypeId, HeroParamsDef};
use h3_heroes::{HeroDetails, VisitBonus};

use super::draws::{object, rules_with};
use super::rewards::{
    AT, BLUE, GOLD, HERO, RED, RING, Rules, add, draw, hero, map_of, store, type_of,
};
use super::*;
use crate::command::{Defender, HeroGains, NoRoom};
use crate::observe::{Known, KnownObject};
use crate::ruleset::{
    BankDef, BankLevelDef, BankQuestion, ObjectKindDef, ObjectTypeDef, OnceBonus, Visit,
};
use crate::setup::Holds;
use crate::state::Resolved;
use crate::testing::{GNOLL, PEASANT, PIKEMAN, WOLF, key, stream};

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
/// fought on the deck), and the attack kind `t:hive` without a reward's creature.
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
            types.extend([
                bank("t:utopia", BankQuestion::Attack, None, false, level()),
                bank("t:crypt", BankQuestion::Search, Some(15), false, level()),
                bank("t:wreck", BankQuestion::Search, Some(14), true, level()),
                bank("t:hive", BankQuestion::Attack, None, false, hive),
            ]);
        },
    )
}

/// A started map with a bank of type `k` at [`AT`], drawn at its one level with the crown and
/// the ring (the relic first), its guards five stacks of 2 peasants.
fn bank(rules: &Rules, k: &str) -> AdventureState {
    let mut s = map_of(rules, vec![object(rules, AT.x, k, Holds::Nothing)]);
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

// covers: ADV-BNK-003, ADV-VIS-006
#[test]
fn an_attack_bank_marks_its_team_at_once_and_asks_unless_looted() {
    let rules = rules();
    let heroes = store(hero());
    let mut s = bank(&rules, "t:utopia");
    let fight = offer(&rules, "t:utopia", OfferKind::Fight);
    // The marks at once, then the question; nothing told of the marks yet (the hint's events are
    // T125a's), blue knows nothing.
    let (outcome, events) = step(&rules, &mut s, &heroes);
    assert_eq!(
        outcome,
        AdventureOutcome {
            offer: Some(fight.clone()),
            ..AdventureOutcome::default()
        }
    );
    assert!(events.is_empty(), "{events:?}");
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
    // `Revisit` on the entrance asks again (ADV-TEL-008).
    let revisit = AdventureCommand::Revisit { hero: HERO };
    assert_eq!(run(&rules, &mut s, &heroes, &revisit).0.offer, Some(fight));
    // Looted: told so, and no question.
    s.visits_mut().empty(0);
    let (outcome, events) = run(&rules, &mut s, &heroes, &revisit);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events, [looted(&rules, "t:utopia")]);
    // Its view says it was visited, never that it is looted.
    assert_eq!(known(&rules, &s, RED), [visited]);
}

// covers: ADV-BNK-003, ADV-VIS-006
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
    // Accepted: the marks, then the battle against its guards at once.
    let (outcome, events) = accept(&rules, &mut s, &heroes, &search);
    assert!(s.knows(RED, 0) && !s.knows(BLUE, 0));
    assert_eq!(outcome, battle_against(guards(), &s, &rules));
    assert!(events.is_empty());
    // Looted: still asked first.
    s.visits_mut().empty(0);
    let revisit = AdventureCommand::Revisit { hero: HERO };
    let (outcome, events) = run(&rules, &mut s, &heroes, &revisit);
    assert_eq!((outcome.offer, events.len()), (Some(search), 0));
}

// covers: ADV-BNK-004, GAM-BAT-001
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
    // Lost or retreated: the survivors by slot stay, not looted; the next yes fights them.
    let context = (&rules.0, &rules.1, &heroes[..]);
    let survivors = [stack(1, PEASANT, 2), stack(4, PIKEMAN, 1)];
    let mut events = Vec::new();
    let lost = s.end_unowned_battle(context, (HERO, AT), (&survivors, false), &mut events);
    assert_eq!((lost, events.len()), (AdventureOutcome::default(), 0));
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

// covers: ADV-BNK-005, ADV-RWD-001, GAM-DEC-007, ADV-ARM-007
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
    let (outcome, events) = step(&rules, &mut s, &heroes);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events, [looted(&rules, "t:utopia")]);
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
    assert_eq!(events.len(), 1, "the wolves joined");
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
    // A looted Crypt's yes: told so, −1 morale by mark 15; a second gives nothing more.
    let (outcome, events) = accept(&rules, &mut s, &heroes, &at("t:crypt", AT.x));
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events, [looted(&rules, "t:crypt")]);
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
