//! The computer's walker (0x421F80, 0x421680; `docs/formats/installation.md` §7.35, §7.37,
//! §7.38; BAT-AI-005, BAT-AI-006): a stack that cannot shoot now values each enemy by the best
//! hex to strike it from (the plan: 0x4360C0, 0x436840 and 0x436300 with the shooters it would
//! block, the multi-headed and breath strikes and the enemy's retaliation), its melee value with
//! the run-up (0x435C70) and a draw of 75 to 100 % (0x421B02), takes the best, and moves and
//! attacks it within one turn, else walks towards the plan's hex (0x41F580). Nothing chosen:
//! Wait on Hard and above or for a human side, outside the wait phase; else Defend.
//!
//! The hex scores (0x4214F0, 0x421590, 0x420260, 0x41FB60; BAT-AI-007) come from [`scores`]: the
//! plan adds the score of its hex and the walk holds back on Hard and above rather than step onto
//! a hex that scores worse (BAT-AI-006).
//!
//! The odds (0x420A80, [`odds`]; BAT-AI-008) decide three branches: a negative melee value takes
//! the plan's value alone, a defender at a fort that holds its walls takes only the enemies inside
//! them ([`Walker::holds_walls`], BAT-AI-011), and a losing attack gives way to the fallbacks. On
//! Hard and above the last retaliation's gain ([`gain`], BAT-AI-009) starts an enemy's value.
//! Without a candidate a besieger walks to its row's wall, and the fallbacks ([`Walker::fallback`],
//! BAT-AI-010) guard a shooter or step to safety.
//!
//! The creature cast entry ([`casts::entry`], 0x421280; BAT-AI-012, BAT-AI-013) comes before the
//! action, with the best's value per turn (0 without a candidate), and again after a choice of
//! nothing. Not built yet: the walker's turn in the tactics phase is T6's (BAT-STUB-5).
//!
//! The spell context's predictions (E23 T4b2, [`predict`](crate::predict)) run a stack's turn as a
//! prediction ([`Walker::predicting`]: 0x421F80 with its flag): its context reads each stack's
//! hit points less the damage planned so far, an enemy without any left is no candidate, no cast
//! is asked, and of the actions only the attack within a turn counts, the original's move and
//! attack even where its walk would stop short (the fallbacks but the attack after all, the
//! walks and the waits plan nothing and are not built).

use core::cell::{Cell, OnceCell};

use h3_battle::computer::{self, Context, Map, Planned};
use h3_battle::{
    BattleSide, BattleState, Command, Field, Hex, HexSet, Stack, StackId, StackTraits, Status,
};
use h3_content::ORIGINAL_AI_SLIPS;
use h3_content_battle::{Ability, BattleRuleset};
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use crate::casts;
use crate::gain::{self, Records};
use crate::hexes::{self, Shape};
use crate::odds;
use crate::scores;

/// Hexes a table of hex scores covers.
pub(crate) const HEXES: usize = 256;

/// The draw of a candidate's value, percent (0x421B02: `random(75, 100)`).
const DRAW: (u32, u32) = (75, 100);

/// The walker's hex scores (0x421680's 187 integers): what standing on a hex costs or gains.
pub(crate) type Scores = [i32; HEXES];

/// The best hex to strike an enemy from (0x4360C0's +0x1C to +0x24): the hex, the turns to it and
/// the value of the hex's extras and dangers.
#[derive(Debug, Clone, Copy)]
struct Plan {
    hex: Hex,
    turns: u32,
    value: i32,
}

/// The best candidate so far (0x421680's locals): the enemy, its plan, its drawn value per turn,
/// its plan's value `P` drawn alike per turn (`trunc(P × r / (100 × turns))`, a 32-bit product)
/// and whether only the plan's hex counts.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Best {
    enemy: StackId,
    plan: Plan,
    value: i32,
    plan_value: i32,
    only: bool,
}

/// A striker's estimate of a strike from a position (0x436620, 0x436760): the striker, its shape
/// there and its position, and the creatures that strike.
#[derive(Debug, Clone, Copy)]
struct Striker {
    id: StackId,
    shape: Shape,
    at: Hex,
    count: u64,
}

