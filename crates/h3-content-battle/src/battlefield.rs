//! Special battlefields (E04c; the battlefield ADR draft, `docs/lanes/battle/adr-drafts/battlefields.md`):
//! the shore, the magic fields, a ship's deck, two boats. A battle names one through
//! `BattleSetup::battlefield` by key; each field below is read by one battle rule
//! (BAT-TER-002 to BAT-TER-006, BAT-GEO-004). The original keeps the battlefield as a number
//! (`docs/formats/installation.md` §7.26); the packs give each its key and its values.

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{CompileError, ContentKey, FactionId, SchoolId};
use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::params::BattleParams;

/// Largest morale or luck change a battlefield gives (the original's are 1 and 2).
pub const MAX_BATTLEFIELD_CHANGE: i8 = 3;

/// A morale or luck change for the creatures of some factions, as authored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionChangeDef {
    /// The factions whose creatures it reaches (creatures without a faction never).
    pub factions: Vec<ContentKey>,
    /// The change, `-3..=3` and not 0.
    pub amount: i8,
}

/// A special battlefield as authored (`[[battlefield]]` in `battle.toml`, pack format v36).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlefieldDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Its random obstacles are the types listing it, not the terrain's (BAT-GEO-004).
    pub own_obstacles: bool,
    /// Hexes blocked instead of any random obstacle (two boats), in hex numbers.
    pub blocked: Vec<u8>,
    /// Morale changes by faction (BAT-TER-003).
    pub morale: Vec<FactionChangeDef>,
    /// Luck changes by faction (BAT-TER-003).
    pub luck: Vec<FactionChangeDef>,
    /// Every stack's morale is 0 (BAT-TER-003).
    pub no_morale: bool,
    /// Every stack's luck is 0 (BAT-TER-003).
    pub no_luck: bool,
    /// No stack fights on its native terrain (BAT-TER-001).
    pub no_native: bool,
    /// The highest spell level cast or taking effect (BAT-TER-004).
    pub max_spell_level: Option<u8>,
    /// No creature casts, raises or periodic casts (BAT-TER-004).
    pub no_creature_casts: bool,
    /// The schools whose spells a hero casts at Expert (BAT-TER-005).
    pub expert_schools: Vec<ContentKey>,
    /// Creature, strike and opening spells are cast at Expert (BAT-TER-005).
    pub expert_other_casts: bool,
}

/// A battlefield's handle: an index into [`crate::BattleRuleset::battlefields`].
pub type BattlefieldId = Handle<Battlefield>;

/// A compiled morale or luck change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionChange {
    /// The factions it reaches.
    pub factions: Vec<FactionId>,
    /// The change.
    pub amount: i8,
}

/// A compiled special battlefield (see [`BattlefieldDef`] for the fields' rules).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battlefield {
    /// Key.
    pub key: ContentKey,
    /// Its random obstacles are its own types.
    pub own_obstacles: bool,
    /// Hexes blocked instead of random obstacles.
    pub blocked: Vec<u8>,
    /// Morale changes by faction.
    pub morale: Vec<FactionChange>,
    /// Luck changes by faction.
    pub luck: Vec<FactionChange>,
    /// Every stack's morale is 0.
    pub no_morale: bool,
    /// Every stack's luck is 0.
    pub no_luck: bool,
    /// No native terrain bonus.
    pub no_native: bool,
    /// The highest spell level, `u8::MAX` without a limit.
    pub max_spell_level: u8,
    /// No creature casts.
    pub no_creature_casts: bool,
    /// Hero spells of these schools at Expert.
    pub expert_schools: Vec<SchoolId>,
    /// Other casts at Expert.
    pub expert_other_casts: bool,
}

impl Battlefield {
    /// The sum of `changes` that reaches a creature of `faction` (none without a faction).
    pub fn change(changes: &[FactionChange], faction: Option<FactionId>) -> i32 {
        faction.map_or(0, |f| {
            changes
                .iter()
                .filter(|c| c.factions.contains(&f))
                .map(|c| i32::from(c.amount))
                .sum()
        })
    }

    /// Whether a hero casts a spell of `schools` at Expert here (BAT-TER-005).
    pub fn raises(&self, schools: &[SchoolId]) -> bool {
        schools.iter().any(|s| self.expert_schools.contains(s))
    }
}

