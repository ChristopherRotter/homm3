//! Eagle Eye's message after the battle's result window (`eagle_eye_message` 0x476FE0 from
//! `battle_end` 0x477951; E17 M6 T5c, `docs/epics/E17-design.md` §12.21): when the window showed
//! the winner (its player a human here) and the winner has a hero, the spells its Eagle Eye picked
//! in the battle, by their number in the original, in boxes of at most eight pictures
//! ([`crate::message::pictures`]) shown one after another, each a sentence of its own spells:
//! GENRLTXT 222 with the hero's name and the box's first spell, each later one after ", "
//! (0x66032C), the set's last or the box's eighth after GENRLTXT 142 (" and "), then "."
//! (0x6603EC). Each box starts a pickup sound as it opens ([`pickups`]). Every pick is named, the
//! spells the hero did not learn (no book, a level above its Wisdom + 2) too (CLT-Q65).

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};

use crate::message::{Picture, SLOTS};

/// A spell the hero's Eagle Eye picked in the battle (the outcome's `learned`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    /// Its number in the original: its picture's frame (`spellScr.def`) and the boxes' order (the
    /// executable's set is sorted by it).
    pub original: u32,
    /// Its name.
    pub name: String,
}

/// A box of Eagle Eye's message: its text and its spells' pictures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// The sentence naming the box's spells.
    pub text: String,
    /// The spells' pictures in order (type 9, `message_box_picture` 0x4F4EB0: the spells' DEF,
    /// its frame the spell's number, the caption its name).
    pub pictures: Vec<Picture>,
}

/// Eagle Eye's boxes after the result window: none when the window showed a side that did not
/// win (`won` false: the loser's window, or no winner named; nothing follows it, 0x477964), when
/// that side has no hero (`hero`: its name) or when its Eagle Eye picked nothing (`picks`); else
/// the picks by their original number, at most [`SLOTS`] a box, their pictures from the spells'
/// DEF `def`, `size` its frames' size.
#[must_use]
pub fn boxes(
    catalogue: &Catalogue,
    (won, hero): (bool, Option<&str>),
    picks: &[Pick],
    (def, size): (&str, [i32; 2]),
) -> Vec<Told> {
    let Some(hero) = hero.filter(|_| won) else {
        return Vec::new();
    };
    let mut picks: Vec<&Pick> = picks.iter().collect();
    picks.sort_by(|a, b| (a.original, &a.name).cmp(&(b.original, &b.name)));
    let plain = |key: &str| catalogue.text(key, &Args::new());
    let (comma, and, stop) = (
        plain("result.eagle-eye.comma"),
        plain("result.eagle-eye.and"),
        plain("result.eagle-eye.stop"),
    );
    let total = picks.len();
    let mut out = Vec::new();
    for (index, chunk) in picks.chunks(SLOTS).enumerate() {
        let mut text = String::new();
        for (i, pick) in chunk.iter().enumerate() {
            if i == 0 {
                let args = Args::new().text("hero", hero).text("spell", &pick.name);
                text = catalogue.text("result.eagle-eye.learnt", &args);
                continue;
            }
            // " and " before the set's last and the box's eighth (0x477106), else ", ".
            let last = index * SLOTS + i + 1 == total;
            text.push_str(if last || i + 1 == SLOTS { &and } else { &comma });
            text.push_str(&pick.name);
        }
        text.push_str(&stop);
        let pictures = chunk
            .iter()
            .map(|pick| Picture {
                def: String::from(def),
                frame: pick.original,
                size,
                caption: pick.name.clone(),
            })
            .collect();
        out.push(Told { text, pictures });
    }
    out
}

/// The pickup sounds of `count` boxes (`pickup0N.82M`, N from 1 to 7: the asset tables'
/// `pickup_sounds`), one drawn as each box opens. *Provisional:* the executable draws
/// `random_between(1, 7)` from the battle's generator (0x4771BA), which the engine does not share;
/// the viewer draws from the C library's `rand()` sequence started at the battle's `seed`, as the
/// opening's sounds ([`crate::sound::opening`]).
#[must_use]
pub fn pickups(seed: u32, count: usize) -> Vec<u8> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            let n = 1 + crate::sound::rand(&mut state) % 7;
            u8::try_from(n).unwrap_or(1)
        })
        .collect()
}

#[cfg(test)]
mod tests;
