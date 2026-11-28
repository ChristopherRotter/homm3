//! Tests of the dwelling and town values (E31 T2g) on the town fixtures: the core's imp (10) and
//! gog (`GOG_AI`, else 40) of `t:town`, a second resource (wood: the Silo's 4), the Alliance at
//! 129; red's T0 at (1, 1) with both dwellings, three guild levels and a University (the part's
//! one skill), blue's T1 at (3, 0) with a Silo and the imps' dwelling; a dwelling of both
//! creatures (its gogs its guards from level 2: 24) at (0, 2). The weights are set by hand (gold,
//! wood) and red's gold too (no starting resources). The imp's and the gog's fight values are
//! their AI values. Where a value adds the fight estimate (T2c), a spell's gain (T2e), a University's value
//! (T2e) or an artifact's (T2f), the test asks that tested function for its part; every number of
//! this task's own comes from an IEEE double model (`E31-log.md` T2g) or is worked by hand.

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    DwellingParams, HeroOnMap, Holds, ObjectKindDef, ObjectSetup, ObjectTypeDef, TileMask, Visit,
};
use h3_content::{ArtifactClass, ArtifactDef, HeroEffectDef, ResourceDef, ResourceId};
use h3_heroes::HeroDetails;
use h3_kernel::arith::{DOUBLE_BITS, X87};
use h3_towns::{
    BuildingDef, BuildingsSetup, GarrisonCreature, GarrisonStack, GuildSlots, SlotSet, TownCommand,
    TownHeroChange,
};

use super::*;
use crate::tests::key;
use crate::town::tests::{rules_with_adventure, town_pack, towns_core_pack, towns_setup};
use crate::{Actor, GameCommand, GameEvent, GameRuleset, GameSetup, apply};

const RED: PlayerId = PlayerId::new(0);
const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);
/// The dwelling's object index (after the two towns) and its tile.
const HUT: usize = 2;
const HUT_AT: Pos = Pos::new(0, 2, 0);

/// The fixture's rules, the gog's AI value `gog`.
pub(in crate::computer) fn rules_with(gog: u32) -> GameRuleset {
    let mut core = towns_core_pack(false);
    for (creature, ai) in core.creatures.iter_mut().zip([10, gog]) {
        creature.info.ai_value = ai;
        creature.info.fight_value = ai;
    }
    core.resources.push(ResourceDef {
        key: key("t:wood"),
        name: "Wood".into(),
    });
    for i in core.artifacts.len()..130 {
        core.artifacts.push(ArtifactDef {
            key: key(&alloc::format!("t:a{i}")),
            name: "Artifact".into(),
            class: ArtifactClass::Treasure,
            cost: vec![],
            slot: Some(key("t:ring_slot")),
            war_machine: None,
            needs_skill: None,
            effects: vec![],
            parts: vec![],
        });
    }
    core.artifacts[129].effects = vec![HeroEffectDef::AlignmentGroup {
        factions: vec![key("t:town"), key("t:other")],
    }];
    let mut towns = town_pack(false);
    let record = &mut towns.towns[0];
    record.silo = vec![(key("t:wood"), 4)];
    for (at, slot) in [(7, 30), (8, 31)] {
        record.buildings[at].slot = Some(slot);
    }
    let building = |k: &str, slot, roles| BuildingDef {
        key: k.into(),
        name: "Building".into(),
        slot: Some(slot),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles,
        war_machine: None,
    };
    for level in 1..=3 {
        let roles = BuildingRoles {
            guild: Some(level),
            ..BuildingRoles::default()
        };
        record
            .buildings
            .push(building(&alloc::format!("guild{level}"), level - 1, roles));
    }
    let university = BuildingRoles {
        university: true,
        ..BuildingRoles::default()
    };
    record
        .buildings
        .push(building("university", 22, university));
    rules_with_adventure(core, &towns, |part| {
        // No starting resources: each test gives red its gold.
        for row in &mut part.starting_resources {
            row.human = vec![];
            row.computer = vec![];
        }
        // The abandoned mine needs a mine of each resource.
        part.object_types.push(ObjectTypeDef {
            key: key("t:wood_mine"),
            name: "Mine".into(),
            visit: Visit::Enter,
            from_above: true,
            kind_mark: false,
            kind: ObjectKindDef::Mine {
                resource: key("t:wood"),
                daily: 1,
                battle_terrain: None,
            },
        });
        part.object_types.push(ObjectTypeDef {
            key: key("t:hut"),
            name: "Hut".into(),
            visit: Visit::Enter,
            from_above: false,
            kind_mark: false,
            kind: ObjectKindDef::Dwelling {
                creatures: vec![key("t:imp"), key("t:gog")],
                blocked: vec![(0, 0)],
                visitable: vec![(0, 0)],
            },
        });
        part.params.as_mut().unwrap().dwellings = Some(DwellingParams {
            guard_level: 2,
            guard_factor: core::num::NonZeroU16::new(3).unwrap(),
            free_level: 1,
        });
    })
}

