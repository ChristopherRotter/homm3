//! Battle state: stacks, round bookkeeping, RNG and outcome (ADR-0004 §8: flat, index-addressed,
//! fixed-capacity, cheap to clone).

use h3_content::{
    Ability, AreaShot, CreatureFlag, CreatureType, CreatureTypeId, MAX_ARMY_SLOTS, Ruleset,
};
use h3_contracts::{Army, ArmyStack, BattleOutcome, BattleSetup, BattleSide};
use h3_kernel::{CanonicalHash, FixedVec, Rng, StateHasher, StreamId};

use crate::error::SetupError;
use crate::event::NoopSink;
use crate::geometry::{Field, Footprint, Hex, HexSet};
use crate::morale::{self, Modifiers};
use crate::turn;

/// Maximum number of stacks in a battle (both sides).
pub const STACK_CAPACITY: usize = 2 * MAX_ARMY_SLOTS;

/// Version of the battle-state hash encoding ([`BattleState::state_hash`]). Bump on any change.
pub const BATTLE_STATE_HASH_VERSION: u32 = 5;

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
    /// The stack has no retaliation left this round (BAT-RET-001, BAT-RET-003).
    pub const RETALIATED: Self = Self(1 << 2);
    /// The stack is defending until its next activation (BAT-TRN-004).
    pub const DEFENDING: Self = Self(1 << 3);
    /// The stack had a good-morale turn this round (BAT-MOR-003).
    pub const HAD_MORALE: Self = Self(1 << 4);

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

/// Battle traits of a stack, derived from its creature's flags and abilities at setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StackTraits(u32);

impl StackTraits {
    /// Occupies two hexes (flag `two_hex`, BAT-GEO-002).
    pub const TWO_HEX: Self = Self(1);
    /// Flies or teleports (flag `flying` or ability `teleports`, BAT-MOV-002).
    pub const FLIES: Self = Self(1 << 1);
    /// A shooter (flag `shooter`, BAT-RNG-001).
    pub const SHOOTER: Self = Self(1 << 2);
    /// No melee penalty (flag `no_melee_penalty`, BAT-RNG-003).
    pub const NO_MELEE_PENALTY: Self = Self(1 << 3);
    /// No distance penalty (ability `no_distance_penalty`, BAT-RNG-002).
    pub const NO_DISTANCE_PENALTY: Self = Self(1 << 4);
    /// Strikes twice (flag `strikes_twice`; shooters shoot twice, BAT-RNG-001).
    pub const STRIKES_TWICE: Self = Self(1 << 5);
    /// Its shots hit an area (ability `area_shot`, BAT-RNG-004).
    pub const AREA_SHOT: Self = Self(1 << 6);
    /// Its area shot is a death cloud that spares stacks that are not alive (else a fireball
    /// that spares fire-immune stacks).
    pub const DEATH_CLOUD: Self = Self(1 << 7);
    /// Alive (flag `alive`).
    pub const ALIVE: Self = Self(1 << 8);
    /// Immune to fire (flag `fire_immune`).
    pub const FIRE_IMMUNE: Self = Self(1 << 9);
    /// Its strikes are not retaliated (flag `no_enemy_retaliation`, BAT-RET-002).
    pub const NO_ENEMY_RETALIATION: Self = Self(1 << 10);
    /// Its melee strikes also hit the stack behind the target (flag `breath_attack`,
    /// BAT-ATK-003).
    pub const BREATH: Self = Self(1 << 11);
    /// Its melee strikes hit every adjacent enemy (ability `attacks_all_adjacent`, BAT-ATK-003).
    pub const ALL_ADJACENT: Self = Self(1 << 12);
    /// Its melee strikes hit the target and the hexes beside it (ability
    /// `three_headed_attack`, BAT-ATK-003).
    pub const THREE_HEADED: Self = Self(1 << 13);
    /// Returns to its starting hex after a melee attack (ability `strike_and_return`,
    /// BAT-ATK-004).
    pub const STRIKE_AND_RETURN: Self = Self(1 << 14);
    /// Regenerates its top creature at the start of its turn (ability `regeneration`,
    /// BAT-TRN-005).
    pub const REGENERATES: Self = Self(1 << 15);
    /// Unaffected by morale: its morale is always 0 (flag `no_morale`, BAT-MOR-001).
    pub const NO_MORALE: Self = Self(1 << 16);

