//! The walker's ends (`docs/formats/installation.md` §7.35, §7.38): what a computer walker does
//! without a candidate or instead of a losing attack, and the sieges' two branches.
//!
//! - **Holding the walls** (0x4213F0; BAT-AI-011): a defender at a fort whose walls and gate are
//!   whole, that is the stronger at range and whose living stacks but the turrets all stand inside
//!   the walls takes only the enemies inside them as candidates. The walls test follows
//!   BAT-QUIRK-25 (slip 2): the original looks up each destroyed part's wall hex by its part number
//!   as a target index, so a destroyed Mid-Upper or Lower Wall gives hex 12, which always stays
//!   blocked, and never counts as a breach, while the Upper Wall's gives no hex (the display byte it
//!   reads instead, the movement overlay of hex 93, is taken as clear: a hidden battle's) and the
//!   gate's and the Mid-Lower Wall's give hex 0, a side column, never blocked at a fort: these
//!   count. Without the slip every destroyed wall or gate counts.
//! - **The besieger's walk** (0x421D46 to 0x421E2D; BAT-AI-005): an attacker's walker at a fort
//!   without a candidate walks to its own row's wall: from the row's wall hex leftwards to its own
//!   position, the first hex its map reached that is not in the moat (for a two-hex walker neither
//!   hex): the open wall hex after a breach in that row, else the hex before the moat; none, or a
//!   walker at or right of the wall, defends.
//! - **The fallbacks** (BAT-AI-010): with no area threat on the walker (0x41F920), guarding a
//!   shooter (0x420760, 0x4205D0); then the attack after all if there is a candidate; else, on
//!   Hard and above or for a human side, the step to safety (0x4208F0).

use h3_battle::computer;
use h3_battle::{BattleSide, Command, Hex, Stack, StackId, StackTraits, Status};
use h3_content::ORIGINAL_AI_SLIPS;
use h3_content_battle::WallKind;
use h3_contracts::FortLevel;

use crate::hexes::{self, Shape};
use crate::scores;
use crate::walker::{Best, Walker};

/// The targets (the original's numbering, the siege field's order) whose destruction the walls
/// test never counts under BAT-QUIRK-25 (slip 2): the Mid-Upper and the Lower Wall, whose parts'
/// numbers (8, 12) as target indices read records past the table that give hex 12.
const UNCOUNTED_BREACHES: [u8; 2] = [2, 5];

/// A hex's screen anchor x on the original's battlefield (the hex record's +0): 44 pixels a hex,
/// even rows half a hex to the right (BAT-MAG-006's geometry).
fn anchor_x(w: &Walker<'_>, hex: Hex) -> i32 {
    let (col, row) = w.field.coords(hex);
    44 * col + if row % 2 == 0 { 22 } else { 0 }
}

/// The hex index `hex` ± 1 by the facing of a stack of `shape` (0x4466A0's arithmetic, a two-hex
/// stack's other hex), wrapping within the score table.
fn beside(shape: Shape, hex: Hex) -> usize {
    let delta = if shape.faces_right() { 1 } else { u8::MAX };
    usize::from(hex.0.wrapping_add(delta))
}

/// Whether target `t` of `kind` breaches the walls for the walls test (0x42141D to 0x421468;
/// BAT-AI-011): a wall or the gate that no longer stands, but under BAT-QUIRK-25 (`slips`, slip 2)
/// not the Mid-Upper or the Lower Wall.
pub(crate) fn breaches((t, kind): (u8, Option<WallKind>), stands: bool, slips: bool) -> bool {
    matches!(kind, Some(WallKind::Wall | WallKind::Gate))
        && !stands
        && !(slips && UNCOUNTED_BREACHES.contains(&t))
}

