//! Tests of a siege's battle through the game (E09 T8b1; the town epic's design note §11 and
//! §15): its setup from the town (the facts, the town's terrain and the tile's special ground,
//! the obstacles, the defending hero's bonuses), the defending hero's spell points and the
//! attacking winner's experience after it, the garrison's survivors, and the capture after an
//! attacker's win, after the winner's levels, the attacker staying outside. Since E09 T8b2a the
//! siege started by a step onto a town its garrison defends (TWN-SIE-001), through `apply`; since
//! T8b2b by a step onto a visitor of another team, with the garrison merged into its army first
//! (TWN-SIE-002).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureCommand, AdventureEventKind, BattleStart, Combatant, Defender, Holds, ObjectSetup,
    Pos, Scenery, TileMask,
};
use h3_adventure::{ArmyRef, ArmySlot};
use h3_battle::{BattleState, Command};
use h3_content::{ContentPack, Formation, HeroParams, Ruleset, TerrainId};
use h3_contracts::{
    Army, ArmyStack, BattleHero, BattleOutcome, BattleResult, BattleSetup, BattleSide, FortLevel,
    HeroId, HeroOutcome, PlayerId, StackOrigin, StackRecord, TownId,
};
use h3_heroes::HeroChange;
use h3_kernel::Rng;
use h3_towns::{
    BuildingDef, BuildingRoles, BuildingsSetup, DefenderBonus, GarrisonCreature, GarrisonStack,
    SlotSet, TownFaction, TownSetup,
};

use super::{faction, rules_of, town_pack, towns_core_pack, towns_rules, towns_setup};
use crate::tests::key;
use crate::town::pool::HeroStatus;
use crate::town::siege::{SIEGE_EXPERIENCE, raise};
use crate::town::{mana_after, siege_experience};
use crate::{
    Actor, Answer, GameCommand, GameEvent, GameRuleset, GameSetup, GameState, HeroArmy, SkillOffer,
    Viewer, apply, legal_commands, legal_moves, project,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const T1: TownId = TownId::new(1);
/// Red's hero, [`towns_setup`]'s first.
const HERO: HeroId = HeroId::new(0);
/// Blue's hero, [`towns_setup`]'s second.
const BLUE_HERO: HeroId = HeroId::new(1);
/// [`towns_setup`]'s blue town 1's entrance, its object 1.
const BLUE_GATE: Pos = Pos::new(3, 0, 0);
/// The tile below it, red's hero's.
const OUTSIDE: Pos = Pos::new(3, 1, 0);
/// The map slot of [`siege_rules`]' defender's building.
const DEFENDER_SLOT: u8 = 22;
/// [`siege_rules`]' native terrain of `t:town`, `t:rock`.
const ROCK: TerrainId = TerrainId::from_raw(1);

/// [`super::towns_rules`] with `t:town`'s native terrain `t:rock` and a building `b12` on
/// [`DEFENDER_SLOT`] raising a siege's defending hero by 20 attack, 2 defense, 1 power and 150
/// spell points.
fn siege_rules() -> GameRuleset {
    siege_rules_with(|_| {})
}

/// [`siege_rules`] with its core pack changed by `edit` first.
fn siege_rules_with(edit: impl FnOnce(&mut ContentPack)) -> GameRuleset {
    let mut core = towns_core_pack(false);
    core.factions[0].native_terrain = Some(key("t:rock"));
    edit(&mut core);
    let mut towns = town_pack(false);
    towns.towns[0].buildings.push(BuildingDef {
        key: "b12".into(),
        name: "Building".into(),
        slot: Some(DEFENDER_SLOT),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        roles: BuildingRoles {
            defender: DefenderBonus {
                attack: 20,
                defense: 2,
                power: 1,
                mana: 150,
            },
            ..BuildingRoles::default()
        },
    });
    rules_of(
        Ruleset::compile(&[core], "t:faithful").unwrap(),
        &towns,
        false,
        4,
    )
}

/// [`towns_setup`] with red's hero below blue's town 1, which has `buildings` and a garrison of
/// four imps in slot 1; blue's hero on the town's entrance, its visitor, when `visitor`.
fn siege_setup(rules: &GameRuleset, visitor: bool, buildings: BuildingsSetup) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.heroes[0].pos = OUTSIDE;
    if visitor {
        setup.heroes[1].pos = BLUE_GATE;
    }
    setup.towns[1].buildings = buildings;
    setup.towns[1].garrison = Some(vec![GarrisonStack {
        slot: 1,
        creature: GarrisonCreature::Creature(rules.core().creature_by_key("t:imp").unwrap()),
        count: 4,
    }]);
    setup
}

/// The standard buildings, with a Fort or without.
const fn standard(fort: bool) -> BuildingsSetup {
    BuildingsSetup::Standard { fort }
}

/// The battle red's hero starts against blue's town 1, as the visit gives it (E09 T8b2): the
/// town's visitor, else its garrison on blue's side; fought at the entrance on the tile's terrain
/// with the coast's battlefield, which the siege replaces; `town` the battle's.
fn siege_start(game: &GameState, town: Option<TownId>) -> BattleStart {
    let adventure = game.adventure();
    let combatant = |id: HeroId| {
        let hero = adventure.hero(id).unwrap();
        Combatant {
            hero: id,
            owner: hero.owner,
            army: hero.army.clone(),
            formation: hero.formation,
            tactics: hero.tactics,
        }
    };
    let defender = adventure.town_visitor(T1).map_or_else(
        || Defender::Garrison {
            owner: BLUE,
            army: adventure.town_garrison(1).unwrap().to_vec(),
        },
        |visitor| Defender::Hero(combatant(visitor)),
    );
    BattleStart {
        attacker: combatant(HERO),
        defender,
        tile: BLUE_GATE,
        terrain: TerrainId::from_raw(0),
        battlefield: Some(key("t:shore")),
        background: 2,
        attacker_ground: None,
        town,
    }
}

