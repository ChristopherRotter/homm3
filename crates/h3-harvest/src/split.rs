//! Splits a pack text into its two files (pack format v35, R01 decision 5): `core.toml` (the
//! `[pack]` manifest and every core table) and `battle.toml` (`[battle]`, the obstacles and the
//! battle fields of the creatures, the creature overrides and the spells). The harvester writes
//! one text per pack and routes it through [`split`]; the tables both files share keep their
//! identity fields (`id`, `target`, `fixes`) in both.
//!
//! Comment lines go with the field or table they precede (a table's leading comments go to every
//! file the table goes to), the text before the first table goes to both files, and a field's
//! value may span lines (open brackets continue it).

/// One field: its leading comment and blank lines, its `key = value` line and the lines that
/// continue the value.
struct Field<'a> {
    key: &'a str,
    lines: Vec<&'a str>,
}

/// One table: its leading lines, its header line and its fields.
struct Table<'a> {
    name: &'a str,
    leading: Vec<&'a str>,
    header: &'a str,
    fields: Vec<Field<'a>>,
}

/// Which file a field of a shared table goes to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Both,
    Core,
    Battle,
}

/// The tables only `battle.toml` holds.
const BATTLE_TABLES: [&str; 2] = ["battle", "obstacle"];

/// The tables whose fields split between the two files.
const SHARED_TABLES: [&str; 3] = ["creature", "creature_override", "spell"];

/// The battle fields of a creature and of a creature override.
const CREATURE_BATTLE_FIELDS: [&str; 7] = [
    "attack",
    "defense",
    "damage",
    "hit_points",
    "shots",
    "casts",
    "abilities",
];

/// The core fields of a spell (every other field is the battle's).
const SPELL_CORE_FIELDS: [&str; 8] = [
    "name",
    "category",
    "level",
    "schools",
    "cost",
    "power",
    "effect",
    "granted_by_name_only",
];

fn part(table: &str, key: &str) -> Part {
    let identity = if table == "creature_override" {
        matches!(key, "target" | "fixes")
    } else {
        key == "id"
    };
    if identity {
        Part::Both
    } else if table == "spell" {
        if SPELL_CORE_FIELDS.contains(&key) {
            Part::Core
        } else {
            Part::Battle
        }
    } else if CREATURE_BATTLE_FIELDS.contains(&key) {
        Part::Battle
    } else {
        Part::Core
    }
}

/// The net count of opening brackets on a line, outside strings and comments.
fn depth_change(line: &str) -> i32 {
    let mut depth = 0;
    let mut quote = None;
    let mut escaped = false;
    for c in line.chars() {
        match quote {
            Some(q) => {
                if escaped {
                    escaped = false;
                } else if c == '\\' && q == '"' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '#' => break,
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                _ => {}
            },
        }
    }
    depth
}

/// The table name of a header line (`[battle]`, `[[creature]]  # comment`).
fn table_name(header: &str) -> &str {
    let inner = header.trim_start_matches('[');
    inner.split(']').next().unwrap_or(inner).trim()
}

/// Splits a pack text into `(core.toml, battle.toml)`.
#[must_use]
pub fn split(text: &str) -> (String, String) {
    let mut preamble = Vec::new();
    let mut tables: Vec<Table<'_>> = Vec::new();
    let mut pending = Vec::new();
    let mut lines = text.split_inclusive('\n');
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            pending.push(line);
        } else if trimmed.starts_with('[') {
            if tables.is_empty() {
                // The file's opening comments go to both files, the blank lines after them stay
                // with the first table.
                let blank = pending.iter().rev().take_while(|l| l.trim().is_empty());
                let comments = pending.len() - blank.count();
                preamble.extend(pending.drain(..comments));
            }
            tables.push(Table {
                name: table_name(trimmed),
                leading: std::mem::take(&mut pending),
                header: line,
                fields: Vec::new(),
            });
        } else {
            let key = trimmed.split('=').next().unwrap_or(trimmed).trim();
            let mut field = std::mem::take(&mut pending);
            field.push(line);
            let mut depth = depth_change(line);
            while depth > 0 {
                let Some(next) = lines.next() else { break };
                field.push(next);
                depth += depth_change(next);
            }
            match tables.last_mut() {
                Some(t) => t.fields.push(Field { key, lines: field }),
                None => preamble.extend(field),
            }
        }
    }
    let mut core: String = preamble.concat();
    let mut battle = core.clone();
    for t in &tables {
        let write = |out: &mut String, keep: &dyn Fn(Part) -> bool| {
            out.push_str(&t.leading.concat());
            out.push_str(t.header);
            for f in t.fields.iter().filter(|f| keep(part(t.name, f.key))) {
                out.push_str(&f.lines.concat());
            }
        };
        if BATTLE_TABLES.contains(&t.name) {
            write(&mut battle, &|_| true);
        } else if SHARED_TABLES.contains(&t.name) {
            let has = |p: Part| t.fields.iter().any(|f| part(t.name, f.key) == p);
            let (with_core, with_battle) = (has(Part::Core), has(Part::Battle));
            // An override goes to the files it changes; one without changes stays in the core
            // (its target and gate are still checked).
            let override_ = t.name == "creature_override";
            if !override_ || with_core || !with_battle {
                write(&mut core, &|p| p != Part::Battle);
            }
            if !override_ || with_battle {
                write(&mut battle, &|p| p != Part::Core);
            }
        } else {
            write(&mut core, &|_| true);
        }
    }
    core.push_str(&pending.concat());
    (core, battle)
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn routes_tables_and_fields() {
        let text = "# head\n\n[pack]\nid = \"p\"\n\n# params\n[battle]\nfield_width = 17\n\n\
                    [[creature]]\nid = \"a\"  # note\nname = \"A\"\nattack = 4\n# its abilities\n\
                    abilities = [\n    { kind = \"x\", s = \"]\" },  # [\n]\nflags = []\n\n\
                    [[creature_override]]\ntarget = \"p:a\"\nfixes = \"q\"\nlevel = 2\n\n\
                    [[creature_override]]\ntarget = \"p:a\"\nattack = 5\n\n\
                    [[creature_override]]\ntarget = \"p:a\"\n\n\
                    [[spell]]\nid = \"s\"\nname = \"S\"\npolarity = \"neutral\"\ntarget = \"stack\"\n\
                    # tail\n";
        let (core, battle) = split(text);
        assert_eq!(
            core,
            "# head\n\n[pack]\nid = \"p\"\n\n[[creature]]\nid = \"a\"  # note\nname = \"A\"\n\
             flags = []\n\n[[creature_override]]\ntarget = \"p:a\"\nfixes = \"q\"\nlevel = 2\n\n\
             [[creature_override]]\ntarget = \"p:a\"\n\n[[spell]]\nid = \"s\"\nname = \"S\"\n\
             # tail\n"
        );
        assert_eq!(
            battle,
            "# head\n\n# params\n[battle]\nfield_width = 17\n\n[[creature]]\nid = \"a\"  # note\n\
             attack = 4\n# its abilities\nabilities = [\n    { kind = \"x\", s = \"]\" },  # [\n]\n\
             \n[[creature_override]]\ntarget = \"p:a\"\nattack = 5\n\n[[spell]]\nid = \"s\"\n\
             polarity = \"neutral\"\ntarget = \"stack\"\n"
        );
    }
}
