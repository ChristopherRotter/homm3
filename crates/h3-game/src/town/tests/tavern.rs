//! Tests of the taverns (E09 T37a, design note `E09-design.md` §61 to §63, §68, §75): the draw
//! (TWN-TAV-001), the start's and the week's fills with the release (TWN-TAV-002), the tavern
//! heroes over a turn (TWN-TAV-005), the records and away records (TWN-SET-005), the views and
//! their events (decisions 134, 135).

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FactionId, HeroClassId, HeroTypeId};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{ArtifactInstance, ArtifactPlace, Position, max_mana};
use h3_kernel::{Rng, StreamId};

use super::command::{BLUE, RED, end_turn};
use super::growth::{to_day, to_day_told};
use super::{faction, hero, rules_of, town_pack, towns_core, towns_setup};
use crate::tests::key;
use crate::town::pool::{HeroPool, HeroStatus};
use crate::town::tavern::{TAVERN_BACKPACK_SHOWN, draw, tell, views};
use crate::town::{FactionChoice, FactionSet};
use crate::{
    GameEvent, GameRuleset, GameSetup, GameState, TavernHeroView, TavernPart, Viewer, observe,
    project,
};

/// The town test rules with `t:town`'s tavern weights the knight 3 and the cleric 2.
fn tavern_rules() -> GameRuleset {
    tavern_rules_with(false)
}

/// [`tavern_rules`]; with `elf`, `t:other` has a town record too, with the same weights.
fn tavern_rules_with(elf: bool) -> GameRuleset {
    let mut towns = town_pack(elf);
    for record in &mut towns.towns {
        record.tavern = vec![(key("t:knight"), 3), (key("t:cleric"), 2)];
    }
    rules_of(towns_core(elf), &towns, elf, 4)
}

/// [`towns_setup`] whose map allows only the types of `keys`.
fn allowing(rules: &GameRuleset, keys: &[&str]) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.allowed_heroes = keys.iter().map(|k| hero(rules, k)).collect();
    setup
}

/// The class of `id`.
fn class_of(rules: &GameRuleset, id: HeroTypeId) -> HeroClassId {
    rules.core().hero_type(id).unwrap().class
}

/// `player`'s two slots' types.
fn slots(game: &GameState, player: PlayerId) -> [Option<HeroTypeId>; 2] {
    game.pool().tavern(player).unwrap().slots
}

/// The record of the type in `player`'s `slot`.
fn record(game: &GameState, player: PlayerId, slot: usize) -> HeroId {
    let pool = game.pool();
    pool.record(slots(game, player)[slot].unwrap()).unwrap()
}