/// `game` after red's hero started `start`, with the state before it.
fn started(rules: &GameRuleset, game: &GameState, start: BattleStart) -> (GameState, GameState) {
    let mut after = game.clone();
    after.start_battle(rules, RED, start, &mut Vec::new());
    (after, game.clone())
}

/// A hero's battle hero as GAM-BAT-001 gives it, with its maximum spell points and its sex.
fn battle_hero(game: &GameState, rules: &GameRuleset, id: HeroId) -> BattleHero {
    let details = game.hero(id).unwrap();
    let mut hero = h3_heroes::derive_battle_hero(details, rules.core()).unwrap();
    hero.magic.max_mana = h3_heroes::max_mana(details, rules.core());
    hero.sex = game.hero_sex(id);
    hero
}

/// The siege's setup as the game builds it from `before` and `start` (GAM-BAT-001, TWN-SIE-003,
/// TWN-SIE-005), with the defending hero as derived (unraised), on `terrain` and `battlefield`,
/// its obstacles drawn from red's stream after the battle's seed (`t:faithful`: no ADV-QUIRK-5);
/// with the battle's seed.
fn expected_setup(
    before: &GameState,
    rules: &GameRuleset,
    start: &BattleStart,
    (terrain, battlefield): (TerrainId, Option<&str>),
) -> (BattleSetup, u64) {
    let core = rules.core();
    let machines = |id: HeroId| h3_heroes::war_machines(before.hero(id).unwrap(), core);
    // A garrison without a hero in the town's formation (E09 T8b2a).
    let town = before.adventure().map().town_formation(T1).unwrap();
    let (army, hero, machine, formation, owner) = match &start.defender {
        Defender::Hero(side) => (
            side.army.clone(),
            Some(battle_hero(before, rules, side.hero)),
            machines(side.hero),
            side.formation,
            Some(side.owner),
        ),
        Defender::Garrison { owner, army } => {
            (army.clone(), None, Default::default(), town, Some(*owner))
        }
        Defender::Guards { army } => (army.clone(), None, Default::default(), town, None),
        other => panic!("{other:?} defends no siege here"),
    };
    let surrender = rules.battle().battle().surrender.unwrap().resource;
    let gold = |player: Option<PlayerId>| {
        player.map_or(0, |player| {
            before.treasury(player).unwrap()[usize::from(surrender.raw())]
        })
    };
    let human = |player: Option<PlayerId>| player.is_some_and(|p| before.human(p).unwrap());
    let record = before.towns().get(T1).unwrap();
    let mut setup = BattleSetup {
        attacker: Army {
            stacks: start.attacker.army.clone(),
        },
        defender: Army { stacks: army },
        heroes: [Some(battle_hero(before, rules, HERO)), hero],
        terrain: Some(terrain),
        battlefield: battlefield.map(key),
        war_machines: [machines(HERO), machine],
        formations: [start.attacker.formation, formation],
        funds: [gold(Some(RED)), gold(owner)],
        difficulty: before.adventure().difficulty(),
        human: [Some(RED), owner].map(human),
        background: Some(2),
        colours: [Some(0), owner.map(|_| 1)],
        siege: Some(h3_towns::siege(record, rules.towns())),
        ..BattleSetup::default()
    };
    let mut stream = before.stream(RED);
    let seed = stream.next_u64();
    let obstacles = stream.next_u64();
    setup.obstacles =
        h3_battle::generate_setup_obstacles(rules.battle(), &setup, &mut Rng::from_seed(obstacles));
    (setup, seed)
}

/// The battle of `setup` and `seed`.
fn battle(rules: &GameRuleset, (setup, seed): &(BattleSetup, u64)) -> BattleState {
    BattleState::new(rules.battle(), setup, *seed).unwrap()
}

/// A special ground `t:holy_ground` lying on blue's town's entrance.
fn holy_ground(rules: &GameRuleset) -> ObjectSetup {
    ObjectSetup {
        anchor: BLUE_GATE,
        blocked: TileMask::default(),
        visitable: TileMask::default(),
        object_type: rules.adventure().object_type_by_key("t:holy_ground"),
        amount: None,
        drawn: TileMask::from_bits(1),
        monster: None,
        guards: None,
        scenery: Scenery::Other,
        holds: Holds::Nothing,
        owner: None,
    }
}

