//! Tests of the heroes' part of a battle's outcome (E08 T19a2a): spell points, destroyed war
//! machines' artifacts, the winner's experience with Learning and +500 for a defeated hero, the
//! loser's artifacts captured; each change told to the hero's owner.

use h3_adventure::{AdventureEvent, AdventureEventKind};
use h3_contracts::{
    Army, BattleOutcome, BattleResult, BattleSide, HeroOutcome, StackOrigin, StackRecord,
};
use h3_heroes::{ArtifactInstance, ArtifactPlace, HeroChange};

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const CART: CreatureTypeId = CreatureTypeId::from_raw(2);

fn art(rules: &GameRuleset, k: &str) -> ArtifactInstance {
    rules.core().artifact_by_key(k).unwrap().into()
}

/// [`setup`] with custom details: red's hero has Expert Lore, a ring and the cart's artifact
/// worn and a charm in the backpack; blue's two rings, the cart's artifact and the spell book,
/// a scroll of Gust and a charm in the backpack.
fn equipped(rules: &GameRuleset) -> GameSetup {
    let core = rules.core();
    let lore = core.skill_by_key("t:lore").unwrap();
    let gust = core.spell_by_key("t:gust").unwrap();
    let [ring, cart, book, charm] =
        ["t:ring", "t:cart_art", "t:book", "t:charm"].map(|k| art(rules, k));
    let scroll = ArtifactInstance::scroll(art(rules, "t:scroll").artifact, gust);
    let mut setup = setup();
    setup.heroes[0].details = HeroDetailsSetup {
        skills: Some(vec![(lore, 3)]),
        artifacts: Some(HeroArtifactsSetup {
            worn: vec![ring, cart],
            backpack: vec![charm],
        }),
        ..HeroDetailsSetup::default()
    };
    setup.heroes[1].details = HeroDetailsSetup {
        artifacts: Some(HeroArtifactsSetup {
            worn: vec![ring, ring, cart, book],
            backpack: vec![scroll, charm],
        }),
        ..HeroDetailsSetup::default()
    };
    setup
}

fn record(origin: StackOrigin, creature: CreatureTypeId, start: u32, end: u32) -> StackRecord {
    StackRecord {
        origin,
        creature,
        start,
        end,
    }
}

/// A battle red's hero (the attacker) and blue's fought, ended by `result` with red's army
/// surviving: red lost 4 of 9 imps and its cart; blue all its stacks — 4 imps and 2 gogs of its
/// army, 2 imps raised in its first slot, 3 summoned imps — and its cart. Red's hero has 3 spell
/// points left, blue's 7.
fn outcome(result: BattleResult) -> BattleOutcome {
    let survivor = ArmyStack {
        slot: 1,
        creature: IMP,
        count: 5,
    };
    BattleOutcome {
        result,
        survivors: [
            Army {
                stacks: vec![survivor],
            },
            Army { stacks: vec![] },
        ],
        war_machines: [vec![], vec![]],
        stacks: [
            vec![
                record(StackOrigin::Army(1), IMP, 9, 5),
                record(StackOrigin::WarMachine, CART, 1, 0),
            ],
            vec![
                record(StackOrigin::Army(0), IMP, 4, 0),
                record(StackOrigin::Army(1), GOG, 2, 0),
                record(StackOrigin::WarMachine, CART, 1, 0),
                record(StackOrigin::Created, IMP, 3, 0),
                record(StackOrigin::Raised(0), IMP, 2, 0),
            ],
        ],
        surrender_cost: None,
        heroes: [3, 7].map(|mana| {
            Some(HeroOutcome {
                mana,
                learned: vec![],
            })
        }),
        rounds: 3,
    }
}

const SIDES: ([Option<PlayerId>; 2], [Option<HeroId>; 2]) =
    ([Some(RED), Some(BLUE)], [Some(RED_HERO), Some(BLUE_HERO)]);