// covers: TWN-TAV-001
#[test]
fn the_draw_weighs_the_classes_with_a_free_type_by_the_players_faction() {
    let rules = tavern_rules();
    let (core, towns) = (rules.core(), rules.towns());
    let castle = Some(faction(&rules, "t:town"));
    let other = Some(faction(&rules, "t:other"));
    let (knight, cleric) = (
        class_of(&rules, hero(&rules, "t:loner")),
        class_of(&rules, hero(&rules, "t:c0")),
    );
    let c0 = hero(&rules, "t:c0");
    let mut setup = towns_setup(&rules);
    // The loner is Red's colour's to take only (a disposed hero's mask).
    setup.hero_masks = vec![(hero(&rules, "t:loner"), 0b10)];
    let pool = HeroPool::new(core, &setup);
    let drawn = |pool: &HeroPool, seed, colour, faction, avoid, own| {
        let mut draws = Rng::for_stream(seed, StreamId::TAVERNS);
        draw(
            pool,
            (core, towns),
            (colour, faction),
            (avoid, own),
            &mut draws,
        )
    };
    let over = |colour, faction, avoid, own| -> BTreeSet<HeroTypeId> {
        (0..64)
            .filter_map(|seed| drawn(&pool, seed, colour, faction, avoid, own))
            .collect()
    };
    // Both classes come, each a free type the colour may take: never the placed Orrin, never the
    // loner for colour 0.
    let any = over(0, castle, None, false);
    let classes: BTreeSet<HeroClassId> = any.iter().map(|&t| class_of(&rules, t)).collect();
    assert_eq!(classes, [knight, cleric].into());
    assert!(
        any.iter()
            .all(|&t| pool.status(t) == Some(HeroStatus::Free))
    );
    assert!(!any.contains(&hero(&rules, "t:loner")));
    assert!(over(1, castle, None, false).contains(&hero(&rules, "t:loner")));
    // The class to avoid weighs 0 while another class has a count.
    assert_eq!(over(0, castle, Some(knight), false), [c0].into());
    // Own faction only: the castle's knights; the cleric's faction (no record: every class 1).
    let own = over(0, castle, None, true);
    assert!(own.iter().all(|&t| class_of(&rules, t) == knight));
    assert_eq!(over(0, other, None, true), [c0].into());
    // No faction: every class weighs 1, none is its own.
    let classes: BTreeSet<_> = (over(0, None, None, true).iter())
        .map(|&t| class_of(&rules, t))
        .collect();
    assert_eq!(classes, [knight, cleric].into());
    // Two draws, the class (1 to 5) then the type (1 to its count) in type order.
    let knights: Vec<HeroTypeId> = pool
        .free_for(core, 0)
        .filter(|&(_, ty)| ty.class == knight)
        .map(|(t, _)| t)
        .collect();
    for seed in 0..16 {
        let mut draws = Rng::for_stream(seed, StreamId::TAVERNS);
        let mut expected = draws.clone();
        let got = draw(&pool, (core, towns), (0, castle), (None, false), &mut draws);
        let class = expected.range_inclusive(1, 5);
        let count = if class <= 3 { knights.len() } else { 1 };
        let nth = expected.range_inclusive(1, u32::try_from(count).unwrap());
        let want = if class <= 3 {
            knights[usize::try_from(nth).unwrap() - 1]
        } else {
            c0
        };
        assert_eq!((got, &draws), (Some(want), &expected), "seed {seed}");
    }
    // A total of 0 (the town test rules weigh the cleric 0, the knight avoided) or no free type:
    // no type and no draw.
    let plain = super::towns_rules();
    let plain_pool = HeroPool::new(plain.core(), &towns_setup(&plain));
    let mut draws = Rng::for_stream(1, StreamId::TAVERNS);
    let untouched = draws.clone();
    let parts = (plain.core(), plain.towns());
    assert_eq!(
        draw(
            &plain_pool,
            parts,
            (0, castle),
            (Some(knight), false),
            &mut draws
        ),
        None
    );
    let mut none = towns_setup(&rules);
    none.allowed_heroes.clear();
    let empty = HeroPool::new(core, &none);
    assert_eq!(
        draw(
            &empty,
            (core, towns),
            (0, castle),
            (None, false),
            &mut draws
        ),
        None
    );
    assert_eq!(draws, untouched);
}

