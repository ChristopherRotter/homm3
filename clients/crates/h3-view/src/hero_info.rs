//! A hero's info window in a battle (E17 M6 T3b1, `docs/epics/E17-design.md` §12.8): the right
//! button over either side's hero shows it while the button is held (`battle_right_click`
//! 0x4769C0's "hexes" 0xFC and 0xFD: `army_info_hero_set` 0x46CC10, `army_info_show` 0x46CDF0,
//! `popup_run_held` 0x602A40 with no popup, then `army_info_hide` 0x46CE70). Its items are the
//! layout's (`[hero_info]`, built by `army_info_hero_create` 0x46C370 at the side's place); the
//! fill gives them:
//!
//! - the background in the hero's player's colours, the hero's large portrait;
//! - four rows "%s:" (GENRLTXT 381 to 384) with their values by "%d": attack and defense held to
//!   0..=99, power and knowledge to 1..=99;
//! - morale's and luck's pictures, their frames the hero's own morale and luck + 3
//!   (`hero_morale` 0x4E39B0, `hero_luck` 0x4E36C0 with the other side's hero, Cursed Ground's
//!   flag and the clamp; M6 T3b2, design §12.8: [`values`]);
//! - the spell points "%s\n%d/%d" (GENRLTXT 388, the points, the most: knowledge × 10 × the
//!   hero's intelligence factor, the observation's `max_mana`).
//!
//! Nothing over a side without a hero (0x4769D9, 0x476A4F).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;

use h3_protocol::wire::{Affects, MoraleLuck, Observation, Side};
use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Dialog, Shown, State};
use serde::Deserialize;

use crate::layout::BattleLayout;
use crate::message::MessageBox;
use crate::mouse::{Spot, spot};
use crate::stats::Field;

/// The background (`CHrPop.pcx`), in the hero's player's colours.
pub const BACKGROUND: u16 = 2100;
/// The portrait.
pub const PORTRAIT: u16 = 2101;
/// The first skill's label; its value is the next id, then the next skills' pairs (attack,
/// defense, power, knowledge: 2102 to 2109).
pub const SKILLS: u16 = 2102;
/// Morale's label.
pub const MORALE_LABEL: u16 = 2110;
/// Morale's picture (`imrls.def`).
pub const MORALE: u16 = 2111;
/// Luck's label.
pub const LUCK_LABEL: u16 = 2112;
/// Luck's picture (`ilcks.def`).
pub const LUCK: u16 = 2113;
/// The spell points.
pub const POINTS: u16 = 2114;

/// The skills' words' keys, attack to knowledge (GENRLTXT 381 to 384).
const SKILL_WORDS: [&str; 4] = [
    "battle.hero-info.attack",
    "battle.hero-info.defense",
    "battle.hero-info.power",
    "battle.hero-info.knowledge",
];

/// The window's layout (the layout's `[hero_info]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroInfoLayout {
    /// Each side's window's corner on the screen: the attacker's (1, 135), the defender's
    /// (721, 135) (`combat_dialog_create` 0x4724A9, 0x4724E2).
    pub places: [[i32; 2]; 2],
    /// The window, its items in the order the constructor adds them (from its corner).
    pub dialog: Dialog,
}

/// What the window shows of a side's hero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// The hero's player's colour (0 red to 7 pink), the background's.
    pub player: u8,
    /// Its large portrait (`HPL*.pcx`, the hero table's +0x34), none: none drawn.
    pub portrait: Option<String>,
    /// Attack, defense, power and knowledge, as the hero has them.
    pub skills: [u8; 4],
    /// Its own morale ([`values`]; its picture's frame + 3, held to −3..=3).
    pub morale: i32,
    /// Its own luck (as morale).
    pub luck: i32,
    /// Its spell points now.
    pub mana: u16,
    /// Its spell points at most.
    pub max_mana: u16,
}

impl Info {
    /// Side `side`'s hero in `observation` (its player's colour, none named: red for the
    /// attacker, blue for the defender, CLT-Q6), its portrait by its hero type's handle from
    /// `portrait`, its morale and luck on `field` ([`values`]); none without a hero.
    #[must_use]
    pub fn of(
        observation: &Observation,
        side: Side,
        field: Field,
        portrait: &dyn Fn(u16) -> Option<String>,
    ) -> Option<Self> {
        let index = usize::from(side == Side::Defender);
        let view = &observation.sides[index];
        let hero = view.hero.as_ref()?;
        let (morale, luck) = values(observation, side, field);
        Some(Self {
            player: view
                .colour
                .unwrap_or_else(|| u8::try_from(index).unwrap_or(0)),
            portrait: portrait(hero.hero_type),
            skills: [hero.attack, hero.defense, hero.power, hero.knowledge],
            morale,
            luck,
            mana: hero.mana,
            max_mana: hero.max_mana,
        })
    }
}