// covers: TWN-SIE-003, TWN-SIE-005, GAM-BAT-001
#[test]
fn a_siege_is_fought_on_its_towns_facts_and_native_terrain() {
    let rules = siege_rules();
    for fort in [true, false] {
        let game = GameState::new(&rules, &siege_setup(&rules, false, standard(fort)), 7).unwrap();
        let start = siege_start(&game, Some(T1));
        let (after, before) = started(&rules, &game, start.clone());
        let expected = expected_setup(&before, &rules, &start, (ROCK, None));
        let battle_at = after.battle(RED).unwrap();
        assert_eq!(battle_at.state(), &battle(&rules, &expected));
        // The garrison on blue's side without a hero; the town's facts: no obstacle within the
        // walls, the usual ones without a fort.
        assert_eq!(battle_at.owners(), [Some(RED), Some(BLUE)]);
        assert_eq!(battle_at.heroes(), [Some(HERO), None]);
        assert_eq!(battle_at.tile(), BLUE_GATE);
        let siege = h3_battle::computer::siege(battle_at.state()).unwrap();
        let level = if fort {
            FortLevel::Fort
        } else {
            FortLevel::None
        };
        assert_eq!((siege.faction, siege.fort), (start_faction(&before), level));
        assert_eq!(expected.0.obstacles.is_empty(), fort);
    }
    // A special ground on the entrance: its battlefield, on the town's terrain.
    let mut setup = siege_setup(&rules, false, standard(true));
    setup.map.objects.push(holy_ground(&rules));
    let game = GameState::new(&rules, &setup, 7).unwrap();
    let start = siege_start(&game, Some(T1));
    let (after, before) = started(&rules, &game, start.clone());
    let expected = expected_setup(&before, &rules, &start, (ROCK, Some("t:holy")));
    assert_eq!(
        after.battle(RED).unwrap().state(),
        &battle(&rules, &expected)
    );
}

/// The faction of blue's town in `game`.
fn start_faction(game: &GameState) -> h3_content::FactionId {
    game.towns().get(T1).unwrap().faction()
}

// covers: TWN-SIE-005, GAM-BAT-001
#[test]
fn a_faction_without_a_native_terrain_keeps_the_tiles_and_a_battle_without_a_town_no_siege() {
    // `towns_rules`' `t:town` has no native terrain: the start's.
    let rules = towns_rules();
    let game = GameState::new(&rules, &siege_setup(&rules, false, standard(true)), 7).unwrap();
    let start = siege_start(&game, Some(T1));
    let (after, before) = started(&rules, &game, start.clone());
    let expected = expected_setup(&before, &rules, &start, (start.terrain, None));
    assert_eq!(
        after.battle(RED).unwrap().state(),
        &battle(&rules, &expected)
    );
    // Neither a battle without a town, nor one naming a town the store lacks, nor one away from
    // the town's entrance (town 0's, red's own) is a siege.
    let elsewhere = BattleStart {
        tile: Pos::new(1, 1, 0),
        ..siege_start(&game, Some(T1))
    };
    let starts = [None, Some(TownId::new(9))].map(|town| siege_start(&game, town));
    for start in starts.into_iter().chain([elsewhere]) {
        let (after, _) = started(&rules, &game, start);
        assert!(h3_battle::computer::siege(after.battle(RED).unwrap().state()).is_none());
    }
}