    /// Whether all traits of `other` are set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The set with the traits of `other` added if `on`.
    #[must_use]
    pub const fn with(self, other: Self, on: bool) -> Self {
        if on { Self(self.0 | other.0) } else { self }
    }

    /// The raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// The traits of a creature type.
    pub(crate) fn of(creature: &CreatureType, abilities: &[h3_content::CompiledAbility]) -> Self {
        let flags = creature.flags;
        let has = |f| flags.contains(f);
        let ability = |f: fn(&h3_content::CompiledAbility) -> bool| abilities.iter().any(f);
        Self::default()
            .with(Self::TWO_HEX, has(CreatureFlag::TwoHex))
            .with(
                Self::FLIES,
                has(CreatureFlag::Flying) || ability(|a| matches!(a, Ability::Teleports)),
            )
            .with(Self::SHOOTER, has(CreatureFlag::Shooter))
            .with(Self::NO_MELEE_PENALTY, has(CreatureFlag::NoMeleePenalty))
            .with(
                Self::NO_DISTANCE_PENALTY,
                ability(|a| matches!(a, Ability::NoDistancePenalty)),
            )
            .with(Self::STRIKES_TWICE, has(CreatureFlag::StrikesTwice))
            .with(
                Self::AREA_SHOT,
                ability(|a| matches!(a, Ability::AreaShot { .. })),
            )
            .with(
                Self::DEATH_CLOUD,
                ability(|a| {
                    matches!(
                        a,
                        Ability::AreaShot {
                            effect: AreaShot::DeathCloud
                        }
                    )
                }),
            )
            .with(Self::ALIVE, has(CreatureFlag::Alive))
            .with(Self::FIRE_IMMUNE, has(CreatureFlag::FireImmune))
            .with(
                Self::NO_ENEMY_RETALIATION,
                has(CreatureFlag::NoEnemyRetaliation),
            )
            .with(Self::BREATH, has(CreatureFlag::BreathAttack))
            .with(
                Self::ALL_ADJACENT,
                ability(|a| matches!(a, Ability::AttacksAllAdjacent)),
            )
            .with(
                Self::THREE_HEADED,
                ability(|a| matches!(a, Ability::ThreeHeadedAttack)),
            )
            .with(
                Self::STRIKE_AND_RETURN,
                ability(|a| matches!(a, Ability::StrikeAndReturn)),
            )
            .with(
                Self::REGENERATES,
                ability(|a| matches!(a, Ability::Regeneration)),
            )
            .with(Self::NO_MORALE, has(CreatureFlag::NoMorale))
    }
}

/// `Stack::retaliations` value for unlimited retaliations (BAT-RET-003).
pub const UNLIMITED_RETALIATIONS: u8 = u8::MAX;

/// Retaliations per round of a creature: one, `1 + count` with extra retaliations, or
/// unlimited (BAT-RET-003).
// rule: BAT-RET-003
pub(crate) fn retaliations_per_round(abilities: &[h3_content::CompiledAbility]) -> u8 {
    abilities.iter().fold(1, |n, a| match a {
        Ability::ExtraRetaliations { count } => {
            count.saturating_add(1).min(UNLIMITED_RETALIATIONS - 1)
        }
        Ability::UnlimitedRetaliations => UNLIMITED_RETALIATIONS,
        _ => n,
    })
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
    /// Creatures at the start of the battle (resurrection limit, BAT-DMG-008).
    pub base_count: u32,
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
    /// Shots left (BAT-RNG-001).
    pub shots: u16,
    /// Retaliations per round (`UNLIMITED_RETALIATIONS` for unlimited, BAT-RET-003).
    pub retaliations: u8,
    /// Retaliations left this round.
    pub retaliations_left: u8,
    /// Morale (BAT-MOR-001), materialised from the battle's modifiers.
    pub morale: i8,
    /// Luck (BAT-LCK-001), materialised from the battle's modifiers.
    pub luck: i8,
    /// Battle traits (two-hex, flying, …).
    pub traits: StackTraits,
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
            base_count: 0,
            top_hp: 0,
            attack: 0,
            defense: 0,
            damage_min: 0,
            damage_max: 0,
            max_hp: 0,
            speed: 0,
            shots: 0,
            retaliations: 1,
            retaliations_left: 1,
            morale: 0,
            luck: 0,
            traits: StackTraits::default(),
            flags: StackFlags::default(),
        }
    }
}

