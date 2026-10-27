//! Rounds and turn order (BAT-TRN-001..003), regeneration (BAT-TRN-005) and the morale checks
//! around a turn (BAT-MOR-003/004).
//!
//! A round has three phases. In the normal phase stacks act from fastest to slowest; then stacks
//! that waited during a good-morale turn, then the other stacks that waited, each from slowest to
//! fastest. Speed ties go to the side that did not act last (the attacker at the start of the
//! battle), then to the lower army slot (war machines after the army, then the stacks created in
//! the battle, by id: BAT-MAG-037). A blind stack's turn starts and passes (BAT-MAG-024).
//! When every living stack has acted or passed blind, a new round starts and per-round flags are
//! reset.

use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::FixedVec;

use crate::effects::Status;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::observe::Observation;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits};
use crate::{control, effects, morale, obstacles, war};

/// Whether stack `s` may still be picked this round: alive, not done, not an ammunition supply
/// (they never act, BAT-WAR-004) and not passed over losing its turns.
fn pickable(s: &Stack) -> bool {
    pickable_as(
        s,
        s.flags.contains(StackFlags::ACTED),
        s.flags.contains(StackFlags::STARTED),
    )
}

/// [`pickable`] with the stack's acted and started marks as given (a new round clears both): a
/// stack that loses its turns (blind, petrified, paralysed) and whose turn has passed this round
/// is passed over until the effect ends (BAT-MAG-024, BAT-MAG-043).
fn pickable_as(s: &Stack, acted: bool, started: bool) -> bool {
    s.is_alive()
        && !acted
        && !s.has(StackTraits::AMMO_SUPPLY)
        && !(started && s.spells.status.contains(Status::NO_TURNS))
}

/// The side the speed tie-break prefers after `last_side_acted` (BAT-TRN-001): the other one, the
/// attacker before any turn.
fn preferred(last_side_acted: Option<BattleSide>) -> BattleSide {
    last_side_acted.map_or(BattleSide::Attacker, BattleSide::opponent)
}

/// A stack's key in the turn order (BAT-TRN-001, BAT-TRN-002): smaller keys act earlier. The side
/// tie-break changes with every turn, so it is applied by [`OrderKey::rank`].
#[derive(Debug, Clone, Copy)]
struct OrderKey {
    /// Phase, turret, Catapult, speed.
    head: (u8, bool, bool, u8),
    side: BattleSide,
    /// The army slot, then the id.
    tail: (u8, u8),
    /// The side the tie-break counts as activated last after the stack's turn ([`acted_side`]).
    acted: BattleSide,
}

impl Default for OrderKey {
    fn default() -> Self {
        Self {
            head: (0, false, false, 0),
            side: BattleSide::Attacker,
            tail: (0, 0),
            acted: BattleSide::Attacker,
        }
    }
}

impl OrderKey {
    /// The key of stack `s` with id `id`, the round's wait phase begun if `wait_phase`.
    fn new(id: u8, s: &Stack, wait_phase: bool) -> Self {
        // Once the wait phase has begun every stack not done is a waited one (the original's
        // pick clears the waited bits there and orders all alike, 0x4652A7, 0x464FDB): a stack
        // created in the wait phase takes its place by its speed, the slower first (batch 2g
        // WP: the Pikemen before the elementals summoned there).
        let waited = s.flags.contains(StackFlags::WAITED) || wait_phase;
        Self::with(id, s, waited, s.flags.contains(StackFlags::HAD_MORALE))
    }

    /// The key of stack `s` with id `id` as a waited stack if `waited`, one that had good morale
    /// this round if `had_morale` (a new round clears both).
    fn with(id: u8, s: &Stack, waited: bool, had_morale: bool) -> Self {
        // Phases: normal, waited during a good-morale turn, waited (BAT-MOR-003).
        let phase = match (waited, had_morale) {
            (false, _) => 0,
            (true, true) => 1,
            (true, false) => 2,
        };
        // Smaller keys act earlier. Normal phase: faster first; wait phases: slower first.
        let speed = if phase == 0 {
            u8::MAX - s.speed_now()
        } else {
            s.speed_now()
        };
        // rule: BAT-SIE-011, BAT-SIE-016
        // The turrets, then the Catapult before every other stack of their phase (the
        // original's 0x464F50); the turrets by slot: the keep, the upper, the lower tower.
        let turret = !s.is_turret();
        let catapult = !s.is_catapult();
        Self {
            head: (phase, turret, catapult, speed),
            side: s.side,
            tail: (s.turn_slot(), id),
            acted: acted_side(s),
        }
    }

