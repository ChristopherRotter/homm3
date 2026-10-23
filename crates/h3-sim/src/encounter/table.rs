//! The encounter distribution's files as checked numbers (E05 T5b1): `encounters-v1.toml` (the
//! stated assumptions, design §4.4) and the census it names (`census-v1.toml`, derived from the
//! shipped maps, §4.3). Content keys stay strings here; [`super::Sampler::new`] resolves them
//! against a ruleset.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::{ENCOUNTERS_VERSION, EncounterError, Kind};

/// The game weeks the distribution covers (1 to `WEEKS`).
pub const WEEKS: usize = 16;

/// The creature levels of a town (1 to `LEVELS`).
pub const LEVELS: usize = 7;

/// The points of a quantile table: the probabilities 0.001, 0.05, 0.10, …, 0.95, 0.999.
pub const QUANTILES: usize = 21;

/// A distribution as a quantile table (permille values, non-decreasing): a draw takes one of the
/// 20 bins evenly and a point inside it evenly, and interpolates linearly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantiles(pub [u32; QUANTILES]);

impl Quantiles {
    /// The value of the point `fraction` / 1000 of bin `bin` (0 to 19).
    pub fn at(&self, bin: usize, fraction: u32) -> u32 {
        let (low, high) = (self.0[bin], self.0[bin + 1]);
        low + (high - low) * fraction / 1000
    }

    /// The bin a value lies in: the last whose lower point is at most `value` (0 to 19).
    pub fn bin(&self, value: u32) -> usize {
        self.0[1..QUANTILES - 1]
            .iter()
            .take_while(|&&point| point <= value)
            .count()
    }
}

/// A kind of encounter's row (A1, A2, A12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindRow {
    /// The estimated share of the battles fought (‰).
    pub share: u32,
    /// What the sampler draws by default (‰; the neutral kinds 0, E06 T7).
    pub weight: u32,
    /// The weight of each game week.
    pub week: [u32; WEEKS],
}

/// One battle place of the census: a terrain and a special battlefield, with how many monsters
/// stand there and how much open land it has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The terrain's key.
    pub terrain: String,
    /// The special battlefield's key, if any.
    pub battlefield: Option<String>,
    /// Open land tiles: the weight of hero battles.
    pub open_land: u32,
    /// Monster placements: the weight of neutral stacks and guards.
    pub monsters: u32,
}

/// The distribution's numbers, checked: every table has its length, every quantile table is
/// non-decreasing, every total fits a draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterTable {
    /// The kinds, in [`Kind::ALL`]'s order.
    pub kinds: [KindRow; 6],
    /// A3: the census's monster placements per level (fixed and random, those of any level spread
    /// evenly), times [`LEVELS`] so that the spread stays whole.
    pub level_weight: [u32; LEVELS],
    /// A3: each level's ramp by week (‰).
    pub level_ramp: [[u32; WEEKS]; LEVELS],
    /// A3: a neutral stack's weekly growth (%) and the count from which it stops.
    pub growth_percent: u32,
    /// A3: see `growth_percent`.
    pub growth_cap: u32,
    /// A4: the army facing a neutral over the stack's value (‰).
    pub ratio: Quantiles,
    /// A4: the most that army may be worth, in ‰ of A5's value of the week (median `k`).
    pub cap: u32,
    /// A10: the stacks a guards' army is divided into.
    pub guard_stacks: u32,
    /// A10: the share of guards at an anti-magic garrison (‰).
    pub anti_magic: u32,
    /// A6: the first week of each level.
    pub level_from_week: [u8; LEVELS],
    /// A5: the army's value over the summed growth value (‰).
    pub k: Quantiles,
    /// A5: the attacker's value over the defender's (‰).
    pub attacker_ratio: Quantiles,
    /// A6: the share of upgraded stacks by week (‰).
    pub upgraded: [u32; WEEKS],
    /// A7: a hero's level by week.
    pub hero_level: [u8; WEEKS],
    /// A7: the level's spread, ± this.
    pub level_spread: u8,
    /// A7: the mage guild's level by week.
    pub guild: [u8; WEEKS],
    /// A7: the guild's level at most, by faction key; `guild_max_default` for the others.
    pub guild_max: BTreeMap<String, u8>,
    /// A7: see `guild_max`.
    pub guild_max_default: u8,
    /// A7: the spells a guild offers per spell level (levels 1 to 5).
    pub spells_per_level: [u8; 5],
    /// A7: one artifact per this many levels.
    pub levels_per_artifact: u8,
    /// A7: the hero level from which each artifact class is drawn (treasure, minor, major,
    /// relic).
    pub artifact_from_level: [u8; 4],
    /// A7: the war machines' artifact keys.
    pub war_machines: Vec<String>,
    /// A7: each war machine's chance (‰).
    pub war_machine: u32,
    /// A7: the spell book's artifact key.
    pub spell_book: String,
    /// A8: the census's starting forts of its towns (none, fort, citadel, castle).
    pub census_forts: [u32; 4],
    /// A8: the forts in weeks 2 to 7 (‰; the last row for every later week).
    pub forts: Vec<[u32; 4]>,
    /// A8: the turrets' structures: first week, added each week, at most, a neutral town's.
    pub structures: [u8; 4],
    /// A8: an owned town's morale.
    pub town_morale: i8,
    /// A8: a town's terrain when its faction has no native one.
    pub town_terrain: String,
    /// A7, A8: the factions' keys and their weights (the census's named towns times 9, plus its
    /// random towns spread evenly: whole numbers).
    pub factions: Vec<(String, u32)>,
    /// A10: the census's battle places.
    pub places: Vec<Place>,
    /// A10: the battlefield of a battle on water.
    pub water_battlefield: String,
    /// A11: the game's difficulty.
    pub difficulty: u8,
}