/// Resolves and checks a battlefield definition: factions and schools of the closure (by
/// `faction` and `school`), changes within ±[`MAX_BATTLEFIELD_CHANGE`] and not 0, no faction twice
/// in one list, a spell level limit of 1 to 5, distinct blocked hexes and no own obstacles beside
/// them ([`validate`] checks the hexes against the field once the parameters are known).
pub(crate) fn compile(
    def: &BattlefieldDef,
    mut faction: impl FnMut(&ContentKey) -> Result<FactionId, CompileError>,
    mut school: impl FnMut(&ContentKey) -> Result<SchoolId, CompileError>,
) -> Result<Battlefield, CompileError> {
    let invalid = |reason| CompileError::InvalidContent {
        kind: "battlefield",
        key: String::from(def.key.as_str()),
        reason,
    };
    let mut changes = |defs: &[FactionChangeDef]| -> Result<Vec<FactionChange>, CompileError> {
        let mut out: Vec<FactionChange> = Vec::with_capacity(defs.len());
        for c in defs {
            if c.amount == 0 || c.amount.unsigned_abs() > MAX_BATTLEFIELD_CHANGE.unsigned_abs() {
                return Err(invalid("a morale or luck change is -3..=3 and not 0"));
            }
            let mut factions = Vec::with_capacity(c.factions.len());
            for key in &c.factions {
                let f = faction(key)?;
                if factions.contains(&f) || out.iter().any(|o| o.factions.contains(&f)) {
                    return Err(invalid("a faction is listed twice in one list"));
                }
                factions.push(f);
            }
            if factions.is_empty() {
                return Err(invalid("a change needs at least one faction"));
            }
            out.push(FactionChange {
                factions,
                amount: c.amount,
            });
        }
        Ok(out)
    };
    let morale = changes(&def.morale)?;
    let luck = changes(&def.luck)?;
    let mut expert_schools = Vec::with_capacity(def.expert_schools.len());
    for key in &def.expert_schools {
        let s = school(key)?;
        if expert_schools.contains(&s) {
            return Err(invalid("a school is listed twice"));
        }
        expert_schools.push(s);
    }
    if def.max_spell_level.is_some_and(|l| !(1..=5).contains(&l)) {
        return Err(invalid("the spell level limit is 1..=5"));
    }
    if !def.blocked.is_empty() && def.own_obstacles {
        return Err(invalid("fixed blocked hexes replace random obstacles"));
    }
    if def
        .blocked
        .iter()
        .enumerate()
        .any(|(i, h)| def.blocked[..i].contains(h))
    {
        return Err(invalid("blocked hexes must be distinct"));
    }
    Ok(Battlefield {
        key: def.key.clone(),
        own_obstacles: def.own_obstacles,
        blocked: def.blocked.clone(),
        morale,
        luck,
        no_morale: def.no_morale,
        no_luck: def.no_luck,
        no_native: def.no_native,
        max_spell_level: def.max_spell_level.unwrap_or(u8::MAX),
        no_creature_casts: def.no_creature_casts,
        expert_schools,
        expert_other_casts: def.expert_other_casts,
    })
}

/// Checks a battlefield's blocked hexes against the field: every one standable.
pub(crate) fn validate(b: &Battlefield, battle: &BattleParams) -> Result<(), CompileError> {
    let f = battle.field;
    let standable = |&hex: &u8| {
        let (x, y) = (hex % f.width, hex / f.width);
        y < f.height && x >= f.side_columns && x < f.width - f.side_columns
    };
    if b.blocked.iter().all(standable) {
        Ok(())
    } else {
        Err(CompileError::InvalidContent {
            kind: "battlefield",
            key: String::from(b.key.as_str()),
            reason: "blocked hexes must be standable hexes of the field",
        })
    }
}

impl CanonicalHash for Battlefield {
    /// Encoding (battle part v2): key; own obstacles; blocked hexes as length + bytes; morale
    /// then luck changes as length + (factions as length + handles, amount); the flags no
    /// morale, no luck, no native; the level limit; no creature casts; the schools as length +
    /// handles; Expert for other casts.
    fn hash_into(&self, h: &mut StateHasher) {
        self.key.as_str().hash_into(h);
        h.write_bool(self.own_obstacles);
        h.write_len(self.blocked.len());
        for &b in &self.blocked {
            h.write_u8(b);
        }
        for list in [&self.morale, &self.luck] {
            h.write_len(list.len());
            for c in list {
                h.write_len(c.factions.len());
                for f in &c.factions {
                    f.hash_into(h);
                }
                h.write_i32(i32::from(c.amount));
            }
        }
        h.write_bool(self.no_morale);
        h.write_bool(self.no_luck);
        h.write_bool(self.no_native);
        h.write_u8(self.max_spell_level);
        h.write_bool(self.no_creature_casts);
        h.write_len(self.expert_schools.len());
        for s in &self.expert_schools {
            s.hash_into(h);
        }
        h.write_bool(self.expert_other_casts);
    }
}
