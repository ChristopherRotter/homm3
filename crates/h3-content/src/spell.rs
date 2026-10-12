//! What a spell does in battle (E04b, ADR-0014 point 1): the target its caster names, the stacks
//! the cast reaches (an area around a hex, a chain from a stack, or every stack), the conditions
//! a stack must meet, and its effects from a closed set of kinds (pack format v4, E04b T2a; timed
//! changes and mass versions since format v5, E04b T3a).
//!
//! Spells are data over these kinds: each kind is one battle rule (`BAT-MAG-…`), so a new spell of
//! a known kind is a data change. Kinds join with the task that implements them: damage in T2;
//! timed changes of a stack's values in T3a; dispel and heal in T3b; and so on. A spell without
//! effects cannot be cast.

use h3_kernel::{CanonicalHash, StateHasher};

use crate::effect::FULL_BP;
use crate::flags::CreatureFlags;

/// Most effects of one spell.
pub const MAX_SPELL_EFFECTS: usize = 4;
/// Largest distance from the target hex an area may reach.
pub const MAX_AREA_RADIUS: u8 = 16;
/// Most stacks a chain may strike.
pub const MAX_CHAIN_TARGETS: u8 = 16;

/// What the caster names when casting (below Expert for spells with a mass version; T3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SpellTarget {
    /// Nothing: the spell reaches every stack that meets its conditions (Armageddon).
    #[default]
    None,
    /// A stack (Magic Arrow; Chain Lightning's first stack).
    Stack,
    /// A hex: the spell reaches the stacks on its area (Fireball).
    Hex,
}

impl SpellTarget {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::None, Self::Stack, Self::Hex];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Stack => "stack",
            Self::Hex => "hex",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.name() == name)
    }
}

/// Whose stacks a spell may reach, seen from the caster (VCMI's "smart" targeting: Magic Arrow
/// only enemies).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TargetSide {
    /// Stacks of either side.
    #[default]
    Any,
    /// The enemy's stacks.
    Enemy,
    /// The caster's own stacks.
    Own,
}

impl TargetSide {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::Any, Self::Enemy, Self::Own];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Enemy => "enemy",
            Self::Own => "own",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// The hexes a spell aimed at a hex reaches: those at a distance of `inner` to `radius[mastery]`
/// from the target hex (Fireball 0 to 1, Frost Ring 1 to 1, Inferno 0 to 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellArea {
    /// Smallest distance reached (1 leaves the target hex out).
    pub inner: u8,
    /// Largest distance reached, by mastery: none, basic, advanced, expert.
    pub radius: [u8; 4],
}

/// How a spell aimed at a stack jumps on to further stacks (Chain Lightning).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellChain {
    /// Stacks struck in all, the first included, by mastery.
    pub targets: [u8; 4],
    /// Each further stack takes this share of the previous stack's value, basis points.
    pub factor_bp: u16,
}

/// How a timed effect changes a stack while it lasts (E04b T3a; the original's code,
/// `docs/formats/installation.md` §7.9). `v` is the spell's `effect[mastery]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimedChange {
    /// Speed + v, not on war machines (Haste, Prayer).
    Speed,
    /// Speed × v %, rounded down, at least 1 (Slow).
    SpeedPercent,
    /// Attack + v (Prayer).
    Attack,
    /// Attack − v, at most down to 0 (Weakness).
    AttackLoss,
    /// Defense + v (Stone Skin, Prayer).
    Defense,
    /// Defense − v, at most down to 0, given back at the end (Disrupting Ray never ends;
    /// Disease).
    DefenseLoss,
    /// Attack + v on melee attacks (Bloodlust).
    MeleeAttack,
    /// Attack + v on ranged attacks (Precision).
    RangedAttack,
    /// Every creature deals its maximum damage + v (Bless).
    MaximumDamage,
    /// Every creature deals its minimum damage − v, at least 1 (Curse).
    MinimumDamage,
    /// Morale + v (Mirth).
    Morale,
    /// Morale − v (Sorrow).
    MoraleLoss,
    /// Luck + v (Fortune).
    Luck,
    /// Luck − v (Misfortune).
    LuckLoss,
    /// Damage of melee strikes taken × v %, single precision (Shield; BAT-MAG-012).
    MeleeProtection,
    /// Damage of shots taken × v %, single precision (Air Shield; BAT-MAG-012).
    RangedProtection,
    /// Damage of spells of this spell's schools taken × v % (the Protections; BAT-MAG-013).
    SchoolProtection,
    /// Attack + v against creatures of a king level the first cast's mastery reaches: king 1
    /// always, king 2 from Advanced, king 3 at Expert (Slayer; BAT-MAG-014).
    Slayer,
    /// Retaliations per round + v; the first cast adds v to those left at once, the end gives
    /// nothing back (Counterstrike; BAT-MAG-015).
    Retaliations,
    /// Melee strikes on the stack reflect v % of their first-stage damage onto the striker as
    /// this spell's damage (Fire Shield; BAT-MAG-018, v8).
    FireShield,
    /// Attack + its defense × v %, defense 0 when struck (Frenzy; BAT-MAG-019, v8).
    Frenzy,
    /// Immune to every spell below level v that is not a creature ability, whatever its
    /// polarity; the first cast ends the stack's negative effects below level v (Anti-Magic;
    /// BAT-MAG-021, v10).
    SpellImmunity,
    /// A single-target spell the other side's hero aims at the stack goes, with a chance of v %,
    /// to a random living stack of the caster's side instead; a creature's spell never does
    /// (Magic Mirror; BAT-MAG-022, v10).
    MagicMirror,
    /// A shooter counts half its creatures, at least one, for the damage of every strike (melee
    /// and retaliations too: survey S-09, the rule, Q53); from the first cast's Advanced mastery
    /// it cannot shoot (Forgetfulness; BAT-MAG-023, v11).
    Forgetfulness,
    /// The stack loses its turns; struck in melee while blind, it retaliates with v % of the
    /// damage (damage stage 2), not at all at 0 (Blind; BAT-MAG-024, v11). Ends with any damage
    /// when the spell `ends = "damaged"`.
    Blind,
    /// The stack's turns are automatic: it attacks the nearest other stack; next to any stack
    /// it counts as that stack's enemy and the stack as its (shots are blocked, multi-headed
    /// strikes reach it), but its own side may not attack or shoot it (Berserk; BAT-MAG-025,
    /// v12).
    Berserk,
    /// The other side controls the stack: it commands its turns, and the stack fights for it
    /// (Hypnotize; BAT-MAG-026, v12).
    Hypnotize,
    /// The stack loses its turns, takes half the damage of strikes (damage stage 3) and does
    /// not retaliate (Stone Gaze; BAT-MAG-043, v20).
    Petrify,
    /// The stack loses its turns; struck in melee it retaliates with v % of the damage (damage
    /// stage 2; with Blind the smaller share) (Paralyze; BAT-MAG-043, v20).
    Paralyze,
    /// The stack's hit points per creature fall by a tenth of their start value at the cast and
    /// at every round start while it lasts, to half at most, and stay down after it ends (Poison;
    /// BAT-MAG-044, v20).
    Poison,
    /// The stack's hit points per creature are halved while it lasts (Age; BAT-MAG-044, v20).
    Age,
    /// The stack cannot move (it still attacks next to it) while the stacks that struck it with
    /// the spell stay (Bind; BAT-MAG-045, v21).
    Bind,
}

impl TimedChange {
    /// All values, in code order.
    pub const ALL: [Self; 32] = [
        Self::Speed,
        Self::SpeedPercent,
        Self::Attack,
        Self::AttackLoss,
        Self::Defense,
        Self::DefenseLoss,
        Self::MeleeAttack,
        Self::RangedAttack,
        Self::MaximumDamage,
        Self::MinimumDamage,
        Self::Morale,
        Self::MoraleLoss,
        Self::Luck,
        Self::LuckLoss,
        Self::MeleeProtection,
        Self::RangedProtection,
        Self::SchoolProtection,
        Self::Slayer,
        Self::Retaliations,
        Self::FireShield,
        Self::Frenzy,
        Self::SpellImmunity,
        Self::MagicMirror,
        Self::Forgetfulness,
        Self::Blind,
        Self::Berserk,
        Self::Hypnotize,
        Self::Petrify,
        Self::Paralyze,
        Self::Poison,
        Self::Age,
        Self::Bind,
    ];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Speed => "speed",
            Self::SpeedPercent => "speed_percent",
            Self::Attack => "attack",
            Self::AttackLoss => "attack_loss",
            Self::Defense => "defense",
            Self::DefenseLoss => "defense_loss",
            Self::MeleeAttack => "melee_attack",
            Self::RangedAttack => "ranged_attack",
            Self::MaximumDamage => "maximum_damage",
            Self::MinimumDamage => "minimum_damage",
            Self::Morale => "morale",
            Self::MoraleLoss => "morale_loss",
            Self::Luck => "luck",
            Self::LuckLoss => "luck_loss",
            Self::MeleeProtection => "melee_protection",
            Self::RangedProtection => "ranged_protection",
            Self::SchoolProtection => "school_protection",
            Self::Slayer => "slayer",
            Self::Retaliations => "retaliations",
            Self::FireShield => "fire_shield",
            Self::Frenzy => "frenzy",
            Self::SpellImmunity => "spell_immunity",
            Self::MagicMirror => "magic_mirror",
            Self::Forgetfulness => "forgetfulness",
            Self::Blind => "blind",
            Self::Berserk => "berserk",
            Self::Hypnotize => "hypnotize",
            Self::Petrify => "petrify",
            Self::Paralyze => "paralyze",
            Self::Poison => "poison",
            Self::Age => "age",
            Self::Bind => "bind",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }

    /// Whether the change makes the stack lose its turns while it lasts (Blind, Stone Gaze,
    /// Paralyze; BAT-MAG-024, BAT-MAG-043). Such an effect never lasts the whole battle: the
    /// compiler refuses it with a permanent duration.
    pub const fn takes_turns(self) -> bool {
        matches!(self, Self::Blind | Self::Petrify | Self::Paralyze)
    }
}

