//! The computer's army values (E31 T2g; survey `docs/rules/surveys/E31-computer-player.md`
//! §2.3): the army plan (`ai_army_plan`, 0x3C bytes on the stack, [`Plan`]) and what it values on
//! copies, never the game's armies: a stack's worth to an army (`ai_stack_marginal_value`
//! 0x42C830, [`Plan::marginal`], CPU-ARM-002) with the slot it would take or replace
//! (`ai_stack_to_replace` 0x42C690), the alignments and the army's base morale it reads
//! (`ai_army_plan_alignments` 0x42C060, `stack_base_morale` 0x44AE60 as the plan calls it,
//! CPU-ARM-001), a stack put into the army (`ai_army_plan_put` 0x42C130) and the worthless ones
//! shed (`ai_army_plan_shed` 0x42C5B0, CPU-ARM-003), an army exchange (`ai_army_exchange_step`
//! 0x42C280, `ai_army_exchange_value` 0x42C4A0, CPU-ARM-004) and a purchase of the plan's offers
//! (`ai_army_purchase_step` 0x42D420, `ai_army_purchase_value` 0x42D780, with the weighted cost
//! `ai_weighted_cost` 0x526CC0, CPU-ARM-005). Every count, product and sum is the original's 32
//! bits (its 16 where it reads a word), every division truncates toward 0.
//!
//! The creatures are told by the core's handles where the original names them by id (the base
//! pack's creatures are the original's ids: the Angels 12 and 13, the Minotaurs 78 and 79), the
//! factions by the core's (the base and AB packs' towns are the original's 0 to 8, so the
//! Necropolis is 4). The original's RoE game counts the four elementals as neutral by its game
//! version; here the enabled packs give their faction (E08 decision 378, the engine's rule).
//! With the Angelic Alliance a creature without a faction is the neutral alignment, where the
//! original throws (`AI-QUIRK-2`, fixed in every profile).
//!
//! Not here: the plan's real purchases and exchanges with the trades (`ai_army_plan_run` 0x42D690,
//! `ai_army_exchange_apply` 0x42C3B0, `ai_pay_with_trades` 0x42A470), increment 3's hires and
//! visits; the plan of one creature's offer (`ai_army_plan_init_creature` 0x42CF50) with its
//! callers, T2h's Refugee Camp and joining creatures.

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{
    ArtifactTypeId, CreatureFlag, CreatureInfo, CreatureTypeId, FactionId, HeroEffect,
};
use h3_contracts::ArmyStack;
use h3_kernel::arith::SignedX87;

use crate::GameRuleset;

#[cfg(test)]
mod tests;

/// The slots of the original's army record.
const SLOTS: usize = 7;

/// The slowest speed the plan starts its search from (0x42CA09: 20).
const SLOWEST: u8 = 20;

/// What the plan adds to the army's value in the speed's loss (0x42CA70: 500).
const SPEED_BASE: i32 = 500;

/// The Angelic Alliance (the original's artifact 129): its flag joins the alignments.
pub(super) const ANGELIC_ALLIANCE: ArtifactTypeId = ArtifactTypeId::from_raw(129);

/// The Necropolis (the original's town 4): its creatures need morale 2, not 1 (0x42C94E).
const NECROPOLIS: FactionId = FactionId::from_raw(4);

/// The Angel and the Archangel (12, 13): either in the army gives its base morale 1 more.
const ANGELS: [u16; 2] = [12, 13];

/// The Minotaur and the Minotaur King (78, 79): they do not count among the morale's stacks.
const MINOTAURS: [u16; 2] = [78, 79];

/// A slot of the original's army record: the creature (−1 for none) and the count (an `i32`).
type Slot = Option<(CreatureTypeId, i32)>;

/// An army as the original's record holds it (+0 the seven creatures, +0x1C the seven counts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Army(pub(super) [Slot; SLOTS]);