    /// The full key with the side tie-break for `preferred`; the trailing id makes it unique, so
    /// the order is total (ADR-0003 §9).
    fn rank(self, preferred: BattleSide) -> ((u8, bool, bool, u8), bool, (u8, u8)) {
        (self.head, self.side != preferred, self.tail)
    }
}

/// The next stack to act this round, or `None` if every living stack has acted or passed its turn
/// losing it.
// rule: BAT-TRN-001, BAT-TRN-002
pub(crate) fn next_active(state: &BattleState) -> Option<StackId> {
    let preferred = preferred(state.last_side_acted);
    (0u8..)
        .zip(state.stacks.iter())
        .filter(|(_, s)| pickable(s))
        .min_by_key(|(id, s)| OrderKey::new(*id, s, state.wait_phase).rank(preferred))
        .map(|(id, _)| StackId(id))
}

/// The turn order as the battle stands (BAT-TRN-001 to BAT-TRN-003), for viewers and agents: the
/// stacks that take a turn in the rest of the round, the active stack first, then every living
/// stack's turn in the next round, in the order [`next_active`]'s picks give them if each stack
/// takes its turn as it comes. It foresees nothing a turn may change: a wait, good or bad
/// morale, Fear, a death, an effect that begins or ends at a turn or the round's start, a stack
/// the battle creates. A stack that loses its turns (blind, petrified, paralysed: BAT-MAG-024,
/// BAT-MAG-043) has no place while it does, and an ammunition supply none (BAT-WAR-004). Within
/// the tactics phase the round is the first; once the battle is over both are empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnOrder {
    /// The stacks to take a turn in the rest of the round, the active stack first.
    pub round: FixedVec<StackId, STACK_CAPACITY>,
    /// The stacks to take a turn in the next round, in its order.
    pub next: FixedVec<StackId, STACK_CAPACITY>,
}

/// The turn order of the battle `obs` shows ([`TurnOrder`]). It reads only the observation's
/// public part, so it is the same for every viewer.
// rule: BAT-TRN-001, BAT-TRN-002, BAT-TRN-003
pub fn turn_order(obs: &Observation) -> TurnOrder {
    let mut order = TurnOrder::default();
    if obs.result.is_some() {
        return order;
    }
    let stacks = obs.stacks.as_slice();
    let mut last = obs.last_side_acted;
    if obs.tactics.is_some() {
        // The phase's placements are no turns: its end starts the first round (BAT-TAC-002).
        let first = keys(stacks, false, true);
        last = fill(&mut order.round, first.as_slice(), 0, last);
    } else {
        let mut done = 0u64;
        if let Some((active, s)) = obs
            .active
            .and_then(|a| stacks.get(a.index()).map(|s| (a, s)))
        {
            push(&mut order.round, active);
            done |= 1u64 << active.0;
            last = Some(acted_side(s));
        }
        let round = keys(stacks, obs.wait_phase, false);
        last = fill(&mut order.round, round.as_slice(), done, last);
    }
    let next = keys(stacks, false, true);
    fill(&mut order.next, next.as_slice(), 0, last);
    order
}

/// The keys of the stacks that may take a turn, sorted by their key without the side tie-break
/// (unique: it ends with the id): in a new round (`fresh`) with the marks its start clears
/// (BAT-TRN-003), else in this round, its wait phase begun if `wait_phase`. A stack that loses
/// its turns passes without one, so it takes no place.
fn keys(stacks: &[Stack], wait_phase: bool, fresh: bool) -> FixedVec<OrderKey, STACK_CAPACITY> {
    let mut out = FixedVec::new();
    for (id, s) in (0u8..).zip(stacks) {
        let takes = if fresh {
            pickable_as(s, false, false)
        } else {
            pickable(s)
        };
        if takes && !s.spells.status.contains(Status::NO_TURNS) {
            let key = if fresh {
                OrderKey::with(id, s, false, false)
            } else {
                OrderKey::new(id, s, wait_phase)
            };
            // At most STACK_CAPACITY stacks.
            let _ = out.push(key);
        }
    }
    out.as_mut_slice()
        .sort_unstable_by_key(|k| (k.head, k.tail));
    out
}

