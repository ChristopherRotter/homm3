//! The danger map (E31 T2i; survey §1.3 step 4, §1.9): `ai_danger_map` 0x42DE50, the turn's tile
//! buffer the hero's own searches read (pathfinder +0x6C, set at `ai_hero_turn` 0x526568 and
//! cleared at 0x526580: no other search reads it, the danger map's own neither).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{ComputerSearch, NoAsks, Pos, SearchFog, SearchJumps, SearchMode, SearchStart};
use h3_contracts::PlayerId;

use super::super::fight::{self, Fighter, Opponent};
use crate::{GameRuleset, GameState};

/// The points beyond an enemy hero's day's maximum its search runs with (0x42DF1B: 300).
const BEYOND: u32 = 300;

/// A fight result below which a tile is written over (0x42DFC3: −500 000 000), and what it is
/// written (0x42E050: −1 000 000 000).
const LOST: (i32, i32) = (-500_000_000, -1_000_000_000);

/// The turn's danger map: an `i32` per tile of every level, (level × height + row) × width +
/// column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct Danger {
    width: u16,
    height: u16,
    tiles: Vec<i32>,
}

impl Danger {
    /// The danger at `pos`, 0 off the map.
    pub(super) fn at(&self, pos: Pos) -> i32 {
        let (width, height) = (usize::from(self.width), usize::from(self.height));
        let inside = pos.x < self.width && pos.y < self.height;
        let index = (usize::from(pos.level) * height + usize::from(pos.y)) * width;
        self.tiles
            .get(index + usize::from(pos.x))
            .filter(|_| inside)
            .copied()
            .unwrap_or(0)
    }
}

/// The danger map of `hero`'s turn (`ai_danger_map` 0x42DE50, its call at `ai_hero_turn`
/// 0x5263B1; survey §1.3 step 4, §1.9; CPU-PLN-001): every tile 0 (the buffer cleared at
/// 0x526378); unless the game is on Easy and no human plays on the current player's team
/// (0x526381..0x5263AA, `team_has_human` 0x42B9E0), for each player in order not of the hero's
/// team (and not defeated: none before E10) each hero of its list (GAM-HER-003): the fight
/// estimate of the hero against that hero and its army, at no tile and no town (0x42DEEF); a
/// negative one runs that hero's search from its tile (mode 3, in a boat as it is, its day's
/// maximum as it would start a turn now plus 300 as bound and points, no danger: the
/// pathfinder's buffer is unset during the map) and every place it reached adds the result to its
/// tile (wrapping), or, the result below −500 000 000, is written −1 000 000 000.
// rule: CPU-PLN-001
pub(in super::super) fn danger_map(
    state: &GameState,
    rules: &GameRuleset,
    hero: Fighter<'_>,
) -> Danger {
    let adventure = state.adventure();
    let map = adventure.map();
    let (width, height) = (map.width(), map.height());
    let size = usize::from(map.levels()) * usize::from(width) * usize::from(height);
    let mut danger = Danger {
        width,
        height,
        tiles: vec![0; size],
    };
    let owner = hero.on_map.owner;
    let players = (0..=u8::MAX)
        .take(state.players())
        .map(PlayerId::new)
        .collect::<Vec<_>>();
    let human = |p: &PlayerId| adventure.allied(*p, owner) && state.human(*p).unwrap_or(false);
    if adventure.difficulty() == 0 && !players.iter().any(human) {
        return danger;
    }
    let (core, part) = (rules.core(), rules.adventure());
    let enemies = players
        .into_iter()
        .filter(|&player| !adventure.allied(player, owner))
        .flat_map(|player| adventure.hero_list(player))
        .filter_map(|enemy| adventure.hero(enemy).zip(state.hero(enemy)));
    for (on_map, details) in enemies {
        let enemy = on_map.id;
        let opponent = Opponent {
            hero: Some(Fighter { on_map, details }),
            army: &on_map.army,
            town: None,
        };
        let result = fight::fight(state, rules, hero, opponent, None);
        if result >= 0 {
            continue;
        }
        let reach = adventure
            .day_maximum_of(core, part, state.heroes(), enemy)
            .and_then(|most| {
                let points = most.saturating_add(BEYOND);
                let start = SearchStart {
                    player: owner,
                    hero: enemy,
                    from: on_map.pos,
                    boat: on_map.boat.is_some(),
                    bound: points,
                    points,
                    mode: SearchMode::Threat,
                    fog: SearchFog::Computer,
                    window: None,
                    jumps: SearchJumps::default(),
                    open: false,
                };
                adventure.computer_search(core, part, state.heroes(), &start, &NoAsks)
            });
        for node in reach.iter().flat_map(ComputerSearch::reached) {
            let at = (usize::from(node.pos.level) * usize::from(height) + usize::from(node.pos.y))
                * usize::from(width)
                + usize::from(node.pos.x);
            let tile = &mut danger.tiles[at];
            *tile = if result < LOST.0 {
                LOST.1
            } else {
                tile.wrapping_add(result)
            };
        }
    }
    danger
}
