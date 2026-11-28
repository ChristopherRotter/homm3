//! The planning (E31 T2i; survey `docs/rules/surveys/E31-computer-player.md` §1.3, §1.4, §1.9):
//! what a computer hero's turn decides before it walks. The danger map of the enemy heroes it
//! would lose to (`ai_danger_map` 0x42DE50, [`danger_map`], CPU-PLN-001); then up to five runs of
//! the destination (`ai_hero_search_and_walk` 0x5267B0, [`search_and_walk`], CPU-PLN-008), each
//! its own search with the jumps (CPU-PLN-007) and its colleagues' claims (`ai_hero_reach_claims`
//! 0x42F570, CPU-PLN-002), the candidates (`ai_hero_candidates` 0x42EDD0, CPU-PLN-003), their
//! value field (CPU-PLN-004), each one's score (`ai_candidate_score` 0x42F980, CPU-PLN-005) and the
//! choice (`ai_hero_destination` 0x42E0B0, CPU-PLN-006). Its searches are the computer's
//! (`h3_adventure::search`, CPU-PTH-001 to -007); its values the object value's (T2h,
//! `super::objects`); its draws the caller's stream, one per scored candidate.
//!
//! The hero's kept plan (its destination, the cost beyond its points, the must-go) is the
//! memory's ([`Plans`]), written at the end of every run: the next run, a colleague's claims and
//! the next turn read it. The walk (`ai_hero_walk` 0x42FEE0, T2j) is not ported: a hero with a
//! destination is done as one without (`AI-STUB-2`), so every hero is still done after its plan.

mod danger;
mod field;
mod reach;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;

use h3_adventure::{ComputerSearch, HeroOnMap, Pos, SearchJumps, SearchNode};
use h3_contracts::PlayerId;
use h3_kernel::Rng;

pub(super) use danger::{Danger, danger_map};

use super::artifacts::Valued;
use super::fight::Fighter;
use super::memory::{Plan, Plans};
use super::objects::{self, Asking, At};
use super::start::Record;
use super::towns::Asked;
use super::turn::towns_of;
use crate::{GameRuleset, GameState};

/// The search's budget without a kept destination (0x5267E0: 1 000).
const BUDGET: i32 = 1000;

/// What a kept destination's budget adds to its cost beyond and the points left (0x5267F3: 200).
const BUDGET_MARGIN: i32 = 200;

/// The budget of a hero alone and the doubling's cap without a patrol (0x52680D, 0x5267BB:
/// 32 000).
const CAP: i32 = 32_000;

/// The runs of the destination at most (0x5268C0: 5).
const RUNS: usize = 5;

/// The rate below which a hero on a town's entrance waits there on the week's last day
/// (0x526992: 75; the day 7 at 0x52699D).
const WAIT_BELOW: i32 = 75;

/// The week's last day (0x52699D: 7).
const LAST_DAY: u8 = 7;

/// The percentage of the day's points within which a candidate is near (0x42E5A4..0x42E5BF:
/// × 21 / 100).
const NEAR_PERCENT: i32 = 21;

/// The best node cost before any candidate is taken (0x42E58A: 0x7FFF).
const NO_COST: i32 = 0x7FFF;

/// The candidates the choice and the field read at most: their 16-bit counters compared
/// unsigned after a sign extension (0x42E195, 0x42E97A) stop at 32 768.
const READ: usize = 0x8000;

/// A candidate destination (`ai_candidate`, `structs.toml`: 16 bytes): its tile (none: −1), its
/// value (+4), its cost (+8), whether it is near (+0xC) and a must-go (+0xD).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Candidate {
    pub(super) pos: Option<Pos>,
    pub(super) value: i32,
    pub(super) cost: i32,
    pub(super) near: bool,
    pub(super) must_go: bool,
}

impl Candidate {
    /// The search loop's destination before its first run (0x5267CD, 0x5267D3): none, not near;
    /// its value and cost, unset there, are read by none before they are written.
    const NONE: Self = Self {
        pos: None,
        value: 0,
        cost: 0,
        near: false,
        must_go: false,
    };
}

