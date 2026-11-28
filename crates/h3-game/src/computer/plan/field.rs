//! The candidates' value field and a candidate's score (E31 T2i; survey §1.9): how
//! `ai_hero_destination` 0x42E0B0 spreads each candidate's value over the tiles near it
//! (CPU-PLN-004) and `ai_candidate_score` 0x42F980 reads it (CPU-PLN-005).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{ComputerSearch, NoAsks, Pos, SearchFog, SearchJumps, SearchMode, SearchNode};
use h3_adventure::{SearchStart, SearchWindow};
use h3_kernel::Rng;
use h3_kernel::arith::{SINGLE_BITS, SignedX87};

use super::super::memory::Plan;
use super::super::objects::{self, Asking};
use super::reach::water;
use super::{Candidate, Planner, entrance, from_neighbour, place};

/// The field search's bound and points (0x42E318: 500; 0x42E30D: 59 999).
const LOCAL: (u32, u32) = (500, 59_999);

/// The field search's box: from 5 tiles before the candidate to 6 after, exclusive
/// (0x42E274..0x42E2C2).
const BOX: (u16, u16) = (5, 6);

/// The cost beyond m over which a value spreads at a third of a tile's (0x42E4DD..0x42E4EC:
/// value × 300 / (cost − m + 300)).
const SPREAD: i32 = 300;

/// A danger and a score at or below which the score softens the danger (0x42FABD, 0x42FAC7:
/// −500 000 000 and 5 000 000), the softened danger (0x42FACF: −2 500 000).
const REFUSED: i32 = -500_000_000;
const SOUGHT: i32 = 5_000_000;
const SOFTENED: i32 = -2_500_000;

/// The kept destination's factor (0x63AC78: the f32 1.5) and what it adds (0x42FBC8: 20).
const KEPT_FACTOR: u32 = 0x3FC0_0000;
const KEPT_ADDED: i32 = 20;

/// The draw of every other scored candidate (0x42FBE9..0x42FBEE: `random_between(1, 25)` + 75).
const DRAW: (u32, u32, i32) = (1, 25, 75);

/// The value field: an `i32` per tile of every level, (level × height + row) × width + column.
pub(super) struct Field(Vec<i32>);

impl Field {
    /// The field at `index`.
    pub(super) fn get(&self, index: usize) -> i32 {
        self.0[index]
    }

    /// Adds `value` to the field at `index` (wrapping).
    fn add(&mut self, index: usize, value: i32) {
        let slot = &mut self.0[index];
        *slot = slot.wrapping_add(value);
    }
}

