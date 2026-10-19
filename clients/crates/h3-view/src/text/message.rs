//! A message (E17 M3 T4b-1, `docs/epics/E17-design.md` §9.2): literal text and named
//! placeholders. A language pack writes a placeholder as `{{name}}` (i18next's interpolation,
//! spaces inside allowed); every other brace is literal text, so the original's highlight braces
//! (`{Tactics Phase}`, the text drawer's markup, §3.6) stay as they are. The original's rows write
//! `%s` and `%d` in order and `%%` for a percent sign; [`Message::from_printf`] names them.

use alloc::string::String;
use alloc::vec::Vec;

/// A part of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Text as written.
    Literal(String),
    /// An argument's value by its name.
    Placeholder(String),
}

/// A message: its segments in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Message {
    /// The segments; consecutive literals are merged.
    pub segments: Vec<Segment>,
}

/// An argument's kind, as the original's `printf` specifier says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `%s`: a text (a name).
    Text,
    /// `%d`: a whole number.
    Number,
}

/// What is wrong with an original row as the key names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintfError {
    /// A `%` not followed by `s`, `d` or `%` (the character after it; `None` at the end).
    Specifier(Option<char>),
    /// More specifiers than the key names (the row's count, the key's).
    TooMany(usize, usize),
    /// Fewer specifiers than the key names (the row's count, the key's).
    TooFew(usize, usize),
    /// The specifier's kind differs from the argument's (the argument's name, the row's kind).
    Kind(String, Kind),
}

impl Message {
    /// Parses a pack's message (`{{name}}` placeholders; a name starts with a lower-case letter
    /// and goes on with lower-case letters, digits and `_`).
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut message = Self::default();
        let mut rest = text;
        while let Some(open) = rest.find("{{") {
            let (before, after) = rest.split_at(open);
            message.literal(before);
            match placeholder(&after[2..]) {
                Some((name, tail)) => {
                    message.segments.push(Segment::Placeholder(name.into()));
                    rest = tail;
                }
                None => {
                    // Not a placeholder: the first brace is text, the second may open one.
                    message.literal("{");
                    rest = &after[1..];
                }
            }
        }
        message.literal(rest);
        message
    }

    /// An original row with its `printf` specifiers named in order (`%s` and `%d`; `%%` is a
    /// percent sign), each checked against the kind `kind` gives for its name.
    ///
    /// # Errors
    /// The first problem found, as [`PrintfError`].
    pub fn from_printf(
        row: &str,
        names: &[String],
        kind: impl Fn(&str) -> Option<Kind>,
    ) -> Result<Self, PrintfError> {
        let mut message = Self::default();
        let mut used = 0;
        let mut chars = row.char_indices();
        let mut start = 0;
        while let Some((at, c)) = chars.next() {
            if c != '%' {
                continue;
            }
            message.literal(&row[start..at]);
            let spec = match chars.next().map(|(_, s)| s) {
                Some('%') => {
                    message.literal("%");
                    start = at + 2;
                    continue;
                }
                Some('s') => Kind::Text,
                Some('d') => Kind::Number,
                other => return Err(PrintfError::Specifier(other)),
            };
            let name = names
                .get(used)
                .ok_or(PrintfError::TooMany(used + 1, names.len()))?;
            if kind(name) != Some(spec) {
                return Err(PrintfError::Kind(name.clone(), spec));
            }
            message.segments.push(Segment::Placeholder(name.clone()));
            used += 1;
            start = at + 2;
        }
        message.literal(&row[start..]);
        if used < names.len() {
            return Err(PrintfError::TooFew(used, names.len()));
        }
        Ok(message)
    }

    /// The names of its placeholders, in order.
    pub fn placeholders(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().filter_map(|s| match s {
            Segment::Placeholder(name) => Some(name.as_str()),
            Segment::Literal(_) => None,
        })
    }

    /// Writes the message, each placeholder by `value` (a placeholder without a value stays
    /// visible as `{{name}}`).
    #[must_use]
    pub fn write(&self, value: impl Fn(&str) -> Option<String>) -> String {
        let mut out = String::new();
        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => out.push_str(text),
                Segment::Placeholder(name) => match value(name) {
                    Some(v) => out.push_str(&v),
                    None => {
                        out.push_str("{{");
                        out.push_str(name);
                        out.push_str("}}");
                    }
                },
            }
        }
        out
    }

    fn literal(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Some(Segment::Literal(last)) = self.segments.last_mut() {
            last.push_str(text);
        } else {
            self.segments.push(Segment::Literal(text.into()));
        }
    }
}

/// A placeholder's name and the text after its closing braces, from the text after `{{`.
fn placeholder(after: &str) -> Option<(&str, &str)> {
    let close = after.find("}}")?;
    let name = after[..close].trim_matches(' ');
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    valid.then(|| (name, &after[close + 2..]))
}