// covers: TWN-SIE-004
#[test]
fn a_sieges_defending_hero_is_raised_by_the_towns_buildings() {
    let rules = siege_rules();
    let built = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(DEFENDER_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    let game = GameState::new(&rules, &siege_setup(&rules, true, built), 7).unwrap();
    let start = siege_start(&game, Some(T1));
    assert!(matches!(start.defender, Defender::Hero(_)));
    let (after, before) = started(&rules, &game, start.clone());
    let mut expected = expected_setup(&before, &rules, &start, (ROCK, None));
    let hero = expected.0.heroes[1].as_mut().unwrap();
    let own = hero.clone();
    hero.attack += 20;
    hero.defense += 2;
    hero.power += 1;
    hero.magic.mana += 150;
    let battle_at = after.battle(RED).unwrap();
    assert_eq!(battle_at.state(), &battle(&rules, &expected));
    assert_eq!(battle_at.heroes(), [Some(HERO), Some(BLUE_HERO)]);
    let view = battle_at.state().hero(BattleSide::Defender).unwrap();
    assert_eq!(
        (view.attack, view.mana),
        (own.attack + 20, own.magic.mana + 150)
    );
    // The hero's own skills and spell points are untouched.
    assert_eq!(after.hero(BLUE_HERO), before.hero(BLUE_HERO));
}

// covers: TWN-SIE-004
#[test]
fn a_raise_holds_the_skills_within_the_highest_and_the_spell_points_saturate() {
    let params = HeroParams {
        primary_max: 99,
        ..siege_rules().core().hero_params().unwrap().clone()
    };
    let mut hero = BattleHero::new(h3_content::HeroTypeId::from_raw(0));
    (hero.attack, hero.defense, hero.power) = (90, 99, 3);
    hero.magic.mana = u16::MAX - 10;
    let bonus = DefenderBonus {
        attack: 20,
        defense: 2,
        power: 12,
        mana: 150,
    };
    raise(&mut hero, bonus, Some(&params));
    assert_eq!(
        (hero.attack, hero.defense, hero.power, hero.magic.mana),
        (99, 99, 15, u16::MAX)
    );
    // Without hero parameters, up to a byte's most.
    (hero.attack, hero.magic.mana) = (250, 7);
    raise(&mut hero, bonus, None);
    assert_eq!((hero.attack, hero.magic.mana), (u8::MAX, 157));
}

/// A battle's end with this result and these army survivors (by side), each side's hero left
/// with `mana` spell points, the defender's army stacks having lost `killed` imps.
fn ended(result: BattleResult, survivors: [&[ArmyStack]; 2], mana: [u16; 2]) -> BattleOutcome {
    BattleOutcome {
        result,
        survivors: survivors.map(|stacks| Army {
            stacks: stacks.to_vec(),
        }),
        war_machines: [vec![], vec![]],
        stacks: [vec![], vec![]],
        surrender_cost: None,
        heroes: mana.map(|mana| {
            Some(HeroOutcome {
                mana,
                learned: vec![],
            })
        }),
        rounds: 1,
    }
}

/// Red's surviving stack.
fn red_stack(game: &GameState) -> ArmyStack {
    game.adventure().hero(HERO).unwrap().army[0]
}

// covers: TWN-SIE-004
#[test]
fn a_sieges_defending_hero_keeps_at_most_the_spell_points_it_had() {
    let rules = siege_rules();
    let game = GameState::new(&rules, &siege_setup(&rules, true, standard(true)), 7).unwrap();
    let before = game.hero(BLUE_HERO).unwrap().mana;
    let red = game.hero(HERO).unwrap().mana;
    let survivor = game.adventure().hero(BLUE_HERO).unwrap().army[0];
    let sides = (
        [Some(RED), Some(BLUE)],
        [Some(HERO), Some(BLUE_HERO)],
        BLUE_GATE,
    );
    let red_stack = red_stack(&game);
    let end = |town: Option<TownId>, winner: BattleSide, left: u16| {
        let mut after = game.clone();
        let survivors: [&[ArmyStack]; 2] = match winner {
            BattleSide::Attacker => [&[red_stack], &[]],
            BattleSide::Defender => [&[], &[survivor]],
        };
        let outcome = ended(BattleResult::Victory(winner), survivors, [red + 9, left]);
        after.end_battle(&rules, sides, town, &outcome, &mut Vec::new());
        (
            after.hero(HERO).unwrap().mana,
            after.hero(BLUE_HERO).unwrap().mana,
        )
    };
    // The town's 150 left over: back to those it had; fewer left: those; whoever won (the beaten
    // visitor's details keep them). The attacker's are its own: its 9 more.
    for winner in BattleSide::BOTH {
        assert_eq!(end(Some(T1), winner, before + 150), (red + 9, before));
        assert_eq!(end(Some(T1), winner, before - 1), (red + 9, before - 1));
    }
    // Outside a siege the spell points left stand.
    let defender = BattleSide::Defender;
    assert_eq!(end(None, defender, before + 150).1, before + 150);
    assert_eq!((mana_after(3, 5), mana_after(9, 5)), (3, 5));
}

/// The garrison's survivors of blue's town 1 in `game`.
fn garrison(game: &GameState) -> &[ArmyStack] {
    game.adventure().town_garrison(1).unwrap()
}

/// Whether `events` tell of town 1's capture by red.
fn captured(events: &[GameEvent]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            GameEvent::Adventure(e) if e.kind == AdventureEventKind::TownCaptured {
                town: T1,
                anchor: BLUE_GATE,
                owner: RED,
            }
        )
    })
}

// covers: TWN-CAP-003
#[test]
fn an_attackers_win_over_the_garrison_captures_the_town_with_the_attacker_outside() {
    let rules = siege_rules();
    let game = GameState::new(&rules, &siege_setup(&rules, false, standard(true)), 7).unwrap();
    let imps = garrison(&game).to_vec();
    let sides = ([Some(RED), Some(BLUE)], [Some(HERO), None], BLUE_GATE);
    let stack = red_stack(&game);
    let end = |result, survivors: [&[ArmyStack]; 2]| {
        let mut after = game.clone();
        let mut events = Vec::new();
        let outcome = ended(result, survivors, [0, 0]);
        after.end_battle(&rules, sides, Some(T1), &outcome, &mut events);
        (after, events)
    };
    // Won: the garrison beaten, the capture held until the winner's levels are given.
    let (mut won, mut events) = end(BattleResult::Victory(BattleSide::Attacker), [&[stack], &[]]);
    assert!(garrison(&won).is_empty() && !captured(&events));
    assert_eq!(
        (won.adventure().owner(1), won.held_rests()),
        (Some(BLUE), 1)
    );
    won.run_rests(&rules, &mut events);
    assert!(captured(&events), "{events:?}");
    let adventure = won.adventure();
    assert_eq!((adventure.owner(1), won.held_rests()), (Some(RED), 0));
    assert_eq!(adventure.hero(HERO).unwrap().pos, OUTSIDE);
    assert_eq!(adventure.town_visitor(T1), None);
    // Lost: the survivors stay as the garrison, red's hero leaves the map, nothing is held.
    let left = [ArmyStack {
        count: 3,
        ..imps[0]
    }];
    let (lost, events) = end(BattleResult::Victory(BattleSide::Defender), [&[], &left]);
    assert_eq!(garrison(&lost), &left[..]);
    assert_eq!(
        (lost.adventure().owner(1), lost.held_rests()),
        (Some(BLUE), 0)
    );
    assert!(lost.adventure().hero(HERO).is_none() && !captured(&events));
    // Both sides destroyed: the town stays blue's, undefended.
    let (draw, _) = end(BattleResult::Draw, [&[], &[]]);
    assert!(garrison(&draw).is_empty());
    assert_eq!(
        (draw.adventure().owner(1), draw.held_rests()),
        (Some(BLUE), 0)
    );
}