/// `game` after the battle ended with `outcome`, and the changes told, by owner and hero.
fn end(
    game: &GameState,
    rules: &GameRuleset,
    sides: ([Option<PlayerId>; 2], [Option<HeroId>; 2]),
    outcome: &BattleOutcome,
) -> (GameState, Vec<(PlayerId, HeroId, HeroChange)>) {
    let mut ended = game.clone();
    let mut events = Vec::new();
    ended.end_battle(rules, sides, outcome, &mut events);
    let changes = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::HeroChanged {
                player,
                hero,
                change,
            } => Some((player, hero, change)),
            _ => None,
        })
        .collect();
    (ended, changes)
}

/// The hit points of `n` creatures of `creature`.
fn hp(rules: &GameRuleset, creature: CreatureTypeId, n: u32) -> u32 {
    n * rules.battle().creature(creature).unwrap().hit_points
}

// covers: GAM-BAT-002, GAM-BAT-004, HER-EXP-001, HER-WAR-002, HER-ART-004, HER-CHG-002
#[test]
fn a_victory_over_a_hero_gives_experience_and_its_artifacts() {
    let rules = rules();
    let game = GameState::new(&rules, &equipped(&rules), 5).unwrap();
    let outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
    let (ended, changes) = end(&game, &rules, SIDES, &outcome);
    // Killed: blue's army and raised imps, not its summoned imps or its cart; +500 for its hero;
    // × 1.15 for Expert Lore, rounded down.
    let killed = hp(&rules, IMP, 4) + hp(&rules, GOG, 2) + hp(&rules, IMP, 2);
    let experience = (killed + 500) * 115 / 100;
    let [ring, charm] = ["t:ring", "t:charm"].map(|k| art(&rules, k));
    let gust = rules.core().spell_by_key("t:gust").unwrap();
    let scroll = ArtifactInstance::scroll(art(&rules, "t:scroll").artifact, gust);
    let red = |change| (RED, RED_HERO, change);
    let blue = |change| (BLUE, BLUE_HERO, change);
    assert_eq!(
        changes,
        [
            red(HeroChange::SetSpellPoints(3)),
            red(HeroChange::RemoveArtifact(ArtifactPlace::Worn(1))),
            blue(HeroChange::SetSpellPoints(7)),
            blue(HeroChange::RemoveArtifact(ArtifactPlace::Worn(2))),
            red(HeroChange::GainExperience(experience)),
            // The worn ones first (the book stays), then the backpack from its end.
            red(HeroChange::AddArtifact(ring)),
            blue(HeroChange::RemoveArtifact(ArtifactPlace::Worn(0))),
            red(HeroChange::AddArtifact(ring)),
            blue(HeroChange::RemoveArtifact(ArtifactPlace::Worn(0))),
            red(HeroChange::AddArtifact(charm)),
            blue(HeroChange::RemoveArtifact(ArtifactPlace::Backpack(1))),
            red(HeroChange::AddArtifact(scroll)),
            blue(HeroChange::RemoveArtifact(ArtifactPlace::Backpack(0))),
        ]
    );
    let (red, blue) = (
        ended.hero(RED_HERO).unwrap(),
        ended.hero(BLUE_HERO).unwrap(),
    );
    // The second ring finds both ring positions taken: the backpack.
    assert_eq!(red.worn, [ring, ring]);
    assert_eq!(red.backpack, [charm, ring, charm, scroll]);
    assert_eq!(
        red.experience,
        game.hero(RED_HERO).unwrap().experience + experience
    );
    assert_eq!((red.mana, blue.mana), (3, 7));
    assert_eq!(blue.worn, [art(&rules, "t:book")]);
    assert!(blue.backpack.is_empty());
    assert_eq!(blue.experience, game.hero(BLUE_HERO).unwrap().experience);
    // The changes are what the store shows.
    for id in [RED_HERO, BLUE_HERO] {
        let mut details = game.hero(id).unwrap().clone();
        for &(_, _, change) in changes.iter().filter(|&&(_, hero, _)| hero == id) {
            h3_heroes::apply_change(&mut details, rules.core(), change);
        }
        assert_eq!(ended.hero(id), Some(&details));
    }
    // Each change shows to its owner only.
    let event = GameEvent::HeroChanged {
        player: RED,
        hero: RED_HERO,
        change: HeroChange::GainExperience(experience),
    };
    assert_eq!(project(&event, Viewer::Player(RED)), Some(event));
    assert_eq!(project(&event, Viewer::Player(BLUE)), None);
}