/// One effect of a spell on each stack it reaches. The value of a cast is
/// `power × power factor + effect[mastery]` (the spell's data row), changed per target and caster
/// by the kind's rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellEffect {
    /// Deals the value as damage (BAT-MAG rules, E04b T2).
    Damage,
    /// Changes the stack's values while the effect lasts (BAT-MAG-009 to 011, E04b T3a).
    Timed(TimedChange),
    /// Ends the stack's timed effects: those of the spell's `dispels`, or, without that list,
    /// every one but those of its `spares`; never a permanent one (BAT-MAG-016, E04b T3b1b).
    Dispel,
    /// Heals the stack's top creature by the value, raising none (BAT-MAG-017, E04b T3b1b).
    Heal,
    /// Places a battlefield obstacle of this kind on the spell's [`ObstacleShape`] at the target
    /// hex; it reaches no stack (BAT-MAG-027, E04b T5a, v13).
    Obstacle(ObstacleKind),
    /// Moves the target stack to the cast's second target, a hex (BAT-MAG-029, E04b T5a, v13).
    Teleport,
    /// Removes the obstacle on the target hex: a removable natural one at every mastery, a spell
    /// obstacle of a kind from its mastery in [`SpellMagic::removes_from`] (BAT-MAG-034, E04b T5b,
    /// v16).
    RemoveObstacle,
    /// Raises the creatures of the target stack, a living or dead one of the caster's side, by
    /// the value in hit points up to its count at the battle's start; below
    /// [`SpellMagic::temporary_below`] for this battle only (Resurrection, Animate Dead;
    /// BAT-MAG-035, E04b T6a, v17).
    Raise,
    /// Kills the cast's second target, a living stack of the caster's side, and raises the
    /// target stack permanently by `(the victim's creature's hit points + the value) × its
    /// creatures` (BAT-MAG-036, E04b T6a, v17).
    Sacrifice,
    /// Creates a copy of the target stack, a living one of the caster's side, for some rounds
    /// (Clone; BAT-MAG-038, E04b T6b, v18).
    Clone,
    /// Creates a stack of the spell's [`crate::Spell::summons`] creature on the caster's side
    /// (Summon Elemental; BAT-MAG-039, E04b T6b, v18).
    Summon,
}

impl SpellEffect {
    /// Pack name of the kind.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Damage => "damage",
            Self::Timed(_) => "timed",
            Self::Dispel => "dispel",
            Self::Heal => "heal",
            Self::Obstacle(_) => "obstacle",
            Self::Teleport => "teleport",
            Self::RemoveObstacle => "remove_obstacle",
            Self::Raise => "raise",
            Self::Sacrifice => "sacrifice",
            Self::Clone => "clone",
            Self::Summon => "summon",
        }
    }

    /// The timed change, if this is one.
    pub const fn timed(self) -> Option<TimedChange> {
        match self {
            Self::Timed(c) => Some(c),
            Self::Damage
            | Self::Dispel
            | Self::Heal
            | Self::Obstacle(_)
            | Self::Teleport
            | Self::RemoveObstacle
            | Self::Raise
            | Self::Sacrifice
            | Self::Clone
            | Self::Summon => None,
        }
    }

    /// Whether the kind acts on the battlefield rather than on the stacks a cast reaches
    /// (obstacles, Teleport, Remove Obstacle): such an effect is the spell's only one.
    pub const fn is_field(self) -> bool {
        matches!(
            self,
            Self::Obstacle(_) | Self::Teleport | Self::RemoveObstacle
        )
    }

    /// Whether the kind raises creatures (Raise, Sacrifice): it reaches dead stacks too, and is
    /// the spell's only effect.
    pub const fn is_raise(self) -> bool {
        matches!(self, Self::Raise | Self::Sacrifice)
    }

    /// Whether the kind creates a stack (Clone, Summon): it is the spell's only effect.
    pub const fn creates(self) -> bool {
        matches!(self, Self::Clone | Self::Summon)
    }

    /// Canonical code (hash): the kind, then the change or the obstacle kind.
    fn code(self) -> [u8; 2] {
        match self {
            Self::Damage => [0, 0],
            Self::Timed(c) => [1, c as u8],
            Self::Dispel => [2, 0],
            Self::Heal => [3, 0],
            Self::Obstacle(k) => [4, k as u8],
            Self::Teleport => [5, 0],
            Self::RemoveObstacle => [6, 0],
            Self::Raise => [7, 0],
            Self::Sacrifice => [8, 0],
            Self::Clone => [9, 0],
            Self::Summon => [10, 0],
        }
    }
}

/// What a spell's battlefield obstacle does (E04b T5; ADR-0014 point 7). Each kind is one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObstacleKind {
    /// Blocks its hexes like a natural obstacle: no stack enters or stands on them (Force Field;
    /// BAT-MAG-027).
    Wall,
    /// Burns a stack that enters one of its hexes, or acts while standing on one, with the
    /// spell's damage; the hexes stay passable (Fire Wall; BAT-MAG-028).
    Fire,
    /// Stops a walker that enters one of its hexes; hidden from the other side until revealed
    /// (Quicksand; BAT-MAG-031, BAT-MAG-032).
    Pit,
    /// Goes off under a stack of the other side that enters its hex unseen, with the spell's
    /// damage, and is gone; hidden from the other side, and not cast while that side's natives
    /// would see it (Land Mine; BAT-MAG-031, BAT-MAG-033; v15).
    Mine,
}

impl ObstacleKind {
    /// All values, in code order.
    pub const ALL: [Self; 4] = [Self::Wall, Self::Fire, Self::Pit, Self::Mine];

    /// How many kinds there are (the size of tables indexed by kind).
    pub const COUNT: usize = Self::ALL.len();

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wall => "wall",
            Self::Fire => "fire",
            Self::Pit => "pit",
            Self::Mine => "mine",
        }
    }

    /// Whether the other side does not see it until it is revealed (ADR-0014 point 7,
    /// BAT-MAG-031): a pit or a mine; walls and fire are seen by both sides.
    pub const fn is_hidden(self) -> bool {
        matches!(self, Self::Pit | Self::Mine)
    }

    /// Whether it carries the cast's value (`power × power factor + effect[mastery]`): fire and
    /// mines, which do the spell's damage (BAT-MAG-028, BAT-MAG-033).
    pub const fn has_value(self) -> bool {
        matches!(self, Self::Fire | Self::Mine)
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A step from a hex to one of its neighbours (the battlefield's geometry, BAT-GEO-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HexStep {
    /// Same row, next column.
    Right,
    /// Row below, towards the right.
    BottomRight,
    /// Row below, towards the left.
    BottomLeft,
    /// Same row, previous column.
    Left,
    /// Row above, towards the left.
    TopLeft,
    /// Row above, towards the right.
    TopRight,
}

impl HexStep {
    /// All values, in code order.
    pub const ALL: [Self; 6] = [
        Self::Right,
        Self::BottomRight,
        Self::BottomLeft,
        Self::Left,
        Self::TopLeft,
        Self::TopRight,
    ];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Right => "right",
            Self::BottomRight => "bottom_right",
            Self::BottomLeft => "bottom_left",
            Self::Left => "left",
            Self::TopLeft => "top_left",
            Self::TopRight => "top_right",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }

    /// The step mirrored left to right (the defender's view of an attacker's step).
    #[must_use]
    pub const fn mirrored(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::BottomRight => Self::BottomLeft,
            Self::BottomLeft => Self::BottomRight,
            Self::Left => Self::Right,
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::TopLeft,
        }
    }
}

/// Most hexes one obstacle spell covers.
pub const MAX_OBSTACLE_HEXES: usize = 8;

