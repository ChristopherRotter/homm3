//! The creature banks on the map (E08 T124b1, T124b2; increment 13's design note parts B and C):
//! their draw when the game is built (ADV-BNK-002, design note §5), called by
//! `draws::resolve_rewards`' second walk in object order; their visit and its questions
//! (ADV-BNK-003, §6), the battle's map half (ADV-BNK-004, §7, §14), its end and the reward after
//! the winner's levels (ADV-BNK-005, §8, §15) and a looted search's once-bonus (HER-ADV-008, part
//! C). The hint and its events come with T125a (§10, §16).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, CreatureTypeId, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, HeroReward};
use h3_kernel::Rng;

use super::draws::ArtifactPool;
use super::{AdventureState, Resolved};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, Defender, Offer, OfferKind, Sink, Viewers,
};
use crate::ruleset::{
    AdventureRuleset, BANK_ARTIFACT_CLASSES, Bank, BankLevel, BankQuestion, ObjectTypeId,
};
use crate::setup::{Guards, Pos, RewardRecord};

/// The rules a visit reads: the core, the ruleset part and the game's store of hero details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

/// A creature bank's draws (ADV-BNK-002, E08 T124b1; increment 13's design note §5, survey
/// `E08-banks.md` §3: `bank_draw` 0x47AD90 in `setup_map_objects`' walk):
/// 1. **the level:** r in 1 to 100; the first level but the last whose running sum of chances
///    reaches r, else the last (the original's subtraction over its levels 0 to 2, else 3);
/// 2. **the artifacts:** the level's counts by class, the relics first, then the majors, the
///    minors and the treasures (D-68), each a draw of its class from the random artifacts' pool
///    (ADV-OBJ-006, the original's `random_artifact_of_level`); a draw that finds none gives none;
/// 3. **the split** of the level's guards into their slots, without a draw ([`bank_split`]);
/// 4. **the upgrade:** when the level's chance is above 0 and the split's upgrade slot holds a
///    creature with an upgrade (`Ruleset::upgrades`, the first; the original's
///    `creature_is_unupgraded` 0x47B120 and `creature_upgrade` 0x47B1A0 agree on every shipped
///    guard), r in 1 to 100: at most the chance, the stack becomes that upgrade, its count kept.
///    The original draws r for every bank; the engine only where it can change a stack (E08
///    decision 17's form).
///
/// The guards ask (their [`Guards::question`]; a bank's visit asks by its kind's question).
// rule: ADV-BNK-002
pub(super) fn bank_draw(
    core: &Ruleset,
    bank: &Bank,
    artifacts: &mut ArtifactPool<'_>,
    draws: &mut Rng,
) -> (Guards, Resolved) {
    let r = draws.range_inclusive(1, 100);
    // A bank has one to four levels (`compile_bank`).
    let last = bank.levels.len() - 1;
    let mut sum = 0;
    let index = (0..last)
        .find(|&l| {
            sum += u32::from(bank.levels[l].chance);
            r <= sum
        })
        .unwrap_or(last);
    let level = &bank.levels[index];
    let mut drawn = Vec::new();
    for (class, &count) in BANK_ARTIFACT_CLASSES.iter().zip(&level.artifacts).rev() {
        for _ in 0..count {
            drawn.extend(artifacts.draw(&[*class], draws));
        }
    }
    let (mut army, slot) = bank_split(&level.guards);
    if level.upgrade > 0
        && let Some(stack) = army.iter_mut().find(|stack| stack.slot == slot)
        && let Some(&upgrade) = core.upgrades(stack.creature).first()
        && draws.range_inclusive(1, 100) <= u32::from(level.upgrade)
    {
        stack.creature = upgrade;
    }
    let guards = Guards {
        army,
        question: true,
    };
    let level = u8::try_from(index).unwrap_or(u8::MAX);
    let resolved = Resolved::Bank {
        level,
        artifacts: drawn,
    };
    (guards, resolved)
}

