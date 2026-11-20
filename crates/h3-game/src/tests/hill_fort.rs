//! Tests of the Hill Fort's decision (GAM-DEC-013; E08 T134b1, increment 14's design note §15):
//! a Hill Fort's visit opens it for its visitor's owner alone, it blocks the turn until its
//! `Leave` (the system's default), a row's upgrade and all of them keep it open, each paid
//! (`ResourcePaid`, nothing for a free one) then upgraded (`StackChanged`), refused by the
//! adventure (no upgrade) and then by the treasury; the view with the original's states
//! (ADV-HFT-002). A rejected answer changes nothing ([`play`] checks it).

use h3_adventure::{HillFort, Holds, MarketParamsDef, ObjectKindDef};
use h3_content::ResourceId;

use super::decisions::{BLUE, GOLD, RED, play};
use super::rewards::{AT, HERO, answered, reward_at, visit};
use super::*;

const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const ELF: CreatureTypeId = CreatureTypeId::from_raw(3);

/// [`rules`] with the elf: the imp of level 1 (50 gold) and the gog of level 3 (100 gold) both
/// upgrading to the elf (160 gold), and an entered, kind-marked `t:fort` with the base pack's
/// percents (its gold the market parameters' cost resource).
fn fort_rules() -> GameRuleset {
    let mut pack = core_pack(true);
    for (at, level) in [(IMP, 1), (GOG, 3), (ELF, 3)] {
        pack.creatures[at.index()].info.level = level;
    }
    pack.creatures[ELF.index()].info.cost = vec![(key("t:gold"), 160)];
    for at in [IMP, GOG] {
        pack.creatures[at.index()].info.upgrades = vec![key("t:elf")];
    }
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 2);
    adventure.object_types.push(ObjectTypeDef {
        kind_mark: true,
        ..entered(
            "t:fort",
            ObjectKindDef::HillFort(HillFort {
                gold: [0, 25, 50, 75, 100, 100, 100],
            }),
        )
    });
    adventure.params.as_mut().unwrap().market = Some(MarketParamsDef {
        cost: key("t:gold"),
        values: vec![(key("t:gold"), 1)],
        artifact_values: vec![(key("t:gold"), 1)],
        resources: vec![10],
        artifacts: vec![20],
        creatures: vec![30],
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part(&core, 7);
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        no_towns(&core),
    )
    .unwrap()
}

/// A game with the Hill Fort at [`AT`], red's hero with `gogs` gogs in slot 0 and nine imps in
/// slot 1 visiting it (2 000 gold): the game and the events of the visit.
fn at_fort(rules: &GameRuleset, gogs: u32) -> (GameState, Vec<GameEvent>) {
    let mut setup = setup();
    setup.map.objects = vec![reward_at(rules, "t:fort", Holds::Nothing)];
    let stack = |slot, creature, count| ArmyStack {
        slot,
        creature,
        count,
    };
    setup.heroes[0].army = HeroArmy::Custom(vec![stack(0, GOG, gogs), stack(1, IMP, 9)]);
    let mut state = GameState::new(rules, &setup, 7).unwrap();
    let events = visit(&mut state, rules);
    (state, events)
}

/// A Hill Fort's answer as red's command.
const fn fort(answer: HillFortAnswer) -> GameCommand {
    answered(Answer::HillFort(answer))
}

/// The Hill Fort's view red sees.
fn view(state: &GameState, rules: &GameRuleset) -> HillFortView {
    match observe(state, rules, Viewer::Player(RED)).decision {
        Some(DecisionView::HillFort(view)) => view,
        other => panic!("{other:?}"),
    }
}

/// The rows' and the whole's states of red's view.
fn states(state: &GameState, rules: &GameRuleset) -> (Vec<HillFortState>, HillFortState) {
    let view = view(state, rules);
    let rows = view.rows.iter().map(|row| row.state).collect();
    (rows, view.all_state)
}

/// Red's gold.
fn gold(state: &GameState) -> u32 {
    state.treasury(RED).unwrap()[GOLD.index()]
}

/// The stack of red's hero told changed at `slot`.
fn stack_changed(state: &GameState, slot: u8) -> GameEvent {
    let at = ArmySlot {
        army: ArmyRef::Hero(HERO),
        slot,
    };
    let stack = state
        .adventure()
        .hero(HERO)
        .unwrap()
        .army
        .iter()
        .find(|s| s.slot == slot)
        .copied();
    GameEvent::StackChanged {
        player: RED,
        at,
        stack,
    }
}