/// One walker's choice: its battle, side, context, map and scores.
pub(crate) struct Walker<'a> {
    pub(crate) state: &'a BattleState,
    pub(crate) ruleset: &'a BattleRuleset,
    pub(crate) field: Field,
    pub(crate) id: StackId,
    pub(crate) me: &'a Stack,
    pub(crate) side: BattleSide,
    pub(crate) ctx: Context,
    pub(crate) map: Map,
    /// The hex scores (BAT-AI-007).
    pub(crate) scores: Scores,
    /// The enemies that can strike the side this turn, by record bit (the reach's mask, 0x420260):
    /// the area threat's (0x41F920).
    pub(crate) mask: u32,
    /// The living stack on each hex (0x4E7170).
    pub(crate) on: [Option<StackId>; HEXES],
    /// The game is above Easy or the side's player human (`[game]+0x1F6D8 > 0`, +0x54A4).
    pub(crate) normal: bool,
    /// The odds (0x420A80), computed when first asked: they read nothing the choice changes
    /// (ADR-0027 point 11: an optimisation that changes no choice).
    stronger: Cell<Option<bool>>,
    /// The side's threat records on the walker's turn (0x422B20 with flag 1), built when the last
    /// retaliation's gain first needs them.
    records: OnceCell<Records>,
    /// With BAT-QUIRK-25 (survey slip 22), the map the computer's pathfinder holds after a
    /// creature cast entry that built a spell context and cast nothing ([`casts::entry`]): the
    /// besieger's walk and the fallbacks read it ([`Walker::cells`]).
    pub(crate) held: OnceCell<Map>,
    /// Whether the turn asks the creature cast entry (0x421280): not in a prediction (its
    /// context's +9).
    asks_entry: bool,
    /// Whether the turn records the stops of the pathfinder's last search after its map
    /// ([`Walker::after_map`]): Teleport's first turn ([`Walker::teleport_turns`]).
    tracks: bool,
    /// With BAT-QUIRK-25, in Teleport's first turn, the stops of the last search after the
    /// walker's map when a creature cast entry's spell context made it (survey slip 30,
    /// [`Walker::searched`]); none when its own map's or its walk's search was the last.
    after_map: Cell<Option<HexSet>>,
    /// The choice's flag 1 (0x421680 from the hero's Teleport, 0x43AA60; BAT-AI-020): the map is
    /// the destination map and the choice ends in a move and attack or Defend
    /// ([`Walker::teleported`]).
    teleport: bool,
}

/// The command of the computer's walker `id` of `side` (0x421F80; BAT-AI-005): the walker's choice
/// ([`Walker::choose`], its draws from `rng`); without one the creature cast entry again with the
/// value the choice left ([`casts::entry`], BAT-AI-012); without a cast, Wait on Hard and above or
/// for a human side outside the wait phase, else Defend.
// rule: BAT-AI-005
pub(crate) fn turn(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    rng: &mut Rng,
) -> Command {
    Walker::new(state, ruleset, (id, side)).turn(rng).0
}

impl<'a> Walker<'a> {
    pub(crate) fn new(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (id, side): (StackId, BattleSide),
    ) -> Self {
        Self::with(state, ruleset, (id, side), None)
    }

    /// The walker `id` of `side` on a predicted turn with the damage `planned` so far (0x421F80
    /// with its prediction flag; E23 T4b2): its context predicts ([`Context::predicting`]).
    pub(crate) fn predicting(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (id, side): (StackId, BattleSide),
        planned: &Planned,
    ) -> Self {
        Self::with(state, ruleset, (id, side), Some(planned))
    }

    fn with(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (id, side): (StackId, BattleSide),
        planned: Option<&Planned>,
    ) -> Self {
        let field = Field::new(ruleset.battle().field);
        let mut on = [None; HEXES];
        for (i, s) in (0u8..).zip(state.stacks()) {
            if s.is_alive() && !s.is_turret() {
                for h in s.footprint(field).hexes() {
                    on[h.index()] = Some(StackId(i));
                }
            }
        }
        let ctx = computer::context(state, ruleset, side);
        let ctx = planned.map_or(ctx, |p| ctx.predicting(p));
        let (scores, mask) = scores::build(state, ruleset, (id, side), &ctx);
        let me = &state.stacks()[id.index()];
        // The map's range is 0 while the stack is blind, petrified or paralysed (0x421771 to
        // 0x42178B): only Teleport's turns ask such a stack.
        let map = if me.spells.status.contains(Status::NO_TURNS) {
            computer::map_within(state, ruleset, (id, side), 0)
        } else {
            computer::map(state, ruleset, id, side)
        };
        Self {
            state,
            ruleset,
            field,
            id,
            me,
            side,
            ctx,
            map,
            scores,
            mask,
            on,
            normal: computer::difficulty(state) > 0 || computer::human(state, side),
            stronger: Cell::new(None),
            records: OnceCell::new(),
            held: OnceCell::new(),
            asks_entry: planned.is_none(),
            tracks: false,
            after_map: Cell::new(None),
            teleport: false,
        }
    }

    /// The map the besieger's walk and the fallbacks read (the computer's pathfinder's cells,
    /// 0x421DA5, 0x421EA9, 0x421ECF): the walker's own, but with BAT-QUIRK-25 after a creature
    /// cast entry that built a spell context and cast nothing the one that context's last search
    /// left (survey slip 22, [`Walker::held`]). Their walks search afresh (0x41F580).
    // rule: BAT-AI-005, BAT-AI-010
    pub(crate) fn cells(&self) -> &Map {
        self.held.get().unwrap_or(&self.map)
    }

