//! A spell's aim (E17 M5 T3b; `docs/epics/E17-design.md` §7.18): the hexes a spell chosen in the
//! hero's book, or a Faerie Dragon's drawn spell by its key F, may be cast at, and what the mouse
//! shows over each (`spell_target_cursor` 0x59F700): over a hex it takes the spell cursor, the
//! cast's line (0x5A8690) and the stack it marks (an area spell: every stack of its area, 0x59F8A0);
//! elsewhere the combat cursor's frame 0 and why the spell is not cast there (`spell_fails_message`
//! 0x5A2880). Which hexes it takes is the engine's answer ([`Target`]: `cast_targets`, a Faerie
//! Dragon's legal casts); the reasons and the area are the executable's own words and shapes.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_protocol::wire::{Side, Stack};

use super::{COLUMNS, Field, Hint, ROWS, Spot, distance};

/// The original's spells whose cast's line names no stack (0x5A8690's first case: Force Field,
/// Fire Wall, Frost Ring, Fireball, Inferno, Meteor Shower).
const UNNAMED: [u16; 6] = [12, 13, 20, 21, 22, 23];
/// The original's raising spells (Resurrection, Animate Dead): the line names the stack they
/// raise (0x5A86E0); with Sacrifice the reason "does not work on" (0x5A2AA7).
const RAISES: [u16; 2] = [38, 39];
/// Sacrifice's original number.
const SACRIFICE: u16 = 40;
/// Remove Obstacle's original number: its own line (GENRLTXT 551, 0x5A87D8).
const REMOVE_OBSTACLE: u16 = 64;

/// What the engine lets a spell be cast at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    /// A stack (its hexes take the spell).
    Stack(u8),
    /// A hex.
    Hex(u8),
}

/// An area spell's shape (0x59F8A0): the hexes within `radius` of the one aimed at, its own
/// only with `centre` (Frost Ring's is left out: `area_hexes` 0x5A4170).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
    /// The radius in hexes.
    pub radius: u8,
    /// The hex aimed at is in it.
    pub centre: bool,
}

/// A spell to aim: what the binary knows of it from the engine's ruleset and targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aim {
    /// Its handle.
    pub spell: u16,
    /// Its number in the original (the spell cursor's `SetCursor(spell + 1, 3)`, the lines'
    /// cases).
    pub original: u16,
    /// A creature casts it (the Faerie Dragon's key F: action 10), not the hero (action 1).
    pub creature: bool,
    /// What the engine lets it be cast at, sorted.
    pub targets: Vec<Target>,
    /// Its area, for the spells that mark one (the original's flags 0x280: Frost Ring,
    /// Fireball, Inferno, Meteor Shower; Berserk by the mastery, 0x5A4810).
    pub area: Option<Area>,
    /// It is aimed at a stack (the original's flags 0x70: `spell_fails_message` names the
    /// stack's reasons).
    pub stacks: bool,
    /// Its polarity: below 0 hostile, above 0 friendly (the spell record's +0).
    pub polarity: i8,
    /// Its level.
    pub level: u8,
    /// Anti-Magic's handle (a stack under it "protects" from the spells below its level).
    pub anti_magic: Option<u16>,
}

/// The executable's number of where the mouse is (`screen_to_hex`): a hex's own, the heroes'
/// rectangles 0xFC and 0xFD, the keep's 0xFE, the upper tower's 0xFF, nothing −1. The aim calls
/// the cursor again only for another number (0x5A30C4).
#[must_use]
pub const fn screen_hex(spot: Spot) -> i32 {
    match spot {
        Spot::Hex(hex) => hex as i32,
        Spot::Hero(Side::Attacker) => 0xFC,
        Spot::Hero(Side::Defender) => 0xFD,
        Spot::Keep => 0xFE,
        Spot::Tower => 0xFF,
        Spot::Nothing => -1,
    }
}

/// A hex of the field outside the two side columns (the cursor's first test, 0x59F77C).
fn inner(hex: i32) -> Option<u8> {
    let hex = u8::try_from(hex).ok().filter(|&h| h < COLUMNS * ROWS)?;
    let column = hex % COLUMNS;
    (column != 0 && column != COLUMNS - 1).then_some(hex)
}

impl Aim {
    /// The hex the cursor takes at `hex` (`spell_target_cursor`'s test, 0x59F77C to 0x59F7FE):
    /// a hex outside the side columns the engine lets the spell be cast at, or one of a stack
    /// it lets it be cast at.
    #[must_use]
    pub fn takes(&self, field: &Field<'_>, hex: i32) -> Option<u8> {
        let hex = inner(hex)?;
        self.target(field, hex).map(|_| hex)
    }

