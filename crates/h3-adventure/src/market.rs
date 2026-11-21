//! The market's rates (ADV-MKT-002, E08 T132a1; increment 14's design note §2, survey
//! `docs/rules/surveys/E08-trades.md` §4): what a Trading Post, a Freelancer's Guild and a Black
//! Market trade at, and the town's Marketplace with its own count (E08 decisions 361, 370).
//!
//! Every step is the original's x87 arithmetic (`market_resources_proc` 0x5ECFE0,
//! `market_creature_rate` 0x5ECE80, `market_buy_artifact_proc` 0x5ED9E0), computed exactly in
//! integers with [`X87`] (ADR-0003: no floats): an integer or a product stored to binary32, a
//! quotient at the FPU's 53 bits stored to binary32, and the rounding `trunc(x + 0.5)` at 53 bits
//! (the runtime's `_ftol` after an `fadd` of 0.5). A rate is a [`Lot`]: so many given for one
//! wanted, or one given for so many. The original's rounding is the rule (D-71, E08 decision
//! 365), not a quirk.

use alloc::vec::Vec;

use h3_content::ResourceId;
use h3_kernel::arith::{SINGLE_BITS, SINGLE_FRAC_BITS, X87};
use h3_kernel::{CanonicalHash, StateHasher};

/// A rate as the original states it (ADV-MKT-002): `give` of the given for `take` of the wanted,
/// one of the two 1, both at least 1. A trade is a whole number of lots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lot {
    /// How many of the given one lot takes.
    pub give: u32,
    /// How many of the wanted one lot gives.
    pub take: u32,
}

/// Why [`MarketParams::new`] refused its tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketParamsError {
    /// The two value tables are empty or differ in length (one value per resource each).
    Values,
    /// A value is 0: a rate would divide by it.
    ZeroValue,
    /// No row, or more than `u8::MAX` (a count is a `u8`).
    Rows,
    /// A factor is 0.
    ZeroFactor,
    /// The cost resource is not one the value tables value (E08 T133b, ADV-Q60).
    Cost,
}

/// The market's parameters (ADV-MKT-002; the adventure part's `[market]`, Tier 2): a value per
/// resource for the trades and the sale (the original's `resource_values` 0x68C482: 250, 500,
/// 250, 500, 500, 500, 1) and one for an artifact's price (`resource_values_artifacts` 0x68C492:
/// 125, 250, 125, 250, 250, 250, 1), both in the core's resource order; and three factors per
/// count of Marketplaces from 1, in hundredths (`market_resource_rates` 0x678344,
/// `market_artifact_rates` 0x678370, `market_creature_rates` 0x67839C), each used as the binary32
/// nearest it, as the executable holds them. A creature's and an artifact's value is their cost in
/// the parameters' cost resource (the original's gold: the creature record's +0x38, the
/// artifact's cost; E08 T133b, ADV-Q60 (a)), which [`Self::cost`] names and the caller reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketParams {
    cost: ResourceId,
    values: Vec<u16>,
    artifact_values: Vec<u16>,
    rows: Vec<Row>,
}

/// One count's three factors, each the binary32 nearest its hundredths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    resources: X87,
    artifacts: X87,
    creatures: X87,
}

/// The scale of a stored quotient: `f32(v / a) · 2^32` is `f32(v · 2^32 / a)` exactly (the two
/// roundings do not depend on a power of two), and the scaled value keeps every bit [`X87`]
/// holds for any quotient of a `u16` value by a product of a `u32` cost and a factor of at most
/// 655.35, so a small quotient (a dear creature sold for gold) loses nothing.
const SCALE: u32 = 32;