/// Appends to `out` the stacks of `keys` (sorted as [`keys`] sorts them) not in `done` (a bit per
/// id) in the order the picks take them, the tie-break starting from `last`; returns the
/// tie-break's side after them. Each pick is the first left of the smallest head, its preferred
/// side's first if it has one there: the smallest [`OrderKey::rank`].
fn fill(
    out: &mut FixedVec<StackId, STACK_CAPACITY>,
    keys: &[OrderKey],
    mut done: u64,
    mut last: Option<BattleSide>,
) -> Option<BattleSide> {
    let left = |done: u64| keys.iter().filter(move |k| done & (1u64 << k.tail.1) == 0);
    while let Some(first) = left(done).next() {
        let side = preferred(last);
        let pick = left(done)
            .take_while(|k| k.head == first.head)
            .find(|k| k.side == side)
            .unwrap_or(first);
        push(out, StackId(pick.tail.1));
        done |= 1u64 << pick.tail.1;
        last = Some(pick.acted);
    }
    last
}

/// Appends `id` to `out`, which has room for every stack.
fn push(out: &mut FixedVec<StackId, STACK_CAPACITY>, id: StackId) {
    let _ = out.push(id);
}

/// Starts a new round, not in its wait phase (BAT-MAG-056): each side's rounds since its periodic
/// cast rise (BAT-MAG-053); resets retaliation (with Counterstrike's extra ones, BAT-MAG-015),
/// wait, morale, started and acted flags (the Defend bonus lasts until the stack's next
/// activation, BAT-TRN-004) and the heroes' casts (BAT-MAG-001); then the timed effects lose a
/// round or end (BAT-MAG-010), the clones lose a round or vanish (BAT-MAG-038), then the spell
/// obstacles (BAT-MAG-027). A clone's original lives, but with quirk BAT-QUIRK-7 a stack that
/// became its own clone can vanish as its side's last one: the battle's end is checked after the
/// clones (BAT-END-001; the original asks for the winner after every action, 0x479BDC).
// rule: BAT-TRN-003, BAT-MAG-001
pub(crate) fn start_round<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, sink: &mut S) {
    state.round = state.round.saturating_add(1);
    state.wait_phase = false;
    // rule: BAT-MAG-053
    crate::casters::round_started(state);
    for hero in state.heroes.iter_mut().flatten() {
        hero.view.casts = 0;
    }
    for s in state.stacks.as_mut_slice() {
        s.flags.set(StackFlags::ACTED, false);
        s.flags.set(StackFlags::WAITED, false);
        s.flags.set(StackFlags::RETALIATED, false);
        s.flags.set(StackFlags::HAD_MORALE, false);
        s.flags.set(StackFlags::STARTED, false);
        s.retaliations_left = s.retaliations;
    }
    effects::round_retaliations(state, ruleset);
    sink.emit(Event::public(EventKind::RoundStarted {
        round: state.round,
    }));
    effects::count_down(state, ruleset, sink);
    // rule: BAT-MAG-038
    crate::created::count_down(state, ruleset, sink);
    // rule: BAT-END-001
    crate::combat::check_end(state, sink);
    obstacles::count_down(state, sink);
}

/// The side the tie-break counts as activated last after stack `s`'s turn (BAT-TRN-001): its
/// own, but the defender after the Catapult's (BAT-SIE-011): the original's pick writes the
/// defender's record 0 as the last activated once that was the attacker's Catapult (0x46508B to
/// 0x4650C5), so at equal speeds the attacker's stack acts first after it.
// rule: BAT-TRN-001, BAT-SIE-011
fn acted_side(s: &Stack) -> BattleSide {
    if s.is_catapult() {
        BattleSide::Defender
    } else {
        s.side
    }
}

/// How a turn ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnEnd {
    /// The stack moved, attacked, shot or cast (BAT-MAG-048).
    Acted,
    /// The stack waited (BAT-TRN-002).
    Waited,
    /// The stack defended (BAT-TRN-004).
    Defended,
}

