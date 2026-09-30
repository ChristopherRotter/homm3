//! Battle state: stacks, round bookkeeping, RNG and outcome (ADR-0004 §8: flat, index-addressed,
//! fixed-capacity, cheap to clone).

use h3_content::{CreatureType, CreatureTypeId, MAX_ARMY_SLOTS, Ruleset};
use h3_contracts::{Army, ArmyStack, BattleOutcome, BattleSetup, BattleSide};
use h3_kernel::{CanonicalHash, FixedVec, Rng, StateHasher, StreamId};

use crate::error::SetupError;
use crate::geometry::{Field, Hex};
use crate::turn;

/// Maximum number of stacks in a battle (both sides).
pub const STACK_CAPACITY: usize = 2 * MAX_ARMY_SLOTS;

/// Version of the battle-state hash encoding ([`BattleState::state_hash`]). Bump on any change.
pub const BATTLE_STATE_HASH_VERSION: u32 = 1;

/// Index of a stack in its battle (attacker stacks first, in slot order, then defender stacks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct StackId(pub u8);

impl StackId {
    /// The raw index.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Per-round flags of a stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StackFlags(u8);

impl StackFlags {
    /// The stack has taken its turn this round (moved, attacked or defended).
    pub const ACTED: Self = Self(1);
    /// The stack has waited this round (BAT-TRN-002).
    pub const WAITED: Self = Self(1 << 1);
    /// The stack has used its retaliation this round (BAT-RET-001).
    pub const RETALIATED: Self = Self(1 << 2);
    /// The stack is defending until its next activation (BAT-TRN-004).
    pub const DEFENDING: Self = Self(1 << 3);

    /// Whether all flags of `other` are set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Sets or clears the flags of `other`.
    pub fn set(&mut self, other: Self, on: bool) {
        if on {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }

    /// The raw bits.
    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// A stack in battle. Creature stats are copied in at setup so the hot path needs no lookups
/// (and later effects can modify them per stack).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack {
    /// Creature type.
    pub creature: CreatureTypeId,
    /// Owning side.
    pub side: BattleSide,
    /// Army slot it came from.
    pub slot: u8,
    /// Position.
    pub hex: Hex,
    /// Living creatures (0 = dead).
    pub count: u32,
    /// Hit points of the top creature (`1..=max_hp` while alive, 0 when dead).
    pub top_hp: u32,
    /// Attack skill.
    pub attack: u16,
    /// Defense skill (before the Defend bonus).
    pub defense: u16,
    /// Minimum damage per creature.
    pub damage_min: u16,
    /// Maximum damage per creature.
    pub damage_max: u16,
    /// Hit points per creature.
    pub max_hp: u32,
    /// Speed.
    pub speed: u8,
    /// Per-round flags.
    pub flags: StackFlags,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            creature: CreatureTypeId::default(),
            side: BattleSide::Attacker,
            slot: 0,
            hex: Hex(0),
            count: 0,
            top_hp: 0,
            attack: 0,
            defense: 0,
            damage_min: 0,
            damage_max: 0,
            max_hp: 0,
            speed: 0,
            flags: StackFlags::default(),
        }
    }
}

impl Stack {
    /// Whether the stack still has creatures.
    pub const fn is_alive(&self) -> bool {
        self.count > 0
    }

    /// Total remaining hit points: `(count - 1) · max_hp + top_hp` (BAT-DMG-003).
    pub fn total_hp(&self) -> u64 {
        if self.count == 0 {
            return 0;
        }
        u64::from(self.count - 1) * u64::from(self.max_hp) + u64::from(self.top_hp)
    }
}

impl CanonicalHash for Stack {
    fn hash_into(&self, h: &mut StateHasher) {
        self.creature.hash_into(h);
        h.write_u8(side_code(self.side));
        h.write_u8(self.slot);
        self.hex.hash_into(h);
        h.write_u32(self.count);
        h.write_u32(self.top_hp);
        h.write_u16(self.attack);
        h.write_u16(self.defense);
        h.write_u16(self.damage_min);
        h.write_u16(self.damage_max);
        h.write_u32(self.max_hp);
        h.write_u8(self.speed);
        h.write_u8(self.flags.bits());
    }
}

pub(crate) const fn side_code(side: BattleSide) -> u8 {
    match side {
        BattleSide::Attacker => 0,
        BattleSide::Defender => 1,
    }
}

/// The complete, authoritative state of one battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {
    pub(crate) stacks: FixedVec<Stack, STACK_CAPACITY>,
    pub(crate) round: u16,
    pub(crate) active: Option<StackId>,
    pub(crate) last_side_acted: Option<BattleSide>,
    pub(crate) winner: Option<BattleSide>,
    pub(crate) rng: Rng,
}