/// Where an obstacle spell's obstacles go and how long they last (E04b T5a, v13; random shapes
/// v14, T5c1; the original's object descriptors, `docs/formats/installation.md` §7.15, §7.16).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObstacleShape {
    /// The hexes after the target hex: each the previous one's neighbour in this direction, as
    /// the attacker's hero casts it.
    pub steps: alloc::vec::Vec<HexStep>,
    /// How many hexes a cast covers by mastery (none, basic, advanced, expert): the target hex
    /// and the first steps' hexes; 1 to `steps + 1` (a random shape: 1 to 8).
    pub hexes: [u8; 4],
    /// The cast has no target: each hex is a random free hex of the battlefield (Quicksand;
    /// BAT-MAG-032; v14). A random shape has no steps, is not mirrored and puts one obstacle on each hex.
    pub random: bool,
    /// The defender's hero casts the steps mirrored left to right (Fire Wall).
    pub mirrored: bool,
    /// Each hex is an obstacle of its own (Fire Wall); otherwise one obstacle covers them all.
    pub separate: bool,
    /// Rounds an obstacle lasts, the cast's included: it loses one at every round start and goes
    /// when none is left (BAT-MAG-027); 0: the whole battle (Quicksand; v14).
    pub rounds: u8,
}

impl ObstacleShape {
    /// Why the shape is invalid, if it is.
    fn error(&self) -> Option<&'static str> {
        let most = if self.random {
            MAX_OBSTACLE_HEXES
        } else {
            self.steps.len() + 1
        };
        if self.steps.len() + 1 > MAX_OBSTACLE_HEXES {
            Some("an obstacle covers at most 8 hexes")
        } else if self.random && (!self.steps.is_empty() || self.mirrored || !self.separate) {
            Some("a random obstacle shape has no steps, is not mirrored and is separate")
        } else if self.hexes.iter().any(|&n| n == 0 || usize::from(n) > most) {
            Some("an obstacle's hexes must be 1..=steps + 1 (random: 1..=8)")
        } else {
            None
        }
    }
}

impl CanonicalHash for ObstacleShape {
    /// Steps (count, codes), hexes by mastery, random (v24), mirrored, separate, rounds.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.steps.len());
        for &s in &self.steps {
            h.write_u8(s as u8);
        }
        h.write_bytes(&self.hexes);
        h.write_bool(self.random);
        h.write_bool(self.mirrored);
        h.write_bool(self.separate);
        h.write_u8(self.rounds);
    }
}

/// How long the timed effects of a spell last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EffectDuration {
    /// The caster's power plus its duration bonuses, in rounds (BAT-MAG-010).
    #[default]
    Rounds,
    /// The rest of the battle (Disrupting Ray).
    Permanent,
    /// Through the stack's next turn: 1 round on the active stack, else 2, counted down at the
    /// stack's own turn start instead of the round start (Frenzy; BAT-MAG-019, v8).
    OwnTurn,
    /// This many rounds whatever the caster's power (Berserk: 255, the original's; v12).
    Fixed(u16),
}

impl EffectDuration {
    /// Canonical code (hash): the kind, then the rounds of a fixed duration.
    const fn code(self) -> (u8, u16) {
        match self {
            Self::Rounds => (0, 0),
            Self::Permanent => (1, 0),
            Self::OwnTurn => (2, 0),
            Self::Fixed(rounds) => (3, rounds),
        }
    }
}

/// What ends a timed effect before its rounds run out, besides Dispel and Cure (v11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EffectEnd {
    /// Nothing.
    #[default]
    Never,
    /// Any damage the stack takes: strikes, shots, spells, a reflection (Blind; BAT-MAG-024,
    /// `installation.md` §7.14).
    Damaged,
    /// The stack's own attack: its melee attack or its shot, not a retaliation (Berserk;
    /// BAT-MAG-025, v12).
    OwnAttack,
}

impl EffectEnd {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::Never, Self::Damaged, Self::OwnAttack];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Damaged => "damaged",
            Self::OwnAttack => "own_attack",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }
}

/// When a spell is useful on a stack for a creature that draws among the useful spells of its
/// `drawn_spell` entries (the Master Genie; the original's 0x447A80, `installation.md` §7.21;
/// v24). Besides its own test, a useful spell is not active on the stack and can affect it. The
/// enemy is the side opposite the caster's controller; its stacks are those it owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Usefulness {
    /// Always.
    #[default]
    Always,
    /// An enemy stack that cannot shoot now, among those alive, not blind, petrified or
    /// paralysed and no war machine that heals or supplies (Shield, Fire Shield).
    EnemyMelee,
    /// An enemy stack among the same that can shoot now (Air Shield).
    EnemyShooter,
    /// The enemy hero carries a spell book (the Protections, Anti-Magic, Magic Mirror).
    EnemySpellBook,
    /// The stack's top creature has lost hit points (Cure).
    Wounded,
    /// The stack cannot shoot now (Bloodlust).
    Melee,
    /// The stack can shoot now (Precision).
    Shooting,
    /// The stack has not acted this round (Prayer).
    NotActed,
    /// An enemy stack, dead or alive, whose creature has flag `king_1` (Slayer: the original's
    /// test, the rule in every profile).
    EnemyKing,
}

impl Usefulness {
    /// All values, in code order.
    pub const ALL: [Self; 9] = [
        Self::Always,
        Self::EnemyMelee,
        Self::EnemyShooter,
        Self::EnemySpellBook,
        Self::Wounded,
        Self::Melee,
        Self::Shooting,
        Self::NotActed,
        Self::EnemyKing,
    ];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::EnemyMelee => "enemy_melee",
            Self::EnemyShooter => "enemy_shooter",
            Self::EnemySpellBook => "enemy_spell_book",
            Self::Wounded => "wounded",
            Self::Melee => "melee",
            Self::Shooting => "shooting",
            Self::NotActed => "not_acted",
            Self::EnemyKing => "enemy_king",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }
}

/// Creature levels a [`SpecialistBonus::CreatureLevel`] table covers (1 to 7).
pub const SPECIALIST_LEVELS: usize = 7;

/// What a hero specialised in a spell adds to the spell's value on a stack, when the spell has a
/// rule of its own (`docs/formats/installation.md` §7.12; v9). Without one, the hero's
/// `spell_specialty` rate scales the value (damage, Cure, Bless).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialistBonus {
    /// An amount by the stack's creature level, 1 to 7 (higher levels: the last).
    CreatureLevel([u8; SPECIALIST_LEVELS]),
    /// A fixed amount.
    Fixed(u16),
    /// The value becomes this total, whatever the mastery.
    Total(u16),
}

impl SpecialistBonus {
    /// The bonus to the value `t` on a stack of creature level `level` (at least 1).
    pub fn bonus(self, level: u8, t: u64) -> i64 {
        match self {
            Self::CreatureLevel(values) => {
                let i = usize::from(level.max(1) - 1).min(SPECIALIST_LEVELS - 1);
                i64::from(values[i])
            }
            Self::Fixed(amount) => i64::from(amount),
            Self::Total(total) => i64::from(total) - i64::try_from(t).unwrap_or(i64::MAX),
        }
    }
}

impl CanonicalHash for SpecialistBonus {
    /// Kind (0 creature level, 1 fixed, 2 total) and its values.
    fn hash_into(&self, h: &mut StateHasher) {
        match *self {
            Self::CreatureLevel(values) => {
                h.write_u8(0);
                h.write_bytes(&values);
            }
            Self::Fixed(amount) => {
                h.write_u8(1);
                h.write_u16(amount);
            }
            Self::Total(total) => {
                h.write_u8(2);
                h.write_u16(total);
            }
        }
    }
}

/// Highest mastery (Expert).
pub const MAX_MASTERY: u8 = 3;

/// Pack names of the masteries that can start a mass version, from Basic (1) to Expert (3).
pub const MASTERY_NAMES: [&str; 3] = ["basic", "advanced", "expert"];

/// Which stacks a spell may affect, by creature flags (Destroy Undead: only `undead`; Death
/// Ripple: no `undead`, no `war_machine`) and by the creature's damage (Bless: above 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TargetConditions {
    /// A stack needs every one of these flags.
    pub requires: CreatureFlags,
    /// A stack may have none of these flags.
    pub excludes: CreatureFlags,
    /// A stack's creature needs a maximum damage above 0 (pack format v32: Bless, Curse,
    /// Fortune, Misfortune and Slayer spare the First Aid Tent and the Ammo Cart; the original
    /// reads the creature table's value, `installation.md` §7.23).
    pub needs_damage: bool,
}

impl TargetConditions {
    /// Whether a creature with `flags` and maximum damage `damage_max` meets the conditions.
    pub const fn admits(self, flags: CreatureFlags, damage_max: u16) -> bool {
        flags.bits() & self.requires.bits() == self.requires.bits()
            && flags.bits() & self.excludes.bits() == 0
            && (!self.needs_damage || damage_max > 0)
    }
}

