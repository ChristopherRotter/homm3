//! What a viewer may learn on imported maps (E08 T8b2b; `docs/05-testing.md`, "Property tests";
//! `docs/06-security.md`, "Information hiding"), on T8b2a's playable maps and driver
//! ([`super::play`]): observation hygiene (what a viewer is shown or told does not depend on what
//! it cannot see) and the observation's consistency with the events it is told.

use h3_adventure::{AdventureEvent, AdventureEventKind, Pos as TilePos, TileView, Viewers};
use h3_contracts::{HeroId, PlayerId};
use h3_game::{
    Actor, GameEvent, GameObservation, GameRuleset, GameState, Viewer, apply, legal_commands,
    legal_moves, observe, project,
};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use super::import::object;
use super::play::{
    BLOCKERS, FAITHFUL, HERO_TEMPLATE, STRICT, Slots, Spot, army, lay_out, options, plan, resolve,
    slots, start_on, step, tile,
};
use crate::model::{Body, Object, Pos, RawMap, Stack};

/// What the hygiene test changes of a game where the viewer cannot see it, drawn as the plan is.
#[derive(Debug, Clone)]
struct Unseen {
    /// Another seed (none: the same).
    seed: Option<u64>,
    /// The army each of the other players' heroes gets instead, in the map's order (none: kept).
    armies: Vec<Option<Slots>>,
    /// Terrain, road and river codes for the tiles the viewer never sees, as many as the largest
    /// map has (none: kept).
    tiles: Vec<Option<(u8, u8, u8)>>,
    /// Which of the blockers the viewer never sees are removed (modulo their number).
    removed: Vec<bool>,
    /// Blockers added where the viewer never sees them: anchor (modulo the map) and template.
    added: Vec<(Spot, usize)>,
    /// Another tile (modulo the map) for each of the other players' heroes, in the map's order
    /// (none: kept): taken where the viewer sees the hero on neither tile, the new one land and
    /// free of heroes and blockers.
    moved: Vec<Option<Spot>>,
}

/// Each kind of change in about half the cases, so that some cases keep the other players'
/// commands' outcomes the same for long (another army or terrain changes a hero's points).
fn unseen() -> impl Strategy<Value = Unseen> {
    let tiles = 2 * 16 * 16;
    (
        proptest::option::of(any::<u64>()),
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::weighted(0.5, army()), 1..=6)
        ],
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::weighted(0.5, tile()), tiles)
        ],
        prop_oneof![Just(Vec::new()), vec(any::<bool>(), 1..=8)],
        vec((any::<Spot>(), 0..BLOCKERS.len()), 0..=8),
        prop_oneof![
            Just(Vec::new()),
            vec(proptest::option::of(any::<Spot>()), 1..=6)
        ],
    )
        .prop_map(|(seed, armies, tiles, removed, added, moved)| Unseen {
            seed,
            armies,
            tiles,
            removed,
            added,
            moved,
        })
}

/// An army of its own for the viewer's heroes: at least one stack (a hero whose slots are all
/// empty gets its type's starting army, drawn from the seed).
fn own_army() -> impl Strategy<Value = Vec<Option<(u16, u16)>>> {
    slots().prop_filter("a stack at least", |slots| {
        slots.iter().any(Option::is_some)
    })
}

/// The tiles a template anchored at `anchor` covers on the map (blocked and visitable).
fn covers(anchor: Spot, template: usize, side: u8) -> impl Iterator<Item = Spot> {
    let (ax, ay, az) = anchor;
    let (blocked, visitable) = BLOCKERS[template];
    blocked
        .iter()
        .chain(visitable)
        .filter_map(move |&(column, row)| {
            let x = ax.checked_sub(column).filter(|&x| x < side)?;
            let y = ay.checked_sub(row).filter(|&y| y < side)?;
            Some((x, y, az))
        })
}

/// The heroes of `m`, in the map's order: each one's owner (a colour) and army.
fn heroes(m: &mut RawMap) -> impl Iterator<Item = (u8, &mut Option<Vec<Option<Stack>>>)> {
    m.objects.iter_mut().filter_map(|o| match &mut o.body {
        Body::Hero(hero) => Some((hero.owner, &mut hero.army)),
        _ => None,
    })
}

/// The tile a hero object stands on: one column left of its anchor.
fn standing_on(o: &Object) -> Spot {
    (o.pos.x - 1, o.pos.y, o.pos.z)
}