/// Ends `mover`'s turn. After moving, attacking, shooting or casting, good morale may give it another turn
/// right away (BAT-MOR-003; a random caster draws its spell again, BAT-MAG-049; see
/// [`morale_turn`]); otherwise it is marked as waited or acted. Then the next stack is activated,
/// starting a new round if needed. Once the battle is over no stack is active.
pub(crate) fn end_turn<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    mover: StackId,
    how: TurnEnd,
    sink: &mut S,
) {
    // rule: BAT-SIE-003
    crate::siege::close_gate(state, sink);
    state.last_side_acted = Some(acted_side(state.at(mover)));
    // rule: BAT-MOR-003
    if state.result.is_none() && how == TurnEnd::Acted && morale::good_morale(state, ruleset, mover)
    {
        if morale_turn(state, ruleset, mover, sink) {
            activation(state, ruleset, mover, sink);
            // rule: BAT-MAG-049
            crate::casters::activated(state, ruleset, mover, sink);
        } else {
            activate_next(state, ruleset, sink);
        }
        return;
    }
    let flag = if how == TurnEnd::Waited {
        StackFlags::WAITED
    } else {
        StackFlags::ACTED
    };
    state.at_mut(mover).flags.set(flag, true);
    if state.result.is_some() {
        state.active = None;
        return;
    }
    activate_next(state, ruleset, sink);
}

/// The good-morale turn of stack `id`, whose action just brought it good morale (BAT-MOR-003): the
/// roll lifts its acted mark, as the original's 0x464920 clears the done bit (which the rebirth
/// sets, BAT-MAG-059: a Phoenix reborn in its own action keeps its turn). A stack its action left
/// blind, petrified or paralysed is passed over until that ends, as at any pick (0x465222: it may
/// still act in the round, BAT-MAG-024); the turn of one Fear freezes outside the wait phase is
/// lost (BAT-MAG-060; the original picks the stack again with its checks, bad morale being
/// impossible with good morale). Returns whether the stack takes the turn now.
// rule: BAT-MOR-003
fn morale_turn<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) -> bool {
    let flags = &mut state.at_mut(id).flags;
    flags.set(StackFlags::ACTED, false);
    flags.set(StackFlags::HAD_MORALE, true);
    sink.emit(Event::public(EventKind::GoodMorale { stack: id }));
    if state.at(id).spells.status.contains(Status::NO_TURNS) {
        return false;
    }
    sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
    // rule: BAT-MAG-060
    if morale::frightened(state, ruleset, id) {
        sink.emit(Event::public(EventKind::Frightened { stack: id }));
        state.at_mut(id).flags.set(StackFlags::ACTED, true);
        return false;
    }
    true
}

/// The next stack to act, starting a new round if every living stack has acted; none once the
/// round's start ended the battle (a side's last stack vanished, BAT-MAG-038).
fn pick<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    sink: &mut S,
) -> Option<StackId> {
    next_active(state).or_else(|| {
        start_round(state, ruleset, sink);
        next_active(state).filter(|_| state.result.is_none())
    })
}

/// Activates the next stack: its Defend bonus ends (BAT-TRN-004); at its first turn of the round
/// it regenerates (BAT-TRN-005) and its effects counted by its own turns count down
/// (BAT-MAG-019); a blind stack then passes without acting (BAT-MAG-024), and bad morale
/// (BAT-MOR-004) or Fear (BAT-MAG-060) may freeze it, in which case it loses its turn; either way
/// the next stack is activated, as after a war machine that acts on its own (BAT-WAR-002/003; a
/// commanded one waits for its side's command, BAT-WAR-006) and after a berserk stack's
/// automatic turn (BAT-MAG-025; unless good morale gives it, no longer berserk, another turn).
/// While the battle is not over some stack is alive, so a stack is always found. The stack that
/// awaits its side's command draws its spell if it is a random caster (BAT-MAG-049). Once a
/// waited stack is activated, the round's wait phase has begun (BAT-MAG-056).
pub(crate) fn activate_next<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    sink: &mut S,
) {
    activate(state, ruleset, true, sink);
}

