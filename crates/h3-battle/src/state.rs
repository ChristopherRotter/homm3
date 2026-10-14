//! Battle state: stacks, round bookkeeping, RNG and outcome (ADR-0004 §8: flat, index-addressed,
//! fixed-capacity, cheap to clone).

use alloc::vec::Vec;
use h3_content::{
    CreatureFlag, CreatureTypeId, MAX_ARMY_SLOTS, OBSTACLE_ANCHORS, TerrainId,
    UNLIMITED_BALLISTA_SHOTS,
};
use h3_content_battle::{
    Ability, AreaShot, BattleRuleset, BattlefieldId, CreatureType, MAX_SIDE_STACKS,
    WAR_MACHINE_KINDS,
};

use h3_contracts::{
    Army, ArmyStack, BattleOutcome, BattleResult, BattleSetup, BattleSide, HeroOutcome,
    StackOrigin, StackRecord,
};
use h3_kernel::{CanonicalHash, FixedVec, Rng, StateHasher, StreamId};

use crate::binds::Binds;
use crate::casters::Casts;
use crate::created::{Created, TEMPLATES};
use crate::effects::{ActiveEffect, Effects, SpellValues, Status};
use crate::error::SetupError;
use crate::event::{NoopSink, Sink};
use crate::geometry::{Field, Footprint, Hex, HexSet};
use crate::hero::{self, HeroRecord, HeroView};
use crate::magic::MagicLimits;
use crate::morale::{self, Modifiers};
use crate::natural::{NaturalObstacles, RemovableObstacle};
use crate::obstacles::{self, SpellObstacle, SpellObstacles};
use crate::siege::{self, SiegeState};
use crate::tactics::{self, TacticsPhase};
use crate::vitality::Vitality;
use crate::{flee, terrain, turn};

/// Most war machines per side, one of each kind (BAT-WAR-001).
pub const MAX_WAR_MACHINES: usize = WAR_MACHINE_KINDS;

/// Stack slots per side, as the original keeps 21 stack records per side (ADR-0014 point 6,
/// `docs/formats/installation.md` §7.7): the army slots and war machines, and room for stacks
/// created in battle (E04b T6).
pub const SIDE_STACK_SLOTS: usize = 21;

/// Maximum number of stacks in a battle (both sides).
pub const STACK_CAPACITY: usize = 2 * SIDE_STACK_SLOTS;

// A side's army slots and war machines fit its stack slots for every ruleset (the content
// allows at most `MAX_ARMY_SLOTS` army slots).
const _: () = assert!(MAX_ARMY_SLOTS + MAX_WAR_MACHINES <= SIDE_STACK_SLOTS);
// A side never has more stacks than its slots: it creates stacks only below the side stack
// limit, which the content keeps at most `MAX_SIDE_STACKS` (BAT-MAG-037).
const _: () = assert!(MAX_SIDE_STACKS <= SIDE_STACK_SLOTS);

/// [`Stack::slot`] of a stack created in battle (a clone or a summoned stack, BAT-MAG-037): after
/// the army slots and the war machines in the turn order's tie-break (BAT-TRN-001).
pub const CREATED_SLOT: u8 = u8::MAX;

/// Version of the battle-state hash encoding ([`BattleState::state_hash`]). Bump on any change.
pub const BATTLE_STATE_HASH_VERSION: u32 = 46;

