//! The whole game's wire (E16 M5 T2): the observation maps every field of the engine's,
//! losslessly (a test-only inverse gives the engine's observation back), for a hand-built one
//! with every variant and for every viewer of games played at random on the fixture maps; its
//! parts round-trip.

use std::sync::{Arc, LazyLock};
use std::vec;
use std::vec::Vec;

use h3_adventure::{
    AdventureObservation, BoatView, ChoiceKind as EngineChoiceKind, GuardsView, HeroView,
    Known as EngineKnown, KnownObject, MonsterView, ObjectView, OfferKind as EngineOfferKind,
    OwnHeroView, StaticTile, TileMask, TileView,
};
use h3_battle::BattleSide;
use h3_content::Formation as EngineFormation;
use h3_contracts::{ArmyStack as EngineStack, FortLevel, HeroId, PlayerId, TownId};
use h3_game::{
    DecisionView, GameRuleset, GarrisonStackView, GarrisonView, HireView, LevelUpView, MonthKind,
    PlayerView, SlotState as EngineSlot, TownOwnerView, TownView, Viewer,
};
use h3_heroes::PrimarySkill as EnginePrimary;
use h3_kernel::handle::Handle;
use h3_kernel::rng::{Rng, StreamId};
use h3_sim::adventure::AdventureEnv;
use h3_sim::adventure::rollout::choose;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::map::game::{adventure_view, observation, observe};
use crate::wire::WireU64;
use crate::wire::game::*;

static RULES: LazyLock<GameRuleset> = LazyLock::new(|| {
    let profile = "base:faithful";
    let (core, part) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile).unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .unwrap();
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS, profile).unwrap();
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .unwrap()
});

fn round_trips<T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug>(value: &T) {
    let json = serde_json::to_vec(value).unwrap();
    assert_eq!(&serde_json::from_slice::<T>(&json).unwrap(), value);
    let binary = postcard::to_allocvec(value).unwrap();
    assert_eq!(&postcard::from_bytes::<T>(&binary).unwrap(), value);
}

// The inverse: the wire back to the engine's types, field by field (test code, independent of
// the mapping it checks).

fn pos_back(p: Pos) -> h3_adventure::Pos {
    h3_adventure::Pos::new(p.x, p.y, p.level)
}

fn army_back(stacks: &[ArmyStack]) -> Vec<EngineStack> {
    stacks
        .iter()
        .map(|s| EngineStack {
            slot: s.slot,
            creature: Handle::from_raw(s.creature),
            count: s.count,
        })
        .collect()
}

fn tile_back(t: Tile) -> TileView {
    match t {
        Tile::Hidden {} => TileView::Hidden,
        Tile::Ground {
            terrain,
            road,
            river,
        } => TileView::Ground {
            terrain: Handle::from_raw(terrain),
            road: road.map(Handle::from_raw),
            river: river.map(Handle::from_raw),
        },
        Tile::Seen {
            terrain,
            road,
            river,
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        } => TileView::Seen(StaticTile {
            terrain: Handle::from_raw(terrain),
            road: road.map(Handle::from_raw),
            river: river.map(Handle::from_raw),
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        }),
    }
}

