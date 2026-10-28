//! Language packs (E17 M3 T4b-1, `docs/epics/E17-design.md` §9.2): messages by name, a key's
//! name with its form's suffix (`battle.log.damage_one`), and the plural rule that picks a form.
//!
//! A pack file is nested JSON with i18next's conventions (the user, 2026-10-19): objects nest the
//! key's segments, a string is a message, `{{name}}` a placeholder, `_<category>` and
//! `_<context>` suffixes the forms; its language is its file's name (`de.json`). The original's
//! pack is built from the user's text tables through the key catalogue ([`Pack::original`]).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use serde_json::Value;

use super::keys::{Keys, Resolved, Row, Source, address_field};
use super::message::Message;
use super::plural::Rule;

/// The user's text tables by name without `.TXT` (`GENRLTXT`): each its records' fields.
pub type Tables = BTreeMap<String, Vec<Vec<String>>>;

/// A language pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    /// Its plural rule.
    pub rule: Rule,
    /// Its messages by name.
    pub messages: BTreeMap<String, Message>,
}

/// What is wrong with a pack's entry or a key's original form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The entry's name.
    pub name: String,
    /// What.
    pub what: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`: {}", self.name, self.what)
    }
}

/// Why a pack file cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// No plural rule for the language.
    Language(String),
    /// Not JSON (the parser's message).
    Json(String),
    /// An entry that is neither an object nor a string (its name).
    Entry(String),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Language(l) => write!(f, "no plural rule for the language `{l}`"),
            Self::Json(m) => write!(f, "not JSON: {m}"),
            Self::Entry(name) => write!(f, "`{name}` is neither an object nor a string"),
        }
    }
}

impl Pack {
    /// Reads a pack file in `language` (a BCP 47 tag; its primary subtag picks the plural rule).
    ///
    /// # Errors
    /// [`PackError`] for a language without a rule, text that is not JSON, and an entry that is
    /// neither an object nor a string.
    pub fn from_json(language: &str, json: &str) -> Result<Self, PackError> {
        let rule = Rule::of(language).ok_or_else(|| PackError::Language(language.into()))?;
        let value: Value =
            serde_json::from_str(json).map_err(|e| PackError::Json(e.to_string()))?;
        let mut messages = BTreeMap::new();
        flatten(String::new(), &value, &mut messages)?;
        Ok(Self { rule, messages })
    }

    /// The original's pack: every original form of `keys` read from the user's text tables
    /// (`tables`: each table's records by its name without `.TXT`, as the parser reads them), with
    /// the executable's plural rule; and the forms that could not be read.
    #[must_use]
    pub fn original(keys: &Keys, tables: &Tables) -> (Self, Vec<Problem>) {
        let mut pack = Self {
            rule: Rule::Original,
            messages: BTreeMap::new(),
        };
        let mut problems = Vec::new();
        for (id, key) in &keys.keys {
            for (form, source, printf) in &key.original {
                let name = if form.is_empty() {
                    id.clone()
                } else {
                    format!("{id}_{form}")
                };
                match cell(source, tables).and_then(|text| {
                    // A table's field as the executable reads it: a run of quotes is one
                    // (0x5BBD29; the executable's own strings have none).
                    let text = match source.row {
                        Row::Address(_) => String::from(text),
                        _ => one_quote(text),
                    };
                    if source.raw {
                        return Ok(Message::literal_text(&text));
                    }
                    Message::from_printf(&text, printf, |n| key.args.get(n).copied())
                        .map_err(|e| format!("{e:?} in {text:?}"))
                }) {
                    Ok(message) => {
                        pack.messages.insert(name, message);
                    }
                    Err(what) => problems.push(Problem { name, what }),
                }
            }
        }
        (pack, problems)
    }

    /// Its entries that no key of `keys` names, and its placeholders that a key's arguments lack.
    #[must_use]
    pub fn check(&self, keys: &Keys) -> Vec<Problem> {
        let mut known: BTreeMap<String, &Resolved> = BTreeMap::new();
        for (id, key) in &keys.keys {
            for name in Keys::names(id, key) {
                known.insert(name, key);
            }
        }
        let mut problems = Vec::new();
        for (name, message) in &self.messages {
            let Some(key) = known.get(name) else {
                problems.push(Problem {
                    name: name.clone(),
                    what: "no key has this name".into(),
                });
                continue;
            };
            for p in message.placeholders() {
                if !key.args.contains_key(p) && !(p == "context" && !key.contexts.is_empty()) {
                    problems.push(Problem {
                        name: name.clone(),
                        what: format!("`{{{{{p}}}}}` is not an argument"),
                    });
                }
            }
        }
        problems
    }
}

/// `text` with each run of quotes made one, as the executable's text tables' reader does once it
/// has cut a quoted field's closing quote (`text_table_records` 0x5BBD29 to 0x5BBD6A, and its
/// columns' kin 0x5BC0AE; E17 M5 T4a: GENRLTXT's record 33 quotes the surrendering's terms as
/// `""…""`). `h3_formats`' table parser keeps them as the file has them.
fn one_quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut quote = false;
    for c in text.chars() {
        if !(c == '"' && quote) {
            out.push(c);
        }
        quote = c == '"';
    }
    out
}

/// A source's text in the user's tables.
fn cell<'t>(source: &Source, tables: &'t Tables) -> Result<&'t str, String> {
    let name = &source.table;
    let records = tables
        .get(name)
        .ok_or_else(|| format!("no table {name}.TXT"))?;
    let row = match source.row {
        Row::Record(r) => records.get(r),
        Row::Address(a) => {
            let field = address_field(a);
            records.iter().find(|r| r.first() == Some(&field))
        }
        Row::Id { first, id, by } => records
            .iter()
            .skip(first)
            .filter(|r| match by {
                Some(column) => r.get(column).is_some_and(|f| !f.trim().is_empty()),
                None => !r.iter().all(|f| f.trim().is_empty()),
            })
            .nth(id),
    };
    let row = row.ok_or_else(|| format!("{name}.TXT has no such record ({:?})", source.row))?;
    row.get(source.column)
        .map(String::as_str)
        .ok_or_else(|| format!("{name}.TXT's record has no column {}", source.column))
}

/// Flattens nested objects into names joined by `.`; strings are messages.
fn flatten(
    prefix: String,
    value: &Value,
    messages: &mut BTreeMap<String, Message>,
) -> Result<(), PackError> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let name = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten(name, v, messages)?;
            }
            Ok(())
        }
        Value::String(s) => {
            messages.insert(prefix, Message::parse(s));
            Ok(())
        }
        _ => Err(PackError::Entry(prefix)),
    }
}
