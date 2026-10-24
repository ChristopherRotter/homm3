//! The battle log and the panel in the adapters (E17 M3 T4b-2b-2, T6b; `docs/epics/E17-design.md`
//! §9.6, §9.8): the catalogue the viewer's lines come from (the key catalogue and the original's
//! pack from the user's text tables), and the panel over a record's battle: each turn's start and
//! the text the log's item shows from each time on, its lines worded from the record's stacks as
//! the opening's snapshot names them.

use h3_assets::fnt::Font;
use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Event, ServerMessage};
use h3_render::Bugs;
use h3_view::battle_log::{self, Named};
use h3_view::layout::BattleLayout;
use h3_view::record::Record;
use h3_view::scene::Panels;
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
    exe: &[u8],
) -> Result<Catalogue, String> {
    let keys = text::read_keys(&text::default_keys(), tables)?;
    let texts = text::tables(&keys, |name| lookup(archives, name), exe)?;
    let (original, problems) = Pack::original(&keys, &texts);
    match problems.first() {
        Some(problem) => Err(format!("the original's text: {problem}")),
        None => Ok(Catalogue {
            packs: vec![original],
        }),
    }
}

/// The record's events, in order.
pub fn events(record: &Record) -> Vec<Event> {
    record
        .messages
        .iter()
        .filter_map(|m| match m {
            ServerMessage::Events { events, .. } => Some(events.iter().cloned()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// The text the log's item shows from each time on through the record's battle as its timeline's
/// `logged` has it (`battle_log::texts`): each line in `catalogue`'s words, the stacks named as the
/// opening's snapshot has them (a creature's key, whether a war machine, its defense), its
/// entries measured in `font` at `width` (the log item's, §9.4); a human side's stacks by the
/// snapshot's `human`. None without a snapshot.
pub fn texts(
    record: &Record,
    logged: &[(u32, battle_log::Logged)],
    catalogue: &Catalogue,
    font: &Font,
    width: i32,
    bugs: Bugs,
) -> Option<Vec<(u32, String)>> {
    let snapshot = record.first_snapshot()?;
    let keys = &record.header.manifest.keys.creatures;
    let stack = |id: u8| snapshot.stacks.iter().find(|s| s.id == id);
    let named = |id: u8| {
        let s = stack(id)?;
        let key = keys.get(usize::from(s.creature))?;
        Some(Named {
            creature: format!("creature.{key}"),
            war_machine: s.war_machine,
        })
    };
    // A spell's and a side's hero's keys (E17 M3 T7b-1).
    let manifest = &record.header.manifest.keys;
    let key = |name: battle_log::Name| match name {
        battle_log::Name::Spell(handle) => manifest
            .spells
            .get(usize::from(handle))
            .map(|k| format!("spell.{k}")),
        battle_log::Name::Hero(side) => {
            let hero = snapshot.sides.get(usize::from(side))?.hero.as_ref()?;
            let k = manifest.hero_types.get(usize::from(hero.hero_type))?;
            Some(format!("hero.{k}"))
        }
        // The town's moat names its damage (E17 M4 T2c-1).
        battle_log::Name::Moat => {
            let siege = snapshot.siege.as_ref()?;
            let k = manifest.factions.get(usize::from(siege.faction))?;
            Some(format!("moat.{k}"))
        }
    };
    let word = |logged: &battle_log::Logged| battle_log::word(catalogue, logged, &named, &key);
    let lines = |text: &str| {
        h3_render::text::line_count(font, &encode_1252(text).0, i64::from(width), bugs)
    };
    let human = |id: u8| {
        stack(id).is_some_and(|s| {
            snapshot.sides[usize::from(s.controller == h3_protocol::wire::Side::Defender)].human
        })
    };
    Some(battle_log::texts(logged, &word, &lines, &human))
}

/// The panel over the record's battle (E17 M3 T6b): its turns and its log's texts ([`texts`])
/// through `logged` (the timeline's log, with the mouse's hints since T6c-2:
/// `crate::pointer::shown`) in the original's words from `archives`' text tables (read only when
/// the battle writes a line), measured in the log item's font of `layout` as the text drawer
/// draws with `bugs` (the original's for the comparisons only: E17 M3 T9b, the reviewer).
pub fn panels(
    record: &Record,
    logged: &[(u32, battle_log::Logged)],
    tables: &Tables,
    layout: &BattleLayout,
    (archives, exe): (&[(&str, LodArchive<'_>)], &[u8]),
    bugs: Bugs,
) -> Result<Panels, String> {
    let writes = logged.iter().any(|(_, l)| {
        !matches!(
            l,
            battle_log::Logged::Action
                | battle_log::Logged::Turn(_)
                | battle_log::Logged::Status { .. }
                | battle_log::Logged::Scroll(_)
                | battle_log::Logged::Unmark
        )
    });
    let catalogue = if writes {
        catalogue(tables, archives, exe)?
    } else {
        Catalogue::default()
    };
    Writer::with(catalogue, layout, archives, bugs)?.panels(record, logged)
}

/// What words the log's lines and measures them, kept once the archives are gone (E17 M3
/// T6c-2: the window makes its panel again as the mouse writes hints).
#[derive(Debug)]
pub struct Writer {
    /// The original's words.
    pub catalogue: Catalogue,
    font: Font,
    width: i32,
    bugs: Bugs,
}

impl Writer {
    /// The original's words from `archives`' text tables and `exe`'s strings, the log item's
    /// font of `layout`, the lines measured as the text drawer draws with `bugs`.
    pub fn new(
        tables: &Tables,
        layout: &BattleLayout,
        (archives, exe): (&[(&str, LodArchive<'_>)], &[u8]),
        bugs: Bugs,
    ) -> Result<Self, String> {
        Self::with(catalogue(tables, archives, exe)?, layout, archives, bugs)
    }

    fn with(
        catalogue: Catalogue,
        layout: &BattleLayout,
        archives: &[(&str, LodArchive<'_>)],
        bugs: Bugs,
    ) -> Result<Self, String> {
        let item = layout.log().ok_or("the layout's panel has no log item")?;
        let font = h3_assets::fnt::decode_fnt(&lookup(archives, &item.font)?)
            .map_err(|e| format!("{}: {e:?}", item.font))?;
        Ok(Self {
            catalogue,
            font,
            width: item.size[0],
            bugs,
        })
    }

    /// The panel over the record's battle through `logged` ([`panels`]).
    pub fn panels(
        &self,
        record: &Record,
        logged: &[(u32, battle_log::Logged)],
    ) -> Result<Panels, String> {
        let snapshot = record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let texts = texts(
            record,
            logged,
            &self.catalogue,
            &self.font,
            self.width,
            self.bugs,
        )
        .ok_or("the record holds no snapshot")?;
        // Each command's action starts at its `Action` (E17 M3 T7b-1: a hero's cast's).
        let starts = logged
            .iter()
            .filter(|(_, l)| matches!(l, battle_log::Logged::Action))
            .map(|(at, _)| *at);
        let commands = record.messages.iter().filter_map(|m| match m {
            ServerMessage::Events { events, .. } => Some(events),
            _ => None,
        });
        let casts = starts
            .zip(commands)
            .filter_map(|(at, events)| {
                events.iter().find_map(|e| match *e {
                    Event::SpellCast { side, .. } => Some((at, side)),
                    _ => None,
                })
            })
            .collect();
        Ok(Panels::new(snapshot, &events(record), texts).with_casts(casts))
    }
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

    /// Every glyph one pixel wide.
    fn narrow() -> Font {
        use h3_assets::fnt::Glyph;
        let glyph = Glyph {
            left: 0,
            width: 1,
            right: 0,
            pixels: vec![0],
        };
        Font {
            height: 1,
            y_offset: 0,
            glyphs: vec![glyph; 256],
        }
    }

    #[test]
    fn a_strikes_line_is_named_by_the_opening() {
        let catalogue = Catalogue {
            packs: vec![Pack::from_json("en", PACK).unwrap()],
        };
        // The Archangel's strike in `seq-strike.toml` (its sequence's command): all ten Pikemen.
        let strike = Command::Attack {
            target: 3,
            from: 47,
        };
        let record = played(&[(Seat::Attacker, strike)]);
        let line = battle_log::Logged::Strike {
            attacker: 0,
            count: 1,
            spread: battle_log::Spread::Target,
            hits: vec![(3, 100, 10)],
        };
        let logged = [(5, battle_log::Logged::Action), (3730, line)];
        // Joined when the line fits the width, split in two entries when it does not.
        let wide = texts(&record, &logged, &catalogue, &narrow(), 400, Bugs::Original).unwrap();
        assert_eq!(
            wide,
            [
                (0, String::new()),
                (3730, "Archangel hits for 100. 10 Pikemen fall.".into())
            ]
        );
        let tight = texts(&record, &logged, &catalogue, &narrow(), 30, Bugs::Fixed).unwrap();
        assert_eq!(
            tight.last().map(|(_, t)| t.as_str()),
            Some("Archangel hits for 100.\n10 Pikemen fall.")
        );
        // A stack the opening lacks names no line; the record's events are its commands'.
        let ghost = battle_log::Logged::Wait { stack: 9, count: 1 };
        let none = texts(
            &record,
            &[(0, ghost)],
            &catalogue,
            &narrow(),
            400,
            Bugs::Original,
        );
        assert_eq!(none.unwrap(), [(0, String::new())]);
        assert!(
            events(&record)
                .iter()
                .any(|e| matches!(e, Event::Struck { .. }))
        );
    }

    #[test]
    fn a_human_turn_starts_with_the_status_line_empty() {
        let catalogue = Catalogue {
            packs: vec![Pack::from_json("en", PACK).unwrap()],
        };
        let record = played(&[(Seat::Attacker, Command::Defend {})]);
        // Both sides are human in `seq-strike.toml`: a turn 3 000 ms after a line empties the
        // item; one sooner leaves it.
        let line = battle_log::Logged::Strike {
            attacker: 0,
            count: 1,
            spread: battle_log::Spread::Target,
            hits: vec![(3, 100, 10)],
        };
        let logged = [
            (100, line),
            (200, battle_log::Logged::Turn(3)),
            (3100, battle_log::Logged::Turn(3)),
        ];
        let got = texts(&record, &logged, &catalogue, &narrow(), 400, Bugs::Original).unwrap();
        assert_eq!(got.len(), 3);
        assert_eq!(got[2], (3100, String::new()));
    }
}
