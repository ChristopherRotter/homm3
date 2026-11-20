//! The kept live parts hold no text of the original (E24 M4, the labels task; ADR-0009): each
//! box's texts and captions (`[[message]]`), each level-up's texts (`[[level_up]]`) and each
//! find's text (`[[found]]`) are written by the harness as the records they were made of
//! (`h3lod.label`): `[ADVEVENT:86]`, two neighbours' `[ADVEVENT:89+90]`, a name's field
//! `[CRTRAITS:3.1]`, `[other: n characters]` and `[the map's text]`, between them only numbers
//! and punctuation. Nothing of the viewer reads them: it composes the boxes from the catalogue.

use std::fs;
use std::path::Path;

/// What a kept text may hold between its labels (`h3lod.KEPT`).
const KEPT: &str = " \t\r\n0123456789+-,.:;()%/'\"!?";

/// Whether `text` holds labels and [`KEPT`] characters only.
fn labelled(text: &str) -> bool {
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if c == '[' {
            let Some(end) = rest.find(']') else {
                return false;
            };
            if !label(&rest[1..end]) {
                return false;
            }
            rest = &rest[end + 1..];
        } else if KEPT.contains(c) {
            rest = &rest[c.len_utf8()..];
        } else {
            return false;
        }
    }
    true
}

/// A label's inside: `TABLE:record` (two neighbours' `n+m`, a name's field `n.f`), `other: n
/// characters` or `the map's text`.
fn label(inside: &str) -> bool {
    if inside == "the map's text" {
        return true;
    }
    if let Some(n) = inside
        .strip_prefix("other: ")
        .and_then(|s| s.strip_suffix(" characters"))
    {
        return digits(n);
    }
    let Some((table, record)) = inside.split_once(':') else {
        return false;
    };
    let (record, pair) = record.split_once('+').unwrap_or((record, "0"));
    let (row, field) = record.split_once('.').unwrap_or((record, "0"));
    let mut name = table.chars();
    name.next().is_some_and(|c| c.is_ascii_uppercase())
        && name.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && [row, field, pair].into_iter().all(digits)
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// A live part's kept texts: its boxes' texts and captions (a picture's third field), its
/// level-ups' texts and its finds' text.
fn kept_texts(part: &toml::Value) -> Vec<String> {
    let tables = |key: &str| {
        part.get(key)
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let strings = |value: Option<&toml::Value>| -> Vec<String> {
        value
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect()
    };
    let mut out = Vec::new();
    for message in tables("message") {
        out.extend(strings(message.get("texts")));
        for picture in message["pictures"].as_array().unwrap() {
            out.extend(picture.get(2).map(|c| c.as_str().unwrap().to_owned()));
        }
    }
    for level_up in tables("level_up") {
        out.extend(strings(level_up.get("texts")));
    }
    for found in tables("found") {
        out.push(found["text"].as_str().unwrap().to_owned());
    }
    out
}

#[test]
fn the_kept_live_parts_hold_labels_not_texts() {
    let dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../map/timelines"));
    let (mut parts, mut texts) = (0, 0);
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if !name.ends_with("-live.toml") {
            continue;
        }
        let part: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let kept = kept_texts(&part);
        for text in &kept {
            assert!(labelled(text), "{name}: a text, not its records: {text:?}");
        }
        parts += 1;
        texts += kept.len();
    }
    // At the labels task: 98 live parts, 42 of them with boxes, level-ups or finds (353 texts).
    assert!(
        parts >= 98 && texts >= 353,
        "{parts} live parts, {texts} texts"
    );
}

#[test]
fn a_text_of_its_own_is_refused() {
    for text in [
        "",
        "1000",
        "+2 [PRISKILL:0]",
        "[ADVEVENT:89+90]",
        "[CRTRAITS:3.1]",
        "[the map's text]",
        "[other: 30 characters]",
        "[SKILLLEV:1]\n[SSTRAITS:25]",
        "[ADVEVENT:127] '[SPTRAITS:61]'.",
        "[CRGEN1:4]",
    ] {
        assert!(labelled(text), "{text:?}");
    }
    for text in [
        "A sign of our own",
        "5 of our own",
        "[A sign]",
        "[GENRLTXT:5",
        "[GENRLTXT]",
        "[GENRLTXT:]",
        "[GENRLTXT:5.]",
        "[genrltxt:5]",
        "[GENRL-TXT:5]",
        "[:5]",
        "[other: five characters]",
        "[the map's own text]",
    ] {
        assert!(!labelled(text), "{text:?}");
    }
}