impl Army {
    /// `stacks` in their slots (a count as the original's `i32`); a slot past the seven is not
    /// the record's.
    pub(super) fn of(stacks: &[ArmyStack]) -> Self {
        let mut slots = [None; SLOTS];
        for stack in stacks {
            slots
                .get_mut(usize::from(stack.slot))
                .into_iter()
                .for_each(|slot| *slot = Some((stack.creature, word(stack.count))));
        }
        Self(slots)
    }

    /// Its stacks (`army_stack_count` 0x44ACC0).
    fn stacks(&self) -> usize {
        self.0.iter().flatten().count()
    }

    /// Its slot of `creature`, if one.
    fn slot_of(&self, creature: CreatureTypeId) -> Option<usize> {
        self.0
            .iter()
            .position(|s| s.is_some_and(|(c, _)| c == creature))
    }

    /// Its first empty slot, if one.
    fn first_empty(&self) -> Option<usize> {
        self.0.iter().position(Option::is_none)
    }

    /// Adds `count` of `creature` (`army_add_stack` 0x44ACE0): into `slot`, else its slot of the
    /// creature or its first empty one; none of them: false, nothing changes. The slot takes the
    /// creature, a negative count there is 0 first, then the count is added (32 bits).
    fn add(&mut self, creature: CreatureTypeId, count: i32, slot: Option<usize>) -> bool {
        let Some(at) = slot
            .or_else(|| self.slot_of(creature))
            .or_else(|| self.first_empty())
        else {
            return false;
        };
        let old = self.0[at].map_or(0, |(_, n)| n.max(0));
        self.0[at] = Some((creature, old.wrapping_add(count)));
        true
    }

    /// Clears `slot` (`army_clear_slot` 0x44AB60).
    fn clear(&mut self, slot: usize) {
        self.0[slot] = None;
    }

    /// Each later stack of a slot's creature merged into the slot (the loop inlined in
    /// 0x42C4A0, 0x42D780, `ai_army_merge_types` 0x42D870): slots 0 to 5, each of its later ones.
    fn merge_types(&mut self) {
        for at in 0..SLOTS - 1 {
            let Some((creature, _)) = self.0[at] else {
                continue;
            };
            for later in at + 1..SLOTS {
                if let Some((c, n)) = self.0[later].filter(|&(c, _)| c == creature) {
                    self.0[at] = self.0[at].map(|(_, sum)| (c, sum.wrapping_add(n)));
                    self.clear(later);
                }
            }
        }
    }

    /// Its value (`army_value` 0x44AC80): each stack's creature's AI value times its count,
    /// summed in 32 bits.
    pub(super) fn value(&self, rules: &GameRuleset) -> i32 {
        self.0.iter().flatten().fold(0_i32, |sum, &(c, n)| {
            sum.wrapping_add(ai_value(rules, c).wrapping_mul(n))
        })
    }
}

/// A 32-bit unsigned value as the original's signed field holds it.
const fn word(value: u32) -> i32 {
    i32::from_ne_bytes(value.to_ne_bytes())
}

/// The low 16 bits of `value` as the original's word reads them.
const fn short(value: i32) -> i16 {
    let [low, high, _, _] = value.to_le_bytes();
    i16::from_le_bytes([low, high])
}

/// `creature`'s core info.
fn info(rules: &GameRuleset, creature: CreatureTypeId) -> Option<&CreatureInfo> {
    rules.core().creature(creature)
}

/// `creature`'s AI value (creature +0x40), 0 for one the core lacks.
fn ai_value(rules: &GameRuleset, creature: CreatureTypeId) -> i32 {
    info(rules, creature).map_or(0, |c| word(c.ai_value))
}

/// Whether `creature` has `flag`.
fn has(rules: &GameRuleset, creature: CreatureTypeId, flag: CreatureFlag) -> bool {
    info(rules, creature).is_some_and(|c| c.flags.contains(flag))
}

