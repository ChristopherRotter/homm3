//! The string catalogue's tests (E17 M3 T4b-1), on invented text only: the original's never
//! enters git (ADR-0009).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};

use super::keys::{EXECUTABLE, Entry, Keys, KeysError, Row, Source, address_field};
use super::message::{Kind, Message, PrintfError, Segment};
use super::pack::{Pack, PackError, Problem, Tables};
use super::plural::{Category, Rule};
use super::{Args, Catalogue, Value, decode_1252, encode_1252};

fn s(text: &str) -> String {
    text.to_string()
}

fn lit(text: &str) -> Segment {
    Segment::Literal(s(text))
}

fn ph(name: &str) -> Segment {
    Segment::Placeholder(s(name))
}

#[test]
fn categories_by_name() {
    for c in Category::ALL {
        assert_eq!(Category::from_name(c.name()), Some(c));
    }
    assert_eq!(Category::from_name("zero"), None);
}

#[test]
fn rules_by_language() {
    let cases = [
        ("en", Some(Rule::OneOther)),
        ("de-AT", Some(Rule::OneOther)),
        ("hu", Some(Rule::OneOther)),
        ("es", Some(Rule::OneManyOther)),
        ("pt_BR", Some(Rule::French)),
        ("pt", Some(Rule::French)),
        // E17 M3 T9b, the reviewer: CLDR's `pt_PT`, `one` for 1 only.
        ("pt-PT", Some(Rule::OneManyOther)),
        ("pt_pt", Some(Rule::OneManyOther)),
        ("fr", Some(Rule::French)),
        ("pl", Some(Rule::Polish)),
        ("uk", Some(Rule::EastSlavic)),
        ("cs", Some(Rule::CzechSlovak)),
        ("ko", Some(Rule::OtherOnly)),
        ("ar", None),
        ("", None),
    ];
    for (language, rule) in cases {
        assert_eq!(Rule::of(language), rule, "{language}");
    }
}

/// CLDR's examples (`plurals.xml`'s `@integer` samples) for each rule.
#[test]
fn categories_of_counts() {
    use Category::{Few, Many, One, Other};
    let cases: [(Rule, &[(i64, Category)]); 8] = [
        (
            Rule::Original,
            &[(1, One), (0, Other), (2, Other), (-1, One)],
        ),
        (Rule::OneOther, &[(1, One), (0, Other), (21, Other)]),
        (
            Rule::OneManyOther,
            &[
                (1, One),
                (0, Other),
                (1_000, Other),
                (1_000_000, Many),
                (3_000_000, Many),
            ],
        ),
        (
            Rule::French,
            &[(0, One), (1, One), (2, Other), (2_000_000, Many)],
        ),
        (
            Rule::Polish,
            &[
                (1, One),
                (2, Few),
                (24, Few),
                (102, Few),
                (0, Many),
                (5, Many),
                (12, Many),
                (21, Many),
            ],
        ),
        (
            Rule::EastSlavic,
            &[
                (1, One),
                (21, One),
                (11, Many),
                (2, Few),
                (34, Few),
                (12, Many),
                (0, Many),
                (5, Many),
            ],
        ),
        (
            Rule::CzechSlovak,
            &[(1, One), (2, Few), (4, Few), (0, Other), (5, Other)],
        ),
        (Rule::OtherOnly, &[(0, Other), (1, Other)]),
    ];
    for (rule, counts) in cases {
        for &(n, category) in counts {
            assert_eq!(rule.category(n), category, "{rule:?} {n}");
        }
    }
}

