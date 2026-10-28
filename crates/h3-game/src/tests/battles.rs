//! Tests of battles through the game (E08 T19a): a visit of another player's hero opens a battle
//! both players own, with its setup from the map and the store of heroes and its seeds by the
//! profile; its commands go to the owner of the side to act; its events and observation to its
//! owners; its outcome's armies, removals and a surrender's gold apply when it ends.

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, ObjectTypeId, Offer, Viewers,
};
use h3_battle::{BattleState, Command, Event, EventKind, LegalCommands, Observation};
use h3_content::ResourceId;
use h3_contracts::{
    Army, BattleOutcome, BattleResult, BattleSetup, BattleSide, HeroOutcome, HeroSex,
};
use h3_heroes::HeroChange;
use h3_kernel::Rng;

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const GREEN: PlayerId = PlayerId::new(2);
const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(0);

/// Red's hero's attack on blue's hero at (2, 2) from (1, 1).
fn attack() -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: vec![Pos::new(1, 1, 0), Pos::new(2, 2, 0)],
    })
}

/// [`setup`] with a third player (green, without heroes) and blue's tile on the coast.
fn three_players() -> GameSetup {
    let mut setup = setup();
    setup.players.push(PlayerSetup {
        colour: 2,
        human: true,
    });
    setup.map.tiles[10].coast = true;
    setup
}

