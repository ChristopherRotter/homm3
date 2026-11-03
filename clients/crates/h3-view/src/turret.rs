//! A siege turret's info (`turret_info` 0x47A380, `turret_info_line` 0x47A2D0; E17 M6 T2b2,
//! `docs/epics/E17-design.md` §12.6): a left press on a turret (the cursor's kind 22) shows it in a
//! message box with its OK, the right button over the keep or a Castle's towers in a popup held
//! while the button is (where: [`crate::mouse::turret_at`]). Its text is a line for the keep and,
//! with a Castle, one for the upper and one for the lower tower, in that order: "The %s has an
//! attack skill of %d and does %d-%d damage." (GENRLTXT 156) while the part stands, else "The %s is
//! destroyed." (155), each starting with its own two line breaks (the original's text, as read);
//! nothing below a Citadel (0x47A39F).

use alloc::string::String;
use alloc::vec::Vec;

use h3_protocol::wire::Observation;
use h3_ui::text::{Args, Catalogue};

use crate::siege::{KEEP_TARGET, TARGET_PARTS, TOWER_TARGETS};

/// The turrets' attack skill the info shows: `town_turret_count` 0x5BF4F0's second output, a
/// literal (0x5BF4FE).
pub const ATTACK: u32 = 10;

/// The lines in order: each turret's wall-shot target (the observation's `parts` are by target;
/// its part is [`TARGET_PARTS`]'s: the keep 14, the upper tower 5, the lower tower 13), its
/// stack's position (BAT-SIE-016, the original's records) and the fort level that builds it (the
/// keep's a Citadel, the towers' a Castle).
const LINES: [(usize, u8, u8); 3] = [
    (KEEP_TARGET, 254, 2),
    (TOWER_TARGETS[0], 255, 3),
    (TOWER_TARGETS[1], 251, 3),
];

/// One turret's line (`turret_info_line` 0x47A2D0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    /// Its part (WALLS.TXT's row), which names it.
    pub part: usize,
    /// While the part has hit points left (+0x13F60 by part; the observation's `parts` by its
    /// wall-shot target), its
    /// turret's count (the keep's `town_turret_count`, a tower's half of it rounded up: what the
    /// observation's turret stack holds, 0 without one); none: destroyed.
    pub count: Option<u32>,
}

/// The lines of a siege's turret info in `observation`: none outside a siege or below a Citadel.
#[must_use]
pub fn lines(observation: &Observation) -> Vec<Line> {
    let Some(siege) = observation.siege.as_ref() else {
        return Vec::new();
    };
    LINES
        .iter()
        .filter(|&&(_, _, fort)| siege.fort >= fort)
        .map(|&(target, hex, _)| {
            let standing = siege.parts.get(target).is_some_and(|&hp| hp > 0);
            let count = standing.then(|| {
                observation
                    .stacks
                    .iter()
                    .find(|s| s.hex == hex && s.alive)
                    .map_or(0, |s| s.count)
            });
            Line {
                part: TARGET_PARTS[target],
                count,
            }
        })
        .collect()
}

/// The info's text in `catalogue`'s words (`battle.turret.standing` with the part's name, the
/// attack skill and the damage's ends, twice and three times the count; `battle.turret.destroyed`
/// with the name), the lines one after another; a part named by `part` (the town's faction and
/// the part: its key, `wall.<faction>.<part>`). None where [`lines`] has none.
#[must_use]
pub fn text(
    observation: &Observation,
    catalogue: &Catalogue,
    part: &dyn Fn(u16, usize) -> Option<String>,
) -> Option<String> {
    let faction = observation.siege.as_ref()?.faction;
    let lines = lines(observation);
    if lines.is_empty() {
        return None;
    }
    let mut out = String::new();
    for line in lines {
        let name = part(faction, line.part)
            .map_or_else(String::new, |key| catalogue.text(&key, &Args::new()));
        let args = Args::new().text("part", &name);
        out.push_str(&match line.count {
            Some(count) => {
                let count = i64::from(count);
                let args = args
                    .number("attack", i64::from(ATTACK))
                    .number("low", 2 * count)
                    .number("high", 3 * count);
                catalogue.text("battle.turret.standing", &args)
            }
            None => catalogue.text("battle.turret.destroyed", &args),
        });
    }
    Some(out)
}

#[cfg(test)]
mod tests;