// covers: GAM-DEC-013, ADV-HFT-001, ADV-HFT-002, ADV-BON-009
#[test]
fn a_hill_forts_visit_opens_its_decision_for_its_owner_until_it_is_left() {
    let rules = fort_rules();
    let ty = rules.adventure().object_type_by_key("t:fort").unwrap();
    let (mut state, events) = at_fort(&rules, 3);
    let opened = GameEvent::HillFortOpened {
        player: RED,
        hero: HERO,
        object_type: ty,
        anchor: AT,
    };
    // The team's kind mark, then the decision.
    let known = GameEvent::Adventure(h3_adventure::AdventureEvent {
        kind: h3_adventure::AdventureEventKind::KindKnown { object_type: ty },
        seen_by: h3_adventure::Viewers::NONE.with(RED),
    });
    assert_eq!(events, [known, opened]);
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let open = state.hill_fort(RED).copied().unwrap();
    assert_eq!((open.hero, open.object_type, open.anchor), (HERO, ty, AT));
    assert_eq!(state.hill_fort(BLUE), None);
    // The view: the three gogs of level 3 for 3 × 60 gold at 50 %, the imps of level 1 free;
    // both within the 2 000 gold, the whole's 90 too.
    let row = |slot, creature, count, price: Vec<(ResourceId, u32)>| HillFortRowView {
        row: HillFortRow {
            slot,
            creature,
            count,
            upgrade: Some(ELF),
            price,
        },
        state: HillFortState::Affordable,
    };
    let seen = HillFortView {
        hero: HERO,
        object_type: ty,
        anchor: AT,
        rows: vec![row(0, GOG, 3, vec![(GOLD, 90)]), row(1, IMP, 9, vec![])],
        all: vec![(GOLD, 90)],
        all_state: HillFortState::Affordable,
    };
    assert_eq!(view(&state, &rules), seen);
    // Its `Leave` is its listed answer and the system's default; the turn waits for it.
    let leave = Answer::HillFort(HillFortAnswer::Leave);
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [answered(leave)]
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(leave));
    let other_kinds = [
        Answer::Offer(Consent::Decline),
        Answer::Altar(AltarAnswer::Leave),
        Answer::Market(MarketAnswer::Leave),
        Answer::Close,
    ];
    for answer in other_kinds {
        let wrong = play(&mut state, &rules, RED, answered(answer));
        assert_eq!(wrong, Err(GameCommandError::WrongAnswer), "{answer:?}");
    }
    let ended = play(&mut state, &rules, RED, GameCommand::EndTurn);
    assert_eq!(ended, Err(GameCommandError::DecisionOpen));
    // Hashed with the decision (game state hash v37).
    let mut left = state.clone();
    assert_eq!(
        play(&mut left, &rules, RED, answered(leave)),
        Ok(vec![GameEvent::DecisionClosed { player: RED }])
    );
    assert_ne!(state.state_hash(), left.state_hash());
    assert_eq!(left.hill_fort(RED), None);
    assert_eq!(default_answer(&left, &rules, RED), None);
}

// covers: GAM-DEC-013, ADV-HFT-002
#[test]
fn a_rows_upgrade_pays_its_price_and_upgrades_the_stack_keeping_the_fort_open() {
    let rules = fort_rules();
    let (mut state, _) = at_fort(&rules, 3);
    let paid = play(&mut state, &rules, RED, fort(HillFortAnswer::Upgrade(0))).unwrap();
    let gold_paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 90,
    };
    assert_eq!(paid, [gold_paid, stack_changed(&state, 0)]);
    assert_eq!(gold(&state), 2000 - 90);
    let army = &state.adventure().hero(HERO).unwrap().army;
    assert_eq!((army[0].creature, army[0].count), (ELF, 3));
    // Open still, the row now without an upgrade; the free row pays nothing.
    assert!(state.hill_fort(RED).is_some());
    use HillFortState::{Affordable, NoUpgrade};
    assert_eq!(
        states(&state, &rules),
        (vec![NoUpgrade, Affordable], Affordable)
    );
    let free = play(&mut state, &rules, RED, fort(HillFortAnswer::Upgrade(1))).unwrap();
    assert_eq!(free, [stack_changed(&state, 1)]);
    assert_eq!(gold(&state), 2000 - 90);
    // Nothing upgrades now: each answer refused by the adventure.
    assert_eq!(
        states(&state, &rules),
        (vec![NoUpgrade, NoUpgrade], NoUpgrade)
    );
    assert_eq!(view(&state, &rules).all, vec![]);
    for (answer, error) in [
        (HillFortAnswer::Upgrade(1), HillFortError::NoUpgrade(1)),
        (HillFortAnswer::Upgrade(5), HillFortError::NoUpgrade(5)),
        (HillFortAnswer::All, HillFortError::Nothing),
    ] {
        assert_eq!(
            play(&mut state, &rules, RED, fort(answer)),
            Err(GameCommandError::HillFort(error))
        );
    }
}

// covers: GAM-DEC-013, ADV-HFT-002
#[test]
fn all_upgrades_every_row_in_slot_order_when_the_whole_is_affordable() {
    let rules = fort_rules();
    // 70 gogs: 4 200 gold at 50 %, 2 100: beyond the 2 000, the row and the whole too dear.
    let (mut state, _) = at_fort(&rules, 70);
    use HillFortState::{Affordable, TooDear};
    assert_eq!(states(&state, &rules), (vec![TooDear, Affordable], TooDear));
    assert_eq!(view(&state, &rules).all, vec![(GOLD, 2100)]);
    for answer in [HillFortAnswer::Upgrade(0), HillFortAnswer::All] {
        assert_eq!(
            play(&mut state, &rules, RED, fort(answer)),
            Err(GameCommandError::CannotPay)
        );
    }
    // Three gogs: All pays and upgrades the gogs, then the free imps.
    let (mut state, _) = at_fort(&rules, 3);
    let all = play(&mut state, &rules, RED, fort(HillFortAnswer::All)).unwrap();
    let gold_paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 90,
    };
    let expected = [
        gold_paid,
        stack_changed(&state, 0),
        stack_changed(&state, 1),
    ];
    assert_eq!(all, expected);
    assert_eq!(gold(&state), 1910);
    let army = &state.adventure().hero(HERO).unwrap().army;
    assert!(army.iter().all(|stack| stack.creature == ELF));
    assert!(state.hill_fort(RED).is_some());
    assert_eq!(
        GameCommandError::HillFort(HillFortError::Nothing).to_string(),
        "no stack of the army upgrades"
    );
}