/// How a spell reaches stacks in battle and what it does to them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellMagic {
    /// What the caster names.
    pub target: SpellTarget,
    /// The area around a hex target, if the target is a hex.
    pub area: Option<SpellArea>,
    /// The chain from a stack target, if any.
    pub chain: Option<SpellChain>,
    /// Whose stacks it may reach (below [`Self::any_side_from`]).
    pub side: TargetSide,
    /// The lowest mastery (1..=3) from which it may reach stacks of either side (Dispel: from
    /// Advanced; v7).
    pub any_side_from: Option<u8>,
    /// A stack it is aimed at needs an active timed effect (Dispel below Expert; v7).
    pub needs_effect: bool,
    /// No creature is immune to it and none resists it: no immunity, magic resistance or
    /// resistance aura applies (Dispel, `installation.md` §7.10; v7); nor do Anti-Magic and
    /// Magic Mirror (§7.13; v10).
    pub ignores_immunity: bool,
    /// The stacks it may affect.
    pub conditions: TargetConditions,
    /// What it does to each stack reached, distinct, in pack order (none: not castable yet).
    pub effects: alloc::vec::Vec<SpellEffect>,
    /// A mind spell (the executable's flag `0x400`): creatures with flag `mind_immune` are immune
    /// (v5).
    pub mind: bool,
    /// The lowest mastery (1..=3) at which the spell has a mass version: it then takes no target
    /// and reaches every stack it may reach (the flag `0x40`: Expert; v5).
    pub mass_from: Option<u8>,
    /// The mass version's mastery when quirk Q-0011 is off, later than `mass_from` (the
    /// original's Forgetfulness is mass from Advanced, the fixed one at Expert; v11).
    pub fixed_mass_from: Option<u8>,
    /// How long its timed effects last (v5).
    pub duration: EffectDuration,
    /// What else ends its timed effects (v11).
    pub ends: EffectEnd,
    /// A cast on a stack it already affects applies its changes again (Disrupting Ray; v5).
    pub cumulative: bool,
    /// What a specialist in the spell adds to its value by a rule of its own (v9).
    pub specialist: Option<SpecialistBonus>,
    /// A stack it may affect has full hit points (creatures × hit points per creature) at most
    /// the cast's value, a specialist's bonus included (Hypnotize; v12).
    pub health_limit: bool,
    /// A stack with an active effect that makes one of these changes is no target (Hypnotize:
    /// Magic Mirror; v12).
    pub refuses: alloc::vec::Vec<TimedChange>,
    /// Where an obstacle effect's obstacles go (v13).
    pub obstacle: Option<ObstacleShape>,
    /// The lowest mastery (1..=3) from which a cast also removes every spell obstacle of the
    /// battlefield (Dispel at Expert; BAT-MAG-030, v13).
    pub clears_obstacles_from: Option<u8>,
    /// A remove effect: the lowest mastery (1..=3) from which it removes a spell obstacle of each
    /// kind, indexed by [`ObstacleKind`] (`None`: never; Remove Obstacle: fire from Advanced, the
    /// others at Expert; BAT-MAG-034, v16).
    pub removes_from: [Option<u8>; ObstacleKind::COUNT],
    /// A raise effect: the mastery (1..=3) from which the creatures it raises stay after the
    /// battle; below it they are raised for the battle only (Resurrection: Advanced;
    /// BAT-MAG-035, v17).
    pub temporary_below: Option<u8>,
    /// Its dispel effect ends only the effects of positive spells, and a strike casts it only on
    /// a stack with one (Dispel Helpful Spells, `installation.md` §7.20; BAT-MAG-042, v19).
    pub dispels_positive: bool,
    /// A cast first gives back the hit points Poison took for good and recomputes those Age
    /// takes, before its dispel (Cure, `installation.md` §7.10; BAT-MAG-044, v20).
    pub restores_health: bool,
    /// When it is useful on a stack for a creature that draws among useful spells (BAT-MAG-050,
    /// v24).
    pub useful: Usefulness,
}

impl SpellMagic {
    /// The lowest mastery of its mass version: `mass_from`, or, without the original's earlier
    /// one (`early` false: quirk Q-0011 off), `fixed_mass_from` where it has one.
    pub fn mass_version(&self, early: bool) -> Option<u8> {
        if early {
            self.mass_from
        } else {
            self.fixed_mass_from.or(self.mass_from)
        }
    }

    /// What the caster names at `mastery`: nothing for the mass version
    /// ([`Self::mass_version`]).
    pub fn target_at(&self, mastery: u8, early: bool) -> SpellTarget {
        match self.mass_version(early) {
            Some(from) if mastery >= from => SpellTarget::None,
            _ => self.target,
        }
    }

    /// Whose stacks it may reach at `mastery`.
    pub fn side_at(&self, mastery: u8) -> TargetSide {
        match self.any_side_from {
            Some(from) if mastery >= from => TargetSide::Any,
            _ => self.side,
        }
    }

    /// Whether any effect is a timed change.
    pub fn is_timed(&self) -> bool {
        self.effects.iter().any(|e| e.timed().is_some())
    }

    /// Whether it has a dispel effect.
    pub fn dispels(&self) -> bool {
        self.effects.contains(&SpellEffect::Dispel)
    }

    /// The kind of obstacle it places, if it places one.
    pub fn obstacle_kind(&self) -> Option<ObstacleKind> {
        self.effects.iter().find_map(|e| match *e {
            SpellEffect::Obstacle(k) => Some(k),
            _ => None,
        })
    }

    /// Whether it moves its target stack to a second target (Teleport).
    pub fn teleports(&self) -> bool {
        self.effects.contains(&SpellEffect::Teleport)
    }

    /// Whether it removes the obstacle on its target hex (Remove Obstacle).
    pub fn removes_obstacles(&self) -> bool {
        self.effects.contains(&SpellEffect::RemoveObstacle)
    }

    /// Whether a cast at `mastery` removes a spell obstacle of kind `kind` (BAT-MAG-034).
    pub fn removes_at(&self, kind: ObstacleKind, mastery: u8) -> bool {
        self.removes_from[kind as usize].is_some_and(|m| mastery >= m)
    }

    /// Whether it raises the creatures of its target stack (Resurrection, Animate Dead).
    pub fn raises(&self) -> bool {
        self.effects.contains(&SpellEffect::Raise)
    }

    /// Whether it sacrifices its second target to raise its target (Sacrifice).
    pub fn sacrifices(&self) -> bool {
        self.effects.contains(&SpellEffect::Sacrifice)
    }

    /// Whether it copies its target stack (Clone).
    pub fn clones(&self) -> bool {
        self.effects.contains(&SpellEffect::Clone)
    }

    /// Whether it summons a stack (Summon Elemental).
    pub fn summons(&self) -> bool {
        self.effects.contains(&SpellEffect::Summon)
    }

    /// Whether a raise at `mastery` lasts only for the battle (BAT-MAG-035).
    pub fn temporary_at(&self, mastery: u8) -> bool {
        self.temporary_below.is_some_and(|m| mastery < m)
    }
}

