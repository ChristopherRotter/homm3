//! The battle log in the adapters (E17 M3 T4b-2b-2, `docs/epics/E17-design.md` §9.6): the
//! catalogue the viewer's lines come from (the key catalogue and the original's pack from the
//! user's text tables), and the text the log's item shows after a record's first strike, for the
//! frame check (`h3-view sequences --frames`, a sequence's `log_at`). *Provisional*: one strike
//! entering an empty log, its stacks named as the opening's snapshot has them; which event writes
//! which line, and the log over a whole battle, are M3 T6's.

use h3_assets::fnt::Font;
use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Event, ServerMessage};
use h3_render::Bugs;
use h3_view::battle_log::{self, Hit, Spread, Stack};
use h3_view::record::Record;
use h3_view::tables::Tables;
use h3_view::text::pack::Pack;
use h3_view::text::{Catalogue, encode_1252};

use crate::render::lookup;
use crate::text;

/// The catalogue of the original's text: the committed key catalogue over `tables`' creatures, its
/// pack read from the text tables of `archives` (the base archive's copies first, §3.1). A key the
/// tables do not give is an error: the gate lists such problems.
pub fn catalogue(
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<Catalogue, String> {
    let keys = text::read_keys(&text::default_keys(), tables)?;
    let texts = text::tables(&keys, |name| lookup(archives, name))?;
    let (original, problems) = Pack::original(&keys, &texts);
    match problems.first() {
        Some(problem) => Err(format!("the original's text: {problem}")),
        None => Ok(Catalogue {
            packs: vec![original],
        }),
    }
}

/// The line of the record's first strike (`battle_log::strike`): the first command's events with a
/// `Struck`, its hits those of the first striker up to another's (the retaliation), at one target
/// and the stack behind it (`Spread::Target`); each stack named by its creature and, the striker,
/// by its number in the opening's snapshot. None without a strike.
pub fn first_strike(record: &Record, catalogue: &Catalogue) -> Option<String> {
    let snapshot = record.first_snapshot()?;
    let keys = &record.header.manifest.keys.creatures;
    let struck = |e: &Event| match *e {
        Event::Struck {
            attacker,
            target,
            ref damage,
            kills,
            ..
        } => Some((attacker, target, damage.0, kills)),
        _ => None,
    };
    let events = record.messages.iter().find_map(|m| match m {
        ServerMessage::Events { events, .. } if events.iter().any(|e| struck(e).is_some()) => {
            Some(events)
        }
        _ => None,
    })?;
    let strikes: Vec<_> = events.iter().filter_map(struck).collect();
    let striker = strikes.first()?.0;
    let stack = |id: u8| snapshot.stacks.iter().find(|s| s.id == id);
    let name = |id: u8| {
        stack(id)
            .and_then(|s| keys.get(usize::from(s.creature)))
            .map(|key| format!("creature.{key}"))
    };
    let named: Vec<(String, bool, u64, u32)> = strikes
        .iter()
        .take_while(|s| s.0 == striker)
        .map(|&(_, target, damage, kills)| {
            let machine = stack(target).is_some_and(|s| s.war_machine);
            Some((name(target)?, machine, damage, kills))
        })
        .collect::<Option<_>>()?;
    let hits: Vec<Hit<'_>> = named
        .iter()
        .map(|(creature, war_machine, damage, kills)| Hit {
            creature,
            war_machine: *war_machine,
            damage: *damage,
            kills: *kills,
        })
        .collect();
    let attacker = name(striker)?;
    let count = stack(striker)?.count;
    Some(battle_log::strike(
        catalogue,
        Stack {
            creature: &attacker,
            count,
        },
        Spread::Target,
        &hits,
    ))
}

/// The log item's text after `line` entered an empty log (0x472E90, 0x472C30): its entries as
/// `font` measures them at `width`, the first shown.
pub fn after_one(line: &str, font: &Font, width: i32, bugs: Bugs) -> String {
    let lines = |text: &str| {
        h3_render::text::line_count(font, &encode_1252(text).0, i64::from(width), bugs)
    };
    battle_log::shown(&battle_log::entries(line, &lines), 0)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use h3_protocol::wire::{Command, Seat};
    use h3_view::record;

    use super::*;

    const PACK: &str = r#"{
        "battle": { "log": {
            "damage_one": "{{attacker}} hits for {{damage}}.",
            "damage_other": "{{attacker}} hit for {{damage}}.",
            "perish_one": "\nA {{creature}} falls.",
            "perish_other": "\n{{count}} {{creature}} fall."
        } },
        "creature": {
            "base:archangel_one": "Archangel", "base:pikeman_other": "Pikemen",
            "base:goblin_one": "Goblin"
        }
    }"#;

    fn played(steps: &[(Seat, Command)]) -> Record {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let text = crate::scenario::played(
            &manifest.join("../../situations/seq-strike.toml"),
            &manifest.join("../../../packs"),
            1,
            steps,
        )
        .unwrap();
        record::parse(&text).unwrap()
    }

    #[test]
    fn the_first_strike_is_named_by_the_opening() {
        let catalogue = Catalogue {
            packs: vec![Pack::from_json("en", PACK).unwrap()],
        };
        // The Archangel's strike in `seq-strike.toml` (its sequence's command): all ten Pikemen.
        let strike = Command::Attack {
            target: 3,
            from: 47,
        };
        let record = played(&[(Seat::Attacker, strike)]);
        assert_eq!(
            first_strike(&record, &catalogue).as_deref(),
            Some("Archangel hits for 100.\n10 Pikemen fall.")
        );
        // A record without a strike has no line.
        let record = played(&[(Seat::Attacker, Command::Defend {})]);
        assert_eq!(first_strike(&record, &catalogue), None);
    }

    #[test]
    fn a_line_shows_joined_when_it_fits() {
        use h3_assets::fnt::Glyph;
        // Every glyph one pixel wide: "Hit. Fall." takes 10.
        let glyph = Glyph {
            left: 0,
            width: 1,
            right: 0,
            pixels: vec![0],
        };
        let font = Font {
            height: 1,
            y_offset: 0,
            glyphs: vec![glyph; 256],
        };
        assert_eq!(
            after_one("Hit.\nFall.", &font, 10, Bugs::Original),
            "Hit. Fall."
        );
        assert_eq!(
            after_one("Hit.\nFall.", &font, 9, Bugs::Original),
            "Hit.\nFall."
        );
        assert_eq!(after_one("Hit.", &font, 2, Bugs::Fixed), "Hit.");
    }
}
