//! The encounter distribution v1 (E05 T5b1, decision 7; `docs/epics/E05-log.md` §4.2 to §4.4):
//! a sampler that turns a seed into a battle as the adventure would bring it, its kind, game
//! week, place, armies, heroes and town drawn by the census of the shipped maps
//! (`data/census-v1.toml`) and the stated assumptions A1 to A12 (`data/encounters-v1.toml`).
//!
//! * Every draw of an encounter comes from stream `ENCOUNTER` of its seed, so a seed always gives the
//!   same encounter on the same rulesets; its battle's own draws (the obstacles, the battle) come
//!   from the other streams of the same seed.
//! * The game's rules are used where the engine has them: a random monster's creature
//!   (ADV-MON-003) and count (ADV-MON-004), the monster's split (ADV-MON-009), a hero's battle
//!   values (`h3_heroes::derive_battle_hero`, its spell points, its war machines). The rest are
//!   the assumptions, approximations and not rules: a hero's level-ups, its spells and artifacts,
//!   the armies, the towns.
//! * The neutral kinds (wandering monsters, guards, a neutral town) have the system actor as their
//!   defender ([`Policy::System`], A12; E06 T7). They are drawn at A1's shares by
//!   [`Sampler::with_shares`] or by a caller's weights; by default they weigh 0, so the held-out
//!   sets keep their episodes (E06 T7, the user).
//!
//! Numbers are integers (permille), so a sample is the same on every platform (ADR-0003).

use std::sync::Arc;

use h3_adventure::{AdventureRuleset, split};
use h3_content::{
    ContentKey, CreatureFlag, CreatureTypeId, FactionId, HeroClassId, HeroTypeId, Ruleset,
    TerrainId,
};
use h3_content_battle::BattleRuleset;
use h3_contracts::{ArmyStack, BattleSetup, FortLevel, Moat, Siege};
use h3_game::GameRuleset;
use h3_kernel::{Rng, StreamId};

use crate::batch::{Policy, Seat};

mod hero;
mod table;
#[cfg(test)]
pub(crate) mod tests;

pub use table::{EncounterTable, KindRow, LEVELS, Place, QUANTILES, Quantiles, WEEKS};

/// The shipped distribution's text (`data/encounters-v1.toml`).
pub const ENCOUNTERS_V1: &str = include_str!("../data/encounters-v1.toml");

/// The version of the encounter distribution this build plays (its file's
/// `distribution.version`, on the census it names); a run's record shows it (E06 T2c).
pub const ENCOUNTERS_VERSION: u32 = 1;

/// The shipped census's text (`data/census-v1.toml`), written by `h3 ai census`.
pub const CENSUS_V1: &str = include_str!("../data/census-v1.toml");

/// How often a neutral stack whose army would exceed the week's cap is drawn again (A4) before
/// the last draw is kept.
pub const MAX_REDRAWS: u32 = 64;

/// A kind of encounter (A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A hero attacks a wandering monster, split into stacks (ADV-MON-009).
    Wandering,
    /// A hero attacks the guards of a pile or a garrison, as they stand.
    Guards,
    /// A hero attacks a hero in the field.
    Hero,
    /// A hero attacks a town a hero defends.
    SiegeHero,
    /// A hero attacks a player's town without a hero: its garrison.
    SiegeGarrison,
    /// A hero attacks a neutral town: its garrison, the system actor's.
    SiegeNeutral,
}

impl Kind {
    /// Every kind, in the distribution's order.
    pub const ALL: [Self; 6] = [
        Self::Wandering,
        Self::Guards,
        Self::Hero,
        Self::SiegeHero,
        Self::SiegeGarrison,
        Self::SiegeNeutral,
    ];