/// `creature`'s faction, none for a neutral one.
fn faction(rules: &GameRuleset, creature: CreatureTypeId) -> Option<FactionId> {
    info(rules, creature).and_then(|c| c.faction)
}

/// Whether `creature` is one of `ids`.
fn is_one_of(creature: CreatureTypeId, ids: [u16; 2]) -> bool {
    ids.contains(&creature.raw())
}

/// The factions the Angelic Alliance joins (`alignment_join_towns` 0x44A460: Castle, Rampart,
/// Tower, Stronghold and Fortress): its own `alignment_group`, as the battle reads it
/// (BAT-HER-007), none without the artifact.
pub(super) fn alliance_factions(rules: &GameRuleset) -> Vec<FactionId> {
    let core = rules.core();
    core.artifact(ANGELIC_ALLIANCE)
        .into_iter()
        .flat_map(|a| &a.effects)
        .filter_map(|e| match e {
            HeroEffect::AlignmentGroup { factions } => Some(factions),
            _ => None,
        })
        .flatten()
        .copied()
        .collect()
}

/// An offer of a plan (`ai_army_offer`, 12 bytes): a creature, its count (an `i16`) and whether
/// it is free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Offer {
    /// The creature.
    pub(super) creature: CreatureTypeId,
    /// How many are offered (+8, an `i16`).
    pub(super) count: i16,
    /// Free (+0xA): taken whole, its cost not paid nor weighed.
    pub(super) free: bool,
}

/// The computer's army plan (`ai_army_plan`; structs.toml): an army and a spare, the alignments
/// it joins, the hero's own morale, the army's alignments, the AI value the puts gained, the
/// primary advantage, the owner's resource weights, the resources it pays with and its offers.
/// Every value works on the plan's copies.
#[derive(Debug, Clone)]
pub(super) struct Plan<'a> {
    rules: &'a GameRuleset,
    /// The army (+0).
    army: Army,
    /// The spare army (+4): the other army of an exchange, a purchase's displaced stacks.
    spare: Option<Army>,
    /// The factions the Angelic Alliance joins when its flag (+8) is set, else none.
    join: &'a [FactionId],
    /// The hero's own morale (+0xA, `hero_morale` 0x4E39B0 clamped).
    morale: i16,
    /// The army's alignments (+0xC) and their stacks by alignment (+0xE: the neutral, then each
    /// faction of the core), as [`Plan::recount`] last counted them.
    alignments: (u16, Vec<u8>),
    /// The AI value the puts gained (+0x18).
    pub(super) gain: i32,
    /// The primary advantage (+0x1C, an `i16`).
    advantage: i16,
    /// The owner's resource weights (+0x20; the start's, `Record::weights`), by resource.
    weights: &'a [SignedX87],
    /// The resources a purchase pays with (+0x24), by resource.
    resources: Vec<i32>,
    /// Its offers (+0x2C).
    pub(super) offers: Vec<Offer>,
}

impl<'a> Plan<'a> {
    /// A plan of `offers` weighing its costs by `weights` (the inits' header,
    /// `ai_army_plan_init_dwelling` 0x42CB70 and the town's 0x42CE30: the armies, the morale, the
    /// counts and the gain cleared, the costs weighed, +0x28 1).
    pub(super) fn new(
        rules: &'a GameRuleset,
        weights: &'a [SignedX87],
        offers: Vec<Offer>,
    ) -> Self {
        Self {
            rules,
            army: Army([None; SLOTS]),
            spare: None,
            join: &[],
            morale: 0,
            alignments: (0, Vec::new()),
            gain: 0,
            advantage: 0,
            weights,
            resources: Vec::new(),
            offers,
        }
    }

    /// The alignment of `creature` in the plan, by its place in [`Plan::alignments`]: 0 for a
    /// creature without a faction (with the Alliance too: `AI-QUIRK-2`), else its faction's (the
    /// Alliance's first faction for one it joins) after it.
    // rule: CPU-ARM-001
    fn alignment(&self, creature: CreatureTypeId) -> usize {
        faction(self.rules, creature).map_or(0, |f| {
            let first = self
                .join
                .iter()
                .copied()
                .min()
                .filter(|_| self.join.contains(&f));
            first.unwrap_or(f).index() + 1
        })
    }

