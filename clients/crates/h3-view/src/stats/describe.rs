//! Morale's and luck's descriptions (E17 M6 T2a2b2, `docs/epics/E17-design.md` §12.4): the right
//! button over the stats dialog's morale or luck shows a header by the value's sign, then the lines
//! that tell where the value comes from (`stack_morale_description` 0x44B960,
//! `stack_luck_description` 0x44C1C0), or the empty list's line, with the sign's picture. [`Account`]
//! holds what a seat's observation tells of the sources: since M6 T3c1 the hero's own lists
//! ([`HeroList`]: its artifacts, its Leadership or Luck and the rest as temporary, from CLT-Q40's
//! carry), and since M6 T3c2 the Hourglass of the Evil Hour's and the Spirit of Oppression's lines
//! (design §12.10; the Angelic Alliance's join is in [`Account::alignments`]), a siege town's
//! buildings and the owner's Grail building ([`Building`]); the spells' line is the spells' change
//! as the executable counts it from the lines the client
//! has.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

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
    /// The Hourglass of the Evil Hour's name when a cap on luck reaches the side (either hero's,
    /// `LuckAtMost`): luck's description is that artifact's line alone (ARRAYTXT 82).
    pub hourglass: Option<String>,
    /// The Spirit of Oppression's name when a cap on morale reaches the side (either hero's,
    /// `MoraleAtMost`): above a base of 0 its line replaces the lines before it (ARRAYTXT 119).
    pub oppression: Option<String>,
    /// The side's hero's morale list (none without a hero: the default).
    pub hero_morale: HeroList,
    /// Its luck list.
    pub hero_luck: HeroList,
    /// A siege town's buildings that change the defender's morale, in the original's order
    /// (T3c2, 0x44B960: the Tavern, a Castle's Brotherhood of the Sword); none for a stack of
    /// the attacker or outside a siege.
    pub siege_morale: Vec<Building>,
    /// Its buildings that change the defender's luck (a Rampart's Fountain of Fortune).
    pub siege_luck: Vec<Building>,
}

/// A town building the descriptions name (E17 M6 T3c2, design §12.9; the original's ids in the
/// town record: the Tavern 5, the Brotherhood of the Sword 22, the Fountain of Fortune 21, the
/// Grail's building 26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Building {
    /// The Tavern: morale +1 for a siege's defender.
    Tavern,
    /// A Castle's Brotherhood of the Sword: morale +2 for a siege's defender.
    Brotherhood,
    /// A Rampart's Fountain of Fortune: luck +2 for a siege's defender.
    Fountain,
    /// A Castle's Grail building (the Colossus): its owner's heroes' morale +2.
    Colossus,
    /// A Rampart's Grail building (the Spirit Guardian): its owner's heroes' luck +2.
    SpiritGuardian,
}

impl Building {
    /// The key context of its name (`stats.building.name`).
    const fn name(self) -> &'static str {
        match self {
            Self::Tavern => "tavern",
            Self::Brotherhood => "brotherhood",
            Self::Fountain => "fountain",
            Self::Colossus => "colossus",
            Self::SpiritGuardian => "spirit-guardian",
        }
    }

    /// Its change, which its line prints (the Tavern's 1, the others' 2).
    const fn change(self) -> i32 {
        match self {
            Self::Tavern => 1,
            _ => 2,
        }
    }

    /// Its line (`"\n%s +N"`, the executable's 0x660B48 or 0x660B40) and its change.
    fn line(self, catalogue: &Catalogue) -> (String, i32) {
        let name = catalogue.text("stats.building.name", &Args::new().context(self.name()));
        let context = if self.change() == 1 {
            "plus-one"
        } else {
            "plus-two"
        };
        let line = catalogue.text(
            "stats.building.line",
            &Args::new().context(context).text("name", &name),
        );
        (line, self.change())
    }
}

/// A hero's morale or luck list (`hero_morale_description` 0x4DC320, `hero_luck_description`
/// 0x4DCAC0, read whole at E17 M6 T3c1; design §12.9), as far as the observation's sources tell
/// it (CLT-Q40's carry): the stack's description writes it first, after the base. Its lines: each
/// listed artifact the hero's changes name, in the list's order (the Pendant of Courage first),
/// its change the line's own (+3, the others +1); Leadership's or Luck's line by its level (its
/// change the level); then the rest, the total less those lines' changes, as temporary (less or
/// more) when it is not 0. The original's visits' lines (their once-marks on the wire name no
/// object type) and its owner's Grail building's (a town record's index, T3c2) are in the rest
/// meanwhile, as the original's own rest holds what its lines do not name; its cheats' line has no
/// source.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroList {
    /// The hero's changes that reach its own stacks, summed (the original's `hero_morale` 0x4E39B0
    /// or `hero_luck` 0x4E36C0 with no other hero, no field and no clamp): part of the base.
    pub total: i32,
    /// The listed artifacts its changes name, by their original ids (any order).
    pub artifacts: Vec<u32>,
    /// The level, 1 (Basic) to 3 (Expert), of the skill its changes name (Leadership for morale,
    /// Luck for luck); 0 none.
    pub skill: u8,
    /// The owner's Grail building the list names (T3c2: a Castle's for morale, a Rampart's for
    /// luck); its change is in `total` and its line is the list's own, after the skill's.
    pub grail: Option<Building>,
}

/// The Pendant of Courage (original id 108): its lines' change is 3.
const PENDANT: u32 = 108;

/// The morale list's artifacts in its order (0x4DC3A0 to 0x4DC559), by their original ids, with
/// their key's context: the Pendant of Courage, the Still Eye of the Dragon, the Badge of Courage,
/// the Crest of Valor, the Glyph of Gallantry.
const MORALE_ARTIFACTS: [(u32, &str); 5] = [
    (PENDANT, "pendant"),
    (45, "still-eye"),
    (49, "badge"),
    (50, "crest"),
    (51, "glyph"),
];