#[test]
fn messages_parse_placeholders_and_keep_other_braces() {
    let m = Message::parse("The {{attacker}} do {{ damage }} damage.");
    assert_eq!(
        m.segments,
        vec![
            lit("The "),
            ph("attacker"),
            lit(" do "),
            ph("damage"),
            lit(" damage.")
        ]
    );
    // Highlight braces stay text, also around a placeholder.
    let m = Message::parse("{Phase} {{{name}}}");
    assert_eq!(m.segments, vec![lit("{Phase} {"), ph("name"), lit("}")]);
    // Not names: upper case, a leading digit, empty, unclosed.
    for text in ["{{Name}}", "{{2x}}", "{{}}", "{{a-b}}", "{{open"] {
        assert_eq!(Message::parse(text).segments, vec![lit(text)], "{text}");
    }
    assert_eq!(Message::parse("{{a_1}}").segments, vec![ph("a_1")]);
    assert_eq!(Message::parse("").segments, Vec::<Segment>::new());
    let two = Message::parse("{{a}} x {{b}}");
    assert_eq!(two.placeholders().collect::<Vec<_>>(), ["a", "b"]);
}

fn kinds(name: &str) -> Option<Kind> {
    match name {
        "who" => Some(Kind::Text),
        "n" => Some(Kind::Number),
        _ => None,
    }
}

#[test]
fn printf_rows_name_their_specifiers() {
    let names = [s("who"), s("n")];
    let m = Message::from_printf("%s hits 50%% for %d.", &names, kinds).unwrap();
    assert_eq!(
        m.segments,
        vec![ph("who"), lit(" hits 50% for "), ph("n"), lit(".")]
    );
    let m = Message::from_printf("plain", &[], kinds).unwrap();
    assert_eq!(m.segments, vec![lit("plain")]);
    let cases = [
        ("%x", PrintfError::Specifier(Some('x'))),
        ("end %", PrintfError::Specifier(None)),
        ("%s %d %s", PrintfError::TooMany(3, 2)),
        ("%s", PrintfError::TooFew(1, 2)),
        ("%d %d", PrintfError::Kind(s("who"), Kind::Number)),
    ];
    for (row, error) in cases {
        assert_eq!(
            Message::from_printf(row, &names, kinds),
            Err(error),
            "{row}"
        );
    }
    // An argument the key does not have.
    assert_eq!(
        Message::from_printf("%s", &[s("nobody")], kinds),
        Err(PrintfError::Kind(s("nobody"), Kind::Text))
    );
}

#[test]
fn messages_write_values_and_show_missing_ones() {
    let m = Message::parse("{{a}}-{{b}}");
    let out = m.write(|n| (n == "a").then(|| s("x")));
    assert_eq!(out, "x-{{b}}");
}

const KEYS: &str = r#"
[[key]]
id = "battle.log.hit"
args = { who = "text", n = "number" }
plural = true
original = [
    { form = "one", at = "T:2", printf = ["who", "n"] },
    { form = "other", at = "T:3:1", printf = ["who", "n"] },
]

[[key]]
id = "pronoun"
contexts = ["male", "female"]
original = [{ form = "male", at = "T:4" }, { form = "female", at = "T:5" }]

[[key]]
id = "own.title"

[[family]]
id = "creature"
table = "C"
first = 1
forms = { one = 0, other = 1 }
"#;

fn entries() -> BTreeMap<String, Vec<Entry>> {
    let list = vec![
        Entry {
            key: s("base:imp"),
            id: 0,
        },
        Entry {
            key: s("base:ogre"),
            id: 1,
        },
    ];
    BTreeMap::from([(s("creature"), list)])
}

fn keys() -> Keys {
    Keys::parse(KEYS, &entries()).unwrap()
}