// covers: TWN-CAP-003, TWN-SET-005
#[test]
fn an_attackers_win_over_the_visitor_captures_the_town_after_the_winners_levels() {
    let rules = siege_rules();
    let game = GameState::new(&rules, &siege_setup(&rules, true, standard(true)), 7).unwrap();
    let visitor = game.adventure().hero(BLUE_HERO).unwrap().army[0];
    let blue_type = game.hero(BLUE_HERO).unwrap().hero_type;
    let sides = (
        [Some(RED), Some(BLUE)],
        [Some(HERO), Some(BLUE_HERO)],
        BLUE_GATE,
    );
    // Red's win over the visitor, whose army lost enough for a level.
    let outcome = BattleOutcome {
        stacks: [
            vec![],
            vec![StackRecord {
                origin: StackOrigin::Army(visitor.slot),
                creature: visitor.creature,
                start: 1000,
                end: 0,
            }],
        ],
        ..ended(
            BattleResult::Victory(BattleSide::Attacker),
            [&[red_stack(&game)], &[]],
            [0, 0],
        )
    };
    let mut won = game.clone();
    let mut events = Vec::new();
    won.end_battle(&rules, sides, Some(T1), &outcome, &mut events);
    won.run_rests(&rules, &mut events);
    // The visitor off the map, its type free again; the garrison's imps, which did not fight,
    // still in the town; the capture waits for red's level.
    assert!(won.adventure().hero(BLUE_HERO).is_none());
    assert_eq!(won.hero_pool().status(blue_type), Some(HeroStatus::Free));
    assert_eq!(garrison(&won), garrison(&game));
    assert!(won.level_up(RED).is_some() && !captured(&events));
    assert_eq!(won.adventure().owner(1), Some(BLUE));
    let answer = GameCommand::Answer(Answer::LevelUp(SkillOffer::First));
    let mut events = Vec::new();
    apply(&mut won, &rules, Actor::Player(RED), answer, &mut events).unwrap();
    while won.level_up(RED).is_some() {
        let answer = GameCommand::Answer(Answer::LevelUp(SkillOffer::First));
        apply(&mut won, &rules, Actor::Player(RED), answer, &mut events).unwrap();
    }
    assert!(captured(&events), "{events:?}");
    // Captured with the garrison lost, red's hero outside.
    assert_eq!(won.adventure().owner(1), Some(RED));
    assert!(garrison(&won).is_empty());
    assert_eq!(won.adventure().hero(HERO).unwrap().pos, OUTSIDE);
}

// covers: TWN-SIE-006
#[test]
fn a_sieges_attacking_winner_gains_500_more_experience() {
    let rules = siege_rules();
    let game = GameState::new(&rules, &siege_setup(&rules, false, standard(false)), 7).unwrap();
    let sides = ([Some(RED), Some(BLUE)], [Some(HERO), None], BLUE_GATE);
    let experience = |town: Option<TownId>, result: BattleResult, survivors: [&[ArmyStack]; 2]| {
        let mut after = game.clone();
        let mut events = Vec::new();
        after.end_battle(
            &rules,
            sides,
            town,
            &ended(result, survivors, [0, 0]),
            &mut events,
        );
        events.iter().find_map(|event| match event {
            GameEvent::HeroChanged {
                change: HeroChange::GainExperience(gained),
                ..
            } => Some(*gained),
            _ => None,
        })
    };
    let stack = red_stack(&game);
    let won = BattleResult::Victory(BattleSide::Attacker);
    // Nothing killed: the siege's 500 alone, none in a field battle.
    assert_eq!(
        experience(Some(T1), won, [&[stack], &[]]),
        Some(SIEGE_EXPERIENCE)
    );
    assert_eq!(experience(None, won, [&[stack], &[]]), None);
    // A visitor that wins its siege gains none of it: the defeated hero's 500 alone.
    let visited = GameState::new(&rules, &siege_setup(&rules, true, standard(false)), 7).unwrap();
    let survivor = visited.adventure().hero(BLUE_HERO).unwrap().army[0];
    let sides = (
        [Some(RED), Some(BLUE)],
        [Some(HERO), Some(BLUE_HERO)],
        BLUE_GATE,
    );
    let mut after = visited.clone();
    let mut events = Vec::new();
    let lost = ended(
        BattleResult::Victory(BattleSide::Defender),
        [&[], &[survivor]],
        [0, 0],
    );
    after.end_battle(&rules, sides, Some(T1), &lost, &mut events);
    let gained: Vec<u32> = events
        .iter()
        .filter_map(|event| match event {
            GameEvent::HeroChanged {
                hero: BLUE_HERO,
                change: HeroChange::GainExperience(gained),
                ..
            } => Some(*gained),
            _ => None,
        })
        .collect();
    assert_eq!(gained, [500]);
    assert_eq!(
        (siege_experience(7, true), siege_experience(7, false)),
        (507, 7)
    );
    assert_eq!(siege_experience(u32::MAX, true), u32::MAX);
}

/// Plays `game`'s battle to its end, each owner to act giving `pick` of its legal battle
/// commands (ADR-0017 point 6: the garrison's owner outside its turn; the system actor an
/// unowned side's, E08 T20b2); the events.
fn play(
    game: &mut GameState,
    rules: &GameRuleset,
    pick: impl Fn(Actor, &[GameCommand]) -> GameCommand,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for _ in 0..2000 {
        let Some(battle) = game.battle(RED) else {
            return events;
        };
        let side = battle.state().side_to_act().unwrap();
        let actor = battle.owners()[side.index()].map_or(Actor::System, Actor::Player);
        let listed = legal_commands(game, rules, actor);
        let command = pick(actor, &listed);
        apply(game, rules, actor, command, &mut events).unwrap();
    }
    panic!("the siege did not end");
}

