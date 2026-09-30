//! Text tables (`*.TXT` inside the bitmap LOD archives), syntax in
//! `docs/formats/installation.md` §4.
//!
//! Observed syntax: Windows-1252 bytes; records end with CRLF (a bare LF outside quotes is
//! accepted as well); fields are separated by TAB; a field that *starts* with `"` runs to the next
//! `"` that is followed by TAB, CR, LF or the end of input, and may contain line breaks (bare LF in
//! the observed files). A quote anywhere else is an ordinary character.

use crate::FormatError;

/// A parsed table: records of fields, header rows included, in file order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextTable {
    /// Records; each record is its list of fields (at least one, possibly empty).
    pub records: Vec<Vec<String>>,
}

impl TextTable {
    /// Parses a table from raw bytes.
    ///
    /// # Errors
    /// [`FormatError::Text`] for a quoted field that is never closed.
    pub fn parse(bytes: &[u8]) -> Result<Self, FormatError> {
        let text = decode_windows_1252(bytes);
        let chars: Vec<char> = text.chars().collect();
        let mut records = Vec::new();
        let mut record: Vec<String> = Vec::new();
        let mut field = String::new();
        let mut i = 0;
        let mut at_field_start = true;
        while i < chars.len() {
            let c = chars[i];
            if at_field_start && c == '"' {
                // Quoted field: find the closing quote.
                let mut j = i + 1;
                loop {
                    match chars.get(j) {
                        None => {
                            return Err(FormatError::Text {
                                record: records.len() + 1,
                                message: "unterminated quoted field",
                            });
                        }
                        Some('"')
                            if matches!(chars.get(j + 1), None | Some('\t' | '\r' | '\n')) =>
                        {
                            break;
                        }
                        Some(&ch) => field.push(ch),
                    }
                    j += 1;
                }
                i = j + 1;
                at_field_start = false;
                continue;
            }
            at_field_start = false;
            match c {
                '\t' => {
                    record.push(std::mem::take(&mut field));
                    at_field_start = true;
                }
                '\r' if chars.get(i + 1) == Some(&'\n') => {}
                '\n' => {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                    at_field_start = true;
                }
                _ => field.push(c),
            }
            i += 1;
        }
        if !at_field_start || !record.is_empty() || !field.is_empty() {
            record.push(field);
            records.push(record);
        }
        Ok(Self { records })
    }

    /// Returns `true` if every field of the record is empty or whitespace (separator rows).
    #[must_use]
    pub fn is_blank(record: &[String]) -> bool {
        record.iter().all(|f| f.trim().is_empty())
    }
}

/// Windows-1252 code points for bytes 0x80–0x9F (undefined bytes map to the C1 control of the
/// same value, as the WHATWG decoder does).
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// Decodes Windows-1252 bytes (total: every byte maps to one character).
#[must_use]
pub fn decode_windows_1252(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => CP1252_HIGH[usize::from(b - 0x80)],
            _ => char::from(b),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<Vec<String>> {
        TextTable::parse(s.as_bytes()).unwrap().records
    }

    #[test]
    fn tabs_crlf_and_empty_fields() {
        assert_eq!(
            parse("a\tb\r\n\tc\t\r\n"),
            vec![vec!["a", "b"], vec!["", "c", ""]]
        );
        assert_eq!(parse("x\ny"), vec![vec!["x"], vec!["y"]]);
        assert_eq!(parse(""), Vec::<Vec<String>>::new());
        assert_eq!(parse("\r\n"), vec![vec![""]]);
    }

    #[test]
    fn quoted_fields_span_lines_and_keep_inner_quotes() {
        let t = parse("Name\t\"one\ntwo\"\tX\r\n\"a \"quoted\" word\"\r\n");
        assert_eq!(
            t,
            vec![vec!["Name", "one\ntwo", "X"], vec!["a \"quoted\" word"]]
        );
        // A quote that does not start a field is an ordinary character.
        assert_eq!(parse("5\" gun\tx\r\n"), vec![vec!["5\" gun", "x"]]);
        // A closed quoted field at end of input.
        assert_eq!(parse("\"q\""), vec![vec!["q"]]);
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        let err = TextTable::parse(b"a\r\n\"open\tfield").unwrap_err();
        assert_eq!(
            err,
            FormatError::Text {
                record: 2,
                message: "unterminated quoted field"
            }
        );
    }

    #[test]
    fn windows_1252() {
        assert_eq!(
            decode_windows_1252(&[0x41, 0x80, 0x93, 0xE9, 0x81]),
            "A\u{20AC}\u{201C}\u{e9}\u{81}"
        );
    }

    #[test]
    fn blank_rows() {
        assert!(TextTable::is_blank(&["".into(), "  ".into()]));
        assert!(!TextTable::is_blank(&["".into(), "x".into()]));
    }
}