/// The rules with the profile `profile` (`t:faithful`, or `t:tile` with ADV-QUIRK-5).
fn rules_for(profile: &str) -> GameRuleset {
    let core = Arc::new(core_with(false, profile));
    let part = part(&core);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

/// The battle's setup as GAM-BAT-001 builds it from `game` before the attack, the obstacles drawn
/// from `obstacles`: the heroes' armies and formations, their battle heroes (with their maximum
/// spell points and the sexes the setup gave) and war machines, blue's tile (on the coast: rock
/// and the shore), the owners' gold; since E08 T54 the game's difficulty, whether each owner
/// counts as human, the owners' colours (red 0, blue 1), the third picture (nothing around) and
/// no ground under red's hero.
fn expected_setup(game: &GameState, rules: &GameRuleset, obstacles: u64) -> BattleSetup {
    let core = rules.core();
    let side = |id: HeroId| {
        let on_map = game.adventure().hero(id).unwrap();
        let details = game.hero(id).unwrap();
        let mut hero = h3_heroes::derive_battle_hero(details, core).unwrap();
        hero.magic.max_mana = h3_heroes::max_mana(details, core);
        hero.sex = game.hero_sex(id);
        let machines = h3_heroes::war_machines(details, core);
        (
            Army {
                stacks: on_map.army.clone(),
            },
            hero,
            machines,
        )
    };
    let ((attacker, red, red_machines), (defender, blue, blue_machines)) =
        (side(RED_HERO), side(BLUE_HERO));
    let mut setup = BattleSetup {
        attacker,
        defender,
        heroes: [Some(red), Some(blue)],
        terrain: Some(TerrainId::from_raw(1)),
        battlefield: Some(key("t:shore")),
        war_machines: [red_machines, blue_machines],
        formations: [Formation::Loose; 2],
        funds: [RED, BLUE].map(|player| game.treasury(player).unwrap()[usize::from(GOLD.raw())]),
        difficulty: game.adventure().difficulty(),
        human: [RED, BLUE].map(|player| game.human(player).unwrap()),
        background: Some(2),
        colours: [Some(0), Some(1)],
        ..BattleSetup::default()
    };
    setup.obstacles =
        h3_battle::generate_setup_obstacles(rules.battle(), &setup, &mut Rng::from_seed(obstacles));
    setup
}

/// The game after red's attack, with the events and the state before it.
fn attacked(rules: &GameRuleset, seed: u64) -> (GameState, Vec<GameEvent>, GameState) {
    let mut game = GameState::new(rules, &three_players(), seed).unwrap();
    let before = game.clone();
    let mut events = Vec::new();
    apply(&mut game, rules, Actor::Player(RED), attack(), &mut events).unwrap();
    (game, events, before)
}

/// The battle's legal commands for `side`.
fn legal(battle: &BattleState, rules: &GameRuleset, side: BattleSide) -> Vec<GameCommand> {
    let mut out = LegalCommands::new();
    h3_battle::legal_commands(battle, rules.battle(), side, &mut out);
    out.iter().map(|&c| GameCommand::Battle(c)).collect()
}

/// The owner of the side to act in `game`'s battle (red's), and that side.
fn to_act(game: &GameState) -> (PlayerId, BattleSide) {
    let battle = game.battle(RED).unwrap();
    let side = battle.state().side_to_act().unwrap();
    (battle.owners()[side.index()].unwrap(), side)
}

// covers: GAM-BAT-001, GAM-BAT-004
#[test]
fn a_visit_of_another_players_hero_opens_a_battle_both_own() {
    let rules = rules();
    let (game, events, before) = attacked(&rules, 5);
    // The battle's seed, then the obstacles' (strict: no ADV-QUIRK-5), from red's stream.
    let mut stream = before.stream(RED);
    let seed = stream.next_u64();
    let obstacles = stream.next_u64();
    assert_eq!(game.stream(RED), stream);
    assert_eq!(game.stream(BLUE), before.stream(BLUE));
    let setup = expected_setup(&before, &rules, obstacles);
    let mut started = Vec::new();
    let expected = BattleState::start(rules.battle(), &setup, seed, &mut started).unwrap();
    let battle = game.battle(RED).unwrap();
    assert_eq!(battle.state(), &expected);
    assert_eq!(game.battle(BLUE), Some(battle));
    assert_eq!(game.battle(GREEN), None);
    assert_eq!(battle.owners(), [Some(RED), Some(BLUE)]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), Some(BLUE_HERO)]);
    assert_eq!(battle.tile(), Pos::new(2, 2, 0));
    assert_eq!(
        (
            battle.side_of(RED),
            battle.side_of(BLUE),
            battle.side_of(GREEN)
        ),
        (Some(BattleSide::Attacker), Some(BattleSide::Defender), None)
    );
    // Red's hero stays on (1, 1), the visit paid as a step: 1500 - 2 × 141.
    let red = game.adventure().hero(RED_HERO).unwrap();
    assert_eq!((red.pos, red.movement), (Pos::new(1, 1, 0), 1218));
    // The step, the start told to both owners, the battle's start's events by their visibility.
    let both = Viewers::NONE.with(RED).with(BLUE);
    let seen = |event: &Event| match event.visibility {
        h3_battle::Visibility::Public => both,
        h3_battle::Visibility::Side(BattleSide::Attacker) => Viewers::NONE.with(RED),
        h3_battle::Visibility::Side(BattleSide::Defender) => Viewers::NONE.with(BLUE),
    };
    let mut told = vec![GameEvent::BattleStarted {
        heroes: [Some(RED_HERO), Some(BLUE_HERO)],
        tile: Pos::new(2, 2, 0),
        seen_by: both,
    }];
    told.extend(started.iter().map(|&event| GameEvent::Battle {
        event,
        seen_by: seen(&event),
    }));
    assert!(matches!(events[0], GameEvent::Adventure(_)));
    assert_eq!(events[1..], told[..]);
    assert!(!started.is_empty());
    // The battle is in the hash, and a fork shares it.
    assert_ne!(game.state_hash(), before.state_hash());
    let fork = game.clone();
    assert_eq!(fork.state_hash(), game.state_hash());
}