/// Index of a stack in its battle (attacker stacks first, in slot order, then defender stacks, then
/// the stacks created in the battle in the order they appear, BAT-MAG-037).
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
    /// The stack's turn has started this round (the original's +0x4F0, `installation.md`
    /// §7.14): its turn-start rules ran, and while blind it is passed over (BAT-TRN-001,
    /// BAT-MAG-024).
    pub const STARTED: Self = Self(1 << 5);

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
    /// Unaffected by morale: its morale is always 0 (flag `no_morale` or `war_machine`,
    /// BAT-MOR-001).
    pub const NO_MORALE: Self = Self(1 << 16);
    /// A war machine: fixed place, automatic, no retaliation, does not count as living for the
    /// end of the battle (flag `war_machine`, BAT-WAR-001).
    pub const WAR_MACHINE: Self = Self(1 << 17);
    /// Supplies its side's shooters with ammunition and never acts (ability `ammo_supply`,
    /// BAT-WAR-004).
    pub const AMMO_SUPPLY: Self = Self(1 << 18);
    /// Heals allied stacks on its turn (ability `first_aid`, BAT-WAR-003).
    pub const FIRST_AID: Self = Self(1 << 19);
    /// Shoots on without shots left: the shooting war machine under quirk Q-0004
    /// (`quirk.battle.unlimited_ballista_shots`, BAT-WAR-002).
    pub const UNLIMITED_SHOTS: Self = Self(1 << 20);
    /// May shoot while an enemy stands next to it (its hero's `ShootWhenAdjacent`, Bow of the
    /// Sharpshooter; BAT-HER-006).
    pub const SHOOTS_WHEN_ADJACENT: Self = Self(1 << 21);
    /// Fights on its native terrain (BAT-TER-001): its side may see the other side's hidden
    /// obstacles (BAT-MAG-031).
    pub const NATIVE: Self = Self(1 << 22);
    /// Created in battle, a summoned stack or a clone (BAT-MAG-037; the original's flag
    /// `0x400000`): no corpse, no victim of a sacrifice, reported as created in the outcome, and
    /// a side left with only such stacks does not win.
    pub const SUMMONED: Self = Self(1 << 23);
    /// A clone (BAT-MAG-038; the original's flag `0x800000`): any damage kills all its creatures,
    /// and it vanishes when its rounds run out or its original dies.
    pub const CLONE: Self = Self(1 << 24);
    /// Raised from a corpse by a corpse raiser (a Pit Lord's Demons, BAT-MAG-052): an army stack
    /// of the dead stack's army slot ([`Stack::slot`]), but after the army slots and the war
    /// machines in the turn order's tie-break, as a created stack (the original's new record).
    pub const RAISED: Self = Self(1 << 25);
    /// Casts on its own at its activation (ability `periodic_spell_caster`, the Enchanter;
    /// BAT-MAG-053): the activation's cheap test.
    pub const PERIODIC_CASTER: Self = Self(1 << 26);
    /// Takes mana from the enemy hero at its activations (ability `mana_drain`, the Wraith;
    /// BAT-MAG-056): the activation's cheap test.
    pub const MANA_DRAIN: Self = Self(1 << 27);
    /// Has a spell of its own (ability `innate_spell`: the Efreet Sultan's Fire Shield, the Faerie
    /// Dragon's Magic Mirror; BAT-MAG-057, BAT-MAG-058): the strike's cheap test before it looks
    /// for an innate Fire Shield (a stack with only an innate mirror passes it and reflects
    /// nothing).
    pub const INNATE_SPELL: Self = Self(1 << 28);
    /// Comes back after the action that killed it (ability `rebirth`, the Phoenix; BAT-MAG-059):
    /// the rebirth's cheap test.
    pub const REBIRTH: Self = Self(1 << 29);
    /// Frightens the other side's stacks (ability `fear`, the Azure Dragon; BAT-MAG-060): the
    /// pick's cheap test, and its own immunity.
    pub const FEAR: Self = Self(1 << 30);
    /// Its shots are not halved by a town's walls (ability `no_wall_penalty`, or its hero's
    /// `NoRangePenalty`: the original's Golden Bow and Bow of the Sharpshooter; BAT-SIE-008).
    pub const NO_WALL_PENALTY: Self = Self(1 << 31);

    /// Whether all traits of `other` are set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The set with the traits of `other` added if `on`.
    #[must_use]
    pub const fn with(self, other: Self, on: bool) -> Self {
        if on { Self(self.0 | other.0) } else { self }
    }

    /// The set without the traits of `other`.
    #[must_use]
    pub(crate) const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// The raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// The traits of a creature type.
    pub(crate) fn of(
        creature: &CreatureType,
        abilities: &[h3_content_battle::CompiledAbility],
    ) -> Self {
        let flags = creature.flags;
        let has = |f| flags.contains(f);
        let ability = |f: fn(&h3_content_battle::CompiledAbility) -> bool| abilities.iter().any(f);
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
            .with(
                Self::NO_WALL_PENALTY,
                ability(|a| matches!(a, Ability::NoWallPenalty)),
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
            .with(
                Self::NO_MORALE,
                has(CreatureFlag::NoMorale) || has(CreatureFlag::WarMachine),
            )
            .with(Self::WAR_MACHINE, has(CreatureFlag::WarMachine))
            .with(
                Self::AMMO_SUPPLY,
                ability(|a| matches!(a, Ability::AmmoSupply)),
            )
            .with(
                Self::FIRST_AID,
                ability(|a| matches!(a, Ability::FirstAid { .. })),
            )
            .with(
                Self::PERIODIC_CASTER,
                ability(|a| matches!(a, Ability::PeriodicSpellCaster { .. })),
            )
            .with(
                Self::MANA_DRAIN,
                ability(|a| matches!(a, Ability::ManaDrain { .. })),
            )
            .with(
                Self::INNATE_SPELL,
                ability(|a| matches!(a, Ability::InnateSpell { .. })),
            )
            .with(
                Self::REBIRTH,
                ability(|a| matches!(a, Ability::Rebirth { .. })),
            )
            .with(Self::FEAR, ability(|a| matches!(a, Ability::Fear { .. })))
    }
}

/// The kind of a war machine, which decides its place (BAT-WAR-001): a shooting machine
/// (Ballista), an ammunition supply (Ammo Cart) or first aid (First Aid Tent).
fn war_machine_kind(
    creature: &CreatureType,
    abilities: &[h3_content_battle::CompiledAbility],
) -> Option<usize> {
    let has = |f: fn(&h3_content_battle::CompiledAbility) -> bool| abilities.iter().any(f);
    if !creature.flags.contains(CreatureFlag::WarMachine) {
        None
    } else if creature.flags.contains(CreatureFlag::Shooter) {
        Some(0)
    } else if has(|a| matches!(a, Ability::AmmoSupply)) {
        Some(1)
    } else if has(|a| matches!(a, Ability::FirstAid { .. })) {
        Some(2)
    } else {
        None
    }
}

/// The war machine stacks of `side`, by kind, placed with their front hexes on the ruleset's
/// hexes (mirrored for the defender) and reserved in `taken` (BAT-WAR-001).
// rule: BAT-WAR-001
fn war_machines(
    ruleset: &BattleRuleset,
    field: Field,
    setup: &BattleSetup,
    side: BattleSide,
    taken: &mut HexSet,
) -> Result<[Option<Stack>; MAX_WAR_MACHINES], SetupError> {
    let params = ruleset.battle();
    let mut out: [Option<Stack>; MAX_WAR_MACHINES] = [None; MAX_WAR_MACHINES];
    for &id in &setup.war_machines[side.index()] {
        let invalid = SetupError::InvalidWarMachine { side };
        let creature = ruleset.creature(id).ok_or(invalid)?;
        let kind = war_machine_kind(creature, ruleset.abilities(id)).ok_or(invalid)?;
        if out[kind].is_some() {
            return Err(invalid);
        }
        // `kind < MAX_WAR_MACHINES <= MAX_ARMY_SLOTS`.
        let slot = u8::try_from(MAX_ARMY_SLOTS + kind).unwrap_or(u8::MAX);
        let (x, y) = params.war_machine_positions[kind];
        let x = match side {
            BattleSide::Attacker => i32::from(x),
            BattleSide::Defender => i32::from(params.field.width) - 1 - i32::from(x),
        };
        // The compiler checked that the front hexes are on the field, with room behind them.
        let front = field.hex(x, i32::from(y)).unwrap_or_default();
        let two_hex = creature.flags.contains(CreatureFlag::TwoHex);
        let fp = field
            .footprint_at_front(front, two_hex, side)
            .filter(|fp| !fp.hexes().any(|h| taken.contains(h)))
            .ok_or(SetupError::NoRoom { side, slot })?;
        fp.hexes().for_each(|h| taken.insert(h));
        let mut stack = Stack::new(ruleset, setup, (id, creature), side, slot, fp.position(), 1);
        // rule: BAT-WAR-002
        stack.traits = stack.traits.with(
            StackTraits::UNLIMITED_SHOTS,
            kind == 0 && ruleset.quirks().is_enabled(UNLIMITED_BALLISTA_SHOTS),
        );
        out[kind] = Some(stack);
    }
    Ok(out)
}