// covers: GAM-BAT-002, GAM-BAT-004
#[test]
fn a_retreat_or_a_surrender_gives_no_artifacts_and_no_500() {
    let rules = rules();
    let game = GameState::new(&rules, &equipped(&rules), 5).unwrap();
    let killed = hp(&rules, IMP, 4) + hp(&rules, GOG, 2) + hp(&rules, IMP, 2);
    for result in [
        BattleResult::Retreat(BattleSide::Defender),
        BattleResult::Surrender(BattleSide::Defender),
    ] {
        let (ended, changes) = end(&game, &rules, SIDES, &outcome(result));
        let gained = changes.iter().find_map(|&(_, _, c)| match c {
            HeroChange::GainExperience(points) => Some(points),
            _ => None,
        });
        assert_eq!(gained, Some(killed * 115 / 100), "{result:?}");
        assert!(
            !changes
                .iter()
                .any(|(_, _, c)| matches!(c, HeroChange::AddArtifact(_))),
            "{result:?}"
        );
        let blue = ended.hero(BLUE_HERO).unwrap();
        assert_eq!(blue.backpack.len(), 2, "{result:?}");
    }
}

// covers: GAM-BAT-002
#[test]
fn a_draw_gives_no_experience_but_spends_mana_and_machines() {
    let rules = rules();
    let game = GameState::new(&rules, &equipped(&rules), 5).unwrap();
    let mut draw = outcome(BattleResult::Draw);
    draw.survivors[0].stacks.clear();
    let (ended, changes) = end(&game, &rules, SIDES, &draw);
    assert_eq!(
        changes.iter().map(|&(_, _, c)| c).collect::<Vec<_>>(),
        [
            HeroChange::SetSpellPoints(3),
            HeroChange::RemoveArtifact(ArtifactPlace::Worn(1)),
            HeroChange::SetSpellPoints(7),
            HeroChange::RemoveArtifact(ArtifactPlace::Worn(2)),
        ]
    );
    assert_eq!(
        ended.hero(RED_HERO).unwrap().experience,
        game.hero(RED_HERO).unwrap().experience
    );
}

// covers: GAM-BAT-002, GAM-BAT-004
#[test]
fn unchanged_mana_lost_machines_without_artifacts_and_absent_heroes_change_nothing() {
    let rules = rules();
    // Neither hero wears the cart's artifact; red keeps its spell points; blue reports none.
    let game = GameState::new(&rules, &setup(), 5).unwrap();
    let mana = game.hero(RED_HERO).unwrap().mana;
    let mut quiet = outcome(BattleResult::Victory(BattleSide::Attacker));
    quiet.heroes = [
        Some(HeroOutcome {
            mana,
            learned: vec![],
        }),
        None,
    ];
    let (_, changes) = end(&game, &rules, SIDES, &quiet);
    let killed = hp(&rules, IMP, 4) + hp(&rules, GOG, 2) + hp(&rules, IMP, 2);
    assert_eq!(
        changes,
        [(RED, RED_HERO, HeroChange::GainExperience(killed + 500))]
    );
    // A victory over a side without a hero: no 500, nothing to capture.
    let sides = ([Some(RED), None], [Some(RED_HERO), None]);
    let (_, changes) = end(&game, &rules, sides, &quiet);
    assert_eq!(
        changes,
        [(RED, RED_HERO, HeroChange::GainExperience(killed))]
    );
    // A winning side without a hero: no one gains; nothing killed gives nothing.
    let sides = ([Some(RED), Some(BLUE)], [None, Some(BLUE_HERO)]);
    let (_, changes) = end(&game, &rules, sides, &quiet);
    assert!(changes.is_empty());
    let mut none = quiet;
    none.stacks[1].clear();
    let sides = ([Some(RED), None], [Some(RED_HERO), None]);
    let (_, changes) = end(&game, &rules, sides, &none);
    assert!(changes.is_empty());
}