impl BattleState {
    /// Starts a battle: validates the setup against the ruleset, places the armies
    /// (BAT-SET-001), seeds the battle RNG stream from `seed` and activates the first stack.
    pub fn new(ruleset: &Ruleset, setup: &BattleSetup, seed: u64) -> Result<Self, SetupError> {
        let params = ruleset.battle();
        let field = Field::new(params.field);
        let mut placed = [Stack::default(); STACK_CAPACITY];
        let mut len = 0;
        for side in BattleSide::BOTH {
            let army = setup.army(side);
            if army.stacks.is_empty() {
                return Err(SetupError::EmptyArmy(side));
            }
            // Index the army by slot: validates slots and yields slot order without sorting.
            let mut by_slot: [Option<(ArmyStack, CreatureType)>; MAX_ARMY_SLOTS] =
                [None; MAX_ARMY_SLOTS];
            for s in &army.stacks {
                let slot = s.slot;
                let entry = by_slot
                    .get_mut(usize::from(slot))
                    .filter(|_| slot < params.army_slots)
                    .ok_or(SetupError::SlotOutOfRange { side, slot })?;
                if entry.is_some() {
                    return Err(SetupError::DuplicateSlot { side, slot });
                }
                if s.count == 0 {
                    return Err(SetupError::EmptyStack { side, slot });
                }
                let creature = ruleset
                    .creature(s.creature)
                    .ok_or(SetupError::UnknownCreature { side, slot })?;
                *entry = Some((*s, *creature));
            }
            let n = army.stacks.len();
            let column = match side {
                BattleSide::Attacker => field.first_standable_column(),
                BattleSide::Defender => field.last_standable_column(),
            };
            for (i, (s, creature)) in by_slot.iter().flatten().enumerate() {
                // The compiler guarantees a formation row for every army size and index.
                let row = params.formation_row(n, i).unwrap_or(0);
                // At most `army_slots <= MAX_ARMY_SLOTS` stacks per side, so `len` stays below
                // STACK_CAPACITY = 2 * MAX_ARMY_SLOTS.
                placed[len] = Stack {
                    creature: s.creature,
                    side,
                    slot: s.slot,
                    // rule: BAT-SET-001
                    hex: field.hex(column, i32::from(row)).unwrap_or_default(),
                    count: s.count,
                    top_hp: creature.hit_points,
                    attack: creature.attack,
                    defense: creature.defense,
                    damage_min: creature.damage_min,
                    damage_max: creature.damage_max,
                    max_hp: creature.hit_points,
                    speed: creature.speed,
                    flags: StackFlags::default(),
                };
                len += 1;
            }
        }
        let mut state = Self {
            stacks: FixedVec::from_array_prefix(placed, len),
            round: 1,
            active: None,
            last_side_acted: None,
            winner: None,
            rng: Rng::for_stream(seed, StreamId::BATTLE),
        };
        state.active = turn::next_active(&state);
        Ok(state)
    }

    /// All stacks (dead ones included; they keep their id).
    pub fn stacks(&self) -> &[Stack] {
        self.stacks.as_slice()
    }

    /// A stack by id.
    pub fn stack(&self, id: StackId) -> Option<&Stack> {
        self.stacks.get(id.index())
    }

    /// The active stack and its id, if the battle is not over.
    pub(crate) fn active_stack(&self) -> Option<(StackId, Stack)> {
        self.active.and_then(|id| self.stack(id).map(|s| (id, *s)))
    }

    /// The stack `id`, which must be valid (ids produced by this state or checked by validation).
    pub(crate) fn at(&self, id: StackId) -> &Stack {
        &self.stacks.as_slice()[id.index()]
    }

    /// The stack `id`, mutably; `id` must be valid.
    pub(crate) fn at_mut(&mut self, id: StackId) -> &mut Stack {
        &mut self.stacks.as_mut_slice()[id.index()]
    }

    /// The current round (1-based).
    pub const fn round(&self) -> u16 {
        self.round
    }

    /// The stack whose turn it is, or `None` once the battle is over.
    pub const fn active(&self) -> Option<StackId> {
        self.active
    }

    /// The side whose turn it is, or `None` once the battle is over.
    pub fn side_to_act(&self) -> Option<BattleSide> {
        self.active.and_then(|id| self.stack(id)).map(|s| s.side)
    }

    /// The side of the last stack that took a turn (speed tie-breaks, BAT-TRN-001).
    pub const fn last_side_acted(&self) -> Option<BattleSide> {
        self.last_side_acted
    }

    /// The winner, once the battle is over (BAT-END-001).
    pub const fn winner(&self) -> Option<BattleSide> {
        self.winner
    }

    /// Whether the battle is over.
    pub const fn is_over(&self) -> bool {
        self.winner.is_some()
    }

    /// The outcome for the orchestrator, once the battle is over: winner and surviving stacks.
    pub fn outcome(&self) -> Option<BattleOutcome> {
        let winner = self.winner?;
        let survivors = BattleSide::BOTH.map(|side| Army {
            stacks: self
                .stacks
                .iter()
                .filter(|s| s.side == side && s.is_alive())
                .map(|s| ArmyStack {
                    slot: s.slot,
                    creature: s.creature,
                    count: s.count,
                })
                .collect(),
        });
        Some(BattleOutcome { winner, survivors })
    }

    /// Canonical hash of the full state, including hidden parts such as the RNG (desync detection
    /// and replay verification; never sent to untrusted clients, docs/06-security.md).
    ///
    /// Encoding v1: version, round, active, last side, winner, stacks, RNG state.
    pub fn state_hash(&self) -> u64 {
        h3_kernel::hash::hash_of(self)
    }
}

impl CanonicalHash for BattleState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(BATTLE_STATE_HASH_VERSION);
        h.write_u16(self.round);
        self.active.map(|s| s.0).hash_into(h);
        self.last_side_acted.map(side_code).hash_into(h);
        self.winner.map(side_code).hash_into(h);
        self.stacks.hash_into(h);
        self.rng.hash_into(h);
    }
}