/// The stacks `side` would have of each creature the battle may create for it, as at the
/// battle's start (the side's hero's bonuses, the native terrain; the original's 0x43D8B0 at the
/// cast, `docs/formats/installation.md` §7.19, §7.21): one per summon spell of its hero's book,
/// for Summon Elemental (BAT-MAG-039), then one of each creature the corpse raisers of either
/// army raise (BAT-MAG-052: a hypnotized raiser raises for the other side).
fn templates(
    ruleset: &BattleRuleset,
    setup: &BattleSetup,
    side: BattleSide,
    hero: Option<&HeroRecord>,
) -> FixedVec<Stack, TEMPLATES> {
    let mut out = FixedVec::new();
    let book = hero.map(|h| h.magic.book).unwrap_or_default();
    let summoned = book
        .iter()
        .filter_map(|id| ruleset.spell(id).and_then(|s| s.summons));
    let armies = [&setup.attacker, &setup.defender];
    let raised = armies
        .into_iter()
        .flat_map(|army| &army.stacks)
        .flat_map(|s| ruleset.abilities(s.creature))
        .filter_map(|a| match *a {
            Ability::CorpseRaiser { creature, .. } => Some(creature),
            _ => None,
        });
    for (id, creature) in summoned
        .chain(raised)
        .filter_map(|id| ruleset.creature(id).map(|c| (id, c)))
    {
        if out.iter().any(|t: &Stack| t.creature == id) {
            continue;
        }
        let stack = Stack::new(
            ruleset,
            setup,
            (id, creature),
            side,
            CREATED_SLOT,
            Hex(0),
            1,
        );
        // At most `MAX_SUMMON_SPELLS` summon spells and `MAX_RAISED_CREATURES` raised
        // creatures in a ruleset.
        let _ = out.push(stack);
    }
    out
}

/// `Stack::retaliations` value for unlimited retaliations (BAT-RET-003).
pub const UNLIMITED_RETALIATIONS: u8 = u8::MAX;

/// Retaliations per round of a creature: one, `1 + count` with extra retaliations, or
/// unlimited (BAT-RET-003).
// rule: BAT-RET-003
pub(crate) fn retaliations_per_round(abilities: &[h3_content_battle::CompiledAbility]) -> u8 {
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
    /// Army slot it came from (war machines: `MAX_ARMY_SLOTS` + kind; created stacks:
    /// [`CREATED_SLOT`]).
    pub slot: u8,
    /// Position: a one-hex stack's hex, a two-hex stack's rear hex (BAT-GEO-002).
    pub hex: Hex,
    /// Living creatures (0 = dead).
    pub count: u32,
    /// Creatures at the start of the battle, or when the battle created it (resurrection limit,
    /// BAT-DMG-008).
    pub base_count: u32,
    /// Hit points of the top creature (`1..=max_hp` while alive, 0 when dead).
    pub top_hp: u32,
    /// Attack skill before timed effects ([`Stack::attack_now`]).
    pub attack: u16,
    /// Defense skill before timed effects and the Defend bonus ([`Stack::defense_now`]).
    pub defense: u16,
    /// Minimum damage per creature.
    pub damage_min: u16,
    /// Maximum damage per creature.
    pub damage_max: u16,
    /// Hit points per creature.
    pub max_hp: u32,
    /// Speed before timed effects ([`Stack::speed_now`]).
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
    /// Most hit points its hero's regeneration effects restore to the top creature at the start
    /// of its turn (BAT-HER-006, BAT-TRN-005; 0: none).
    pub regeneration: u16,
    /// Per-round flags.
    pub flags: StackFlags,
    /// What its timed effects change, materialised from the battle's effect list (BAT-MAG-011).
    pub spells: SpellValues,
}

/// [`BattleState::corpse`] of a stack that left no corpse: it can never be raised (BAT-MAG-036).
pub const NO_CORPSE: u16 = u16::MAX;

/// What the raising spells keep per stack, by stack id (E04b T6a; kept beside the stacks so that
/// the stacks the hot paths walk stay small).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Raised {
    /// Creatures raised for this battle only, at most the stack's start count: left out of its
    /// end count (Basic Resurrection, BAT-MAG-035; the original's counter +0x54,
    /// `installation.md` §7.18).
    pub(crate) temporary: [u32; STACK_CAPACITY],
    /// Its corpse (BAT-MAG-035): 0 while it lives or until its death is marked after the command
    /// that killed it, then the order of its death among the battle's corpses (a later corpse
    /// lies above an earlier one on a shared hex), [`NO_CORPSE`] if it left none (sacrificed,
    /// BAT-MAG-036).
    pub(crate) corpse: [u16; STACK_CAPACITY],
}

impl Default for Raised {
    fn default() -> Self {
        Self {
            temporary: [0; STACK_CAPACITY],
            corpse: [0; STACK_CAPACITY],
        }
    }
}

impl CanonicalHash for Raised {
    fn hash_into(&self, h: &mut StateHasher) {
        for (&t, &c) in self.temporary.iter().zip(&self.corpse) {
            h.write_u32(t);
            h.write_u16(c);
        }
    }
}