impl SpellMagic {
    /// Why the targeting is invalid, if it is (`castable` is false for adventure spells, which
    /// have no battle effects).
    pub(crate) fn error(&self, castable: bool) -> Option<&'static str> {
        let c = self.conditions;
        if !castable && !self.effects.is_empty() {
            Some("adventure spells have no battle effects")
        } else if self.effects.len() > MAX_SPELL_EFFECTS {
            Some("more than 4 effects")
        } else if self
            .effects
            .iter()
            .enumerate()
            .any(|(i, e)| self.effects.get(..i).unwrap_or_default().contains(e))
        {
            Some("an effect is listed twice")
        } else if self.effects.is_empty() && *self != Self::default() {
            Some("targeting without effects")
        } else if self.area.is_some() && self.target != SpellTarget::Hex {
            Some("an area needs target hex")
        } else if self
            .area
            .is_some_and(|a| a.radius.iter().any(|&r| r < a.inner || r > MAX_AREA_RADIUS))
        {
            Some("area radius must be inner..=16")
        } else if self.chain.is_some() && self.target != SpellTarget::Stack {
            Some("a chain needs target stack")
        } else if self.chain.is_some_and(|ch| {
            ch.factor_bp > FULL_BP
                || ch
                    .targets
                    .iter()
                    .any(|&t| !(1..=MAX_CHAIN_TARGETS).contains(&t))
        }) {
            Some("a chain needs 1..=16 targets and factor_bp <= 10000")
        } else if c.requires.bits() & c.excludes.bits() != 0 {
            Some("a flag is both required and excluded")
        } else if self
            .mass_from
            .is_some_and(|m| self.target != SpellTarget::Stack || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("a mass version needs target stack and a mastery 1..=3")
        } else if (self.cumulative
            || self.duration != EffectDuration::Rounds
            || self.ends != EffectEnd::Never
            || self.health_limit
            || !self.refuses.is_empty())
            && !self.is_timed()
        {
            Some("duration, ends, cumulative, health_limit and refuses need a timed effect")
        } else if self.duration == EffectDuration::Fixed(0) {
            Some("a fixed duration needs at least 1 round")
        } else if self.duration == EffectDuration::Permanent
            && self
                .effects
                .iter()
                .any(|e| e.timed().is_some_and(TimedChange::takes_turns))
        {
            // No stack would ever act again once every stack has such an effect: the battle
            // would go on round after round within one command (E04b T11b review).
            Some("an effect that takes a stack's turns away cannot be permanent")
        } else if self
            .fixed_mass_from
            .is_some_and(|f| self.mass_from.is_none_or(|m| f <= m) || f > MAX_MASTERY)
        {
            Some("fixed_mass needs a mass version and a later mastery up to 3")
        } else if self
            .any_side_from
            .is_some_and(|m| self.side == TargetSide::Any || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("any_side needs a side and a mastery 1..=3")
        } else if self.needs_effect && self.target != SpellTarget::Stack {
            Some("needs_effect needs target stack")
        } else if self.dispels_positive && !self.dispels() {
            Some("dispels_positive needs a dispel effect")
        } else if self.restores_health && !self.effects.contains(&SpellEffect::Heal) {
            Some("restores_health needs a heal effect")
        } else {
            self.field_error()
        }
    }

    /// Why the battlefield effects (obstacles, Teleport; v13) are invalid, if they are.
    fn field_error(&self) -> Option<&'static str> {
        let field = self.effects.iter().any(|e| e.is_field());
        if field && self.effects.len() > 1 {
            Some("an obstacle or teleport effect is the spell's only effect")
        } else if field && (self.area.is_some() || self.chain.is_some() || self.mass_from.is_some())
        {
            Some("an obstacle or teleport spell has no area, chain or mass version")
        } else if self.obstacle_kind().is_some() != self.obstacle.is_some() {
            Some("an obstacle effect and its shape go together")
        } else if self.obstacle.as_ref().is_some_and(|o| o.random)
            && self.target != SpellTarget::None
        {
            Some("an obstacle effect with a random shape needs target none")
        } else if self.obstacle.as_ref().is_some_and(|o| !o.random)
            && self.target != SpellTarget::Hex
        {
            Some("an obstacle effect needs target hex")
        } else if self.teleports() && self.target != SpellTarget::Stack {
            Some("a teleport effect needs target stack")
        } else if self.removes_obstacles() && self.target != SpellTarget::Hex {
            Some("a remove_obstacle effect needs target hex")
        } else if self
            .removes_from
            .iter()
            .flatten()
            .any(|&m| !self.removes_obstacles() || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("removes needs a remove_obstacle effect and masteries 1..=3")
        } else if let Some(e) = self.obstacle.as_ref().and_then(ObstacleShape::error) {
            Some(e)
        } else if self
            .clears_obstacles_from
            .is_some_and(|m| !self.dispels() || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("clears_obstacles needs a dispel effect and a mastery 1..=3")
        } else {
            self.raise_error()
        }
    }

    /// Why the raise effects (Raise, Sacrifice; v17) are invalid, if they are.
    fn raise_error(&self) -> Option<&'static str> {
        let raise = self.effects.iter().any(|e| e.is_raise());
        if raise && self.effects.len() > 1 {
            Some("a raise or sacrifice effect is the spell's only effect")
        } else if raise
            && (self.target != SpellTarget::Stack
                || self.chain.is_some()
                || self.mass_from.is_some())
        {
            // An area needs target hex already.
            Some("a raise or sacrifice effect needs target stack, without chain or mass")
        } else if self
            .temporary_below
            .is_some_and(|m| !self.raises() || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("temporary_below needs a raise effect and a mastery 1..=3")
        } else {
            self.create_error()
        }
    }

    /// Why the effects that create stacks (Clone, Summon; v18) are invalid, if they are.
    fn create_error(&self) -> Option<&'static str> {
        let creates = self.effects.iter().any(|e| e.creates());
        if creates && self.effects.len() > 1 {
            Some("a clone or summon effect is the spell's only effect")
        } else if self.clones()
            && (self.target != SpellTarget::Stack
                || self.side != TargetSide::Own
                || self.any_side_from.is_some()
                || self.chain.is_some()
                || self.mass_from.is_some())
        {
            // The clone joins the caster's side, whose room the cast checks.
            Some("a clone effect needs target stack and side own, without chain or mass")
        } else if self.summons() && self.target != SpellTarget::None {
            // Without a target there is no area, chain or mass version.
            Some("a summon effect needs target none")
        } else {
            None
        }
    }
}

impl CanonicalHash for SpellMagic {
    /// Target, area (presence, inner, radii), chain (presence, targets, factor), side (v15),
    /// condition bits, effects (count, codes: kind and change), mind, mass (0: none), duration,
    /// cumulative (v17); any side (0: never), needs effect, ignores immunity (v19); the
    /// specialist bonus (presence, then the bonus; v20); what ends it, the fixed mass version
    /// (0: none; v21); a fixed duration's rounds, the health limit, the refused changes (count,
    /// codes; v22); the obstacle shape (presence, then the shape) and the mastery that clears
    /// obstacles (0: none; v23); the masteries that remove each obstacle kind (0: never; v25);
    /// the mastery from which a raise lasts (0: always; v26); then dispels positive, restores
    /// health; when it is useful (v33).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.target as u8);
        h.write_bool(self.area.is_some());
        if let Some(a) = self.area {
            h.write_u8(a.inner);
            h.write_bytes(&a.radius);
        }
        h.write_bool(self.chain.is_some());
        if let Some(ch) = self.chain {
            h.write_bytes(&ch.targets);
            h.write_u16(ch.factor_bp);
        }
        h.write_u8(self.side as u8);
        h.write_u32(self.conditions.requires.bits());
        h.write_u32(self.conditions.excludes.bits());
        h.write_bool(self.conditions.needs_damage);
        h.write_len(self.effects.len());
        for e in &self.effects {
            h.write_bytes(&e.code());
        }
        h.write_bool(self.mind);
        h.write_u8(self.mass_from.unwrap_or(0));
        let (duration, rounds) = self.duration.code();
        h.write_u8(duration);
        h.write_bool(self.cumulative);
        h.write_u8(self.any_side_from.unwrap_or(0));
        h.write_bool(self.needs_effect);
        h.write_bool(self.ignores_immunity);
        h.write_bool(self.specialist.is_some());
        if let Some(b) = self.specialist {
            b.hash_into(h);
        }
        h.write_u8(self.ends as u8);
        h.write_u8(self.fixed_mass_from.unwrap_or(0));
        h.write_u16(rounds);
        h.write_bool(self.health_limit);
        h.write_len(self.refuses.len());
        for &c in &self.refuses {
            h.write_u8(c as u8);
        }
        h.write_bool(self.obstacle.is_some());
        if let Some(shape) = &self.obstacle {
            shape.hash_into(h);
        }
        h.write_u8(self.clears_obstacles_from.unwrap_or(0));
        for m in self.removes_from {
            h.write_u8(m.unwrap_or(0));
        }
        h.write_u8(self.temporary_below.unwrap_or(0));
        h.write_bool(self.dispels_positive);
        h.write_bool(self.restores_health);
        h.write_u8(self.useful as u8);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;

    use super::*;
    use crate::flags::CreatureFlag;

    #[test]
    fn names_round_trip() {
        for t in SpellTarget::ALL {
            assert_eq!(SpellTarget::from_name(t.name()), Some(t));
        }
        for c in TimedChange::ALL {
            assert_eq!(TimedChange::from_name(c.name()), Some(c));
            assert_eq!(SpellEffect::Timed(c).name(), "timed");
            assert_eq!(SpellEffect::Timed(c).timed(), Some(c));
        }
        assert_eq!(SpellEffect::Damage.name(), "damage");
        assert_eq!(SpellEffect::Damage.timed(), None);
        assert_eq!(SpellEffect::Dispel.name(), "dispel");
        assert_eq!(SpellEffect::Heal.name(), "heal");
        assert_eq!(SpellEffect::Dispel.timed(), None);
        assert_eq!(SpellEffect::Heal.timed(), None);
        assert_eq!(TimedChange::from_name("heal"), None);
        for e in EffectEnd::ALL {
            assert_eq!(EffectEnd::from_name(e.name()), Some(e));
        }
        assert_eq!(EffectEnd::from_name("attacked"), None);
        for u in Usefulness::ALL {
            assert_eq!(Usefulness::from_name(u.name()), Some(u));
        }
        assert_eq!(Usefulness::from_name("never"), None);
        for s in TargetSide::ALL {
            assert_eq!(TargetSide::from_name(s.name()), Some(s));
        }
        assert_eq!(TargetSide::from_name("both"), None);
        assert_eq!(SpellTarget::from_name("obstacle"), None);
    }

    #[test]
    fn mass_versions_take_no_target() {
        let haste = SpellMagic {
            target: SpellTarget::Stack,
            mass_from: Some(3),
            effects: vec![SpellEffect::Timed(TimedChange::Speed)],
            ..SpellMagic::default()
        };
        assert_eq!(haste.target_at(2, true), SpellTarget::Stack);
        assert_eq!(haste.target_at(3, false), SpellTarget::None);
        let single = SpellMagic {
            mass_from: None,
            ..haste.clone()
        };
        assert_eq!(single.target_at(3, true), SpellTarget::Stack);
        // Forgetfulness: mass from Advanced in the original, at Expert without quirk Q-0011.
        let forget = SpellMagic {
            mass_from: Some(2),
            fixed_mass_from: Some(3),
            ..haste.clone()
        };
        assert_eq!(forget.target_at(2, true), SpellTarget::None);
        assert_eq!(forget.target_at(2, false), SpellTarget::Stack);
        assert_eq!(forget.target_at(3, false), SpellTarget::None);
        assert_eq!(forget.mass_version(false), Some(3));
        assert_eq!(forget.error(true), None);
        assert!(haste.is_timed());
        assert!(!SpellMagic::default().is_timed());
        assert!(!haste.dispels());
    }