impl Stack {
    /// Whether the stack still has creatures.
    pub const fn is_alive(&self) -> bool {
        self.count > 0
    }

    /// Whether the stack has trait `t`.
    pub const fn has(&self, t: StackTraits) -> bool {
        self.traits.contains(t)
    }

    /// Whether the stack occupies two hexes.
    pub const fn is_two_hex(&self) -> bool {
        self.traits.contains(StackTraits::TWO_HEX)
    }

    /// The hexes the stack covers on `field` (BAT-GEO-002). Placement and movement keep the
    /// footprint of every stack on the field.
    pub fn footprint(&self, field: Field) -> Footprint {
        field
            .footprint(self.hex, self.is_two_hex(), self.side)
            .unwrap_or(Footprint {
                head: self.hex,
                tail: None,
            })
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
        h.write_u32(self.base_count);
        h.write_u32(self.top_hp);
        h.write_u16(self.attack);
        h.write_u16(self.defense);
        h.write_u16(self.damage_min);
        h.write_u16(self.damage_max);
        h.write_u32(self.max_hp);
        h.write_u8(self.speed);
        h.write_u16(self.shots);
        h.write_u8(self.retaliations);
        h.write_u8(self.retaliations_left);
        h.write_i32(i32::from(self.morale));
        h.write_i32(i32::from(self.luck));
        h.write_u32(self.traits.bits());
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
    pub(crate) obstacles: HexSet,
    pub(crate) round: u16,
    pub(crate) active: Option<StackId>,
    pub(crate) last_side_acted: Option<BattleSide>,
    pub(crate) winner: Option<BattleSide>,
    pub(crate) modifiers: Modifiers,
    pub(crate) led_by_hero: [bool; 2],
    pub(crate) rng: Rng,
}

impl BattleState {
    /// Starts a battle: validates the setup against the ruleset, places the obstacles and the
    /// armies (BAT-SET-001, BAT-GEO-003), seeds the battle RNG stream from `seed` and activates
    /// the first stack.
    pub fn new(ruleset: &Ruleset, setup: &BattleSetup, seed: u64) -> Result<Self, SetupError> {
        let params = ruleset.battle();
        let field = Field::new(params.field);
        let obstacles = obstacles(field, &setup.obstacles)?;
        let mut taken = obstacles;
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
                // rule: BAT-SET-001
                let desired = field.hex(column, i32::from(row)).unwrap_or_default();
                let abilities = ruleset.abilities(s.creature);
                let traits = StackTraits::of(creature, abilities);
                let retaliations = retaliations_per_round(abilities);
                let two_hex = traits.contains(StackTraits::TWO_HEX);
                let head = closest_free(field, desired, two_hex, side, taken)
                    .ok_or(SetupError::NoRoom { side, slot: s.slot })?;
                let stack = Stack {
                    creature: s.creature,
                    side,
                    slot: s.slot,
                    hex: head,
                    count: s.count,
                    base_count: s.count,
                    top_hp: creature.hit_points,
                    attack: creature.attack,
                    defense: creature.defense,
                    damage_min: creature.damage_min,
                    damage_max: creature.damage_max,
                    max_hp: creature.hit_points,
                    speed: creature.speed,
                    shots: creature.shots,
                    retaliations,
                    retaliations_left: retaliations,
                    morale: 0,
                    luck: 0,
                    traits,
                    flags: StackFlags::default(),
                };
                stack.footprint(field).hexes().for_each(|h| taken.insert(h));
                // At most `army_slots <= MAX_ARMY_SLOTS` stacks per side, so `len` stays below
                // STACK_CAPACITY = 2 * MAX_ARMY_SLOTS.
                placed[len] = stack;
                len += 1;
            }
        }
        let mut state = Self {
            stacks: FixedVec::from_array_prefix(placed, len),
            obstacles,
            round: 1,
            active: None,
            last_side_acted: None,
            winner: None,
            modifiers: Modifiers::new(),
            led_by_hero: setup.led_by_hero,
            rng: Rng::for_stream(seed, StreamId::BATTLE),
        };
        morale::setup(&mut state, ruleset);
        // The first activation runs the usual checks (bad morale, BAT-MOR-004); its events are not
        // reported, like the start of round 1.
        turn::activate_next(&mut state, ruleset, &mut NoopSink);
        Ok(state)
    }

    /// The hexes blocked by obstacles (BAT-GEO-003).
    pub const fn obstacles(&self) -> HexSet {
        self.obstacles
    }

    /// The hexes that block `except`'s movement: obstacles and the footprints of every other
    /// living stack (BAT-MOV-001).
    pub(crate) fn blocked_for(&self, field: Field, except: StackId) -> HexSet {
        let mut blocked = self.obstacles;
        for (id, s) in (0u8..).zip(self.stacks.iter()) {
            if s.is_alive() && StackId(id) != except {
                s.footprint(field).hexes().for_each(|h| blocked.insert(h));
            }
        }
        blocked
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
    /// Encoding v5: version, round, active, last side, winner, stacks (each with base count,
    /// shots, retaliations, morale, luck and traits), obstacles, modifiers, hero flags, RNG
    /// state.
    pub fn state_hash(&self) -> u64 {
        h3_kernel::hash::hash_of(self)
    }
}