fn inverse(a: &Adventure) -> AdventureObservation {
    AdventureObservation {
        viewer: PlayerId::new(a.viewer),
        width: a.width,
        height: a.height,
        levels: a.levels,
        stride: usize::from(a.stride),
        explored: a.explored.iter().map(|w| w.0).collect(),
        fringe: a.fringe.iter().map(|w| w.0).collect(),
        tiles: (0..a.tiles.len())
            .map(|i| tile_back(*a.tile(i).unwrap()))
            .collect(),
        heroes: a
            .heroes
            .iter()
            .map(|h| HeroView {
                id: HeroId::new(h.id),
                owner: PlayerId::new(h.owner),
                pos: pos_back(h.pos),
                boat: h.boat,
                own: h.own.as_ref().map(|o| OwnHeroView {
                    army: army_back(&o.army),
                    movement: o.movement,
                    max_movement: o.max_movement,
                    formation: match o.formation {
                        Formation::Loose => EngineFormation::Loose,
                        Formation::Tight => EngineFormation::Tight,
                    },
                    tactics: o.tactics,
                    marks: o
                        .marks
                        .iter()
                        .map(|m| (Handle::from_raw(m.object_type), pos_back(m.anchor)))
                        .collect(),
                }),
            })
            .collect(),
        objects: a
            .objects
            .iter()
            .map(|o| ObjectView {
                object_type: Handle::from_raw(o.object_type),
                anchor: pos_back(o.anchor),
                blocked: TileMask::from_bits(o.blocked.0),
                visitable: TileMask::from_bits(o.visitable.0),
                drawn: TileMask::from_bits(o.drawn.0),
                monster: o.monster.map(|m| MonsterView {
                    creature: Handle::from_raw(m.creature),
                    count_word: m.count_word,
                }),
                owner: o.owner.map(PlayerId::new),
                production: o
                    .production
                    .map(|p| (Handle::from_raw(p.resource), p.amount)),
                guards: o.guards.as_ref().map(|g| GuardsView {
                    creature: g.creature.map(Handle::from_raw),
                    count_word: g.count_word,
                    army: g.army.as_deref().map(army_back),
                }),
                visited: o.visited,
                boat_tile: o.boat_tile.map(pos_back),
                town: o.town.map(TownId::new),
            })
            .collect(),
        bought: a.bought.iter().copied().map(Handle::from_raw).collect(),
        keys: a.keys,
        known: a
            .known
            .iter()
            .map(|k| KnownObject {
                object_type: Handle::from_raw(k.object_type),
                anchor: pos_back(k.anchor),
                known: match &k.known {
                    Known::Spell { spell } => EngineKnown::Spell(spell.map(Handle::from_raw)),
                    Known::Skill { skill } => EngineKnown::Skill(skill.map(Handle::from_raw)),
                    Known::Price { price } => EngineKnown::Price(
                        price
                            .iter()
                            .map(|p| (Handle::from_raw(p.resource), p.amount))
                            .collect(),
                    ),
                    Known::Skills { skills } => {
                        EngineKnown::Skills(skills.iter().copied().map(Handle::from_raw).collect())
                    }
                    Known::Emptied { emptied } => EngineKnown::Emptied(*emptied),
                },
            })
            .collect(),
        boats: a
            .boats
            .iter()
            .map(|b| BoatView {
                tile: pos_back(b.tile),
                kind: b.kind,
                facing: b.facing,
            })
            .collect(),
    }
}

/// The palette holds each view once, in the order the tiles first use them.
fn check_palette(a: &Adventure) {
    let mut next = 0u32;
    for &i in &a.tiles {
        assert!(i <= next, "index {i} before {next}");
        if i == next {
            next += 1;
        }
    }
    assert_eq!(next as usize, a.palette.len());
    let distinct: std::collections::BTreeSet<_> = a.palette.iter().collect();
    assert_eq!(distinct.len(), a.palette.len());
}

fn stacks(n: u8) -> Vec<EngineStack> {
    (0..n)
        .map(|slot| EngineStack {
            slot,
            creature: Handle::from_raw(10 + u16::from(slot)),
            count: 5 * u32::from(slot) + 1,
        })
        .collect()
}

