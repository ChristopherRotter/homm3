//! The key catalogue (E17 M3 T4b-1, `docs/epics/E17-design.md` §9.1): `clients/text/keys.toml`,
//! every key the client shows a text by, with the arguments its messages may use and, for the
//! original's text, where the user's text tables hold it (a table, a record, a column: numbers
//! only, ADR-0009). A family binds one key per entry of the asset tables (`creature.base:pikeman`:
//! a creature's names by its original id). Unknown fields are errors, as in the asset tables.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;

use super::message::Kind;
use super::plural::Category;

/// The catalogue as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeysFile {
    /// The keys, each by its id.
    #[serde(default)]
    pub key: Vec<Key>,
    /// The families of keys over the asset tables' entries.
    #[serde(default)]
    pub family: Vec<Family>,
}

/// A key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    /// Its id: lower-case segments of letters, digits and `-`, joined by `.` (`battle.log.damage`).
    pub id: String,
    /// The arguments its messages may use, by name, and their kinds (`text`, `number`).
    #[serde(default)]
    pub args: BTreeMap<String, ArgKind>,
    /// Whether its messages have plural forms, picked by the number argument `count`.
    #[serde(default)]
    pub plural: bool,
    /// The values of the argument `context` its messages have forms for (`male`, `female`).
    #[serde(default)]
    pub contexts: Vec<String>,
    /// Its forms in the original's text tables (none: the client's own text).
    #[serde(default)]
    pub original: Vec<Original>,
}

/// An argument's kind in the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgKind {
    /// A text (`%s` in the original).
    Text,
    /// A whole number (`%d`).
    Number,
}

impl ArgKind {
    /// The message's kind.
    #[must_use]
    pub const fn kind(self) -> Kind {
        match self {
            Self::Text => Kind::Text,
            Self::Number => Kind::Number,
        }
    }
}

/// A form of a key in the original's text tables.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Original {
    /// Its suffix (`one`, `other`, `male`; none: the key's only form).
    #[serde(default)]
    pub form: Option<String>,
    /// Where: `<TABLE>:<record>` or `<TABLE>:<record>:<column>` (the column 0 if left out), the
    /// table by its entry's name without `.TXT`, its records counted from 0 with the header rows,
    /// as the parser reads them.
    pub at: String,
    /// The arguments its `%s` and `%d` stand for, in order.
    #[serde(default)]
    pub printf: Vec<String>,
}

/// A family: a key per entry of an asset-table list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Family {
    /// Its id and the list's: `creature` (the asset tables' creatures, keys
    /// `creature.<table>:<key>`).
    pub id: String,
    /// The table holding a record per original id.
    pub table: String,
    /// The first record of those: the table's non-blank records from it are the original ids in
    /// order (the harvesters' reading).
    pub first: usize,
    /// The columns of the forms (`one = 0`, `other = 1`): the family's keys are plural; or none,
    /// and [`Family::column`] holds its keys' only form (E17 M3 T7b-1).
    #[serde(default)]
    pub forms: BTreeMap<String, usize>,
    /// The column of a single-form family's keys (the spells' and heroes' names).
    #[serde(default)]
    pub column: Option<usize>,
    /// Which records count as ids: those whose column `by` is not blank (`SPTRAITS.TXT`'s group
    /// rows fill only their first); none: every non-blank record.
    #[serde(default)]
    pub by: Option<usize>,
}

/// A family's entry: its key's last part and its original id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The entry's qualified key (`base:pikeman`).
    pub key: String,
    /// Its original id (the executable's creature number).
    pub id: usize,
}

/// A key with its forms resolved, as the catalogue keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// Its arguments' kinds (`count` included for a plural key).
    pub args: BTreeMap<String, Kind>,
    /// Whether it has plural forms.
    pub plural: bool,
    /// The contexts it has forms for.
    pub contexts: Vec<String>,
    /// Its forms in the original's tables: the suffix (empty for the only form), where its text
    /// is and the arguments of its specifiers in order.
    pub original: Vec<(String, Source, Vec<String>)>,
}