impl Planner<'_> {
    /// The candidates' value field (`ai_hero_destination` 0x42E0B0 to 0x42E545; CPU-PLN-004), the
    /// first 32 768 in order: an unexplored candidate's value on its tile; an explored one's over
    /// the places a search from it reaches (mode 1, bound 500, 59 999 points, in a boat on water,
    /// the box from 5 tiles before it to 6 after, the computer's fog; its tile no entrance when
    /// its class is visited from the neighbouring tile, as 0x42E263 clears the entrance bit: the
    /// search's open start): the whole value at a cost up to m, value × 300 / (cost − m + 300)
    /// beyond (`i32`, the product wrapping). m is 0, or for a class visited from the neighbouring
    /// tile the highest cost of the places on the 3 × 3 tiles around it (on the map, a place not
    /// reached 0; compared as read, the cost against m as a signed word).
    // rule: CPU-PLN-004
    pub(super) fn field(&self, candidates: &[Candidate]) -> Field {
        let (state, rules) = (self.state, self.rules);
        let adventure = state.adventure();
        let map = adventure.map();
        let (width, height) = (map.width(), map.height());
        let mut field = Field(vec![0; self.tiles()]);
        for c in candidates.iter().take(super::READ) {
            let pos = c.pos.unwrap_or(self.on_map().pos);
            if !self.explored(pos) {
                field.add(self.index(pos), c.value);
                continue;
            }
            let open = from_neighbour(rules, objects::at(state, rules, pos).as_ref());
            let window = SearchWindow {
                left: pos.x.saturating_sub(BOX.0),
                top: pos.y.saturating_sub(BOX.0),
                right: pos.x.saturating_add(BOX.1).min(width),
                bottom: pos.y.saturating_add(BOX.1).min(height),
            };
            let start = SearchStart {
                player: self.player(),
                hero: self.on_map().id,
                from: pos,
                boat: water(state, rules, pos),
                bound: LOCAL.0,
                points: LOCAL.1,
                mode: SearchMode::Local,
                fog: SearchFog::Computer,
                window: Some(window),
                jumps: SearchJumps::default(),
                open,
            };
            let search = adventure.computer_search(
                rules.core(),
                rules.adventure(),
                state.heroes(),
                &start,
                &NoAsks,
            );
            let m = search.as_ref().filter(|_| open).map_or(0, |search| {
                let mut m = 0_i16;
                for x in pos.x.saturating_sub(1)..pos.x.saturating_add(2).min(width) {
                    for y in pos.y.saturating_sub(1)..pos.y.saturating_add(2).min(height) {
                        let cost = place(search, Pos::new(x, y, pos.level)).cost;
                        if i32::from(cost) > i32::from(m) {
                            m = i16::from_ne_bytes(cost.to_ne_bytes());
                        }
                    }
                }
                i32::from(m)
            });
            for node in search.iter().flat_map(ComputerSearch::reached) {
                let cost = i32::from(node.cost);
                let add = if cost <= m {
                    c.value
                } else {
                    c.value.wrapping_mul(SPREAD) / (cost - m + SPREAD)
                };
                field.add(self.index(node.pos), add);
            }
        }
        field
    }

    /// The score of the candidate `c` at its node `node` of the hero's search `search`
    /// (`ai_candidate_score` 0x42F980; survey §1.9; CPU-PLN-005), on the value field: for an
    /// explored entrance of a class visited from the neighbouring tile its cost moved onto the
    /// place it is visited from (its node's cost off, the source's place's on) and the score read
    /// at that place; else at its own. The score is the field there plus the node's key, plus its
    /// danger while the score is not below −500 000 000; a danger at or below −500 000 000 under
    /// a score of 5 000 000 or more is first softened to −2 500 000, in the search's node (kept
    /// for this run in `softened`). For a class entered (a tile without an object's too), the
    /// monster guarding the tile (`find_guarding_monster` 0x41A460: `AdventureState::guard_of`'s,
    /// on its tile) but the one its path met adds its object value while the score is not below
    /// −500 000 000. The hero's kept destination: × 1.5 (÷ 1.5 below 0; the score stored as an
    /// f32), truncated, plus 20, and a must-go when the kept one is. Any other: one draw r of 76
    /// to 100 from `rng`; above 0, × r / 100 (32 bits) and at least 1; else × (100.0 / r) in
    /// double precision, truncated.
    // rule: CPU-PLN-005
    pub(super) fn score(
        &self,
        c: &mut Candidate,
        node: &SearchNode,
        (search, field, (asking, kept)): (&ComputerSearch, &Field, (Asking<'_, '_>, Option<Plan>)),
        softened: &mut Vec<Pos>,
        rng: &mut Rng,
    ) -> i32 {
        let (state, rules) = (self.state, self.rules);
        let pos = c.pos.unwrap_or(node.pos);
        let at = objects::at(state, rules, pos);
        let visited_beside = from_neighbour(rules, at.as_ref());
        let mut node = *node;
        if entrance(at.as_ref()) && visited_beside && self.explored(pos) {
            c.cost = c.cost.wrapping_sub(node.cost.into());
            node = place(search, node.from);
            c.cost = c.cost.wrapping_add(node.cost.into());
        }
        let tile = node.pos;
        let mut danger = if softened.contains(&tile) {
            SOFTENED
        } else {
            node.danger
        };
        let mut s = field.get(self.index(tile)).wrapping_add(node.key);
        if danger <= REFUSED && s >= SOUGHT {
            danger = SOFTENED;
            softened.push(tile);
        }
        if s >= REFUSED {
            s = s.wrapping_add(danger);
        }
        if !visited_beside {
            let adventure = state.adventure();
            let guard = adventure
                .guard_of(rules.adventure(), pos)
                .and_then(|index| around(pos).find(|&t| adventure.tile_index(t) == Some(index)));
            if let Some(guard) = guard.filter(|&g| node.guard != Some(g) && s >= REFUSED) {
                s = s.wrapping_add(objects::value(asking, guard, &mut c.cost));
            }
        }
        if let Some(plan) = kept.filter(|plan| Some(plan.destination) == c.pos) {
            let f = SignedX87::int(s.into()).round(SINGLE_BITS);
            let factor = SignedX87::from_f32_bits(KEPT_FACTOR);
            let scaled = if s >= 0 {
                f.times(factor)
            } else {
                f.over(factor).unwrap_or(f)
            };
            c.must_go |= plan.must_go;
            return scaled.ftol().wrapping_add(KEPT_ADDED);
        }
        let r = DRAW.2 + i32::try_from(rng.range_inclusive(DRAW.0, DRAW.1)).unwrap_or(0);
        if s > 0 {
            return (s.wrapping_mul(r) / 100).max(1);
        }
        SignedX87::int(100)
            .over(SignedX87::int(r.into()))
            .unwrap_or(SignedX87::ZERO)
            .times(SignedX87::int(s.into()))
            .ftol()
    }
}

/// The 3 × 3 tiles around `pos` on its level, column by column from the left and each from the
/// top (`find_guarding_monster` 0x41A460's order); a tile past the map's left or top edge read
/// as the edge's, past its right or bottom one off the map (where no object stands).
fn around(pos: Pos) -> impl Iterator<Item = Pos> {
    let rows = move || pos.y.saturating_sub(1)..=pos.y.saturating_add(1);
    (pos.x.saturating_sub(1)..=pos.x.saturating_add(1))
        .flat_map(move |x| rows().map(move |y| Pos::new(x, y, pos.level)))
}