/// [`rules_with`] the gog at 40.
fn rules() -> GameRuleset {
    rules_with(40)
}

/// Buildings by their slots.
fn built(slots: &[u8]) -> BuildingsSetup {
    BuildingsSetup::Custom {
        built: slots.iter().fold(SlotSet::EMPTY, |set, &s| set.with(s)),
        forbidden: SlotSet::EMPTY,
    }
}

/// The fixture's game with `edit` made to its setup, red given `gold`.
pub(in crate::computer) fn game(rules: &GameRuleset, gold: u32, edit: impl FnOnce(&mut GameSetup)) -> GameState {
    let mut setup: GameSetup = towns_setup(rules);
    setup.towns[0].buildings = built(&[0, 1, 2, 22, 30, 31]);
    setup.towns[1].buildings = built(&[15, 30]);
    setup.map.objects.push(ObjectSetup {
        scenery: Default::default(),
        anchor: HUT_AT,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:hut"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    });
    edit(&mut setup);
    let mut state = GameState::new(rules, &setup, 3).unwrap();
    let mut events: Vec<GameEvent> = Vec::new();
    state.gain(RED, &[(ResourceId::from_raw(0), gold)], &mut events);
    state
}

/// Red's hero on the map with its details.
fn red(state: &GameState) -> (HeroOnMap, HeroDetails) {
    let on_map = state
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == RED)
        .unwrap()
        .clone();
    let details = state.hero(on_map.id).unwrap().clone();
    (on_map, details)
}

/// `n` tenths as a double.
fn tenths(n: u32) -> SignedX87 {
    SignedX87::from(X87::ratio(n, 10, DOUBLE_BITS).unwrap())
}

/// `stacks` (creature, count) in slots 0 up.
fn army(stacks: &[(u16, u32)]) -> Vec<ArmyStack> {
    (0..)
        .zip(stacks)
        .map(|(slot, &(raw, count))| ArmyStack {
            slot,
            creature: CreatureTypeId::from_raw(raw),
            count,
        })
        .collect()
}

/// Asks `f` of red's hero `(on_map, details)` with `record` and `weights`.
fn ask<T>(
    state: &GameState,
    rules: &GameRuleset,
    (on_map, details): &(HeroOnMap, HeroDetails),
    (record, weights): (&Record, &[SignedX87]),
    f: impl FnOnce(Asked<'_, '_>) -> T,
) -> T {
    let hero = Fighter { on_map, details };
    let v = Valued::new(state, rules, (record, weights), hero);
    f(Asked { hero: &v, record })
}

// covers: CPU-TWN-001
#[test]
fn a_dwellings_value_is_its_guards_fight_and_its_purchase() {
    let rules = rules();
    let weights = [tenths(3), SignedX87::ZERO];
    let record = Record::default();
    // Red's own: no fight; the 15 free imps whole (150) into its 30 imps; no gold for the gogs.
    let state = game(&rules, 0, |s| s.map.objects[HUT].owner = Some(RED));
    let (mut on_map, details) = red(&state);
    on_map.army = army(&[(0, 30)]);
    let hero = (on_map, details);
    let value = |hero: &(HeroOnMap, HeroDetails), cost| {
        ask(&state, &rules, hero, (&record, &weights), |a| {
            dwelling_value(a, HUT, (cost, HUT_AT))
        })
    };
    assert_eq!(value(&hero, 0), 150);
    // Far (400): the 150 put in is a third of the army's 300: kept; of 3 000: dropped.
    assert_eq!(value(&hero, 400), 150);
    let mut big = hero.clone();
    big.0.army = army(&[(0, 300)]);
    assert_eq!(value(&big, 399), 150);
    assert_eq!(value(&big, 400), 0);
    // An object that is no dwelling (red's T0): nothing to buy.
    let town = ask(&state, &rules, &hero, (&record, &weights), |a| {
        dwelling_value(a, 0, (0, HUT_AT))
    });
    assert_eq!(town, 0);
    // A neutral one: the fight against its 24 gogs first, then the purchase; another's far
    // purchase is never dropped.
    let state = game(&rules, 0, |_| {});
    let guards = state.adventure().dwelling_guards(HUT);
    assert_eq!(guards, army(&[(1, 24)]));
    let fought = |hero: &(HeroOnMap, HeroDetails)| {
        let fighter = Fighter {
            on_map: &hero.0,
            details: &hero.1,
        };
        let opponent = Opponent {
            hero: None,
            army: &guards,
            town: None,
        };
        fight::fight(&state, &rules, fighter, opponent, Some(HUT_AT))
    };
    let strong = (big.0.clone(), big.1.clone());
    let won = fought(&strong);
    assert!(won > LOST);
    let neutral = |hero: &(HeroOnMap, HeroDetails), cost| {
        ask(&state, &rules, hero, (&record, &weights), |a| {
            dwelling_value(a, HUT, (cost, HUT_AT))
        })
    };
    assert_eq!(neutral(&strong, 400), won + 150);
    // Lost: the fight's value alone.
    let mut weak = hero.clone();
    weak.0.army = army(&[(0, 1)]);
    assert_eq!(fought(&weak), fight::LOST);
    assert_eq!(neutral(&weak, 0), fight::LOST);
}

// covers: CPU-TWN-002
#[test]
fn a_captures_value_is_the_fight_the_income_and_the_recruits() {
    let rules = rules();
    // Gold 0.1, wood 2.5 (the model: 500 × 0.1 × 3.0 = 150; the Silo 3 × trunc(4 × 2.5) = 30;
    // an imp 10 − trunc(50 × 0.1) = 5 a creature).
    let weights = [tenths(1), tenths(25)];
    let record = Record::default();
    let state = game(&rules, 0, |_| {});
    let (mut on_map, details) = red(&state);
    on_map.army = army(&[(0, 300)]);
    on_map.movement = 1000;
    on_map.max_movement = 1500;
    let hero = (on_map, details);
    let fighter = Fighter {
        on_map: &hero.0,
        details: &hero.1,
    };
    let tile = state.adventure().map().town_entrance(T1).unwrap();
    let opponent = Opponent {
        hero: None,
        army: state.adventure().upper_bar(T1),
        town: Some(T1),
    };
    let r = fight::fight(&state, &rules, fighter, opponent, Some(tile));
    let available = i32::from(state.towns().get(T1).unwrap().available(0));
    assert_eq!(available, 15);
    let town = state.towns().get(T1).unwrap();
    let inputs = state.growth_inputs(&rules)[1];
    let growth = i32::from(h3_towns::growth(
        town,
        rules.towns(),
        rules.core(),
        0,
        &inputs,
    ));
    assert_eq!(growth, 15);
    let value = |cost| {
        ask(&state, &rules, &hero, (&record, &weights), |a| {
            capture_value(a, T1, (cost, tile))
        })
    };
    // Blue is human, the difficulty 1: the weight (1 + 1) / 4. V = 150 + 30 + 15 × 5 = 255,
    // trunc(1.5 × 255) = 382; 5 000 000 over the two towns.
    assert_eq!(value(0), r + 382 + 2_500_000);
    // Arriving in six more days (9 000 / 1 500 + day 1 = 7): a week's growth, 30 imps:
    // trunc(1.5 × 330) = 495.
    assert_eq!(value(10_000), r + 495 + 2_500_000);
    assert_eq!(value(9_999), r + 382 + 2_500_000);
}

// covers: CPU-TWN-002
#[test]
fn a_lost_capture_is_worth_its_fight_or_less_without_a_town() {
    let rules = rules();
    let weights = [tenths(1), tenths(25)];
    let record = Record::default();
    let guarded = |s: &mut GameSetup| {
        s.towns[1].garrison = Some(vec![GarrisonStack {
            slot: 0,
            creature: GarrisonCreature::Creature(CreatureTypeId::from_raw(1)),
            count: 500,
        }]);
    };
    let state = game(&rules, 0, guarded);
    let hero = red(&state);
    let tile = state.adventure().map().town_entrance(T1).unwrap();
    let value = |state: &GameState| {
        ask(state, &rules, &hero, (&record, &weights), |a| {
            capture_value(a, T1, (0, tile))
        })
    };
    assert_eq!(value(&state), fight::LOST);
    // Without a town of red's: −2 500 000 + V + 5 000 000.
    let state = game(&rules, 0, |s| {
        guarded(s);
        s.map.objects[0].owner = None;
        s.players[0].start.main_town = None;
    });
    assert_eq!(value(&state), -2_500_000 + 382 + 5_000_000);
}

// covers: CPU-TWN-003
#[test]
fn an_own_towns_army_value_is_half_the_exchange_and_the_purchase() {
    let rules = rules();
    let weights = [tenths(3), SignedX87::ZERO];
    let record = Record::default();
    let state = game(&rules, 1000, |_| {});
    let t0 = state.towns().get(T0).unwrap();
    assert_eq!((t0.available(0), t0.available(1)), (15, 8));
    let (mut on_map, details) = red(&state);
    on_map.army = army(&[(0, 10)]);
    let hero = (on_map, details);
    let value = |hero: &(HeroOnMap, HeroDetails), cost| {
        ask(&state, &rules, hero, (&record, &weights), |a| {
            own_army_value(a, T0, cost)
        })
    };
    // No garrison: no exchange. The gogs first: 320 − 28 (their speed: 1 430 × 600 / 1 500 −
    // 600) less 240 capped at 219: 73; then four imps for the 200 left, 40 less 30: 10.
    assert_eq!(value(&hero, 0), 83);
    // 1 000 imps: the 15 imps (150 less 112: 38), then no gogs worth a place (their speed); far,
    // the 150 put in is below a third of 10 150: 0.
    let mut big = hero.clone();
    big.0.army = army(&[(0, 1000)]);
    assert_eq!(value(&big, 399), 38);
    assert_eq!(value(&big, 400), 0);
    // A cost's word: 65 936 is 400.
    assert_eq!(value(&big, 65_936), 0);
}

// covers: CPU-TWN-004
#[test]
fn a_visit_values_the_university_and_the_guilds_spells() {
    let rules = rules();
    let weights = [tenths(3), SignedX87::ZERO];
    let record = Record::default();
    let mut state = game(&rules, 2500, |_| {});
    let gust = rules.core().spell_by_key("t:gust").unwrap();
    let gate = rules.core().spell_by_key("t:gate").unwrap();
    let t0 = state.towns().get(T0).unwrap();
    let slots = GuildSlots::EMPTY.with(0, 0, gust).with(2, 0, gate);
    let written = t0.with_guild(slots);
    state.towns_mut().write(T0, written);
    let (on_map, mut details) = red(&state);
    let book = rules.core().artifact_by_key("t:book").unwrap();
    details.worn[0] = Some(book.into());
    let mut hero = (on_map, details);
    let spells = |hero: &(HeroOnMap, HeroDetails)| {
        ask(&state, &rules, hero, (&record, &weights), |a| {
            guild_spells(a, state.towns().get(T0).unwrap())
        })
    };
    // Levels 1 and 2 without Wisdom; with Basic Wisdom 3 too; a learned spell not.
    assert_eq!(spells(&hero), [gust]);
    hero.1.skills.push((WISDOM, 1));
    let reads = |hero: &(HeroOnMap, HeroDetails)| {
        let fighter = Fighter {
            on_map: &hero.0,
            details: &hero.1,
        };
        let reads = skills::reads(
            &state,
            &rules,
            fighter,
            &spells::SpellGains::default(),
            &record,
        );
        let offered = &rules.towns().params().unwrap().university_skills;
        let university = skills::university_value(&state, &rules, fighter, &reads, (offered, true));
        let caster = spells::record(&state, &rules, fighter);
        (university, caster)
    };
    assert_eq!(spells(&hero), [gust, gate]);
    hero.1.spells.push(gust);
    assert_eq!(spells(&hero), [gate]);
    let (university, caster) = reads(&hero);
    let value = |hero: &(HeroOnMap, HeroDetails)| {
        ask(&state, &rules, hero, (&record, &weights), |a| {
            visit_value(a, T0)
        })
    };
    assert_eq!(
        value(&hero),
        university + spells::gain(&rules, &caster, gate)
    );
    // No spell book: 1 000 for the 500 gold.
    hero.1.worn[0] = None;
    let (university, _) = reads(&hero);
    assert_eq!(value(&hero), university + 1000);
    let poor = game(&rules, 499, |_| {});
    let value_poor = ask(&poor, &rules, &hero, (&record, &weights), |a| {
        visit_value(a, T0)
    });
    let (university_poor, _) = {
        let fighter = Fighter {
            on_map: &hero.0,
            details: &hero.1,
        };
        let reads = skills::reads(
            &poor,
            &rules,
            fighter,
            &spells::SpellGains::default(),
            &record,
        );
        let offered = &rules.towns().params().unwrap().university_skills;
        (
            skills::university_value(&poor, &rules, fighter, &reads, (offered, true)),
            (),
        )
    };
    assert_eq!(value_poor, university_poor);
    // The record's spell gains for the hero are the University's reads.
    let gains = spells::SpellGains {
        power: 7,
        knowledge: 9,
        ..spells::SpellGains::default()
    };
    let listed = Record {
        heroes: vec![(
            hero.0.id,
            super::super::start::HeroValues {
                experience: 0,
                spells: gains,
            },
        )],
        ..Record::default()
    };
    let fighter = Fighter {
        on_map: &hero.0,
        details: &hero.1,
    };
    let reads = skills::reads(&state, &rules, fighter, &gains, &listed);
    let offered = &rules.towns().params().unwrap().university_skills;
    let university = skills::university_value(&state, &rules, fighter, &reads, (offered, true));
    let listed_value = ask(&state, &rules, &hero, (&listed, &weights), |a| {
        visit_value(a, T0)
    });
    assert_eq!(listed_value, university + 1000);
    // T1: neither; a town the map lacks: 0.
    let t1 = ask(&state, &rules, &hero, (&record, &weights), |a| {
        visit_value(a, T1)
    });
    assert_eq!(t1, 0);
    let lacked = ask(&state, &rules, &hero, (&record, &weights), |a| {
        visit_value(a, TownId::new(9))
    });
    assert_eq!(lacked, 0);
}

// covers: CPU-TWN-005
#[test]
fn a_towns_value_by_its_owner() {
    let rules = rules();
    let weights = [tenths(3), SignedX87::ZERO];
    let grail = ArtifactInstance::from(GRAIL);
    let part = ArtifactInstance::from(ArtifactTypeId::from_raw(118));
    // The sage at T0's gate goes into its garrison.
    let mut state = game(&rules, 1000, |s| {
        let mut sage = s.heroes[0].clone();
        sage.hero_type = rules.core().hero_type_by_key("t:sage");
        sage.pos = Pos::new(1, 1, 0);
        sage.army = h3_game_army(&army(&[(1, 2)]));
        s.heroes.push(sage);
    });
    let enter = GameCommand::Town(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    });
    let mut events: Vec<GameEvent> = Vec::new();
    apply(&mut state, &rules, Actor::Player(RED), enter, &mut events).unwrap();
    assert!(state.adventure().town_hero(T0).is_some());
    let (mut on_map, mut details) = red(&state);
    on_map.army = army(&[(0, 10)]);
    details.backpack = vec![grail, part];
    let hero = (on_map, details);
    let tile = Pos::new(1, 1, 0);
    // Another's: the capture's value.
    let none = Record::default();
    let asked = |record: &Record, town, cost| {
        ask(&state, &rules, &hero, (record, &weights), |a| {
            town_value(a, town, (cost, tile))
        })
    };
    let capture = ask(&state, &rules, &hero, (&none, &weights), |a| {
        capture_value(a, T1, (0, tile))
    });
    assert_eq!(asked(&none, T1, 0), capture);
    // Its own: the army's and the visit's, the Grail's value to red (its building free), 5 000
    // 000 over two towns while threatened, the Legion's part to the garrison hero, counted.
    let threatened = Record {
        towns: vec![(T0, 1)],
        ..Record::default()
    };
    let parts = ask(&state, &rules, &hero, (&threatened, &weights), |a| {
        let own = own_army_value(a, T0, 0).wrapping_add(visit_value(a, T0));
        let heroes = artifacts::players_heroes(&state, &rules, (&threatened, &weights), (RED, &[]));
        let grail = artifacts::value_to_player(&heroes, grail);
        let garrison = state.adventure().town_hero(T0).unwrap();
        let details = state.hero(garrison.id).unwrap();
        let g = Fighter {
            on_map: garrison,
            details,
        };
        let reads = Valued::new(&state, &rules, (&threatened, &weights), g);
        (own, grail, artifacts::value_to_hero(&reads, part, true))
    });
    let (own, grail_value, part_value) = parts;
    assert!(grail_value >= 10);
    assert_eq!(
        asked(&threatened, T0, 0),
        own + grail_value + 2_500_000 + part_value
    );
    assert_eq!(asked(&none, T0, 0), own + grail_value + part_value);
    // A town the map lacks: 0.
    assert_eq!(asked(&none, TownId::new(9), 0), 0);
    // The capture's fight is against a garrison hero too (T0's sage, as if T0 were another's):
    // V = trunc(500 × 0.3 × 3.0) = 450, no Silo, the gogs 8 × (40 − 30); trunc(1.5 × 530) = 795.
    let sage = state.adventure().town_hero(T0).unwrap();
    let sage_details = state.hero(sage.id).unwrap();
    let opponent = Opponent {
        hero: Some(Fighter {
            on_map: sage,
            details: sage_details,
        }),
        army: state.adventure().upper_bar(T0),
        town: Some(T0),
    };
    let fighter = Fighter {
        on_map: &hero.0,
        details: &hero.1,
    };
    let r = fight::fight(&state, &rules, fighter, opponent, Some(tile));
    let capture = ask(&state, &rules, &hero, (&none, &weights), |a| {
        capture_value(a, T0, (0, tile))
    });
    assert_eq!(capture, r + 795 + 2_500_000);
}