/// Validates the setup's obstacle hexes: inside the field, not in a side column, each once.
// rule: BAT-GEO-003
fn obstacles(field: Field, hexes: &[u8]) -> Result<HexSet, SetupError> {
    let mut set = HexSet::new();
    for &raw in hexes {
        let hex = Hex(raw);
        if !field.is_standable(hex) || set.contains(hex) {
            return Err(SetupError::InvalidObstacle(raw));
        }
        set.insert(hex);
    }
    Ok(set)
}

/// The head hex closest to `desired` where a stack of this shape fits: every hex of its
/// footprint standable and not `taken`. Ties go to the column nearer the centre of the field,
/// then the row nearer `desired`'s, then the lower hex index (BAT-SET-001).
// rule: BAT-SET-001
pub(crate) fn closest_free(
    field: Field,
    desired: Hex,
    two_hex: bool,
    side: BattleSide,
    taken: HexSet,
) -> Option<Hex> {
    let (_, dy) = field.coords(desired);
    (0..field.hex_count())
        .filter_map(|i| u8::try_from(i).ok().map(Hex))
        .filter_map(|h| field.footprint(h, two_hex, side))
        .filter(|fp| {
            fp.hexes()
                .all(|h| field.is_standable(h) && !taken.contains(h))
        })
        .map(|fp| fp.head)
        .min_by_key(|&h| {
            let (x, y) = field.coords(h);
            let toward_centre = match side {
                BattleSide::Attacker => -x,
                BattleSide::Defender => x,
            };
            (field.distance(desired, h), toward_centre, (y - dy).abs(), h)
        })
}

impl CanonicalHash for BattleState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(BATTLE_STATE_HASH_VERSION);
        h.write_u16(self.round);
        self.active.map(|s| s.0).hash_into(h);
        self.last_side_acted.map(side_code).hash_into(h);
        self.winner.map(side_code).hash_into(h);
        self.stacks.hash_into(h);
        self.obstacles.hash_into(h);
        self.modifiers.hash_into(h);
        h.write_bool(self.led_by_hero[0]);
        h.write_bool(self.led_by_hero[1]);
        self.rng.hash_into(h);
    }
}