#[test]
fn keys_resolve_forms_and_families() {
    let keys = keys();
    let ids: Vec<&str> = keys.keys.keys().map(String::as_str).collect();
    assert_eq!(
        ids,
        [
            "battle.log.hit",
            "creature.base:imp",
            "creature.base:ogre",
            "own.title",
            "pronoun"
        ]
    );
    let hit = &keys.keys["battle.log.hit"];
    assert!(hit.plural);
    assert_eq!(hit.args["count"], Kind::Number);
    assert_eq!(hit.args["who"], Kind::Text);
    assert_eq!(
        hit.original[1].1,
        Source {
            table: s("T"),
            row: Row::Record(3),
            column: 1,
            raw: false
        }
    );
    let ogre = &keys.keys["creature.base:ogre"];
    assert_eq!(
        ogre.original[0],
        (
            s("one"),
            Source {
                table: s("C"),
                row: Row::Id {
                    first: 1,
                    id: 1,
                    by: None
                },
                column: 0,
                raw: false
            },
            Vec::new()
        )
    );
    assert!(keys.keys["own.title"].original.is_empty());
    assert_eq!(
        Keys::names("pronoun", &keys.keys["pronoun"]),
        ["pronoun", "pronoun_male", "pronoun_female"]
    );
    let names = Keys::names("x", &keys.keys["battle.log.hit"]);
    assert_eq!(names, ["x_one", "x_few", "x_many", "x_other", "x"]);
}

#[test]
fn keys_refuse_mistakes() {
    let one = |body: &str| Keys::parse(&format!("[[key]]\n{body}"), &entries()).unwrap_err();
    let form = |id: &str, what: &str| KeysError::Form(s(id), s(what));
    assert!(matches!(one("id = 3"), KeysError::Parse(_)));
    assert_eq!(one("id = \"A.b\""), KeysError::Id(s("A.b")));
    assert_eq!(one("id = \"a..b\""), KeysError::Id(s("a..b")));
    assert_eq!(
        one("id = \"a\"\nplural = true\nargs = { count = \"text\" }"),
        form("a", "`count` is a plural key's number")
    );
    assert_eq!(
        one("id = \"a\"\noriginal = [{ form = \"one\", at = \"T:1\" }]"),
        form("a", "no form `one` here")
    );
    assert_eq!(
        one("id = \"a\"\nplural = true\noriginal = [{ at = \"T:1\" }]"),
        form("a", "no form `` here")
    );
    assert_eq!(
        one("id = \"a\"\ncontexts = [\"male\"]\noriginal = [{ form = \"female\", at = \"T:1\" }]"),
        form("a", "no form `female` here")
    );
    assert_eq!(
        one("id = \"a\"\noriginal = [{ at = \"T:1\", printf = [\"who\"] }]"),
        form("a", "`who` is not an argument")
    );
    for at in ["T", ":1", "T:x", "T:1:2:3"] {
        let body = format!("id = \"a\"\noriginal = [{{ at = \"{at}\" }}]");
        assert_eq!(
            one(&body),
            form("a", &format!("`{at}` is not <TABLE>:<record>[:<column>]")),
            "{at}"
        );
    }
    let twice = "[[key]]\nid = \"a\"\n[[key]]\nid = \"a\"";
    assert_eq!(
        Keys::parse(twice, &entries()),
        Err(KeysError::Duplicate(s("a")))
    );
    let clash = "[[key]]\nid = \"creature.base:imp\"\n[[family]]\nid = \"creature\"\ntable = \"C\"\nfirst = 0\nforms = { one = 0 }";
    assert!(matches!(
        Keys::parse(clash, &entries()),
        Err(KeysError::Id(_))
    ));
    let clash = "[[key]]\nid = \"creature.base\"\n[[family]]\nid = \"creature\"\ntable = \"C\"\nfirst = 0\nforms = { one = 0 }";
    let both = BTreeMap::from([(
        s("creature"),
        vec![Entry {
            key: s("base"),
            id: 0,
        }],
    )]);
    assert_eq!(
        Keys::parse(clash, &both),
        Err(KeysError::Duplicate(s("creature.base")))
    );
    let family = |forms: &str| {
        format!("[[family]]\nid = \"creature\"\ntable = \"C\"\nfirst = 0\nforms = {{ {forms} }}")
    };
    assert_eq!(
        Keys::parse(&family("male = 0"), &entries()),
        Err(form("creature.base:imp", "no form `male` here"))
    );
    let unknown = family("one = 0").replace("\"creature\"", "\"spell\"");
    assert_eq!(
        Keys::parse(&unknown, &entries()),
        Err(KeysError::Family(s("spell")))
    );
    // A family only some clients list (E24 M3 T5b1): none of its keys without entries, its
    // keys with them.
    let optional = format!("{unknown}\noptional = true");
    assert_eq!(Keys::parse(&optional, &entries()), Ok(Keys::default()));
    let listed = optional.replace("\"spell\"", "\"creature\"");
    let keys = Keys::parse(&listed, &entries()).unwrap();
    assert!(keys.keys.contains_key("creature.base:imp"));
    let shown: Vec<String> = [
        KeysError::Parse(s("x")),
        KeysError::Id(s("A")),
        KeysError::Duplicate(s("a")),
        form("a", "why"),
        KeysError::Family(s("f")),
    ]
    .iter()
    .map(|e| format!("{e}"))
    .collect();
    assert_eq!(
        shown,
        [
            "not a key catalogue: x",
            "`A` is not a key id",
            "`a` is defined twice",
            "`a`: why",
            "no family `f`",
        ]
    );
}