/// The last legal command that fights on (an attack where there is one).
fn fight(_: Actor, listed: &[GameCommand]) -> GameCommand {
    let fights = |c: &&GameCommand| !matches!(c, GameCommand::Battle(c) if c.leaves_battle());
    listed.iter().rfind(fights).unwrap().clone()
}

// covers: TWN-CAP-003, TWN-SIE-006, GAM-BAT-001
#[test]
fn a_siege_played_to_its_end_is_settled_as_a_siege() {
    let rules = siege_rules();
    // Behind its walls: red's hero retreats at once, blue's garrison wins untouched.
    let game = GameState::new(&rules, &siege_setup(&rules, false, standard(true)), 7).unwrap();
    let imps = garrison(&game).to_vec();
    let (mut after, _) = started(&rules, &game, siege_start(&game, Some(T1)));
    let events = play(&mut after, &rules, |actor, listed| {
        if actor == Actor::Player(RED) {
            GameCommand::Battle(Command::Retreat)
        } else {
            fight(actor, listed)
        }
    });
    assert!(after.adventure().hero(HERO).is_none() && !captured(&events));
    assert_eq!(garrison(&after), &imps[..]);
    assert_eq!(after.adventure().owner(1), Some(BLUE));
    // Without a fort red's nine imps beat the four: the capture, the siege's experience.
    let game = GameState::new(&rules, &siege_setup(&rules, false, standard(false)), 7).unwrap();
    let (mut after, _) = started(&rules, &game, siege_start(&game, Some(T1)));
    let events = play(&mut after, &rules, fight);
    assert!(captured(&events), "{events:?}");
    assert_eq!(after.adventure().owner(1), Some(RED));
    assert_eq!(after.adventure().hero(HERO).unwrap().pos, OUTSIDE);
    let imp = rules
        .battle()
        .creature(imps[0].creature)
        .unwrap()
        .hit_points;
    let gained = events.iter().find_map(|event| match event {
        GameEvent::HeroChanged {
            change: HeroChange::GainExperience(gained),
            ..
        } => Some(*gained),
        _ => None,
    });
    assert_eq!(gained, Some(4 * imp + SIEGE_EXPERIENCE));
}

/// [`siege_setup`] without a visitor, town 1 in a tight formation, neutral (of `t:town`, its
/// garrison and buildings kept) when `neutral`, with a Fort when `fort`.
fn garrisoned(rules: &GameRuleset, neutral: bool, fort: bool) -> GameSetup {
    let mut setup = siege_setup(rules, false, standard(fort));
    setup.map.objects[1].holds = Holds::Town {
        formation: Formation::Tight,
    };
    if neutral {
        setup.map.objects[1].owner = None;
        setup.towns[1] = TownSetup {
            faction: TownFaction::Fixed(faction(rules, "t:town")),
            ..setup.towns[1].clone()
        };
    }
    setup
}

/// Red's hero stepped from [`OUTSIDE`] onto blue's town 1's entrance through `apply`: the events.
fn step_onto_gate(rules: &GameRuleset, game: &mut GameState) -> Vec<GameEvent> {
    let command = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![BLUE_GATE],
    });
    let mut events = Vec::new();
    apply(game, rules, Actor::Player(RED), command, &mut events).unwrap();
    events
}

/// Whether `events` hold a step of a hero.
fn stepped(events: &[GameEvent]) -> bool {
    events.iter().any(|event| {
        matches!(event, GameEvent::Adventure(event)
            if matches!(event.kind, AdventureEventKind::HeroStepped { .. }))
    })
}

// covers: TWN-SIE-001, TWN-VIS-001, GAM-BAT-001
#[test]
fn a_step_onto_a_town_its_garrison_defends_starts_its_siege_from_the_tile_before() {
    let rules = siege_rules();
    for neutral in [false, true] {
        let game = GameState::new(&rules, &garrisoned(&rules, neutral, true), 7).unwrap();
        let moves = legal_moves(&game, &rules, Actor::Player(RED), HERO).unwrap();
        assert_eq!(moves.path(BLUE_GATE), Some(vec![BLUE_GATE]));
        let left = moves.points_left(BLUE_GATE).unwrap();
        let mut after = game.clone();
        let events = step_onto_gate(&rules, &mut after);
        // The battle the visit gives (the step draws nothing): the garrison without a hero in
        // the town's formation, on its owner's side or the neutral seat's.
        let mut start = siege_start(&game, Some(T1));
        if neutral {
            start.defender = Defender::Guards {
                army: garrison(&game).to_vec(),
            };
        }
        let expected = expected_setup(&game, &rules, &start, (ROCK, None));
        assert_eq!(expected.0.formations[1], Formation::Tight);
        let battle_at = after.battle(RED).unwrap();
        assert_eq!(battle_at.state(), &battle(&rules, &expected));
        let owner = (!neutral).then_some(BLUE);
        assert_eq!(battle_at.owners(), [Some(RED), owner]);
        assert_eq!(battle_at.heroes(), [Some(HERO), None]);
        assert_eq!(battle_at.tile(), BLUE_GATE);
        // Visited from the tile before: no step, its points paid; the town as it was.
        assert!(!stepped(&events), "{events:?}");
        assert!(
            events
                .iter()
                .any(|event| matches!(event, GameEvent::BattleStarted { .. }))
        );
        let hero = after.adventure().hero(HERO).unwrap();
        assert_eq!((hero.pos, hero.movement), (OUTSIDE, left));
        assert_eq!(garrison(&after), garrison(&game));
        assert_eq!(after.adventure().owner(1), owner);
    }
}