    /// The engine's target at `hex`: the hex itself, else a stack on it (a raising spell's dead
    /// one too).
    #[must_use]
    pub fn target(&self, field: &Field<'_>, hex: u8) -> Option<Target> {
        if self.targets.contains(&Target::Hex(hex)) {
            return Some(Target::Hex(hex));
        }
        self.targets.iter().copied().find(|t| match *t {
            Target::Stack(id) => field
                .observation
                .stacks
                .iter()
                .any(|s| s.id == id && field.hexes(s).contains(&hex)),
            Target::Hex(_) => false,
        })
    }

    /// The cast's line over `hex`, a hex the cursor takes (0x5A8690): "Cast %s" (27) for the
    /// spells naming no stack, Remove Obstacle's own (551), else "Cast %s on %s" (28) with the
    /// stack there (a raising spell's: the one it raises) by its number, "Cast %s" without one.
    #[must_use]
    pub fn line(&self, field: &Field<'_>, hex: u8) -> Hint {
        if UNNAMED.contains(&self.original) {
            return Hint::Cast(self.spell, None);
        }
        if self.original == REMOVE_OBSTACLE {
            return Hint::RemoveObstacle;
        }
        let stack = if RAISES.contains(&self.original) {
            match self.target(field, hex) {
                Some(Target::Stack(id)) => Some(id),
                _ => None,
            }
        } else {
            field.at(hex).map(|s| s.id)
        };
        Hint::Cast(self.spell, stack)
    }

    /// Why the spell is not cast at `hex` (`spell_fails_message` 0x5A2880 with GENRLTXT 24),
    /// the acting side `side`: for a spell aimed at a stack with a living stack there, Anti-Magic
    /// protecting it (171), a hostile spell at an own stack (172), a friendly one at an enemy
    /// (178), else its immunity (a raising spell's 180, any other's 181); else "Select Spell
    /// Target". *Provisional:* the executable words the immunity only where its creature's
    /// factor is 0 (0x44A4D0) and the default otherwise; the client takes any other refusal of
    /// the engine for it (the engine's tests are not public).
    #[must_use]
    pub fn refusal(&self, field: &Field<'_>, hex: i32, side: Side) -> Hint {
        let Some(stack) = u8::try_from(hex)
            .ok()
            .filter(|&h| self.stacks && h < COLUMNS * ROWS)
            .and_then(|h| field.at(h))
        else {
            return Hint::SelectTarget;
        };
        if self
            .anti_magic
            .is_some_and(|a| protects(stack, a, self.level))
        {
            return Hint::Protects(self.anti_magic.unwrap_or_default(), stack.id);
        }
        let own = stack.side == side;
        if self.polarity < 0 && own {
            return Hint::HostileOnly(self.spell);
        }
        if self.polarity > 0 && !own {
            return Hint::FriendlyOnly(self.spell);
        }
        if RAISES.contains(&self.original) || self.original == SACRIFICE {
            Hint::NotOn(self.spell, stack.id)
        } else {
            Hint::Immune(stack.id, self.spell)
        }
    }

    /// The living stacks an area spell aimed at `hex` marks (0x59F8A0 through `area_stacks`
    /// 0x5A46F0): those on its hexes (`area_hexes` 0x5A4170: within the radius, in the columns
    /// 1 to 15, its own only with the centre), each once; none without an area or off the field.
    /// *Provisional:* the executable marks only the stacks the spell may affect (0x5A8090 above
    /// 0); the client marks every living stack of the area.
    #[must_use]
    pub fn marks(&self, field: &Field<'_>, hex: i32) -> BTreeSet<u8> {
        let mut marked = BTreeSet::new();
        let (Some(area), Some(centre)) = (
            self.area,
            u8::try_from(hex).ok().filter(|&h| h < COLUMNS * ROWS),
        ) else {
            return marked;
        };
        for h in 0..COLUMNS * ROWS {
            let column = h % COLUMNS;
            let d = distance(centre, h);
            if column == 0
                || column == COLUMNS - 1
                || d > i32::from(area.radius)
                || (d == 0 && !area.centre)
            {
                continue;
            }
            if let Some(stack) = field.at(h) {
                marked.insert(stack.id);
            }
        }
        marked
    }
}

/// Whether Anti-Magic (`anti_magic`) protects `stack` from a spell of `level` (0x5A298E: the
/// stack's mark +0x220 and a level below its +0x460): an active Anti-Magic whose value is above
/// the level. *Provisional:* the effect's value taken for +0x460.
fn protects(stack: &Stack, anti_magic: u16, level: u8) -> bool {
    stack
        .effects
        .iter()
        .any(|e| e.spell == anti_magic && u16::from(level) < e.value)
}

#[cfg(test)]
mod tests;