/// Activates the battle's first stack without a tactics phase as [`activate_next`] does, but
/// without the bad-morale and Fear checks: the original's battle start picks it with them off
/// (0x462A20's call of 0x465080 with argument 0; after a tactics phase they run, BAT-MOR-004).
pub(crate) fn activate_first<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    sink: &mut S,
) {
    activate(state, ruleset, false, sink);
}

/// [`activate_next`], with the freezing checks if `checks`.
fn activate<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    mut checks: bool,
    sink: &mut S,
) {
    let mut next = pick(state, ruleset, sink);
    while let Some(id) = next {
        // The stacks the last action killed lie below any killed later (BAT-MAG-035): automatic
        // turns may follow within one command.
        crate::raise::mark_corpses(state);
        // A waited stack acts once no other is left: the round's wait phase has begun (the
        // original's +0x13DE4, BAT-MAG-056).
        state.wait_phase |= state.at(id).flags.contains(StackFlags::WAITED);
        let waiting = state.wait_phase;
        let stack = state.at_mut(id);
        // rule: BAT-TRN-004
        stack.flags.set(StackFlags::DEFENDING, false);
        // The turn start runs at a stack's first activation of the round outside the wait phase:
        // the original's pick runs none there (0x465211 to 0x46521D), so a stack first picked
        // in the wait phase (created there) gets none that round (batch 2g TS: waited Wights
        // wounded after their wait do not regenerate in the wait phase).
        let first = !stack.flags.contains(StackFlags::STARTED) && !waiting;
        stack.flags.set(StackFlags::STARTED, true);
        sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
        if first {
            regenerate(state, id, sink);
            effects::own_turn(state, ruleset, id, sink);
        }
        // rule: BAT-MAG-024, BAT-MAG-043
        if state.at(id).spells.status.contains(Status::NO_TURNS) {
            sink.emit(Event::public(EventKind::TurnPassed { stack: id }));
            next = pick(state, ruleset, sink);
            continue;
        }
        // A frozen stack goes back to the pick without an activation (0x465283 to 0x46529C):
        // the speed tie-break still goes by the side that acted before it (BAT-TRN-001; batch
        // 2g FT: after Blue's Griffins and Red's frozen Wolf Riders, Red's Demons before Blue's
        // Medusas of equal speed).
        let mut activated = true;
        // rule: BAT-MOR-004
        if checks && morale::bad_morale(state, ruleset, id) {
            sink.emit(Event::public(EventKind::BadMorale { stack: id }));
            state.at_mut(id).flags.set(StackFlags::ACTED, true);
            activated = false;
        // rule: BAT-MAG-060
        } else if checks && morale::frightened(state, ruleset, id) {
            sink.emit(Event::public(EventKind::Frightened { stack: id }));
            state.at_mut(id).flags.set(StackFlags::ACTED, true);
            activated = false;
        } else if begins(state, ruleset, id, sink) {
            break;
        }
        // Later picks follow an action: the original's dispatcher asks with the checks on.
        checks = true;
        // rule: BAT-TRN-001
        if activated {
            state.last_side_acted = Some(acted_side(state.at(id)));
        }
        // A war machine's shot can end the battle.
        next = if state.result.is_some() {
            None
        } else {
            pick(state, ruleset, sink)
        };
    }
    state.active = next;
    // rule: BAT-MAG-049
    if let Some(id) = next {
        crate::casters::activated(state, ruleset, id, sink);
    }
}

/// What the original's activation (0x465330) does for a stack it activates for a turn, after the
/// checks that pass a stack over (blind, petrified, paralysed; bad morale, Fear): at its side's
/// first activation the hero's opening spells (BAT-MAG-068), then a Wraith drains its enemy
/// hero's mana (BAT-MAG-056), an Enchanter may cast (BAT-MAG-053).
// rule: BAT-MAG-053, BAT-MAG-056, BAT-MAG-068
fn activation<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) {
    crate::magic::opening(state, ruleset, id, sink);
    crate::mana::drain(state, ruleset, id, sink);
    crate::casters::periodic(state, ruleset, id, sink);
}