impl Stack {
    /// A new stack of `count` creatures with the creature's stats, the native-terrain bonus
    /// (BAT-TER-001), the attack and defense of its side's hero (BAT-HER-002), for war machines
    /// the damage grown by the hero's attack (BAT-WAR-005), the hero's statistic effects
    /// (BAT-HER-005), and its traits and retaliations.
    fn new(
        ruleset: &BattleRuleset,
        setup: &BattleSetup,
        (id, creature): (CreatureTypeId, &CreatureType),
        side: BattleSide,
        slot: u8,
        hex: Hex,
        count: u32,
    ) -> Self {
        let abilities = ruleset.abilities(id);
        // rule: BAT-TER-001
        let native = u8::from(terrain::is_native(
            ruleset,
            id,
            terrain::native_terrain(ruleset, setup),
        ));
        // rule: BAT-HER-002
        let (hero_attack, hero_defense) = setup.heroes[side.index()]
            .as_ref()
            .map_or((0, 0), |hero| (hero.attack, hero.defense));
        // rule: BAT-WAR-005
        let machine_factor = if creature.flags.contains(CreatureFlag::WarMachine) {
            setup.heroes[side.index()]
                .as_ref()
                .map_or(1, |hero| u16::from(hero.attack) + 1)
        } else {
            1
        };
        // rule: BAT-HER-005
        let [attack, defense, speed, hit_points, min_damage, max_damage] =
            hero::stat_changes(ruleset, setup.heroes[side.index()].as_ref(), (id, creature));
        let damage_max = changed_u16(
            creature.damage_max.saturating_mul(machine_factor),
            max_damage,
        );
        let max_hp = changed(creature.hit_points.into(), hit_points, 1, u32::MAX.into());
        // Within `u32` after the clamp.
        let max_hp = u32::try_from(max_hp).unwrap_or(u32::MAX);
        let retaliations = retaliations_per_round(abilities);
        // rule: BAT-HER-006
        let (traits, regeneration) = hero::trait_effects(
            ruleset,
            setup.heroes[side.index()].as_ref(),
            id,
            StackTraits::of(creature, abilities),
        );
        let traits = traits.with(StackTraits::NATIVE, native != 0);
        let speed = u8::try_from(changed(
            creature.speed.saturating_add(native).into(),
            speed,
            0,
            u8::MAX.into(),
        ))
        .unwrap_or(u8::MAX);
        Self {
            creature: id,
            side,
            slot,
            hex,
            count,
            base_count: count,
            top_hp: max_hp,
            attack: changed_u16(
                creature
                    .attack
                    .saturating_add(native.into())
                    .saturating_add(hero_attack.into()),
                attack,
            ),
            defense: changed_u16(
                creature
                    .defense
                    .saturating_add(native.into())
                    .saturating_add(hero_defense.into()),
                defense,
            ),
            damage_min: changed_u16(
                creature.damage_min.saturating_mul(machine_factor),
                min_damage,
            )
            .min(damage_max),
            damage_max,
            max_hp,
            speed,
            shots: creature.shots,
            retaliations,
            retaliations_left: retaliations,
            morale: 0,
            luck: 0,
            traits,
            regeneration,
            flags: StackFlags::default(),
            spells: SpellValues::new(speed),
        }
    }
}

/// `value` changed by `delta`, kept within `min..=max` (BAT-HER-005).
fn changed(value: i64, delta: i64, min: i64, max: i64) -> i64 {
    value.saturating_add(delta).clamp(min, max)
}

/// A `u16` statistic changed by `delta`, kept within its type (BAT-HER-005).
fn changed_u16(value: u16, delta: i64) -> u16 {
    // Within `u16` after the clamp.
    u16::try_from(changed(value.into(), delta, 0, u16::MAX.into())).unwrap_or(u16::MAX)
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
            regeneration: 0,
            flags: StackFlags::default(),
            spells: SpellValues::default(),
        }
    }
}

impl Stack {
    /// Whether the stack still has creatures.
    pub const fn is_alive(&self) -> bool {
        self.count > 0
    }

    /// Whether the battle created the stack (a clone or a summoned stack, BAT-MAG-037).
    pub const fn is_created(&self) -> bool {
        self.traits.contains(StackTraits::SUMMONED)
    }

    /// Its place in the turn order's tie-break (BAT-TRN-001): its army slot, but
    /// [`CREATED_SLOT`] for a raised stack, which the original keeps in a new record
    /// (BAT-MAG-052).
    pub const fn turn_slot(&self) -> u8 {
        if self.traits.contains(StackTraits::RAISED) {
            CREATED_SLOT
        } else {
            self.slot
        }
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
                front: self.hex,
                rear: None,
            })
    }

    /// The side that controls the stack: its own, or the other while it is hypnotized
    /// (BAT-MAG-026). The controller commands its turns and its hero's bonuses apply to it.
    pub const fn controller(&self) -> BattleSide {
        if self.spells.status.contains(Status::HYPNOTIZED) {
            self.side.opponent()
        } else {
            self.side
        }
    }

    /// Whether this stack, on a turn its controller commands, may attack or shoot `other`
    /// (BAT-MAG-025, BAT-MAG-026): `other` is of the side its controller fights, whoever controls
    /// `other` and berserk or not. The original's cursor and pathfinder compare the acting side
    /// with the target's own side (0x476490; VCMI's `battleMatchOwner`; batch 2c row H1): the
    /// side that hypnotized a stack may attack it and its own side may not, and no side attacks
    /// its own berserk stack. (A hypnotized stack passes this test for itself.)
    pub const fn may_attack(&self, other: &Self) -> bool {
        self.controller().index() != other.side.index()
    }

    /// Whether `other`, a stack next to this one, counts as its enemy: either of them is
    /// berserk, or `other` is of the side this stack's controller fights (BAT-MAG-025,
    /// BAT-MAG-026; the original's 0x442880, which reads the first stack's Hypnotize and the
    /// second stack's own side). Its two callers ask it: whether a neighbour blocks a shooter
    /// (BAT-RNG-001) and whether a multi-headed strike reaches a neighbour (BAT-ATK-003). Who may
    /// be attacked or shot is [`Self::may_attack`].
    pub const fn is_hostile_to(&self, other: &Self) -> bool {
        (self.spells.status.bits() | other.spells.status.bits()) & Status::BERSERK.bits() != 0
            || self.may_attack(other)
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
        h.write_u16(self.regeneration);
        h.write_u8(self.flags.bits());
        self.spells.hash_into(h);
    }
}