    #[test]
    fn the_side_may_widen_by_mastery() {
        let dispel = SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Own,
            any_side_from: Some(2),
            effects: vec![SpellEffect::Dispel],
            ..SpellMagic::default()
        };
        assert_eq!(dispel.side_at(1), TargetSide::Own);
        assert_eq!(dispel.side_at(2), TargetSide::Any);
        assert_eq!(dispel.side_at(3), TargetSide::Any);
        assert!(dispel.dispels());
        let fixed = SpellMagic {
            any_side_from: None,
            ..dispel.clone()
        };
        assert_eq!(fixed.side_at(3), TargetSide::Own);
        assert_eq!(dispel.error(true), None);
    }

    #[test]
    fn conditions_admit_by_flags() {
        let undead = CreatureFlags::default().with(CreatureFlag::Undead);
        let machine = CreatureFlags::default().with(CreatureFlag::WarMachine);
        let only_undead = TargetConditions {
            requires: undead,
            ..TargetConditions::default()
        };
        let no_undead = TargetConditions {
            excludes: undead.with(CreatureFlag::WarMachine),
            ..TargetConditions::default()
        };
        assert!(only_undead.admits(undead.with(CreatureFlag::Flying), 1));
        assert!(!only_undead.admits(machine, 1));
        assert!(no_undead.admits(CreatureFlags::default(), 1));
        assert!(!no_undead.admits(undead, 1) && !no_undead.admits(machine, 1));
        assert!(TargetConditions::default().admits(undead, 0));
    }

    #[test]
    fn conditions_admit_by_damage() {
        let damaging = TargetConditions {
            needs_damage: true,
            ..TargetConditions::default()
        };
        let machine = CreatureFlags::default().with(CreatureFlag::WarMachine);
        assert!(damaging.admits(machine, 3));
        assert!(!damaging.admits(machine, 0));
    }

    #[test]
    fn targeting_is_validated() {
        let damage = |target| SpellMagic {
            target,
            effects: vec![SpellEffect::Damage],
            ..SpellMagic::default()
        };
        let area = SpellArea {
            inner: 1,
            radius: [1; 4],
        };
        let chain = SpellChain {
            targets: [4, 4, 5, 5],
            factor_bp: 5000,
        };
        let ok = [
            SpellMagic::default(),
            damage(SpellTarget::None),
            SpellMagic {
                area: Some(area),
                ..damage(SpellTarget::Hex)
            },
            SpellMagic {
                chain: Some(chain),
                ..damage(SpellTarget::Stack)
            },
        ];
        for m in &ok {
            assert_eq!(m.error(true), None, "{m:?}");
        }
        assert_eq!(SpellMagic::default().error(false), None);
        let undead = CreatureFlags::default().with(CreatureFlag::Undead);
        let bad = [
            (damage(SpellTarget::Stack), false, "adventure spells"),
            (
                SpellMagic {
                    effects: vec![SpellEffect::Damage; 5],
                    ..SpellMagic::default()
                },
                true,
                "more than 4",
            ),
            (
                SpellMagic {
                    effects: vec![SpellEffect::Damage; 2],
                    ..SpellMagic::default()
                },
                true,
                "listed twice",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Stack,
                    ..SpellMagic::default()
                },
                true,
                "without effects",
            ),
            (
                SpellMagic {
                    area: Some(area),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "needs target hex",
            ),
            (
                SpellMagic {
                    area: Some(SpellArea {
                        inner: 2,
                        radius: [2, 2, 1, 2],
                    }),
                    ..damage(SpellTarget::Hex)
                },
                true,
                "inner..=16",
            ),
            (
                SpellMagic {
                    area: Some(SpellArea {
                        inner: 0,
                        radius: [17; 4],
                    }),
                    ..damage(SpellTarget::Hex)
                },
                true,
                "inner..=16",
            ),
            (
                SpellMagic {
                    chain: Some(chain),
                    ..damage(SpellTarget::Hex)
                },
                true,
                "needs target stack",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [0, 4, 4, 4],
                        ..chain
                    }),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "1..=16 targets",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [17; 4],
                        ..chain
                    }),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "1..=16 targets",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        factor_bp: 10_001,
                        ..chain
                    }),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "1..=16 targets",
            ),
            (
                SpellMagic {
                    conditions: TargetConditions {
                        requires: undead,
                        excludes: undead,
                        needs_damage: false,
                    },
                    ..damage(SpellTarget::None)
                },
                true,
                "both required and excluded",
            ),
        ];
        let timed = |change| SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Timed(change)],
            ..SpellMagic::default()
        };
        let ray = SpellMagic {
            duration: EffectDuration::Permanent,
            cumulative: true,
            ..timed(TimedChange::DefenseLoss)
        };
        assert_eq!(ray.error(true), None);
        let berserk = SpellMagic {
            target: SpellTarget::Hex,
            area: Some(SpellArea {
                inner: 0,
                radius: [0, 0, 1, 2],
            }),
            duration: EffectDuration::Fixed(255),
            ends: EffectEnd::OwnAttack,
            ..timed(TimedChange::Berserk)
        };
        assert_eq!(berserk.error(true), None);
        let hypnotize = SpellMagic {
            health_limit: true,
            refuses: vec![TimedChange::MagicMirror],
            ..timed(TimedChange::Hypnotize)
        };
        assert_eq!(hypnotize.error(true), None);
        assert_eq!(
            SpellMagic {
                mass_from: Some(3),
                mind: true,
                ..timed(TimedChange::Morale)
            }
            .error(true),
            None
        );
        let other = |side, any_side_from, needs_effect| SpellMagic {
            target: SpellTarget::Stack,
            side,
            any_side_from,
            needs_effect,
            effects: vec![SpellEffect::Dispel],
            ..SpellMagic::default()
        };
        assert_eq!(other(TargetSide::Own, Some(2), true).error(true), None);
        let bad = bad.into_iter().chain([
            (
                other(TargetSide::Any, Some(2), false),
                true,
                "any_side needs",
            ),
            (
                other(TargetSide::Own, Some(0), false),
                true,
                "any_side needs",
            ),
            (
                other(TargetSide::Own, Some(4), false),
                true,
                "any_side needs",
            ),
            (
                SpellMagic {
                    target: SpellTarget::None,
                    ..other(TargetSide::Own, None, true)
                },
                true,
                "needs_effect needs",
            ),
            (
                SpellMagic {
                    dispels_positive: true,
                    ..timed(TimedChange::Speed)
                },
                true,
                "dispels_positive needs",
            ),
            (
                SpellMagic {
                    restores_health: true,
                    ..timed(TimedChange::Speed)
                },
                true,
                "restores_health needs",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Hex,
                    mass_from: Some(3),
                    ..timed(TimedChange::Speed)
                },
                true,
                "mass version",
            ),
            (
                SpellMagic {
                    mass_from: Some(0),
                    ..timed(TimedChange::Speed)
                },
                true,
                "mass version",
            ),
            (
                SpellMagic {
                    cumulative: true,
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
            (
                SpellMagic {
                    specialist: Some(SpecialistBonus::Fixed(2)),
                    ..SpellMagic::default()
                },
                true,
                "targeting without effects",
            ),
            (
                SpellMagic {
                    duration: EffectDuration::Permanent,
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
            (
                SpellMagic {
                    ends: EffectEnd::Damaged,
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
            (
                SpellMagic {
                    fixed_mass_from: Some(3),
                    ..timed(TimedChange::Blind)
                },
                true,
                "fixed_mass needs",
            ),
            (
                SpellMagic {
                    duration: EffectDuration::Fixed(0),
                    ..timed(TimedChange::Berserk)
                },
                true,
                "at least 1 round",
            ),
            // E04b T11b review (F-robust-4): a stack that never gets a turn again.
            (
                SpellMagic {
                    duration: EffectDuration::Permanent,
                    ..timed(TimedChange::Blind)
                },
                true,
                "cannot be permanent",
            ),
            (
                SpellMagic {
                    duration: EffectDuration::Permanent,
                    ..timed(TimedChange::Petrify)
                },
                true,
                "cannot be permanent",
            ),
            (
                SpellMagic {
                    duration: EffectDuration::Permanent,
                    effects: vec![
                        SpellEffect::Timed(TimedChange::DefenseLoss),
                        SpellEffect::Timed(TimedChange::Paralyze),
                    ],
                    ..timed(TimedChange::Paralyze)
                },
                true,
                "cannot be permanent",
            ),
            (
                SpellMagic {
                    health_limit: true,
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
            (
                SpellMagic {
                    refuses: vec![TimedChange::MagicMirror],
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
            (
                SpellMagic {
                    mass_from: Some(3),
                    fixed_mass_from: Some(3),
                    ..timed(TimedChange::Blind)
                },
                true,
                "fixed_mass needs",
            ),
            (
                SpellMagic {
                    mass_from: Some(2),
                    fixed_mass_from: Some(4),
                    ..timed(TimedChange::Blind)
                },
                true,
                "fixed_mass needs",
            ),
        ]);
        for (m, castable, reason) in bad {
            let error = m.error(castable).unwrap_or_default();
            assert!(error.contains(reason), "{m:?}: {error}");
        }
    }

    #[test]
    fn specialist_bonuses_by_kind() {
        let table = SpecialistBonus::CreatureLevel([3, 3, 2, 2, 1, 1, 0]);
        assert_eq!(table.bonus(1, 6), 3);
        assert_eq!(table.bonus(4, 6), 2);
        assert_eq!(table.bonus(7, 6), 0);
        // Level 0 counts as 1, levels above 7 as 7.
        assert_eq!(table.bonus(0, 6), 3);
        assert_eq!(table.bonus(9, 6), 0);
        assert_eq!(SpecialistBonus::Fixed(2).bonus(3, 5), 2);
        assert_eq!(SpecialistBonus::Total(3).bonus(3, 1), 2);
        assert_eq!(SpecialistBonus::Total(3).bonus(3, 5), -2);
    }

    #[test]
    fn hash_covers_every_field() {
        let digest = h3_kernel::hash::hash_of::<SpellMagic>;
        let base = SpellMagic {
            target: SpellTarget::Hex,
            area: Some(SpellArea {
                inner: 0,
                radius: [1; 4],
            }),
            chain: None,
            side: TargetSide::Any,
            conditions: TargetConditions::default(),
            effects: vec![SpellEffect::Damage],
            ..SpellMagic::default()
        };
        let undead = CreatureFlags::default().with(CreatureFlag::Undead);
        let variants = [
            SpellMagic {
                target: SpellTarget::Stack,
                area: None,
                chain: Some(SpellChain {
                    targets: [1; 4],
                    factor_bp: 0,
                }),
                ..base.clone()
            },
            SpellMagic {
                area: Some(SpellArea {
                    inner: 1,
                    radius: [1; 4],
                }),
                ..base.clone()
            },
            SpellMagic {
                conditions: TargetConditions {
                    requires: undead,
                    excludes: CreatureFlags::default(),
                    needs_damage: false,
                },
                ..base.clone()
            },
            SpellMagic {
                conditions: TargetConditions {
                    requires: CreatureFlags::default(),
                    excludes: undead,
                    needs_damage: false,
                },
                ..base.clone()
            },
            SpellMagic {
                conditions: TargetConditions {
                    needs_damage: true,
                    ..TargetConditions::default()
                },
                ..base.clone()
            },
            SpellMagic {
                effects: vec![],
                ..base.clone()
            },
            SpellMagic {
                side: TargetSide::Enemy,
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Timed(TimedChange::Speed)],
                ..base.clone()
            },
            SpellMagic {
                mind: true,
                ..base.clone()
            },
            SpellMagic {
                mass_from: Some(3),
                ..base.clone()
            },
            SpellMagic {
                duration: EffectDuration::Permanent,
                ..base.clone()
            },
            SpellMagic {
                cumulative: true,
                ..base.clone()
            },
            SpellMagic {
                any_side_from: Some(2),
                ..base.clone()
            },
            SpellMagic {
                needs_effect: true,
                ..base.clone()
            },
            SpellMagic {
                ends: EffectEnd::Damaged,
                ..base.clone()
            },
            SpellMagic {
                fixed_mass_from: Some(3),
                ..base.clone()
            },
            SpellMagic {
                duration: EffectDuration::OwnTurn,
                ..base.clone()
            },
            SpellMagic {
                duration: EffectDuration::Fixed(255),
                ..base.clone()
            },
            SpellMagic {
                health_limit: true,
                ..base.clone()
            },
            SpellMagic {
                refuses: vec![TimedChange::MagicMirror],
                ..base.clone()
            },
            SpellMagic {
                ignores_immunity: true,
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Dispel],
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Heal],
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::CreatureLevel([3, 3, 2, 2, 1, 1, 0])),
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::CreatureLevel([4, 3, 2, 1, 0, 0, 0])),
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::Fixed(2)),
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::Total(2)),
                ..base.clone()
            },
            SpellMagic {
                temporary_below: Some(2),
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Raise],
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Sacrifice],
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Clone],
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Summon],
                ..base.clone()
            },
        ];
        let timed = |c| SpellMagic {
            effects: vec![SpellEffect::Timed(c)],
            ..base.clone()
        };
        assert_ne!(
            digest(&timed(TimedChange::Speed)),
            digest(&timed(TimedChange::SpeedPercent))
        );
        for v in &variants {
            assert_ne!(digest(v), digest(&base), "{v:?}");
        }
        let fixed = |rounds| SpellMagic {
            duration: EffectDuration::Fixed(rounds),
            ..base.clone()
        };
        assert_ne!(digest(&fixed(254)), digest(&fixed(255)));
        let wall = |shape: ObstacleShape| SpellMagic {
            target: SpellTarget::Hex,
            effects: vec![SpellEffect::Obstacle(ObstacleKind::Wall)],
            obstacle: Some(shape),
            ..SpellMagic::default()
        };
        let shape = ObstacleShape {
            steps: vec![HexStep::TopRight],
            hexes: [2; 4],
            random: false,
            mirrored: false,
            separate: false,
            rounds: 2,
        };
        let shapes = [
            ObstacleShape {
                steps: vec![HexStep::TopLeft],
                ..shape.clone()
            },
            ObstacleShape {
                hexes: [1, 1, 2, 2],
                ..shape.clone()
            },
            ObstacleShape {
                mirrored: true,
                ..shape.clone()
            },
            ObstacleShape {
                separate: true,
                ..shape.clone()
            },
            ObstacleShape {
                rounds: 3,
                ..shape.clone()
            },
            ObstacleShape {
                random: true,
                ..shape.clone()
            },
        ];
        for s in shapes {
            assert_ne!(
                digest(&wall(s.clone())),
                digest(&wall(shape.clone())),
                "{s:?}"
            );
        }
        let fire = SpellMagic {
            effects: vec![SpellEffect::Obstacle(ObstacleKind::Fire)],
            ..wall(shape.clone())
        };
        assert_ne!(digest(&fire), digest(&wall(shape)));
        let dispel = SpellMagic {
            effects: vec![SpellEffect::Dispel],
            ..base.clone()
        };
        let clears = SpellMagic {
            clears_obstacles_from: Some(3),
            ..dispel.clone()
        };
        assert_ne!(digest(&clears), digest(&dispel));
    }

    #[test]
    fn battlefield_effects() {
        for k in ObstacleKind::ALL {
            assert_eq!(ObstacleKind::from_name(k.name()), Some(k));
            let e = SpellEffect::Obstacle(k);
            assert_eq!(
                (e.name(), e.timed(), e.is_field()),
                ("obstacle", None, true)
            );
        }
        assert_eq!(ObstacleKind::from_name("trap"), None);
        assert!(ObstacleKind::Pit.is_hidden() && ObstacleKind::Mine.is_hidden());
        assert!(!ObstacleKind::Wall.is_hidden() && !ObstacleKind::Fire.is_hidden());
        assert!(ObstacleKind::Fire.has_value() && ObstacleKind::Mine.has_value());
        assert!(!ObstacleKind::Wall.has_value() && !ObstacleKind::Pit.has_value());
        for s in HexStep::ALL {
            assert_eq!(HexStep::from_name(s.name()), Some(s));
            assert_eq!(s.mirrored().mirrored(), s);
            assert_ne!(s.mirrored(), s);
        }
        assert_eq!(HexStep::TopRight.mirrored(), HexStep::TopLeft);
        assert_eq!(HexStep::from_name("up"), None);
        let teleport = SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Own,
            effects: vec![SpellEffect::Teleport],
            ..SpellMagic::default()
        };
        assert_eq!(SpellEffect::Teleport.name(), "teleport");
        assert_eq!(SpellEffect::Teleport.timed(), None);
        assert!(teleport.teleports() && !SpellEffect::Damage.is_field());
        assert_eq!(teleport.obstacle_kind(), None);
        assert_eq!(teleport.error(true), None);
        let shape = ObstacleShape {
            steps: vec![HexStep::TopRight, HexStep::TopLeft],
            hexes: [2, 2, 3, 3],
            random: false,
            mirrored: true,
            separate: true,
            rounds: 2,
        };
        let fire = SpellMagic {
            target: SpellTarget::Hex,
            effects: vec![SpellEffect::Obstacle(ObstacleKind::Fire)],
            obstacle: Some(shape.clone()),
            ..SpellMagic::default()
        };
        assert_eq!(fire.error(true), None);
        assert_eq!(fire.obstacle_kind(), Some(ObstacleKind::Fire));
        assert!(!fire.teleports());
        // Quicksand: random pits for the whole battle, no target.
        let random = ObstacleShape {
            steps: vec![],
            hexes: [4, 4, 6, 8],
            random: true,
            mirrored: false,
            separate: true,
            rounds: 0,
        };
        let pits = SpellMagic {
            target: SpellTarget::None,
            effects: vec![SpellEffect::Obstacle(ObstacleKind::Pit)],
            obstacle: Some(random.clone()),
            ..SpellMagic::default()
        };
        assert_eq!(pits.error(true), None);
        let dispel = SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Dispel],
            clears_obstacles_from: Some(3),
            ..SpellMagic::default()
        };
        assert_eq!(dispel.error(true), None);
        // Remove Obstacle: natural obstacles always, fire from Advanced, the rest at Expert.
        let remove = SpellMagic {
            target: SpellTarget::Hex,
            effects: vec![SpellEffect::RemoveObstacle],
            removes_from: [Some(3), Some(2), Some(3), Some(3)],
            ..SpellMagic::default()
        };
        assert_eq!(remove.error(true), None);
        assert_eq!(SpellEffect::RemoveObstacle.name(), "remove_obstacle");
        assert_eq!(SpellEffect::RemoveObstacle.timed(), None);
        assert!(SpellEffect::RemoveObstacle.is_field());
        assert!(remove.removes_obstacles() && !fire.removes_obstacles());
        assert!(!remove.removes_at(ObstacleKind::Fire, 1));
        assert!(
            remove.removes_at(ObstacleKind::Fire, 2) && !remove.removes_at(ObstacleKind::Mine, 2)
        );
        assert!(remove.removes_at(ObstacleKind::Mine, 3));
        assert!(!fire.removes_at(ObstacleKind::Fire, 3));
        // Raising (v17): Resurrection, temporary below Advanced, and Sacrifice.
        let raise = SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Raise],
            temporary_below: Some(2),
            ..SpellMagic::default()
        };
        let sacrifice = SpellMagic {
            effects: vec![SpellEffect::Sacrifice],
            temporary_below: None,
            ..raise.clone()
        };
        assert_eq!((raise.error(true), sacrifice.error(true)), (None, None));
        assert_eq!(
            (SpellEffect::Raise.name(), SpellEffect::Sacrifice.name()),
            ("raise", "sacrifice")
        );
        assert_eq!(
            (SpellEffect::Raise.timed(), SpellEffect::Sacrifice.timed()),
            (None, None)
        );
        assert!(SpellEffect::Raise.is_raise() && SpellEffect::Sacrifice.is_raise());
        assert!(!SpellEffect::Raise.is_field() && !SpellEffect::Heal.is_raise());
        assert!(raise.raises() && !raise.sacrifices() && sacrifice.sacrifices());
        assert!(raise.temporary_at(1) && !raise.temporary_at(2) && !sacrifice.temporary_at(0));
        let bad = [
            (
                SpellMagic {
                    target: SpellTarget::Stack,
                    ..remove.clone()
                },
                "remove_obstacle effect needs target hex",
            ),
            (
                SpellMagic {
                    removes_from: [None, Some(0), None, None],
                    ..remove.clone()
                },
                "removes needs",
            ),
            (
                SpellMagic {
                    removes_from: [Some(3), None, None, None],
                    ..fire.clone()
                },
                "removes needs",
            ),
            (
                SpellMagic {
                    effects: vec![SpellEffect::Teleport, SpellEffect::Damage],
                    ..teleport.clone()
                },
                "only effect",
            ),
            (
                SpellMagic {
                    mass_from: Some(3),
                    ..teleport.clone()
                },
                "no area, chain or mass",
            ),
            (
                SpellMagic {
                    area: Some(SpellArea {
                        inner: 0,
                        radius: [1; 4],
                    }),
                    ..fire.clone()
                },
                "no area, chain or mass",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [2; 4],
                        factor_bp: 5000,
                    }),
                    ..teleport.clone()
                },
                "no area, chain or mass",
            ),
            (
                SpellMagic {
                    obstacle: None,
                    ..fire.clone()
                },
                "go together",
            ),
            (
                SpellMagic {
                    obstacle: Some(shape.clone()),
                    ..teleport.clone()
                },
                "go together",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Stack,
                    ..fire.clone()
                },
                "needs target hex",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Hex,
                    ..teleport.clone()
                },
                "needs target stack",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        steps: vec![HexStep::Right; 8],
                        ..shape.clone()
                    }),
                    ..fire.clone()
                },
                "at most 8",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        hexes: [0, 2, 3, 3],
                        ..shape.clone()
                    }),
                    ..fire.clone()
                },
                "1..=steps + 1",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        hexes: [2, 2, 3, 4],
                        ..shape.clone()
                    }),
                    ..fire.clone()
                },
                "1..=steps + 1",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Hex,
                    ..pits.clone()
                },
                "random shape needs target none",
            ),
            (
                SpellMagic {
                    target: SpellTarget::None,
                    ..fire.clone()
                },
                "needs target hex",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        steps: vec![HexStep::Right],
                        ..random.clone()
                    }),
                    ..pits.clone()
                },
                "no steps",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        mirrored: true,
                        ..random.clone()
                    }),
                    ..pits.clone()
                },
                "not mirrored",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        separate: false,
                        ..random.clone()
                    }),
                    ..pits.clone()
                },
                "is separate",
            ),
            (
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        hexes: [4, 4, 6, 9],
                        ..random.clone()
                    }),
                    ..pits.clone()
                },
                "random: 1..=8",
            ),
            (
                SpellMagic {
                    clears_obstacles_from: Some(0),
                    ..dispel.clone()
                },
                "clears_obstacles",
            ),
            (
                SpellMagic {
                    clears_obstacles_from: Some(4),
                    ..dispel.clone()
                },
                "clears_obstacles",
            ),
            (
                SpellMagic {
                    clears_obstacles_from: Some(3),
                    ..teleport.clone()
                },
                "clears_obstacles",
            ),
            (
                SpellMagic {
                    effects: vec![SpellEffect::Raise, SpellEffect::Heal],
                    ..raise.clone()
                },
                "only effect",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Hex,
                    temporary_below: None,
                    ..raise.clone()
                },
                "raise or sacrifice effect needs target stack",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [2; 4],
                        factor_bp: 5000,
                    }),
                    ..sacrifice.clone()
                },
                "raise or sacrifice effect needs target stack",
            ),
            (
                SpellMagic {
                    mass_from: Some(3),
                    ..raise.clone()
                },
                "raise or sacrifice effect needs target stack",
            ),
            (
                SpellMagic {
                    temporary_below: Some(2),
                    ..sacrifice.clone()
                },
                "temporary_below",
            ),
            (
                SpellMagic {
                    temporary_below: Some(4),
                    ..raise.clone()
                },
                "temporary_below",
            ),
            (
                SpellMagic {
                    temporary_below: Some(0),
                    ..raise.clone()
                },
                "temporary_below",
            ),
        ];
        for (m, want) in bad {
            let got = m.error(true);
            assert!(got.is_some_and(|e| e.contains(want)), "{m:?}: {got:?}");
        }
    }

    #[test]
    fn clone_and_summon_effects_create_stacks() {
        // Clone takes a stack, Summon Elemental nothing (v18).
        let clone = SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Own,
            effects: vec![SpellEffect::Clone],
            ..SpellMagic::default()
        };
        let summon = SpellMagic {
            effects: vec![SpellEffect::Summon],
            ..SpellMagic::default()
        };
        assert_eq!((clone.error(true), summon.error(true)), (None, None));
        assert_eq!(
            (SpellEffect::Clone.name(), SpellEffect::Summon.name()),
            ("clone", "summon")
        );
        assert_eq!(
            (SpellEffect::Clone.timed(), SpellEffect::Summon.timed()),
            (None, None)
        );
        assert!(SpellEffect::Clone.creates() && SpellEffect::Summon.creates());
        assert!(!SpellEffect::Raise.creates() && !SpellEffect::Clone.is_raise());
        assert!(clone.clones() && !clone.summons() && summon.summons() && !summon.clones());
        let bad = [
            (
                SpellMagic {
                    effects: vec![SpellEffect::Summon, SpellEffect::Heal],
                    ..summon.clone()
                },
                "clone or summon effect is the spell's only effect",
            ),
            (
                SpellMagic {
                    target: SpellTarget::None,
                    ..clone.clone()
                },
                "clone effect needs target stack",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [2; 4],
                        factor_bp: 5000,
                    }),
                    ..clone.clone()
                },
                "clone effect needs target stack",
            ),
            (
                SpellMagic {
                    mass_from: Some(3),
                    ..clone.clone()
                },
                "clone effect needs target stack",
            ),
            (
                SpellMagic {
                    side: TargetSide::Enemy,
                    ..clone.clone()
                },
                "clone effect needs target stack and side own",
            ),
            (
                SpellMagic {
                    any_side_from: Some(3),
                    ..clone.clone()
                },
                "clone effect needs target stack and side own",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Stack,
                    ..summon.clone()
                },
                "summon effect needs target none",
            ),
        ];
        for (m, want) in bad {
            let got = m.error(true);
            assert!(got.is_some_and(|e| e.contains(want)), "{m:?}: {got:?}");
        }
    }
}