// covers: TWN-TAV-002, TWN-SET-005
#[test]
fn the_start_fills_each_players_slots_with_new_records() {
    let rules = tavern_rules();
    let core = rules.core();
    let setup = towns_setup(&rules);
    let game = GameState::new(&rules, &setup, 3).unwrap();
    let pool = game.pool();
    let castle = faction(&rules, "t:town");
    let knight = class_of(&rules, hero(&rules, "t:loner"));
    let c0 = hero(&rules, "t:c0");
    // Red: slot 0 a knight (its own faction's), slot 1 the cleric (the knight avoided). Blue
    // (its random choice drawn by the setup): slot 0 a knight, slot 1 a knight too, as the
    // cleric is offered (no other class has a free type).
    let [Some(r0), Some(r1)] = slots(&game, RED) else {
        panic!("Red's slots")
    };
    let [Some(b0), Some(b1)] = slots(&game, BLUE) else {
        panic!("Blue's slots")
    };
    assert_eq!((class_of(&rules, r0), r1), (knight, c0));
    assert_eq!(
        (class_of(&rules, b0), class_of(&rules, b1)),
        (knight, knight)
    );
    assert_eq!(BTreeSet::from([r0, r1, b0, b1]).len(), 4);
    for player in [RED, BLUE] {
        let tavern = pool.tavern(player).unwrap();
        assert_eq!(
            (tavern.faction, tavern.returning),
            (Some(castle), [false; 2])
        );
    }
    // Each offered, its record the next id after the setup's two heroes, in fill order, with
    // GAM-HER-001's defaults: level 1, 40 to 90 experience, its spell points full; its full
    // starting army in its away record.
    for (index, t) in [r0, r1, b0, b1].into_iter().enumerate() {
        assert_eq!(pool.status(t), Some(HeroStatus::Offered));
        let id = pool.record(t).unwrap();
        assert_eq!(usize::from(id.raw()), 2 + index);
        let details = game.hero(id).unwrap();
        assert_eq!((details.hero_type, details.level), (t, 1));
        assert!((40..=90).contains(&details.experience));
        assert_eq!(details.mana, max_mana(details, core));
        let ty = core.hero_type(t).unwrap();
        let away = pool.away(t).unwrap();
        // A type with more starting slots than an army's has none (the setup refuses it).
        let slots = usize::from(rules.adventure().params().army_slots());
        match ty.army.first().filter(|_| ty.army.len() <= slots) {
            None => assert!(away.army.is_empty()),
            Some(first) => {
                assert_eq!(away.army[0].creature, first.creature);
                let [low, high] = first.count;
                assert!((u32::from(low)..=u32::from(high)).contains(&away.army[0].count));
            }
        }
    }
    assert_eq!(game.heroes().len(), 6);
    // The placed heroes' types name their records (Orrin, hero 0: the first of the two).
    assert_eq!(pool.record(hero(&rules, "t:orrin")), Some(HeroId::new(0)));
    // The same seed, the same offers; the setup's own draws did not move (hero 1's army).
    let again = GameState::new(&rules, &setup, 3).unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
    let mut plain = setup.clone();
    plain.allowed_heroes.clear();
    let without = GameState::new(&rules, &plain, 3).unwrap();
    let army = |g: &GameState| g.adventure().hero(HeroId::new(1)).unwrap().army.clone();
    assert_eq!(army(&without), army(&game));
    assert_eq!(slots(&without, RED), [None; 2]);
}

// covers: TWN-TAV-002
#[test]
fn a_random_choice_no_start_reads_is_drawn_at_the_first_fill() {
    let rules = tavern_rules_with(true);
    let castle = faction(&rules, "t:town");
    let other = faction(&rules, "t:other");
    // No towns, no random hero: the setup draws no faction (TWN-SET-001); the fill does, from the
    // taverns' stream, before its slot 0.
    let mut setup = crate::tests::setup();
    setup.allowed_heroes = vec![hero(&rules, "t:gunner"), hero(&rules, "t:c0")];
    setup.players[0].start.factions = FactionSet::EMPTY.with(castle).with(other);
    setup.players[0].start.choice = Some(FactionChoice::Random);
    let mut factions = BTreeSet::new();
    for seed in 0..32 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        let tavern = *game.pool().tavern(RED).unwrap();
        let drawn: FactionId = tavern.faction.unwrap();
        factions.insert(drawn);
        // Slot 0 of its faction's classes: the gunner for the castle, the cleric for the other.
        let first = if drawn == castle { "t:gunner" } else { "t:c0" };
        assert_eq!(tavern.slots[0], Some(hero(&rules, first)));
        // Blue: no choice, no faction; every class weighs 1, so its slots took nothing left.
        assert_eq!(game.pool().tavern(BLUE).unwrap().faction, None);
    }
    assert_eq!(factions, [castle, other].into());
}