/// A bank level's guards in their slots and the slot its upgrade tests (ADV-BNK-002; increment
/// 13's design note §5 step 3, survey `E08-banks.md` §3 step 3): the named creatures in slots 0
/// up, then by their number n, each new stack in the next slot as the original's first empty slot
/// puts it:
/// - n = 1: slots 1 to 4 take ⌊c / 5⌋, then of what is left ⌊· / 4⌋, ⌊· / 3⌋ and ⌊· / 2⌋, slot 0
///   the rest; the upgrade's slot 2;
/// - n = 2: the second's count halved into slot 2, the first's divided by 3 into slot 3 and what
///   is left of it halved into slot 4; the upgrade's slot 3;
/// - n = 3: the second's halved into slot 3, the first's into slot 4; the upgrade's slot 0;
/// - n = 4: no split; the upgrade's slot 0.
///
/// A quotient of 0 leaves its slot empty (*agent's call*: unreachable with the shipped counts,
/// every one at least 10; the original's write of a stack of 0 is unread). What moves is the
/// quotient; what stays the rest, so no stack named is ever empty.
// rule: ADV-BNK-002
fn bank_split(guards: &[(CreatureTypeId, u32)]) -> (Vec<ArmyStack>, u8) {
    let mut slots = guards.to_vec();
    // Moves `divisor`'s quotient of slot `from`'s count into a new slot.
    let mut split = |from: usize, divisor: u32| {
        let (creature, count) = slots[from];
        let part = count / divisor;
        slots[from].1 = count - part;
        slots.push((creature, part));
    };
    let upgrade = match guards.len() {
        1 => {
            for divisor in [5, 4, 3, 2] {
                split(0, divisor);
            }
            2
        }
        2 => {
            split(1, 2);
            split(0, 3);
            split(0, 2);
            3
        }
        3 => {
            split(1, 2);
            split(0, 2);
            0
        }
        _ => 0,
    };
    let army = slots
        .into_iter()
        .enumerate()
        .filter(|&(_, (_, count))| count > 0)
        .map(|(slot, (creature, count))| ArmyStack {
            slot: u8::try_from(slot).unwrap_or(u8::MAX),
            creature,
            count,
        })
        .collect();
    (army, upgrade)
}

impl Resolved {
    /// A creature bank's drawn level and artifacts ([`Self::Bank`]), if these are.
    pub fn bank(&self) -> Option<(u8, &[ArtifactTypeId])> {
        match self {
            Self::Bank { level, artifacts } => Some((*level, artifacts)),
            _ => None,
        }
    }
}