/// The luck list's (0x4DCB40 to 0x4DCCF9): the Pendant, the Still Eye, the Clover of Fortune, the
/// Cards of Prophecy, the Ladybird of Luck.
const LUCK_ARTIFACTS: [(u32, &str); 5] = [
    (PENDANT, "pendant"),
    (45, "still-eye"),
    (46, "clover"),
    (47, "cards"),
    (48, "ladybird"),
];

/// The skill's line's context by its level less 1.
const LEVELS: [&str; 3] = ["basic", "advanced", "expert"];

/// The lines of `list` with the keys of `prefix` (`stats.morale`, `stats.luck`) and the list's
/// artifacts `artifacts`.
fn hero_lines(
    catalogue: &Catalogue,
    prefix: &str,
    artifacts: &[(u32, &str)],
    list: &HeroList,
) -> String {
    let mut lines = String::new();
    let mut listed = 0;
    for &(id, context) in artifacts {
        if list.artifacts.contains(&id) {
            lines.push_str(
                &catalogue.text(&format!("{prefix}.artifact"), &Args::new().context(context)),
            );
            listed += if id == PENDANT { 3 } else { 1 };
        }
    }
    if let Some(context) = usize::from(list.skill)
        .checked_sub(1)
        .and_then(|k| LEVELS.get(k))
    {
        lines.push_str(&catalogue.text(&format!("{prefix}.skill"), &Args::new().context(context)));
        listed += i32::from(list.skill);
    }
    if let Some(grail) = list.grail {
        let (line, change) = grail.line(catalogue);
        lines.push_str(&line);
        listed += change;
    }
    let rest = list.total - listed;
    if rest != 0 {
        let args = Args::new()
            .context(if rest < 0 { "less" } else { "more" })
            .number("amount", i64::from(rest.unsigned_abs()));
        lines.push_str(&catalogue.text(&format!("{prefix}.temporary"), &args));
    }
    lines
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

/// Morale's description (0x5F4B86 with 0x44B960) for a dialog whose morale shows `value` (the
/// stack's, unclamped): on Cursed Ground its line alone, for a creature without morale its line
/// alone; else the hero's list ([`HeroList`]), the field's line by the creature's town, the army's
/// alignments, undead, Angels, the other army's dragons, the Minotaurs' minimum and the spells'
/// line.
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
        // The hero's list after the base (0x44BA58); its changes are the base's.
        let mut lines = hero_lines(
            catalogue,
            "stats.morale",
            &MORALE_ARTIFACTS,
            &account.hero_morale,
        );
        let mut base = account.hero_morale.total;
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
        base += buildings(catalogue, &mut lines, &account.siege_morale);
        let field_change = field.map_or(0, |(_, change)| change);
        let mut shown = base + account.morale_spells + field_change;
        if let Some(minotaurs) = &account.morale_minimum {
            minimum(&mut lines, (&mut base, &mut shown), || {
                named("stats.morale.minimum", minotaurs)
            });
        }
        if let Some(artifact) = &account.oppression {
            // The Spirit of Oppression (0x44B960): above a base of 0 its line replaces every
            // line before it; the stack's morale is cut to 0 when it is above (0x43E0D3).
            if base > 0 {
                lines = catalogue.text(
                    "stats.morale.oppression",
                    &Args::new().text("artifact", artifact),
                );
                base = 0;
            }
            shown = shown.min(0);
        }
        spells(catalogue, &mut lines, shown - field_change - base);
        lines
    };
    compose(catalogue, "stats.morale", value, &lines)
}

/// Luck's description (0x5F4937 with 0x44C1C0) for a dialog whose luck shows `value` (the
/// stack's, unclamped: below 0 too), in the profile `bugs`: on Cursed Ground its line alone; else
/// the hero's list ([`HeroList`]), Clover Field's line for a creature of a neutral town, the other
/// army's Devils, the Halflings' minimum (its line without a line break before it in the
/// original's profile, on its own row in the client's: CLT-Q41 (1)) and the spells' line.
#[must_use]
pub fn luck(catalogue: &Catalogue, account: &Account, value: i32, bugs: Bugs) -> String {
    let lines = if account.field == Field::Cursed {
        catalogue.text("stats.luck.cursed", &Args::new())
    } else if let Some(artifact) = &account.hourglass {
        // Either hero's Hourglass of the Evil Hour (0x44C1C0): its line alone.
        catalogue.text(
            "stats.luck.hourglass",
            &Args::new().text("artifact", artifact),
        )
    } else {
        // The hero's list after the base (0x44C2A5); its changes are the base's.
        let mut lines = hero_lines(catalogue, "stats.luck", &LUCK_ARTIFACTS, &account.hero_luck);
        let mut base = account.hero_luck.total;
        let clover = account.field == Field::Clover && matches!(account.town, Some(6..=8));
        if clover {
            lines.push_str(&catalogue.text("stats.luck.clover", &Args::new()));
        }
        if let Some(devils) = &account.devils {
            lines.push_str(
                &catalogue.text("stats.morale.facing", &Args::new().text("creature", devils)),
            );
            base -= 1;
        }
        base += buildings(catalogue, &mut lines, &account.siege_luck);
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

/// The siege town's `list` of buildings: their lines appended, their changes' sum.
fn buildings(catalogue: &Catalogue, lines: &mut String, list: &[Building]) -> i32 {
    let mut sum = 0;
    for building in list {
        let (line, change) = building.line(catalogue);
        lines.push_str(&line);
        sum += change;
    }
    sum
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