/// `encounters-v1.toml` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EncountersFile {
    distribution: DistributionDef,
    kind: BTreeMap<String, KindDef>,
    neutral: NeutralDef,
    guards: GuardsDef,
    army: ArmyDef,
    hero: HeroDef,
    siege: SiegeDef,
    factions: FactionsDef,
    places: PlacesDef,
    difficulty: DifficultyDef,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DistributionDef {
    id: String,
    version: u32,
    census: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KindDef {
    share: u32,
    weight: u32,
    week: Vec<u32>,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NeutralDef {
    growth_percent: u32,
    growth_cap: u32,
    level_ramp: Vec<Vec<u32>>,
    ratio: Vec<u32>,
    cap: u32,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuardsDef {
    stacks: u32,
    anti_magic: u32,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArmyDef {
    level_from_week: Vec<u8>,
    k: Vec<u32>,
    attacker_ratio: Vec<u32>,
    upgraded: Vec<u32>,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroDef {
    level: Vec<u8>,
    level_spread: u8,
    guild: Vec<u8>,
    guild_max: BTreeMap<String, u8>,
    guild_max_default: u8,
    spells_per_level: Vec<u8>,
    levels_per_artifact: u8,
    artifact_from_level: ArtifactLevels,
    war_machines: Vec<String>,
    war_machine: u32,
    spell_book: String,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactLevels {
    treasure: u8,
    minor: u8,
    major: u8,
    relic: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiegeDef {
    fort: Vec<Vec<u32>>,
    structures: StructuresDef,
    morale: i8,
    terrain: String,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuresDef {
    first: u8,
    per_week: u8,
    max: u8,
    neutral: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FactionsDef {
    keys: Vec<String>,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacesDef {
    water_battlefield: String,
    #[allow(dead_code)]
    why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DifficultyDef {
    value: u8,
    #[allow(dead_code)]
    why: String,
}

/// The parts of the census the distribution reads (the census has more).
#[derive(Deserialize)]
struct CensusFile {
    census: CensusHeader,
    monsters: CensusMonsters,
    towns: BTreeMap<String, CensusForts>,
    place: Vec<CensusPlace>,
}

#[derive(Deserialize)]
struct CensusHeader {
    id: String,
    version: u32,
}

#[derive(Deserialize)]
struct CensusMonsters {
    level: BTreeMap<String, CensusLevel>,
}

#[derive(Deserialize)]
struct CensusLevel {
    fixed: u32,
    random: u32,
}

#[derive(Deserialize)]
struct CensusForts {
    none: u32,
    fort: u32,
    citadel: u32,
    castle: u32,
}

#[derive(Deserialize)]
struct CensusPlace {
    terrain: String,
    battlefield: Option<String>,
    open_land: u32,
    monsters: u32,
}

/// A table error at `field`.
fn bad(field: &'static str, reason: &'static str) -> EncounterError {
    EncounterError::Table { field, reason }
}

/// `values` as an array of `N`.
fn exact<T: Copy + Default, const N: usize>(
    values: &[T],
    field: &'static str,
) -> Result<[T; N], EncounterError> {
    <[T; N]>::try_from(values).map_err(|_| bad(field, "wrong length"))
}

/// A quantile table: 21 non-decreasing values.
fn quantiles(values: &[u32], field: &'static str) -> Result<Quantiles, EncounterError> {
    let points: [u32; QUANTILES] = exact(values, field)?;
    if points.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(bad(field, "not non-decreasing"));
    }
    Ok(Quantiles(points))
}

/// Weights a draw can take: their sum is positive and fits a `u32`.
fn drawable(
    weights: impl IntoIterator<Item = u32>,
    field: &'static str,
) -> Result<(), EncounterError> {
    let total = weights
        .into_iter()
        .try_fold(0u32, u32::checked_add)
        .ok_or(bad(field, "the weights exceed u32::MAX"))?;
    if total == 0 {
        return Err(bad(field, "the weights are all 0"));
    }
    Ok(())
}

impl EncounterTable {
    /// The distribution of `encounters` (the text of an `encounters-v1.toml`) on the census
    /// `census` (the text of the census it names), checked.
    ///
    /// # Errors
    /// [`EncounterError::Toml`] for a text that does not parse into its shape,
    /// [`EncounterError::Census`] when the census is not the one the distribution names, or
    /// [`EncounterError::Table`] for a number out of shape.
    pub fn parse(encounters: &str, census: &str) -> Result<Self, EncounterError> {
        let file: EncountersFile =
            toml::from_str(encounters).map_err(|e| EncounterError::Toml(e.message().to_owned()))?;
        let counts: CensusFile =
            toml::from_str(census).map_err(|e| EncounterError::Toml(e.message().to_owned()))?;
        if file.distribution.id != "encounters" || file.distribution.version != ENCOUNTERS_VERSION {
            return Err(bad("distribution", "not encounters v1"));
        }
        let named = format!("{}-v{}", counts.census.id, counts.census.version);
        if file.distribution.census != named {
            return Err(EncounterError::Census(named));
        }

        let mut kinds = Vec::with_capacity(Kind::ALL.len());
        for kind in Kind::ALL {
            let def = file
                .kind
                .get(kind.key())
                .ok_or(bad("kind", "a kind is missing"))?;
            let week = exact(&def.week, "kind.week")?;
            drawable(week, "kind.week")?;
            kinds.push(KindRow {
                share: def.share,
                weight: def.weight,
                week,
            });
        }
        if file.kind.len() != Kind::ALL.len() {
            return Err(bad("kind", "an unknown kind"));
        }
        let kinds: [KindRow; 6] = kinds.try_into().map_err(|_| bad("kind", "wrong length"))?;
        drawable(kinds.iter().map(|k| k.share), "kind.share")?;
        drawable(kinds.iter().map(|k| k.weight), "kind.weight")?;

        let level = |name: &str| {
            counts
                .monsters
                .level
                .get(name)
                .map(|l| l.fixed + l.random)
                .ok_or(bad("census", "a monster level is missing"))
        };
        let any = level("any")?;
        let mut level_weight = [0; LEVELS];
        for (index, weight) in level_weight.iter_mut().enumerate() {
            *weight = level(&(index + 1).to_string())? * LEVELS as u32 + any;
        }
        let ramps = file
            .neutral
            .level_ramp
            .iter()
            .map(|row| exact(row, "neutral.level_ramp"))
            .collect::<Result<Vec<[u32; WEEKS]>, _>>()?;
        let level_ramp: [[u32; WEEKS]; LEVELS] = exact(&ramps, "neutral.level_ramp")?;
        for week in 0..WEEKS {
            let weights = level_weight.iter().zip(&level_ramp);
            drawable(
                weights.map(|(weight, ramp)| weight * ramp[week] / 1000),
                "neutral.level_ramp",
            )?;
        }

        let forts = file
            .siege
            .fort
            .iter()
            .map(|row| exact(row, "siege.fort"))
            .collect::<Result<Vec<[u32; 4]>, _>>()?;
        if forts.is_empty() {
            return Err(bad("siege.fort", "no row"));
        }
        for row in &forts {
            drawable(*row, "siege.fort")?;
        }
        let mut census_forts = [0u32; 4];
        for town in counts.towns.values() {
            for (sum, n) in
                census_forts
                    .iter_mut()
                    .zip([town.none, town.fort, town.citadel, town.castle])
            {
                *sum += n;
            }
        }
        drawable(census_forts, "census.towns")?;

        let factions = file
            .factions
            .keys
            .iter()
            .map(|key| {
                let named = counts
                    .towns
                    .get(key)
                    .map_or(0, |t| t.none + t.fort + t.citadel + t.castle);
                let random = counts
                    .towns
                    .get("random")
                    .map_or(0, |t| t.none + t.fort + t.citadel + t.castle);
                (
                    key.clone(),
                    named * file.factions.keys.len() as u32 + random,
                )
            })
            .collect::<Vec<_>>();
        drawable(factions.iter().map(|f| f.1), "factions")?;

        let places = counts
            .place
            .into_iter()
            .map(|p| Place {
                terrain: p.terrain,
                battlefield: p.battlefield,
                open_land: p.open_land,
                monsters: p.monsters,
            })
            .collect::<Vec<_>>();
        drawable(places.iter().map(|p| p.open_land), "census.place")?;
        drawable(places.iter().map(|p| p.monsters), "census.place")?;

        let army = &file.army;
        let hero = &file.hero;
        let level_from_week: [u8; LEVELS] = exact(&army.level_from_week, "army.level_from_week")?;
        if level_from_week[0] != 1 || level_from_week.contains(&0) {
            return Err(bad("army.level_from_week", "level 1 must come in week 1"));
        }
        let spells_per_level = exact(&hero.spells_per_level, "hero.spells_per_level")?;
        if hero.levels_per_artifact == 0 {
            return Err(bad("hero.levels_per_artifact", "must be positive"));
        }
        if file.guards.stacks == 0 || file.guards.stacks > 7 {
            return Err(bad("guards.stacks", "must be 1 to 7"));
        }
        let levels = &hero.artifact_from_level;
        Ok(Self {
            kinds,
            level_weight,
            level_ramp,
            growth_percent: file.neutral.growth_percent,
            growth_cap: file.neutral.growth_cap,
            ratio: quantiles(&file.neutral.ratio, "neutral.ratio")?,
            cap: file.neutral.cap,
            guard_stacks: file.guards.stacks,
            anti_magic: file.guards.anti_magic,
            level_from_week,
            k: quantiles(&army.k, "army.k")?,
            attacker_ratio: quantiles(&army.attacker_ratio, "army.attacker_ratio")?,
            upgraded: exact(&army.upgraded, "army.upgraded")?,
            hero_level: exact(&hero.level, "hero.level")?,
            level_spread: hero.level_spread,
            guild: exact(&hero.guild, "hero.guild")?,
            guild_max: hero.guild_max.clone(),
            guild_max_default: hero.guild_max_default,
            spells_per_level,
            levels_per_artifact: hero.levels_per_artifact,
            artifact_from_level: [levels.treasure, levels.minor, levels.major, levels.relic],
            war_machines: hero.war_machines.clone(),
            war_machine: hero.war_machine,
            spell_book: hero.spell_book.clone(),
            census_forts,
            forts,
            structures: [
                file.siege.structures.first,
                file.siege.structures.per_week,
                file.siege.structures.max,
                file.siege.structures.neutral,
            ],
            town_morale: file.siege.morale,
            town_terrain: file.siege.terrain.clone(),
            factions,
            places,
            water_battlefield: file.places.water_battlefield.clone(),
            difficulty: file.difficulty.value,
        })
    }

    /// The shipped distribution: `encounters-v1.toml` on `census-v1.toml`.
    ///
    /// # Panics
    /// Never: a test parses both files.
    pub fn v1() -> Self {
        Self::parse(super::ENCOUNTERS_V1, super::CENSUS_V1).expect("the shipped files parse")
    }
}
