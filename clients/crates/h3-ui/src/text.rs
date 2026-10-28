//! The string catalogue (E17 M3 T4b-1; the user's decisions of 2026-10-13 and 2026-10-19;
//! `docs/epics/E17-design.md` §9): every text the client shows is a key's message, looked up in
//! language packs by priority. [`keys`] is the committed catalogue of keys (`clients/text/keys.toml`:
//! a key's arguments and where the original's text tables hold its forms), [`pack`] a language pack
//! (nested JSON, i18next's conventions; or the original's, built at run time from the user's text
//! tables), [`message`] a message's text and placeholders, [`plural`] which form a count takes.
//!
//! The original's text is never committed (ADR-0009); a translation of it is a pack the player
//! supplies, which the project neither ships nor hosts (CLT-Q7, the user, 2026-10-19). Strings stay
//! Unicode until they are drawn: [`encode_1252`] gives the bytes the installation's fonts draw.

pub mod keys;
pub mod message;
pub mod pack;
pub mod plural;

#[cfg(test)]
mod tests;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pack::Pack;
use plural::Category;

/// An argument's value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A text, as given.
    Text(String),
    /// A whole number, written in decimal digits (a `-` before a negative one), as `%d` does.
    Number(i64),
}

/// A message's arguments: values by name, and the context that picks a form (`male`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    values: Vec<(String, Value)>,
    context: Option<String>,
}

impl Args {
    /// No arguments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// With a text argument.
    #[must_use]
    pub fn text(mut self, name: &str, value: &str) -> Self {
        self.values.push((name.into(), Value::Text(value.into())));
        self
    }

    /// With a number argument.
    #[must_use]
    pub fn number(mut self, name: &str, value: i64) -> Self {
        self.values.push((name.into(), Value::Number(value)));
        self
    }

    /// With the number `count`, which picks a plural key's form.
    #[must_use]
    pub fn count(self, value: i64) -> Self {
        self.number("count", value)
    }

    /// With the context, which picks a form and is the placeholder `{{context}}`'s value.
    #[must_use]
    pub fn context(mut self, value: &str) -> Self {
        self.context = Some(value.into());
        self
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.values
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    fn written(&self, name: &str) -> Option<String> {
        match self.get(name) {
            Some(Value::Text(t)) => Some(t.clone()),
            Some(Value::Number(n)) => Some(n.to_string()),
            None if name == "context" => self.context.clone(),
            None => None,
        }
    }

    fn count_value(&self) -> Option<i64> {
        match self.get("count") {
            Some(&Value::Number(n)) => Some(n),
            _ => None,
        }
    }
}

/// The packs a client looks texts up in, the first that has a key's message winning (the
/// player's pack, the client's own, the original's).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalogue {
    /// The packs by priority.
    pub packs: Vec<Pack>,
}

impl Catalogue {
    /// The text of `key` with `args`: the first pack's message among the names a key's form may
    /// have, tried in this order: `<key>_<context>_<category>`, `<key>_<context>_other`,
    /// `<key>_<context>`, `<key>_<category>`, `<key>_other`, `<key>` (the context's names only
    /// with a context, the categories' only with a `count`, by the pack's rule). A key no pack
    /// has is shown as itself.
    #[must_use]
    pub fn text(&self, key: &str, args: &Args) -> String {
        let count = args.count_value();
        for pack in &self.packs {
            let category = count.map(|n| pack.rule.category(n));
            let found = names(key, args.context.as_deref(), category)
                .iter()
                .find_map(|name| pack.messages.get(name));
            if let Some(message) = found {
                return message.write(|name| args.written(name));
            }
        }
        key.to_string()
    }
}

/// The names a key's form may have, in the order [`Catalogue::text`] tries them.
fn names(key: &str, context: Option<&str>, category: Option<Category>) -> Vec<String> {
    let bases = context
        .map(|c| format!("{key}_{c}"))
        .into_iter()
        .chain(core::iter::once(key.to_string()));
    let mut names = Vec::new();
    for base in bases {
        if let Some(c) = category {
            names.push(format!("{base}_{}", c.name()));
            if c != Category::Other {
                names.push(format!("{base}_other"));
            }
        }
        names.push(base);
    }
    names
}

/// Windows-1252's characters for the bytes 0x80–0x9F (an undefined byte as the C1 control of its
/// value, as `h3-formats` decodes the text tables, so that every byte round-trips).
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// A text as the bytes of Windows-1252, the code page of the installation's fonts (`TextDraw`
/// draws a byte's glyph, §3.6), and the characters it has no byte for, each written as `?`.
#[must_use]
pub fn encode_1252(text: &str) -> (Vec<u8>, Vec<char>) {
    let mut bytes = Vec::with_capacity(text.len());
    let mut missing = Vec::new();
    for c in text.chars() {
        let byte = match u32::from(c) {
            n @ (0..=0x7F | 0xA0..=0xFF) => u8::try_from(n).ok(),
            _ => CP1252_HIGH
                .iter()
                .position(|&h| h == c)
                .and_then(|i| u8::try_from(0x80 + i).ok()),
        };
        bytes.push(byte.unwrap_or_else(|| {
            missing.push(c);
            b'?'
        }));
    }
    (bytes, missing)
}
