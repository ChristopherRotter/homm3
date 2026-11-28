//! Tests of the planning (E31 T2i), hand-computed on the object value's fixture
//! (`objects::tests`: a 12 × 12 map of grass, red's hero 0 with 9 imps on (0, 0), blue's hero 1
//! on (2, 2); the imp's Fight Value 100), the town fixture (`towns::tests`: the 4 × 4 map, red's
//! T0 on (1, 1), blue's T1 on (3, 0)) and, for the jumps, a core of their four spells. A step on
//! grass costs 100, a diagonal 141; the test hero's day's points are read, not assumed.

extern crate std;

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    AdventurePack, AdventureRuleset, AdventureSpell, AdventureSpellDef, BoatSetup, ObjectKind,
    ObjectKindDef, ObjectTypeDef, OnceBonus, SpellParams, Visit,
};
use h3_content::{
    ArtifactDef, ContentPack, CreatureTypeId, HeroEffectDef, Ruleset, SecondarySkillDef,
    SpellCategory, SpellDef, TerrainId,
};
use h3_contracts::{HeroId, TownId};
use h3_heroes::{ArtifactInstance, HeroDetails};
use h3_kernel::StreamId;
use h3_kernel::arith::SignedX87;
use h3_towns::TownRuleset;

use super::field::Field;
use super::reach::{claimed, class_number, jumps, number_of};
use super::*;
use crate::computer::fight::{self, Opponent};
use crate::computer::memory::Plan;
use crate::computer::objects::Class;
use crate::computer::objects::tests::{self as fixture, Case, army, kind, object, tile};
use crate::tests::{adventure_pack, battle_part_with, core_pack, key, setup};
use crate::{GameCommand, GameSetup, HeroArmy};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

fn stream() -> Rng {
    Rng::for_stream(5, StreamId::AGENT)
}

/// A danger map of `state`'s size, every tile `value`.
fn flat(state: &GameState, value: i32) -> Danger {
    let map = state.adventure().map();
    let tiles = usize::from(map.levels()) * usize::from(map.width()) * usize::from(map.height());
    Danger {
        width: map.width(),
        height: map.height(),
        tiles: vec![value; tiles],
    }
}

/// `f` of the planner of the hero `on_map` (its details the game's) with the case's record and
/// weights, the danger map `danger` and the turn's flag `flag`.
fn planning<T>(
    case: &Case<'_>,
    on_map: &HeroOnMap,
    (danger, flag): (&Danger, bool),
    f: impl FnOnce(&Planner<'_>) -> T,
) -> T {
    let details = case.state.hero(on_map.id).unwrap();
    let fighter = Fighter { on_map, details };
    let valued = Valued::new(case.state, case.rules, (&case.record, &case.weights), fighter);
    let planner = Planner::new((case.state, case.rules), &valued, (&case.record, danger, flag));
    f(&planner)
}

/// [`planning`] of red's hero 0 as it stands.
fn red<T>(case: &Case<'_>, danger: &Danger, flag: bool, f: impl FnOnce(&Planner<'_>) -> T) -> T {
    let on_map = case.state.adventure().hero(HERO).unwrap();
    planning(case, on_map, (danger, flag), f)
}

/// The object value fixture's game with `objects` (each type `k` on its tile) and `edit`.
fn world(
    rules: &GameRuleset,
    objects: &[(&str, Pos)],
    edit: impl FnOnce(&mut GameSetup),
) -> GameState {
    let objects = objects.iter().map(|&(k, at)| object(rules, k, at)).collect();
    fixture::game(rules, objects, edit)
}

/// Blue's hero 1 with `stacks`.
fn blue_army(setup: &mut GameSetup, stacks: &[(CreatureTypeId, u32)]) {
    setup.heroes[1].army = HeroArmy::Custom(army(stacks));
}