// covers: TWN-SIE-001, TWN-CAP-003, TWN-VIS-001
#[test]
fn a_siege_a_step_started_is_played_and_captures_the_town_with_the_attacker_outside() {
    let rules = siege_rules();
    // Without a fort red's nine imps beat the four, blue's or the neutral seat's.
    for neutral in [false, true] {
        let mut game = GameState::new(&rules, &garrisoned(&rules, neutral, false), 7).unwrap();
        let mut events = step_onto_gate(&rules, &mut game);
        events.extend(play(&mut game, &rules, fight));
        assert!(captured(&events), "{events:?}");
        let adventure = game.adventure();
        assert_eq!(adventure.owner(1), Some(RED));
        assert_eq!(adventure.hero(HERO).unwrap().pos, OUTSIDE);
        assert!(garrison(&game).is_empty());
        // The next step enters the town, red's now.
        let events = step_onto_gate(&rules, &mut game);
        assert!(stepped(&events) && !captured(&events), "{events:?}");
        assert_eq!(game.adventure().town_visitor(T1), Some(HERO));
    }
    // Behind a neutral town's walls red's hero retreats; the system actor's garrison keeps it.
    let mut game = GameState::new(&rules, &garrisoned(&rules, true, true), 7).unwrap();
    let imps = garrison(&game).to_vec();
    let mut events = step_onto_gate(&rules, &mut game);
    events.extend(play(&mut game, &rules, |actor, listed| {
        if actor == Actor::Player(RED) {
            GameCommand::Battle(Command::Retreat)
        } else {
            fight(actor, listed)
        }
    }));
    assert!(game.battle(RED).is_none() && !captured(&events));
    assert!(game.adventure().hero(HERO).is_none());
    assert_eq!(garrison(&game), &imps[..]);
    assert_eq!(game.adventure().owner(1), None);
}

/// [`siege_rules`] whose imp is worth 50, gog 100, and with the test core's extra elf, worth 1
/// (E09 T8b2b: the garrison's merge reads the creatures' AI values, TWN-SIE-002; the battle part
/// knows the elf).
fn merge_rules() -> GameRuleset {
    siege_rules_with(|core| {
        core.creatures[0].info.ai_value = 50;
        core.creatures[1].info.ai_value = 100;
        let mut elf = core.creatures[0].clone();
        elf.key = key("t:elf");
        elf.info.ai_value = 1;
        core.creatures.push(elf);
    })
}