/// Invented tables: `T` (records 2–5 the key's forms) and `C` (a header, then the creatures by id
/// with a blank record between).
fn tables() -> Tables {
    let row = |fields: &[&str]| fields.iter().map(|f| s(f)).collect::<Vec<_>>();
    let t = vec![
        row(&["head"]),
        row(&[""]),
        row(&["%s hits for %d!"]),
        row(&["x", "%s hit for %d!"]),
        row(&["his"]),
        // A run of quotes is one, as the executable reads a table (E17 M5 T4a).
        row(&["\"\"her\"\"\""]),
    ];
    let c = vec![
        row(&["Singular", "Plural"]),
        row(&["Imp", "Imps"]),
        row(&["", " "]),
        row(&["Ogre", "Ogres"]),
    ];
    BTreeMap::from([(s("T"), t), (s("C"), c)])
}

fn original() -> Pack {
    let tables = tables();
    let (pack, problems) = Pack::original(&keys(), &tables);
    assert_eq!(problems, Vec::<Problem>::new());
    pack
}

#[test]
fn the_original_pack_reads_the_tables() {
    let pack = original();
    assert_eq!(pack.rule, Rule::Original);
    let names: Vec<&str> = pack.messages.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        [
            "battle.log.hit_one",
            "battle.log.hit_other",
            "creature.base:imp_one",
            "creature.base:imp_other",
            "creature.base:ogre_one",
            "creature.base:ogre_other",
            "pronoun_female",
            "pronoun_male",
        ]
    );
    assert_eq!(
        pack.messages["battle.log.hit_other"].segments,
        vec![ph("who"), lit(" hit for "), ph("n"), lit("!")]
    );
    assert_eq!(
        pack.messages["creature.base:ogre_other"].segments,
        vec![lit("Ogres")]
    );
    assert_eq!(
        pack.messages["pronoun_female"].segments,
        vec![lit("\"her\"")]
    );
    assert_eq!(pack.check(&keys()), Vec::<Problem>::new());
}

#[test]
fn the_original_pack_lists_what_it_cannot_read() {
    let catalogue = r#"
[[key]]
id = "a"
original = [{ at = "NONE:0" }]
[[key]]
id = "b"
original = [{ at = "T:99" }]
[[key]]
id = "c"
original = [{ at = "T:2:5" }]
[[key]]
id = "d"
original = [{ at = "T:2" }]
[[family]]
id = "creature"
table = "C"
first = 1
forms = { one = 0 }
"#;
    let many = BTreeMap::from([(
        s("creature"),
        vec![Entry {
            key: s("base:x"),
            id: 7,
        }],
    )]);
    let keys = Keys::parse(catalogue, &many).unwrap();
    let tables = tables();
    let (pack, problems) = Pack::original(&keys, &tables);
    assert!(pack.messages.is_empty());
    let shown: Vec<String> = problems.iter().map(|p| format!("{p}")).collect();
    assert_eq!(
        shown,
        [
            "`a`: no table NONE.TXT",
            "`b`: T.TXT has no such record (Record(99))",
            "`c`: T.TXT's record has no column 5",
            "`creature.base:x_one`: C.TXT has no such record (Id { first: 1, id: 7, by: None })",
            "`d`: TooMany(1, 0) in \"%s hits for %d!\"",
        ]
    );
}