/// An observation with every variant and every optional part both present and absent.
fn every_part() -> AdventureObservation {
    let p = h3_adventure::Pos::new;
    let seen = |blocked| {
        TileView::Seen(StaticTile {
            terrain: Handle::from_raw(2),
            road: Some(Handle::from_raw(1)),
            river: Some(Handle::from_raw(3)),
            blocked,
            visitable: !blocked,
            no_native: true,
            winds: false,
            hole: true,
        })
    };
    let ground = TileView::Ground {
        terrain: Handle::from_raw(7),
        road: Some(Handle::from_raw(0)),
        river: None,
    };
    let object = |town: bool| ObjectView {
        object_type: Handle::from_raw(40),
        anchor: p(3, 1, 0),
        blocked: TileMask::from_bits(0b0110 << 40),
        visitable: TileMask::from_bits(1 << 47),
        drawn: TileMask::from_bits(0xffff_ffff_ffff),
        monster: town.then_some(MonsterView {
            creature: Handle::from_raw(12),
            count_word: 4,
        }),
        owner: town.then_some(PlayerId::new(1)),
        production: town.then_some((Handle::from_raw(6), 1000)),
        guards: town.then(|| GuardsView {
            creature: None,
            count_word: 2,
            army: Some(stacks(2)),
        }),
        visited: town,
        boat_tile: town.then_some(p(4, 2, 0)),
        town: town.then_some(TownId::new(3)),
    };
    AdventureObservation {
        viewer: PlayerId::new(1),
        width: 3,
        height: 2,
        levels: 1,
        stride: 1,
        explored: vec![0b11_0100],
        fringe: vec![0b00_1010],
        tiles: vec![
            TileView::Hidden,
            ground,
            seen(true),
            seen(true),
            seen(false),
            TileView::Hidden,
        ],
        heroes: vec![
            HeroView {
                id: HeroId::new(4),
                owner: PlayerId::new(1),
                pos: p(2, 1, 0),
                boat: None,
                own: Some(OwnHeroView {
                    army: stacks(3),
                    movement: 1500,
                    max_movement: 2000,
                    formation: EngineFormation::Tight,
                    tactics: false,
                    marks: vec![(Handle::from_raw(9), p(1, 1, 0))],
                }),
            },
            HeroView {
                id: HeroId::new(5),
                owner: PlayerId::new(1),
                pos: p(0, 1, 0),
                boat: Some(2),
                own: Some(OwnHeroView {
                    army: Vec::new(),
                    movement: 0,
                    max_movement: 1300,
                    formation: EngineFormation::Loose,
                    tactics: true,
                    marks: Vec::new(),
                }),
            },
            HeroView {
                id: HeroId::new(9),
                owner: PlayerId::new(0),
                pos: p(1, 0, 0),
                boat: None,
                own: None,
            },
        ],
        objects: vec![
            object(true),
            object(false),
            ObjectView {
                guards: Some(GuardsView {
                    creature: Some(Handle::from_raw(8)),
                    count_word: 0,
                    army: None,
                }),
                ..object(false)
            },
        ],
        bought: vec![Handle::from_raw(30), Handle::from_raw(31)],
        keys: 0b1000_0001,
        known: [
            EngineKnown::Spell(Some(Handle::from_raw(17))),
            EngineKnown::Spell(None),
            EngineKnown::Skill(Some(Handle::from_raw(5))),
            EngineKnown::Skill(None),
            EngineKnown::Price(vec![(Handle::from_raw(6), 2000), (Handle::from_raw(2), 5)]),
            EngineKnown::Skills(vec![Handle::from_raw(1), Handle::from_raw(11)]),
            EngineKnown::Emptied(true),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, known)| KnownObject {
            object_type: Handle::from_raw(50),
            anchor: p(u16::try_from(i).unwrap(), 0, 0),
            known,
        })
        .collect(),
        boats: vec![BoatView {
            tile: p(1, 1, 0),
            kind: 1,
            facing: 6,
        }],
    }
}

#[test]
fn every_part_of_the_adventure_map_maps_back_and_round_trips() {
    let o = every_part();
    let wire = adventure_view(&o);
    assert_eq!(inverse(&wire), o);
    assert_eq!(wire.tiles, [0, 1, 2, 2, 3, 0]);
    assert_eq!(wire.palette.len(), 4);
    check_palette(&wire);
    assert_eq!(wire.tile(6), None);
    round_trips(&wire);
}

fn text<T: Serialize>(value: &T) -> std::string::String {
    serde_json::to_string(value).unwrap()
}