// covers: GAM-BAT-001, GAM-DEC-001
#[test]
fn only_the_owner_of_the_side_to_act_commands_the_battle() {
    let rules = rules();
    let (game, _, _) = attacked(&rules, 5);
    let (owner, side) = to_act(&game);
    let waiting = if owner == RED { BLUE } else { RED };
    let battle = game.battle(RED).unwrap().state().clone();
    // The legal commands: the battle's for the owner to act, none for the other nor green.
    let listed = legal(&battle, &rules, side);
    assert!(!listed.is_empty());
    assert_eq!(legal_commands(&game, &rules, Actor::Player(owner)), listed);
    assert_eq!(legal_commands(&game, &rules, Actor::Player(waiting)), []);
    assert_eq!(legal_commands(&game, &rules, Actor::Player(GREEN)), []);
    // E08 T30: a battle has no default answer (the server lane's), nor has a player without one.
    assert_eq!(default_answer(&game, &rules, owner), None);
    assert_eq!(default_answer(&game, &rules, GREEN), None);
    // Red, whose turn runs, may neither move nor end its turn.
    assert_eq!(
        legal_moves(&game, &rules, Actor::Player(RED), RED_HERO),
        None
    );
    let rejected = |actor: PlayerId, command: GameCommand| {
        let mut probe = game.clone();
        let mut events = Vec::new();
        let result = apply(
            &mut probe,
            &rules,
            Actor::Player(actor),
            command,
            &mut events,
        );
        assert_eq!(probe, game);
        assert_eq!(probe.state_hash(), game.state_hash());
        assert!(events.is_empty());
        result.unwrap_err()
    };
    use GameCommandError as E;
    assert_eq!(rejected(RED, GameCommand::EndTurn), E::DecisionOpen);
    assert_eq!(rejected(RED, attack()), E::DecisionOpen);
    assert_eq!(
        rejected(owner, GameCommand::Answer(Answer::Offer(Consent::Accept))),
        E::BattleOpen
    );
    let any = listed[0].clone();
    assert_eq!(rejected(waiting, any.clone()), E::NotYourSide);
    assert_eq!(rejected(GREEN, any.clone()), E::NoBattle);
    assert_eq!(
        rejected(PlayerId::new(5), any.clone()),
        E::UnknownPlayer(PlayerId::new(5))
    );
    assert_eq!(
        rejected(owner, GameCommand::Battle(Command::EndTactics)),
        E::Battle(
            h3_battle::validate(&battle, rules.battle(), side, Command::EndTactics).unwrap_err()
        )
    );
    // Every listed command applies, in the owner's turn or not.
    for command in &listed {
        let mut probe = game.clone();
        apply(
            &mut probe,
            &rules,
            Actor::Player(owner),
            command.clone(),
            &mut NoopSink,
        )
        .unwrap();
        assert_ne!(probe, game);
    }
}

// covers: GAM-BAT-001
#[test]
fn each_owner_sees_its_side_and_no_one_else_the_battle() {
    let rules = rules();
    let (game, events, _) = attacked(&rules, 5);
    let battle = game.battle(RED).unwrap();
    let view = |player| observe(&game, &rules, Viewer::Player(player)).battle;
    let side = |side| Some(h3_battle::observe(battle.state(), rules.battle(), side));
    assert_eq!(view(RED), side(BattleSide::Attacker));
    assert_eq!(view(BLUE), side(BattleSide::Defender));
    assert_eq!(view(GREEN), None::<Observation>);
    // The events: each owner its own, green nothing of the battle.
    for event in &events[1..] {
        assert_eq!(project(event, Viewer::Player(GREEN)), None);
        for player in [RED, BLUE] {
            let projected = project(event, Viewer::Player(player));
            let seen_by = match *event {
                GameEvent::BattleStarted { seen_by, .. }
                | GameEvent::Battle { seen_by, .. }
                | GameEvent::BattleEnded { seen_by, .. } => seen_by,
                _ => panic!("{event:?} is no battle event"),
            };
            assert_eq!(projected.is_some(), seen_by.contains(player));
            if let Some(
                GameEvent::Battle { seen_by, .. } | GameEvent::BattleStarted { seen_by, .. },
            ) = projected
            {
                assert_eq!(seen_by, Viewers::NONE.with(player));
            }
        }
    }
    let ended = GameEvent::BattleEnded {
        heroes: [Some(RED_HERO), Some(BLUE_HERO)],
        result: BattleResult::Draw,
        seen_by: Viewers::NONE.with(RED).with(BLUE),
    };
    assert_eq!(
        project(&ended, Viewer::Player(BLUE)),
        Some(GameEvent::BattleEnded {
            heroes: [Some(RED_HERO), Some(BLUE_HERO)],
            result: BattleResult::Draw,
            seen_by: Viewers::NONE.with(BLUE),
        })
    );
    assert_eq!(project(&ended, Viewer::Player(GREEN)), None);
}

/// Plays `game`'s battle to its end, each owner to act picking its last legal command (an attack
/// where it can), and returns the events of the last command, checked against the battle's own
/// outcome of it.
fn play_out(game: &mut GameState, rules: &GameRuleset) -> (Vec<GameEvent>, BattleOutcome) {
    for _ in 0..2000 {
        let (owner, side) = to_act(game);
        let listed = legal_commands(game, rules, Actor::Player(owner));
        let fights = |c: &&GameCommand| !matches!(c, GameCommand::Battle(c) if c.leaves_battle());
        let command = listed.iter().rfind(fights).unwrap().clone();
        let mut battle = game.battle(RED).unwrap().state().clone();
        let mut events = Vec::new();
        apply(
            game,
            rules,
            Actor::Player(owner),
            command.clone(),
            &mut events,
        )
        .unwrap();
        let GameCommand::Battle(command) = command else {
            panic!("{command:?} is no battle command")
        };
        h3_battle::apply(&mut battle, rules.battle(), side, command, &mut Vec::new()).unwrap();
        if let Some(outcome) = battle.outcome() {
            return (events, outcome);
        }
        assert_eq!(game.battle(RED).unwrap().state(), &battle);
    }
    panic!("the battle did not end");
}