fn stacks(slots: &[Option<(u16, u16)>]) -> Vec<Option<Stack>> {
    slots
        .iter()
        .map(|slot| slot.map(|(creature, count)| Stack { creature, count }))
        .collect()
}

/// `m` with `unseen`'s changes where the viewer (`colour`) never sees them: `ground` says whether
/// it never sees a tile's terrain, road and river, `objects` whether it never sees what objects
/// cover there and the heroes standing there. A hero's tile stays land, and no blocker is added
/// over it.
fn vary(
    m: &RawMap,
    colour: u8,
    unseen: &Unseen,
    ground: impl Fn(Spot) -> bool,
    objects: impl Fn(Spot) -> bool,
) -> RawMap {
    let mut m = m.clone();
    let side = u8::try_from(m.terrain.size).unwrap();
    let levels = u8::try_from(m.terrain.levels).unwrap();
    let mut standing: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| o.template == HERO_TEMPLATE)
        .map(standing_on)
        .collect();
    let spots =
        (0..levels).flat_map(|z| (0..side).flat_map(move |y| (0..side).map(move |x| (x, y, z))));
    for ((spot, tile), new) in spots.zip(&mut m.terrain.tiles).zip(&unseen.tiles) {
        if let Some((terrain, road, river)) = new.filter(|_| ground(spot)) {
            tile.terrain = if terrain >= 8 && standing.contains(&spot) {
                2
            } else {
                terrain
            };
            tile.road = road;
            tile.river = river;
        }
    }
    let unseen_blocker =
        |anchor: Spot, template: usize| covers(anchor, template, side).all(&objects);
    let mut blocker = 0;
    m.objects.retain(|o| {
        if o.template == HERO_TEMPLATE {
            return true;
        }
        let template = usize::try_from(o.template - 1).unwrap();
        let index = blocker;
        blocker += 1;
        let removed = unseen.removed.get(index % unseen.removed.len().max(1)) == Some(&true);
        !(removed && unseen_blocker((o.pos.x, o.pos.y, o.pos.z), template))
    });
    for &((x, y, z), template) in &unseen.added {
        let anchor = (x % side, y % side, z % levels);
        if unseen_blocker(anchor, template)
            && covers(anchor, template, side).all(|spot| !standing.contains(&spot))
        {
            let mut o = object(
                anchor.0,
                anchor.1,
                1 + u32::try_from(template).unwrap(),
                Body::None,
            );
            o.pos.z = anchor.2;
            m.objects.push(o);
        }
    }
    let blocked: Vec<Spot> = m
        .objects
        .iter()
        .filter(|o| o.template != HERO_TEMPLATE)
        .flat_map(|o| {
            let template = usize::try_from(o.template - 1).unwrap();
            covers((o.pos.x, o.pos.y, o.pos.z), template, side)
        })
        .collect();
    let mut other = 0;
    for o in &mut m.objects {
        let Body::Hero(hero) = &mut o.body else {
            continue;
        };
        if hero.owner == colour {
            continue;
        }
        let index = other;
        other += 1;
        if let Some(Some(slots)) = unseen.armies.get(index % unseen.armies.len().max(1)) {
            hero.army = slots.as_deref().map(stacks);
        }
        if let Some(Some((x, y, z))) = unseen.moved.get(index % unseen.moved.len().max(1)) {
            let (from, to) = (standing_on(o), (x % side, y % side, z % levels));
            let at = (usize::from(to.2) * usize::from(side) + usize::from(to.1))
                * usize::from(side)
                + usize::from(to.0);
            if objects(from)
                && objects(to)
                && m.terrain.tiles[at].terrain < 8
                && !blocked.contains(&to)
                && !standing.contains(&to)
            {
                o.pos = Pos {
                    x: to.0 + 1,
                    y: to.1,
                    z: to.2,
                };
                standing.retain(|&spot| spot != from);
                standing.push(to);
            }
        }
    }
    m
}

/// The viewer sees the same of `a` and `b`: its observation, its legal commands and the legal
/// moves of every hero it sees.
fn check_same(
    a: &GameState,
    b: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
) -> Result<(), TestCaseError> {
    let seen = observe(a, rules, Viewer::Player(viewer));
    prop_assert_eq!(&seen, &observe(b, rules, Viewer::Player(viewer)));
    let actor = Actor::Player(viewer);
    prop_assert_eq!(
        legal_commands(a, rules, actor),
        legal_commands(b, rules, actor)
    );
    for hero in &seen.adventure.heroes {
        prop_assert_eq!(
            legal_moves(a, rules, actor, hero.id),
            legal_moves(b, rules, actor, hero.id)
        );
    }
    Ok(())
}