// covers: TWN-TAV-002, TWN-SET-005
#[test]
fn the_week_releases_and_refills_and_a_record_keeps_its_id() {
    let rules = tavern_rules();
    let setup = allowing(&rules, &["t:gunner", "t:c0", "t:crowd"]);
    let mut game = GameState::new(&rules, &setup, 5).unwrap();
    let gunner = hero(&rules, "t:gunner");
    let (crowd, c0) = (hero(&rules, "t:crowd"), hero(&rules, "t:c0"));
    let red = slots(&game, RED);
    assert_eq!(red[1], Some(c0));
    let ids: Vec<HeroId> = [gunner, crowd, c0]
        .iter()
        .filter_map(|&t| game.pool().record(t))
        .collect();
    assert_eq!(ids.len(), 3, "every type drawn once at the start");
    let before = game.heroes().len();
    // The week: every slot released, then refilled from the same three types, each with its id.
    to_day(&rules, &mut game, 8);
    let pool = game.pool();
    let mut offered: Vec<HeroTypeId> = [RED, BLUE]
        .iter()
        .flat_map(|&p| slots(&game, p))
        .flatten()
        .collect();
    offered.sort_unstable();
    let mut all = vec![gunner, crowd, c0];
    all.sort_unstable();
    assert_eq!(offered, all);
    assert_eq!(game.heroes().len(), before);
    for (&t, &id) in [gunner, crowd, c0].iter().zip(&ids) {
        assert_eq!(pool.record(t), Some(id));
        assert_eq!(pool.status(t), Some(HeroStatus::Offered));
    }
    // A released type nobody draws again is free with its record and away record kept: one
    // type more than the slots' four can hold.
    let setup = allowing(
        &rules,
        &["t:gunner", "t:c0", "t:crowd", "t:sage", "t:loner"],
    );
    let mut game = GameState::new(&rules, &setup, 5).unwrap();
    to_day(&rules, &mut game, 8);
    let pool = game.pool();
    let free: Vec<HeroTypeId> = ["t:gunner", "t:c0", "t:crowd", "t:sage", "t:loner"]
        .iter()
        .map(|k| hero(&rules, k))
        .filter(|&t| pool.status(t) == Some(HeroStatus::Free))
        .collect();
    assert_eq!(free.len(), 1);
    let left = free[0];
    if let Some(id) = pool.record(left) {
        assert!(pool.away(left).is_some(), "{id:?}'s away record kept");
    }
}

// covers: TWN-TAV-002, TWN-TAV-005
#[test]
fn a_returning_hero_stays_through_the_weeks_release_and_the_marks_clear() {
    let rules = tavern_rules();
    let setup = towns_setup(&rules);
    let mut game = GameState::new(&rules, &setup, 9).unwrap();
    to_day(&rules, &mut game, 7);
    let kept = slots(&game, RED)[1];
    let army = game.pool().away(kept.unwrap()).unwrap().army.clone();
    // Red's slot 1 came back after Red's turn ended (another player's attack, TD-18): the week's
    // release spares it, the fill leaves it as it was; the day's start clears the mark after.
    end_turn(&rules, &mut game, RED);
    game.set_returning(RED, 1);
    end_turn(&rules, &mut game, BLUE);
    assert_eq!(game.day(), 8);
    assert_eq!(slots(&game, RED)[1], kept);
    assert_eq!(game.pool().away(kept.unwrap()).unwrap().army, army);
    assert_eq!(game.pool().tavern(RED).unwrap().returning, [false; 2]);
    // A mark its owner's turn end clears (a return in its own turn): the next week releases it.
    game.set_returning(RED, 0);
    end_turn(&rules, &mut game, RED);
    assert_eq!(game.pool().tavern(RED).unwrap().returning, [false; 2]);
    // A turn's end clears its player's marks only; the marks are in the state's hash.
    game.set_returning(BLUE, 1);
    let mut marked = game.clone();
    marked.set_returning(BLUE, 0);
    assert_ne!(marked.state_hash(), game.state_hash());
    assert_eq!(game.pool().tavern(BLUE).unwrap().returning, [false, true]);
    end_turn(&rules, &mut game, BLUE);
    assert_eq!(game.pool().tavern(BLUE).unwrap().returning, [false; 2]);
}