impl MarketParams {
    /// The parameters: the cost resource (a creature's and an artifact's value is their cost in
    /// it), the two value tables by resource and, by count from 1, the resources', the
    /// artifacts' and the creatures' factors in hundredths.
    pub fn new(
        cost: ResourceId,
        values: Vec<u16>,
        artifact_values: Vec<u16>,
        rows: &[[u16; 3]],
    ) -> Result<Self, MarketParamsError> {
        if values.is_empty() || values.len() != artifact_values.len() {
            return Err(MarketParamsError::Values);
        }
        if cost.index() >= values.len() {
            return Err(MarketParamsError::Cost);
        }
        if values.iter().chain(&artifact_values).any(|&v| v == 0) {
            return Err(MarketParamsError::ZeroValue);
        }
        if rows.is_empty() || rows.len() > usize::from(u8::MAX) {
            return Err(MarketParamsError::Rows);
        }
        let factor = |hundredths: u16| match hundredths {
            0 => Err(MarketParamsError::ZeroFactor),
            p => Ok(single_ratio(p)),
        };
        let rows = rows
            .iter()
            .map(|&[r, a, c]| {
                Ok(Row {
                    resources: factor(r)?,
                    artifacts: factor(a)?,
                    creatures: factor(c)?,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            cost,
            values,
            artifact_values,
            rows,
        })
    }

    /// The resource a creature's and an artifact's cost is read in for the market (E08 T133b,
    /// ADV-Q60 (a): the base pack's gold, the original's creature record's +0x38 and artifact
    /// cost).
    pub const fn cost(&self) -> ResourceId {
        self.cost
    }

    /// The counts of Marketplaces the parameters know: 1 to this.
    pub fn counts(&self) -> u8 {
        // At most `u8::MAX` rows (`new`).
        u8::try_from(self.rows.len()).unwrap_or(u8::MAX)
    }

    /// The value of `resource` in a trade and a creature sale, if the parameters value it.
    pub fn value(&self, resource: ResourceId) -> Option<u16> {
        self.values.get(resource.index()).copied()
    }

    /// The value of `resource` in an artifact's price, if the parameters value it.
    pub fn artifact_value(&self, resource: ResourceId) -> Option<u16> {
        self.artifact_values.get(resource.index()).copied()
    }

    /// The binary32 bit patterns of the resources', the artifacts' and the creatures' factors at
    /// `count` Marketplaces (the original-data check compares them with the executable's
    /// tables); none outside the counts.
    pub fn factor_bits(&self, count: u8) -> Option<[u32; 3]> {
        let row = self.row(count)?;
        Some([
            row.resources.to_f32_bits(),
            row.artifacts.to_f32_bits(),
            row.creatures.to_f32_bits(),
        ])
    }

    /// The lot of a trade of `give` for `take` at `count` Marketplaces (screen 0,
    /// `market_resources_proc` 0x5ECFE0): a = f32(v(give) × R), q = f32(v(take) / a); q ≥ 1
    /// gives round(q) of `give` for one `take`, else one `give` for round(1 / q) of `take`. None
    /// for a resource for itself, a count outside the parameters, a resource they do not value,
    /// or a lot past `u32`.
    // rule: ADV-MKT-002
    pub fn resource_lot(&self, count: u8, give: ResourceId, take: ResourceId) -> Option<Lot> {
        if give == take {
            return None;
        }
        let row = self.row(count)?;
        let (given, wanted) = (value(&self.values, give)?, value(&self.values, take)?);
        lot(
            stored(given).times(row.resources).round(SINGLE_BITS),
            wanted,
        )
    }

    /// The lot of a creature of cost `cost` (in [`Self::cost`]'s resource) sold for `take` at `count` Marketplaces (screen 4,
    /// `market_creature_rate` 0x5ECE80): a = f32(f32(cost) × C), q = f32(v(take) / a); q ≥ 1
    /// gives round(q) creatures for one `take`, else one creature for round(1 / q) of `take`. None
    /// for a creature of no cost, a count outside the parameters, a resource they do not
    /// value, or a lot past `u32`.
    // rule: ADV-MKT-002
    pub fn creature_lot(&self, count: u8, cost: u32, take: ResourceId) -> Option<Lot> {
        let row = self.row(count)?;
        let wanted = value(&self.values, take)?;
        lot(stored(cost).times(row.creatures).round(SINGLE_BITS), wanted)
    }

    /// The price in `pay` of an artifact of cost `cost` (in [`Self::cost`]'s resource) at `count` Marketplaces (screen 2,
    /// `market_buy_artifact_proc` 0x5ED9E0): a = f32(f32(w(pay)) × A), round(f32(f32(cost) /
    /// a)). None for an artifact of no cost, a price that rounds to 0 (no price, as the
    /// original's 0), a count outside the parameters, a resource they do not value, or a price
    /// past `u32`.
    // rule: ADV-MKT-002
    pub fn artifact_price(&self, count: u8, cost: u32, pay: ResourceId) -> Option<u32> {
        let row = self.row(count)?;
        let a = stored(value(&self.artifact_values, pay)?)
            .times(row.artifacts)
            .round(SINGLE_BITS);
        // `a` is above 0 (a value from 1 times a factor from 0.01).
        let quotient = stored(cost).over(a).unwrap_or(X87::ZERO).round(SINGLE_BITS);
        match u32::try_from(half_up(quotient)) {
            Ok(0) | Err(_) => None,
            Ok(price) => Some(price),
        }
    }

    /// The row of `count` Marketplaces, from 1.
    fn row(&self, count: u8) -> Option<Row> {
        self.rows.get(usize::from(count).checked_sub(1)?).copied()
    }
}

impl CanonicalHash for MarketParams {
    /// The cost resource (v24, E08 T133b), the two value tables as length and entries, then the
    /// rows as their number and each one's three factors as binary32 bit patterns (E08 T132a2:
    /// in the adventure part's hash, v23).
    fn hash_into(&self, h: &mut StateHasher) {
        self.cost.hash_into(h);
        for table in [&self.values, &self.artifact_values] {
            h.write_len(table.len());
            table.iter().for_each(|&v| h.write_u16(v));
        }
        h.write_len(self.rows.len());
        for row in &self.rows {
            for factor in [row.resources, row.artifacts, row.creatures] {
                h.write_u32(factor.to_f32_bits());
            }
        }
    }
}

/// A resource's value in `table`.
fn value(table: &[u16], resource: ResourceId) -> Option<u32> {
    table.get(resource.index()).map(|&v| u32::from(v))
}

/// The binary32 nearest `hundredths / 100` (one rounding, as the executable's constants).
fn single_ratio(hundredths: u16) -> X87 {
    // The denominator is not 0.
    X87::ratio(u32::from(hundredths), 100, SINGLE_BITS).unwrap_or(X87::ZERO)
}

/// An integer as the original keeps it: `fild`, then a store to binary32.
fn stored(n: u32) -> X87 {
    X87::int(u64::from(n)).round(SINGLE_BITS)
}

/// `trunc(x + 0.5)`, the addition at the FPU's 53 bits (`fadd` of 0.5, then `_ftol`).
fn half_up(x: X87) -> u64 {
    x.plus(HALF).trunc()
}

/// One half, exact.
const HALF: X87 = X87::fixed(1 << (SINGLE_FRAC_BITS - 1));

/// `2^-SCALE`, exact.
const UNSCALE: X87 = X87::fixed(1 << (SINGLE_FRAC_BITS - SCALE));

/// The lot for a given whose value times its factor is `a` (stored), against one wanted of value
/// `wanted`: q = f32(wanted / a), held scaled by `2^SCALE`; none when `a` is 0 (a creature of no
/// gold cost) or the lot passes `u32`.
fn lot(a: X87, wanted: u32) -> Option<Lot> {
    let unit = X87::int(1 << SCALE);
    let scaled = X87::int(u64::from(wanted) << SCALE)
        .over(a)?
        .round(SINGLE_BITS);
    let (give, take) = if scaled >= unit {
        // q itself, exact (a binary32 of at least 1 times a power of two).
        (half_up(scaled.times(UNSCALE)), 1)
    } else {
        // 1 / q at 53 bits: the same quotient as `2^SCALE` over the scaled q, which is above 0
        // (at least 2^-10 for any value, cost and factor the parameters take).
        (1, half_up(unit.over(scaled).unwrap_or(X87::MAX)))
    };
    match (u32::try_from(give), u32::try_from(take)) {
        (Ok(give), Ok(take)) => Some(Lot { give, take }),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