/// The turn of stack `id`, activated without bad morale: at its side's first activation the
/// opening spells are cast (BAT-MAG-068), a draining stack drains and a periodic caster casts
/// first, as the original's activation does for any stack it activates (BAT-MAG-056,
/// BAT-MAG-053); then a war machine that
/// acts on its own acts (BAT-WAR-002/003) and a berserk stack takes its automatic turn
/// (BAT-MAG-025), each marked as having acted when its turn is over. Returns whether the stack
/// awaits its side's command. (A periodic cast kills nothing: its spells have no damage, so the
/// battle goes on.)
fn begins<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) -> bool {
    activation(state, ruleset, id, sink);
    if state.at(id).has(StackTraits::WAR_MACHINE)
        && !war::is_commanded(state, ruleset, Field::new(ruleset.battle().field), id)
    {
        // rule: BAT-WAR-002, BAT-WAR-003
        war::act(state, ruleset, id, sink);
        state.at_mut(id).flags.set(StackFlags::ACTED, true);
        false
    } else {
        !state.at(id).spells.status.contains(Status::BERSERK)
            || berserk_turn(state, ruleset, id, sink)
    }
}

/// The automatic turns of the berserk stack `id` (BAT-MAG-025): it acts ([`control::berserk`]);
/// good morale may give it another turn (BAT-MOR-003, [`morale_turn`]), automatic while it is
/// still berserk. Returns whether the stack keeps the turn, no longer berserk, for its side's
/// command; otherwise its turn is over (it has acted, unless good morale left it to act later in
/// the round).
// rule: BAT-MAG-025
pub(crate) fn berserk_turn<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) -> bool {
    loop {
        let acted = control::berserk(state, ruleset, id, sink);
        // rule: BAT-MAG-035
        crate::raise::mark_corpses(state);
        state.last_side_acted = Some(acted_side(state.at(id)));
        if state.result.is_some() || !acted || !morale::good_morale(state, ruleset, id) {
            state.at_mut(id).flags.set(StackFlags::ACTED, true);
            return false;
        }
        if !morale_turn(state, ruleset, id, sink) {
            return false;
        }
        activation(state, ruleset, id, sink);
        if !state.at(id).spells.status.contains(Status::BERSERK) {
            return true;
        }
    }
}

/// If a cast made the active stack berserk during its own turn, it takes its automatic turn at
/// once (the original's battle loop checks the active stack, `installation.md` §7.14) and, unless
/// it keeps the turn, its turn ends (BAT-MAG-025).
pub(crate) fn berserk_now<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, sink: &mut S) {
    let Some(id) = state
        .active
        .filter(|&id| state.at(id).spells.status.contains(Status::BERSERK))
    else {
        return;
    };
    if berserk_turn(state, ruleset, id, sink) {
        // rule: BAT-MAG-049
        crate::casters::activated(state, ruleset, id, sink);
        return;
    }
    if state.result.is_some() {
        state.active = None;
    } else {
        activate_next(state, ruleset, sink);
    }
}

/// If a cast (its hero's Earthquake, BAT-SIE-015) brought down the last standing target during
/// the commanded Catapult's turn, its turn passes at once, as the turn of a Catapult with no
/// standing target does (BAT-SIE-011: the Catapult's turn 0x473C00, which the original's battle
/// loop calls at 0x4743A3, then sets action 12); it has no command left: no Wait, no Defend.
// rule: BAT-SIE-011
pub(crate) fn catapult_passes<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    sink: &mut S,
) {
    if let Some(id) = state
        .active
        .filter(|&id| state.at(id).is_catapult() && !crate::siege::any_standing(state))
    {
        end_turn(state, ruleset, id, TurnEnd::Acted, sink);
    }
}

/// At the start of its first turn of the round (not after waiting), a regenerating stack's top
/// creature regains all lost hit points (Q20: at the stack's own turn, as VCMI does); a stack its
/// hero makes regenerate (BAT-HER-006) regains at most the stack's `regeneration`.
// rule: BAT-TRN-005
fn regenerate<S: Sink>(state: &mut BattleState, id: StackId, sink: &mut S) {
    let s = state.at_mut(id);
    let most = if s.has(StackTraits::REGENERATES) {
        u32::MAX
    } else {
        u32::from(s.regeneration)
    };
    let hit_points = (s.max_hp - s.top_hp).min(most);
    if hit_points > 0 {
        s.top_hp += hit_points;
        sink.emit(Event::public(EventKind::Regenerated {
            stack: id,
            hit_points,
        }));
    }
}
