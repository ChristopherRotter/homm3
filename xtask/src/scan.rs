//! A small Rust source scanner used by the architecture, rule and stub checks.
//!
//! It is not a full lexer, but it understands enough of Rust to be reliable for our checks:
//! line/block comments (nested), string/char/byte/raw-string literals, lifetimes, identifiers,
//! numbers and punctuation. Comments are kept separately (markers live there). Items annotated
//! with `#[cfg(test)]` are flagged as test code; whole files are classified by [`classify`]
//! (module tree from the crate root, plus Cargo's `tests/`, `benches/`, `examples/`).
//!
//! Float literals (`1.5`, `2e3`, `1f64`) additionally produce a synthetic `f64` token so that the
//! float confinement rule (ADR-0003 §4) also catches untyped float literals.

use std::path::Path;

/// Kind of a source token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Ident,
    Number,
    Punct,
}

/// A token with its 1-based line and whether it belongs to test-only code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub text: String,
    pub line: usize,
    pub in_test: bool,
}

/// A comment (line or block) with the line it starts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub text: String,
    pub line: usize,
    pub in_test: bool,
}

/// Result of scanning one file.
#[derive(Debug, Default)]
pub struct Scanned {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

/// Scans Rust source text.
pub fn scan(source: &str, whole_file_is_test: bool) -> Scanned {
    let mut out = lex(source);
    mark_test_items(&mut out, whole_file_is_test);
    out
}

fn lex(source: &str) -> Scanned {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Scanned::default();
    let mut i = 0;
    let mut line = 1;
    let n = chars.len();
    let at = |j: usize| -> char { if j < n { chars[j] } else { '\0' } };

    while i < n {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '/' && at(i + 1) == '/' {
            let start = i;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            out.comments.push(Comment {
                text: chars[start..i].iter().collect(),
                line,
                in_test: false,
            });
        } else if c == '/' && at(i + 1) == '*' {
            let start = i;
            let start_line = line;
            let mut depth = 0usize;
            while i < n {
                if chars[i] == '/' && at(i + 1) == '*' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && at(i + 1) == '/' {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
            out.comments.push(Comment {
                text: chars[start..i.min(n)].iter().collect(),
                line: start_line,
                in_test: false,
            });
        } else if let Some(end) = raw_string_end(&chars, i) {
            line += chars[i..end].iter().filter(|&&ch| ch == '\n').count();
            i = end;
        } else if c == '"' || (c == 'b' && at(i + 1) == '"') || (c == 'c' && at(i + 1) == '"') {
            i += if c == '"' { 1 } else { 2 };
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                if i < n && chars[i] == '\n' {
                    line += 1;
                }
                i += 1;
            }
            i += 1;
        } else if c == '\'' || (c == 'b' && at(i + 1) == '\'') {
            let q = if c == '\'' { i } else { i + 1 };
            // Char literal: 'x', '\n', '\u{..}', b'x'. Otherwise a lifetime or label.
            if at(q + 1) == '\\' {
                let mut j = q + 2;
                while j < n && chars[j] != '\'' && chars[j] != '\n' {
                    j += 1;
                }
                i = j + 1;
            } else if at(q + 2) == '\'' {
                i = q + 3;
            } else {
                // Lifetime: skip the quote; the identifier is lexed next.
                i = q + 1;
            }
        } else if c.is_alphabetic() || c == '_' {
            if c == 'r' && at(i + 1) == '#' && (at(i + 2).is_alphabetic() || at(i + 2) == '_') {
                i += 2;
            }
            let ident_start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.tokens.push(Token {
                kind: Kind::Ident,
                text: chars[ident_start..i].iter().collect(),
                line,
                in_test: false,
            });
        } else if c.is_ascii_digit() {
            let start = i;
            let mut is_float = false;
            let hex = c == '0' && matches!(at(i + 1), 'x' | 'o' | 'b');
            i += 1;
            while i < n {
                let ch = chars[i];
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    if !hex
                        && (ch == 'e' || ch == 'E')
                        && (at(i + 1).is_ascii_digit() || matches!(at(i + 1), '+' | '-'))
                    {
                        is_float = true;
                        i += 1;
                        if matches!(chars[i], '+' | '-') {
                            i += 1;
                        }
                        continue;
                    }
                    i += 1;
                } else if ch == '.' && at(i + 1).is_ascii_digit() && !hex {
                    is_float = true;
                    i += 1;
                } else if ch == '.'
                    && at(i + 1) != '.'
                    && !at(i + 1).is_alphabetic()
                    && at(i + 1) != '_'
                    && !hex
                {
                    // `1.` is a float literal; `1..2` and `1.max(2)` are not.
                    is_float = true;
                    i += 1;
                    break;
                } else {
                    break;
                }
            }
            let text: String = chars[start..i].iter().collect();
            if !hex && (text.ends_with("f32") || text.ends_with("f64")) {
                is_float = true;
            }
            out.tokens.push(Token {
                kind: Kind::Number,
                text,
                line,
                in_test: false,
            });
            if is_float {
                out.tokens.push(Token {
                    kind: Kind::Ident,
                    text: "f64".to_owned(),
                    line,
                    in_test: false,
                });
            }
        } else if c == ':' && at(i + 1) == ':' {
            out.tokens.push(Token {
                kind: Kind::Punct,
                text: "::".to_owned(),
                line,
                in_test: false,
            });
            i += 2;
        } else {
            out.tokens.push(Token {
                kind: Kind::Punct,
                text: c.to_string(),
                line,
                in_test: false,
            });
            i += 1;
        }
    }
    out
}

/// If a raw (byte/C) string literal starts at `i`, returns the index just past its end.
fn raw_string_end(chars: &[char], i: usize) -> Option<usize> {
    let n = chars.len();
    let mut j = i;
    if j < n && (chars[j] == 'b' || chars[j] == 'c') {
        j += 1;
    }
    if j >= n || chars[j] != 'r' {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while j < n && chars[j] == '#' {
        hashes += 1;
        j += 1;
    }
    if j >= n || chars[j] != '"' {
        return None;
    }
    // Must not be part of a longer identifier (e.g. `bar"`): the caller only calls us at a token
    // start, so the previous char is not an identifier char.
    j += 1;
    while j < n {
        if chars[j] == '"' {
            let mut k = j + 1;
            let mut count = 0;
            while k < n && chars[k] == '#' && count < hashes {
                count += 1;
                k += 1;
            }
            if count == hashes {
                return Some(k);
            }
        }
        j += 1;
    }
    Some(n)
}

/// Marks tokens and comments inside `#[cfg(test)]` items (and whole test files) as test code.
fn mark_test_items(out: &mut Scanned, whole_file_is_test: bool) {
    if whole_file_is_test {
        for t in &mut out.tokens {
            t.in_test = true;
        }
        for c in &mut out.comments {
            c.in_test = true;
        }
        return;
    }
    let toks = &out.tokens;
    let mut ranges: Vec<(usize, usize)> = Vec::new(); // line ranges (inclusive)
    let mut test_token = vec![false; toks.len()];
    let mut i = 0;
    while i < toks.len() {
        if is_cfg_test_attr(toks, i) {
            let start_line = toks[i].line;
            let mut j = i + 7; // past `# [ cfg ( test ) ]`
            // Skip further attributes on the same item.
            while j < toks.len() && toks[j].text == "#" {
                j = skip_attribute(toks, j);
            }
            // Skip the annotated item: it ends at `;` or `,` at depth 0 (items, fields, variants,
            // match arms), at the end of its first top-level `{}` block, or just before a closing
            // bracket of the enclosing construct (a last field/variant without a comma).
            // Generic arguments are not tracked, so a `,` inside `<...>` ends the item early:
            // that errs on the side of scanning more code, never less.
            let mut depth = 0i32;
            let mut end = toks.len().saturating_sub(1);
            while j < toks.len() {
                let t = toks[j].text.as_str();
                match t {
                    "{" | "(" | "[" => depth += 1,
                    "}" | ")" | "]" if depth == 0 => {
                        end = j.saturating_sub(1);
                        break;
                    }
                    "}" | ")" | "]" => {
                        depth -= 1;
                        if depth == 0 && t == "}" {
                            end = j;
                            break;
                        }
                    }
                    ";" | "," if depth == 0 => {
                        end = j;
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }
            let end = end.max(i);
            for flag in &mut test_token[i..=end] {
                *flag = true;
            }
            ranges.push((start_line, toks[end].line));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    for (t, flag) in out.tokens.iter_mut().zip(test_token) {
        t.in_test = flag;
    }
    for c in &mut out.comments {
        c.in_test = ranges.iter().any(|&(a, b)| c.line >= a && c.line <= b);
    }
}

fn is_cfg_test_attr(toks: &[Token], i: usize) -> bool {
    let want = ["#", "[", "cfg", "(", "test", ")", "]"];
    i + want.len() <= toks.len() && want.iter().enumerate().all(|(k, w)| toks[i + k].text == *w)
}

/// Given `toks[j] == "#"`, returns the index after the attribute's closing `]`.
pub fn skip_attribute(toks: &[Token], j: usize) -> usize {
    let mut k = j + 1;
    if k < toks.len() && toks[k].text == "!" {
        k += 1;
    }
    if k >= toks.len() || toks[k].text != "[" {
        return j + 1;
    }
    let mut depth = 0;
    while k < toks.len() {
        match toks[k].text.as_str() {
            "[" => depth += 1,
            "]" => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            _ => {}
        }
        k += 1;
    }
    k
}

/// Classifies every `.rs` file of the crates below `crates_dir` as test-only (`true`) or not.
///
/// Files under a crate's `tests/`, `benches/` or `examples/` directory are test/bench code by
/// Cargo convention. Files under `src/` are classified by walking the module tree from `lib.rs`
/// / `main.rs`: a file is test-only if it is reached through a `#[cfg(test)] mod name;`
/// declaration. Files not reached from the crate root are treated as production code, so they
/// are scanned (and flagged) rather than silently exempted. `#[path]` attributes are not
/// supported.
pub fn classify(crates_dir: &Path) -> Result<Vec<(std::path::PathBuf, bool)>, String> {
    let mut crate_dirs: Vec<std::path::PathBuf> = std::fs::read_dir(crates_dir)
        .map_err(|e| format!("{}: {e}", crates_dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    crate_dirs.sort();
    let mut out = Vec::new();
    for dir in crate_dirs {
        out.extend(classify_crate(&dir)?);
    }
    Ok(out)
}

/// [`classify`] for one crate directory.
pub fn classify_crate(dir: &Path) -> Result<Vec<(std::path::PathBuf, bool)>, String> {
    let tree = module_tree(&dir.join("src"))?;
    let mut out = Vec::new();
    for file in crate::util::rs_files(dir)? {
        let rel = file
            .strip_prefix(dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let area = rel.split('/').next().unwrap_or_default().to_owned();
        let is_test = match area.as_str() {
            "src" => tree.get(&file).copied().unwrap_or(false),
            "tests" | "benches" | "examples" => true,
            _ => false,
        };
        out.push((file, is_test));
    }
    Ok(out)
}

/// Test-ness of the files reachable from `src/lib.rs` and `src/main.rs`.
fn module_tree(src: &Path) -> Result<std::collections::BTreeMap<std::path::PathBuf, bool>, String> {
    let mut seen = std::collections::BTreeMap::new();
    let mut queue: Vec<(std::path::PathBuf, bool)> = ["lib.rs", "main.rs"]
        .iter()
        .map(|r| (src.join(r), false))
        .filter(|(p, _)| p.exists())
        .collect();
    while let Some((file, is_test)) = queue.pop() {
        if seen.contains_key(&file) {
            continue;
        }
        seen.insert(file.clone(), is_test);
        let text =
            std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let scanned = scan(&text, false);
        let name = file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parent = file.parent().map(Path::to_path_buf).unwrap_or_default();
        let dir = if matches!(name.as_str(), "lib.rs" | "main.rs" | "mod.rs") {
            parent
        } else {
            parent.join(file.file_stem().unwrap_or_default())
        };
        let toks = &scanned.tokens;
        for w in toks.windows(3) {
            if w[0].text == "mod" && w[1].kind == Kind::Ident && w[2].text == ";" {
                let flat = dir.join(format!("{}.rs", w[1].text));
                let nested = dir.join(&w[1].text).join("mod.rs");
                let child = if flat.exists() { flat } else { nested };
                queue.push((child, is_test || w[0].in_test));
            }
        }
    }
    Ok(seen)
}

/// A lint level set by an attribute: `#[allow(clippy::x)]`, `#![deny(a, b)]`, also inside
/// `cfg_attr(...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintLevel {
    /// `allow`, `expect`, `warn`, `deny` or `forbid`.
    pub level: String,
    /// Lint path, e.g. `clippy::unwrap_used` or `unsafe_code`.
    pub lint: String,
    /// Inner attribute (`#![...]`).
    pub inner: bool,
    pub line: usize,
    pub in_test: bool,
}

/// Inner attribute names (`#![name]` or `#![name(...)]`), e.g. `no_std`, in non-test code.
pub fn inner_attributes(scanned: &Scanned) -> Vec<String> {
    let t = &scanned.tokens;
    let mut out = Vec::new();
    for i in 0..t.len().saturating_sub(3) {
        if t[i].text == "#" && t[i + 1].text == "!" && t[i + 2].text == "[" && !t[i].in_test {
            out.push(t[i + 3].text.clone());
        }
    }
    out
}

/// All lint levels set by attributes in the file.
pub fn lint_levels(scanned: &Scanned) -> Vec<LintLevel> {
    let t = &scanned.tokens;
    let mut out = Vec::new();
    let mut i = 0;
    while i < t.len() {
        if t[i].text != "#" {
            i += 1;
            continue;
        }
        let inner = t.get(i + 1).is_some_and(|x| x.text == "!");
        let open = if inner { i + 2 } else { i + 1 };
        if t.get(open).is_none_or(|x| x.text != "[") {
            i += 1;
            continue;
        }
        let end = skip_attribute(t, i);
        let mut k = open + 1;
        while k < end {
            let is_level = matches!(
                t[k].text.as_str(),
                "allow" | "expect" | "warn" | "deny" | "forbid"
            );
            if is_level && t.get(k + 1).is_some_and(|x| x.text == "(") {
                let level = t[k].text.clone();
                let mut j = k + 2;
                let mut path = String::new();
                while j < end && t[j].text != ")" {
                    match t[j].text.as_str() {
                        "," => {
                            if !path.is_empty() {
                                out.push(LintLevel {
                                    level: level.clone(),
                                    lint: std::mem::take(&mut path),
                                    inner,
                                    line: t[j].line,
                                    in_test: t[j].in_test,
                                });
                            }
                        }
                        other => path.push_str(other),
                    }
                    j += 1;
                }
                if !path.is_empty() {
                    out.push(LintLevel {
                        level,
                        lint: path,
                        inner,
                        line: t[k].line,
                        in_test: t[k].in_test,
                    });
                }
                k = j;
            }
            k += 1;
        }
        i = end.max(i + 1);
    }
    out
}

/// Finds occurrences of a whitespace-separated token pattern (e.g. `"static mut"`, `". sort"`,
/// `"std"`) in non-test code. Returns the line of each match.
pub fn find_pattern(scanned: &Scanned, pattern: &str) -> Vec<usize> {
    let want: Vec<&str> = pattern.split_whitespace().collect();
    if want.is_empty() {
        return Vec::new();
    }
    let toks = &scanned.tokens;
    let mut lines = Vec::new();
    for i in 0..toks.len() {
        if i + want.len() > toks.len() {
            break;
        }
        if toks[i].in_test {
            continue;
        }
        if want.iter().enumerate().all(|(k, w)| toks[i + k].text == *w) {
            lines.push(toks[i].line);
        }
    }
    lines
}

/// Extracts marker values from comments: for `// rule: BAT-DMG-001, BAT-DMG-002` and key `rule`
/// returns `[(line, in_test, "BAT-DMG-001"), (line, in_test, "BAT-DMG-002")]`.
pub fn markers(scanned: &Scanned, key: &str) -> Vec<(usize, bool, String)> {
    let needle = format!("{key}:");
    let mut out = Vec::new();
    for c in &scanned.comments {
        let body = c
            .text
            .trim_start_matches('/')
            .trim_start_matches('*')
            .trim_start_matches('!')
            .trim_end_matches("*/")
            .trim();
        if let Some(rest) = body.strip_prefix(&needle) {
            for id in rest.split(|ch: char| ch == ',' || ch.is_whitespace()) {
                let id = id.trim();
                if !id.is_empty() {
                    out.push((c.line, c.in_test, id.to_owned()));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idents(s: &Scanned) -> Vec<&str> {
        s.tokens
            .iter()
            .filter(|t| t.kind == Kind::Ident)
            .map(|t| t.text.as_str())
            .collect()
    }

    #[test]
    fn ignores_comments_and_strings() {
        let s = scan(
            "let a = \"HashMap\"; // HashMap\n/* HashMap /* nested */ HashMap */ let b = r#\"HashMap\"#;",
            false,
        );
        assert!(!idents(&s).contains(&"HashMap"));
        assert_eq!(s.comments.len(), 2);
        assert_eq!(find_pattern(&s, "HashMap"), Vec::<usize>::new());
    }

    #[test]
    fn distinguishes_lifetimes_and_chars() {
        let s = scan(
            "fn f<'a>(x: &'a str) -> char { 'x' } let c = '\\n'; let d = b'q';",
            false,
        );
        assert!(idents(&s).contains(&"a"));
        assert!(
            !idents(&s).contains(&"x") || idents(&s).iter().filter(|i| **i == "x").count() == 1
        );
    }

    #[test]
    fn float_literals_produce_f64_token() {
        for src in [
            "let x = 1.5;",
            "let x = 2e3;",
            "let x = 1f32;",
            "let x = 1.;",
        ] {
            let s = scan(src, false);
            assert_eq!(find_pattern(&s, "f64").len(), 1, "{src}");
        }
        for src in [
            "let x = 1..2;",
            "let x = 1.max(2);",
            "let x = 0xE5;",
            "let x = t.0;",
            "let x = 10_u32;",
        ] {
            let s = scan(src, false);
            assert!(find_pattern(&s, "f64").is_empty(), "{src}");
        }
    }

    #[test]
    fn cfg_test_items_are_marked() {
        let src = "fn a() { HashMap }\n#[cfg(test)]\nmod tests {\n  // covers: X-1\n  fn b() { HashMap }\n}\nfn c() {}\n#[cfg(test)]\nextern crate std;\nfn d() { std }";
        let s = scan(src, false);
        assert_eq!(find_pattern(&s, "HashMap"), vec![1]);
        assert_eq!(find_pattern(&s, "std"), vec![10]);
        let m = markers(&s, "covers");
        assert_eq!(m, vec![(4, true, "X-1".to_owned())]);
    }

    #[test]
    fn cfg_test_on_fields_arms_and_variants_does_not_swallow_following_code() {
        let src = "struct S { #[cfg(test)] dbg: u8, x: u8 }\nfn prod() { HashSet }\n\
                   enum E { A, #[cfg(test)] B }\nfn prod2() { HashSet }\n\
                   fn m(x: u8) { match x { #[cfg(test)] 1 => {} _ => {} } }\nfn prod3() { HashSet }";
        let s = scan(src, false);
        assert_eq!(find_pattern(&s, "HashSet"), vec![2, 4, 6]);
    }

    #[test]
    fn module_tree_classifies_files() {
        use crate::util::test_support::TempDir;
        let dir = TempDir::new("scan");
        dir.write(
            "crates/a/src/lib.rs",
            "mod prod;\n#[cfg(test)]\nmod tests;\npub mod nested;\n",
        );
        dir.write("crates/a/src/prod.rs", "");
        dir.write("crates/a/src/tests.rs", "mod helper;\n");
        dir.write("crates/a/src/tests/helper.rs", "");
        dir.write("crates/a/src/nested/mod.rs", "#[cfg(test)] mod t;\n");
        dir.write("crates/a/src/nested/t.rs", "");
        dir.write("crates/a/src/orphan/tests.rs", "");
        dir.write("crates/a/tests/it.rs", "");
        dir.write("crates/a/build.rs", "");
        let got: Vec<(String, bool)> = classify(&dir.path().join("crates"))
            .unwrap()
            .into_iter()
            .map(|(p, t)| (crate::util::rel(&dir.path().join("crates/a"), &p), t))
            .collect();
        let expect = [
            ("build.rs", false),
            ("src/lib.rs", false),
            ("src/nested/mod.rs", false),
            ("src/nested/t.rs", true),
            ("src/orphan/tests.rs", false),
            ("src/prod.rs", false),
            ("src/tests.rs", true),
            ("src/tests/helper.rs", true),
            ("tests/it.rs", true),
        ];
        let mut got = got;
        got.sort();
        let expect: Vec<(String, bool)> =
            expect.iter().map(|(p, t)| ((*p).to_owned(), *t)).collect();
        assert_eq!(got, expect);
    }

    #[test]
    fn extracts_attributes_and_lint_levels() {
        let src = "//! doc\n#![no_std]\n#![forbid(unsafe_code)]\n#![deny(\n  clippy::unwrap_used,\n  clippy::panic\n)]\n\
                   // #![allow(clippy::panic)]\n#[allow(clippy::float_arithmetic)]\nfn f() {}\n\
                   #[cfg_attr(test, expect(clippy::todo))]\nfn g() {}\n#[cfg(test)]\n#[allow(clippy::exit)]\nmod tests {}\n";
        let s = scan(src, false);
        assert_eq!(inner_attributes(&s), ["no_std", "forbid", "deny"]);
        let levels: Vec<(String, String, bool, bool)> = lint_levels(&s)
            .into_iter()
            .map(|l| (l.level, l.lint, l.inner, l.in_test))
            .collect();
        let want = [
            ("forbid", "unsafe_code", true, false),
            ("deny", "clippy::unwrap_used", true, false),
            ("deny", "clippy::panic", true, false),
            ("allow", "clippy::float_arithmetic", false, false),
            ("expect", "clippy::todo", false, false),
            ("allow", "clippy::exit", false, true),
        ];
        let want: Vec<(String, String, bool, bool)> = want
            .iter()
            .map(|(a, b, c, d)| ((*a).to_owned(), (*b).to_owned(), *c, *d))
            .collect();
        assert_eq!(levels, want);
    }

    #[test]
    fn whole_file_test_flag() {
        let s = scan("fn a() { HashMap }", true);
        assert!(find_pattern(&s, "HashMap").is_empty());
    }

    #[test]
    fn multi_token_patterns_and_markers() {
        let s = scan(
            "static mut X: u8 = 0; v.sort(); v.sort_by_key(|x| x);\n// rule: A-B-001, A-B-002\n/// rule: A-B-003",
            false,
        );
        assert_eq!(find_pattern(&s, "static mut").len(), 1);
        assert_eq!(find_pattern(&s, ". sort").len(), 1);
        assert_eq!(find_pattern(&s, ". sort_by_key").len(), 1);
        let ids: Vec<_> = markers(&s, "rule").into_iter().map(|m| m.2).collect();
        assert_eq!(ids, ["A-B-001", "A-B-002", "A-B-003"]);
    }
}
