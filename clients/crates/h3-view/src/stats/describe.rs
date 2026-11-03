//! Morale's and luck's descriptions (E17 M6 T2a2b2, `docs/epics/E17-design.md` §12.4): the right
//! button over the stats dialog's morale or luck shows a header by the value's sign, then the lines
//! that tell where the value comes from (`stack_morale_description` 0x44B960,
//! `stack_luck_description` 0x44C1C0), or the empty list's line, with the sign's picture. [`Account`]
//! holds what a seat's observation tells of the sources: the hero's own lists, the Hourglass of the
//! Evil Hour and the Spirit of Oppression, the Angelic Alliance's join and a siege town's buildings
//! are not in it (CLT-Q40: requested from the battle lane), so their lines are not written; the
//! spells' line is the spells' change as the executable counts it from the lines the client has.

use alloc::format;
use alloc::string::String;

use h3_render::Bugs;
use h3_ui::text::{Args, Catalogue};

/// The special battlefields the descriptions name (the battle manager's +0x53C0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Field {
    /// Any other field, or none.
    #[default]
    Other,
    /// Cursed Ground (2): no morale, no luck; its line alone.
    Cursed,
    /// Holy Ground (3): the good towns' creatures +1, the evil ones' −1.
    Holy,
    /// Evil Fog (4): the good towns' −1, the evil ones' +1.
    EvilFog,
    /// Clover Field (5): luck +2 for the creatures of the neutral towns (6 to 8).
    Clover,
}

/// What the descriptions tell of a stack's sources, as far as a seat's observation shows them
/// (the binary's `stats` derives it; CLT-Q40). The armies are those of the side whose sources the
/// description lists: in the client's profile the stack's controller's, in the original's its
/// owner's (CLT-Q41 (2)).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    /// The special battlefield.
    pub field: Field,
    /// The creature's town (0 Castle to 8 Conflux), none for a neutral creature.
    pub town: Option<u8>,
    /// The creature is unaffected by morale (its flag 0x20000): that line alone.
    pub no_morale: bool,
    /// The army's alignments (`army_alignment_count` 0x44ABB0): the towns of its slots'
    /// creatures but war machines, the neutral ones one.
    pub alignments: u32,
    /// An undead creature in the army (its flag 0x40000).
    pub undead: bool,
    /// The Archangels' plural when the army holds them, else the Angels' (12, 13).
    pub angels: Option<String>,
    /// The Ghost Dragons' plural when the other army holds them, else the Bone Dragons' (68, 69).
    pub dragons: Option<String>,
    /// The Arch Devils' plural when the other army holds them, else the Devils' (54, 55).
    pub devils: Option<String>,
    /// The creature's plural when its morale is at least 1 (the Minotaurs, 78 and 79).
    pub morale_minimum: Option<String>,
    /// The creature's plural when its luck is at least 1 (the Halflings, 138).
    pub luck_minimum: Option<String>,
    /// What the stack's timed spells change of its morale (Mirth's value less Sorrow's).
    pub morale_spells: i32,
    /// What they change of its luck (Fortune's less Misfortune's).
    pub luck_spells: i32,
}

/// The description's picture's frame by the value's sign (`message_box_picture`'s types 11 to 13
/// and 14 to 16: `ilck82.def`'s and `imrl82.def`'s frame 4 above 0, 3 at 0, 2 below).
#[must_use]
pub const fn picture_frame(value: i32) -> u32 {
    if value > 0 {
        4
    } else if value == 0 {
        3
    } else {
        2
    }
}