// covers: GAM-BAT-002, GAM-BAT-003
#[test]
fn a_battle_played_to_its_end_applies_its_outcome() {
    let rules = rules();
    for seed in [5, 6, 7] {
        let (mut game, _, _) = attacked(&rules, seed);
        let before = game.clone();
        let (events, outcome) = play_out(&mut game, &rules);
        let winner = outcome.result.winner().unwrap();
        let heroes = [RED_HERO, BLUE_HERO];
        let (won, lost) = (heroes[winner.index()], heroes[winner.opponent().index()]);
        // The decision closed: both players act again as before (red in its turn).
        assert_eq!((game.battle(RED), game.battle(BLUE)), (None, None));
        assert_eq!(
            legal_commands(&game, &rules, Actor::Player(RED)),
            [GameCommand::EndTurn]
        );
        // The winner keeps its survivors; the loser left the map, told where it was seen.
        assert_eq!(
            game.adventure().hero(won).unwrap().army,
            outcome.survivors[winner.index()].stacks
        );
        assert_eq!(game.adventure().hero(lost), None);
        let lost_at = before.adventure().hero(lost).unwrap().pos;
        let both = Viewers::NONE.with(RED).with(BLUE);
        let ended = events
            .iter()
            .position(|e| matches!(e, GameEvent::BattleEnded { .. }))
            .unwrap();
        // The winner gains the hit points the loser's army lost and 500 for its hero (E08
        // T19a2a); neither hero has artifacts or spells to change.
        let killed: u32 = outcome.stacks[winner.opponent().index()]
            .iter()
            .map(|s| (s.start - s.end) * rules.battle().creature(s.creature).unwrap().hit_points)
            .sum();
        assert!(killed > 0);
        assert_eq!(
            events[ended..],
            [
                GameEvent::BattleEnded {
                    heroes: [Some(RED_HERO), Some(BLUE_HERO)],
                    result: outcome.result,
                    seen_by: both,
                },
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::HeroRemoved {
                        hero: lost,
                        pos: lost_at,
                    },
                    seen_by: both,
                }),
                GameEvent::HeroChanged {
                    player: [RED, BLUE][winner.index()],
                    hero: won,
                    change: HeroChange::GainExperience(killed + 500),
                },
            ]
        );
        let experience = |game: &GameState, id| game.hero(id).unwrap().experience;
        assert_eq!(
            experience(&game, won),
            experience(&before, won) + killed + 500
        );
        assert_eq!(experience(&game, lost), experience(&before, lost));
        // The treasuries did not change; the store keeps both heroes' details.
        assert_eq!(game.treasury(RED), before.treasury(RED));
        assert_eq!(game.heroes().len(), 2);
    }
}

/// A sink that keeps its events but says it does not record (`Sink::RECORDS`).
struct Quiet(Vec<GameEvent>);

impl Sink for Quiet {
    const RECORDS: bool = false;

    fn emit(&mut self, event: GameEvent) {
        self.0.push(event);
    }
}

/// The game's battle sink records what the game's sink records (E08 T19b1): a sink that does not
/// gets no walker's path hex by hex, the state is the same either way.
#[test]
fn a_battle_reports_a_walkers_path_to_a_recording_sink_only() {
    let rules = rules();
    let (mut loud, _, _) = attacked(&rules, 5);
    let mut quiet = loud.clone();
    let (mut heard, mut said) = (Vec::new(), Quiet(Vec::new()));
    while loud.battle(RED).is_some() {
        let (owner, _) = to_act(&loud);
        let listed = legal_commands(&loud, &rules, Actor::Player(owner));
        let fights = |c: &&GameCommand| !matches!(c, GameCommand::Battle(c) if c.leaves_battle());
        let command = listed.iter().rfind(fights).unwrap().clone();
        apply(
            &mut loud,
            &rules,
            Actor::Player(owner),
            command.clone(),
            &mut heard,
        )
        .unwrap();
        apply(&mut quiet, &rules, Actor::Player(owner), command, &mut said).unwrap();
        assert_eq!(loud, quiet);
    }
    // Only the moves differ: a quiet sink gets a walk as one move.
    assert!(said.0.len() < heard.len());
    let not_a_move = |e: &&GameEvent| match e {
        GameEvent::Battle { event, .. } => !matches!(event.kind, EventKind::Moved { .. }),
        _ => true,
    };
    assert!(
        said.0
            .iter()
            .filter(not_a_move)
            .eq(heard.iter().filter(not_a_move))
    );
}