pub(crate) const fn side_code(side: BattleSide) -> u8 {
    match side {
        BattleSide::Attacker => 0,
        BattleSide::Defender => 1,
    }
}

/// Hash code of a result: the winner's side code, 2 for a draw, 3 + the side's code for a
/// retreat, 5 + the side's code for a surrender.
const fn result_code(result: BattleResult) -> u8 {
    match result {
        BattleResult::Victory(side) => side_code(side),
        BattleResult::Draw => 2,
        BattleResult::Retreat(side) => 3 + side_code(side),
        BattleResult::Surrender(side) => 5 + side_code(side),
    }
}

/// The complete, authoritative state of one battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {
    pub(crate) stacks: FixedVec<Stack, STACK_CAPACITY>,
    /// The natural obstacles (BAT-GEO-003; pieces since E04b T5b).
    pub(crate) natural: NaturalObstacles,
    pub(crate) round: u16,
    /// The round's wait phase has begun: no stack is left to act but those that waited (the
    /// original's +0x13DE4; no Wraith drains then, BAT-MAG-056).
    pub(crate) wait_phase: bool,
    pub(crate) tactics: Option<TacticsPhase>,
    pub(crate) active: Option<StackId>,
    pub(crate) last_side_acted: Option<BattleSide>,
    pub(crate) result: Option<BattleResult>,
    pub(crate) surrender_cost: Option<u32>,
    pub(crate) modifiers: Modifiers,
    pub(crate) heroes: [Option<HeroRecord>; 2],
    pub(crate) terrain: Option<TerrainId>,
    /// The special battlefield, if any (BAT-TER-002; public).
    pub(crate) battlefield: Option<BattlefieldId>,
    /// The battle is at an anti-magic garrison (BAT-MAG-061; public).
    pub(crate) anti_magic: bool,
    /// The town the battle is fought at, if any (BAT-SIE-001; public).
    pub(crate) siege: Option<SiegeState>,
    /// The timed effects on stacks (ADR-0014 point 5).
    pub(crate) effects: Effects,
    /// The spell obstacles (ADR-0014 points 5 and 7).
    pub(crate) spell_obstacles: SpellObstacles,
    /// What the raising spells keep per stack (BAT-MAG-035, BAT-MAG-036).
    pub(crate) raised: Raised,
    /// What the battle keeps about the stacks it creates (BAT-MAG-037 to BAT-MAG-039).
    pub(crate) created: Created,
    /// What Poison and Age keep per stack (BAT-MAG-044).
    pub(crate) vitality: Vitality,
    /// What Bind keeps per stack (BAT-MAG-045).
    pub(crate) binds: Binds,
    /// The casts each stack has left (BAT-MAG-048).
    pub(crate) casts: Casts,
    /// What the heroes do to all of the battle's magic (BAT-MAG-061 to BAT-MAG-066), derived from
    /// them at the start (not hashed).
    pub(crate) limits: MagicLimits,
    pub(crate) rng: Rng,
}

impl BattleState {
    /// Starts a battle: validates the setup against the ruleset, places the obstacles and the
    /// armies (BAT-SET-001, BAT-GEO-003), seeds the battle RNG stream from `seed` and either
    /// starts the tactics phase (BAT-TAC-001) or activates the first stack. The events of that
    /// start (the phase, bad morale, war machines acting before any command) are dropped; use
    /// [`BattleState::start`] to receive them.
    pub fn new(
        ruleset: &BattleRuleset,
        setup: &BattleSetup,
        seed: u64,
    ) -> Result<Self, SetupError> {
        Self::start(ruleset, setup, seed, &mut NoopSink)
    }