// covers: TWN-TAV-005
#[test]
fn a_tavern_heros_points_are_set_at_its_owners_turn_start() {
    let rules = tavern_rules();
    let (core, part) = (rules.core(), rules.adventure());
    let setup = allowing(&rules, &["t:gunner", "t:c0", "t:crowd", "t:sage"]);
    let mut game = GameState::new(&rules, &setup, 2).unwrap();
    let points = |game: &GameState, player: PlayerId, slot: usize| {
        let t = slots(game, player)[slot].unwrap();
        let away = game.pool().away(t).unwrap();
        (away.movement, away.max_movement)
    };
    let expected = |game: &GameState, player: PlayerId, slot: usize| {
        let t = slots(game, player)[slot].unwrap();
        let details = game.hero(game.pool().record(t).unwrap());
        let army = &game.pool().away(t).unwrap().army;
        let p = h3_adventure::day_points(core, part, details, army, 0);
        (p, p)
    };
    // Red's turn started in the build: its offers have their points; Blue's none yet.
    for slot in 0..2 {
        assert_eq!(points(&game, RED, slot), expected(&game, RED, slot));
        assert_eq!(points(&game, BLUE, slot), (0, 0));
    }
    assert!(points(&game, RED, 0).0 > 0);
    end_turn(&rules, &mut game, RED);
    for slot in 0..2 {
        assert_eq!(points(&game, BLUE, slot), expected(&game, BLUE, slot));
    }
    // A turn start that changes nothing writes nothing: a fork keeps sharing the pool.
    let mut fork = game.clone();
    end_turn(&rules, &mut fork, BLUE);
    assert!(fork.shares_pool(&game));
}

// covers: TWN-TAV-002
#[test]
fn a_fill_equips_the_backpack_and_fills_the_spell_points() {
    let rules = tavern_rules();
    let core = rules.core();
    let setup = allowing(&rules, &["t:sage", "t:c0"]);
    let mut game = GameState::new(&rules, &setup, 4).unwrap();
    let sage = hero(&rules, "t:sage");
    assert_eq!(slots(&game, RED)[0], Some(sage));
    let id = game.pool().record(sage).unwrap();
    // Before the week: two rings in the backpack (the test core's ring slot has two positions),
    // the spell points spent.
    let ring = core
        .artifacts()
        .iter()
        .position(|a| {
            a.slot
                .is_some_and(|s| core.artifact_slot(s).is_some_and(|s| s.count == 2))
        })
        .and_then(h3_content::ArtifactTypeId::from_index)
        .map(ArtifactInstance::from)
        .unwrap();
    game.set_backpack(id, vec![ring, ring, ring]);
    game.set_mana(id, 0);
    to_day(&rules, &mut game, 8);
    assert_eq!(slots(&game, RED)[0], Some(sage));
    let details = game.hero(id).unwrap();
    let worn = details
        .worn
        .iter()
        .flatten()
        .filter(|&&a| a == ring)
        .count();
    assert_eq!((worn, details.backpack.as_slice()), (2, &[ring][..]));
    assert_eq!(details.mana, max_mana(details, core));
}