// covers: CPU-PLN-002
#[test]
fn the_claims_take_the_classes_0_to_36_and_the_numbers_part_the_merged_classes() {
    assert!(claimed(0) && claimed(36));
    assert!(!claimed(37) && !claimed(u8::MAX));
    let rules = fixture::rules();
    let state = fixture::shared(&rules, |s| {
        s.map.boats.push(BoatSetup {
            tile: p(11, 11),
            kind: 0,
        });
        s.map.tiles[11 * 12 + 11].terrain = TerrainId::from_raw(2);
        // The Faerie Ring on water: the Mermaids' class.
        s.map.tiles[5 * 12 + 2].terrain = TerrainId::from_raw(2);
    });
    let number = |pos| class_number(&state, &rules, pos);
    assert_eq!(number(p(0, 0)), 34);
    assert_eq!(number(p(11, 11)), 8);
    assert_eq!(number(p(1, 0)), 0);
    assert_eq!(number(tile("t:artifact")), 5);
    assert_eq!(number(tile("t:marletto")), 23);
    assert_eq!(number(tile("t:youth")), 31);
    assert_eq!(number(tile("t:faerie")), 52);
    assert_eq!(number(tile("t:bank")), 16);
    assert_eq!(number(tile("t:monster")), 54);
    assert_eq!(number(tile("t:chart")), 13);
    // The kinds' parameters that part the merged classes.
    let of = |k: &str, class: Class, kind: &ObjectKind, water: bool| {
        let ty = rules.adventure().object_type_by_key(k).unwrap();
        let at = At::Object {
            index: 0,
            ty,
            kind,
            class,
        };
        number_of(&at, || water)
    };
    let ObjectKind::Bank(bank) = kind(&rules, "t:bank") else {
        panic!("a bank");
    };
    let banks = [
        ((true, true, None), 85),
        ((true, false, None), 24),
        ((false, false, Some(OnceBonus { morale: -1, luck: 0, mark: 14 })), 84),
        ((false, true, None), 16),
    ];
    for ((deck, coast, looted), class) in banks {
        let bank = ObjectKind::Bank(h3_adventure::Bank {
            deck,
            coast,
            looted,
            ..bank.clone()
        });
        assert_eq!(of("t:bank", Class::Bank, &bank, false), class);
    }
    let ObjectKind::Gift(mut gift) = kind(&rules, "t:marletto") else {
        panic!("a gift");
    };
    gift.gives = h3_adventure::Gives::Primary([1, 0, 0, 0]);
    let camp = ObjectKind::Gift(gift);
    assert_eq!(of("t:marletto", Class::OneLevel, &camp, false), 51);
    let level = ObjectKind::Gift(h3_adventure::Gift {
        gives: h3_adventure::Gives::Level,
        ..match kind(&rules, "t:marletto") {
            ObjectKind::Gift(gift) => gift,
            _ => unreachable!(),
        }
    });
    assert_eq!(of("t:marletto", Class::OneLevel, &level, false), 23);
    let oasis = match kind(&rules, "t:youth") {
        ObjectKind::Bonus {
            mark, days, drain, ..
        } => ObjectKind::Bonus {
            mark,
            days,
            points: 800,
            drain,
        },
        _ => unreachable!(),
    };
    assert_eq!(of("t:youth", Class::Oasis, &oasis, false), 56);
    let faerie = kind(&rules, "t:faerie");
    assert_eq!(of("t:faerie", Class::FaerieRing, &faerie, false), 28);
    let guard = ObjectKind::KeyGuard { key: 0 };
    assert_eq!(of("t:chart", Class::Other, &guard, false), 9);
    assert_eq!(of("t:chart", Class::Other, &ObjectKind::Tavern, false), u8::MAX);
}