#[test]
fn packs_read_nested_json() {
    let json = r#"{ "battle": { "log": { "hit_one": "{{who}} trifft", "hit_other": "{{who}} treffen" } },
                   "own": { "title": "Titel" } }"#;
    let pack = Pack::from_json("de", json).unwrap();
    assert_eq!(pack.rule, Rule::OneOther);
    assert_eq!(pack.messages["own.title"].segments, vec![lit("Titel")]);
    assert_eq!(pack.messages.len(), 3);
    assert_eq!(pack.check(&keys()), Vec::<Problem>::new());
    // A top-level string is a message named by its key alone.
    let flat = Pack::from_json("en", r#"{ "own.title": "T" }"#).unwrap();
    assert!(flat.messages.contains_key("own.title"));
    assert_eq!(
        Pack::from_json("xx", "{}"),
        Err(PackError::Language(s("xx")))
    );
    assert!(matches!(
        Pack::from_json("en", "{"),
        Err(PackError::Json(_))
    ));
    assert_eq!(
        Pack::from_json("en", r#"{ "a": { "b": 1 } }"#),
        Err(PackError::Entry(s("a.b")))
    );
    assert_eq!(Pack::from_json("en", "[]"), Err(PackError::Entry(s(""))));
    let shown: Vec<String> = [
        PackError::Language(s("xx")),
        PackError::Json(s("eof")),
        PackError::Entry(s("a.b")),
    ]
    .iter()
    .map(|e| format!("{e}"))
    .collect();
    assert_eq!(
        shown,
        [
            "no plural rule for the language `xx`",
            "not JSON: eof",
            "`a.b` is neither an object nor a string",
        ]
    );
}

#[test]
fn packs_are_checked_against_the_keys() {
    let json = r#"{ "nothing": "x", "own": { "title": "{{who}}" },
                   "pronoun_male": "{{context}}", "battle": { "log": { "hit_few": "{{count}} {{n}}" } } }"#;
    let pack = Pack::from_json("pl", json).unwrap();
    let shown: Vec<String> = pack.check(&keys()).iter().map(|p| format!("{p}")).collect();
    assert_eq!(
        shown,
        [
            "`nothing`: no key has this name",
            "`own.title`: `{{who}}` is not an argument",
        ]
    );
}

fn catalogue() -> Catalogue {
    let player = r#"{ "battle": { "log": { "hit_few": "{{n}} by {{count}} few {{who}}" } },
                      "pronoun_female": "she ({{context}})" }"#;
    Catalogue {
        packs: vec![Pack::from_json("pl", player).unwrap(), original()],
    }
}

#[test]
fn the_catalogue_looks_up_by_priority_and_form() {
    let c = catalogue();
    let hit = |count| Args::new().text("who", "Imps").number("n", -4).count(count);
    // The player's pack has the Polish `few` (3); for 5 (`many`) it has nothing, so the
    // original's `other` wins; for 1 the original's `one`.
    assert_eq!(c.text("battle.log.hit", &hit(3)), "-4 by 3 few Imps");
    assert_eq!(c.text("battle.log.hit", &hit(5)), "Imps hit for -4!");
    assert_eq!(c.text("battle.log.hit", &hit(1)), "Imps hits for -4!");
    // Without a count only the bare name is tried: no pack has it.
    let none = Args::new().text("who", "Imp");
    assert_eq!(c.text("battle.log.hit", &none), "battle.log.hit");
    // Contexts.
    let female = Args::new().context("female");
    assert_eq!(c.text("pronoun", &female), "she (female)");
    assert_eq!(c.text("pronoun", &Args::new().context("male")), "his");
    assert_eq!(c.text("pronoun", &Args::new()), "pronoun");
    // A family's names.
    assert_eq!(c.text("creature.base:imp", &Args::new().count(2)), "Imps");
    // An unknown key shows itself.
    assert_eq!(c.text("no.such.key", &Args::new()), "no.such.key");
}