// covers: GAM-BAT-003, GAM-BAT-005
#[test]
fn a_surrender_pays_the_winners_owner_and_a_retreat_leaves_the_map() {
    let rules = rules();
    let (game, _, _) = attacked(&rules, 5);
    let (owner, side) = to_act(&game);
    let other = if owner == RED { BLUE } else { RED };
    let (leaver, stayer) = if side == BattleSide::Attacker {
        (RED_HERO, BLUE_HERO)
    } else {
        (BLUE_HERO, RED_HERO)
    };
    // A surrender: the price from the surrendering side's owner to the other side's.
    let mut surrendered = game.clone();
    let mut events = Vec::new();
    let surrender = GameCommand::Battle(Command::Surrender);
    apply(
        &mut surrendered,
        &rules,
        Actor::Player(owner),
        surrender,
        &mut events,
    )
    .unwrap();
    let price = game
        .battle(RED)
        .unwrap()
        .state()
        .surrender_price(rules.battle(), side)
        .unwrap();
    assert!(price > 0);
    assert_eq!(
        events[events.len() - 2..],
        [
            GameEvent::ResourcePaid {
                player: owner,
                resource: GOLD,
                amount: price,
            },
            GameEvent::ResourceGained {
                player: other,
                resource: GOLD,
                amount: price,
            },
        ]
    );
    assert_eq!(surrendered.treasury(owner), Some(&[2000 - price][..]));
    assert_eq!(surrendered.treasury(other), Some(&[2000 + price][..]));
    assert_eq!(surrendered.adventure().hero(leaver), None);
    let stays = surrendered.adventure().hero(stayer).unwrap();
    assert_eq!(&stays.army, &game.adventure().hero(stayer).unwrap().army);
    // A retreat: the hero leaves the map, no gold moves.
    let mut retreated = game.clone();
    let retreat = GameCommand::Battle(Command::Retreat);
    apply(
        &mut retreated,
        &rules,
        Actor::Player(owner),
        retreat,
        &mut NoopSink,
    )
    .unwrap();
    assert_eq!(retreated.adventure().hero(leaver), None);
    assert!(retreated.adventure().hero(stayer).is_some());
    assert_eq!(retreated.treasury(owner), game.treasury(owner));
    assert_eq!(retreated.treasury(other), game.treasury(other));
}

/// An outcome of `result` with these army survivors (by side) and surrender price.
fn outcome(result: BattleResult, survivors: [&[ArmyStack]; 2], cost: Option<u32>) -> BattleOutcome {
    BattleOutcome {
        result,
        survivors: survivors.map(|stacks| Army {
            stacks: stacks.to_vec(),
        }),
        war_machines: [vec![], vec![]],
        stacks: [vec![], vec![]],
        surrender_cost: cost,
        heroes: [
            Some(HeroOutcome {
                mana: 0,
                learned: vec![],
            }),
            None,
        ],
        rounds: 1,
    }
}