// covers: TWN-TAV-002
#[test]
fn the_owner_sees_its_tavern_heroes_and_is_told_each_change() {
    let rules = tavern_rules();
    let setup = towns_setup(&rules);
    let mut game = GameState::new(&rules, &setup, 6).unwrap();
    // The first observation holds the start's offers, as the screen shows them.
    let seen = observe(&game, &rules, Viewer::Player(RED)).tavern;
    assert_eq!(seen, views(&game, RED));
    let t = slots(&game, RED)[0].unwrap();
    let view = seen[0].clone().unwrap();
    let details = game.hero(game.pool().record(t).unwrap()).unwrap();
    assert_eq!((view.hero.hero_type, view.hero.level), (t, 1));
    assert_eq!(view.army, game.pool().away(t).unwrap().army);
    assert_eq!(view.skills, details.skills);
    assert_ne!(seen, observe(&game, &rules, Viewer::Player(BLUE)).tavern);
    // The week tells each changed slot, its parts after it, to its owner alone; folding them
    // gives the views after (but the spell points a new day regains, untold).
    let before: Vec<_> = [RED, BLUE]
        .map(|p| observe(&game, &rules, Viewer::Player(p)).tavern)
        .into();
    let events = to_day_told(&rules, &mut game, 8);
    for (player, before) in [RED, BLUE].into_iter().zip(before) {
        let mut folded = before;
        let told: Vec<GameEvent> = events
            .iter()
            .filter_map(|e| project(e, Viewer::Player(player)))
            .collect();
        let mut count = 0;
        for event in &told {
            match *event {
                GameEvent::TavernOffered {
                    player: p,
                    slot,
                    hero,
                } => {
                    assert_eq!(p, player);
                    count += 1;
                    folded[usize::from(slot)] = hero.map(|hero| TavernHeroView {
                        hero,
                        army: Vec::new(),
                        skills: Vec::new(),
                        artifacts: Vec::new(),
                    });
                }
                GameEvent::TavernPartSeen { slot, part, .. } => {
                    let view = folded[usize::from(slot)].as_mut().unwrap();
                    match part {
                        TavernPart::Stack(stack) => view.army.push(stack),
                        TavernPart::Skill { skill, level } => view.skills.push((skill, level)),
                        TavernPart::Artifact { place, artifact } => {
                            view.artifacts.push((place, artifact));
                        }
                    }
                }
                _ => {}
            }
        }
        assert!(count > 0, "the week changed {player:?}'s slots");
        assert_eq!(folded, views(&game, player));
        // Another player's are never told to it.
        let other = if player == RED { BLUE } else { RED };
        assert!(told.iter().all(|e| !matches!(
            e,
            GameEvent::TavernOffered { player: p, .. } | GameEvent::TavernPartSeen { player: p, .. }
                if *p == other
        )));
    }
    // An unchanged slot is not told; an emptied one is, with no hero.
    let now = vec![views(&game, RED), views(&game, BLUE)];
    let mut events = Vec::new();
    tell(&game, &now, &mut events);
    assert!(events.is_empty());
    let mut plain = setup.clone();
    plain.allowed_heroes.clear();
    let bare = GameState::new(&rules, &plain, 6).unwrap();
    tell(&bare, &now, &mut events);
    let emptied = GameEvent::TavernOffered {
        player: RED,
        slot: 0,
        hero: None,
    };
    assert_eq!((events.first(), events.len()), (Some(&emptied), 4));
}

// covers: TWN-TAV-002
#[test]
fn the_view_shows_the_backpacks_first_places_and_its_count() {
    let rules = tavern_rules();
    let core = rules.core();
    let setup = allowing(&rules, &["t:sage", "t:c0"]);
    let mut game = GameState::new(&rules, &setup, 4).unwrap();
    let id = record(&game, RED, 0);
    // Seven artifacts no free position takes (the spell book, which the sage wears already).
    let book = core
        .artifacts()
        .iter()
        .position(|a| a.effects.contains(&h3_content::HeroEffect::SpellBook))
        .and_then(h3_content::ArtifactTypeId::from_index)
        .map(ArtifactInstance::from)
        .unwrap();
    game.set_backpack(id, vec![book; 7]);
    let view = views(&game, RED)[0].clone().unwrap();
    assert_eq!(view.hero.carried, 7);
    let carried: Vec<ArtifactPlace> = view
        .artifacts
        .iter()
        .filter(|(place, _)| matches!(place, ArtifactPlace::Backpack(_)))
        .map(|&(place, _)| place)
        .collect();
    let first: Vec<ArtifactPlace> = (0..TAVERN_BACKPACK_SHOWN)
        .map(ArtifactPlace::Backpack)
        .collect();
    assert_eq!(carried, first);
    // The worn ones first, by position.
    assert!(matches!(
        view.artifacts[0].0,
        ArtifactPlace::Worn(Position(_))
    ));
}

// covers: TWN-TAV-002
#[test]
fn a_forks_week_is_its_parents() {
    let rules = tavern_rules();
    let setup = towns_setup(&rules);
    let mut game = GameState::new(&rules, &setup, 8).unwrap();
    to_day(&rules, &mut game, 6);
    let mut fork = game.clone();
    assert!(fork.shares_pool(&game));
    to_day(&rules, &mut game, 15);
    to_day(&rules, &mut fork, 15);
    assert_eq!(fork.state_hash(), game.state_hash());
}