/// Where an original form's text is: a table, a row of it and a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The table's entry name without `.TXT`, as written (`GENRLTXT`).
    pub table: String,
    /// The row.
    pub row: Row,
    /// The column, from 0.
    pub column: usize,
}

/// A source's row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// A record, from 0 with the header rows.
    Record(usize),
    /// A string of the executable at its address (the table [`EXECUTABLE`], E17 M3 T7b-2b-3): its
    /// record is the one whose first field is the address ([`address_field`]), its text the
    /// second.
    Address(u32),
    /// A family's: the `id`-th non-blank record from `first` (with `by`, the `id`-th whose column
    /// `by` is not blank).
    Id {
        /// The first record of the id-ordered ones.
        first: usize,
        /// The original id.
        id: usize,
        /// The column that tells an id's record, if not every non-blank one.
        by: Option<usize>,
    },
}

/// The key catalogue: every key by id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keys {
    /// The keys.
    pub keys: BTreeMap<String, Resolved>,
}

/// What is wrong with a catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeysError {
    /// Not a valid file (the parser's message).
    Parse(String),
    /// A key's id is not lower-case segments joined by `.`.
    Id(String),
    /// A key is defined twice (also through a family).
    Duplicate(String),
    /// A key's form is wrong (the key, what).
    Form(String, String),
    /// A family the client does not know.
    Family(String),
}

impl core::fmt::Display for KeysError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(m) => write!(f, "not a key catalogue: {m}"),
            Self::Id(id) => write!(f, "`{id}` is not a key id"),
            Self::Duplicate(id) => write!(f, "`{id}` is defined twice"),
            Self::Form(id, what) => write!(f, "`{id}`: {what}"),
            Self::Family(id) => write!(f, "no family `{id}`"),
        }
    }
}

impl Keys {
    /// Reads a catalogue and expands its families over `entries` (by family id: the creatures'
    /// qualified keys and original ids).
    ///
    /// # Errors
    /// The first problem found, as [`KeysError`].
    pub fn parse(text: &str, entries: &BTreeMap<String, Vec<Entry>>) -> Result<Self, KeysError> {
        let file: KeysFile = toml::from_str(text).map_err(|e| KeysError::Parse(e.to_string()))?;
        let mut keys = Self::default();
        for key in file.key {
            let resolved = resolve(&key)?;
            keys.insert(key.id, resolved)?;
        }
        for family in file.family {
            let list = entries
                .get(&family.id)
                .ok_or_else(|| KeysError::Family(family.id.clone()))?;
            let single = match (family.forms.is_empty(), family.column) {
                (true, Some(column)) => Some(column),
                (false, None) => None,
                _ => return form_error(&family.id, "a family has forms or a column, not both"),
            };
            for entry in list {
                let id = format!("{}.{}", family.id, entry.key);
                let source = |column: usize| Source {
                    table: family.table.clone(),
                    row: Row::Id {
                        first: family.first,
                        id: entry.id,
                        by: family.by,
                    },
                    column,
                };
                let resolved = match single {
                    Some(column) => Resolved {
                        args: BTreeMap::new(),
                        plural: false,
                        contexts: Vec::new(),
                        original: vec![(String::new(), source(column), Vec::new())],
                    },
                    None => {
                        let mut original = Vec::new();
                        for (form, &column) in &family.forms {
                            plural_form(&id, form)?;
                            original.push((form.clone(), source(column), Vec::new()));
                        }
                        Resolved {
                            args: BTreeMap::from([("count".to_string(), Kind::Number)]),
                            plural: true,
                            contexts: Vec::new(),
                            original,
                        }
                    }
                };
                keys.insert(id, resolved)?;
            }
        }
        Ok(keys)
    }

    fn insert(&mut self, id: String, key: Resolved) -> Result<(), KeysError> {
        if self.keys.contains_key(&id) {
            return Err(KeysError::Duplicate(id));
        }
        self.keys.insert(id, key);
        Ok(())
    }

