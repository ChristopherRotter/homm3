//! Across archives (E17 M1 T3, the real-asset gate's inventory): which names appear in more than
//! one archive, which names the executable or the text tables mention, and which names break the
//! archives' 8.3 pattern. The gate's report lists the results as *candidates*: a name the game
//! builds at run time (`AH%02d_.def`) is mentioned only as a pattern, and a name may be used by
//! data the survey does not read (maps, campaigns).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::archive::extension;

/// The names some text mentions: exact names and `printf`-style patterns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mentions {
    exact: BTreeSet<String>,
    patterns: BTreeSet<String>,
}

/// How a name is mentioned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mention {
    /// Literally.
    Exact,
    /// Through a pattern with at least [`MIN_PATTERN_LITERALS`] literal characters before the
    /// extension (`AOFLGB%c.DEF`; `%s.def` names every DEF and counts for none).
    Pattern,
    /// Not itself, but the DEF of the same base name (an MSK).
    Sibling,
    /// Not at all.
    None,
}

/// A pattern needs this many literal characters before its extension to count.
pub const MIN_PATTERN_LITERALS: usize = 3;

impl Mentions {
    /// Collects every token of `text` that looks like a file name: a run of letters, digits, `_`,
    /// `-` and `%`, a dot, and three letters or digits not followed by another one. Tokens with
    /// a `%` are patterns. Compared ignoring ASCII case.
    pub fn scan(&mut self, text: &[u8]) {
        let is_base = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'%');
        for (dot, _) in text.iter().enumerate().filter(|&(_, &b)| b == b'.') {
            let start = text
                .get(..dot)
                .unwrap_or_default()
                .iter()
                .rposition(|&b| !is_base(b))
                .map_or(0, |p| p + 1);
            let ext = text.get(dot + 1..).unwrap_or_default();
            let ext_len = ext.iter().take_while(|b| b.is_ascii_alphanumeric()).count();
            if start == dot || ext_len != 3 {
                continue;
            }
            let token: String = text
                .get(start..dot + 4)
                .unwrap_or_default()
                .iter()
                .map(|&b| char::from(b.to_ascii_uppercase()))
                .collect();
            if token.contains('%') {
                self.patterns.insert(token);
            } else {
                self.exact.insert(token);
            }
        }
    }

    /// How `name` is mentioned; `names` are all the archives' names (upper case), for an MSK's
    /// DEF.
    #[must_use]
    pub fn mention(&self, name: &str, names: &BTreeSet<String>) -> Mention {
        let name = name.to_ascii_uppercase();
        if self.exact.contains(&name) {
            Mention::Exact
        } else if self
            .patterns
            .iter()
            .any(|p| literals(p) >= MIN_PATTERN_LITERALS && matches(p.as_bytes(), name.as_bytes()))
        {
            Mention::Pattern
        } else if extension(&name) == "MSK" && self.sibling_mentioned(&name, names) {
            Mention::Sibling
        } else {
            Mention::None
        }
    }

    fn sibling_mentioned(&self, msk: &str, names: &BTreeSet<String>) -> bool {
        let def = alloc::format!("{}DEF", msk.get(..msk.len() - 3).unwrap_or_default());
        names.contains(&def) && self.mention(&def, names) != Mention::None
    }

    /// The patterns found, upper case.
    #[must_use]
    pub fn patterns(&self) -> &BTreeSet<String> {
        &self.patterns
    }

    /// How many exact names were found.
    #[must_use]
    pub fn exact_count(&self) -> usize {
        self.exact.len()
    }
}

/// Literal characters of a pattern before its extension (a `%` and its specifier are none).
fn literals(pattern: &str) -> usize {
    let base = pattern.len().saturating_sub(4);
    let mut count = 0;
    let mut rest = pattern.as_bytes().get(..base).unwrap_or_default();
    while let Some((&b, tail)) = rest.split_first() {
        rest = if b == b'%' { skip_spec(tail) } else { tail };
        count += usize::from(b != b'%');
    }
    count
}

/// `tail` after a specifier's width digits and its letter.
fn skip_spec(tail: &[u8]) -> &[u8] {
    let digits = tail.iter().take_while(|b| b.is_ascii_digit()).count();
    tail.get(digits + 1..).unwrap_or_default()
}

/// Whether `name` matches `pattern`: `%c` is one character, `%d` one or more digits, `%0Nd` (or
/// `%Nd`) exactly `N` digits, `%s` one or more characters; anything else literally.
fn matches(pattern: &[u8], name: &[u8]) -> bool {
    let Some((&p, rest)) = pattern.split_first() else {
        return name.is_empty();
    };
    if p != b'%' {
        return name
            .split_first()
            .is_some_and(|(&n, tail)| n == p && matches(rest, tail));
    }
    let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    let width: usize = rest
        .get(..digits)
        .unwrap_or_default()
        .iter()
        .fold(0, |w, d| w * 10 + usize::from(d - b'0'));
    let after = skip_spec(rest);
    match rest.get(digits) {
        Some(b'C') => name.get(1..).is_some_and(|tail| matches(after, tail)),
        Some(b'D') if width > 0 => name.get(..width).is_some_and(|run| {
            run.iter().all(u8::is_ascii_digit)
                && matches(after, name.get(width..).unwrap_or_default())
        }),
        Some(b'D') => {
            let run = name.iter().take_while(|b| b.is_ascii_digit()).count();
            (1..=run).any(|n| matches(after, name.get(n..).unwrap_or_default()))
        }
        Some(b'S') => (1..=name.len()).any(|n| matches(after, name.get(n..).unwrap_or_default())),
        _ => false,
    }
}