/// What a hero's search and walk ends in (`ai_hero_search_and_walk` 0x5267B0): every one leaves
/// the hero done until the walk (T2j).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    /// No destination: asleep, its destination none (0x526926).
    Nothing,
    /// On a town's entrance on the week's last day with a rate below 75: asleep there to meet the
    /// new week's growth (0x5269A7).
    Waits,
    /// The walk to this destination (`ai_hero_walk` 0x42FEE0, T2j: `AI-STUB-2`).
    Walks(Pos),
}

/// What one hero's planning reads: the game, the hero and its values' reads, its owner's turn
/// start's record, the turn's danger map and flag, its jumps.
pub(super) struct Planner<'a> {
    pub(super) state: &'a GameState,
    pub(super) rules: &'a GameRuleset,
    /// The hero.
    pub(super) hero: Fighter<'a>,
    /// Its reads for the object value.
    pub(super) valued: &'a Valued<'a>,
    /// Its owner's turn start's record.
    pub(super) record: &'a Record,
    /// The turn's danger map.
    pub(super) danger: &'a Danger,
    /// The turn's flag F.
    pub(super) flag: bool,
    /// Its mode-4 jumps ([`reach::jumps`]).
    pub(super) jumps: SearchJumps,
}

impl<'a> Planner<'a> {
    /// The planner of `hero` (its values' reads `valued`) with the record, the danger map and the
    /// turn's flag.
    pub(super) fn new(
        (state, rules): (&'a GameState, &'a GameRuleset),
        valued: &'a Valued<'a>,
        (record, danger, flag): (&'a Record, &'a Danger, bool),
    ) -> Self {
        let hero = valued.hero;
        Self {
            state,
            rules,
            hero,
            valued,
            record,
            danger,
            flag,
            jumps: reach::jumps(state, rules, hero),
        }
    }

    /// The hero on the map.
    fn on_map(&self) -> &'a HeroOnMap {
        self.hero.on_map
    }

    /// The hero's owner, the current player.
    fn player(&self) -> PlayerId {
        self.hero.on_map.owner
    }

    /// The hero's points left (+0x4D) as the original's `i32`.
    fn points(&self) -> i32 {
        i32::from_ne_bytes(self.on_map().movement.to_ne_bytes())
    }

    /// The object value's question with the hero's kept destination `destination`.
    fn asking(&self, destination: Option<Pos>) -> Asking<'_, 'a> {
        Asking {
            asked: Asked {
                hero: self.valued,
                record: self.record,
            },
            destination,
        }
    }

    /// Whether the player owns a town (player +0x3E above 0).
    fn owns_town(&self) -> bool {
        towns_of(self.state, self.player()).next().is_some()
    }

    /// Whether the player has explored `pos` (`tile_vision_word` 0x4F79B0 against
    /// `current_player_bit` 0x69CCC4).
    fn explored(&self, pos: Pos) -> bool {
        self.state.adventure().is_explored(self.player(), pos)
    }

    /// The index of `pos` in a buffer of a value per tile of every level: (level × height +
    /// row) × width + column.
    fn index(&self, pos: Pos) -> usize {
        let map = self.state.adventure().map();
        let (width, height) = (usize::from(map.width()), usize::from(map.height()));
        (usize::from(pos.level) * height + usize::from(pos.y)) * width + usize::from(pos.x)
    }

    /// The tiles of every level.
    fn tiles(&self) -> usize {
        let map = self.state.adventure().map();
        usize::from(map.levels()) * usize::from(map.width()) * usize::from(map.height())
    }
}

