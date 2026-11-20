//! Tests of the War Machine Factory through the game (GAM-DEC-008 amended; E08 T134b2, increment
//! 14's design note §16): its visit asks the hire question, whose yes opens the hire dialog of its
//! machines to the visitor's owner alone with one of each the hero lacks (ADV-WMF-001); the most
//! is 1 or 0 with the gold bound; a purchase pays and gives the machine's artifact into its
//! position (`HeroChanged`, no army change), the dialog of several machines staying open; each
//! refusal leaves the state as it was ([`play`] checks it); every visit asks again.

use h3_adventure::{
    HireError, HireTarget, Holds, ObjectKindDef, Offer, OfferKind, WarMachineFactoryDef,
};
use h3_content::ArtifactTypeId;
use h3_heroes::{ArtifactInstance, HeroChange, positions};

use super::decisions::{ACCEPT, BLUE, GOLD, RED, play, revisit};
use super::rewards::{AT, HERO, reward_at, visit};
use super::*;

/// [`rules`] with the cart (the core's war machine, its artifact in the machine position) at
/// 1 000 gold, a second war machine, the tent, at 750 gold with its artifact in a position of its
/// own, and an entered `t:factory` selling the tent, then the cart.
fn factory_rules() -> GameRuleset {
    let mut pack = core_pack(true);
    let cart = pack
        .creatures
        .iter_mut()
        .find(|c| c.key == key("t:cart"))
        .unwrap();
    cart.info.cost = vec![(key("t:gold"), 1000)];
    let mut tent = war_machine("t:tent");
    tent.info.cost = vec![(key("t:gold"), 750)];
    pack.creatures.push(tent);
    pack.artifact_slots.push(ArtifactSlotDef {
        key: key("t:machine_2"),
        name: "Slot".into(),
        count: 1,
        backpack: false,
    });
    pack.artifacts.push(ArtifactDef {
        war_machine: Some(key("t:tent")),
        ..artifact("t:tent_art", "t:machine_2")
    });
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 2);
    adventure.object_types.push(ObjectTypeDef {
        from_above: false,
        kind_mark: false,
        ..entered(
            "t:factory",
            ObjectKindDef::WarMachineFactory(WarMachineFactoryDef {
                machines: vec![key("t:tent"), key("t:cart")],
            }),
        )
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    // The tent's battle row: the cart's, without its supply.
    let battle = battle_part_with(&core, 7, |pack| {
        let mut row = pack.creatures[2].clone();
        row.key = key("t:tent");
        row.abilities = vec![];
        pack.creatures.push(row);
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        no_towns(&core),
    )
    .unwrap()
}

fn creature_of(rules: &GameRuleset, k: &str) -> CreatureTypeId {
    rules.core().creature_by_key(k).unwrap()
}

fn artifact_of(rules: &GameRuleset, k: &str) -> ArtifactTypeId {
    rules.core().artifact_by_key(k).unwrap()
}

/// A game with the factory at [`AT`] beside red's hero, which wears the cart's artifact
/// (2 000 gold).
fn at_factory(rules: &GameRuleset) -> GameState {
    let mut setup = setup();
    setup.map.objects = vec![reward_at(rules, "t:factory", Holds::Nothing)];
    let mut worn: Vec<Option<ArtifactInstance>> = vec![None; positions(rules.core()).len()];
    worn[1] = Some(artifact_of(rules, "t:cart_art").into());
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn,
        backpack: vec![],
    });
    GameState::new(rules, &setup, 7).unwrap()
}

const fn buy(creature: CreatureTypeId, amount: u32) -> GameCommand {
    GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount }))
}

const HIRE_CLOSE: GameCommand = GameCommand::Answer(Answer::Hire(HireAnswer::Close));

/// The hire dialog's creatures with their counts red sees.
fn counts(state: &GameState, rules: &GameRuleset) -> Vec<(CreatureTypeId, u32)> {
    match observe(state, rules, Viewer::Player(RED)).decision {
        Some(DecisionView::Hire(view)) => view
            .creatures
            .iter()
            .map(|c| (c.creature, c.count))
            .collect(),
        other => panic!("{other:?}"),
    }
}

/// Red's legal commands.
fn legal(state: &GameState, rules: &GameRuleset) -> Vec<GameCommand> {
    legal_commands(state, rules, Actor::Player(RED))
}