// covers: CPU-PLN-001
#[test]
fn the_danger_map_marks_the_reach_of_the_enemies_the_hero_would_not_beat() {
    let rules = fixture::rules();
    // On Easy blue's (human) 5 imps cost red 19 and 17 imps and a gog beat it (−1 000 000 000);
    // a third hero of blue's after them.
    let state = world(&rules, &[], |s| {
        s.difficulty = 0;
        blue_army(s, &[(IMP, 17), (GOG, 1)]);
        let mut small = s.heroes[1].clone();
        small.pos = p(9, 9);
        small.army = HeroArmy::Custom(army(&[(IMP, 5)]));
        s.heroes.push(small);
    });
    let adventure = state.adventure();
    let on_map = adventure.hero(HERO).unwrap();
    let details = state.hero(HERO).unwrap();
    let hero = Fighter { on_map, details };
    let fight_of = |id| {
        let enemy = adventure.hero(id).unwrap();
        let opponent = Opponent {
            hero: Some(Fighter {
                on_map: enemy,
                details: state.hero(id).unwrap(),
            }),
            army: &enemy.army,
            town: None,
        };
        fight::fight(&state, &rules, hero, opponent, None)
    };
    assert_eq!(fight_of(BLUE_HERO), -1_000_000_000);
    assert_eq!(fight_of(HeroId::new(2)), -19);
    let danger = danger_map(&state, &rules, hero);
    // The lost fight's reach written over, then the small one's added after it; the tiles out
    // of both are 0, off the map too.
    assert_eq!(danger.at(p(2, 2)), -1_000_000_019);
    assert_eq!(danger.at(p(9, 9)), -1_000_000_019);
    assert_eq!(danger.at(p(12, 0)), 0);
    assert_eq!(danger.at(p(0, 12)), 0);
    assert!(danger.tiles.iter().all(|&d| d == -1_000_000_019 || d == 0));
    // Added in the other order, the small loss is written over.
    let reordered = world(&rules, &[], |s| {
        s.difficulty = 0;
        blue_army(s, &[(IMP, 5)]);
        let mut big = s.heroes[1].clone();
        big.pos = p(9, 9);
        big.army = HeroArmy::Custom(army(&[(IMP, 17), (GOG, 1)]));
        s.heroes.push(big);
    });
    let on_map = reordered.adventure().hero(HERO).unwrap();
    let details = reordered.hero(HERO).unwrap();
    let danger = danger_map(&reordered, &rules, Fighter { on_map, details });
    assert_eq!(danger.at(p(2, 2)), -1_000_000_000);
    // Without a human on red's team, Easy draws no map; an ally's heroes and a fight won count
    // nothing.
    for edit in [
        (|s: &mut GameSetup| s.players[0].human = false) as fn(&mut GameSetup),
        |s| s.players[1].team = 0,
        |s| {
            s.difficulty = 1;
            blue_army(s, &[(IMP, 5)]);
        },
    ] {
        let state = world(&rules, &[], |s| {
            s.difficulty = 0;
            blue_army(s, &[(IMP, 17), (GOG, 1)]);
            edit(s);
        });
        let on_map = state.adventure().hero(HERO).unwrap();
        let details = state.hero(HERO).unwrap();
        let danger = danger_map(&state, &rules, Fighter { on_map, details });
        let lost = fight::fight(
            &state,
            &rules,
            Fighter { on_map, details },
            Opponent {
                hero: Some(Fighter {
                    on_map: state.adventure().hero(BLUE_HERO).unwrap(),
                    details: state.hero(BLUE_HERO).unwrap(),
                }),
                army: &state.adventure().hero(BLUE_HERO).unwrap().army,
                town: None,
            },
            None,
        ) < 0;
        assert!(danger.tiles.iter().all(|&d| d == 0), "{lost}");
    }
}