    /// Its key in `encounters-v1.toml`.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Wandering => "wandering",
            Self::Guards => "guards",
            Self::Hero => "hero",
            Self::SiegeHero => "siege_hero",
            Self::SiegeGarrison => "siege_garrison",
            Self::SiegeNeutral => "siege_neutral",
        }
    }

    /// Whether its defender is neutral, the system actor's (A12).
    pub const fn neutral(self) -> bool {
        matches!(self, Self::Wandering | Self::Guards | Self::SiegeNeutral)
    }

    /// Its position in [`Kind::ALL`].
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Why a distribution or a sampler was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncounterError {
    /// A file does not parse into its shape (the parser's message).
    Toml(String),
    /// The census is not the one the distribution names (the census's own name).
    Census(String),
    /// A number of the distribution is out of shape.
    Table {
        /// The field.
        field: &'static str,
        /// What is wrong.
        reason: &'static str,
    },
    /// A key the distribution names is not the ruleset's.
    Unknown {
        /// What it should name.
        what: &'static str,
        /// The key.
        key: String,
    },
    /// The rulesets do not belong together, or lack what a draw needs.
    Rules(&'static str),
}

impl std::fmt::Display for EncounterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml(e) => write!(f, "toml: {e}"),
            Self::Census(name) => write!(f, "the distribution names another census than {name}"),
            Self::Table { field, reason } => write!(f, "`{field}`: {reason}"),
            Self::Unknown { what, key } => write!(f, "unknown {what} `{key}`"),
            Self::Rules(reason) => write!(f, "rulesets: {reason}"),
        }
    }
}

impl std::error::Error for EncounterError {}

/// What an encounter drew besides its setup: for the frequency check and logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facts {
    /// The census place's index (neutral stacks, guards, hero battles), if any.
    pub place: Option<usize>,
    /// The neutral stack's level (wandering, guards).
    pub level: Option<u8>,
    /// The neutral stack's count before its split.
    pub count: u32,
    /// How often the neutral stack was drawn again (A4).
    pub redraws: u32,
    /// The ratio drawn (‰): the army over a neutral stack (A4) or the attacker over the defender
    /// (A5); 0 when none was drawn.
    pub ratio: u32,
    /// The army factor `k` drawn (‰; A5); 0 when none was drawn.
    pub k: u32,
    /// Each side's faction (indexed by `BattleSide::index`): a hero's, or the town's.
    pub factions: [Option<FactionId>; 2],
    /// Each side's hero's level.
    pub hero_levels: [Option<u8>; 2],
    /// Each side's army value (ai_value × count).
    pub values: [u64; 2],
    /// The town's fort (sieges).
    pub fort: Option<FortLevel>,
}

/// One sampled battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encounter {
    /// Its kind.
    pub kind: Kind,
    /// Its game week, 1 to [`WEEKS`].
    pub week: u8,
    /// The battle, its obstacles left to the episode (`random_obstacles`); both sides play as
    /// human players' (ADR-0025 point 6).
    pub setup: BattleSetup,
    /// Who plays each side (A12): the system actor's internal seat for a neutral defender.
    pub seats: [Seat; 2],
    /// Whether the episode generates its obstacles (always, as the adventure's battles do).
    pub random_obstacles: bool,
    /// What it drew.
    pub facts: Facts,
}

/// An encounter being drawn: its setup and what it drew.
#[derive(Debug)]
struct Draft {
    setup: BattleSetup,
    facts: Facts,
}

/// A battle place, resolved.
#[derive(Debug, Clone)]
struct PlaceRef {
    terrain: TerrainId,
    battlefield: Option<ContentKey>,
}

/// A faction's army data, resolved (A5, A6): per level its town creature, its first upgrade and
/// its growth value.
#[derive(Debug, Clone)]
struct FactionArmy {
    faction: FactionId,
    creatures: [(CreatureTypeId, CreatureTypeId); LEVELS],
    growth: [u32; LEVELS],
    growth_value: [u64; LEVELS],
    /// The summed growth value of the levels each week reaches, over the weeks so far (A5).
    cumulative: [u64; WEEKS],
    classes: Vec<(HeroClassId, Vec<HeroTypeId>)>,
    guild_max: u8,
    terrain: TerrainId,
}

/// The sampler: a distribution resolved against the original packs' rulesets.
#[derive(Debug, Clone)]
pub struct Sampler {
    rules: GameRuleset,
    battle: Arc<BattleRuleset>,
    adventure: Arc<AdventureRuleset>,
    table: EncounterTable,
    weights: [u32; 6],
    places: Vec<PlaceRef>,
    water: ContentKey,
    factions: Vec<FactionArmy>,
    faction_weights: Vec<u32>,
    levels: [Vec<CreatureTypeId>; LEVELS],
    hero: hero::HeroData,
}

