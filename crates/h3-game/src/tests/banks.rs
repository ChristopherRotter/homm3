//! Tests of what the game applies of the creature banks (E08 T124b2; increment 13's design note
//! §14, §15): the question by the kind, the yes starting a bank battle (`BattleSetup::
//! creature_bank` from `Defender::Bank`: each side's stacks at the bank positions by their
//! ordinal), the reward after the winner's levels with its gains and the
//! creature joined, the army dialog for a creature without room, a lost battle's survivors.

use h3_adventure::{BankDef, BankLevelDef, BankQuestion, Holds, OfferKind};
use h3_battle::Field;
use h3_contracts::{Army, BattleOutcome, BattleResult, BattleSide, StackOrigin, StackRecord};

use super::decisions::{ACCEPT, CLOSE, DECLINE, GOLD, RED, play};
use super::rewards::{AT, HERO, answered, reward_at, starts_battle, visit, won};
use super::*;

const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

/// [`rules`] with armies of seven slots and two entered banks after the other types: the attack
/// kind `t:utopia` (10 imps, five stacks of 2 after the split; 500 gold and 2 gogs) and the search
/// kind `t:crypt` (the same level; looted −1 morale under mark 15).
fn bank_rules() -> GameRuleset {
    let core = Arc::new(core(false));
    let mut pack = adventure_pack(vec![88, 25], 7);
    let level = BankLevelDef {
        chance: 100,
        guards: vec![(key("t:imp"), 10)],
        upgrade: 0,
        resources: vec![(key("t:gold"), 500)],
        creature: Some((key("t:gog"), 2)),
        artifacts: [0; 4],
    };
    let bank = |k: &str, question, looted| ObjectTypeDef {
        visit: Visit::Enter,
        from_above: false,
        ..reward(
            k,
            ObjectKindDef::Bank(BankDef {
                question,
                levels: vec![level.clone()],
                looted,
                deck: false,
                coast: false,
            }),
        )
    };
    let once = OnceBonus {
        morale: -1,
        luck: 0,
        mark: 15,
    };
    pack.object_types.extend([
        bank("t:utopia", BankQuestion::Attack, None),
        bank("t:crypt", BankQuestion::Search, Some(once)),
    ]);
    let part = AdventureRuleset::compile(&core, &[pack]).unwrap();
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle_part(&core, 7)),
        no_towns(&core),
    )
    .unwrap()
}

/// The game with a bank of type `k` at [`AT`] and red's hero's army `army`, at `experience`.
fn bank_game(rules: &GameRuleset, k: &str, army: Vec<ArmyStack>, experience: u32) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(army);
    setup.heroes[0].details.experience = Some(experience);
    setup.map.objects = vec![reward_at(rules, k, Holds::Nothing)];
    GameState::new(rules, &setup, 7).unwrap()
}

/// `count` of `creature` in `slot`.
const fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// Red's victory at [`AT`] over the bank's guards, its imps all killed: enough experience for a
/// level from 999.
fn won_a_level() -> BattleOutcome {
    BattleOutcome {
        stacks: [
            vec![],
            vec![StackRecord {
                origin: StackOrigin::Army(0),
                creature: IMP,
                start: 10,
                end: 0,
            }],
        ],
        ..won()
    }
}

/// The index of the first of `events` that `matches`.
fn first(events: &[GameEvent], matches: impl Fn(&GameEvent) -> bool) -> Option<usize> {
    events.iter().position(matches)
}

// covers: ADV-BNK-003, ADV-BNK-004, GAM-BAT-001
#[test]
fn a_banks_yes_starts_a_bank_battle_at_the_bank_positions() {
    let rules = bank_rules();
    for (k, kind) in [
        ("t:utopia", OfferKind::Fight),
        ("t:crypt", OfferKind::Search),
    ] {
        let army = vec![stack(1, IMP, 9), stack(4, GOG, 1)];
        let mut state = bank_game(&rules, k, army, 0);
        visit(&mut state, &rules);
        assert_eq!(state.decision(RED).unwrap().kind, kind, "{k}");
        // Declined: nothing starts.
        let mut declined = state.clone();
        let events = play(&mut declined, &rules, RED, DECLINE).unwrap();
        assert!(!starts_battle(&events) && declined.battle(RED).is_none());
        // Accepted: the battle against the guards as drawn, each side's stacks at the bank
        // positions by their ordinal among its filled slots (the formation unused).
        let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
        assert!(starts_battle(&events), "{k}");
        let open = state.battle(RED).unwrap();
        assert_eq!(open.owners(), [Some(RED), None]);
        assert_eq!(open.tile(), AT);
        let battle = open.state();
        let params = rules.battle().battle();
        let field = Field::new(params.field);
        for side in BattleSide::BOTH {
            let stacks: Vec<_> = battle.stacks().iter().filter(|s| s.side == side).collect();
            let expected = if side == BattleSide::Attacker { 2 } else { 5 };
            assert_eq!(stacks.len(), expected, "{k} {side:?}");
            for (ordinal, s) in stacks.into_iter().enumerate() {
                let (x, y) = params.bank_position(side.index(), ordinal).unwrap();
                let hex = field.hex(i32::from(x), i32::from(y)).unwrap();
                assert_eq!(s.hex, hex, "{k} {side:?} {ordinal}");
            }
        }
    }
}