    /// The army's alignments as `army_alignment_count` 0x44ABB0 counts them: each stack but a war
    /// machine's by its faction (the neutral first), and how many hold one.
    fn raw_alignments(&self) -> (u16, Vec<u8>) {
        let mut counts = vec![0_u8; self.rules.core().factions().len() + 1];
        for &(c, _) in self.army.0.iter().flatten() {
            if !has(self.rules, c, CreatureFlag::WarMachine) {
                let at = faction(self.rules, c).map_or(0, |f| f.index() + 1);
                counts
                    .get_mut(at)
                    .into_iter()
                    .for_each(|n| *n = n.wrapping_add(1));
            }
        }
        let held = counts.iter().filter(|&&n| n > 0).count();
        (u16::try_from(held).unwrap_or(u16::MAX), counts)
    }

    /// Recounts the alignments (`ai_army_plan_alignments` 0x42C060; CPU-ARM-001): the raw ones,
    /// then with the Alliance each faction it joins moved onto its first one, one alignment fewer
    /// for each that meets that one already counted.
    // rule: CPU-ARM-001
    fn recount(&mut self) {
        let (mut held, mut counts) = self.raw_alignments();
        if let Some(first) = self.join.iter().copied().min() {
            let to = first.index() + 1;
            for at in 1..counts.len() {
                let joins = FactionId::from_index(at - 1).is_some_and(|f| self.join.contains(&f));
                if counts[at] != 0 && joins && at != to {
                    if counts[to] > 0 {
                        held = held.wrapping_sub(1);
                    }
                    counts[to] = counts[to].wrapping_add(counts[at]);
                    counts[at] = 0;
                }
            }
        }
        self.alignments = (held, counts);
    }

    /// The army's base morale as the plan asks it (`stack_base_morale` 0x44AE60 with no hero, no
    /// town, no enemy army, the plan's Alliance flag and no clamp; CPU-ARM-001): 2 less its raw
    /// alignments, with the Alliance one more for each faction it joins past the first present;
    /// 1 less with an undead stack; 1 more with an Angel or an Archangel.
    // rule: CPU-ARM-001
    fn base_morale(&self) -> i32 {
        let (held, counts) = self.raw_alignments();
        let joined = (1..counts.len())
            .filter(|&at| counts[at] > 0)
            .filter(|&at| FactionId::from_index(at - 1).is_some_and(|f| self.join.contains(&f)))
            .count();
        let joined = i32::try_from(joined).unwrap_or(i32::MAX);
        let mut held = i32::from(held);
        if joined > 1 {
            held = held.wrapping_add(1 - joined);
        }
        let stacks = || self.army.0.iter().flatten().map(|&(c, _)| c);
        let undead = stacks().any(|c| has(self.rules, c, CreatureFlag::Undead));
        let angels = stacks().any(|c| is_one_of(c, ANGELS));
        (2 - held) - i32::from(undead) + i32::from(angels)
    }

    /// Whether a stack of `creature` feels morale (not flag 0x20000, not a Minotaur).
    fn feels_morale(&self, creature: CreatureTypeId) -> bool {
        !has(self.rules, creature, CreatureFlag::NoMorale) && !is_one_of(creature, MINOTAURS)
    }