/// The index drawn by `weights` (positive sum fitting a `u32`, checked when the table was
/// built).
fn pick(rng: &mut Rng, weights: impl Iterator<Item = u32> + Clone) -> usize {
    let total: u32 = weights.clone().sum();
    let mut point = rng.below(total);
    weights
        .take_while(|&w| {
            let past = point >= w;
            point = point.saturating_sub(w);
            past
        })
        .count()
}

/// A draw of `quantiles`.
fn draw(rng: &mut Rng, quantiles: &Quantiles) -> u32 {
    let bin = rng.below(QUANTILES as u32 - 1) as usize;
    quantiles.at(bin, rng.below(1000))
}

/// Whether a draw with the chance `permille` succeeds.
fn chance(rng: &mut Rng, permille: u32) -> bool {
    rng.below(1000) < permille
}

/// `value × permille / 1000`, saturating.
fn scale(value: u64, permille: u32) -> u64 {
    value.saturating_mul(u64::from(permille)) / 1000
}

impl Sampler {
    /// The sampler of `table` on the battle part `battle` and the adventure part `adventure` of
    /// the same core (the original packs: `base`, `ab`, `sod`), with the table's default weights.
    ///
    /// # Errors
    /// [`EncounterError::Rules`] when the parts belong to different cores or a faction lacks a
    /// level, a class or its heroes; [`EncounterError::Unknown`] for a key the rulesets lack.
    pub fn new(
        battle: Arc<BattleRuleset>,
        adventure: Arc<AdventureRuleset>,
        table: &EncounterTable,
    ) -> Result<Self, EncounterError> {
        let core = Arc::clone(battle.core());
        let rules = GameRuleset::new(
            Arc::clone(&core),
            Arc::clone(&adventure),
            Arc::clone(&battle),
        )
        .map_err(|_| EncounterError::Rules("the battle and adventure parts differ"))?;
        let unknown = |what: &'static str, key: &str| EncounterError::Unknown {
            what,
            key: key.to_owned(),
        };
        let content_key =
            |what: &'static str, key: &str| ContentKey::parse(key).map_err(|_| unknown(what, key));
        let battlefield = |key: &str| {
            battle
                .battlefield_by_key(key)
                .map(|_| ())
                .ok_or_else(|| unknown("battlefield", key))
                .and_then(|()| content_key("battlefield", key))
        };
        let places = table
            .places
            .iter()
            .map(|place| {
                Ok(PlaceRef {
                    terrain: core
                        .terrain_by_key(&place.terrain)
                        .ok_or_else(|| unknown("terrain", &place.terrain))?,
                    battlefield: place.battlefield.as_deref().map(battlefield).transpose()?,
                })
            })
            .collect::<Result<Vec<_>, EncounterError>>()?;
        let water = battlefield(&table.water_battlefield)?;
        let town_terrain = core
            .terrain_by_key(&table.town_terrain)
            .ok_or_else(|| unknown("terrain", &table.town_terrain))?;
        let factions = table
            .factions
            .iter()
            .map(|(key, _)| {
                let faction = core
                    .faction_by_key(key)
                    .ok_or_else(|| unknown("faction", key))?;
                let guild_max = table.guild_max.get(key).copied();
                faction_army(&core, table, faction, guild_max, town_terrain)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if table
            .guild_max
            .keys()
            .any(|key| core.faction_by_key(key).is_none())
        {
            return Err(EncounterError::Rules("a guild's faction is unknown"));
        }
        let levels: [Vec<CreatureTypeId>; LEVELS] = std::array::from_fn(|index| {
            let level = index as u8 + 1;
            rules
                .adventure()
                .random_monsters(&core, [level, level])
                .collect()
        });
        if levels.iter().any(Vec::is_empty) {
            return Err(EncounterError::Rules("a level has no random monster"));
        }
        let hero = hero::HeroData::new(&core, table)?;
        Ok(Self {
            weights: table.kinds.clone().map(|k| k.weight),
            faction_weights: table.factions.iter().map(|f| f.1).collect(),
            rules,
            battle,
            adventure,
            table: table.clone(),
            places,
            water,
            factions,
            levels,
            hero,
        })
    }

    /// This sampler drawing the kinds by `weights` (in [`Kind::ALL`]'s order; a curriculum).
    ///
    /// # Errors
    /// [`EncounterError::Table`] when the weights are all 0 or exceed `u32::MAX` together.
    pub fn with_weights(mut self, weights: [u32; 6]) -> Result<Self, EncounterError> {
        let total = weights.iter().try_fold(0u32, |sum, &w| sum.checked_add(w));
        if !total.is_some_and(|t| t > 0) {
            return Err(EncounterError::Table {
                field: "weights",
                reason: "all 0 or above u32::MAX",
            });
        }
        self.weights = weights;
        Ok(self)
    }

    /// This sampler drawing the kinds by A1's shares, the neutral kinds included.
    pub fn with_shares(self) -> Self {
        let shares = self.table.kinds.clone().map(|k| k.share);
        Self {
            weights: shares,
            ..self
        }
    }

    /// The kinds' weights it draws by.
    pub fn weights(&self) -> [u32; 6] {
        self.weights
    }

    /// The distribution it samples.
    pub fn table(&self) -> &EncounterTable {
        &self.table
    }

    /// The adventure part its monsters come from.
    pub fn adventure(&self) -> &Arc<AdventureRuleset> {
        &self.adventure
    }

    /// The battle part its battles run on.
    pub fn battle(&self) -> &Arc<BattleRuleset> {
        &self.battle
    }

    /// The core ruleset.
    pub fn core(&self) -> &Ruleset {
        self.rules.core()
    }

    /// The encounter of `seed`: every draw from its stream `ENCOUNTER`.
    pub fn sample(&self, seed: u64) -> Encounter {
        let rng = &mut Rng::for_stream(seed, StreamId::ENCOUNTER);
        let kind = Kind::ALL[pick(rng, self.weights.iter().copied())];
        let row = &self.table.kinds[kind.index()];
        let week = pick(rng, row.week.iter().copied()) as u8 + 1;
        let facts = Facts {
            place: None,
            level: None,
            count: 0,
            redraws: 0,
            ratio: 0,
            k: 0,
            factions: [None, None],
            hero_levels: [None, None],
            values: [0, 0],
            fort: None,
        };
        let setup = BattleSetup {
            difficulty: self.table.difficulty,
            human: [true, true],
            ..BattleSetup::default()
        };
        let mut draft = Draft { setup, facts };
        match kind {
            Kind::Wandering | Kind::Guards => self.neutral(rng, kind, week, &mut draft),
            Kind::Hero => self.field(rng, week, &mut draft),
            Kind::SiegeHero | Kind::SiegeGarrison | Kind::SiegeNeutral => {
                self.siege(rng, kind, week, &mut draft);
            }
        }
        let defender = if kind.neutral() {
            Seat::Internal(Policy::System)
        } else {
            Seat::External
        };
        Encounter {
            kind,
            week,
            setup: draft.setup,
            seats: [Seat::External, defender],
            random_obstacles: true,
            facts: draft.facts,
        }
    }

    /// A faction's index drawn by the census's towns (A7, A8).
    fn faction(&self, rng: &mut Rng) -> usize {
        pick(rng, self.faction_weights.iter().copied())
    }

    /// The summed growth value of `faction`'s levels by `week` (A5).
    fn production(&self, faction: usize, week: u8) -> u64 {
        self.factions[faction].cumulative[usize::from(week) - 1]
    }

    /// Whether `level` (1 to 7) is built by `week` (A6).
    fn built(&self, level: usize, week: u8) -> bool {
        self.table.level_from_week[level - 1] <= week
    }

    /// The creature of a level's stack: upgraded with the week's share (A6).
    fn creature(&self, rng: &mut Rng, faction: usize, level: usize, week: u8) -> CreatureTypeId {
        let (base, upgrade) = self.factions[faction].creatures[level - 1];
        if chance(rng, self.table.upgraded[usize::from(week) - 1]) {
            upgrade
        } else {
            base
        }
    }

    /// An army of `faction` worth about `value` by `week` (A6): one stack a built level, its
    /// value by the level's growth value; what a level's share cannot buy goes to the level below
    /// (a small army has no creature of the high levels), so at most a level-1 creature's worth is
    /// lost; at least one creature of level 1.
    fn army(&self, rng: &mut Rng, faction: usize, week: u8, value: u64) -> Vec<ArmyStack> {
        let data = &self.factions[faction];
        let levels: Vec<usize> = (1..=LEVELS).filter(|&l| self.built(l, week)).collect();
        let built: u64 = levels.iter().map(|&l| data.growth_value[l - 1]).sum();
        let creatures: Vec<CreatureTypeId> = levels
            .iter()
            .map(|&level| self.creature(rng, faction, level, week))
            .collect();
        let mut counts = vec![0; levels.len()];
        let mut carry = 0;
        for (index, &level) in levels.iter().enumerate().rev() {
            let share = value.saturating_mul(data.growth_value[level - 1]) / built.max(1) + carry;
            let price = u64::from(self.ai_value(creatures[index]).max(1));
            counts[index] = share / price;
            carry = share % price;
        }
        let mut stacks = Vec::with_capacity(LEVELS);
        for (creature, count) in creatures.into_iter().zip(counts) {
            if count > 0 {
                stacks.push(ArmyStack {
                    slot: stacks.len() as u8,
                    creature,
                    count: u32::try_from(count).unwrap_or(u32::MAX),
                });
            }
        }
        if stacks.is_empty() {
            stacks.push(ArmyStack {
                slot: 0,
                creature: data.creatures[0].0,
                count: 1,
            });
        }
        stacks
    }

    /// A town's garrison of `faction` by `week`: one week's growth of each built level (A8).
    fn garrison(&self, rng: &mut Rng, faction: usize, week: u8) -> Vec<ArmyStack> {
        let data = &self.factions[faction];
        let mut stacks = Vec::with_capacity(LEVELS);
        for level in (1..=LEVELS).filter(|&level| self.built(level, week)) {
            let creature = self.creature(rng, faction, level, week);
            stacks.push(ArmyStack {
                slot: stacks.len() as u8,
                creature,
                count: data.growth[level - 1].max(1),
            });
        }
        stacks
    }

    /// A creature's AI value.
    fn ai_value(&self, creature: CreatureTypeId) -> u32 {
        self.core().creature(creature).map_or(0, |c| c.ai_value)
    }

    /// An army's value (ai_value × count).
    fn value(&self, stacks: &[ArmyStack]) -> u64 {
        stacks
            .iter()
            .map(|s| u64::from(self.ai_value(s.creature)) * u64::from(s.count))
            .sum()
    }

    /// A hero of `faction` by `week` with its army worth `value`, placed on `side` of `setup`.
    fn hero_side(
        &self,
        rng: &mut Rng,
        faction: usize,
        week: u8,
        value: u64,
        side: usize,
        draft: &mut Draft,
    ) -> Vec<ArmyStack> {
        let details = self.hero.details(self, rng, faction, week);
        draft.facts.hero_levels[side] = Some(details.level);
        draft.facts.factions[side] = Some(self.factions[faction].faction);
        let (hero, machines) = self.hero.battle(self.core(), &details);
        draft.setup.heroes[side] = Some(hero);
        draft.setup.war_machines[side] = machines;
        let stacks = self.army(rng, faction, week, value);
        draft.facts.values[side] = self.value(&stacks);
        stacks
    }

    /// A neutral stack and the hero attacking it (A3, A4, A10).
    fn neutral(&self, rng: &mut Rng, kind: Kind, week: u8, draft: &mut Draft) {
        let place = pick(rng, self.table.places.iter().map(|p| p.monsters));
        draft.facts.place = Some(place);
        let faction = self.faction(rng);
        let ratio = draw(rng, &self.table.ratio);
        let cap = scale(self.production(faction, week), self.table.cap);
        let w = usize::from(week) - 1;
        let (mut creature, mut count, mut level) = (self.levels[0][0], 1, 1);
        for redraw in 0..MAX_REDRAWS {
            level = pick(
                rng,
                (0..LEVELS)
                    .map(|l| self.table.level_weight[l] * self.table.level_ramp[l][w] / 1000),
            ) + 1;
            let candidates = &self.levels[level - 1];
            creature = candidates[rng.below(candidates.len() as u32) as usize];
            // The count from the creature's adventure-map range.
            // rule: ADV-MON-004
            let [low, high] = self
                .core()
                .creature(creature)
                .map_or([1, 1], |c| c.adventure_count);
            count = rng.range_inclusive(u32::from(low), u32::from(high)).max(1);
            if kind == Kind::Wandering {
                for _ in 1..week {
                    if count < self.table.growth_cap {
                        count += count * self.table.growth_percent / 100;
                    }
                }
            }
            draft.facts.redraws = redraw;
            let army = scale(u64::from(self.ai_value(creature)) * u64::from(count), ratio);
            if army <= cap {
                break;
            }
        }
        draft.facts.level = Some(level as u8);
        draft.facts.count = count;
        draft.facts.ratio = ratio;
        let stack_value = u64::from(self.ai_value(creature)) * u64::from(count);
        draft.setup.attacker.stacks =
            self.hero_side(rng, faction, week, scale(stack_value, ratio), 0, draft);
        draft.setup.defender.stacks = if kind == Kind::Wandering {
            // rule: ADV-MON-009
            split(
                self.core(),
                &draft.setup.attacker.stacks,
                (creature, count),
                || rng.range_inclusive(1, 100),
            )
        } else {
            draft.setup.anti_magic = chance(rng, self.table.anti_magic);
            divide(creature, count, self.table.guard_stacks)
        };
        draft.facts.values[1] = stack_value;
        let at = &self.places[place];
        draft.setup.terrain = Some(at.terrain);
        draft.setup.battlefield = if self.table.places[place].open_land == 0 {
            Some(self.water.clone())
        } else {
            at.battlefield.clone()
        };
    }

    /// A hero against a hero on open land (A5, A7, A10).
    fn field(&self, rng: &mut Rng, week: u8, draft: &mut Draft) {
        let place = pick(rng, self.table.places.iter().map(|p| p.open_land));
        draft.facts.place = Some(place);
        let at = &self.places[place];
        draft.setup.terrain = Some(at.terrain);
        draft.setup.battlefield = at.battlefield.clone();
        let (attacker, defender) = (self.faction(rng), self.faction(rng));
        let k = draw(rng, &self.table.k);
        let ratio = draw(rng, &self.table.attacker_ratio);
        draft.facts.k = k;
        draft.facts.ratio = ratio;
        let value = scale(self.production(defender, week), k);
        draft.setup.defender.stacks = self.hero_side(rng, defender, week, value, 1, draft);
        draft.setup.attacker.stacks =
            self.hero_side(rng, attacker, week, scale(value, ratio), 0, draft);
    }

    /// A hero attacking a town (A5, A7, A8).
    fn siege(&self, rng: &mut Rng, kind: Kind, week: u8, draft: &mut Draft) {
        let town = self.faction(rng);
        let data = &self.factions[town];
        let fort = if kind == Kind::SiegeNeutral || week == 1 {
            pick(rng, self.table.census_forts.iter().copied())
        } else {
            let row = (usize::from(week) - 2).min(self.table.forts.len() - 1);
            pick(rng, self.table.forts[row].iter().copied())
        };
        let fort = [
            FortLevel::None,
            FortLevel::Fort,
            FortLevel::Citadel,
            FortLevel::Castle,
        ][fort];
        draft.facts.fort = Some(fort);
        let [first, per_week, max, neutral] = self.table.structures;
        let structures = if kind == Kind::SiegeNeutral {
            neutral
        } else {
            first
                .saturating_add(per_week.saturating_mul(week - 1))
                .min(max)
        };
        draft.setup.siege = Some(Siege {
            faction: data.faction,
            fort,
            moat: if fort >= FortLevel::Citadel {
                Moat::Damaging
            } else {
                Moat::Absent
            },
            structures,
            escape_tunnel: false,
            morale: if kind == Kind::SiegeNeutral {
                0
            } else {
                self.table.town_morale
            },
            luck: 0,
        });
        draft.setup.terrain = Some(data.terrain);
        let attacker = self.faction(rng);
        let k = draw(rng, &self.table.k);
        draft.facts.k = k;
        if kind == Kind::SiegeHero {
            let ratio = draw(rng, &self.table.attacker_ratio);
            draft.facts.ratio = ratio;
            let value = scale(self.production(town, week), k);
            draft.setup.defender.stacks = self.hero_side(rng, town, week, value, 1, draft);
            draft.setup.attacker.stacks =
                self.hero_side(rng, attacker, week, scale(value, ratio), 0, draft);
        } else {
            draft.setup.defender.stacks = self.garrison(rng, town, week);
            draft.facts.factions[1] = Some(data.faction);
            draft.facts.values[1] = self.value(&draft.setup.defender.stacks);
            let value = scale(self.production(attacker, week), k);
            draft.setup.attacker.stacks = self.hero_side(rng, attacker, week, value, 0, draft);
        }
    }
}

/// `count` creatures divided evenly into at most `stacks` stacks, the first ones one more.
fn divide(creature: CreatureTypeId, count: u32, stacks: u32) -> Vec<ArmyStack> {
    let n = stacks.min(count).max(1);
    (0..n)
        .map(|slot| ArmyStack {
            slot: slot as u8,
            creature,
            count: count / n + u32::from(slot < count % n),
        })
        .collect()
}

/// A faction's army data (A5, A6) and its heroes (A7): per level its town creature (the first of
/// the faction's creatures of that level that upgrades none) and that creature's first upgrade.
fn faction_army(
    core: &Ruleset,
    table: &EncounterTable,
    faction: FactionId,
    guild_max: Option<u8>,
    town_terrain: TerrainId,
) -> Result<FactionArmy, EncounterError> {
    let upgrades: Vec<CreatureTypeId> = (0..core.creature_count())
        .filter_map(CreatureTypeId::from_index)
        .flat_map(|id| core.upgrades(id).iter().copied())
        .collect();
    let mut creatures = Vec::with_capacity(LEVELS);
    let mut growth = [0; LEVELS];
    let mut growth_value = [0; LEVELS];
    for level in 1..=LEVELS as u8 {
        let base = (0..core.creature_count())
            .filter_map(CreatureTypeId::from_index)
            .find(|&id| {
                core.creature(id).is_some_and(|c| {
                    c.faction == Some(faction)
                        && c.level == level
                        && !c.flags.contains(CreatureFlag::WarMachine)
                }) && !upgrades.contains(&id)
            })
            .ok_or(EncounterError::Rules("a faction lacks a level"))?;
        let info = core
            .creature(base)
            .ok_or(EncounterError::Rules("a creature is missing"))?;
        let index = usize::from(level) - 1;
        growth[index] = u32::from(info.growth);
        growth_value[index] = u64::from(info.growth) * u64::from(info.ai_value);
        creatures.push((base, core.upgrades(base).first().copied().unwrap_or(base)));
    }
    let mut cumulative = [0u64; WEEKS];
    let mut sum = 0;
    for (week, total) in cumulative.iter_mut().enumerate() {
        sum += (0..LEVELS)
            .filter(|&l| usize::from(table.level_from_week[l]) <= week + 1)
            .map(|l| growth_value[l])
            .sum::<u64>();
        *total = sum;
    }
    let classes = core
        .hero_classes()
        .iter()
        .enumerate()
        .filter(|(_, class)| class.faction == faction)
        .filter_map(|(index, _)| HeroClassId::from_index(index))
        .map(|class| {
            let types = core
                .hero_types()
                .iter()
                .enumerate()
                .filter(|(_, hero)| hero.class == class)
                .filter_map(|(index, _)| HeroTypeId::from_index(index))
                .collect::<Vec<_>>();
            (class, types)
        })
        .filter(|(_, types)| !types.is_empty())
        .collect::<Vec<_>>();
    if classes.is_empty() {
        return Err(EncounterError::Rules("a faction has no hero"));
    }
    Ok(FactionArmy {
        faction,
        creatures: creatures
            .try_into()
            .map_err(|_| EncounterError::Rules("a faction lacks a level"))?,
        growth,
        growth_value,
        cumulative,
        classes,
        guild_max: guild_max.unwrap_or(table.guild_max_default),
        terrain: core
            .factions()
            .get(faction.index())
            .and_then(|f| f.native_terrain)
            .unwrap_or(town_terrain),
    })
}