/// The node of `pos`'s place in `search`, a node not reached read as the original's reset node:
/// every field 0 (`pathfinder_reset` 0x4B1530), its source the tile (0, 0, 0).
pub(super) fn place(search: &ComputerSearch, pos: Pos) -> SearchNode {
    search.node(pos).unwrap_or(SearchNode {
        pos,
        marks: 0,
        from: Pos::new(0, 0, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 0,
        cost2: 0,
        points: 0,
    })
}

/// Whether what stands at a tile is an entrance (map item `+0xD` bit 0x10): a hero, an empty boat
/// or an object visited from the tile (an event is none: the original records its class on its
/// tile without the bit, the engine keeps it apart, `StaticMap::events_at`).
pub(super) fn entrance(at: Option<&At<'_>>) -> bool {
    at.is_some()
}

/// Whether the class of what stands at a tile is visited from the neighbouring tile (the object
/// settings' byte +0, `object_settings_pointer` 0x660428): a hero's and an empty boat's are, an
/// object's by its type's visit (`Visit::Neighbour`); a tile without an object's is not.
pub(super) fn from_neighbour(rules: &GameRuleset, at: Option<&At<'_>>) -> bool {
    match at {
        None => false,
        Some(At::Hero(_) | At::Boat) => true,
        Some(At::Object { ty, .. }) => rules
            .adventure()
            .object_type(*ty)
            .is_some_and(|t| t.visit == h3_adventure::Visit::Neighbour),
    }
}

/// A hero's search and walk (`ai_hero_search_and_walk` 0x5267B0; survey §1.4; CPU-PLN-008), its
/// alone flag `alone`, its owner's kept plans `plans`, the draws from `rng`: with no kept
/// destination its must-go cleared (the memory keeps none then); the budget 1 000, with a kept
/// destination its cost beyond plus the points left plus 200 if more, 32 000 when alone (the
/// patrol's cap, `AI-STUB-5`: none, the cap 32 000); up to five runs of the destination
/// ([`Planner::destination`]) on one destination and score carried over, stopped by a
/// destination whose rate is not negative or that is a must-go; else, the search cut short by its
/// bound and the budget below the cap, the budget doubled (at most the cap) and run again. No
/// destination: [`Outcome::Nothing`]. On a town's entrance (any town's) with a rate below 75 on the
/// week's last day: [`Outcome::Waits`]. Else the walk (T2j, `AI-STUB-2`). The original's
/// `ai_turn_counter` 0x691684, which it moves here when the hero has not moved today, is a vestige
/// nothing reads: not ported.
// rule: CPU-PLN-008
pub(super) fn search_and_walk(
    planner: &Planner<'_>,
    plans: &mut Plans,
    alone: bool,
    rng: &mut Rng,
) -> Outcome {
    let hero = planner.on_map();
    let mut budget = plans.get(hero.id).map_or(BUDGET, |plan| {
        let sum = i32::from(plan.beyond)
            .wrapping_add(planner.points())
            .wrapping_add(BUDGET_MARGIN);
        sum.max(BUDGET)
    });
    if alone {
        budget = CAP;
    }
    // stub: AI-STUB-5
    let mut chosen = Candidate::NONE;
    let mut score = 0;
    let mut rate = 0;
    for _ in 0..RUNS {
        let short;
        (rate, short) = planner.destination(plans, budget, (&mut chosen, &mut score), rng);
        let done = chosen.pos.is_some() && (rate >= 0 || chosen.must_go);
        if done || !short || budget >= CAP {
            break;
        }
        budget = CAP.min(budget * 2);
    }
    let Some(pos) = chosen.pos else {
        return Outcome::Nothing;
    };
    let state = planner.state;
    let on_town = state.adventure().map().town_at(hero.pos).is_some();
    if on_town && rate < WAIT_BELOW && state.weekday(planner.rules).get() == LAST_DAY {
        return Outcome::Waits;
    }
    // stub: AI-STUB-2
    Outcome::Walks(pos)
}

impl Planner<'_> {
    /// One run of the destination (`ai_hero_destination` 0x42E0B0; survey §1.9; CPU-PLN-006) with
    /// the budget `budget`, the destination `chosen` and its score carried from the run before:
    /// the hero's search, its claims and candidates ([`Self::candidates`]) and their value field
    /// ([`Self::field`]); then the choice. Its rate R starts at the danger of the hero's tile and
    /// its best cost at 0x7FFF; the destination's must-go is cleared. With no destination: not
    /// near; R negative, valued 0; else valued 1 and, the search not cut short, a hero on an
    /// object's entrance (not its own town's; the Sanctuary's, E12) steps off it to the cheapest
    /// of its eight neighbours ([`Self::step_off`]), its best cost 0. With one: R its score per
    /// 100 points of its cost (above 100), its best cost its cost, near as [`Self::near`] tells of
    /// its node. Then each candidate in order ([`Self::choose`]). At the end the hero's kept plan
    /// is written (0x42EBAB..0x42EBE9): the destination, the best cost beyond the points left (0
    /// when not beyond, a word) and the must-go. Returns R and whether the search was cut short;
    /// for a hero not on the map 0 and not short, nothing written.
    // rule: CPU-PLN-006
    fn destination(
        &self,
        plans: &mut Plans,
        budget: i32,
        (chosen, score): (&mut Candidate, &mut i32),
        rng: &mut Rng,
    ) -> (i32, bool) {
        let hero = self.on_map();
        let kept = plans.get(hero.id);
        let asking = self.asking(kept.map(|plan| plan.destination));
        let Some(search) = self.search(budget, asking) else {
            return (0, false);
        };
        let (danger, candidates) = self.candidates(&search, plans, asking);
        let field = self.field(&candidates);
        let short = search.short();
        let mut best = danger;
        let mut cheapest = NO_COST;
        chosen.must_go = false;
        match chosen.pos {
            None => {
                chosen.near = false;
                chosen.value = i32::from(best >= 0);
                if best >= 0 && !short {
                    self.step_off(&search).into_iter().for_each(|pos| {
                        chosen.pos = Some(pos);
                        cheapest = 0;
                    });
                }
            }
            Some(pos) => {
                best = if chosen.cost > 100 {
                    score.wrapping_mul(100) / chosen.cost
                } else {
                    *score
                };
                cheapest = chosen.cost;
                let node = place(&search, pos);
                let close = node.from == hero.pos || i32::from(node.cost) <= self.near_points();
                chosen.near = close && self.near(&node, best);
            }
        }
        let mut choice = Choice {
            best,
            cheapest,
            softened: Vec::new(),
        };
        let asked = (asking, kept);
        self.choose(
            (&search, asked),
            (&candidates, &field),
            (chosen, score),
            &mut choice,
            rng,
        );
        let points = self.points();
        let beyond = if choice.cheapest > points {
            choice.cheapest.wrapping_sub(points).to_le_bytes()
        } else {
            [0; 4]
        };
        let plan = chosen.pos.map(|destination| Plan {
            destination,
            beyond: u16::from_le_bytes([beyond[0], beyond[1]]),
            must_go: chosen.must_go,
        });
        plans.set(hero.id, plan);
        (choice.best, short)
    }

    /// The day's points within which a node is near: the day's maximum (+0x49) × 21 / 100
    /// (0x42E5A4..0x42E5BF, 32 bits).
    fn near_points(&self) -> i32 {
        let most = i32::from_ne_bytes(self.on_map().max_movement.to_ne_bytes());
        most.wrapping_mul(NEAR_PERCENT) / 100
    }

    /// Whether a destination near by its tile or cost stays near: not when its node's second cost
    /// is above its cost, its rate `rate` negative or the player without a town (0x42E943..0x42E95C,
    /// 0x42EA73..0x42EA80).
    fn near(&self, node: &SearchNode, rate: i32) -> bool {
        node.cost2 <= node.cost && rate >= 0 && self.owns_town()
    }

    /// The step off an object's entrance (0x42E625..0x42E87D): the hero, hidden, on an entrance
    /// (`objects::beneath`) that is not its own town's (class 98 of its owner) nor a Sanctuary's
    /// (80, none in the engine: `AI-STUB-23`), takes of its eight neighbours on the map
    /// (`step_directions` 0x678150's order) the first with no entrance, no guard
    /// (`AdventureState::guard_of`, the vision word's guard flag), its place reached and a danger
    /// not negative, and after it each one strictly cheaper.
    fn step_off(&self, search: &ComputerSearch) -> Option<Pos> {
        let (state, rules) = (self.state, self.rules);
        let hero = self.on_map();
        let adventure = state.adventure();
        let below = objects::beneath(state, rules, hero.pos);
        let own_town = adventure.map().town_at(hero.pos).is_some_and(|town| {
            let map = adventure.map();
            map.town_object(town)
                .and_then(|index| adventure.owner(index))
                == Some(hero.owner)
        });
        // stub: AI-STUB-23
        if !entrance(below.as_ref()) || own_town {
            return None;
        }
        let mut best: Option<(Pos, u16)> = None;
        for (dx, dy) in reach::STEPS {
            let tile = hero
                .pos
                .x
                .checked_add_signed(dx.into())
                .zip(hero.pos.y.checked_add_signed(dy.into()))
                .map(|(x, y)| Pos::new(x, y, hero.pos.level))
                .filter(|&tile| adventure.map().tile(tile).is_some());
            let free = tile.filter(|&tile| {
                !entrance(objects::at(state, rules, tile).as_ref())
                    && adventure.guard_of(rules.adventure(), tile).is_none()
            });
            let node = free.and_then(|tile| search.node(tile));
            let kept = node.filter(|node| {
                best.is_none_or(|(_, cost)| node.cost < cost) && self.danger.at(node.pos) >= 0
            });
            if let Some(node) = kept {
                best = Some((node.pos, node.cost));
            }
        }
        best.map(|(pos, _)| pos)
    }
}

