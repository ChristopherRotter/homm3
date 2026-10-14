//! What a viewer may learn on imported maps (E08 T8b2b; `docs/05-testing.md`, "Property tests";
//! `docs/06-security.md`, "Information hiding"), on T8b2a's playable maps and driver
//! ([`super::play`]): observation hygiene (what a viewer is shown or told does not depend on what
//! it cannot see) and the observation's consistency with the events it is told.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, ObjectView, Offer, OfferKind, Pos as TilePos, TileMask,
    TileView, Viewers,
};
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
    let (blocked, visitable) = BLOCKERS[template % BLOCKERS.len()];
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
/// `told` applied as a client would. The day, the acting players and the turn group follow the
/// game's events; the
/// heroes it sees move as the projected steps say (a step names the viewer alone and tells at
/// least one tile; a step's tile it leaves is where the viewer last saw the hero, a step it is
/// not told the tile entered takes the hero out of its sight); a hero shows without a step only
/// on a tile the viewer did not see before. The viewer forgets nothing: a tile's view only
/// deepens, and keeps what it showed; an object it saw leaves its view only as an object removal
/// it is told (E08 T13b), and its treasury changes only by the gains and payments it is told,
/// which are its own. Its open decision and its purchases change only as its offers' events say:
/// an offer it is told opened is its decision, an answer closes it, an accepted Cartographer's
/// adds the type to its purchases (E08 T14b); a reveal it is told explores at least that many
/// more tiles; its keys change only by the keys it is told it gained (T15a). Its planes change
/// only by its own command (`own`).
fn check_told(
    viewer: PlayerId,
    (before, after): (&GameObservation, &GameObservation),
    told: &[GameEvent],
    own: bool,
) -> Result<(), TestCaseError> {
    let mut day = before.day;
    let mut acting: Vec<bool> = before.players.iter().map(|p| p.acting).collect();
    // The turn group: the players whose turns start after the last turn that ended, if any did.
    let mut group = before.turn_group.clone();
    let mut started = false;
    let mut heroes: Vec<(HeroId, TilePos)> = before
        .adventure
        .heroes
        .iter()
        .map(|h| (h.id, h.pos))
        .collect();
    let mut stepped = Vec::new();
    let mut removed = Vec::new();
    let mut treasury = before.treasury.clone();
    let named = |o: &Offer| (o.hero, o.object_type, o.anchor);
    let mut decision = before.decision.as_ref().map(named);
    let mut bought = before.adventure.bought.clone();
    let mut keys = before.adventure.keys;
    let mut revealed = 0;
    // E08 T19a: whether the viewer is in a battle, by its start and end.
    let mut fighting = before.battle.is_some();
    let alone = Viewers::NONE.with(viewer);
    for event in told {
        match *event {
            GameEvent::BattleStarted { seen_by, .. } => {
                prop_assert_eq!(seen_by, alone);
                prop_assert!(!fighting);
                fighting = true;
            }
            GameEvent::Battle { seen_by, .. } => {
                prop_assert_eq!(seen_by, alone);
                prop_assert!(fighting);
            }
            GameEvent::BattleEnded { seen_by, .. } => {
                prop_assert_eq!(seen_by, alone);
                prop_assert!(fighting);
                fighting = false;
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::HeroRemoved { hero, pos },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, alone);
                let at = heroes.iter().position(|&(id, _)| id == hero);
                prop_assert_eq!(at.map(|i| heroes[i].1), Some(pos), "{:?}", event);
                heroes.remove(at.unwrap());
            }
            GameEvent::OfferOpened {
                player,
                hero,
                object_type,
                anchor,
            } => {
                prop_assert_eq!(player, viewer);
                prop_assert!(decision.is_none());
                decision = Some((hero, object_type, anchor));
            }
            GameEvent::OfferRefused { player, .. } => prop_assert_eq!(player, viewer),
            GameEvent::OfferAnswered {
                player,
                object_type,
                anchor,
                accepted,
            } => {
                prop_assert_eq!(player, viewer);
                let (_, ty, at) = decision.take().unwrap();
                prop_assert_eq!((ty, at), (object_type, anchor));
                // The offer was open before the answer: the observation showed its kind.
                let kind = before.decision.as_ref().map(|o| &o.kind);
                if accepted && matches!(kind, Some(OfferKind::Reveal { .. })) {
                    bought.push(object_type);
                    bought.sort();
                }
            }
            GameEvent::ResourcePaid {
                player,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, viewer);
                treasury[usize::from(resource.raw())] -= amount;
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::TilesRevealed { count, .. },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(viewer));
                revealed += count;
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::KeyGained { key },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(viewer));
                keys |= 1 << key;
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectRemoved {
                        object_type,
                        anchor,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(viewer));
                removed.push((object_type, anchor));
            }
            GameEvent::ResourceGained {
                player,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, viewer);
                treasury[usize::from(resource.raw())] += amount;
            }
            GameEvent::DayStarted { day: started } => day = started,
            GameEvent::TurnEnded { player } => {
                acting[usize::from(player.raw())] = false;
                started = false;
            }
            GameEvent::TurnStarted { player } => {
                acting[usize::from(player.raw())] = true;
                if !started {
                    group.clear();
                    started = true;
                }
                group.push(player);
            }
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
    prop_assert_eq!(after.battle.is_some(), fighting);
    prop_assert_eq!(after.day, day);
    let now: Vec<bool> = after.players.iter().map(|p| p.acting).collect();
    prop_assert_eq!(now, acting);
    prop_assert_eq!(&after.turn_group, &group);
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
    prop_assert_eq!(&after.treasury, &treasury);
    prop_assert_eq!(after.decision.as_ref().map(named), decision);
    prop_assert_eq!(&after.adventure.bought, &bought);
    prop_assert_eq!(after.adventure.keys, keys);
    let ones = |planes: &[u64]| planes.iter().map(|w| w.count_ones()).sum::<u32>();
    prop_assert!(ones(&after.adventure.explored) >= ones(&before.adventure.explored) + revealed);
    let named = |o: &ObjectView| (o.object_type, o.anchor);
    for object in &after.adventure.objects {
        prop_assert!(!removed.contains(&named(object)), "{:?} removed", object);
    }
    for object in &before.adventure.objects {
        prop_assert!(
            after.adventure.objects.contains(object) || removed.contains(&named(object)),
            "{:?} left untold",
            object
        );
    }
    // The tiles a removed object the viewer saw covered: what covers there may thin out.
    let (width, height) = (
        usize::from(before.adventure.width),
        usize::from(before.adventure.height),
    );
    let mut freed = vec![false; before.adventure.tiles.len()];
    for object in before
        .adventure
        .objects
        .iter()
        .filter(|o| removed.contains(&named(o)))
    {
        let mask = TileMask::from_bits(object.blocked.bits() | object.visitable.bits());
        for (column, row) in mask.offsets() {
            let a = object.anchor;
            if let (Some(x), Some(y)) = (
                usize::from(a.x).checked_sub(usize::from(column)),
                usize::from(a.y).checked_sub(usize::from(row)),
            ) {
                freed[(usize::from(a.level) * height + y) * width + x] = true;
            }
        }
    }
    for ((was, is), freed) in before
        .adventure
        .tiles
        .iter()
        .zip(&after.adventure.tiles)
        .zip(freed)
    {
        prop_assert!(depth(was) <= depth(is));
        match (was, is) {
            (TileView::Seen(was), TileView::Seen(is)) if freed => {
                prop_assert_eq!(
                    (was.terrain, was.road, was.river),
                    (is.terrain, is.road, is.river)
                );
                prop_assert!(is.blocked <= was.blocked && is.visitable <= was.visitable);
            }
            _ if depth(was) == depth(is) => prop_assert_eq!(was, is),
            _ => {}
        }
    }
    if !own {
        prop_assert_eq!(&before.adventure.explored, &after.adventure.explored);
        prop_assert_eq!(&before.adventure.fringe, &after.adventure.fringe);
    }
    Ok(())
}