impl Walker<'_> {
    /// Whether the walker's side holds the walls (0x4213F0; BAT-AI-011).
    // rule: BAT-AI-011
    pub(crate) fn holds_walls(&self) -> bool {
        let Some(siege) = computer::siege(self.state).filter(|s| s.fort > FortLevel::None) else {
            return false;
        };
        if self.side != BattleSide::Defender {
            return false;
        }
        let slips = self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
        let breached =
            (0..siege.targets).any(|t| breaches((t, siege.kind(t)), siege.stands(t), slips));
        if breached || !self.stronger() {
            return false;
        }
        self.state
            .stacks()
            .iter()
            .filter(|s| s.side == BattleSide::Defender && s.is_alive() && !s.is_turret())
            .all(|s| siege.inside(s.hex))
    }

    /// Whether enemy `t` stands inside the walls (0x467460 at its position, or at a two-hex
    /// enemy's other hex).
    pub(crate) fn inside_walls(&self, t: &Stack) -> bool {
        computer::siege(self.state).is_some_and(|siege| {
            siege.inside(t.hex)
                || (t.is_two_hex() && siege.inside(Shape::of(t).other(self.field, t.hex)))
        })
    }

    /// Whether the walker besieges: its side is the attacker at a town with a fort (0x421D51:
    /// +0x132F4 above 0, the acting side 0).
    pub(crate) fn besieging(&self) -> bool {
        self.side == BattleSide::Attacker
            && computer::siege(self.state).is_some_and(|s| s.fort > FortLevel::None)
    }

    /// The besieger's walk to its row's wall (0x421D5F to 0x421E2D; BAT-AI-005): the walk with the
    /// scores (no flag) to the first hex from the row's wall hex leftwards that its map reached and
    /// that is not in the moat, neither hex of a two-hex walker (the other by the index); Defend
    /// without one, at or right of the wall, or without a path (the original leaves its last
    /// action there).
    // rule: BAT-AI-005
    pub(crate) fn besiege(&self) -> Command {
        let me = self.me;
        let (_, row) = self.field.coords(me.hex);
        let wall = self
            .ruleset
            .siege_field()
            .and_then(|f| f.walls.get(usize::try_from(row).unwrap_or(0)).copied());
        let moat = computer::siege(self.state)
            .map(|s| s.moat())
            .unwrap_or_default();
        let shape = Shape::of(me);
        let in_moat = |i: usize| u8::try_from(i).is_ok_and(|h| moat.contains(Hex(h)));
        let free = |h: u8| {
            let hex = Hex(h);
            self.map.reached(hex)
                && !in_moat(hex.index())
                && !(me.is_two_hex() && in_moat(beside(shape, hex)))
        };
        let to = wall
            .and_then(|w| (me.hex.0.saturating_add(1)..=w).rev().find(|&h| free(h)))
            .map(Hex);
        to.and_then(|to| self.walk_with(to, false, true))
            .unwrap_or(Command::Defend)
    }

    /// The fallbacks (0x421E84 to 0x421EF5; BAT-AI-010) without a candidate (`best` none) or
    /// instead of a losing attack on `best`: with no area threat on the walker (0x41F920 with the
    /// reach's mask; [`scores::area_threat`]), guarding a shooter ([`Walker::guard`]); then the
    /// attack on `best` after all; else the step to safety ([`Walker::step_to_safety`]); else
    /// nothing.
    // rule: BAT-AI-010
    pub(crate) fn fallback(&self, best: Option<&Best>) -> Option<Command> {
        let threat = scores::area_threat(
            self.state,
            self.ruleset,
            (self.id, self.side),
            &self.ctx,
            self.mask,
        );
        if threat == 0
            && let Some(command) = self.guard()
        {
            return Some(command);
        }
        match best {
            Some(b) => self.attack(b),
            None => self.step_to_safety(),
        }
    }

    /// The turns the walker needs to reach `hex` (0x4B3F20): its map's cost over its speed,
    /// rounded up, at least 1.
    fn turns_to(&self, hex: Hex) -> u32 {
        let speed = u32::from(self.me.speed_now()).max(1);
        u32::from(self.map.cost(hex)).div_ceil(speed).max(1)
    }

    /// Guarding a shooter (0x420760; BAT-AI-010): over the walker's side's living stacks but the
    /// turrets and the Catapult that can shoot now and have the melee penalty, in record order,
    /// each with its guard hex ([`Walker::guard_hex`]): the fewest turns to it, then the higher
    /// strength (0x442E60) over its open sides when more than one, then the later stack. Defend
    /// on the walker's own position, else the walk there without scores (Defend without a path).
    // rule: BAT-AI-010
    pub(crate) fn guard(&self) -> Option<Command> {
        let mut best: Option<(Hex, u32, i32)> = None;
        for (i, r) in (0u8..).zip(self.state.stacks()) {
            let id = StackId(i);
            if r.side != self.side
                || !r.is_alive()
                || r.is_turret()
                || r.is_catapult()
                || r.has(StackTraits::NO_MELEE_PENALTY)
                || !computer::shoots(self.state, self.ruleset, id)
            {
                continue;
            }
            let Some((hex, open)) = self.guard_hex(r) else {
                continue;
            };
            let turns = self.turns_to(hex);
            let mut value = computer::strength(self.state, self.ruleset, &self.ctx, id);
            if open > 1 {
                value /= open;
            }
            if best.is_some_and(|(_, t, v)| turns > t || (turns == t && v > value)) {
                continue;
            }
            best = Some((hex, turns, value));
        }
        let (hex, ..) = best?;
        Some(if hex == self.me.hex {
            Command::Defend
        } else {
            self.walk_with(hex, false, false).unwrap_or(Command::Defend)
        })
    }

    /// The guard hex beside the shooter `r` (0x4205D0): of `r`'s neighbours (six, eight for a
    /// two-hex `r`; 0x523DF0) that are free or hold the walker, counted as its open sides, those
    /// the walker's map reached: the fewest turns, then more of the walker's hexes touching `r`
    /// (2 for a two-hex walker whose other hex, by the index, touches it too), then the hex
    /// further the way `r` faces (its screen anchor's x: the larger facing right, the smaller
    /// facing left), the later direction at an equal x. With the open sides; `None` without one.
    fn guard_hex(&self, r: &Stack) -> Option<(Hex, i32)> {
        let shape = Shape::of(r);
        let me = Shape::of(self.me);
        let around = |h: usize| {
            u8::try_from(h).ok().is_some_and(|h| {
                r.footprint(self.field)
                    .hexes()
                    .any(|rh| self.field.neighbors(rh).contains(&Some(Hex(h))))
            })
        };
        let mut open = 0;
        let mut best: Option<(Hex, u32, u8)> = None;
        for d in 0..hexes::directions(shape) {
            let Some(h) = hexes::stack_neighbor(self.field, shape, r.hex, d) else {
                continue;
            };
            if self.on[h.index()].is_some_and(|x| x != self.id) {
                continue;
            }
            open += 1;
            if !self.map.reached(h) {
                continue;
            }
            let turns = self.turns_to(h);
            let touches = 1 + u8::from(me.two_hex && around(beside(me, h)));
            let take = best.is_none_or(|(b, t, k)| {
                let further = if shape.faces_right() {
                    anchor_x(self, h) >= anchor_x(self, b)
                } else {
                    anchor_x(self, h) <= anchor_x(self, b)
                };
                turns < t || (turns == t && (touches > k || (touches == k && further)))
            });
            if take {
                best = Some((h, turns, touches));
            }
        }
        best.map(|(h, ..)| (h, open))
    }

    /// The step to safety (0x4208F0; BAT-AI-010): on Hard and above or for a human side (the
    /// walker's own side), a walker whose own score is below 0 (the lower of a two-hex walker's
    /// hexes, its other by the index) and that is not blind, petrified, paralysed or bound walks
    /// with the scores (no flag) to the position its map reaches within its speed with the highest
    /// score above its own: at an equal score the lower cost, then the later hex. Nothing without
    /// one.
    // rule: BAT-AI-010
    pub(crate) fn step_to_safety(&self) -> Option<Command> {
        let me = self.me;
        let hard = computer::difficulty(self.state) >= 2 || computer::human(self.state, me.side);
        let shape = Shape::of(me);
        let score = |h: Hex| {
            let own = self.scores[h.index()];
            if me.is_two_hex() {
                own.min(self.scores[beside(shape, h)])
            } else {
                own
            }
        };
        let own = score(me.hex);
        if !hard
            || own >= 0
            || me.spells.status.contains(Status::NO_TURNS)
            || computer::bound(self.state, self.id)
        {
            return None;
        }
        let speed = u16::from(me.speed_now());
        let mut best = (own, 0u16, None);
        for h in (0u8..).take(self.field.hex_count()).map(Hex) {
            let Some(cost) = self.map.position(h).filter(|&c| c <= speed) else {
                continue;
            };
            let s = score(h);
            if s < best.0 || (s == best.0 && best.1 < cost) {
                continue;
            }
            best = (s, cost, Some(h));
        }
        let to = best.2.filter(|&h| h != me.hex)?;
        self.walk_with(to, false, true)
    }
}
