//! Namespaced content keys (`pack:name`, ADR-0006: "IDs in data are namespaced strings").

use alloc::string::String;
use core::fmt;

/// Error for malformed keys or pack ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError {
    /// The offending text.
    pub text: String,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a valid key: expected `pack:name` with each part made of a-z, 0-9, `_`, `-`",
            self.text
        )
    }
}

/// Whether `s` is a valid key part (pack id or local name): non-empty, `[a-z0-9_-]`.
pub fn is_valid_part(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// A validated `pack:name` key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentKey {
    text: String,
    colon: usize,
}

impl ContentKey {
    /// Parses `pack:name`.
    pub fn parse(text: &str) -> Result<Self, KeyError> {
        let err = || KeyError {
            text: String::from(text),
        };
        let (pack, name) = text.split_once(':').ok_or_else(err)?;
        if is_valid_part(pack) && is_valid_part(name) {
            Ok(Self {
                text: String::from(text),
                colon: pack.len(),
            })
        } else {
            Err(err())
        }
    }

    /// Builds `pack:name` from its parts.
    pub fn new(pack: &str, name: &str) -> Result<Self, KeyError> {
        let mut text = String::from(pack);
        text.push(':');
        text.push_str(name);
        Self::parse(&text)
    }

    /// The pack part.
    pub fn pack(&self) -> &str {
        self.text.get(..self.colon).unwrap_or_default()
    }

    /// The name part.
    pub fn name(&self) -> &str {
        self.text.get(self.colon + 1..).unwrap_or_default()
    }

    /// The full key.
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for ContentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::ToString;

    #[test]
    fn parses_valid_keys() {
        let k = ContentKey::parse("synthetic:spear-man_2").unwrap();
        assert_eq!(k.pack(), "synthetic");
        assert_eq!(k.name(), "spear-man_2");
        assert_eq!(k.as_str(), "synthetic:spear-man_2");
        assert_eq!(k.to_string(), "synthetic:spear-man_2");
        assert_eq!(
            ContentKey::new("a", "b").unwrap(),
            ContentKey::parse("a:b").unwrap()
        );
    }

    #[test]
    fn rejects_invalid_keys() {
        for bad in ["", "nocolon", ":x", "x:", "A:b", "a:b:c", "a :b", "a:é"] {
            let e = ContentKey::parse(bad).unwrap_err();
            assert_eq!(e.text, bad);
        }
        assert!(
            ContentKey::parse("a:B")
                .unwrap_err()
                .to_string()
                .contains("`a:B` is not a valid key")
        );
        assert!(!is_valid_part(""));
        assert!(is_valid_part("x-1_y"));
    }
}