    /// Like [`BattleState::new`], but reports the events of the start to `sink`: `TacticsStarted`
    /// if a tactics phase runs, the first `TurnStarted` and anything that happens before the
    /// first command (a bad-morale
    /// freeze, BAT-MOR-004; war machines acting, BAT-WAR-002/003; even the end of the battle).
    pub fn start<S: Sink>(
        ruleset: &BattleRuleset,
        setup: &BattleSetup,
        seed: u64,
        sink: &mut S,
    ) -> Result<Self, SetupError> {
        let params = ruleset.battle();
        let field = Field::new(params.field);
        let natural = NaturalObstacles::new(
            ruleset,
            field,
            &setup.obstacles,
            ruleset.quirks().is_enabled(OBSTACLE_ANCHORS),
        )?;
        // rule: BAT-TER-001
        if setup
            .terrain
            .is_some_and(|t| ruleset.terrains().get(t.index()).is_none())
        {
            return Err(SetupError::UnknownTerrain);
        }
        // rule: BAT-TER-002
        let battlefield = setup
            .battlefield
            .as_ref()
            .map(|k| {
                ruleset
                    .battlefield_by_key(k.as_str())
                    .ok_or(SetupError::UnknownBattlefield)
            })
            .transpose()?;
        let mut heroes = [None; 2];
        for (side, hero) in BattleSide::BOTH.into_iter().zip(&setup.heroes) {
            if let Some(hero) = hero {
                let mut record = hero::check(ruleset, side, hero, setup.funds[side.index()])?;
                // rule: BAT-MAG-054
                record.magic.cost_change = crate::mana::cost_change(ruleset, setup, side);
                heroes[side.index()] = Some(record);
            }
        }
        // rule: BAT-SIE-001
        let (siege, mines) = siege::setup(
            ruleset,
            field,
            (setup, &heroes, battlefield),
            natural.blocked(),
        )?
        .map_or((None, None), |(s, m)| (Some(s), m));
        // stub: BAT-STUB-2 — the town's other rules (the turrets, the Catapult, the controls, the
        // experience, the defenders' morale and luck) are not read yet (E04c T6 to T8).
        let mut taken = natural
            .blocked()
            .union(siege::placement_blocked(siege.as_ref()));
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
                // rule: BAT-WAR-001
                if creature.flags.contains(CreatureFlag::WarMachine) {
                    return Err(SetupError::WarMachineInArmy { side, slot });
                }
                *entry = Some((*s, *creature));
            }
            // War machines take their fixed places first (BAT-WAR-001).
            let machines = war_machines(ruleset, field, setup, side, &mut taken)?;
            let n = army.stacks.len();
            let column = match side {
                BattleSide::Attacker => field.first_standable_column(),
                BattleSide::Defender => field.last_standable_column(),
            };
            for (i, (s, creature)) in by_slot.iter().flatten().enumerate() {
                // The compiler guarantees a formation row for every army size and index.
                let row = params
                    .formation_row(setup.formations[side.index()], n, i)
                    .unwrap_or(0);
                // rule: BAT-SET-001
                let desired = field.hex(column, i32::from(row)).unwrap_or_default();
                let two_hex = creature.flags.contains(CreatureFlag::TwoHex);
                let position = closest_free(field, desired, two_hex, side, taken)
                    .ok_or(SetupError::NoRoom { side, slot: s.slot })?;
                let stack = Stack::new(
                    ruleset,
                    setup,
                    (s.creature, creature),
                    side,
                    s.slot,
                    position,
                    s.count,
                );
                stack.footprint(field).hexes().for_each(|h| taken.insert(h));
                // At most `army_slots + MAX_WAR_MACHINES` stacks per side, so `len` stays below
                // STACK_CAPACITY.
                placed[len] = stack;
                len += 1;
            }
            for stack in machines.into_iter().flatten() {
                placed[len] = stack;
                len += 1;
            }
        }
        let mut state = Self {
            stacks: FixedVec::from_array_prefix(placed, len),
            natural,
            round: 1,
            wait_phase: false,
            tactics: None,
            active: None,
            last_side_acted: None,
            result: None,
            surrender_cost: None,
            modifiers: Modifiers::new(),
            heroes,
            terrain: setup.terrain,
            battlefield,
            anti_magic: setup.anti_magic,
            siege,
            effects: Effects::new(),
            spell_obstacles: SpellObstacles::new(),
            raised: Raised::default(),
            created: Created::new(
                BattleSide::BOTH
                    .map(|side| templates(ruleset, setup, side, heroes[side.index()].as_ref())),
            ),
            vitality: Vitality::default(),
            binds: Binds::default(),
            casts: Casts::at_start(ruleset, &placed[..len]),
            limits: MagicLimits::of(
                &heroes,
                battlefield.and_then(|b| ruleset.battlefield(b)),
                setup.anti_magic,
            ),
            rng: Rng::for_stream(seed, StreamId::BATTLE),
        };
        morale::setup(&mut state, ruleset, &setup.heroes);
        // rule: BAT-MAG-031
        let natives = BattleSide::BOTH.map(|side| {
            state
                .stacks
                .iter()
                .any(|s| s.side == side && s.has(StackTraits::NATIVE))
        });
        state.spell_obstacles.set_sight(natives);
        // rule: BAT-SIE-007
        if let Some(mines) = mines {
            obstacles::lay_unseen(&mut state, mines, sink);
        }
        // rule: BAT-TAC-001
        if let Some(phase) = tactics::phase(ruleset, &state.heroes) {
            // Round 0: the tactics phase; `EndTactics` starts round 1 (BAT-TAC-002).
            tactics::begin(&mut state, phase, sink);
        } else {
            // The first activation runs without the freezing checks (BAT-MOR-004, BAT-MAG-060);
            // round 1 starts without a `RoundStarted` event.
            turn::activate_first(&mut state, ruleset, sink);
        }
        Ok(state)
    }

    /// The timed effects on stacks, by stack and spell (public, ADR-0014 point 8).
    pub fn effects(&self) -> &[ActiveEffect] {
        self.effects.as_slice()
    }

    /// The spell obstacles, in the order they were placed (BAT-MAG-027).
    pub fn spell_obstacles(&self) -> &[SpellObstacle] {
        self.spell_obstacles.as_slice()
    }

    /// What both sides see of `side`'s hero, if the side has one (BAT-HER-001).
    pub fn hero(&self, side: BattleSide) -> Option<HeroView> {
        self.heroes[side.index()].map(|h| h.view)
    }

    /// Whether `side` is led by a hero.
    pub(crate) const fn has_hero(&self, side: BattleSide) -> bool {
        self.heroes[side.index()].is_some()
    }

    /// Whether `side` may retreat (BAT-FLE-001; public, ADR-0013 point 9): it has a hero and no
    /// hero in the battle has a `NoRetreat` effect. It retreats on its own turn or in its
    /// tactics phase.
    pub fn may_retreat(&self, side: BattleSide) -> bool {
        hero::may_retreat(&self.heroes, side)
    }

    /// Whether `side` may surrender at all (BAT-FLE-002; public, ADR-0013 point 9): it may
    /// retreat, the enemy is led by a hero and the ruleset prices surrenders. Whether it can pay
    /// is private ([`may_surrender`](Self::may_surrender)).
    pub fn surrender_possible(&self, ruleset: &BattleRuleset, side: BattleSide) -> bool {
        flee::surrender_terms(self, ruleset, side).is_some()
    }

    /// The price of `side`'s surrender, if it may surrender at all (BAT-FLE-002): it may retreat,
    /// the enemy is led by a hero and the ruleset prices surrenders. Depends on the side's
    /// private discount: show it to that side only.
    pub fn surrender_price(&self, ruleset: &BattleRuleset, side: BattleSide) -> Option<u32> {
        flee::surrender_price(self, ruleset, side)
    }

    /// Whether `side` may surrender now (BAT-FLE-002): its surrender has a price and its funds
    /// cover it. Private to the side.
    pub fn may_surrender(&self, ruleset: &BattleRuleset, side: BattleSide) -> bool {
        self.surrender_price(ruleset, side)
            .is_some_and(|price| self.heroes[side.index()].is_some_and(|h| h.funds >= price))
    }

    /// The hexes blocked by natural obstacles (BAT-GEO-003).
    pub const fn obstacles(&self) -> HexSet {
        self.natural.blocked()
    }

    /// The natural obstacles a spell can remove, in setup order (BAT-MAG-034; public).
    pub fn removable_obstacles(&self) -> &[RemovableObstacle] {
        self.natural.removable()
    }

    /// The hexes that block `except`'s movement: obstacles, the hexes Force Fields block
    /// (BAT-MAG-027), a town's walls and closed gate (BAT-SIE-002, BAT-SIE-003) and the
    /// footprints of every other living stack (BAT-MOV-001).
    pub(crate) fn blocked_for(&self, field: Field, except: StackId) -> HexSet {
        // rule: BAT-SIE-002, BAT-SIE-003
        let mut blocked = self
            .natural
            .blocked()
            .union(self.spell_obstacles.walls())
            .union(siege::blocked_for(self, self.stack(except)));
        for (id, s) in (0u8..).zip(self.stacks.iter()) {
            if s.is_alive() && StackId(id) != except {
                s.footprint(field).hexes().for_each(|h| blocked.insert(h));
            }
        }
        blocked
    }

    /// The hexes of every living stack.
    pub(crate) fn living_hexes(&self, field: Field) -> HexSet {
        let mut taken = HexSet::new();
        for s in self.stacks.iter().filter(|s| s.is_alive()) {
            s.footprint(field).hexes().for_each(|h| taken.insert(h));
        }
        taken
    }

    /// All stacks (dead ones included; they keep their id, but a new created stack takes a killed
    /// created stack's, BAT-MAG-037).
    pub fn stacks(&self) -> &[Stack] {
        self.stacks.as_slice()
    }

    /// A stack by id.
    pub fn stack(&self, id: StackId) -> Option<&Stack> {
        self.stacks.get(id.index())
    }

    /// The creatures of stack `id` raised for this battle only (BAT-MAG-035; 0 for no stack).
    pub fn temporary(&self, id: StackId) -> u32 {
        self.raised.temporary.get(id.index()).copied().unwrap_or(0)
    }

    /// The corpse of stack `id` (BAT-MAG-035): 0 while it lives, else the order of its death
    /// among the battle's corpses, [`NO_CORPSE`] if it left none (0 for no stack).
    pub fn corpse(&self, id: StackId) -> u16 {
        self.raised.corpse.get(id.index()).copied().unwrap_or(0)
    }

    /// The rounds left of stack `id` if it is a clone (BAT-MAG-038): it vanishes at the start of
    /// the round its last one ends; 0 for any other stack.
    pub fn clone_rounds(&self, id: StackId) -> u16 {
        self.created.rounds.get(id.index()).copied().unwrap_or(0)
    }

    /// The casts stack `id` has left this battle (BAT-MAG-048): its creature's casts per battle
    /// less those it used; 0 for no stack.
    pub fn casts_left(&self, id: StackId) -> u8 {
        self.casts.left.get(id.index()).copied().unwrap_or(0)
    }

    /// The round starts since `side`'s periodic casters last cast (BAT-MAG-053), at most 255:
    /// 255 until they first cast (every battle starts ready, BAT-QUIRK-14).
    pub fn periodic_rounds(&self, side: BattleSide) -> u8 {
        self.casts.periodic[side.index()]
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

    /// The current round (1-based; 0 during the tactics phase, BAT-TAC-001).
    pub const fn round(&self) -> u16 {
        self.round
    }

    /// The running tactics phase, if any (BAT-TAC-001/002; public, ADR-0013 point 9).
    pub const fn tactics(&self) -> Option<TacticsPhase> {
        self.tactics
    }

    /// The stack whose turn it is, or `None` once the battle is over.
    pub const fn active(&self) -> Option<StackId> {
        self.active
    }

    /// The side whose turn it is, or `None` once the battle is over: the active stack's
    /// controller (BAT-MAG-026).
    pub fn side_to_act(&self) -> Option<BattleSide> {
        self.active
            .and_then(|id| self.stack(id))
            .map(Stack::controller)
    }

    /// The side of the last stack that took a turn (speed tie-breaks, BAT-TRN-001).
    pub const fn last_side_acted(&self) -> Option<BattleSide> {
        self.last_side_acted
    }

    /// How the battle ended, once it is over (BAT-END-001/002).
    pub const fn result(&self) -> Option<BattleResult> {
        self.result
    }

    /// The winner: `None` while the battle goes on and after a draw (BAT-END-002).
    pub fn winner(&self) -> Option<BattleSide> {
        self.result.and_then(BattleResult::winner)
    }

    /// Whether the battle is over.
    pub const fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// The outcome for the orchestrator, once the battle is over (BAT-END-003): the result,
    /// every stack's start and end counts by origin (the stacks the battle created too: BAT-Q7,
    /// BAT-Q13), the survivors derived from them, the surrender's cost, the heroes' mana and
    /// spells, the rounds.
    // rule: BAT-END-003
    pub fn outcome(&self) -> Option<BattleOutcome> {
        let result = self.result?;
        let stacks = BattleSide::BOTH.map(|side| {
            let mut records = Vec::new();
            let mut last_alive = None;
            let others = self.stacks.iter().zip(self.raised.temporary);
            for ((s, temporary), vanished) in others.zip(self.created.vanished) {
                if s.side != side {
                    continue;
                }
                let origin = if s.is_created() {
                    // rule: BAT-MAG-037
                    StackOrigin::Created
                } else if s.has(StackTraits::WAR_MACHINE) {
                    StackOrigin::WarMachine
                } else if s.has(StackTraits::RAISED) {
                    // rule: BAT-MAG-052
                    StackOrigin::Raised(s.slot)
                } else {
                    StackOrigin::Army(s.slot)
                };
                let army = matches!(origin, StackOrigin::Army(_) | StackOrigin::Raised(_));
                if army && s.is_alive() {
                    last_alive = Some(records.len());
                }
                records.push(StackRecord {
                    origin,
                    creature: s.creature,
                    start: s.base_count,
                    // rule: BAT-MAG-035, BAT-MAG-038
                    end: s.count.max(vanished).saturating_sub(temporary),
                });
            }
            // A side whose army has creatures alive only for the battle keeps one in its last
            // living army stack (the original's end of battle, 0x4774E2, counting only army
            // stacks: quirk BAT-QUIRK-6, S-20).
            // rule: BAT-MAG-035
            let none_left = records.iter().all(|r| {
                !matches!(r.origin, StackOrigin::Army(_) | StackOrigin::Raised(_)) || r.end == 0
            });
            if let Some(i) = last_alive.filter(|_| none_left) {
                records[i].end = 1;
            }
            records
        });
        let survivors = stacks.each_ref().map(|records| {
            let mut stacks: Vec<_> = records
                .iter()
                .filter_map(|r| match r.origin {
                    StackOrigin::Army(slot) | StackOrigin::Raised(slot) if r.end > 0 => {
                        Some(ArmyStack {
                            slot,
                            creature: r.creature,
                            count: r.end,
                        })
                    }
                    _ => None,
                })
                .collect();
            // A raised stack's slot is a dead army stack's (BAT-MAG-052): one stack per slot, so the
            // slot is a unique key.
            let sorted = h3_kernel::order::sort_by_unique_key(&mut stacks, |s| s.slot);
            debug_assert!(sorted.is_ok(), "two survivors in one army slot");
            Army { stacks }
        });
        let war_machines = stacks.each_ref().map(|records| {
            records
                .iter()
                .filter(|r| r.origin == StackOrigin::WarMachine && r.end > 0)
                .map(|r| r.creature)
                .collect()
        });
        Some(BattleOutcome {
            result,
            survivors,
            war_machines,
            stacks,
            surrender_cost: self.surrender_cost,
            heroes: self.heroes.map(|h| {
                h.map(|h| HeroOutcome {
                    mana: h.view.mana,
                    learned: h.magic.learned.iter().collect(),
                })
            }),
            rounds: self.round,
        })
    }

    /// Canonical hash of the full state, including hidden parts such as the RNG (desync detection
    /// and replay verification; never sent to untrusted clients, docs/06-security.md).
    ///
    /// Encoding v16 (v15 with the surrender's cost): version, round, the tactics phase if any
    /// (side, zone columns), active, last side, result (0/1 = victory of the attacker/defender,
    /// 2 = draw, 3/4 = retreat and 5/6 = surrender of the attacker/defender), the surrender's cost
    /// if any, stacks (each with base count,
    /// shots, retaliations, morale, luck and traits), obstacles, modifiers, per side the hero if
    /// any (type, attack, defense, power, knowledge, Tactics level, mana, maximum mana, ranged,
    /// melee and reduction basis points, the Ballista's control if any: shots and double-damage
    /// chance, the tent's control if any: heal bonus; whether it has a `NoRetreat` effect, the
    /// surrender discount, the funds; v18: the casts this round after the maximum mana and the
    /// magic after the funds: spell book and cast spells as bits, masteries, spell bonuses),
    /// terrain, RNG state. v19 (E04b T2d): the hero's resistance. v20 (E04b T3a): each stack's
    /// spell values after its flags, the hero's duration bonus after its resistance, the effect
    /// list before the RNG. v25 (E04b T4b): the stack flag `STARTED` and the spell values'
    /// status bits. v26 (E04b T5a): the spell obstacles after the effect list. v28 (E04b T5b):
    /// the obstacles as the blocked hexes, the removable obstacles (count; each anchor, count,
    /// hexes) and whether anchors belong to them. v36 (E04b T6c): per stack the creatures it
    /// vanished with, after the clones' rounds. v42 (E17 M1, CLT-Q3): each removable obstacle's
    /// type after its hexes, the absolute obstacle's type after whether anchors belong. v43: a
    /// two-hex stack's position is its rear hex (BAT-GEO-002), so the same field hashes
    /// differently. v45 (E04c T2b): the special battlefield (presence flag + handle) and the
    /// anti-magic flag after the terrain.
    pub fn state_hash(&self) -> u64 {
        h3_kernel::hash::hash_of(self)
    }
}