// covers: GAM-BAT-004, HER-CHG-002
#[test]
fn the_capture_stops_at_the_first_artifact_the_winner_has_no_room_for() {
    let rules = rules();
    let mut setup = equipped(&rules);
    // Red's backpack is full and both ring positions taken: blue keeps everything.
    let charm = art(&rules, "t:charm");
    let ring = art(&rules, "t:ring");
    let red = setup.heroes[0].details.artifacts.as_mut().unwrap();
    red.worn = vec![ring, ring];
    red.backpack = vec![charm; 63];
    let game = GameState::new(&rules, &setup, 5).unwrap();
    let mut outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
    outcome.stacks[0].pop();
    let (ended, changes) = end(&game, &rules, SIDES, &outcome);
    // One backpack place: the first ring takes it, the second finds none and the capture stops.
    let added: Vec<HeroChange> = changes
        .iter()
        .map(|&(_, _, c)| c)
        .filter(|c| matches!(c, HeroChange::AddArtifact(_)))
        .collect();
    assert_eq!(added, [HeroChange::AddArtifact(ring)]);
    let blue = ended.hero(BLUE_HERO).unwrap();
    assert_eq!(blue.worn, [ring, art(&rules, "t:book")]);
    assert_eq!(blue.backpack.len(), 2);
    assert_eq!(ended.hero(RED_HERO).unwrap().backpack.len(), 64);
}

// covers: GAM-BAT-002
#[test]
fn the_heroes_removal_comes_before_their_changes() {
    let rules = rules();
    let game = GameState::new(&rules, &equipped(&rules), 5).unwrap();
    let mut ended = game.clone();
    let mut events = Vec::new();
    let outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
    ended.end_battle(&rules, SIDES, &outcome, &mut events);
    let removed = events.iter().position(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::HeroRemoved { .. },
                ..
            })
        )
    });
    let changed = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroChanged { .. }));
    assert!(removed.unwrap() < changed.unwrap());
}

/// [`equipped`] with red's hero (the winner) also at `dark` Dark (Necromancy: 10/20/70 %) and
/// `sense` Sense (Wisdom), wearing the spell book when `book`.
fn necromancer(rules: &GameRuleset, dark: u8, sense: u8, book: bool) -> GameSetup {
    let core = rules.core();
    let mut setup = equipped(rules);
    let red = &mut setup.heroes[0].details;
    let skills = red.skills.as_mut().unwrap();
    for (k, level) in [("t:dark", dark), ("t:sense", sense)] {
        if level > 0 {
            skills.push((core.skill_by_key(k).unwrap(), level));
        }
    }
    if book {
        red.artifacts
            .as_mut()
            .unwrap()
            .worn
            .push(art(rules, "t:book"));
    }
    setup
}

/// The `CreaturesRaised` events among `events`, as (player, hero, creature, count).
fn raised(events: &[GameEvent]) -> Vec<(PlayerId, HeroId, CreatureTypeId, u32)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::CreaturesRaised {
                player,
                hero,
                creature,
                count,
            } => Some((player, hero, creature, count)),
            _ => None,
        })
        .collect()
}