// covers: GAM-BAT-002, GAM-BAT-003, GAM-BAT-005
#[test]
fn every_result_takes_the_heroes_that_did_not_win_off_the_map() {
    let rules = rules();
    let game = GameState::new(&rules, &three_players(), 5).unwrap();
    let stack = ArmyStack {
        slot: 0,
        creature: IMP,
        count: 3,
    };
    let sides = (
        [Some(RED), Some(BLUE)],
        [Some(RED_HERO), Some(BLUE_HERO)],
        Pos::new(0, 0, 0),
    );
    let end = |outcome: BattleOutcome| {
        let mut ended = game.clone();
        let mut events = Vec::new();
        ended.end_battle(&rules, sides, &outcome, &mut events);
        let on_map =
            [RED_HERO, BLUE_HERO].map(|id| ended.adventure().hero(id).map(|h| h.army.clone()));
        (on_map, ended, events)
    };
    let attacker = BattleSide::Attacker;
    // A victory with survivors: the winner keeps them.
    let (on_map, ..) = end(outcome(
        BattleResult::Victory(attacker),
        [&[stack], &[]],
        None,
    ));
    assert_eq!(on_map, [Some(vec![stack]), None]);
    // A draw: both leave.
    let (on_map, ..) = end(outcome(BattleResult::Draw, [&[], &[]], None));
    assert_eq!(on_map, [None, None]);
    // The other side retreated, but the winner has no army survivors (only stacks created in the
    // battle were left, BAT-END-001): it has lost too.
    let (on_map, ..) = end(outcome(
        BattleResult::Retreat(BattleSide::Defender),
        [&[], &[stack]],
        None,
    ));
    assert_eq!(on_map, [None, None]);
    // A surrender without a price moves no gold.
    let (on_map, ended, _) = end(outcome(
        BattleResult::Surrender(attacker),
        [&[stack], &[stack]],
        None,
    ));
    assert_eq!(on_map, [None, Some(vec![stack])]);
    assert_eq!(ended.treasury(RED), game.treasury(RED));
    // A surrender's price above the payer's gold: what it holds moves.
    let (_, ended, events) = end(outcome(
        BattleResult::Surrender(attacker),
        [&[stack], &[stack]],
        Some(5000),
    ));
    assert_eq!(ended.treasury(RED), Some(&[0][..]));
    assert_eq!(ended.treasury(BLUE), Some(&[4000][..]));
    assert!(events.contains(&GameEvent::ResourceGained {
        player: BLUE,
        resource: GOLD,
        amount: 2000,
    }));
}

// covers: GAM-BAT-010, GAM-BAT-012
#[test]
fn a_battles_setup_carries_the_difficulty_the_human_sides_colours_and_sexes() {
    let rules = rules();
    let mut setup = three_players();
    setup.difficulty = 0;
    // Blue counts as a computer player; red's hero is female by the map.
    setup.players[1].human = false;
    setup.heroes[0].details.sex = Some(HeroSex::Female);
    let mut game = GameState::new(&rules, &setup, 5).unwrap();
    assert_eq!(game.human(RED), Some(true));
    assert_eq!(game.human(BLUE), Some(false));
    assert_eq!(game.human(PlayerId::new(3)), None);
    assert_eq!(game.hero_sex(RED_HERO), Some(HeroSex::Female));
    let before = game.clone();
    apply(
        &mut game,
        &rules,
        Actor::Player(RED),
        attack(),
        &mut Vec::new(),
    )
    .unwrap();
    let mut stream = before.stream(RED);
    let seed = stream.next_u64();
    let expected = expected_setup(&before, &rules, stream.next_u64());
    assert_eq!(expected.difficulty, 0);
    assert_eq!(expected.human, [true, false]);
    let red = expected.heroes[0].as_ref().unwrap();
    assert_eq!(red.sex, Some(HeroSex::Female));
    assert_eq!(expected.heroes[1].as_ref().unwrap().sex, None);
    let battle = BattleState::start(rules.battle(), &expected, seed, &mut Vec::new()).unwrap();
    assert_eq!(game.battle(RED).unwrap().state(), &battle);
    // The flags and the sexes are hashed: another human flag or sex is another game.
    let mut other = setup.clone();
    other.players[1].human = true;
    let hash = |setup: &GameSetup| GameState::new(&rules, setup, 5).unwrap().state_hash();
    assert_ne!(hash(&other), hash(&setup));
    other = setup.clone();
    other.heroes[0].details.sex = Some(HeroSex::Male);
    assert_ne!(hash(&other), hash(&setup));
}

// covers: GAM-BAT-001
#[test]
fn the_obstacles_follow_the_tile_under_adv_quirk_5() {
    let rules = rules_for("t:tile");
    // 110 291 × 2 + 167 801 × 2 + 81 569.
    let seed = 637_753;
    let mut layouts = Vec::new();
    for game_seed in [5, 6] {
        let (game, _, before) = attacked(&rules, game_seed);
        // One draw from red's stream: the battle's seed.
        let mut stream = before.stream(RED);
        let battle_seed = stream.next_u64();
        assert_eq!(game.stream(RED), stream);
        let setup = expected_setup(&before, &rules, seed);
        let expected = BattleState::new(rules.battle(), &setup, battle_seed).unwrap();
        assert_eq!(game.battle(RED).unwrap().state(), &expected);
        layouts.push(setup.obstacles);
    }
    // The same tile, the same obstacles, whatever the game's seed.
    assert_eq!(layouts[0], layouts[1]);
    assert!(!layouts[0].is_empty());
}