/// A core with the jumps' spells (`t:summon`, `t:gate` as Fly, `t:walk` as Water Walk, `t:door`
/// as Dimension Door, all of air), Air Magic (`t:air`, 1 to 3), Angel Wings and the Boots of
/// Levitation (`t:wings`, `t:boots`, ring positions) and the Magic Plains (air at Expert).
fn jump_rules(boats: u16) -> GameRuleset {
    let mut pack: ContentPack = core_pack(false);
    for (k, level, cost) in [
        ("t:summon", 1, [8, 7, 7, 7]),
        ("t:walk", 2, [12, 9, 9, 9]),
        ("t:door", 5, [25, 25, 25, 20]),
    ] {
        pack.spells.push(SpellDef {
            key: key(k),
            name: "Jump".into(),
            category: SpellCategory::Adventure,
            level,
            schools: vec![key("t:air")],
            cost,
            power: 0,
            effect: [50, 50, 75, 100],
            granted_by_name_only: false,
        });
    }
    pack.skills.push(SecondarySkillDef {
        key: key("t:air_magic"),
        name: "Air".into(),
        levels: [1, 2, 3].map(|level| {
            vec![HeroEffectDef::SchoolMastery {
                school: key("t:air"),
                level,
            }]
        }),
    });
    let ring = pack.artifacts[4].clone();
    pack.artifacts.push(ArtifactDef {
        key: key("t:wings"),
        effects: vec![HeroEffectDef::Flying { level: 3 }],
        ..ring.clone()
    });
    pack.artifacts.push(ArtifactDef {
        key: key("t:boots"),
        effects: vec![HeroEffectDef::WaterWalking { level: 3 }],
        ..ring
    });
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure: AdventurePack = adventure_pack(vec![88, 25], 2);
    adventure.spells = [
        ("t:gate", AdventureSpell::Fly),
        ("t:walk", AdventureSpell::WaterWalk),
        ("t:door", AdventureSpell::DimensionDoor),
        ("t:summon", AdventureSpell::SummonBoat),
    ]
    .map(|(k, effect)| AdventureSpellDef {
        spell: key(k),
        effect,
    })
    .into();
    adventure.params.as_mut().unwrap().spells = Some(SpellParams {
        door_points: [300, 300, 300, 200],
        target_range: [9, 8],
        boats_most: core::num::NonZeroU16::new(boats).unwrap(),
    });
    adventure.object_types.push(ObjectTypeDef {
        key: key("t:plains_ground"),
        name: "Ground".into(),
        visit: Visit::Enter,
        from_above: true,
        kind_mark: false,
        kind: ObjectKindDef::Ground {
            battlefield: key("t:plains"),
            no_native: false,
        },
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        let plain = pack.battlefields[0].clone();
        pack.battlefields.push(h3_content_battle::BattlefieldDef {
            key: key("t:plains"),
            expert_schools: vec![key("t:air")],
            ..plain
        });
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(TownRuleset::empty(&core)),
    )
    .unwrap()
}

/// The jumps of red's hero on `state` with its details `edit`ed: the spells known, the mana,
/// the skills, the worn artifacts.
fn jumps_of(state: &GameState, rules: &GameRuleset, edit: impl FnOnce(&mut HeroDetails)) -> SearchJumps {
    let on_map = state.adventure().hero(HERO).unwrap();
    let mut details = state.hero(HERO).unwrap().clone();
    edit(&mut details);
    jumps(state, rules, Fighter { on_map, details: &details })
}

/// The spell `k` of `rules`.
fn spell(rules: &GameRuleset, k: &str) -> h3_content::SpellId {
    rules.core().spell_by_key(k).unwrap()
}

/// An artifact instance of `k`.
fn worn(rules: &GameRuleset, k: &str) -> Option<ArtifactInstance> {
    Some(ArtifactInstance {
        artifact: rules.core().artifact_by_key(k).unwrap(),
        spell: None,
    })
}

// covers: CPU-PLN-007
#[test]
fn the_jumps_are_the_castable_spells_and_the_levels_of_their_masteries() {
    let rules = jump_rules(64);
    let state = GameState::new(&rules, &setup(), 7).unwrap();
    let none = SearchJumps {
        door_price: 300,
        fly_level: Some(0),
        water_walk_level: Some(0),
        ..SearchJumps::default()
    };
    assert_eq!(jumps_of(&state, &rules, |_| {}), none);
    // Known with their costs plus 20 within the mana (the door's 25 + 20 = 45); one point short,
    // not castable; Summon Boat with no boat to bring and no Advanced, neither.
    let known = |mana: u16, skills: Vec<_>| {
        let spells: Vec<_> = ["t:summon", "t:gate", "t:walk", "t:door"]
            .map(|k| spell(&rules, k))
            .into();
        move |d: &mut HeroDetails| {
            d.spells = spells;
            d.mana = mana;
            d.skills = skills;
        }
    };
    let all = jumps_of(&state, &rules, known(45, vec![]));
    assert_eq!(
        all,
        SearchJumps {
            door: true,
            fly: true,
            water_walk: true,
            ..none
        }
    );
    let short = jumps_of(&state, &rules, known(44, vec![]));
    assert_eq!(
        short,
        SearchJumps {
            fly: true,
            water_walk: true,
            ..none
        }
    );
    // Expert Air: the door's leg 200, the levels 3; the day's own levels lower them (Fly at 1),
    // a higher own level not (Water Walk at 3 against Expert's 3).
    let air = rules.core().skill_by_key("t:air_magic").unwrap();
    let expert = jumps_of(&state, &rules, known(45, vec![(air, 3)]));
    assert_eq!(
        (expert.door_price, expert.fly_level, expert.water_walk_level),
        (200, Some(3), Some(3))
    );
    // Fly cast at Basic Air (its own level 1) lowers Expert's 3; Water Walk not cast keeps 3.
    let mut flown = {
        let mut s = setup();
        s.heroes[0].details.spells = Some(vec![spell(&rules, "t:gate")]);
        s.heroes[0].details.skills = Some(vec![(air, 1)]);
        GameState::new(&rules, &s, 7).unwrap()
    };
    let points = h3_heroes::HeroChange::SetSpellPoints(50);
    flown.change_hero(rules.core(), HERO, None, points, &mut crate::NoopSink);
    let cast = GameCommand::Cast {
        hero: HERO,
        spell: spell(&rules, "t:gate"),
        target: None,
    };
    let mut events = Vec::new();
    crate::apply(&mut flown, &rules, crate::Actor::Player(RED), cast, &mut events).unwrap();
    assert_eq!(flown.adventure().day_spells(HERO).fly, Some(1));
    let lowered = jumps_of(&flown, &rules, known(45, vec![(air, 3)]));
    assert_eq!((lowered.fly_level, lowered.water_walk_level), (Some(1), Some(3)));
    // Angel Wings and the Boots: flying and water walking at 3 without the spells.
    let wearing = jumps_of(&state, &rules, |d| {
        d.worn[2] = worn(&rules, "t:wings");
        d.worn[3] = worn(&rules, "t:boots");
    });
    assert_eq!(
        wearing,
        SearchJumps {
            fly: true,
            water_walk: true,
            fly_level: Some(3),
            water_walk_level: Some(3),
            ..none
        }
    );
    // On the Magic Plains the door's cost at Expert (20 + 20 = 40) and its price 200.
    let plains = GameState::new(
        &rules,
        &{
            let mut s = setup();
            s.map.objects.push(ground(&rules, p(0, 0)));
            s
        },
        7,
    )
    .unwrap();
    let raised = jumps_of(&plains, &rules, known(40, vec![]));
    assert_eq!((raised.door, raised.door_price), (true, 200));
}

/// The Magic Plains on `tile`.
fn ground(rules: &GameRuleset, tile: Pos) -> h3_adventure::ObjectSetup {
    h3_adventure::ObjectSetup {
        anchor: tile,
        blocked: h3_adventure::TileMask::default(),
        visitable: h3_adventure::TileMask::default(),
        object_type: rules.adventure().object_type_by_key("t:plains_ground"),
        amount: None,
        drawn: h3_adventure::TileMask::from_bits(1),
        monster: None,
        guards: None,
        scenery: h3_adventure::Scenery::Other,
        holds: h3_adventure::Holds::Nothing,
        owner: None,
    }
}

// covers: CPU-PLN-007
#[test]
fn summon_boat_is_a_jump_with_a_boat_to_bring_or_room_for_one_at_advanced_but_none_beside() {
    let rules = jump_rules(1);
    let summon = |mana: u16, skills: Vec<_>| {
        let spells = vec![spell(&rules, "t:summon")];
        move |d: &mut HeroDetails| {
            d.spells = spells;
            d.mana = mana;
            d.skills = skills;
        }
    };
    let air = rules.core().skill_by_key("t:air_magic").unwrap();
    let with_boat = |at: Pos| {
        let mut s = setup();
        s.map.tiles[usize::from(at.y) * 4 + usize::from(at.x)].terrain = TerrainId::from_raw(2);
        s.map.boats.push(BoatSetup { tile: at, kind: 0 });
        GameState::new(&rules, &s, 7).unwrap()
    };
    // A boat of nobody's to bring, its cost 8 within the mana (no margin here): summoned.
    let far = with_boat(p(3, 3));
    assert!(jumps_of(&far, &rules, summon(8, vec![])).summon_boat);
    assert!(!jumps_of(&far, &rules, summon(7, vec![])).summon_boat);
    // A boat beside the hero: no jump.
    let near = with_boat(p(1, 0));
    assert!(!jumps_of(&near, &rules, summon(8, vec![])).summon_boat);
    // No boat: Advanced makes one while the part has room (its most 1: none in use).
    let state = GameState::new(&rules, &setup(), 7).unwrap();
    assert!(!jumps_of(&state, &rules, summon(8, vec![(air, 1)])).summon_boat);
    assert!(jumps_of(&state, &rules, summon(8, vec![(air, 2)])).summon_boat);
    // Aboard the only boat the part allows: no boat to bring and no room for one.
    for (most, room) in [(1, false), (2, true)] {
        let rules = jump_rules(most);
        let mut s = setup();
        s.map.tiles[1].terrain = TerrainId::from_raw(2);
        s.map.boats.push(BoatSetup {
            tile: p(1, 0),
            kind: 0,
        });
        let mut state = GameState::new(&rules, &s, 7).unwrap();
        let board = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
            hero: HERO,
            path: vec![p(1, 0)],
        });
        let mut events = Vec::new();
        crate::apply(&mut state, &rules, crate::Actor::Player(RED), board, &mut events).unwrap();
        assert!(state.adventure().hero(HERO).unwrap().boat.is_some());
        let air = rules.core().skill_by_key("t:air_magic").unwrap();
        let spells = vec![spell(&rules, "t:summon")];
        let jumps = jumps_of(&state, &rules, |d| {
            d.spells = spells;
            d.mana = 8;
            d.skills = vec![(air, 2)];
        });
        assert_eq!(jumps.summon_boat, room);
    }
}