/// The position where a stack of this shape fits (every hex of its footprint standable and not
/// `taken`) with its front hex ([`Footprint::front`]) closest to `desired`. Ties go to the column
/// nearer the centre of the field, then the row nearer `desired`'s, then the lower hex index
/// (BAT-SET-001).
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
        .min_by_key(|fp| {
            let front = fp.front;
            let (x, y) = field.coords(front);
            let toward_centre = match side {
                BattleSide::Attacker => -x,
                BattleSide::Defender => x,
            };
            (
                field.distance(desired, front),
                toward_centre,
                (y - dy).abs(),
                front,
            )
        })
        .map(|fp| fp.position())
}

impl CanonicalHash for BattleState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(BATTLE_STATE_HASH_VERSION);
        h.write_u16(self.round);
        h.write_bool(self.wait_phase);
        self.tactics.hash_into(h);
        self.active.map(|s| s.0).hash_into(h);
        self.last_side_acted.map(side_code).hash_into(h);
        self.result.map(result_code).hash_into(h);
        self.surrender_cost.hash_into(h);
        self.stacks.hash_into(h);
        self.natural.hash_into(h);
        self.modifiers.hash_into(h);
        self.heroes.hash_into(h);
        self.terrain.hash_into(h);
        self.battlefield.hash_into(h);
        h.write_bool(self.anti_magic);
        self.siege.hash_into(h);
        self.effects.hash_into(h);
        self.spell_obstacles.hash_into(h);
        self.raised.hash_into(h);
        self.created.hash_into(h);
        self.vitality.hash_into(h);
        self.binds.hash_into(h);
        self.casts.hash_into(h);
        self.rng.hash_into(h);
    }
}