#[test]
fn the_names_tried_and_their_order() {
    assert_eq!(
        super::names("k", Some("male"), Some(Category::Few)),
        [
            "k_male_few",
            "k_male_other",
            "k_male",
            "k_few",
            "k_other",
            "k"
        ]
    );
    assert_eq!(
        super::names("k", None, Some(Category::Other)),
        ["k_other", "k"]
    );
    assert_eq!(super::names("k", None, None), ["k"]);
}

#[test]
fn arguments_are_written() {
    let args = Args::new()
        .text("a", "x")
        .number("n", 12)
        .number("a", 7)
        .text("count", "not a number")
        .context("c");
    // The last value of a name wins; a text `count` picks no form.
    assert_eq!(args.written("a"), Some(s("7")));
    assert_eq!(args.written("n"), Some(s("12")));
    assert_eq!(args.written("context"), Some(s("c")));
    assert_eq!(args.written("none"), None);
    assert_eq!(args.count_value(), None);
    assert_eq!(Args::new().written("context"), None);
    assert_eq!(args.get("n"), Some(&Value::Number(12)));
}

#[test]
fn windows_1252_encoding() {
    assert_eq!(encode_1252("Aé€™"), (vec![0x41, 0xE9, 0x80, 0x99], vec![]));
    assert_eq!(encode_1252("Łx"), (vec![b'?', b'x'], vec!['Ł']));
    // Every byte round-trips through the decoder (E24 M3 T5b1: `decode_1252`).
    for byte in 0..=255_u8 {
        let text = decode_1252(&[byte]);
        assert_eq!(text.chars().count(), 1);
        assert_eq!(encode_1252(&text), (vec![byte], vec![]), "{byte:#x}");
    }
    assert_eq!(decode_1252(&[0x41, 0xE9, 0x80, 0x99]), "Aé€™");
}

#[test]
fn a_single_form_family_counts_the_records_its_column_fills() {
    // E17 M3 T7b-1: SPTRAITS.TXT's group rows fill only their first column, so `by` counts the
    // records whose second is filled; `column` gives the keys' one form.
    let catalogue = "[[family]]\nid = \"spell\"\ntable = \"S\"\nfirst = 1\nby = 1\ncolumn = 0\n";
    let entry = |key: &str, id| Entry { key: s(key), id };
    let list = vec![entry("base:arrow", 0), entry("base:bolt", 1)];
    let entries = BTreeMap::from([(s("spell"), list)]);
    let keys = Keys::parse(catalogue, &entries).unwrap();
    let bolt = &keys.keys["spell.base:bolt"];
    assert!(!bolt.plural);
    assert!(bolt.args.is_empty());
    let row = |fields: &[&str]| fields.iter().map(|f| s(f)).collect::<Vec<_>>();
    let rows = vec![
        row(&["head"]),
        row(&["Group"]),
        row(&["Arrow", "A"]),
        row(&["Group 2", ""]),
        row(&["Bolt", "B"]),
    ];
    let tables = BTreeMap::from([(s("S"), rows)]);
    let (pack, problems) = Pack::original(&keys, &tables);
    assert_eq!(problems, Vec::<Problem>::new());
    assert_eq!(
        pack.messages["spell.base:arrow"].segments,
        vec![lit("Arrow")]
    );
    assert_eq!(pack.messages["spell.base:bolt"].segments, vec![lit("Bolt")]);
    // Two of forms, contexts and a column, or none of them: refused.
    for body in [
        "forms = { one = 0 }\ncolumn = 0",
        "contexts = { none = 0 }\ncolumn = 0",
        "forms = { one = 0 }\ncontexts = { none = 0 }",
        "",
    ] {
        let text = format!("[[family]]\nid = \"spell\"\ntable = \"S\"\nfirst = 1\n{body}\n");
        assert_eq!(
            Keys::parse(&text, &entries).unwrap_err(),
            KeysError::Form(
                s("spell"),
                s("a family has forms, contexts or a column, one of them")
            )
        );
    }
}