/// A stack.
const fn stack(slot: u8, creature: h3_content::CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// [`siege_setup`] with blue's hero the visitor of its town 1, without a fort, its army (the part's
/// two slots) three gogs and a gog, and the garrison's four imps in slot 1 joined by ten elves in
/// slot 0; red's hero with `red` for its army.
fn visited(rules: &GameRuleset, red: Vec<ArmyStack>) -> GameSetup {
    let core = rules.core();
    let (gog, elf) = (
        core.creature_by_key("t:gog").unwrap(),
        core.creature_by_key("t:elf").unwrap(),
    );
    let mut setup = siege_setup(rules, true, standard(false));
    setup.heroes[0].army = HeroArmy::Custom(red);
    setup.heroes[1].army = HeroArmy::Custom(vec![stack(0, gog, 3), stack(1, gog, 1)]);
    setup.towns[1]
        .garrison
        .as_mut()
        .unwrap()
        .push(GarrisonStack {
            slot: 0,
            creature: GarrisonCreature::Creature(elf),
            count: 10,
        });
    setup
}

/// [`visited`]'s armies after the merge (survey §9.2): the gog of slot 1, the weakest, changes
/// places with the four imps (no room for them; 200 − 100), then joins slot 0's gogs (room: its
/// whole value); the elves, without room and worth less than the weakest (the imps' 200), stay.
/// The visitor's army, then the garrison.
fn merged(rules: &GameRuleset) -> (Vec<ArmyStack>, Vec<ArmyStack>) {
    let core = rules.core();
    let [imp, gog, elf] = ["t:imp", "t:gog", "t:elf"].map(|k| core.creature_by_key(k).unwrap());
    (
        vec![stack(0, gog, 4), stack(1, imp, 4)],
        vec![stack(0, elf, 10)],
    )
}

/// The merge's events among `events`, as their owners see them: each `StackChanged`.
fn stacks_changed(events: &[GameEvent]) -> Vec<&GameEvent> {
    events
        .iter()
        .filter(|event| matches!(event, GameEvent::StackChanged { .. }))
        .collect()
}

// covers: TWN-SIE-002, TWN-SIE-001, TWN-VIS-002, GAM-BAT-001
#[test]
fn a_step_onto_a_towns_visitor_merges_the_garrison_into_its_army_and_besieges_it() {
    let rules = merge_rules();
    let imps = vec![stack(1, rules.core().creature_by_key("t:imp").unwrap(), 9)];
    let game = GameState::new(&rules, &visited(&rules, imps), 7).unwrap();
    // The legal moves reach the visitor's tile from the tile before, as a monster's.
    let moves = legal_moves(&game, &rules, Actor::Player(RED), HERO).unwrap();
    assert_eq!(moves.path(BLUE_GATE), Some(vec![BLUE_GATE]));
    let left = moves.points_left(BLUE_GATE).unwrap();
    let mut after = game.clone();
    let events = step_onto_gate(&rules, &mut after);
    // The merge's slots, the visitor's then the garrison's, told to blue alone, before the battle.
    let (army, rest) = merged(&rules);
    let at = |army, slot| ArmySlot { army, slot };
    let changed = |army, slot, stack| GameEvent::StackChanged {
        player: BLUE,
        at: at(army, slot),
        stack,
    };
    let hero = ArmyRef::Hero(BLUE_HERO);
    assert_eq!(
        stacks_changed(&events),
        [
            &changed(hero, 0, Some(army[0])),
            &changed(hero, 1, Some(army[1])),
            &changed(ArmyRef::Garrison(T1), 1, None),
        ]
    );
    let first = |f: fn(&GameEvent) -> bool| events.iter().position(f).unwrap();
    assert!(
        first(|e| matches!(e, GameEvent::StackChanged { .. }))
            < first(|e| matches!(e, GameEvent::BattleStarted { .. }))
    );
    for event in stacks_changed(&events) {
        assert_eq!(project(event, Viewer::Player(RED)), None);
        assert_eq!(project(event, Viewer::Player(BLUE)).as_ref(), Some(event));
    }
    // The siege against the visitor with its merged army, in the town; red's hero outside with
    // the legal moves' points, no step.
    let mut start = siege_start(&game, Some(T1));
    let Defender::Hero(visitor) = &mut start.defender else {
        panic!("{:?}", start.defender);
    };
    visitor.army = army.clone();
    let expected = expected_setup(&game, &rules, &start, (ROCK, None));
    let battle_at = after.battle(RED).unwrap();
    assert_eq!(battle_at.state(), &battle(&rules, &expected));
    assert_eq!(battle_at.owners(), [Some(RED), Some(BLUE)]);
    assert_eq!(battle_at.heroes(), [Some(HERO), Some(BLUE_HERO)]);
    assert!(!stepped(&events), "{events:?}");
    let adventure = after.adventure();
    let red = adventure.hero(HERO).unwrap();
    assert_eq!((red.pos, red.movement), (OUTSIDE, left));
    assert_eq!(adventure.hero(BLUE_HERO).unwrap().army, army);
    assert_eq!(garrison(&after), &rest[..]);
    assert_eq!(adventure.owner(1), Some(BLUE));
}

// covers: TWN-SIE-002, TWN-CAP-003
#[test]
fn a_siege_against_a_visitor_keeps_its_merged_army_or_captures_the_town_with_the_attacker_outside()
{
    let rules = merge_rules();
    let (army, rest) = merged(&rules);
    let imp = rules.core().creature_by_key("t:imp").unwrap();
    // Red retreats at once: blue's visitor wins and keeps the merged army, the garrison what the
    // merge left; red's hero leaves the map.
    let mut game = GameState::new(&rules, &visited(&rules, vec![stack(1, imp, 9)]), 7).unwrap();
    let mut events = step_onto_gate(&rules, &mut game);
    events.extend(play(&mut game, &rules, |actor, listed| {
        if actor == Actor::Player(RED) {
            GameCommand::Battle(Command::Retreat)
        } else {
            fight(actor, listed)
        }
    }));
    assert!(game.battle(RED).is_none() && !captured(&events));
    let adventure = game.adventure();
    assert!(adventure.hero(HERO).is_none());
    assert_eq!(adventure.hero(BLUE_HERO).unwrap().army, army);
    assert_eq!(garrison(&game), &rest[..]);
    assert_eq!(adventure.owner(1), Some(BLUE));
    // Red's horde of imps beats the merged army: the visitor off the map, the capture after
    // red's levels with the merge's leftovers lost, red's hero outside.
    let mut game = GameState::new(&rules, &visited(&rules, vec![stack(1, imp, 4000)]), 7).unwrap();
    let mut events = step_onto_gate(&rules, &mut game);
    events.extend(play(&mut game, &rules, fight));
    while game.level_up(RED).is_some() {
        let answer = GameCommand::Answer(Answer::LevelUp(SkillOffer::First));
        apply(&mut game, &rules, Actor::Player(RED), answer, &mut events).unwrap();
    }
    assert!(captured(&events), "{events:?}");
    let adventure = game.adventure();
    assert!(adventure.hero(BLUE_HERO).is_none());
    assert_eq!(adventure.owner(1), Some(RED));
    assert_eq!(adventure.hero(HERO).unwrap().pos, OUTSIDE);
    assert!(garrison(&game).is_empty());
}

// covers: TWN-SIE-002
#[test]
fn a_visitors_siege_with_nothing_to_merge_tells_no_stack() {
    // Blue's visitor of two stacks of three gogs (300 each): the imps (200), without room, gain
    // nothing; the garrison stays as it is and nothing is told before the battle.
    let rules = merge_rules();
    let gog = rules.core().creature_by_key("t:gog").unwrap();
    let mut setup = visited(&rules, vec![stack(1, gog, 9)]);
    setup.heroes[1].army = HeroArmy::Custom(vec![stack(0, gog, 3), stack(1, gog, 3)]);
    setup.towns[1].garrison.as_mut().unwrap().truncate(1);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let before = garrison(&game).to_vec();
    let events = step_onto_gate(&rules, &mut game);
    assert!(stacks_changed(&events).is_empty(), "{events:?}");
    assert!(game.battle(RED).is_some());
    assert_eq!(garrison(&game), &before[..]);
}