/// The events as `viewer` is told them.
fn told(events: &[GameEvent], viewer: PlayerId) -> Vec<GameEvent> {
    events
        .iter()
        .filter_map(|event| project(event, Viewer::Player(viewer)))
        .collect()
}

/// Whether the viewer sees what stands on the tile at `pos`: objects and heroes (ADV-FOG-004,
/// ADV-FOG-005).
fn sees(view: &GameObservation, pos: TilePos) -> bool {
    matches!(view.adventure.tile(pos), Some(TileView::Seen(_)))
}

/// How much the viewer sees of a tile: nothing, its ground, everything.
fn depth(view: &TileView) -> u8 {
    match view {
        TileView::Hidden => 0,
        TileView::Ground { .. } => 1,
        TileView::Seen(_) => 2,
    }
}

/// The observation's consistency with the events: `after` is `before` with what `viewer` was
/// `told` applied as a client would. The day and whose turn it is follow the game's events; the
/// heroes it sees move as the projected steps say (a step names the viewer alone and tells at
/// least one tile; a step's tile it leaves is where the viewer last saw the hero, a step it is
/// not told the tile entered takes the hero out of its sight); a hero shows without a step only
/// on a tile the viewer did not see before. The viewer forgets nothing: a tile's view only
/// deepens, and keeps what it showed. Its planes change only by its own command (`own`).
fn check_told(
    viewer: PlayerId,
    (before, after): (&GameObservation, &GameObservation),
    told: &[GameEvent],
    own: bool,
) -> Result<(), TestCaseError> {
    let mut day = before.day;
    let mut acting: Vec<bool> = before.players.iter().map(|p| p.acting).collect();
    let mut heroes: Vec<(HeroId, TilePos)> = before
        .adventure
        .heroes
        .iter()
        .map(|h| (h.id, h.pos))
        .collect();
    let mut stepped = Vec::new();
    for event in told {
        match *event {
            GameEvent::DayStarted { day: started } => day = started,
            GameEvent::TurnEnded { player } => acting[usize::from(player.raw())] = false,
            GameEvent::TurnStarted { player } => acting[usize::from(player.raw())] = true,
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::HeroStepped {
                        hero,
                        from,
                        to,
                        seen_from,
                        seen_to,
                    },
                seen_by,
            }) => {
                let alone = Viewers::NONE.with(viewer);
                prop_assert_eq!(seen_by, alone);
                prop_assert_eq!(
                    (seen_from, seen_to),
                    (
                        if from.is_some() { alone } else { Viewers::NONE },
                        if to.is_some() { alone } else { Viewers::NONE },
                    )
                );
                prop_assert!(from.is_some() || to.is_some(), "{:?} tells no tile", event);
                let at = heroes.iter().position(|&(id, _)| id == hero);
                prop_assert_eq!(at.map(|i| heroes[i].1), from, "{:?}", event);
                match (at, to) {
                    (Some(i), Some(to)) => heroes[i].1 = to,
                    (Some(i), None) => {
                        heroes.remove(i);
                    }
                    (None, to) => heroes.push((hero, to.unwrap())),
                }
                stepped.push(hero);
            }
        }
    }
    prop_assert_eq!(after.day, day);
    let now: Vec<bool> = after.players.iter().map(|p| p.acting).collect();
    prop_assert_eq!(now, acting);
    for &(id, pos) in &heroes {
        prop_assert_eq!(after.adventure.hero(id).map(|h| h.pos), Some(pos));
    }
    for hero in &after.adventure.heroes {
        if !heroes.iter().any(|&(id, _)| id == hero.id) {
            prop_assert!(
                own && !stepped.contains(&hero.id),
                "{:?} shows untold",
                hero
            );
            prop_assert!(!sees(before, hero.pos) && sees(after, hero.pos));
        }
    }
    for (was, is) in before.adventure.tiles.iter().zip(&after.adventure.tiles) {
        prop_assert!(depth(was) <= depth(is));
        if depth(was) == depth(is) {
            prop_assert_eq!(was, is);
        }
    }
    if !own {
        prop_assert_eq!(&before.adventure.explored, &after.adventure.explored);
        prop_assert_eq!(&before.adventure.fringe, &after.adventure.fringe);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// Observation hygiene: two games that differ only in what a viewer never sees (the seed,
    /// the other players' armies and so their points, the tiles it never sees and what objects
    /// cover there) show it the same observation, legal commands and moves after the same
    /// commands, accept or reject its own commands alike and tell it the same events, as long as
    /// the other players' commands are accepted in both or in neither (a move that one hero can
    /// pay and the other cannot ends where the viewer may see it).
    #[test]
    fn a_viewer_learns_nothing_it_cannot_see(
        plan in plan(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in vec(step(), 0..40),
        viewer in any::<u8>(),
        own in own_army(),
        unseen in unseen(),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut m = lay_out(&plan);
        let game = start_on(&m, rules, seed);
        let viewer = PlayerId::new(viewer % u8::try_from(game.players()).unwrap());
        let colour = game.colour(viewer).unwrap();
        // The viewer's heroes get armies of their own: a starting army is drawn from the seed (also
        // for a hero whose slots are all empty).
        for (owner, army) in heroes(&mut m) {
            if owner == colour && army.iter().flatten().all(Option::is_none) {
                *army = Some(stacks(&own));
            }
        }
        // What the viewer never sees, from the tiles it has explored at the end of the game (no
        // rule hides tiles) by the rules, not by the observation under test: a tile neither
        // explored nor next to an explored one shows nothing; one next to one (the fringe) its
        // ground, and what objects cover there only under ADV-QUIRK-1 (ADV-FOG-005).
        let mut end = start_on(&m, rules, seed);
        for step in &steps {
            let legal = options(&end, rules);
            let (actor, command) = resolve(&end, &legal, step);
            let _ = apply(&mut end, rules, Actor::Player(actor), command, &mut Vec::new());
        }
        let explored = |x: i16, y: i16, z: u8| {
            u16::try_from(x)
                .ok()
                .zip(u16::try_from(y).ok())
                .is_some_and(|(x, y)| end.adventure().is_explored(viewer, TilePos::new(x, y, z)))
        };
        let near = |(x, y, z): Spot| {
            let (x, y) = (i16::from(x), i16::from(y));
            (-1..=1).any(|dy| (-1..=1).any(|dx| explored(x + dx, y + dy, z)))
        };
        let other = vary(
            &m,
            colour,
            &unseen,
            |spot| !near(spot),
            |(x, y, z)| !explored(x.into(), y.into(), z) && (strict || !near((x, y, z))),
        );
        let mut a = start_on(&m, rules, seed);
        let mut b = start_on(&other, rules, unseen.seed.unwrap_or(seed));
        check_same(&a, &b, rules, viewer)?;
        for step in &steps {
            let legal = options(&a, rules);
            let (actor, command) = resolve(&a, &legal, step);
            let (mut ea, mut eb) = (Vec::new(), Vec::new());
            let ra = apply(&mut a, rules, Actor::Player(actor), command.clone(), &mut ea);
            let rb = apply(&mut b, rules, Actor::Player(actor), command, &mut eb);
            if actor == viewer {
                prop_assert_eq!(ra, rb);
            } else if ra.is_ok() != rb.is_ok() {
                break;
            }
            prop_assert_eq!(told(&ea, viewer), told(&eb, viewer));
            check_same(&a, &b, rules, viewer)?;
        }
    }

    /// The observation's consistency with the events, for every player at every step of random
    /// play on imported maps, in both profiles ([`check_told`]).
    #[test]
    fn views_move_as_the_events_say(
        plan in plan(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in vec(step(), 0..40),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start_on(&lay_out(&plan), rules, seed);
        let players: Vec<PlayerId> =
            (0..u8::try_from(state.players()).unwrap()).map(PlayerId::new).collect();
        let mut views: Vec<GameObservation> = players
            .iter()
            .map(|&p| observe(&state, rules, Viewer::Player(p)))
            .collect();
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, &legal, step);
            let mut events = Vec::new();
            let result = apply(&mut state, rules, Actor::Player(actor), command, &mut events);
            for (&player, view) in players.iter().zip(&mut views) {
                let after = observe(&state, rules, Viewer::Player(player));
                let own = player == actor && result.is_ok();
                check_told(player, (view, &after), &told(&events, player), own)?;
                *view = after;
            }
        }
    }
}