// covers: GAM-DEC-008, ADV-WMF-001, ADV-HIR-002
#[test]
fn a_factorys_dialog_sells_one_of_each_machine_the_hero_lacks_as_its_artifact() {
    let rules = factory_rules();
    let (tent, cart) = (creature_of(&rules, "t:tent"), creature_of(&rules, "t:cart"));
    let factory = rules.adventure().object_type_by_key("t:factory").unwrap();
    let mut state = at_factory(&rules);
    // The visit asks the hire question naming the machines; its yes opens the dialog, told to red
    // alone, the factory by type and anchor.
    let asked = visit(&mut state, &rules);
    let offered = GameEvent::OfferOpened {
        player: RED,
        hero: HERO,
        object_type: factory,
        anchor: AT,
    };
    assert_eq!(asked.last(), Some(&offered));
    let question = DecisionView::Offer(Offer {
        hero: HERO,
        object_type: factory,
        anchor: AT,
        kind: OfferKind::Hire {
            creatures: vec![tent, cart],
        },
    });
    let decision = |state: &GameState| observe(state, &rules, Viewer::Player(RED)).decision;
    assert_eq!(decision(&state), Some(question.clone()));
    let source = HireSeen::Object {
        object_type: factory,
        anchor: AT,
    };
    let opened = GameEvent::HireOpened {
        player: RED,
        target: HireTarget::Hero(HERO),
        source,
    };
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert_eq!(events.last(), Some(&opened));
    assert_eq!(observe(&state, &rules, Viewer::Player(BLUE)).decision, None);
    // The cart worn: none of it; one tent, at its cost. The most is 1 or 0: only the tent is
    // listed, and only while the gold pays it.
    let view = HireView {
        target: HireTarget::Hero(HERO),
        source,
        creatures: vec![
            HireCreature {
                creature: tent,
                count: 1,
                cost: vec![(GOLD, 750)],
            },
            HireCreature {
                creature: cart,
                count: 0,
                cost: vec![(GOLD, 1000)],
            },
        ],
    };
    assert_eq!(decision(&state), Some(DecisionView::Hire(view)));
    assert_eq!(legal(&state, &rules), [buy(tent, 1), HIRE_CLOSE]);
    assert_eq!(hire_range(&state, &rules, RED, tent), 1..=1);
    assert!(hire_range(&state, &rules, RED, cart).is_empty());
    let mut poor = state.clone();
    poor.set_resource(RED, GOLD, 749);
    assert_eq!(legal(&poor, &rules), [HIRE_CLOSE]);
    assert_eq!(
        play(&mut poor, &rules, RED, buy(tent, 1)),
        Err(GameCommandError::CannotPay)
    );
    // The adventure's refusals first: a machine held, an amount but 1.
    for (creature, amount) in [(cart, 1), (tent, 2)] {
        assert_eq!(
            play(&mut state, &rules, RED, buy(creature, amount)),
            Err(GameCommandError::Hire(HireError::Amount(amount)))
        );
    }
    // Bought: the gold paid, the tent's artifact given into its position, no army change; the
    // dialog of two stays open with none left to buy.
    let army = state.adventure().hero(HERO).unwrap().army.clone();
    let tent_art = artifact_of(&rules, "t:tent_art");
    let events = play(&mut state, &rules, RED, buy(tent, 1)).unwrap();
    let given = GameEvent::HeroChanged {
        player: RED,
        hero: HERO,
        change: HeroChange::AddArtifact(tent_art.into()),
    };
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 750,
    };
    assert_eq!(events, [paid, given]);
    assert_eq!(state.treasury(RED).unwrap()[GOLD.index()], 2000 - 750);
    assert_eq!(state.heroes()[0].worn[4], Some(tent_art.into()));
    assert_eq!(state.adventure().hero(HERO).unwrap().army, army);
    assert!(state.has_decision(RED));
    assert_eq!(counts(&state, &rules), [(tent, 0), (cart, 0)]);
    assert_eq!(legal(&state, &rules), [HIRE_CLOSE]);
    // `Close` closes it; every visit asks again.
    let events = play(&mut state, &rules, RED, HIRE_CLOSE).unwrap();
    assert_eq!(events, [GameEvent::DecisionClosed { player: RED }]);
    let again = play(&mut state, &rules, RED, revisit(0)).unwrap();
    assert!(again.contains(&offered), "{again:?}");
    assert_eq!(decision(&state), Some(question));
}