/// The choice's running state (`ai_hero_destination` 0x42E0B0's locals): the best rate R
/// (`ebp − 0x28`), the best node cost (`ebp − 0x18`) and the nodes whose danger the score
/// softened in this run (written into the hero's search, 0x42FACF).
pub(super) struct Choice {
    pub(super) best: i32,
    pub(super) cheapest: i32,
    pub(super) softened: Vec<Pos>,
}

impl Planner<'_> {
    /// Each candidate in order, the first 32 768 (0x42E960..0x42EB8B; CPU-PLN-006): skipped (no
    /// score, no draw) when the destination is valued and it is not; near (N) when its node's
    /// source is the hero's tile and it is an entrance, or both its node's cost and its cost are
    /// within the day's 21 %, as [`Self::near`] keeps it; scored ([`Self::score`]); its rate the
    /// score per 100 points of its cost above 100, at least 1 when the score is positive, the
    /// score else; a negative rate not near. Then: without value and not above R, skipped; a
    /// must-go taken; to a destination without value, one with value taken; else near against
    /// far, among near ones the cheaper node (equal: the higher rate), among far ones the higher
    /// rate. Taken: R its rate, the destination it with N, the best cost its node's, the score
    /// its; a must-go ends the choice.
    fn choose(
        &self,
        (search, asked): (&ComputerSearch, (Asking<'_, '_>, Option<Plan>)),
        (candidates, field): (&[Candidate], &field::Field),
        (chosen, score): (&mut Candidate, &mut i32),
        choice: &mut Choice,
        rng: &mut Rng,
    ) {
        let (state, rules) = (self.state, self.rules);
        let hero = self.on_map().pos;
        let within = self.near_points();
        for candidate in candidates.iter().take(READ) {
            let mut c = *candidate;
            if chosen.value > 0 && c.value == 0 {
                continue;
            }
            let pos = c.pos.unwrap_or(hero);
            let node = place(search, pos);
            let beside = node.from == hero && entrance(objects::at(state, rules, pos).as_ref());
            let close = beside || (i32::from(node.cost) <= within && c.cost <= within);
            let s = self.score(
                &mut c,
                &node,
                (search, field, asked),
                &mut choice.softened,
                rng,
            );
            let rate = if s > 0 {
                let per = if c.cost > 100 {
                    s.wrapping_mul(100) / c.cost
                } else {
                    s
                };
                per.max(1)
            } else {
                s
            };
            let near = close && self.near(&node, rate);
            if c.value == 0 && rate <= choice.best {
                continue;
            }
            let cost = i32::from(node.cost);
            let take = c.must_go
                || (chosen.value == 0 && c.value != 0)
                || match (chosen.near, near) {
                    (true, false) => false,
                    (false, true) => true,
                    (true, true) => {
                        choice.cheapest > cost || (choice.cheapest == cost && rate > choice.best)
                    }
                    (false, false) => rate > choice.best,
                };
            if take {
                choice.best = rate;
                *chosen = Candidate { near, ..c };
                choice.cheapest = cost;
                *score = s;
                if chosen.must_go {
                    break;
                }
            }
        }
    }
}