impl AdventureState {
    /// The kind of the creature bank at object index `index`, if it is one.
    pub(super) fn bank_of<'p>(&self, part: &'p AdventureRuleset, index: usize) -> Option<&'p Bank> {
        self.object_at(index)
            .object_type
            .and_then(|id| part.object_type(id))
            .and_then(|ty| ty.kind.bank())
    }

    /// The visit of the creature bank at object index `index`, of type `object_type`, by
    /// `player`'s hero `hero` (ADV-BNK-003, E08 T124b2; increment 13's design note §6, survey
    /// `E08-banks.md` §4, D-66 the original), by its kind's question:
    /// - **attack** (`visit_creature_bank` 0x4A15A0, `visit_dragon_utopia` 0x4A2140): the team's
    ///   marks at once (ADV-VIS-006: the visitor's owner's, its team's from E10); a looted one
    ///   tells the owner so ([`AdventureEventKind::ObjectEmpty`]: ADVEVENT 33) and asks nothing;
    ///   else its question ([`OfferKind::Fight`], ADVEVENT 32);
    /// - **search** (`visit_search_bank` 0x4AC490): its question first, looted or not
    ///   ([`OfferKind::Search`], ADVEVENT 41, 119, 122), the marks only after a yes
    ///   ([`Self::enter_bank`]).
    ///
    /// Declined, nothing more: an attack kind's marks stay, the hero stays where the move ended.
    /// The marks tell no event yet: the hint and its events are T125a's (design note §10, §16).
    // rule: ADV-BNK-003, ADV-VIS-006
    pub(super) fn visit_bank<S: Sink>(
        &mut self,
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        bank: &Bank,
        sink: &mut S,
    ) -> AdventureOutcome {
        match bank.question {
            BankQuestion::Attack => {
                self.learn(player, index);
                if self.is_emptied(index) {
                    self.tell_looted(player, (index, object_type), sink);
                    return AdventureOutcome::default();
                }
                self.ask(hero, (index, object_type), OfferKind::Fight)
            }
            BankQuestion::Search => self.ask(hero, (index, object_type), OfferKind::Search),
        }
    }

    /// The creature bank at object index `index` of kind `bank`, its question `offer` accepted by
    /// `player` ([`OfferKind::Fight`] or [`OfferKind::Search`], routed by the object's kind as a
    /// mine's `Fight`; ADV-BNK-003 to -005, E08 T124b2; increment 13's design note §6 to §8): a
    /// search kind's team marks first (ADV-VIS-006); a looted one tells the owner so
    /// ([`AdventureEventKind::ObjectEmpty`], the original's second text) and gives a search kind's
    /// `looted` once-bonus by its mark (HER-ADV-008: −1 morale until the end of the hero's next
    /// battle, while the mark is not held; part C); else the battle against its guards as they
    /// stand at once ([`Self::bank_battle`]), or with none standing (a draw left none) its reward
    /// at once, with no battle and no levels ([`Self::loot_bank`]; *agent's call*, the guarded
    /// piles' "without guards standing"). An attack kind looted while its question was open
    /// (another player's yes meanwhile) is told so too.
    // rule: ADV-BNK-003, ADV-BNK-004, ADV-BNK-005, ADV-VIS-006, HER-ADV-008
    pub(super) fn enter_bank<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        player: PlayerId,
        (index, bank): (usize, &Bank),
        offer: &Offer,
        sink: &mut S,
    ) -> AdventureOutcome {
        if bank.question == BankQuestion::Search {
            self.learn(player, index);
        }
        if self.is_emptied(index) {
            self.tell_looted(player, (index, offer.object_type), sink);
            // rule: HER-ADV-008
            if let Some(looted) = bank.looted {
                self.give_once(offer.hero, looted.visit());
            }
            return AdventureOutcome::default();
        }
        if self.guards_left(index).is_empty() {
            return self.loot_bank(rules, offer.hero, (index, bank), sink);
        }
        self.bank_battle(rules.1, offer.hero, (index, bank))
    }

    /// Tells `player` that the bank at `index` is looted ([`AdventureEventKind::ObjectEmpty`]).
    fn tell_looted<S: Sink>(
        &self,
        player: PlayerId,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) {
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::ObjectEmpty {
                object_type,
                anchor: self.object_at(index).anchor,
            },
            seen_by: Viewers::NONE.with(player),
        });
    }

    /// The battle of the hero `hero` against the guards of the bank at `index` as they stand
    /// (ADV-BNK-004, GAM-BAT-001; increment 13's design note §7, §14): an earlier battle's
    /// survivors, an upgraded stack as its own creature, by slot, unowned and without a hero
    /// ([`Defender::Bank`], so the game sets the setup's bank battle from its cause, ADR-0022
    /// point 8), fought at its entrance (where the hero stands for the ten entered kinds; the
    /// Shipwreck's water tile beside it). A `deck` kind's (the Shipwreck, the Derelict Ship: the
    /// original's ship flag +0x53C6) on the sea terrain's deck as with one hero in a boat, the
    /// bank counted as one boat at least ([`Self::sea_battle_ground`]): from the coast and from a
    /// boat alike, a special ground on the tile replacing the deck; a land kind's where any
    /// battle on its tile is fought. None when the hero is not on the map.
    // rule: ADV-BNK-004, GAM-BAT-001
    fn bank_battle(
        &self,
        part: &AdventureRuleset,
        hero: HeroId,
        (index, bank): (usize, &Bank),
    ) -> AdventureOutcome {
        let army = self.guards_left(index).to_vec();
        let battle = self
            .hero(hero)
            .zip(self.pile_tile(index))
            .map(|(attacker, tile)| {
                let mut start = self.battle_at(part, attacker, Defender::Bank { army }, tile);
                if bank.deck {
                    (start.terrain, start.battlefield) = self.sea_battle_ground(part, tile, 1);
                }
                start
            });
        AdventureOutcome {
            battle,
            ..AdventureOutcome::default()
        }
    }

    /// The end of a battle against the guards of the bank at `index` (ADV-BNK-004, ADV-BNK-005;
    /// increment 13's design note §7): lost or retreated, its guards' `survivors` by slot stay as
    /// its army left (an upgraded stack as its own creature), not looted, asked again at the next
    /// visit; a draw leaves none standing, not looted (the next yes gives the reward at once);
    /// won, none stand and the reward comes after the winner's levels
    /// ([`AdventureOutcome::reward_after`] at `tile`, GAM-DEC-007: [`Self::bank_won`]).
    // rule: ADV-BNK-004, ADV-BNK-005, GAM-DEC-007
    pub(super) fn end_bank_battle(
        &mut self,
        index: usize,
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
    ) -> AdventureOutcome {
        let left = if won { Vec::new() } else { survivors.to_vec() };
        self.army_left.set(index, Some(left));
        AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        }
    }

    /// What a won battle against the bank at `index` of kind `bank` gives the hero `hero` after
    /// the winner's levels (ADV-BNK-005, GAM-DEC-007; increment 13's design note §15): its reward
    /// ([`Self::loot_bank`]), read when given; nothing when it was looted meanwhile (another
    /// player's yes at it without guards standing while the winner's levels were open).
    // rule: ADV-BNK-005, GAM-DEC-007
    pub(super) fn bank_won<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        hero: HeroId,
        bank: (usize, &Bank),
        sink: &mut S,
    ) -> AdventureOutcome {
        if self.is_emptied(bank.0) {
            return AdventureOutcome::default();
        }
        self.loot_bank(rules, hero, bank, sink)
    }

    /// The reward of the bank at `index` of kind `bank` to the hero `hero`, then it is looted,
    /// staying on the map (ADV-BNK-005, ADV-RWD-001; increment 13's design note §8, D-68): its
    /// drawn level's as a reward record's ([`Self::give_record`]): **the creature**, when the
    /// level has one, onto the hero's army as `army_add_stack` (ADV-ARM-007), else the army
    /// dialog's group ([`AdventureOutcome::no_room`], GAM-DEC-004: what is left in it lost); **the
    /// artifacts** in the draw's order, relics first, each the hero cannot receive lost
    /// (`h3_heroes::grant`, ADR-0013 point 15.4); **the resources** as the outcome's gains; no
    /// experience, skill or spell. The record's own order (the hero's half, the resources, the
    /// creatures) against the original's (the creatures, the artifacts, the resources): none
    /// reads another's (*agent's call*, design note §8).
    // rule: ADV-BNK-005, ADV-RWD-001
    pub(super) fn loot_bank<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        hero: HeroId,
        (index, bank): (usize, &Bank),
        sink: &mut S,
    ) -> AdventureOutcome {
        let record = self
            .object_at(index)
            .resolved
            .bank()
            .and_then(|(level, artifacts)| {
                let level = bank.levels.get(usize::from(level));
                level.map(|level| bank_record(level, artifacts))
            });
        self.visits_mut().empty(index);
        record
            .map(|record| self.give_record(rules, hero, &record, sink))
            .unwrap_or_default()
    }
}

/// A bank level's reward as a reward record (ADV-BNK-005, ADV-RWD-001): the drawn `artifacts` as
/// its hero's half, the level's resources and its creature, in slot 0.
fn bank_record(level: &BankLevel, artifacts: &[ArtifactTypeId]) -> RewardRecord {
    let to_amount = |amount: u32| i32::try_from(amount).unwrap_or(i32::MAX);
    RewardRecord {
        hero: HeroReward {
            artifacts: artifacts.to_vec(),
            ..HeroReward::default()
        },
        resources: level
            .resources
            .iter()
            .map(|&(resource, amount)| (resource, to_amount(amount)))
            .collect(),
        creatures: level
            .creature
            .map(|(creature, count)| ArmyStack {
                slot: 0,
                creature,
                count: u32::from(count),
            })
            .into_iter()
            .collect(),
    }
}