/// [`rules`] under the profile `t:single` (quirk Q-0005).
fn single_rules() -> GameRuleset {
    let core = Arc::new(core_with(false, "t:single"));
    let part = part_with(&core, vec![88, 25], 2);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

// covers: GAM-BAT-006, HER-NEC-001, HER-NEC-002
#[test]
fn the_winners_necromancy_raises_from_every_killed_record_into_its_army() {
    let rules = rules();
    let game = GameState::new(&rules, &necromancer(&rules, 3, 0, false), 5).unwrap();
    let outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
    let mut ended = game.clone();
    let mut events = Vec::new();
    ended.end_battle(&rules, SIDES, &outcome, &mut events);
    // 70 % of each killed record's hit points at most the imp's 4, in imps (4 hit points), at
    // most the record's killed: 4 imps 2 (2.8), 2 gogs 1 (1.4), the cart 0 (0.7), 3 summoned imps
    // 2 (2.1), 2 raised imps 1 (1.4).
    assert_eq!(raised(&events), [(RED, RED_HERO, IMP, 6)]);
    // They join red's surviving imps (slot 1: 5).
    let army = &ended.adventure().hero(RED_HERO).unwrap().army;
    assert_eq!(
        army[..],
        [ArmyStack {
            slot: 1,
            creature: IMP,
            count: 11,
        }]
    );
    // After the experience, before the capture; told to red only.
    let at = |f: fn(&GameEvent) -> bool| events.iter().position(f).unwrap();
    let experience = at(|e| {
        matches!(
            e,
            GameEvent::HeroChanged {
                change: HeroChange::GainExperience(_),
                ..
            }
        )
    });
    let raise = at(|e| matches!(e, GameEvent::CreaturesRaised { .. }));
    let capture = at(|e| {
        matches!(
            e,
            GameEvent::HeroChanged {
                change: HeroChange::AddArtifact(_),
                ..
            }
        )
    });
    assert!(experience < raise && raise < capture);
    let event = events[raise];
    assert_eq!(project(&event, Viewer::Player(RED)), Some(event));
    assert_eq!(project(&event, Viewer::Player(BLUE)), None);
    // A retreat or a surrender of the loser raises too; the loser's Necromancy never does.
    for result in [
        BattleResult::Retreat(BattleSide::Defender),
        BattleResult::Surrender(BattleSide::Defender),
    ] {
        let (mut again, mut events) = (game.clone(), Vec::new());
        again.end_battle(&rules, SIDES, &outcome_of(result), &mut events);
        assert_eq!(raised(&events), [(RED, RED_HERO, IMP, 6)], "{result:?}");
    }
}

/// [`outcome`] with `result`.
fn outcome_of(result: BattleResult) -> BattleOutcome {
    outcome(result)
}

// covers: GAM-BAT-006
#[test]
fn necromancy_raises_at_least_one_and_nothing_from_nothing() {
    let rules = rules();
    // Basic Dark: 10 %. Only the cart was killed: 0.1 of 4 hit points, rounded down 0, but one.
    let game = GameState::new(&rules, &necromancer(&rules, 1, 0, false), 5).unwrap();
    let mut outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
    outcome.stacks[1] = vec![record(StackOrigin::WarMachine, CART, 1, 0)];
    let (mut ended, mut events) = (game.clone(), Vec::new());
    ended.end_battle(&rules, SIDES, &outcome, &mut events);
    assert_eq!(raised(&events), [(RED, RED_HERO, IMP, 1)]);
    // Nothing killed: nothing raised, no event.
    outcome.stacks[1] = vec![record(StackOrigin::Army(0), IMP, 4, 4)];
    let (mut ended, mut events) = (game.clone(), Vec::new());
    ended.end_battle(&rules, SIDES, &outcome, &mut events);
    assert!(raised(&events).is_empty());
    assert_eq!(
        ended.adventure().hero(RED_HERO).unwrap().army,
        outcome.survivors[0].stacks
    );
    // A hero without Necromancy raises nothing.
    let game = GameState::new(&rules, &equipped(&rules), 5).unwrap();
    let outcome = outcome_of(BattleResult::Victory(BattleSide::Attacker));
    let (mut ended, mut events) = (game, Vec::new());
    ended.end_battle(&rules, SIDES, &outcome, &mut events);
    assert!(raised(&events).is_empty());
}

// covers: GAM-BAT-006
#[test]
fn with_quirk_q0005_the_count_is_the_originals_single_precision_arithmetic() {
    // 10 imps killed at 70 %: exactly 7; the original's f32(0.7) is below 0.7, so 40 × it / 4
    // truncates to 6.
    for (rules, count) in [(rules(), 7), (single_rules(), 6)] {
        let game = GameState::new(&rules, &necromancer(&rules, 3, 0, false), 5).unwrap();
        let mut outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
        outcome.stacks[1] = vec![record(StackOrigin::Army(0), IMP, 10, 0)];
        let (mut ended, mut events) = (game, Vec::new());
        ended.end_battle(&rules, SIDES, &outcome, &mut events);
        assert_eq!(raised(&events), [(RED, RED_HERO, IMP, count)]);
    }
}

// covers: GAM-BAT-007, HER-SPL-005, HER-SPL-006
#[test]
fn the_winner_learns_its_eagle_eye_picks_with_its_book_up_to_its_level() {
    let rules = rules();
    let core = rules.core();
    let (gust, gate) = (
        core.spell_by_key("t:gust").unwrap(),
        core.spell_by_key("t:gate").unwrap(),
    );
    let mut outcome = outcome(BattleResult::Victory(BattleSide::Attacker));
    for hero in outcome.heroes.iter_mut().flatten() {
        hero.learned = vec![gust, gate];
    }
    let learned = |setup: &GameSetup| {
        let game = GameState::new(&rules, setup, 5).unwrap();
        let (_, changes) = end(&game, &rules, SIDES, &outcome);
        changes
            .into_iter()
            .filter(|(_, _, c)| matches!(c, HeroChange::LearnSpell(_)))
            .collect::<Vec<_>>()
    };
    let learn = |spell| (RED, RED_HERO, HeroChange::LearnSpell(spell));
    // Without Sense up to level 2: Gust, not Gate; with Basic Sense both; without its book none.
    // Blue's picks (it lost) are never learned.
    assert_eq!(learned(&necromancer(&rules, 0, 0, true)), [learn(gust)]);
    assert_eq!(
        learned(&necromancer(&rules, 0, 1, true)),
        [learn(gust), learn(gate)]
    );
    assert!(learned(&necromancer(&rules, 0, 3, false)).is_empty());
    // After the raised creatures, before the capture; the store learned them.
    let game = GameState::new(&rules, &necromancer(&rules, 3, 1, true), 5).unwrap();
    let (mut ended, mut events) = (game, Vec::new());
    ended.end_battle(&rules, SIDES, &outcome, &mut events);
    let raise = events
        .iter()
        .position(|e| matches!(e, GameEvent::CreaturesRaised { .. }))
        .unwrap();
    let first_learned = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::LearnSpell(_),
                    ..
                }
            )
        })
        .unwrap();
    assert!(raise < first_learned);
    let spells = &ended.hero(RED_HERO).unwrap().spells;
    assert!(spells.contains(&gust) && spells.contains(&gate));
}

// covers: GAM-BAT-006
#[test]
fn the_count_takes_the_shares_own_single_precision_value() {
    // A share with its own single-precision value (a specialist's, HER-NEC-001) is the one the
    // original computes with: f32(0.7) given as bits counts as the share of 7000 basis points.
    let killed = [record(StackOrigin::Army(0), IMP, 10, 0)];
    let share = |single| h3_heroes::Necromancy {
        share: h3_contracts::Rate { bp: 7000, single },
        creature: IMP,
    };
    let bits = NonZeroU32::new(0x3f33_3333);
    let count = crate::state::raised_count;
    assert_eq!(count(&single_rules(), &killed, share(bits)), 6);
    assert_eq!(count(&single_rules(), &killed, share(None)), 6);
    assert_eq!(count(&rules(), &killed, share(bits)), 7);
}