    /// The walker's turn (0x421F80; BAT-AI-005) and its value: the choice ([`Walker::choose`],
    /// its draws from `rng`) and the value it wrote; without one the creature cast entry again
    /// with that value ([`casts::entry`], BAT-AI-012), when the turn asks it, and the value the
    /// entry left; without a cast, Wait on Hard and above or for a human side outside the wait
    /// phase, else Defend, and 0.
    // rule: BAT-AI-005
    pub(crate) fn turn(&self, rng: &mut Rng) -> (Command, i32) {
        let mut value = 0;
        self.choose(rng, &mut value)
            .or_else(|| self.entry(&mut value, rng))
            .map_or_else(
                || {
                    let hard = computer::difficulty(self.state) >= 2
                        || computer::human(self.state, self.side);
                    let idle = if hard && !computer::wait_phase(self.state) {
                        Command::Wait
                    } else {
                        Command::Defend
                    };
                    (idle, 0)
                },
                |command| (command, value),
            )
    }

    /// The creature cast entry with `value` ([`casts::entry`]) when the turn asks it.
    fn entry(&self, value: &mut i32, rng: &mut Rng) -> Option<Command> {
        if self.asks_entry {
            casts::entry(self, value, rng)
        } else {
            None
        }
    }

    /// Teleport's two turns of the walker `id` of `side` (0x43AA60's calls of 0x421F80, casts not
    /// skipped; BAT-AI-020), their draws from `rng`: the real turn's value ([`Walker::turn`]: the
    /// creature cast entry asked as on the stack's own turn, the active stack or not, from its
    /// record, §7.59), then the command and value of its turn on the destination map of
    /// `destinations` ([`Walker::teleported`]). The scores, the odds and the threat records are the
    /// first turn's: they read nothing the map changes (ADR-0027 point 11).
    // rule: BAT-AI-020
    pub(crate) fn teleport_turns(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (id, side): (StackId, BattleSide),
        destinations: HexSet,
        rng: &mut Rng,
    ) -> (i32, (Command, i32)) {
        let real = Self::tracking(state, ruleset, (id, side));
        let (_, before) = real.turn(rng);
        (before, real.teleported(destinations).turn(rng))
    }

    /// The walker `id` of `side` for Teleport's first turn ([`Walker::teleport_turns`]): it
    /// records the stops of the last search after its map ([`Walker::after_map`]).
    pub(crate) fn tracking(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (id, side): (StackId, BattleSide),
    ) -> Self {
        Self {
            tracks: true,
            ..Self::new(state, ruleset, (id, side))
        }
    }

    /// Whether the turn records the last search's stops after its map (Teleport's first turn).
    pub(crate) const fn tracks_search(&self) -> bool {
        self.tracks
    }

    /// A creature cast entry's spell context left a search with `stops` in the computer's
    /// pathfinder after the walker's map (survey slip 30; [`Walker::after_map`]).
    pub(crate) fn searched(&self, stops: HexSet) {
        self.after_map.set(Some(stops));
    }