/// What a planner reads besides the game: the record, the weights, the danger map, the flag.
struct Reads {
    record: Record,
    weights: Vec<SignedX87>,
    danger: Danger,
    flag: bool,
}

impl Reads {
    /// The object value fixture's case reads (its record and weights), no danger, no flag.
    fn of(case: &Case<'_>) -> Self {
        Self {
            record: case.record.clone(),
            weights: case.weights.clone(),
            danger: flat(case.state, 0),
            flag: false,
        }
    }

    /// `f` of the planner of `on_map` (its details the game's) on `state`.
    fn plan<T>(
        &self,
        (state, rules): (&GameState, &GameRuleset),
        on_map: &HeroOnMap,
        f: impl FnOnce(&Planner<'_>) -> T,
    ) -> T {
        let details = state.hero(on_map.id).unwrap();
        let fighter = Fighter { on_map, details };
        let valued = Valued::new(state, rules, (&self.record, &self.weights), fighter);
        let reads = (&self.record, &self.danger, self.flag);
        f(&Planner::new((state, rules), &valued, reads))
    }

    /// [`Self::plan`] of red's hero 0 as it stands.
    fn red<T>(
        &self,
        (state, rules): (&GameState, &GameRuleset),
        f: impl FnOnce(&Planner<'_>) -> T,
    ) -> T {
        let on_map = state.adventure().hero(HERO).unwrap();
        self.plan((state, rules), on_map, f)
    }
}

/// The planner's candidates with the kept `plans` and the budget 1 000.
fn candidates_of(planner: &Planner<'_>, plans: &Plans) -> (i32, Vec<Candidate>) {
    let kept = plans.get(HERO).map(|plan| plan.destination);
    let asking = planner.asking(kept);
    let search = planner.search(1000, asking).unwrap();
    planner.candidates(&search, plans, asking)
}

/// The candidate on `pos` among `list`.
fn at_tile(list: &[Candidate], pos: Pos) -> Option<Candidate> {
    list.iter().copied().find(|c| c.pos == Some(pos))
}

// covers: CPU-PLN-003
#[test]
fn the_candidates_are_the_reached_entrances_of_value_but_the_hero_s_own_tile() {
    let rules = fixture::rules();
    let state = world(&rules, &[("t:campfire", p(1, 0)), ("t:chart", p(0, 1))], |_| {});
    let case = Case::new(&state, &rules);
    let reads = Reads::of(&case);
    let (danger, list) = reads.red((&state, &rules), |planner| {
        candidates_of(planner, &Plans::default())
    });
    // No town: no unexplored tile; the campfire's 55 at its node's second cost; the chart's 0
    // and the hero's own tile left out.
    assert_eq!(danger, 0);
    let campfire = Candidate {
        pos: Some(p(1, 0)),
        value: 55,
        cost: 100,
        near: false,
        must_go: false,
    };
    assert_eq!(at_tile(&list, p(1, 0)), Some(campfire));
    assert_eq!(at_tile(&list, p(0, 1)), None);
    assert_eq!(at_tile(&list, p(0, 0)), None);
    assert!(list.iter().all(|c| c.pos.is_some_and(|pos| {
        crate::computer::objects::at(&state, &rules, pos).is_some()
    })));
    // In danger on its own tile, the chart's 0 counts: a place to flee to.
    let fled = Reads {
        danger: flat(&state, -1),
        ..Reads::of(&case)
    };
    let (danger, list) = fled.red((&state, &rules), |planner| {
        candidates_of(planner, &Plans::default())
    });
    assert_eq!(danger, -1);
    assert_eq!(at_tile(&list, p(0, 1)).map(|c| c.value), Some(0));
}

// covers: CPU-PLN-002, CPU-PLN-003
#[test]
fn a_colleague_claims_a_claimed_class_it_reaches_more_cheaply_from_its_kept_destination() {
    let rules = fixture::rules();
    // Red's second hero 2 on (2, 0), the campfire (class 12) on (3, 0): red's 382 round it
    // (100 + 141 + 141) against its 100.
    let state = world(&rules, &[("t:campfire", p(3, 0))], |s| {
        let mut colleague = s.heroes[0].clone();
        colleague.pos = p(2, 0);
        s.heroes.push(colleague);
    });
    let case = Case::new(&state, &rules);
    let reads = Reads::of(&case);
    let colleague = HeroId::new(2);
    let points = state.adventure().hero(colleague).unwrap().movement;
    let points16 = u16::try_from(points).unwrap();
    let run = |plans: &Plans| {
        reads.red((&state, &rules), |planner| {
            let asking = planner.asking(None);
            let search = planner.search(1000, asking).unwrap();
            let claims = planner.claims(&search, plans);
            let list = planner.candidates(&search, plans, asking).1;
            (claims[planner.index(p(3, 0))], at_tile(&list, p(3, 0)))
        })
    };
    // From its tile, offset 0: claimed at 100, red's 300 left out.
    let (claim, campfire) = run(&Plans::default());
    assert_eq!((claim, campfire), (100, None));
    // A kept destination on (5, 0) with a cost beyond above its points: offset 0, claimed at 200.
    let mut plans = Plans::default();
    let far = |beyond| Plan {
        destination: p(5, 0),
        beyond,
        must_go: false,
    };
    plans.set(colleague, Some(far(points16 + 1)));
    let (claim, campfire) = run(&plans);
    assert_eq!((claim, campfire), (200, None));
    // A cost beyond not above its points: the offset beyond − points wraps (survey §5.5 #2), the
    // claim 200 + 0 − points in 16 bits, above red's 300: the campfire stays.
    plans.set(colleague, Some(far(0)));
    let (claim, campfire) = run(&plans);
    assert_eq!(claim, 200_u16.wrapping_sub(points16));
    assert!(claim > 382);
    assert_eq!(campfire.map(|c| (c.value, c.cost)), Some((55, 382)));
    // A colleague the search does not reach claims nothing.
    let boxed = world(&rules, &[("t:campfire", p(3, 0))], |s| {
        let mut colleague = s.heroes[0].clone();
        colleague.pos = p(2, 0);
        s.heroes.push(colleague);
        for at in [p(1, 0), p(1, 1), p(0, 1)] {
            s.map.tiles[usize::from(at.y) * 12 + usize::from(at.x)].terrain = TerrainId::from_raw(1);
        }
    });
    let case = Case::new(&boxed, &rules);
    let claims = Reads::of(&case).red((&boxed, &rules), |planner| {
        let search = planner.search(1000, planner.asking(None)).unwrap();
        planner.claims(&search, &Plans::default())
    });
    assert!(claims.iter().all(|&c| c == u16::MAX));
}

/// A record with red's hero's values and the towns' threats `towns`, for the town fixture.
fn town_reads(state: &GameState, towns: Vec<(TownId, u8)>) -> Reads {
    let resources = 2;
    Reads {
        record: Record {
            heroes: vec![(
                HERO,
                crate::computer::start::HeroValues {
                    experience: fixture::f32_bits(1, 2),
                    spells: crate::computer::spells::SpellGains::default(),
                },
            )],
            towns,
            artifacts: fixture::f32_bits(1000, 1),
            ..Record::default()
        },
        weights: (0..resources).map(|_| fixture::tenths(1)).collect(),
        danger: flat(state, 0),
        flag: false,
    }
}

/// The town fixture with red's hero 0 on (1, 1), T0's entrance, `edit` made to its setup.
fn on_town(rules: &GameRuleset, edit: impl FnOnce(&mut GameSetup)) -> GameState {
    crate::computer::towns::tests::game(rules, 0, |s| {
        s.heroes[0].pos = p(1, 1);
        edit(s);
    })
}

// covers: CPU-PLN-003
#[test]
fn a_threatened_town_keeps_its_hero_for_an_enemy_hero_in_reach_or_none() {
    let rules = crate::computer::towns::tests::rules_with(40);
    let t0 = TownId::new(0);
    let state = on_town(&rules, |_| {});
    let strong = |state: &GameState| {
        let mut on_map = state.adventure().hero(HERO).unwrap().clone();
        on_map.army = army(&[(IMP, 400), (GOG, 100)]);
        on_map
    };
    let run = |state: &GameState, count: u8, on_map: &HeroOnMap| {
        let reads = town_reads(state, vec![(t0, count)]);
        reads.plan((state, &rules), on_map, |planner| {
            candidates_of(planner, &Plans::default()).1
        })
    };
    // Two threats: none. One: only blue's hero, a must-go.
    let on_map = strong(&state);
    assert_eq!(run(&state, 2, &on_map), []);
    let list = run(&state, 1, &on_map);
    assert_eq!(list.len(), 1);
    let blue = list[0];
    assert_eq!((blue.pos, blue.must_go), (Some(p(2, 2)), true));
    assert!(blue.value > 0);
    // Beyond its points left, or an ally's: none.
    let tired = HeroOnMap {
        movement: 50,
        ..on_map.clone()
    };
    assert_eq!(run(&state, 1, &tired), []);
    let allied = on_town(&rules, |s| s.players[1].team = 0);
    assert_eq!(run(&allied, 1, &strong(&allied)), []);
}

/// The town fixture on a 12 × 12 map of grass, red's hero 0 on `at`.
fn wide_town(rules: &GameRuleset, at: Pos, edit: impl FnOnce(&mut GameSetup)) -> GameState {
    crate::computer::towns::tests::game(rules, 0, |s| {
        let grass = s.map.tiles[0];
        s.map.width = 12;
        s.map.height = 12;
        s.map.tiles = vec![grass; 144];
        s.heroes[0].pos = at;
        edit(s);
    })
}

// covers: CPU-PLN-003
#[test]
fn with_a_town_the_unexplored_tiles_are_worth_100_or_100_000_under_the_turn_s_flag() {
    let rules = crate::computer::towns::tests::rules_sized(40, 12);
    let state = wide_town(&rules, p(0, 0), |_| {});
    let adventure = state.adventure();
    let mut reads = town_reads(&state, vec![]);
    for (flag, value) in [(false, 100), (true, 100_000)] {
        reads.flag = flag;
        let list = reads.red((&state, &rules), |planner| {
            candidates_of(planner, &Plans::default()).1
        });
        let unexplored: Vec<_> = list
            .iter()
            .filter(|c| !adventure.is_explored(RED, c.pos.unwrap()))
            .collect();
        assert!(!unexplored.is_empty());
        assert!(unexplored.iter().all(|c| c.value == value));
    }
}