    /// What adding `count` of `creature` to the army is worth, and its slot
    /// (`ai_stack_marginal_value` 0x42C830; CPU-ARM-002), `forced` a swap: v = its AI value times
    /// the count. A new alignment (none of the army's stacks hold it) in an army with stacks whose
    /// base morale ([`Plan::base_morale`]) plus the hero's is below 1 (2 for a Necropolis
    /// creature) is **morale-bad** when the stacks that feel morale (the new one too when it
    /// does) are worth at least 10 v. Slower than the army's slowest (from 20): v plus
    /// trunc(V × b / a − V), V the army's value plus 500, a and b the day's land points at the old
    /// and the new slowest speed (doubles). Its creature's slot: that slot, v (forced: −1). Else,
    /// unless morale-bad or forced, the first empty slot, v (none, or six stacks with a spare
    /// army: a replacement). Else the slot [`Plan::to_replace`] names, v less that stack's AI
    /// value times its count; none: 0.
    // rule: CPU-ARM-002
    fn marginal(&self, creature: CreatureTypeId, count: i16, forced: bool) -> (i32, Option<usize>) {
        let rules = self.rules;
        let mut v = ai_value(rules, creature).wrapping_mul(count.into());
        let mut bad = false;
        let new = self
            .alignments
            .1
            .get(self.alignment(creature))
            .copied()
            .unwrap_or(0)
            == 0;
        if new && self.army.stacks() > 0 {
            let least = if faction(rules, creature) == Some(NECROPOLIS) {
                2
            } else {
                1
            };
            if self.base_morale().wrapping_add(self.morale.into()) < least {
                let sum = self
                    .army
                    .0
                    .iter()
                    .flatten()
                    .filter(|&&(c, _)| self.feels_morale(c))
                    .fold(0_i32, |sum, &(c, n)| {
                        sum.wrapping_add(ai_value(rules, c).wrapping_mul(n))
                    });
                let sum = if self.feels_morale(creature) {
                    sum.wrapping_add(v)
                } else {
                    sum
                };
                bad = sum >= v.wrapping_mul(10);
            }
        }
        let speed = |c| info(rules, c).map_or(0, |i| i.speed);
        let slowest = self
            .army
            .0
            .iter()
            .flatten()
            .map(|&(c, _)| speed(c))
            .fold(SLOWEST, u8::min);
        let own = speed(creature);
        if slowest > own {
            let land = |s| SignedX87::int(rules.adventure().params().land_points(s).into());
            let base = SignedX87::int(self.army.value(rules).wrapping_add(SPEED_BASE).into());
            let loss = land(own)
                .times(base)
                .over(land(slowest))
                .map_or(0, |q| q.minus(base).ftol());
            v = v.wrapping_add(loss);
        }
        if let Some(at) = self.army.slot_of(creature) {
            return (if forced { -1 } else { v }, Some(at));
        }
        if !bad && !forced {
            let room = self.army.stacks() < 6 || self.spare.is_none();
            if let Some(at) = self.army.first_empty().filter(|_| room) {
                return (v, Some(at));
            }
        }
        let shooter = has(rules, creature, CreatureFlag::Shooter);
        self.to_replace(shooter, bad).map_or((0, None), |at| {
            let worth = self.army.0[at].map_or(0, |(c, n)| ai_value(rules, c).wrapping_mul(n));
            (v.wrapping_sub(worth), Some(at))
        })
    }

    /// The slot a new stack displaces, none for none (`ai_stack_to_replace` 0x42C690; CPU-ARM-002):
    /// the stack of least AI value times count (the first on ties) among those that may go:
    /// under morale-bad only a stack alone in its alignment; a new shooter with more than three
    /// shooting stacks only a shooter; a new non-shooter never the army's only shooter.
    // rule: CPU-ARM-002
    fn to_replace(&self, shooter: bool, bad: bool) -> Option<usize> {
        let rules = self.rules;
        let shoots = |c| has(rules, c, CreatureFlag::Shooter);
        let shooters = self
            .army
            .0
            .iter()
            .flatten()
            .filter(|&&(c, _)| shoots(c))
            .count();
        let only_shooters = shooter && shooters > 3;
        let keep_shooter = !shooter && shooters == 1;
        let alone = |c| self.alignments.1.get(self.alignment(c)).copied() == Some(1);
        (0..SLOTS)
            .filter_map(|at| self.army.0[at].map(|(c, n)| (at, c, n)))
            .filter(|&(_, c, _)| !bad || alone(c))
            .filter(|&(_, c, _)| !only_shooters || shoots(c))
            .filter(|&(_, c, _)| !keep_shooter || !shoots(c))
            .fold(None, |best: Option<(usize, i32)>, (at, c, n)| {
                let worth = n.wrapping_mul(ai_value(rules, c));
                match best {
                    Some((_, least)) if worth >= least => best,
                    _ => Some((at, worth)),
                }
            })
            .map(|(at, _)| at)
    }