// covers: GAM-BAT-001
#[test]
fn a_battle_its_part_refuses_does_not_open() {
    // The battle part gives the cart no kind, and red's hero wears its artifact: the battle
    // refuses the war machine (the parts disagree; a game of matching parts never does), and the
    // visit ends there.
    let core = Arc::new(core(false));
    let part = part(&core);
    let battle = battle_part_with(&core, 7, |pack| pack.creatures[2].abilities.clear());
    let rules = GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap();
    let mut setup = three_players();
    let cart = rules.core().artifact_by_key("t:cart_art").unwrap().into();
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn: wearing(rules.core(), &[cart]),
        backpack: vec![],
    });
    let mut game = GameState::new(&rules, &setup, 5).unwrap();
    let mut events = Vec::new();
    apply(&mut game, &rules, Actor::Player(RED), attack(), &mut events).unwrap();
    assert_eq!(game.battle(RED), None);
    assert_eq!(events.len(), 1);
    assert_eq!(
        game.adventure().hero(RED_HERO).unwrap().pos,
        Pos::new(1, 1, 0)
    );
}

// covers: GAM-BAT-001
#[test]
fn a_battle_command_with_an_offer_open_is_refused() {
    // Green owns an offer and red a battle (two decisions of different kinds open at once, as
    // in simultaneous play): green's battle command finds no battle, red's answer no offer.
    let rules = rules();
    let (mut game, _, _) = attacked(&rules, 5);
    let offer = Offer {
        hero: RED_HERO,
        object_type: ObjectTypeId::from_raw(1),
        anchor: Pos::new(3, 0, 0),
        kind: h3_adventure::OfferKind::Pass,
    };
    game.open(&rules, GREEN, offer.clone(), &mut NoopSink);
    assert_eq!(game.decision(GREEN), Some(&offer));
    assert_eq!(game.battle(GREEN), None);
    assert_eq!(game.decision(RED), None);
    let (owner, _) = to_act(&game);
    let command = legal_commands(&game, &rules, Actor::Player(owner))[0].clone();
    let mut probe = game.clone();
    assert_eq!(
        apply(
            &mut probe,
            &rules,
            Actor::Player(GREEN),
            command,
            &mut NoopSink
        ),
        Err(GameCommandError::NoBattle)
    );
    assert_eq!(probe, game);
    // Green's decline closes its offer and leaves the battle open.
    apply(
        &mut game,
        &rules,
        Actor::Player(GREEN),
        GameCommand::Answer(Answer::Offer(Consent::Decline)),
        &mut NoopSink,
    )
    .unwrap();
    assert_eq!(game.decision(GREEN), None);
    assert!(game.battle(RED).is_some());
}

// covers: GAM-BAT-001
#[test]
fn a_sides_event_goes_to_its_owner_alone() {
    use crate::state::seen_by;
    use h3_battle::Visibility;
    let owners = [Some(RED), Some(BLUE)];
    assert_eq!(
        seen_by(owners, Visibility::Public),
        Viewers::NONE.with(RED).with(BLUE)
    );
    assert_eq!(
        seen_by(owners, Visibility::Side(BattleSide::Attacker)),
        Viewers::NONE.with(RED)
    );
    assert_eq!(
        seen_by(owners, Visibility::Side(BattleSide::Defender)),
        Viewers::NONE.with(BLUE)
    );
    // An unowned side's events reach no player (monsters, E08 T20b).
    assert_eq!(
        seen_by([Some(RED), None], Visibility::Side(BattleSide::Defender)),
        Viewers::NONE
    );
}

#[test]
fn the_battle_errors_read_well() {
    for (error, text) in [
        (
            GameCommandError::BattleOpen,
            "an open battle waits for its commands",
        ),
        (GameCommandError::NoBattle, "no battle is open"),
        (
            GameCommandError::NotYourSide,
            "the side to act is not the player's",
        ),
        (
            GameCommandError::Battle(h3_battle::CommandError::BattleOver),
            "the battle is over",
        ),
        (
            GameCommandError::NotForSystem,
            "the system actor issues only an unowned side's commands",
        ),
    ] {
        assert_eq!(error.to_string(), text);
    }
}