    /// The walker on the destination map of `destinations` with the choice's flag 1 (0x421680
    /// from 0x43AA60; BAT-AI-020): every destination one turn away ([`computer::destination_map`])
    /// at the stops of the pathfinder's last search before the map, with BAT-QUIRK-25 (survey slip
    /// 30: [`Walker::last_search`]), else none (a teleported stack enters no hex, the user).
    // rule: BAT-AI-020
    pub(crate) fn teleported(self, destinations: HexSet) -> Self {
        let (state, ruleset) = (self.state, self.ruleset);
        let stops = if ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS) {
            self.last_stops()
        } else {
            HexSet::new()
        };
        let map =
            computer::destination_map(state, ruleset, (self.id, self.side), destinations, stops);
        Self {
            map,
            held: OnceCell::new(),
            teleport: true,
            ..self
        }
    }

    /// The stops of the pathfinder's last search before the destination map (survey slip 30;
    /// [`computer::search_stops`]): of 0x420260's last enemy, when the hex scores search the
    /// enemies' reach ([`scores::last_reached`]); else of the last of the side's threat records'
    /// searches (0x422B20 with flag 1, [`gain::last_recorded`]); else the first turn's last after
    /// its map: a creature cast entry's spell context's, unless a walk searched after it
    /// ([`Walker::after_map`], §7.59); else the walker's own.
    fn last_stops(&self) -> HexSet {
        let (state, ruleset) = (self.state, self.ruleset);
        let reached = if self.normal {
            scores::last_reached(state, ruleset, self.side)
        } else {
            None
        };
        reached
            .or_else(|| gain::last_recorded(state, ruleset, &self.ctx, (self.id, self.side)))
            .map(|id| computer::search_stops(state, ruleset, id))
            .or_else(|| self.after_map.get())
            .unwrap_or_else(|| computer::search_stops(state, ruleset, self.id))
    }

    /// Whether the walker's turn is a prediction (its context's +9).
    pub(crate) fn predicts(&self) -> bool {
        self.ctx.predicts()
    }

    /// Whether the walker's side is the stronger at range (0x420A80; BAT-AI-008).
    pub(crate) fn stronger(&self) -> bool {
        self.stronger.get().unwrap_or_else(|| {
            let stronger = odds::stronger(self.state, self.ruleset, &self.ctx, self.side);
            self.stronger.set(Some(stronger));
            stronger
        })
    }

    /// The last retaliation's gain against `enemy` (0x41F3B0; BAT-AI-009), the side's threat
    /// records built the first time.
    fn gain(&self, enemy: StackId) -> i32 {
        let records = self.records.get_or_init(|| {
            gain::records(self.state, self.ruleset, &self.ctx, (self.id, self.side))
        });
        gain::gain(
            self.state,
            self.ruleset,
            &self.ctx,
            records,
            (self.id, self.side, enemy),
        )
    }

    /// The walker with the hex scores `scores` (the tests' scores by hand).
    #[cfg(test)]
    pub(crate) fn scored(self, scores: Scores) -> Self {
        Self { scores, ..self }
    }

    fn stack(&self, id: StackId) -> &'a Stack {
        &self.state.stacks()[id.index()]
    }

    /// The record index of stack `id` among its side's (+0xF8): the bit the computer's masks set.
    pub(crate) fn record(&self, id: StackId) -> u32 {
        let side = self.stack(id).side;
        let before = self.state.stacks().iter().take(id.index());
        let n = before.filter(|s| s.side == side).count();
        u32::try_from(n).unwrap_or(31)
    }

    pub(crate) fn score(&self, hex: Hex) -> i32 {
        self.scores[hex.index()]
    }

    /// The walker's choice (0x421680 outside a prediction; BAT-AI-005): over the other side's
    /// living stacks but the turrets, in record order (a bound walker only those next to it; a
    /// defender holding its walls only those inside them, [`Walker::holds_walls`]), each with a
    /// [`Walker::plan`]: its value `v` starts at the last retaliation's gain (on Hard and above or
    /// for a human side, an enemy it strikes within its speed that is not multi-headed; 0x41F3B0,
    /// BAT-AI-009), adds the melee value with the plan's cost as the run-up when the plan takes
    /// one turn (0x435C70), and the plan's value `P` joins it (outside Easy for a computer side, a
    /// positive `P` of a stack not summoned: the Harpies take the larger, `P` alone for a negative
    /// `v` when the side is the stronger at range, 0x420A80, else the sum); a negative `v` outside
    /// Easy drops the enemy when `P` is below its hex's score; then the draw, `v × random(75,
    /// 100) / 100`, and `P` alike. The best: one not blind, petrified or paralysed, then fewer
    /// turns, the value (the best's per turn against the candidate's whole), more hit points lost
    /// by the top creature, the lower cost to the plan's hex, the later record. Then: only the
    /// plan's hex when `P` was taken (a walk there, nothing on it); a losing attack (its value per
    /// turn below 0 and below its drawn `P` per turn, the stack not summoned, outside Easy for a
    /// computer side, the side the stronger at range) the fallbacks first
    /// ([`Walker::fallback`]); the attack ([`Walker::attack`]). No candidate: a besieger walks to
    /// its row's wall ([`Walker::besiege`]), any other the fallbacks. Before any of these, `value`
    /// becomes the larger of the best's value and its drawn `P` per turn (0 without a candidate)
    /// and the creature cast entry is asked with it ([`casts::entry`], BAT-AI-012: a cast ends
    /// the choice); only the plan's hex sets `value` to the drawn `P` per turn.
    // rule: BAT-AI-005
    pub(crate) fn choose(&self, rng: &mut Rng, value: &mut i32) -> Option<Command> {
        let me = self.me;
        let still = me.speed_now() == 0 || computer::bound(self.state, self.id);
        let other = self.side.opponent();
        let holds = self.holds_walls();
        let hard = computer::difficulty(self.state) >= 2 || computer::human(self.state, self.side);
        let speed = u16::from(me.speed_now());
        // On the destination map the run-up is the map's cost of 1 where a teleported stack
        // strikes from where it stands: with BAT-QUIRK-25 as the original (survey slip 31), else
        // none.
        let runs = !self.teleport || self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
        // The stack can act (0x4219DB to 0x421A18): not blind, petrified, paralysed or dead, not
        // the First Aid Tent or the Ammo Cart.
        let able = me.is_alive()
            && !me.spells.status.contains(Status::NO_TURNS)
            && !me.has(StackTraits::FIRST_AID)
            && !me.has(StackTraits::AMMO_SUPPLY);
        let mut best: Option<Best> = None;
        for (i, t) in (0u8..).zip(self.state.stacks()) {
            let enemy = StackId(i);
            // A bound walker or one without speed only those next to it (its map's marks at 0).
            // In a prediction not an enemy without hit points left after the planned damage.
            // stub: BAT-STUB-10
            // Not the walker itself: the original's loop over the other side's records holds a
            // hypnotized walker too and has no test for it (BAT-Q58); the engine refuses the
            // attack on itself (ADR-0027 point 3; the user, E23 T4b2).
            if t.side != other
                || enemy == self.id
                || !t.is_alive()
                || t.is_turret()
                || self.ctx.spent(self.state, enemy)
                || (still && self.map.cost(t.hex) > 0)
                || (holds && !self.inside_walls(t))
            {
                continue;
            }
            // The last retaliation's gain and the melee value, for a walker that can act (on its
            // turn and in a prediction always: only Teleport's turns ask one that cannot).
            let gains = able
                && hard
                && self.map.mark(t.hex).is_some_and(|c| c <= speed)
                && !t.has(StackTraits::ALL_ADJACENT)
                && !t.has(StackTraits::THREE_HEADED);
            let gain = if gains { self.gain(enemy) } else { 0 };
            let Some(plan) = self.plan(enemy) else {
                continue;
            };
            // The melee value with the run-up when the plan takes one turn.
            let run_up = u8::try_from(self.map.cost(plan.hex) * u16::from(plan.turns == 1 && runs))
                .unwrap_or(u8::MAX);
            let pair = (self.id, enemy);
            let ctx = &self.ctx;
            let melee = if able {
                computer::strike_value(self.state, self.ruleset, ctx, pair, run_up)
            } else {
                0
            };
            let mut v = gain.wrapping_add(melee);
            let p = plan.value;
            let mut only = false;
            if p > 0 && !me.has(StackTraits::SUMMONED) && self.normal {
                if me.has(StackTraits::STRIKE_AND_RETURN) {
                    // The Harpies and Harpy Hags (72, 73 by number; the engine: strike and return).
                    only = v < p;
                    v = v.max(p);
                } else if v < 0 && self.stronger() {
                    only = true;
                    v = p;
                } else {
                    v = v.wrapping_add(p);
                }
            } else {
                v = v.wrapping_add(p);
            }
            if v < 0 && self.normal && p < self.score(plan.hex) {
                continue;
            }
            let r = i32::try_from(rng.range_inclusive(DRAW.0, DRAW.1)).unwrap_or(100);
            let v = r.wrapping_mul(v) / 100;
            if best.is_some_and(|b| !self.better(&b, (t, &plan), v)) {
                continue;
            }
            let turns = i32::try_from(plan.turns).unwrap_or(1);
            best = Some(Best {
                enemy,
                plan,
                value: v / turns,
                plan_value: p.wrapping_mul(r) / turns.wrapping_mul(100),
                only,
            });
        }
        *value = best.map_or(0, |b| b.value.max(b.plan_value));
        if self.teleport {
            // With the flag 1 (0x421C89): the best from its plan's hex when the value is not
            // below 0, else Defend; nothing else is asked.
            return Some(best.filter(|_| *value >= 0).map_or(Command::Defend, |b| {
                Command::Attack {
                    target: b.enemy,
                    from: b.plan.hex,
                }
            }));
        }
        if let Some(cast) = self.entry(value, rng) {
            return Some(cast);
        }
        let Some(b) = best else {
            // A predicted walker without a candidate plans nothing (its walks and fallbacks).
            return if self.predicts() {
                None
            } else if self.besieging() {
                Some(self.besiege())
            } else {
                self.fallback(None)
            };
        };
        if b.only {
            *value = b.plan_value;
            if b.plan.hex == me.hex {
                return None;
            }
            return self.walk(b.plan.hex, b.plan.turns > 1);
        }
        let losing = b.value < 0
            && b.value < b.plan_value
            && !me.has(StackTraits::SUMMONED)
            && self.normal
            && self.stronger();
        if losing {
            self.fallback(Some(&b))
        } else {
            self.attack(&b)
        }
    }

    /// The attack on the best candidate `b` (0x421EF8): within one turn a move and attack (a move
    /// alone where the original's walk stops early, but the move and attack in a prediction, which
    /// predicts the original's action); further, a careful walk towards the plan's hex (never the
    /// walker's own: its own hexes are no stops, so a plan there takes one turn; the original's
    /// test for it, 0x421F39, finds none).
    pub(crate) fn attack(&self, b: &Best) -> Option<Command> {
        if b.plan.turns <= 1 {
            // The original's move and attack stops where a step enters a stop and then skips the
            // attack (0x441FA0, 0x478971; §7.16): a two-hex walker whose other hex lies on one
            // at the plan's hex (the plan counts a stop by the position hex alone) only moves.
            let other = Shape::of(self.me).other(self.field, b.plan.hex);
            return Some(if self.map.is_stop(other) && !self.predicts() {
                Command::Move { to: b.plan.hex }
            } else {
                Command::Attack {
                    target: b.enemy,
                    from: b.plan.hex,
                }
            });
        }
        self.walk(b.plan.hex, true)
    }

    /// Whether the candidate `t` with `plan` and the drawn value `v` replaces `best` (0x421B2F to
    /// 0x421C16): one not blind, petrified or paralysed over one that is; then fewer turns, the
    /// higher value, more hit points lost by the top creature, and a cost to the plan's hex no
    /// higher than the best's (so the later record at a tie).
    fn better(&self, best: &Best, (t, plan): (&Stack, &Plan), v: i32) -> bool {
        let able = |s: &Stack| !s.spells.status.contains(Status::NO_TURNS);
        let lost = |s: &Stack| s.max_hp - s.top_hp;
        let b = self.stack(best.enemy);
        able(t)
            .cmp(&able(b))
            .then(plan.turns.cmp(&best.plan.turns).reverse())
            .then(v.cmp(&best.value))
            .then(lost(t).cmp(&lost(b)))
            .then(self.map.cost(best.plan.hex).cmp(&self.map.cost(plan.hex)))
            .is_ge()
    }

    /// The plan against `enemy` (0x4360C0, 0x436840; BAT-AI-005): the best hex to strike it from,
    /// [`Walker::around`] its position, a two-hex enemy's other hex and, for a two-hex walker,
    /// three hexes beyond the hex next to the enemy's side that faces it (left of it for a walker
    /// facing right, right for one facing left); none without such a hex.
    fn plan(&self, enemy: StackId) -> Option<Plan> {
        let (me, t) = (self.me, self.stack(enemy));
        let per = |s: &Stack, hp: u64| hp.div_ceil(u64::from(s.max_hp.max(1)));
        let own = per(me, self.ctx.left(self.state, self.id));
        let pair = (self.id, enemy);
        let dealt = computer::damage_estimate(self.state, self.ruleset, pair, own, false, 0);
        let left = per(t, self.ctx.left(self.state, enemy).saturating_sub(dealt));
        let mut best = None;
        let (shape, t_shape) = (Shape::of(me), Shape::of(t));
        let job = (enemy, own, left);
        self.around(&mut best, job, (t.hex, 0..6));
        if t.is_two_hex() {
            self.around(&mut best, job, (t_shape.other(self.field, t.hex), 0..6));
        }
        if me.is_two_hex() {
            // 0x436880: the enemy's hex on the walker's side when they face each other.
            let right = me.side == BattleSide::Attacker;
            let facing = t.is_two_hex() && (t.side == BattleSide::Attacker) != right;
            let at = if facing {
                t_shape.other(self.field, t.hex)
            } else {
                t.hex
            };
            let (d, dirs) = if right { (4, 3..6) } else { (1, 0..3) };
            // Off the field there is no such hex (0x4368D8): no more candidates.
            hexes::neighbor(self.field, shape, at, d)
                .into_iter()
                .for_each(|beyond| self.around(&mut best, job, (beyond, dirs.clone())));
        }
        best
    }

    /// The hexes around `h` in the original's directions `dirs` as the plan's candidates against
    /// `enemy` (0x436300; BAT-AI-005), the walker striking by `own` creatures and the enemy by
    /// `left` after it: a position the walker's map reaches standing there; its turns `⌈cost /
    /// speed⌉`, one more on a stop, at least 1, no more than the best's. Its value `P`: the shooters it would block ([`Walker::blocked`]); outside Easy
    /// for a computer side the multi-headed and breath strikes' values ([`Walker::heads`],
    /// [`Walker::breath`]) less those of the enemy's retaliation while it has one (the walker's
    /// strikes retaliated, the enemy not turned to stone); a two-hex walker's other hex's blocked
    /// shooters (one mask) and the lower of its hexes' scores, else the hex's score. The best:
    /// fewer turns, the higher `P`, the lower cost (for the jousters outside Easy the higher: a
    /// longer run-up), the earlier.
    fn around(
        &self,
        best: &mut Option<Plan>,
        (enemy, own, left): (StackId, u64, u64),
        (h, dirs): (Hex, core::ops::Range<u8>),
    ) {
        let (me, t) = (self.me, self.stack(enemy));
        let shape = Shape::of(me);
        let speed = u32::from(me.speed_now());
        let retaliates = !me.has(StackTraits::NO_ENEMY_RETALIATION)
            && !t.spells.status.contains(Status::PETRIFIED)
            && t.retaliations_left > 0;
        let jousts = self
            .ruleset
            .abilities(me.creature)
            .iter()
            .any(|a| matches!(a, Ability::Jousting { .. }));
        for d in dirs {
            let Some(n) = hexes::neighbor(self.field, shape, h, d) else {
                continue;
            };
            let Some(cost) = self.map.position(n).map(u32::from) else {
                continue;
            };
            // Without speed 1 at cost 0, else 100: only a war machine has none, and none walks.
            let turns = (cost.div_ceil(speed.max(1)) + u32::from(self.map.is_stop(n))).max(1);
            if best.is_some_and(|b| b.turns < turns) {
                continue;
            }
            let mut mask = 0;
            let mut p = self.blocked(n, &mut mask);
            if self.normal {
                let me_at = Striker {
                    id: self.id,
                    shape,
                    at: n,
                    count: own,
                };
                let them = Striker {
                    id: enemy,
                    shape: Shape::of(t),
                    at: t.hex,
                    count: left,
                };
                p = p
                    .wrapping_add(self.heads(self.side, me_at, (enemy, t.hex)))
                    .wrapping_add(self.breath(self.side, me_at, (enemy, t.hex)));
                if retaliates {
                    p = p
                        .wrapping_sub(self.heads(t.side, them, (self.id, n)))
                        .wrapping_sub(self.breath(t.side, them, (self.id, n)));
                }
            }
            let other = shape.other(self.field, n);
            if me.is_two_hex() {
                p = p.wrapping_add(self.blocked(other, &mut mask));
            }
            p = p.wrapping_add(self.score(n).min(self.score(other)));
            let take = best.is_none_or(|b| {
                b.turns != turns
                    || p.cmp(&b.value).then_with(|| {
                        let (c, kept) = (self.map.cost(n), self.map.cost(b.hex));
                        if jousts && self.normal {
                            c.cmp(&kept)
                        } else {
                            kept.cmp(&c)
                        }
                    }) == core::cmp::Ordering::Greater
            });
            if take {
                *best = Some(Plan {
                    hex: n,
                    turns,
                    value: p,
                });
            }
        }
    }

    /// The shooters a walker at `h` would block (0x436180; BAT-AI-005): 0 on Easy for a computer
    /// side; else over `h`'s neighbours, each stack not of the walker's side, not the walker and
    /// not in `mask` (now added to it) that can shoot now with the walker left out of its blockers
    /// counts `max(1, ⌊h × (w₁ − w₀) / hp⌋)`: its hit points times its worth with the shot flag
    /// less without, over its hit points per creature, as the original's doubles (a shooter the
    /// walker blocks already has no shot flag in `w₁`: 1).
    fn blocked(&self, h: Hex, mask: &mut u32) -> i32 {
        if !self.normal {
            return 0;
        }
        let shape = Shape::of(self.me);
        let double = |n: u64| X87::int(n).round(DOUBLE_BITS);
        (0..6)
            .filter_map(|d| {
                hexes::neighbor(self.field, shape, h, d).and_then(|n| self.on[n.index()])
            })
            .fold(0i32, |sum, x| {
                let s = self.stack(x);
                let bit = 1u32 << self.record(x);
                if s.side == self.side || x == self.id || *mask & bit != 0 {
                    return sum;
                }
                *mask |= bit;
                // In a prediction not one without hit points left after the planned damage.
                let left = self.ctx.left(self.state, x);
                if !computer::shoots_without(self.state, self.ruleset, x, self.id) || left == 0 {
                    return sum;
                }
                let w1 = computer::worth(self.state, self.ruleset, &self.ctx, x, true);
                let w0 = computer::worth(self.state, self.ruleset, &self.ctx, x, false);
                let gain = double(left)
                    .times(w1.minus(w0))
                    .over(double(s.max_hp.into()))
                    .unwrap_or(X87::ZERO);
                sum.wrapping_add(computer::ftol(gain).max(1))
            })
    }

    /// The value of a multi-headed strike of `striker` on the stack `t` at `t_hex` (0x436620;
    /// BAT-AI-005): 0 for a striker that is not multi-headed; else over the directions its strike
    /// hits from its position ([`hexes::pattern`]: every one, the Cerberi three), each stack there
    /// not of `side`, not `t` and not counted yet adds the value of the striker's damage on it
    /// (by its count, capped; the context's worth, no shot flag).
    fn heads(&self, side: BattleSide, striker: Striker, (t, t_hex): (StackId, Hex)) -> i32 {
        let s = self.stack(striker.id);
        if !s.has(StackTraits::ALL_ADJACENT) && !s.has(StackTraits::THREE_HEADED) {
            return 0;
        }
        let shape = (striker.shape, striker.at, s.has(StackTraits::THREE_HEADED));
        let pattern = hexes::pattern(self.field, shape, (Shape::of(self.stack(t)), t_hex));
        let mut counted = 1u32 << self.record(t);
        (0..8u8)
            .filter(|d| pattern & 1 << d != 0)
            .filter_map(|d| {
                hexes::stack_neighbor(self.field, striker.shape, striker.at, d)
                    .and_then(|n| self.on[n.index()])
            })
            .fold(0i32, |sum, x| {
                let bit = 1u32 << self.record(x);
                if self.stack(x).side == side || counted & bit != 0 {
                    return sum;
                }
                counted |= bit;
                let damage = self.capped(striker, x);
                let value = computer::damage_value(
                    self.state,
                    self.ruleset,
                    &self.ctx,
                    (x, damage),
                    (false, false),
                );
                sum.wrapping_add(value)
            })
    }

    /// The value of a breath strike of `striker` on the stack `t` at `t_hex` (0x436760;
    /// BAT-AI-005): 0 for a striker without breath; else the stack on the hex one further from
    /// the hex it strikes, in the direction it strikes ([`hexes::direction_to`]), unless it is
    /// `t`: the value of the striker's damage on it (by its count, capped; in the context's
    /// desperate mode at 1000 per creature), negative for a stack of `side` (the walker's own hexes
    /// count where it stands now).
    fn breath(&self, side: BattleSide, striker: Striker, (t, t_hex): (StackId, Hex)) -> i32 {
        if !self.stack(striker.id).has(StackTraits::BREATH) {
            return 0;
        }
        let target = (Shape::of(self.stack(t)), t_hex);
        hexes::direction_to(self.field, (striker.shape, striker.at), target)
            .and_then(|d| {
                hexes::stack_neighbor(self.field, striker.shape, striker.at, d)
                    .and_then(|first| hexes::neighbor(self.field, striker.shape, first, d))
            })
            .and_then(|h| self.on[h.index()])
            .filter(|&x| x != t)
            .map_or(0, |x| {
                let damage = self.capped(striker, x);
                let desperate = self.ctx.desperate();
                let value = computer::damage_value(
                    self.state,
                    self.ruleset,
                    &self.ctx,
                    (x, damage),
                    (false, desperate),
                );
                if self.stack(x).side == side {
                    value.wrapping_neg()
                } else {
                    value
                }
            })
    }

    /// The damage of `striker`'s strike on `x` (0x442780 by its creatures, no shot, capped; in a
    /// prediction at most `x`'s hit points left after the planned damage, 0x4366F7, 0x4367F0).
    fn capped(&self, striker: Striker, x: StackId) -> u64 {
        let pair = (striker.id, x);
        let damage =
            computer::damage_estimate(self.state, self.ruleset, pair, striker.count, false, 0);
        if self.predicts() {
            damage.min(self.ctx.left(self.state, x))
        } else {
            damage
        }
    }

    /// The computer's walk towards the position `to` with the hex scores ([`Walker::walk_with`]).
    pub(crate) fn walk(&self, to: Hex, careful: bool) -> Option<Command> {
        self.walk_with(to, careful, true)
    }

    /// The computer's walk towards the position `to` (0x41F580; BAT-AI-006): nothing without a
    /// path ([`computer::path`]; for a bound walker or one without speed, the plan's hex of such
    /// a walker is its own and the fallbacks find no other). Along the path, at most its speed in
    /// steps and ending after a stop it enters (a flyer has none), each position it stands on may
    /// become the end. With the scores (`scored`; the guard's walk has none, BAT-AI-010): with
    /// `careful` on Hard and above or for a human side (the walker's own side), outside the wait
    /// phase, every end must score no lower than the last end (both hexes of a two-hex walker; its
    /// start's score first), else only from the step that leaves at most `s × ⌊(n − 1) / s⌋` of
    /// the path's `n` steps once an end was found, so it first reaches the hex from which the rest
    /// takes whole turns. Then, careful: an end short of the last hex reached with a score no
    /// lower than the start's is Wait; an end at its own position Wait, else Defend; any other end
    /// a move there. Nothing in a prediction.
    // rule: BAT-AI-006
    pub(crate) fn walk_with(&self, to: Hex, careful: bool, scored: bool) -> Option<Command> {
        // A predicted walk plans nothing and draws nothing (ADR-0027 point 11: not built).
        if self.predicts() {
            return None;
        }
        let me = self.me;
        let speed = usize::from(me.speed_now());
        // The walk's search is the pathfinder's last now (survey slip 30), its stops the walker's
        // own; a bound walker or one without speed searches nothing (0x41F58D to 0x41F5A7).
        if speed > 0 && !computer::bound(self.state, self.id) {
            self.after_map.set(None);
        }
        let path = computer::path(self.state, self.ruleset, (self.id, self.side), to);
        let steps = path.steps();
        let n = steps.len();
        let hard = computer::difficulty(self.state) >= 2 || computer::human(self.state, me.side);
        let careful = scored && careful && hard && !computer::wait_phase(self.state);
        let shape = Shape::of(me);
        let start = self
            .score(me.hex)
            .min(self.score(shape.other(self.field, me.hex)));
        let whole = speed * (n.saturating_sub(1) / speed.max(1));
        let (mut end, mut found, mut kept, mut last) = (me.hex, false, start, me.hex);
        for (k, &(hex, flown)) in steps.iter().take(speed).enumerate() {
            last = hex;
            if flown {
                continue;
            }
            let other = shape.other(self.field, hex);
            let tested = careful || (scored && n - k <= whole && found);
            if !tested || (self.score(hex) >= kept && self.score(other) >= kept) {
                (end, found, kept) = (hex, true, self.score(hex).min(self.score(other)));
            }
            if !me.has(StackTraits::FLIES) && (path.is_stop(hex) || path.is_stop(other)) {
                break;
            }
        }
        let command = if end == me.hex {
            if careful {
                Command::Wait
            } else {
                Command::Defend
            }
        } else if careful && end != last && kept >= start {
            Command::Wait
        } else {
            Command::Move { to: end }
        };
        // No path (the walker's map reaches the plan's hex, so its walk finds one): nothing.
        (n > 0).then_some(command)
    }
}