    /// Puts `count` of `creature` into `slot` (`ai_army_plan_put` 0x42C130; CPU-ARM-003): the gain
    /// plus its AI value times the count; another creature there: the gain less that one's AI
    /// value times its count's low 16 bits, and with a spare army it goes there (its slot of the
    /// creature or its first empty one; a full spare without it: over the spare's stack of least
    /// AI value times count below its own, the first on ties, which is lost; none such: the
    /// displaced stack is lost), the slot cleared; then the stack added into the slot.
    // rule: CPU-ARM-003
    fn put(&mut self, creature: CreatureTypeId, count: i16, slot: usize) {
        let rules = self.rules;
        self.gain = self
            .gain
            .wrapping_add(ai_value(rules, creature).wrapping_mul(count.into()));
        if let Some((old, n)) = self.army.0[slot].filter(|&(c, _)| c != creature) {
            let n = i32::from(short(n));
            let worth = ai_value(rules, old).wrapping_mul(n);
            self.gain = self.gain.wrapping_sub(worth);
            if let Some(spare) = self.spare.as_mut()
                && !spare.add(old, n, None)
            {
                {
                    let least = (0..SLOTS)
                        .filter_map(|at| {
                            spare.0[at].map(|(c, m)| (at, ai_value(rules, c).wrapping_mul(m)))
                        })
                        .fold((None, worth.wrapping_neg()), |(best, most), (at, w)| {
                            if w.wrapping_neg() > most {
                                (Some(at), w.wrapping_neg())
                            } else {
                                (best, most)
                            }
                        })
                        .0;
                    if let Some(at) = least {
                        spare.clear(at);
                        spare.add(old, n, Some(at));
                    }
                }
            }
            self.army.clear(slot);
        }
        self.army.add(creature, count.into(), Some(slot));
    }

    /// Moves the army's stacks worth nothing to the spare army (`ai_army_plan_shed` 0x42C5B0;
    /// CPU-ARM-003): only with a spare; each slot in order, while the spare has fewer than seven
    /// stacks and the army more than one, the stack taken out, the alignments recounted, its
    /// [`Plan::marginal`] asked with its count's low 16 bits: above 0 back into its slot, else into
    /// the spare.
    // rule: CPU-ARM-003
    fn shed(&mut self) {
        let full =
            |plan: &Self| plan.spare.is_none_or(|s| s.stacks() == SLOTS) || plan.army.stacks() == 1;
        if full(self) {
            return;
        }
        // The original tests the counts again before each stack (0x42C5F3); only a stack moved to
        // the spare changes them, and that move tests them at once.
        for at in 0..SLOTS {
            let Some((creature, count)) = self.army.0[at] else {
                continue;
            };
            self.army.clear(at);
            self.recount();
            let (gain, _) = self.marginal(creature, short(count), false);
            if gain > 0 {
                self.army.0[at] = Some((creature, count));
            } else {
                // The spare has room: the test before the loop or after the last move.
                self.spare.iter_mut().for_each(|spare| {
                    let _ = spare.add(creature, count, None);
                });
                if full(self) {
                    return;
                }
            }
        }
    }