    /// The names a pack may give a key's messages: the id, with `_<context>` and with
    /// `_<category>` (a plural key's), and both (`_male_one`).
    #[must_use]
    pub fn names(id: &str, key: &Resolved) -> Vec<String> {
        let plurals: Vec<&str> = if key.plural {
            Category::ALL.iter().map(|c| c.name()).collect()
        } else {
            Vec::new()
        };
        let bases = core::iter::once(id.to_string())
            .chain(key.contexts.iter().map(|c| format!("{id}_{c}")));
        let mut names = Vec::new();
        for base in bases {
            names.extend(plurals.iter().map(|p| format!("{base}_{p}")));
            names.push(base);
        }
        names
    }
}

fn resolve(key: &Key) -> Result<Resolved, KeysError> {
    if !valid_id(&key.id) {
        return Err(KeysError::Id(key.id.clone()));
    }
    let mut args: BTreeMap<String, Kind> = key
        .args
        .iter()
        .map(|(n, k)| (n.clone(), k.kind()))
        .collect();
    if key.plural && args.insert("count".to_string(), Kind::Number) == Some(Kind::Text) {
        return form_error(&key.id, "`count` is a plural key's number");
    }
    let mut original = Vec::new();
    for o in &key.original {
        let form = o.form.clone().unwrap_or_default();
        let fits = if form.is_empty() {
            !key.plural && key.contexts.is_empty()
        } else {
            Category::from_name(&form).map_or_else(|| key.contexts.contains(&form), |_| key.plural)
        };
        if !fits {
            return form_error(&key.id, &format!("no form `{form}` here"));
        }
        for name in &o.printf {
            if !args.contains_key(name) {
                return form_error(&key.id, &format!("`{name}` is not an argument"));
            }
        }
        let source = source(&key.id, &o.at)?;
        original.push((form, source, o.printf.clone()));
    }
    Ok(Resolved {
        args,
        plural: key.plural,
        contexts: key.contexts.clone(),
        original,
    })
}

/// A plural family's form must be a category.
fn plural_form(id: &str, form: &str) -> Result<(), KeysError> {
    match Category::from_name(form) {
        Some(_) => Ok(()),
        None => form_error(id, &format!("no form `{form}` here")),
    }
}

fn form_error<T>(id: &str, what: &str) -> Result<T, KeysError> {
    Err(KeysError::Form(id.into(), what.into()))
}

/// The table of the executable's own strings (E17 M3 T7b-2b-3): `EXE:0x<address>` names the
/// string at that address of the user's `Heroes3.exe`, read at run time as the text tables are
/// (numbers only here, ADR-0009); a line the executable writes from a literal, not a table's row.
pub const EXECUTABLE: &str = "EXE";

/// The first field of the executable's string at `address` in the table [`EXECUTABLE`].
#[must_use]
pub fn address_field(address: u32) -> String {
    format!("{address:#X}")
}

/// `<TABLE>:<record>[:<column>]`, or `EXE:0x<address>` ([`EXECUTABLE`]).
fn source(id: &str, at: &str) -> Result<Source, KeysError> {
    if let Some(hex) = at.strip_prefix("EXE:0x") {
        return match u32::from_str_radix(hex, 16) {
            Ok(address) => Ok(Source {
                table: EXECUTABLE.into(),
                row: Row::Address(address),
                column: 1,
            }),
            Err(_) => form_error(id, &format!("`{at}` is not EXE:0x<address>")),
        };
    }
    let mut parts = at.split(':');
    let table = parts.next().unwrap_or_default();
    let numbers: Option<Vec<usize>> = parts.map(|p| p.parse().ok()).collect();
    match (table.is_empty(), numbers.as_deref()) {
        (false, Some(&[record])) => Ok(Source {
            table: table.into(),
            row: Row::Record(record),
            column: 0,
        }),
        (false, Some(&[record, column])) => Ok(Source {
            table: table.into(),
            row: Row::Record(record),
            column,
        }),
        _ => form_error(id, &format!("`{at}` is not <TABLE>:<record>[:<column>]")),
    }
}

/// Lower-case segments of letters, digits and `-`, joined by `.`.
fn valid_id(id: &str) -> bool {
    id.split('.').all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}