/// Morale's description (0x5F4B86 with 0x44B960) for a dialog whose morale shows `value`: on
/// Cursed Ground its line alone, for a creature without morale its line alone; else the field's
/// line by the creature's town, the army's alignments, undead, Angels, the other army's dragons,
/// the Minotaurs' minimum and the spells' line.
#[must_use]
pub fn morale(catalogue: &Catalogue, account: &Account, value: i32) -> String {
    let text = |key: &str| catalogue.text(key, &Args::new());
    let named =
        |key: &str, creature: &str| catalogue.text(key, &Args::new().text("creature", creature));
    let lines = if account.field == Field::Cursed {
        text("stats.morale.cursed")
    } else if account.no_morale {
        text("stats.morale.unaffected")
    } else {
        let mut lines = String::new();
        // The field's line by the creature's town (0 to 2 good, 3 to 5 evil); its change is in
        // the value, not in the base.
        let field = match (account.field, account.town) {
            (Field::Holy, Some(0..=2)) => Some(("stats.morale.holy-good", 1)),
            (Field::Holy, Some(3..=5)) => Some(("stats.morale.holy-evil", -1)),
            (Field::EvilFog, Some(0..=2)) => Some(("stats.morale.fog-good", -1)),
            (Field::EvilFog, Some(3..=5)) => Some(("stats.morale.fog-evil", 1)),
            _ => None,
        };
        if let Some((key, _)) = field {
            lines.push_str(&text(key));
        }
        let mut base = 0;
        let n = account.alignments;
        if n >= 3 {
            let change = if n >= 5 {
                -3
            } else {
                2 - i32::try_from(n).unwrap_or(0)
            };
            lines.push_str(
                &catalogue.text(
                    "stats.morale.alignments",
                    &Args::new()
                        .number("alignments", i64::from(n))
                        .number("change", i64::from(change)),
                ),
            );
            base += change;
        } else if n == 1 {
            lines.push_str(&text("stats.morale.one-alignment"));
            base += 1;
        }
        if account.undead {
            lines.push_str(&text("stats.morale.undead"));
            base -= 1;
        }
        if let Some(angels) = &account.angels {
            lines.push_str(&named("stats.morale.in-group", angels));
            base += 1;
        }
        if let Some(dragons) = &account.dragons {
            lines.push_str(&named("stats.morale.facing", dragons));
            base -= 1;
        }
        let field_change = field.map_or(0, |(_, change)| change);
        let mut shown = base + account.morale_spells + field_change;
        if let Some(minotaurs) = &account.morale_minimum {
            minimum(&mut lines, (&mut base, &mut shown), || {
                named("stats.morale.minimum", minotaurs)
            });
        }
        spells(catalogue, &mut lines, shown - field_change - base);
        lines
    };
    compose(catalogue, "stats.morale", value, &lines)
}

/// Luck's description (0x5F4937 with 0x44C1C0) for a dialog whose luck shows `value`, in the
/// profile `bugs`: on Cursed Ground its line alone; else Clover Field's line for a creature of a
/// neutral town, the other army's Devils, the Halflings' minimum (its line without a line break
/// before it in the original's profile, on its own row in the client's: CLT-Q41 (1)) and the
/// spells' line.
#[must_use]
pub fn luck(catalogue: &Catalogue, account: &Account, value: i32, bugs: Bugs) -> String {
    let lines = if account.field == Field::Cursed {
        catalogue.text("stats.luck.cursed", &Args::new())
    } else {
        let mut lines = String::new();
        let clover = account.field == Field::Clover && matches!(account.town, Some(6..=8));
        if clover {
            lines.push_str(&catalogue.text("stats.luck.clover", &Args::new()));
        }
        let mut base = 0;
        if let Some(devils) = &account.devils {
            lines.push_str(
                &catalogue.text("stats.morale.facing", &Args::new().text("creature", devils)),
            );
            base -= 1;
        }
        let field_change = if clover { 2 } else { 0 };
        let mut shown = base + account.luck_spells + field_change;
        if let Some(halflings) = &account.luck_minimum {
            minimum(&mut lines, (&mut base, &mut shown), || {
                let line = catalogue.text(
                    "stats.luck.halflings",
                    &Args::new().text("creature", halflings),
                );
                match bugs {
                    Bugs::Fixed => format!("\n{line}"),
                    Bugs::Original => line,
                }
            });
        }
        spells(catalogue, &mut lines, shown - field_change - base);
        lines
    };
    compose(catalogue, "stats.luck", value, &lines)
}

/// A creature's minimum of 1 (the Minotaurs' morale 0x44BF7B, the Halflings' luck 0x44C462): the
/// value shown is at least 1; a base below 1 writes its line (`line`) and becomes 1.
fn minimum(lines: &mut String, (base, shown): (&mut i32, &mut i32), line: impl FnOnce() -> String) {
    *shown = (*shown).max(1);
    if *base < 1 {
        lines.push_str(&line());
        *base = 1;
    }
}

/// The spells' line (ARRAYTXT 121, `%+d`) with `change`, when it is not 0.
fn spells(catalogue: &Catalogue, lines: &mut String, change: i32) {
    if change != 0 {
        lines.push_str(&catalogue.text(
            "stats.morale.spells",
            &Args::new().text("change", &format!("{change:+}")),
        ));
    }
}

/// The popup's text: the header (`<prefix>.header`) with the good, neutral or bad text by the
/// sign of `value`, then `lines`, or the empty list's line (`<prefix>.none`) when there are none.
fn compose(catalogue: &Catalogue, prefix: &str, value: i32, lines: &str) -> String {
    let sign = match value {
        v if v > 0 => "good",
        0 => "neutral",
        _ => "bad",
    };
    let said = catalogue.text(&format!("{prefix}.{sign}"), &Args::new());
    let mut text = catalogue.text(
        &format!("{prefix}.header"),
        &Args::new().text("text", &said),
    );
    if lines.is_empty() {
        text.push_str(&catalogue.text(&format!("{prefix}.none"), &Args::new()));
    } else {
        text.push_str(lines);
    }
    text
}