    /// One transfer of an exchange (`ai_army_exchange_step` 0x42C280; CPU-ARM-004), `several`
    /// when the other army (the spare) has more than one stack: the alignments recounted; for
    /// each of the other army's stacks its whole count's [`Plan::marginal`] with `several`, else
    /// the better of the whole count forced (a swap) and, above one, all but one; each times the
    /// primary advantage over 40. The best above 0 (the first on ties) is taken off the other
    /// army (its slot cleared when whole) and [`Plan::put`]; it is returned (else 0).
    // rule: CPU-ARM-004
    fn exchange_step(&mut self, several: bool) -> i32 {
        self.recount();
        let other = self.spare.unwrap_or(Army([None; SLOTS]));
        let mut best = (0_i32, None);
        for at in 0..SLOTS {
            let Some((creature, count)) = other.0[at] else {
                continue;
            };
            let whole = short(count);
            let (mut gain, mut slot) = self.marginal(creature, whole, !several);
            let mut taken = whole;
            if !several && whole > 1 {
                let (fewer, into) = self.marginal(creature, whole - 1, false);
                if fewer > gain {
                    (gain, slot, taken) = (fewer, into, whole - 1);
                }
            }
            let worth = i32::from(self.advantage).wrapping_mul(gain) / 40;
            if worth > best.0 {
                best = (worth, slot.map(|slot| (at, creature, taken, slot)));
            }
        }
        if let (worth, Some((at, creature, taken, slot))) = best {
            self.spare.iter_mut().for_each(|spare| {
                spare.0[at] = spare.0[at].and_then(|(c, n)| {
                    (short(n) != taken).then(|| (c, n.wrapping_sub(taken.into())))
                });
            });
            self.put(creature, taken, slot);
            return worth;
        }
        best.0
    }