// covers: ADV-BNK-005, GAM-DEC-007, GAM-BAT-008, GAM-RES-001
#[test]
fn a_won_banks_reward_comes_after_the_winners_levels() {
    let rules = bank_rules();
    let mut state = bank_game(&rules, "t:utopia", vec![stack(1, IMP, 9)], 999);
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    let mut events = Vec::new();
    state.end_battle(&rules, sides, None, &won_a_level(), &mut events);
    state.run_rests(&rules, &mut events);
    // The level-up first: the reward waits, the guards beaten, the bank not yet looted.
    let gained = |e: &GameEvent| {
        matches!(
            e,
            GameEvent::ResourceGained {
                player: RED,
                amount: 500,
                ..
            }
        )
    };
    assert!(state.level_up(RED).is_some());
    assert_eq!(first(&events, gained), None, "{events:?}");
    assert!(!state.adventure().is_emptied(0));
    assert!(state.adventure().guards_left(0).is_empty());
    // Answered: the level, then the gold and the gogs, the bank looted and still on the map.
    let first_skill = answered(Answer::LevelUp(SkillOffer::First));
    let events = play(&mut state, &rules, RED, first_skill).unwrap();
    let level = first(&events, |e| matches!(e, GameEvent::LevelGained { .. })).unwrap();
    let gold = first(&events, gained).unwrap();
    assert!(level < gold, "{events:?}");
    assert_eq!(state.treasury(RED).unwrap()[usize::from(GOLD.raw())], 2500);
    let army = &state.adventure().hero(HERO).unwrap().army;
    assert_eq!(army, &[stack(0, GOG, 2), stack(1, IMP, 9)]);
    assert!(state.adventure().is_emptied(0) && !state.adventure().is_removed(0));
}

// covers: ADV-BNK-005, GAM-DEC-004
#[test]
fn a_banks_creature_without_room_opens_the_army_dialog() {
    let rules = bank_rules();
    // Seven slots of imps, all surviving: the gogs find no room; the dialog opens with them, the
    // gold given.
    let army: Vec<ArmyStack> = (0..7).map(|slot| stack(slot, IMP, 1)).collect();
    let mut state = bank_game(&rules, "t:utopia", army.clone(), 0);
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    let all_survive = BattleOutcome {
        survivors: [Army { stacks: army }, Army { stacks: vec![] }],
        ..won()
    };
    let mut events = Vec::new();
    state.end_battle(&rules, sides, None, &all_survive, &mut events);
    state.run_rests(&rules, &mut events);
    assert_eq!(
        state
            .army_dialog(RED)
            .map(|(hero, group)| (hero, group.to_vec())),
        Some((HERO, vec![stack(0, GOG, 2)]))
    );
    assert_eq!(state.treasury(RED).unwrap()[usize::from(GOLD.raw())], 2500);
    // Closed: what is left lost.
    let events = play(&mut state, &rules, RED, CLOSE).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::GroupLost { .. }))
    );
    assert!(state.adventure().is_emptied(0));
}

// covers: ADV-BNK-004
#[test]
fn a_lost_banks_survivors_stay_as_its_guards() {
    let rules = bank_rules();
    let mut state = bank_game(&rules, "t:utopia", vec![stack(1, IMP, 9)], 0);
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    let lost = BattleOutcome {
        result: BattleResult::Victory(BattleSide::Defender),
        survivors: [
            Army { stacks: vec![] },
            Army {
                stacks: vec![stack(0, IMP, 2), stack(3, IMP, 1)],
            },
        ],
        heroes: [None, None],
        ..won()
    };
    state.end_battle(&rules, sides, None, &lost, &mut Vec::new());
    // The hero leaves the map; the survivors stay as the bank's guards, not looted.
    assert!(state.adventure().hero(HERO).is_none());
    assert_eq!(
        state.adventure().guards_left(0),
        [stack(0, IMP, 2), stack(3, IMP, 1)]
    );
    assert!(!state.adventure().is_emptied(0));
}
