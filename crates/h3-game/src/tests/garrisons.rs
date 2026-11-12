//! Tests of the map's garrisons through the game (E08 T114a1; increment 12's design note §6, §7,
//! §14): another's or a neutral garrison is fought at once, unsplit, its owner's side or the
//! system's; a battle on an anti-magic garrison's entrance, the garrison's own or a hero's
//! standing in it, is fought without magic; a win flags the garrison after the winner's levels,
//! a loss leaves its survivors.

use h3_adventure::{AdventureEventKind, Holds, ObjectSetup, TileMask};

use super::decisions::{after_steps, move_hero, play};
use super::manage::imp;
use super::mines::{adventure, fight};
use super::rewards::win;
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);

/// A garrison of type `k` at `anchor`, entered there, `owner`'s or neutral, holding `army`.
fn garrison(
    rules: &GameRuleset,
    k: &str,
    anchor: Pos,
    owner: Option<PlayerId>,
    army: Vec<ArmyStack>,
) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Garrison {
            army,
            removable: true,
        },
        owner,
    }
}

/// [`setup`] with `objects`, red's hero at (0, 0) with `imps` imps in slot 1, blue's at (2, 2)
/// with its starting army.
fn game(rules: &GameRuleset, imps: u32, objects: Vec<ObjectSetup>) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![imp(1, imps)]);
    setup.map.objects = objects;
    GameState::new(rules, &setup, 7).unwrap()
}

/// Whether the battle `player` has open is fought without magic, as `player` sees it.
fn anti_magic(state: &GameState, rules: &GameRuleset, player: PlayerId) -> bool {
    observe(state, rules, Viewer::Player(player))
        .battle
        .unwrap()
        .anti_magic
}

// covers: ADV-GAR-002, ADV-GAR-005, GAM-BAT-001, GAM-DEC-007
#[test]
fn a_neutral_anti_magic_garrison_is_fought_without_magic_and_flagged_after_the_win() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let ty = rules
        .adventure()
        .object_type_by_key("t:anti_garrison")
        .unwrap();
    let held = vec![imp(1, 1)];
    let object = garrison(&rules, "t:anti_garrison", anchor, None, held.clone());
    let mut state = game(&rules, 300, vec![object]);
    // No question: the battle at once, the system playing the garrison's unsplit army.
    let events = play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    assert!(matches!(
        after_steps(events)[..],
        [GameEvent::BattleStarted { .. }, ..]
    ));
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    assert!(anti_magic(&state, &rules, RED));
    let events = win(&mut state, &rules);
    // Won: after the levels, the flag with its reveal; the army gone.
    let told = adventure(&events);
    assert!(told.iter().any(|e| e.kind
        == AdventureEventKind::ObjectFlagged {
            object_type: ty,
            anchor,
            owner: RED,
        }));
    let adventure = state.adventure();
    assert_eq!(adventure.owner(0), Some(RED));
    assert_eq!(adventure.garrison_army(0), []);
    assert_eq!(adventure.hero(RED_HERO).unwrap().pos, anchor);
    assert!(!state.has_decision(RED));
}

// covers: ADV-GAR-002, GAM-BAT-001
#[test]
fn an_owned_garrison_defends_as_its_owner_s_side_and_keeps_its_survivors() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let held = vec![imp(0, 40)];
    let object = garrison(&rules, "t:garrison", anchor, Some(BLUE), held);
    let mut state = game(&rules, 1, vec![object]);
    play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    // Blue's side without a hero, played by blue outside its turn; with magic.
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), Some(BLUE)]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    assert!(!anti_magic(&state, &rules, BLUE));
    fight(&mut state, &rules);
    // Lost: red's hero off the map, the garrison blue's with its survivors.
    let adventure = state.adventure();
    assert_eq!(adventure.hero(RED_HERO), None);
    assert_eq!(adventure.owner(0), Some(BLUE));
    let left = adventure.garrison_army(0);
    assert!(matches!(left, [ArmyStack { slot: 0, count, .. }] if (1..=40).contains(count)));
}

// covers: ADV-GAR-005
#[test]
fn a_hero_standing_in_an_anti_magic_garrison_is_attacked_without_magic() {
    let rules = rules();
    let entrance = Pos::new(2, 1, 0);
    let object = garrison(&rules, "t:anti_garrison", entrance, Some(BLUE), vec![]);
    let mut state = game(&rules, 9, vec![object]);
    // Blue's hero steps into its own garrison (crossed, visited where its move ends: nothing yet,
    // the exchange coming with T114a2); then red attacks it there.
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    play(&mut state, &rules, BLUE, move_hero(1, &[entrance])).unwrap();
    assert_eq!(
        state.adventure().hero(HeroId::new(1)).unwrap().pos,
        entrance
    );
    play(&mut state, &rules, BLUE, GameCommand::EndTurn).unwrap();
    let path = [Pos::new(1, 0, 0), entrance];
    play(&mut state, &rules, RED, move_hero(0, &path)).unwrap();
    assert_eq!(state.battle(RED).unwrap().heroes()[1], Some(HeroId::new(1)));
    assert!(anti_magic(&state, &rules, RED));
}