    /// The value to a hero of taking another army's creatures (`ai_army_exchange_value` 0x42C4A0;
    /// CPU-ARM-004): on copies of its `army` and the `other` army, with the Alliance's factions
    /// `join` (empty without its flag), the hero's own `morale` and its primary `advantage` (its
    /// primary sum less the other hero's, none 0; at least 0): the army's later stacks of a slot's
    /// creature merged into it, [`Plan::shed`], then [`Plan::exchange_step`] while it gains; the
    /// gains summed. So a hero no stronger than the other values the exchange at 0.
    // rule: CPU-ARM-004
    pub(super) fn exchange_value(
        &mut self,
        (army, other): (&[ArmyStack], &[ArmyStack]),
        (join, morale, advantage): (&'a [FactionId], i16, i16),
    ) -> i32 {
        self.army = Army::of(army);
        self.spare = Some(Army::of(other));
        self.join = join;
        self.morale = morale;
        self.advantage = advantage.max(0);
        self.army.merge_types();
        self.shed();
        let mut sum = 0_i32;
        loop {
            let several = self.spare.is_some_and(|s| s.stacks() > 1);
            let gain = self.exchange_step(several);
            sum = sum.wrapping_add(gain);
            if gain <= 0 {
                return sum;
            }
        }
    }

    /// How many of `offer` the plan's resources pay: its count, at most each resource's stock over
    /// the creature's cost of it (a resource with a cost and no stock: 0).
    fn affordable(&self, offer: Offer) -> i32 {
        let costs = creature_costs(self.rules, offer.creature);
        costs
            .iter()
            .zip(self.resources.iter().copied().chain(core::iter::repeat(0)))
            .filter(|&(&cost, _)| cost > 0)
            .fold(i32::from(offer.count), |most, (&cost, stock)| {
                most.min(if stock > 0 { stock / cost } else { 0 })
            })
    }

    /// One purchase of the plan's offers (`ai_army_purchase_step` 0x42D420 without the trades;
    /// CPU-ARM-005): the alignments recounted; for each offer with a count, its count when free,
    /// else as many as the resources pay; its [`Plan::marginal`], above 0 and not free less its
    /// [`weighted_cost`] times the count, but never more than three quarters of it (truncated).
    /// The best above 0 (the first on ties) is [`Plan::put`] into its slot, the resources less its
    /// cost (when not free) and the offer's count less the count; it is returned (else 0).
    // rule: CPU-ARM-005
    fn purchase_step(&mut self) -> i32 {
        self.recount();
        let mut best = (0_i32, None);
        for (index, &offer) in self.offers.iter().enumerate() {
            if offer.count <= 0 {
                continue;
            }
            let count = if offer.free {
                i32::from(offer.count)
            } else {
                self.affordable(offer)
            };
            if count <= 0 {
                continue;
            }
            let (mut gain, slot) = self.marginal(offer.creature, short(count), false);
            if gain <= 0 {
                continue;
            }
            if !offer.free {
                let cost = weighted_cost(self.weights, &creature_costs(self.rules, offer.creature))
                    .wrapping_mul(count);
                let most = gain.wrapping_mul(3) / 4;
                gain = gain.wrapping_sub(cost.min(most));
            }
            if gain > best.0 {
                best = (gain, slot.map(|slot| (index, count, slot)));
            }
        }
        if let (gain, Some((index, count, slot))) = best {
            let offer = self.offers[index];
            self.put(offer.creature, short(count), slot);
            if !offer.free {
                let costs = creature_costs(self.rules, offer.creature);
                let n = i32::from(short(count));
                for (stock, cost) in self.resources.iter_mut().zip(costs) {
                    *stock = stock.wrapping_sub(cost.wrapping_mul(n));
                }
            }
            self.offers[index].count = offer.count.wrapping_sub(short(count));
            return gain;
        }
        best.0
    }

    /// The value of buying the plan's offers into an army (`ai_army_purchase_value` 0x42D780;
    /// CPU-ARM-005): on copies of `army`, the `spare` army (none: no spare) and `resources` (by
    /// resource), with the Alliance's factions `join` and the hero's own `morale`: the army's
    /// later stacks of a slot's creature merged into it, [`Plan::shed`], then
    /// [`Plan::purchase_step`] while it gains; the gains summed. The offers' counts and the gain
    /// are the plan's (its +0x18 the caller reads).
    // rule: CPU-ARM-005
    pub(super) fn purchase_value(
        &mut self,
        (army, spare): (&[ArmyStack], Option<&[ArmyStack]>),
        resources: Vec<i32>,
        (join, morale): (&'a [FactionId], i16),
    ) -> i32 {
        self.army = Army::of(army);
        self.morale = morale;
        self.resources = resources;
        self.join = join;
        self.spare = spare.map(Army::of);
        self.army.merge_types();
        self.shed();
        let mut sum = 0_i32;
        loop {
            let gain = self.purchase_step();
            sum = sum.wrapping_add(gain);
            if gain <= 0 {
                return sum;
            }
        }
    }
}

/// `creature`'s cost by resource in the core's order (`creature_cost_array` 0x54E7C0's seven
/// `i32`), none for one the core lacks.
pub(super) fn creature_costs(rules: &GameRuleset, creature: CreatureTypeId) -> Vec<i32> {
    let core = rules.core();
    let mut costs = vec![0_i32; core.resources().len()];
    for &(resource, amount) in info(rules, creature).map_or(&[][..], |c| &c.cost[..]) {
        costs
            .get_mut(resource.index())
            .into_iter()
            .for_each(|c| *c = word(amount));
    }
    costs
}

/// A cost's worth to a player (`ai_weighted_cost` 0x526CC0; CPU-ARM-005), `weights` its resource
/// weights (`Record::weights`): over the resources in order, the running total plus the cost
/// times the weight, in double precision, truncated at each step.
// rule: CPU-ARM-005
pub(super) fn weighted_cost(weights: &[SignedX87], costs: &[i32]) -> i32 {
    costs
        .iter()
        .zip(weights)
        .fold(0_i32, |total, (&cost, &weight)| {
            SignedX87::int(cost.into())
                .times(weight)
                .plus(SignedX87::int(total.into()))
                .ftol()
        })
}