/// Side `side`'s hero's own morale and luck on `field`, as the window shows them (E17 M6 T3b2,
/// design §12.8): `hero_morale` 0x4E39B0 and `hero_luck` 0x4E36C0, called with the other side's
/// hero, Cursed Ground's flag and the clamp (0x46CD3B, 0x46CD55). The hero's own list alone (the
/// observation's `morale_and_luck`, CLT-Q40: its skills, artifacts, specialty, visits, its owner's
/// Grail building and the rest as temporary), not its army's alignments or the field's other
/// changes, which the stacks' values add:
///
/// - morale: 0 on Cursed Ground; else the hero's morale changes that reach its own stacks, held
///   to −3..=3. A cap (the Spirit of Oppression) is not read: the hero's own or the other's
///   (CLT-Q42);
/// - luck: 0 on Cursed Ground; else, when a luck cap reaches the side (the Hourglass of the Evil
///   Hour, worn by either hero: 0x4E36F4, 0x4E3731), the smallest cap's value whatever the
///   changes (the original's 0, a Fountain of Fortune's −1 too); else the hero's luck changes
///   that reach its own stacks, held to −3..=3. The other hero's changes are not read.
///
/// Both 0 without a hero. The original's cheats (+500 by the status flags' bits 22 and 23, luck's
/// passing the field and the Hourglass) have no source in the observation.
#[must_use]
pub fn values(observation: &Observation, side: Side, field: Field) -> (i32, i32) {
    if field == Field::Cursed {
        return (0, 0);
    }
    let index = usize::from(side == Side::Defender);
    let own = &observation.morale_and_luck[index];
    let other = &observation.morale_and_luck[1 - index];
    let to_own = |a: Affects| matches!(a, Affects::Own | Affects::Both);
    let to_enemy = |a: Affects| matches!(a, Affects::Enemy | Affects::Both);
    let mut morale = 0;
    let mut luck = 0;
    for effect in own {
        match *effect {
            MoraleLuck::Morale {
                amount, affects, ..
            } if to_own(affects) => morale += i32::from(amount),
            MoraleLuck::Luck {
                amount, affects, ..
            } if to_own(affects) => luck += i32::from(amount),
            _ => {}
        }
    }
    let cap = own
        .iter()
        .filter_map(|e| match *e {
            MoraleLuck::LuckAtMost { value, affects, .. } if to_own(affects) => Some(value),
            _ => None,
        })
        .chain(other.iter().filter_map(|e| match *e {
            MoraleLuck::LuckAtMost { value, affects, .. } if to_enemy(affects) => Some(value),
            _ => None,
        }))
        .min();
    let luck = cap.map_or_else(|| luck.clamp(-3, 3), i32::from);
    (morale.clamp(-3, 3), luck)
}

/// The side whose hero's window the right button at `point` (on the screen) shows in
/// `observation`: over a side's hero's rectangle ("hexes" 0xFC, 0xFD; [`spot`]) while that side
/// has a hero; none elsewhere.
#[must_use]
pub fn at(layout: &BattleLayout, observation: &Observation, point: [i32; 2]) -> Option<Side> {
    let Spot::Hero(side) = spot(layout, point, false) else {
        return None;
    };
    observation.sides[usize::from(side == Side::Defender)]
        .hero
        .is_some()
        .then_some(side)
}

/// Side `side`'s window showing `info`, its words from `catalogue` (`army_info_hero_set`
/// 0x46CC10): at the side's place, no shadow.
#[must_use]
pub fn window(
    layout: &HeroInfoLayout,
    catalogue: &Catalogue,
    side: Side,
    info: &Info,
) -> MessageBox {
    let mut dialog = layout.dialog.clone();
    dialog.place = layout.places[usize::from(side == Side::Defender)];
    let word = |key: &str| catalogue.text(key, &Args::new());
    let label = |key: &str| {
        catalogue.text(
            "battle.hero-info.label",
            &Args::new().text("word", &word(key)),
        )
    };
    let mut texts = BTreeMap::new();
    for (k, (key, value)) in SKILL_WORDS.iter().zip(info.skills).enumerate() {
        let id = SKILLS + 2 * u16::try_from(k).unwrap_or(0);
        let value = if k < 2 {
            value.min(99)
        } else {
            value.clamp(1, 99)
        };
        texts.insert(id, label(key));
        texts.insert(id + 1, format!("{value}"));
    }
    texts.insert(MORALE_LABEL, label("battle.hero-info.morale"));
    texts.insert(LUCK_LABEL, label("battle.hero-info.luck"));
    let points = Args::new()
        .text("word", &word("battle.hero-info.spell-points"))
        .number("points", i64::from(info.mana))
        .number("most", i64::from(info.max_mana));
    texts.insert(POINTS, catalogue.text("battle.hero-info.points", &points));
    let frame = |v: i32| u32::try_from(v.clamp(-3, 3) + 3).unwrap_or(3);
    let mut shown = Shown {
        texts,
        frames: [(MORALE, frame(info.morale)), (LUCK, frame(info.luck))].into(),
        players: [(BACKGROUND, info.player)].into(),
        ..Shown::default()
    };
    match &info.portrait {
        Some(portrait) => {
            shown.pictures.insert(PORTRAIT, portrait.clone());
        }
        None => {
            shown.states.insert(
                PORTRAIT,
                State {
                    hidden: true,
                    ..State::default()
                },
            );
        }
    }
    MessageBox { dialog, shown }
}

#[cfg(test)]
mod tests;