#[test]
fn a_family_of_contexts_over_another_family_s_list() {
    // E17 M5 T3a: a spell's description by the hero's mastery (SPTRAITS.TXT's columns 29 to 32),
    // its keys made for the spells' list.
    let catalogue = "[[family]]\nid = \"spell-text\"\nlist = \"spell\"\ntable = \"S\"\nfirst = 0\nraw = true\ncontexts = { none = 1, expert = 2 }\n";
    let entries = BTreeMap::from([(
        s("spell"),
        vec![Entry {
            key: s("base:arrow"),
            id: 0,
        }],
    )]);
    let keys = Keys::parse(catalogue, &entries).unwrap();
    let arrow = &keys.keys["spell-text.base:arrow"];
    assert!(!arrow.plural);
    assert_eq!(arrow.contexts, [s("expert"), s("none")]);
    let row = |fields: &[&str]| fields.iter().map(|f| s(f)).collect::<Vec<_>>();
    // Raw: a percent sign is text (0x59BAA0 shows the record as it is).
    let tables = BTreeMap::from([(
        s("S"),
        vec![row(&["Arrow", "{Arrow}", "{Expert Arrow} 5%"])],
    )]);
    let (pack, problems) = Pack::original(&keys, &tables);
    assert_eq!(problems, Vec::<Problem>::new());
    let catalogue = Catalogue { packs: vec![pack] };
    let text = |context| catalogue.text("spell-text.base:arrow", &Args::new().context(context));
    assert_eq!(
        (text("none"), text("expert")),
        (s("{Arrow}"), s("{Expert Arrow} 5%"))
    );
    // A list nobody gave: refused.
    let other = catalogue_with_list("nothing");
    assert_eq!(
        Keys::parse(&other, &entries).unwrap_err(),
        KeysError::Family(s("nothing"))
    );
}

fn catalogue_with_list(list: &str) -> String {
    format!("[[family]]\nid = \"t\"\nlist = \"{list}\"\ntable = \"S\"\nfirst = 0\ncolumn = 0\n")
}

#[test]
fn the_executables_own_strings_are_a_table_by_address() {
    // E17 M3 T7b-2b-3: `EXE:0x<address>` names a string of the executable (Acid Breath's line):
    // the table `EXE`'s record whose first field is the address, its text the second; `%i` is a
    // number as `%d` is.
    let catalogue = r#"
[[key]]
id = "acid"
args = { who = "text", n = "number" }
original = [{ at = "EXE:0x688450", printf = ["who", "n"] }]
[[key]]
id = "gone"
original = [{ at = "EXE:0x10" }]
"#;
    let keys = Keys::parse(catalogue, &entries()).unwrap();
    let row = |fields: &[&str]| fields.iter().map(|f| s(f)).collect::<Vec<_>>();
    let exe = vec![row(&[&address_field(0x68_8450), "Acid on %s: -%i"])];
    assert_eq!(address_field(0x68_8450), "0x688450");
    let tables = BTreeMap::from([(s(EXECUTABLE), exe)]);
    let (pack, problems) = Pack::original(&keys, &tables);
    assert_eq!(
        pack.messages["acid"].segments,
        vec![lit("Acid on "), ph("who"), lit(": -"), ph("n")]
    );
    let shown: Vec<String> = problems.iter().map(|p| format!("{p}")).collect();
    assert_eq!(shown, ["`gone`: EXE.TXT has no such record (Address(16))"]);
    // An address that is no hexadecimal number.
    let bad = Keys::parse(
        "[[key]]\nid = \"a\"\noriginal = [{ at = \"EXE:0xZZ\" }]",
        &entries(),
    );
    assert_eq!(
        bad.unwrap_err(),
        KeysError::Form(s("a"), s("`EXE:0xZZ` is not EXE:0x<address>"))
    );
}