/// The names (upper case) that appear more than once across `archives` (each a list of entry
/// names), with the indices of the archives holding them, in order (an index twice if one archive
/// holds the name twice).
#[must_use]
pub fn duplicates(archives: &[Vec<String>]) -> BTreeMap<String, Vec<usize>> {
    let mut seen: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, names) in archives.iter().enumerate() {
        for name in names {
            seen.entry(name.to_ascii_uppercase()).or_default().push(i);
        }
    }
    seen.retain(|_, holders| holders.len() > 1);
    seen
}

/// Whether `name` breaks the archives' pattern: one to eight letters, digits or `_`, a dot, three
/// letters or digits.
#[must_use]
pub fn breaks_pattern(name: &str) -> bool {
    let ok = |s: &str| s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
    match name.split_once('.') {
        Some((base, ext)) => {
            !(1..=8).contains(&base.len())
                || ext.len() != 3
                || !ok(base)
                || !ext.bytes().all(|b| b.is_ascii_alphanumeric())
        }
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::borrow::ToOwned;
    use alloc::vec;

    fn names(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn scanning_finds_names_and_patterns() {
        let mut m = Mentions::default();
        m.scan(b"\0CPKMAN.def\0x ah%02d_.DEF; .pcx a.pcxx b.tx Game.pal,");
        assert_eq!(m.exact_count(), 2);
        assert_eq!(m.patterns(), &names(&["AH%02D_.DEF"]));
        let all = names(&[]);
        assert_eq!(m.mention("cpkman.DEF", &all), Mention::Exact);
        assert_eq!(m.mention("GAME.PAL", &all), Mention::Exact);
        assert_eq!(m.mention("AH07_.DEF", &all), Mention::Pattern);
        assert_eq!(m.mention("AH7_.DEF", &all), Mention::None);
        assert_eq!(m.mention("A.PCX", &all), Mention::None);
        m.scan(b"x.pc");
        assert_eq!(m.exact_count(), 2);
    }

    #[test]
    fn an_msk_is_mentioned_through_its_def() {
        let mut m = Mentions::default();
        m.scan(b"avwang.def AVX%04D.def");
        let all = names(&[
            "AVWANG.DEF",
            "AVWANG.MSK",
            "LONE.MSK",
            "AVX0001.DEF",
            "AVX0001.MSK",
        ]);
        assert_eq!(m.mention("avwang.msk", &all), Mention::Sibling);
        assert_eq!(m.mention("avx0001.msk", &all), Mention::Sibling);
        assert_eq!(m.mention("lone.msk", &all), Mention::None);
        // A mentioned DEF that is not in the archives does not count.
        m.scan(b"ghost.def");
        assert_eq!(m.mention("ghost.msk", &all), Mention::None);
    }

    #[test]
    fn broad_patterns_count_for_nothing() {
        assert_eq!(literals("%S.DEF"), 0);
        assert_eq!(literals("PUZ%S%02D.PCX"), 3);
        assert_eq!(literals("%SBACK.PCX"), 4);
        let mut m = Mentions::default();
        m.scan(b"%s.def");
        assert_eq!(m.mention("ANY.DEF", &names(&[])), Mention::None);
    }

    #[test]
    fn patterns_match_their_specifiers() {
        let yes = [
            ("AOFLGB%C.DEF", "AOFLGBR.DEF"),
            ("AVA%04D.DEF", "AVA0123.DEF"),
            ("GAMSELB%D.PCX", "GAMSELB12.PCX"),
            ("GAMSELB%D.PCX", "GAMSELB1.PCX"),
            ("PUZ%S%02D.PCX", "PUZCAS07.PCX"),
            ("%SBACK.PCX", "TOWNBACK.PCX"),
        ];
        for (p, n) in yes {
            assert!(matches(p.as_bytes(), n.as_bytes()), "{p} {n}");
        }
        let no = [
            ("AOFLGB%C.DEF", "AOFLGB.DEF"),
            ("AVA%04D.DEF", "AVA012.DEF"),
            ("AVA%04D.DEF", "AVA01X3.DEF"),
            ("GAMSELB%D.PCX", "GAMSELB.PCX"),
            ("%SBACK.PCX", "BACK.PCX"),
            ("A%X.DEF", "AB.DEF"),
            ("A%", "AB"),
            ("AB", "ABC"),
            ("AB", "AX"),
        ];
        for (p, n) in no {
            assert!(!matches(p.as_bytes(), n.as_bytes()), "{p} {n}");
        }
    }

    #[test]
    fn duplicates_name_their_archives() {
        let a = vec!["X.DEF".to_owned(), "y.pcx".to_owned()];
        let b = vec!["x.def".to_owned(), "Z.MSK".to_owned(), "Z.msk".to_owned()];
        let d = duplicates(&[a, b]);
        assert_eq!(d.len(), 2);
        assert_eq!(d["X.DEF"], [0, 1]);
        assert_eq!(d["Z.MSK"], [1, 1]);
    }

    #[test]
    fn the_8_3_pattern() {
        for good in ["CPKMAN.DEF", "Ar_Bg.pcx", "A.TXT", "ABCDEFGH.H3C"] {
            assert!(!breaks_pattern(good), "{good}");
        }
        for bad in [
            "NOEXT",
            ".DEF",
            "ABCDEFGHI.DEF",
            "A.DE",
            "A-B.DEF",
            "A.D_F",
            "A.B.DEF",
        ] {
            assert!(breaks_pattern(bad), "{bad}");
        }
    }
}