#[test]
fn the_adventure_maps_json_is_serdes_default_form() {
    assert_eq!(text(&Tile::Hidden {}), r#"{"Hidden":{}}"#);
    assert_eq!(
        text(&Tile::Ground {
            terrain: 7,
            road: Some(0),
            river: None
        }),
        r#"{"Ground":{"terrain":7,"road":0,"river":null}}"#
    );
    assert_eq!(text(&Formation::Tight), r#""Tight""#);
    assert_eq!(
        text(&Known::Spell { spell: None }),
        r#"{"Spell":{"spell":null}}"#
    );
    // A plane's word and a mask are 64-bit values: decimal strings.
    let wire = adventure_view(&every_part());
    assert_eq!(text(&wire.explored), r#"["52"]"#);
    assert_eq!(text(&wire.objects[0].visitable), r#""140737488355328""#);
    assert_eq!(text(&wire.objects[0].anchor), r#"{"x":3,"y":1,"level":0}"#);
    // Unknown fields are refused.
    let extra = r#"{"x":3,"y":1,"level":0,"z":1}"#;
    assert!(serde_json::from_str::<Pos>(extra).is_err());
    assert_eq!(WireU64(52), wire.explored[0]);
}

/// Plays `steps` random steps of `map` and checks every viewer's observation after each.
fn check_played(map: &[u8], seed: u64, steps: usize) -> (usize, usize) {
    let rules = &*RULES;
    let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
    let setup = Arc::new(h3_map::import(&m, rules).unwrap());
    let mut env = AdventureEnv::new(rules.clone(), setup, seed, 28).unwrap();
    let mut rng = Rng::for_stream(seed, StreamId::AGENT);
    let (mut largest, mut checked) = (0, 0);
    for step in 0..steps {
        for player in (0..=u8::MAX).take(env.state().players()) {
            let viewer = PlayerId::new(player);
            let o = h3_game::observe(env.state(), rules, Viewer::Player(viewer));
            let wire = observe(env.state(), rules, Viewer::Player(viewer));
            assert_eq!(wire, observation(&o));
            assert_eq!(
                inverse_game(&wire, o.battle.clone()),
                o,
                "step {step}, player {player}"
            );
            check_palette(&wire.adventure);
            if step % 50 == 0 || wire.decision.is_some() || wire.battle.is_some() {
                round_trips(&wire);
            }
            largest = largest.max(wire.adventure.palette.len());
            checked += 1;
        }
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        env.step(seat, action).unwrap();
    }
    (largest, checked)
}

#[test]
fn a_played_games_observations_map_back_for_every_viewer() {
    let maps: [(&[u8], u64); 5] = [
        (
            include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m"),
            1,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e09-towns-36.h3m"),
            2,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-rewards-36.h3m"),
            3,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-economy-36.h3m"),
            4,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-sea-36.h3m"),
            5,
        ),
    ];
    for (map, seed) in maps {
        let (largest, checked) = check_played(map, seed, 400);
        assert!(checked >= 300, "{checked}");
        // A map's distinct tile views are few: 4 to 64 on these maps of 2 592 tiles.
        assert!((2..200).contains(&largest), "{largest}");
    }
}

// The whole observation's inverse (T2b); the battle's own mapping is the battle wire's
// (`tests/mapping.rs`), so the battle is passed in and compared on the wire.

fn amounts_back(a: &[Amount]) -> Vec<(h3_content::ResourceId, u32)> {
    a.iter()
        .map(|p| (Handle::from_raw(p.resource), p.amount))
        .collect()
}

fn primary_back(p: PrimarySkill) -> EnginePrimary {
    match p {
        PrimarySkill::Attack => EnginePrimary::Attack,
        PrimarySkill::Defense => EnginePrimary::Defense,
        PrimarySkill::Power => EnginePrimary::Power,
        PrimarySkill::Knowledge => EnginePrimary::Knowledge,
    }
}

fn decision_back(d: &Decision) -> DecisionView {
    match d {
        Decision::Offer { offer } => DecisionView::Offer(h3_adventure::Offer {
            hero: HeroId::new(offer.hero),
            object_type: Handle::from_raw(offer.object_type),
            anchor: pos_back(offer.anchor),
            kind: match &offer.kind {
                OfferKind::Reveal { price } => EngineOfferKind::Reveal {
                    price: amounts_back(price),
                },
                OfferKind::Pass {} => EngineOfferKind::Pass,
                OfferKind::Join {} => EngineOfferKind::Join,
                OfferKind::JoinForGold { price } => EngineOfferKind::JoinForGold {
                    price: amounts_back(price),
                },
                OfferKind::Pursue {} => EngineOfferKind::Pursue,
                OfferKind::Fight {} => EngineOfferKind::Fight,
                OfferKind::Gift { price } => EngineOfferKind::Gift {
                    price: amounts_back(price),
                },
                OfferKind::Search {} => EngineOfferKind::Search,
                OfferKind::OpenBox {} => EngineOfferKind::OpenBox,
                OfferKind::SearchPyramid {} => EngineOfferKind::SearchPyramid,
                OfferKind::Boat { price } => EngineOfferKind::Boat {
                    price: amounts_back(price),
                },
                OfferKind::Hire { creatures } => EngineOfferKind::Hire {
                    creatures: creatures.iter().copied().map(Handle::from_raw).collect(),
                },
                OfferKind::Guarded {
                    creature,
                    count_word,
                } => EngineOfferKind::Guarded {
                    creature: Handle::from_raw(*creature),
                    count_word: *count_word,
                },
            },
        }),
        Decision::LevelUp { level_up } => DecisionView::LevelUp(LevelUpView {
            hero: HeroId::new(level_up.hero),
            level: level_up.level,
            primary: primary_back(level_up.primary),
            offers: level_up.offers.map(Handle::from_raw),
        }),
        Decision::Meeting { heroes } => DecisionView::Meeting {
            heroes: heroes.map(HeroId::new),
        },
        Decision::ArmyDialog { hero, group, mine } => DecisionView::ArmyDialog {
            hero: HeroId::new(*hero),
            group: army_back(group),
            mine: mine.map(|m| (Handle::from_raw(m.object_type), pos_back(m.anchor))),
        },
        Decision::Choice { choice } => DecisionView::Choice(h3_adventure::Choice {
            hero: HeroId::new(choice.hero),
            object_type: Handle::from_raw(choice.object_type),
            anchor: pos_back(choice.anchor),
            kind: match &choice.kind {
                ChoiceKind::Chest { gold, experience } => EngineChoiceKind::Chest {
                    gold: (Handle::from_raw(gold.resource), gold.amount),
                    experience: *experience,
                },
                ChoiceKind::Primary {
                    skills,
                    amount,
                    price,
                    leave,
                } => EngineChoiceKind::Primary {
                    skills: skills.map(primary_back),
                    amount: *amount,
                    price: amounts_back(price),
                    leave: *leave,
                },
            },
        }),
        Decision::University { visit, slots } => DecisionView::University {
            visit: h3_adventure::UniversityVisit {
                hero: HeroId::new(visit.hero),
                object_type: Handle::from_raw(visit.object_type),
                anchor: pos_back(visit.anchor),
                skills: visit.skills.iter().copied().map(Handle::from_raw).collect(),
                price: amounts_back(&visit.price),
            },
            slots: slots
                .iter()
                .map(|s| match s {
                    SlotState::Known => EngineSlot::Known,
                    SlotState::NoRoom => EngineSlot::NoRoom,
                    SlotState::Barred => EngineSlot::Barred,
                    SlotState::Open => EngineSlot::Open,
                })
                .collect(),
        },
        Decision::Hire { hire } => DecisionView::Hire(HireView {
            target: match hire.target {
                HireTarget::Hero { hero } => h3_adventure::HireTarget::Hero(HeroId::new(hero)),
            },
            object_type: Handle::from_raw(hire.object_type),
            anchor: pos_back(hire.anchor),
            creatures: hire
                .creatures
                .iter()
                .map(|c| h3_game::HireCreature {
                    creature: Handle::from_raw(c.creature),
                    count: c.count,
                    cost: amounts_back(&c.cost),
                })
                .collect(),
        }),
    }
}

fn artifact_back(a: Artifact) -> h3_heroes::ArtifactInstance {
    h3_heroes::ArtifactInstance {
        artifact: Handle::from_raw(a.artifact),
        spell: a.spell.map(Handle::from_raw),
    }
}

fn details_back(d: &HeroDetails) -> h3_heroes::HeroDetails {
    h3_heroes::HeroDetails {
        hero_type: Handle::from_raw(d.hero_type),
        level: d.level,
        experience: d.experience,
        offered: h3_heroes::OfferLevels {
            wisdom: d.offered.wisdom,
            school: d.offered.school,
        },
        primary: d.primary,
        skills: d
            .skills
            .iter()
            .map(|s| (Handle::from_raw(s.skill), s.level))
            .collect(),
        worn: d.worn.iter().map(|a| a.map(artifact_back)).collect(),
        backpack: d.backpack.iter().copied().map(artifact_back).collect(),
        spells: d.spells.iter().copied().map(Handle::from_raw).collect(),
        mana: d.mana,
        bonuses: h3_heroes::BattleBonuses {
            morale: d.bonuses.morale,
            luck: d.bonuses.luck,
            marks: d.bonuses.marks,
        },
    }
}

fn town_back(t: &Town) -> TownView {
    TownView {
        town: TownId::new(t.town),
        anchor: pos_back(t.anchor),
        owner: t.owner.map(PlayerId::new),
        faction: Handle::from_raw(t.faction),
        name: t.name.clone(),
        fort: [
            FortLevel::None,
            FortLevel::Fort,
            FortLevel::Citadel,
            FortLevel::Castle,
        ][usize::from(t.fort)],
        capitol: t.capitol,
        garrison: GarrisonView {
            level: t.garrison.level,
            stacks: t
                .garrison
                .stacks
                .iter()
                .map(|s| GarrisonStackView {
                    creature: Handle::from_raw(s.creature),
                    count_word: s.count_word,
                    count: s.count,
                })
                .collect(),
            army: t.garrison.army.as_deref().map(army_back),
        },
        owner_facts: t.owner_facts.as_ref().map(|f| TownOwnerView {
            hall: f.hall,
            income: amounts_back(&f.income),
        }),
    }
}

/// The wire's observation back in the engine's types, with `battle` for the battle's part.
fn inverse_game(
    g: &GameObservation,
    battle: Option<h3_battle::Observation>,
) -> h3_game::GameObservation {
    assert_eq!(g.battle, battle.as_ref().map(crate::map::observation));
    h3_game::GameObservation {
        viewer: PlayerId::new(g.viewer),
        day: g.day,
        month: match g.month {
            Month::Normal {} => MonthKind::Normal,
            Month::Creature { creature } => MonthKind::Creature(Handle::from_raw(creature)),
            Month::Plague {} => MonthKind::Plague,
        },
        players: g
            .players
            .iter()
            .map(|p| PlayerView {
                colour: p.colour,
                acting: p.acting,
            })
            .collect(),
        turn_group: g.turn_group.iter().copied().map(PlayerId::new).collect(),
        treasury: g.treasury.clone(),
        events_today: g.events_today.clone(),
        decision: g.decision.as_ref().map(decision_back),
        heroes: g
            .heroes
            .iter()
            .map(|h| h3_game::HeroInfo {
                id: HeroId::new(h.id),
                hero_type: Handle::from_raw(h.hero_type),
                details: h.details.as_ref().map(|d| Arc::new(details_back(d))),
            })
            .collect(),
        adventure: inverse(&g.adventure),
        towns: g.towns.iter().map(town_back).collect(),
        battle,
    }
}

fn price() -> Vec<(h3_content::ResourceId, u32)> {
    vec![(Handle::from_raw(6), 1500), (Handle::from_raw(1), 2)]
}

/// One open decision of every kind, every offer's kind and every choice's.
fn every_decision() -> Vec<DecisionView> {
    let p = h3_adventure::Pos::new;
    let hero = HeroId::new(4);
    let (object_type, anchor) = (Handle::from_raw(70), p(5, 6, 1));
    let offers = [
        EngineOfferKind::Reveal { price: price() },
        EngineOfferKind::Pass,
        EngineOfferKind::Join,
        EngineOfferKind::JoinForGold { price: price() },
        EngineOfferKind::Pursue,
        EngineOfferKind::Fight,
        EngineOfferKind::Gift { price: Vec::new() },
        EngineOfferKind::Search,
        EngineOfferKind::OpenBox,
        EngineOfferKind::SearchPyramid,
        EngineOfferKind::Boat { price: price() },
        EngineOfferKind::Hire {
            creatures: vec![Handle::from_raw(3), Handle::from_raw(4)],
        },
        EngineOfferKind::Guarded {
            creature: Handle::from_raw(9),
            count_word: 3,
        },
    ];
    let mut out: Vec<DecisionView> = offers
        .into_iter()
        .map(|kind| {
            DecisionView::Offer(h3_adventure::Offer {
                hero,
                object_type,
                anchor,
                kind,
            })
        })
        .collect();
    let primaries = [
        EnginePrimary::Attack,
        EnginePrimary::Defense,
        EnginePrimary::Power,
        EnginePrimary::Knowledge,
    ];
    out.extend(primaries.map(|primary| {
        DecisionView::LevelUp(LevelUpView {
            hero,
            level: 5,
            primary,
            offers: [Handle::from_raw(2), Handle::from_raw(14)],
        })
    }));
    out.push(DecisionView::Meeting {
        heroes: [hero, HeroId::new(7)],
    });
    out.push(DecisionView::ArmyDialog {
        hero,
        group: stacks(2),
        mine: Some((object_type, anchor)),
    });
    out.push(DecisionView::ArmyDialog {
        hero,
        group: Vec::new(),
        mine: None,
    });
    let choice = |kind| {
        DecisionView::Choice(h3_adventure::Choice {
            hero,
            object_type,
            anchor,
            kind,
        })
    };
    out.push(choice(EngineChoiceKind::Chest {
        gold: (Handle::from_raw(6), 1000),
        experience: 500,
    }));
    out.push(choice(EngineChoiceKind::Primary {
        skills: [EnginePrimary::Attack, EnginePrimary::Knowledge],
        amount: 2,
        price: price(),
        leave: true,
    }));
    out.push(DecisionView::University {
        visit: h3_adventure::UniversityVisit {
            hero,
            object_type,
            anchor,
            skills: vec![
                Handle::from_raw(1),
                Handle::from_raw(2),
                Handle::from_raw(3),
            ],
            price: price(),
        },
        slots: vec![
            EngineSlot::Known,
            EngineSlot::NoRoom,
            EngineSlot::Barred,
            EngineSlot::Open,
        ],
    });
    out.push(DecisionView::Hire(HireView {
        target: h3_adventure::HireTarget::Hero(hero),
        object_type,
        anchor,
        creatures: vec![h3_game::HireCreature {
            creature: Handle::from_raw(12),
            count: 7,
            cost: price(),
        }],
    }));
    out
}

/// A whole observation with every optional part present, over the hand-built adventure map.
fn every_game_part(decision: Option<DecisionView>, month: MonthKind) -> h3_game::GameObservation {
    let p = h3_adventure::Pos::new;
    let (ruleset, state) = super::battle();
    let details = h3_heroes::HeroDetails {
        hero_type: Handle::from_raw(33),
        level: 6,
        experience: 4500,
        offered: h3_heroes::OfferLevels {
            wisdom: 3,
            school: 5,
        },
        primary: [2, 1, 3, 4],
        skills: vec![(Handle::from_raw(7), 2), (Handle::from_raw(21), 1)],
        worn: vec![
            None,
            Some(Handle::from_raw(40).into()),
            Some(h3_heroes::ArtifactInstance::scroll(
                Handle::from_raw(1),
                Handle::from_raw(15),
            )),
        ],
        backpack: vec![Handle::from_raw(55).into()],
        spells: vec![Handle::from_raw(10), Handle::from_raw(12)],
        mana: 30,
        bonuses: h3_heroes::BattleBonuses {
            morale: -3,
            luck: 2,
            marks: 0b101,
        },
    };
    let town = |fort, top: bool| TownView {
        town: TownId::new(2),
        anchor: p(9, 9, 0),
        owner: top.then_some(PlayerId::new(1)),
        faction: Handle::from_raw(4),
        name: top.then(|| std::string::String::from("Arran")),
        fort,
        capitol: top,
        garrison: GarrisonView {
            level: if top { 3 } else { 1 },
            stacks: vec![GarrisonStackView {
                creature: Handle::from_raw(8),
                count_word: top.then_some(2),
                count: top.then_some(14),
            }],
            army: top.then(|| stacks(1)),
        },
        owner_facts: top.then(|| TownOwnerView {
            hall: 4,
            income: price(),
        }),
    };
    h3_game::GameObservation {
        viewer: PlayerId::new(1),
        day: 30,
        month,
        players: vec![
            PlayerView {
                colour: 0,
                acting: false,
            },
            PlayerView {
                colour: 3,
                acting: true,
            },
        ],
        turn_group: vec![PlayerId::new(1)],
        treasury: vec![20, 5, 10, 5, 5, 5, 7500],
        events_today: vec![0, 2],
        decision,
        heroes: vec![
            h3_game::HeroInfo {
                id: HeroId::new(4),
                hero_type: Handle::from_raw(33),
                details: Some(Arc::new(details)),
            },
            h3_game::HeroInfo {
                id: HeroId::new(9),
                hero_type: Handle::from_raw(70),
                details: None,
            },
        ],
        adventure: every_part(),
        towns: vec![
            town(FortLevel::None, false),
            town(FortLevel::Fort, true),
            town(FortLevel::Citadel, false),
            town(FortLevel::Castle, true),
        ],
        battle: Some(h3_battle::observe(&state, &ruleset, BattleSide::Attacker)),
    }
}

#[test]
fn every_part_of_the_game_maps_back_and_round_trips() {
    let months = [
        MonthKind::Normal,
        MonthKind::Creature(Handle::from_raw(13)),
        MonthKind::Plague,
    ];
    let cases = every_decision()
        .into_iter()
        .map(Some)
        .chain([None])
        .zip(months.into_iter().cycle());
    for (decision, month) in cases {
        let o = every_game_part(decision, month);
        let wire = observation(&o);
        assert_eq!(inverse_game(&wire, o.battle.clone()), o);
        round_trips(&wire);
    }
    // Without a battle.
    let o = h3_game::GameObservation {
        battle: None,
        ..every_game_part(None, MonthKind::Normal)
    };
    assert_eq!(inverse_game(&observation(&o), None), o);
}

#[test]
fn the_games_json_is_serdes_default_form() {
    assert_eq!(text(&Month::Plague {}), r#"{"Plague":{}}"#);
    assert_eq!(
        text(&Decision::Meeting { heroes: [4, 7] }),
        r#"{"Meeting":{"heroes":[4,7]}}"#
    );
    assert_eq!(text(&OfferKind::Pass {}), r#"{"Pass":{}}"#);
    assert_eq!(text(&SlotState::NoRoom), r#""NoRoom""#);
    assert_eq!(
        text(&HireTarget::Hero { hero: 4 }),
        r#"{"Hero":{"hero":4}}"#
    );
}