/// The tiles of `player`'s heroes, by id.
fn tiles(game: &GameState, player: PlayerId) -> Vec<(HeroId, TilePos)> {
    let heroes = game.adventure().heroes().iter();
    heroes
        .filter(|h| h.owner == player)
        .map(|h| (h.id, h.pos))
        .collect()
}

/// Whether a command was accepted, each hero's move (id, columns, rows, levels), the steps told,
/// the decisions opened (0), refused (1), declined (2) or accepted (3), and (E08 T19a) every
/// battle event, start and end, and the heroes removed.
type Outcome = (
    bool,
    Vec<(HeroId, i32, i32, i32)>,
    usize,
    Vec<u8>,
    Vec<GameEvent>,
);

/// What a command of `player` did in a game, as the hygiene test compares it between two games
/// (E08 T14b's review): whether it was accepted, how far each of the player's heroes moved from
/// its tile `before` (columns, rows, levels), how many steps the game told, and the decisions it
/// opened, refused or closed. Where the heroes stand is left out: the test may put them on other
/// tiles in the other game from the start.
fn outcome(
    (player, accepted): (PlayerId, bool),
    before: &[(HeroId, TilePos)],
    after: &GameState,
    events: &[GameEvent],
) -> Outcome {
    let moved = before
        .iter()
        .zip(tiles(after, player))
        .map(|(&(id, from), (_, to))| {
            let d = |a: u16, b: u16| i32::from(b) - i32::from(a);
            (
                id,
                d(from.x, to.x),
                d(from.y, to.y),
                i32::from(to.level) - i32::from(from.level),
            )
        })
        .collect();
    let steps = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::HeroStepped { .. },
                    ..
                })
            )
        })
        .count();
    let decisions = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::OfferOpened { .. } => Some(0),
            GameEvent::OfferRefused { .. } => Some(1),
            GameEvent::OfferAnswered { accepted, .. } => Some(2 + u8::from(*accepted)),
            _ => None,
        })
        .collect();
    let battles = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::BattleStarted { .. }
                    | GameEvent::Battle { .. }
                    | GameEvent::BattleEnded { .. }
                    | GameEvent::Adventure(AdventureEvent {
                        kind: AdventureEventKind::HeroRemoved { .. },
                        ..
                    })
            )
        })
        .copied()
        .collect();
    (accepted, moved, steps, decisions, battles)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// Observation hygiene: two games that differ only in what a viewer never sees (the seed,
    /// the other players' armies and so their points, the tiles it never sees and what objects
    /// cover there, the objects' contents) show it the same observation, legal commands and moves
    /// after the same commands, accept or reject its own commands alike and tell it the same
    /// events, as long as the other players' commands have the same outcome in both ([`outcome`]:
    /// accepted in both or in neither, their heroes moved alike, the same steps and decisions; a
    /// move that one hero can pay and the other cannot, or that visits a pile from its neighbour
    /// in one game and walks onto the pile's tile in the other, which lacks it, ends where the
    /// viewer may see it), and until
    /// the viewer picks up an object: a pickup tells its visitor the contents (E08 T13b; survey
    /// `E08-objects.md` §8), so the two games may then tell it other gains, and nothing else; or
    /// until it accepts an offer: a Cartographer reveals tiles by their terrain, hidden ones
    /// included, so the two games may then tell it other counts (E08 T14b). An offer opened,
    /// refused or declined tells it nothing hidden, nor does a key or a guard's passage, and an
    /// observatory's disk (T15a) is the same tiles in both games: the comparison goes on. A battle
    /// of the viewer's shows it the other side's stacks: the comparison ends when one starts (E08
    /// T19a); another player's battle must go alike in both games ([`outcome`]).
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
        // The viewer's heroes get armies and experience of their own: a starting army (also for a
        // hero whose slots are all empty) and a default experience are drawn from the seed (E08
        // T18b), and the viewer sees its heroes' details.
        for o in &mut m.objects {
            let Body::Hero(hero) = &mut o.body else { continue };
            if hero.owner == colour {
                if hero.army.iter().flatten().all(Option::is_none) {
                    hero.army = Some(stacks(&own));
                }
                hero.experience = Some(500);
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
            let (from_a, from_b) = (tiles(&a, actor), tiles(&b, actor));
            let ra = apply(&mut a, rules, Actor::Player(actor), command.clone(), &mut ea);
            let rb = apply(&mut b, rules, Actor::Player(actor), command, &mut eb);
            if actor == viewer {
                prop_assert_eq!(ra, rb);
            } else if outcome((actor, ra.is_ok()), &from_a, &a, &ea)
                != outcome((actor, rb.is_ok()), &from_b, &b, &eb)
            {
                // The other player's command did not do the same in both games.
                break;
            }
            let (ta, tb) = (told(&ea, viewer), told(&eb, viewer));
            // A battle shows its side the other's stacks (survey `E08-battles.md` §9; E08
            // T19a): the comparison ends when the viewer's starts.
            let battle = |events: &[GameEvent]| {
                events
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
            };
            if battle(&ta) || battle(&tb) {
                break;
            }
            let bought = ta
                .iter()
                .any(|e| matches!(e, GameEvent::OfferAnswered { accepted: true, .. }));
            if actor == viewer && (ta != tb || bought) {
                // The viewer learned a pickup's contents or bought a reveal: only what it gained
                // and how many tiles it explored may differ.
                let blind = |events: Vec<GameEvent>| -> Vec<Option<GameEvent>> {
                    events
                        .into_iter()
                        .map(|e| {
                            (!matches!(
                                e,
                                GameEvent::ResourceGained { .. }
                                    | GameEvent::Adventure(AdventureEvent {
                                        kind: AdventureEventKind::TilesRevealed { .. },
                                        ..
                                    })
                            ))
                            .then_some(e)
                        })
                        .collect()
                };
                prop_assert_eq!(blind(ta), blind(tb));
                break;
            }
            prop_assert_eq!(ta, tb);
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

/// [`vary`] changes what it may and nothing else: with every tile unseen, each kind of change
/// shows on a known plan; with none, only the other players' armies change.
#[test]
fn vary_changes_what_the_viewer_never_sees_only() {
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 8,
        two_levels: false,
        tiles: vec![(0, 0, 0); 64],
        players: vec![(0, true), (1, true)],
        heroes: vec![((1, 1, 0), None), ((6, 6, 0), None)],
        hero_types: 0,
        blockers: vec![((4, 1, 0), 0), ((1, 5, 0), 1)],
        difficulty: 1,
    };
    let m = lay_out(&plan);
    let unseen = Unseen {
        seed: None,
        armies: vec![Some(Some(vec![Some((10, 3))]))],
        tiles: vec![Some((4, 1, 2)); 64],
        removed: vec![true],
        added: vec![((3, 6, 0), 0)],
        moved: vec![Some((6, 2, 0))],
    };
    let other = vary(&m, 0, &unseen, |_| true, |_| true);
    // Every tile is swamp with a road and a river but the heroes', which stay land as swamp is.
    assert!(
        other
            .terrain
            .tiles
            .iter()
            .all(|t| (t.terrain, t.road, t.river) == (4, 1, 2))
    );
    let blockers = |m: &RawMap| -> Vec<(u8, u8, u32)> {
        m.objects
            .iter()
            .filter(|o| o.template != HERO_TEMPLATE)
            .map(|o| (o.pos.x, o.pos.y, o.template))
            .collect()
    };
    assert_eq!(blockers(&m), [(4, 1, 1), (1, 5, 2)]);
    assert_eq!(blockers(&other), [(3, 6, 1)]);
    let mut seen = other.clone();
    let mine: Vec<_> = heroes(&mut seen)
        .map(|(owner, army)| (owner, army.clone()))
        .collect();
    assert_eq!(mine[0], (0, None));
    assert_eq!(mine[1], (1, Some(stacks(&[Some((10, 3))]))));
    let tiles: Vec<Spot> = other
        .objects
        .iter()
        .filter(|o| o.template == HERO_TEMPLATE)
        .map(standing_on)
        .collect();
    assert_eq!(tiles, [(1, 1, 0), (6, 2, 0)]);
    // Nothing unseen: only the other player's army changes (armies never show).
    let same = vary(&m, 0, &unseen, |_| false, |_| false);
    assert_eq!(same.terrain.tiles, m.terrain.tiles);
    assert_eq!(blockers(&same), blockers(&m));
    let mut kept = same.clone();
    let armies: Vec<_> = heroes(&mut kept).map(|(_, army)| army.clone()).collect();
    assert_eq!(armies, [None, Some(stacks(&[Some((10, 3))]))]);
}

// covers: ADV-FOG-004, ADV-PIK-001, ADV-PIK-003
#[test]
fn a_typed_object_in_sight_shows_its_type_never_its_drawn_contents() {
    // Red's hero at (1, 1), with an army of its own, sees a wood pile of a drawn amount (object 3:
    // shape 0, kind 1) at (3, 1) and a campfire (object 9) at (1, 3); blue's hero is far away.
    let plan = super::play::Plan {
        version: crate::model::Version::Sod,
        side: 16,
        two_levels: false,
        tiles: vec![(0, 0, 0); 256],
        players: vec![(0, true), (1, true)],
        heroes: vec![
            (
                (1, 1, 0),
                Some(vec![Some((0, 10)), None, None, None, None, None, None]),
            ),
            ((14, 14, 0), None),
        ],
        hero_types: 0,
        blockers: vec![((3, 1, 0), 3), ((1, 3, 0), 9)],
        difficulty: 1,
    };
    let m = lay_out(&plan);
    let red = PlayerId::new(0);
    let contents = |seed| -> Vec<Vec<(h3_content::ResourceId, u32)>> {
        let game = start_on(&m, &FAITHFUL, seed);
        let objects = game.adventure().map().objects();
        objects.iter().map(|o| o.contents.clone()).collect()
    };
    let view = |seed| {
        let game = start_on(&m, &FAITHFUL, seed);
        observe(&game, &FAITHFUL, Viewer::Player(red)).adventure
    };
    let base = view(0);
    assert_eq!(base.objects.len(), 2);
    let part = FAITHFUL.adventure();
    assert_eq!(
        base.objects[0].object_type,
        part.object_type_by_key("base:pile_wood").unwrap()
    );
    assert_eq!(
        base.objects[1].object_type,
        part.object_type_by_key("base:campfire").unwrap()
    );
    // Some seeds draw other contents; the observation is the same for each.
    let drawn: Vec<_> = (0..20).map(contents).collect();
    assert!(drawn.iter().any(|c| *c != drawn[0]));
    for seed in 0..20 {
        assert_eq!(view(seed), base, "seed {seed}");
    }
}