/// An army setup of `stacks`.
fn h3_game_army(stacks: &[ArmyStack]) -> crate::HeroArmy {
    crate::HeroArmy::Custom(stacks.to_vec())
}

// covers: CPU-TWN-002, CPU-TWN-003
#[test]
fn the_grails_gold_and_the_alliances_flag() {
    let rules = rules();
    // A town with the Grail's building ever built: its hall's 500 and the Grail's 5 000.
    let state = game(&rules, 0, |_| {});
    let t0 = state.towns().get(T0).unwrap();
    let grail = h3_towns::BuildingId::from_index(9).unwrap();
    let with_grail = Town::new(t0.faction(), t0.built().with(grail), t0.forbidden());
    let record = rules.towns().town(t0.faction());
    assert_eq!(town_gold(record, &with_grail, &rules), 5500);
    assert_eq!(town_gold(record, t0, &rules), 500);
    // Red's hero wearing the Alliance: its factions join; else none.
    let factions = armies::alliance_factions(&rules);
    assert_eq!(factions.len(), 2);
    let record = Record::default();
    let hero = red(&state);
    let none = ask(&state, &rules, &hero, (&record, &[]), |a| {
        a.join(&factions).len()
    });
    assert_eq!(none, 0);
    let alliance = ArtifactInstance::from(armies::ANGELIC_ALLIANCE);
    let state = game(&rules, 0, |s| {
        s.heroes[0].details.artifacts = Some(crate::HeroArtifactsSetup {
            worn: vec![None, None, Some(alliance), None],
            backpack: vec![],
        });
    });
    let hero = red(&state);
    let joined = ask(&state, &rules, &hero, (&record, &[]), |a| {
        a.join(&factions).len()
    });
    assert_eq!(joined, 2);
}

// covers: CPU-TWN-005
#[test]
fn an_own_towns_value_is_at_most_5_000_000() {
    // Gogs worth 400 each: 30 000 in T0's garrison, a hero of primary sum 160 over no garrison
    // hero: the exchange's 48 000 000 or so, half of it far above the cap.
    let rules = rules_with(400);
    let weights = [tenths(3), SignedX87::ZERO];
    let record = Record::default();
    let state = game(&rules, 0, |s| {
        s.towns[0].garrison = Some(vec![GarrisonStack {
            slot: 0,
            creature: GarrisonCreature::Creature(CreatureTypeId::from_raw(1)),
            count: 30_000,
        }]);
    });
    let (mut on_map, mut details) = red(&state);
    on_map.army = army(&[(0, 10)]);
    details.primary = [40, 40, 40, 40];
    let hero = (on_map, details);
    let tile = Pos::new(1, 1, 0);
    let own = ask(&state, &rules, &hero, (&record, &weights), |a| {
        own_army_value(a, T0, 0)
    });
    assert!(own > MOST);
    let value = ask(&state, &rules, &hero, (&record, &weights), |a| {
        town_value(a, T0, (0, tile))
    });
    assert_eq!(value, MOST);
}
