//! The string catalogue in the adapters (E17 M3 T4b-1, `docs/epics/E17-design.md` §9): the key
//! catalogue `clients/text/keys.toml`, the original's text read from the installation's text
//! tables through it, a player's language pack, and the command `h3-view text`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_formats::text::TextTable;
use h3_ui::text::keys::{EXECUTABLE, Entry, Keys, Row, address_field};
use h3_ui::text::pack::{Pack, Problem, Tables};
use h3_ui::text::{Args, Catalogue};
use h3_view::battle_log::{self, Hit, Spread, Stack};
use h3_view::tables::{SIEGE_PARTS, Tables as AssetTables};

use crate::assets;
use crate::gate::find_ci;

/// The committed key catalogue.
pub fn default_keys() -> PathBuf {
    assets::clients_dir().join("text").join("keys.toml")
}

/// The families' entries: the asset tables' creatures, spells and heroes (E17 M3 T7b-1), the
/// towns' moats by their faction (E17 M4 T2c-1) and their wall-shot targets' parts (E17 M4 T2c-2:
/// `<faction>.<part>`, the town type's 18 parts in order), by qualified key, with their original
/// ids.
pub fn entries(tables: &AssetTables) -> BTreeMap<String, Vec<Entry>> {
    let entry = |key: &String, original: u16| Entry {
        key: key.clone(),
        id: usize::from(original),
    };
    let creatures = tables
        .creatures
        .iter()
        .map(|(key, c)| entry(key, c.original))
        .collect();
    let spells = tables
        .spells
        .iter()
        .map(|(key, s)| entry(key, s.original))
        .collect();
    let heroes = tables
        .heroes
        .iter()
        .map(|(key, h)| entry(key, h.original))
        .collect();
    let moats = tables
        .towns
        .iter()
        .map(|(key, t)| entry(key, u16::from(t.original)))
        .collect();
    let walls = tables
        .towns
        .iter()
        .flat_map(|(key, t)| {
            h3_view::siege::TARGET_PARTS.map(|part| Entry {
                key: format!("{key}.{part}"),
                id: usize::from(t.original) * SIEGE_PARTS + part,
            })
        })
        .collect();
    BTreeMap::from([
        (String::from("creature"), creatures),
        (String::from("hero"), heroes),
        (String::from("moat"), moats),
        (String::from("spell"), spells),
        (String::from("wall"), walls),
    ])
}

/// Reads the key catalogue at `path`, its families over `tables`.
pub fn read_keys(path: &Path, tables: &AssetTables) -> Result<Keys, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Keys::parse(&text, &entries(tables)).map_err(|e| format!("{}: {e}", path.display()))
}

/// The user's `Heroes3.exe` in the installation `install`, whose own strings some keys name (the
/// table [`EXECUTABLE`], E17 M3 T7b-2b-3).
pub fn executable(install: &Path) -> Result<Vec<u8>, String> {
    let path = find_ci(install, "Heroes3.exe")
        .map_err(|e| format!("{}: Heroes3.exe: {e}", install.display()))?;
    fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The executable's string at file offset `at` (E17 M5 T3a: `h3_formats`' `c_string` with line
/// breaks, which the spell book's formats hold): at most 256 bytes to a NUL, each printable
/// ASCII, a space or a line break; none for any other or without the NUL.
fn exe_string(exe: &[u8], at: Option<usize>) -> Option<String> {
    let tail = exe.get(at?..)?;
    let len = tail.iter().take(256).position(|&b| b == 0)?;
    let bytes = &tail[..len];
    bytes
        .iter()
        .all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\n'))
        .then(|| bytes.iter().map(|&b| char::from(b)).collect())
}

/// The text tables the keys name, each entry's bytes from `entry` (by its name, `GENRLTXT.TXT`);
/// the executable's strings they name from `exe`'s bytes (the table [`EXECUTABLE`]: a record of
/// each string's address and its text; a build `h3_formats::exe` knows).
pub fn tables(
    keys: &Keys,
    entry: impl Fn(&str) -> Result<Vec<u8>, String>,
    exe: &[u8],
) -> Result<Tables, String> {
    let sources = || {
        keys.keys
            .values()
            .flat_map(|k| k.original.iter().map(|(_, source, _)| source))
    };
    let names: BTreeSet<&str> = sources().map(|source| source.table.as_str()).collect();
    let mut tables = Tables::new();
    for name in names {
        if name == EXECUTABLE {
            let (_, image) =
                h3_formats::exe::identify(exe).map_err(|e| format!("Heroes3.exe: {e:?}"))?;
            let mut records = Vec::new();
            for source in sources() {
                if let Row::Address(va) = source.row {
                    let text = exe_string(exe, image.file_offset(va))
                        .ok_or_else(|| format!("Heroes3.exe: 0x{va:X}: not a string"))?;
                    records.push(vec![address_field(va), text]);
                }
            }
            tables.insert(name.to_owned(), records);
            continue;
        }
        let file = format!("{name}.TXT");
        let table = TextTable::parse(&entry(&file)?).map_err(|e| format!("{file}: {e:?}"))?;
        tables.insert(name.to_owned(), table.records);
    }
    Ok(tables)
}

/// A player's language pack: its language is its file's name (`de.json`, `pt-BR.json`).
pub fn read_pack(path: &Path, keys: &Keys) -> Result<(Pack, Vec<Problem>), String> {
    let language = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("{}: no language in the name", path.display()))?;
    let json = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let pack = Pack::from_json(language, &json).map_err(|e| format!("{}: {e}", path.display()))?;
    let problems = pack.check(keys);
    Ok((pack, problems))
}

/// The bitmap archives of `install`, read whole (CLT-Q2), in the game's search order (§3.1).
fn bitmap_archives(install: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let data = find_ci(install, "Data").map_err(|e| format!("{}: {e}", install.display()))?;
    ["H3bitmap.lod", "H3ab_bmp.lod"]
        .iter()
        .map(|name| {
            let path =
                find_ci(&data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(((*name).to_owned(), bytes))
        })
        .collect()
}

/// The entry `name` from the first of `archives` that holds it.
fn first_holding(archives: &[(&str, LodArchive<'_>)], name: &str) -> Result<Vec<u8>, String> {
    for (archive_name, archive) in archives {
        if let Some(bytes) = archive
            .read_by_name(name)
            .map_err(|e| format!("{archive_name}:{name}: {e:?}"))?
        {
            return Ok(bytes);
        }
    }
    Err(format!("{name}: in no bitmap archive"))
}

/// `h3-view text`'s options.
#[derive(Debug)]
pub struct Options {
    install: PathBuf,
    tables: PathBuf,
    keys: PathBuf,
    pack: Option<PathBuf>,
    key: Option<String>,
    args: Vec<(String, String)>,
    log: bool,
}

impl Options {
    /// Parses `[<key> [<name>=<value>]...] [--log] [--pack <file>] [--install <dir>]
    /// [--tables <dir>] [--keys <file>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut options = Self {
            install: env.unwrap_or_else(|| manifest.join("../../../original-data/install")),
            tables: assets::default_dir(),
            keys: default_keys(),
            pack: None,
            key: None,
            args: Vec::new(),
            log: false,
        };
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = |what: &str| {
                rest.next()
                    .map(PathBuf::from)
                    .ok_or_else(|| format!("text: `{arg}` needs {what}"))
            };
            match arg.as_str() {
                "--install" => options.install = value("a directory")?,
                "--tables" => options.tables = value("a directory")?,
                "--keys" => options.keys = value("a file")?,
                "--pack" => options.pack = Some(value("a file")?),
                "--log" => options.log = true,
                flag if flag.starts_with("--") => {
                    return Err(format!("text: unknown option `{flag}`"));
                }
                _ if options.key.is_none() => options.key = Some(arg.clone()),
                _ => {
                    let (name, v) = arg
                        .split_once('=')
                        .ok_or_else(|| format!("text: `{arg}` is not <name>=<value>"))?;
                    options.args.push((name.to_owned(), v.to_owned()));
                }
            }
        }
        Ok(options)
    }
}

/// The arguments as given: a value that parses as a whole number is a number, `context` the
/// context.
fn arguments(given: &[(String, String)]) -> Args {
    given.iter().fold(Args::new(), |args, (name, value)| {
        match (name.as_str(), value.parse::<i64>()) {
            ("context", _) => args.context(value),
            (_, Ok(n)) => args.number(name, n),
            (_, Err(_)) => args.text(name, value),
        }
    })
}

/// Runs `h3-view text`: every key's forms from the original (and the player's pack), or one
/// key's text.
pub fn run(options: &Options) -> ExitCode {
    match text(options) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("text: {message}");
            ExitCode::FAILURE
        }
    }
}

fn text(options: &Options) -> Result<Vec<String>, String> {
    let assets = assets::load(&options.tables, assets::ALL)?;
    let keys = read_keys(&options.keys, &assets)?;
    let bytes = bitmap_archives(&options.install)?;
    let mut archives = Vec::new();
    for (name, b) in &bytes {
        archives.push((
            name.as_str(),
            LodArchive::parse(b).map_err(|e| format!("{name}: {e:?}"))?,
        ));
    }
    let exe = executable(&options.install)?;
    let texts = tables(&keys, |name| first_holding(&archives, name), &exe)?;
    let (original, problems) = Pack::original(&keys, &texts);
    let mut lines: Vec<String> = problems.iter().map(|p| format!("original: {p}")).collect();
    let mut catalogue = Catalogue::default();
    if let Some(path) = &options.pack {
        let (pack, problems) = read_pack(path, &keys)?;
        lines.extend(problems.iter().map(|p| format!("{}: {p}", path.display())));
        catalogue.packs.push(pack);
    }
    catalogue.packs.push(original);
    if options.log {
        lines.extend(
            log_samples(&catalogue)
                .into_iter()
                .map(|(what, line)| format!("{what}\t{}", line.replace('\n', "\\n"))),
        );
        return Ok(lines);
    }
    match &options.key {
        Some(key) => lines.push(catalogue.text(key, &arguments(&options.args))),
        None => {
            for pack in &catalogue.packs {
                for (name, message) in &pack.messages {
                    let shown = message.write(|_| None).replace('\n', "\\n");
                    lines.push(format!("{name}\t{shown}"));
                }
            }
        }
    }
    Ok(lines)
}

/// The battle log's lines (`h3_view::battle_log`, E17 M3 T4b-2a; the heal's and the round's,
/// T6b) for a few stacks of the asset tables: each composer once or twice, by its name.
pub fn log_samples(catalogue: &Catalogue) -> Vec<(&'static str, String)> {
    let stack = |key, count| Stack {
        creature: key,
        count,
    };
    let hit = |key, damage, kills| Hit {
        creature: key,
        war_machine: false,
        damage,
        kills,
    };
    let (wolves, pikemen, ballista, tent) = (
        "creature.base:wolf_rider",
        "creature.base:pikeman",
        "creature.base:ballista",
        "creature.base:first_aid_tent",
    );
    let c = catalogue;
    let strike = |attacker, spread, hits: &[Hit<'_>]| battle_log::strike(c, attacker, spread, hits);
    vec![
        (
            "strike",
            strike(stack(wolves, 4), Spread::Target, &[hit(pikemen, 37, 1)]),
        ),
        (
            "strike",
            strike(stack(wolves, 1), Spread::Target, &[hit(pikemen, 5, 0)]),
        ),
        (
            "strike",
            strike(stack(pikemen, 10), Spread::Target, &[hit(wolves, 60, 3)]),
        ),
        (
            "strike",
            strike(
                stack(wolves, 20),
                Spread::Target,
                &[Hit {
                    war_machine: true,
                    ..hit(ballista, 250, 1)
                }],
            ),
        ),
        (
            "strike",
            strike(
                stack(wolves, 6),
                Spread::Around,
                &[hit(pikemen, 20, 2), hit(wolves, 30, 1)],
            ),
        ),
        ("wait", battle_log::wait(c, stack(pikemen, 1))),
        ("wait", battle_log::wait(c, stack(pikemen, 12))),
        ("defend", battle_log::defend(c, stack(pikemen, 1), 1)),
        ("defend", battle_log::defend(c, stack(wolves, 8), 2)),
        ("morale-good", battle_log::morale_good(c, stack(wolves, 8))),
        ("morale-bad", battle_log::morale_bad(c, stack(pikemen, 1))),
        ("luck-good", battle_log::luck_good(c, stack(wolves, 8))),
        // E17 M3 T7b-2b-2b: Fear, the frightening creature by its number.
        (
            "fear",
            battle_log::fear(c, stack(wolves, 1), stack(pikemen, 12)),
        ),
        (
            "fear",
            battle_log::fear(c, stack(wolves, 2), stack(pikemen, 1)),
        ),
        (
            "heal",
            battle_log::heal(c, stack(tent, 1), stack(pikemen, 12), 17),
        ),
        ("round", c.text("battle.log.round", &Args::new())),
        // E17 M3 T7b-1: a hero's cast on a stack, a resisted one, a spell's damage.
        (
            "cast-on",
            battle_log::cast_on(
                c,
                &c.text("hero.base:orrin", &Args::new()),
                &c.text("spell.base:slow", &Args::new()),
                stack(wolves, 8),
            ),
        ),
        ("resisted", battle_log::resisted(c, stack(pikemen, 1))),
        ("resisted", battle_log::resisted(c, stack(wolves, 8))),
        (
            "strike",
            strike(
                stack("spell.base:magic_arrow", 1),
                Spread::Target,
                &[hit(wolves, 20, 2)],
            ),
        ),
    ]
}

/// The samples whose text still shows a placeholder (`{{`), a specifier (`%`) or a key (`.`
/// between letters: `battle.log`, `creature.base`), each as `<what>: <text>`.
pub fn log_problems(samples: &[(&str, String)]) -> Vec<String> {
    samples
        .iter()
        .filter(|(_, line)| {
            line.contains("{{")
                || line.contains('%')
                || line.contains("battle.log")
                || line.contains("creature.")
                || line.contains("word.")
        })
        .map(|(what, line)| format!("{what}: {line}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_and_arguments() {
        let o = Options::parse_with(
            &args(&[
                "battle.log.damage",
                "attacker=Imps",
                "damage=4",
                "count=2",
                "--pack",
                "de.json",
            ]),
            Some(PathBuf::from("/env")),
        )
        .unwrap();
        assert_eq!(o.install, PathBuf::from("/env"));
        assert_eq!(o.key.as_deref(), Some("battle.log.damage"));
        assert_eq!(o.pack, Some(PathBuf::from("de.json")));
        assert!(o.keys.ends_with("text/keys.toml"));
        let given = arguments(&o.args);
        assert_eq!(
            given,
            Args::new()
                .text("attacker", "Imps")
                .number("damage", 4)
                .count(2)
        );
        let context = arguments(&[(String::from("context"), String::from("female"))]);
        assert_eq!(context, Args::new().context("female"));
        for bad in [&["--bogus"][..], &["--pack"], &["k", "novalue"]] {
            assert!(Options::parse_with(&args(bad), None).is_err(), "{bad:?}");
        }
        assert!(Options::parse_with(&args(&["--log"]), None).unwrap().log);
        let o = Options::parse_with(
            &args(&["--keys", "k.toml", "--tables", "t", "--install", "i"]),
            None,
        )
        .unwrap();
        assert!(!o.log);
        assert_eq!(
            (o.keys, o.tables, o.install, o.key),
            (
                PathBuf::from("k.toml"),
                PathBuf::from("t"),
                PathBuf::from("i"),
                None
            )
        );
    }

    /// The committed catalogue parses over the committed asset tables, and names its keys in
    /// GENRLTXT, CRTRAITS, HELP (the panel's hints, E17 M3 T6c-1), HOTRAITS and SPTRAITS (the
    /// casts' lines, T7b-1) only (the real check, that each stays within its table's rows, is the
    /// gate's).
    #[test]
    fn the_committed_catalogue_parses() {
        let assets = assets::load(&assets::default_dir(), assets::ALL).unwrap();
        let keys = read_keys(&default_keys(), &assets).unwrap();
        assert!(keys.keys.contains_key("battle.log.damage"));
        assert!(keys.keys.contains_key("creature.base:pikeman"));
        assert!(keys.keys.contains_key("spell.sod:titans_lightning_bolt"));
        assert!(keys.keys.contains_key("hero.base:orrin"));
        // E17 M4 T2c-1: a town's moat by its faction.
        assert!(keys.keys.contains_key("moat.base:castle"));
        // E17 M4 T2c-2: a wall's part by its faction (the upper wall, part 6).
        assert!(keys.keys.contains_key("wall.base:castle.6"));
        let names: BTreeSet<&str> = keys
            .keys
            .values()
            .flat_map(|k| k.original.iter().map(|(_, s, _)| s.table.as_str()))
            .collect();
        // E17 M3 T7b-2b-3: the executable's own strings too (Acid Breath's line); E17 M4 T2c-1
        // the moats' names; E17 M4 T2c-2 the walls' parts (WALLS.TXT).
        let tables = [
            "ARRAYTXT", "CRTRAITS", "EXE", "GENRLTXT", "HELP", "HOTRAITS", "SKILLLEV", "SPTRAITS",
            "WALLS",
        ];
        assert_eq!(names, BTreeSet::from(tables));
    }

    #[test]
    fn a_pack_file_names_its_language() {
        let dir = std::env::temp_dir().join(format!("h3-view-text-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let keys = Keys::parse("[[key]]\nid = \"a\"", &BTreeMap::new()).unwrap();
        let de = dir.join("de.json");
        fs::write(&de, r#"{ "a": "x", "b": "y" }"#).unwrap();
        let (pack, problems) = read_pack(&de, &keys).unwrap();
        assert_eq!(pack.messages.len(), 2);
        assert_eq!(problems.len(), 1);
        let xx = dir.join("xx.json");
        fs::write(&xx, "{}").unwrap();
        assert!(
            read_pack(&xx, &keys)
                .unwrap_err()
                .contains("no plural rule")
        );
        assert!(read_pack(&dir.join("none.json"), &keys).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Without a pack every sample shows its keys, which the check lists; an invented pack's
    /// texts pass it.
    #[test]
    fn the_log_samples_and_their_check() {
        let empty = log_samples(&Catalogue::default());
        assert_eq!(log_problems(&empty).len(), empty.len());
        let pack = Pack::from_json(
            "en",
            r#"{ "battle": { "log": {
                "damage": "{{attacker}} {{damage}}", "perish": " {{count}} {{creature}}",
                "destroyed": " {{creature}}", "wait": "{{creature}}",
                "defend": "{{creature}} {{defense}}", "morale-good": "{{creature}}",
                "morale-bad": "{{creature}}", "luck-good": "{{creature}}",
                "fear": "{{frightener}} {{creature}}",
                "heal": "{{healer}} {{creature}} {{hit_points}}", "round": "R",
                "cast-on": "{{hero}} {{spell}} {{creature}}", "resisted": "{{creature}} {{verb}}"
                } },
                "word": { "creature": "w", "resist": "r" }, "creature": { "base:wolf_rider": "W",
                "base:pikeman": "P", "base:ballista": "B", "base:first_aid_tent": "T" },
                "spell": { "base:slow": "S", "base:magic_arrow": "A" },
                "hero": { "base:orrin": "O" } }"#,
        )
        .unwrap();
        let catalogue = Catalogue { packs: vec![pack] };
        let samples = log_samples(&catalogue);
        assert_eq!(log_problems(&samples), Vec::<String>::new());
        assert_eq!(samples[0], ("strike", String::from("W 37 1 P")));
        let bad = [("x", String::from("50%")), ("y", String::from("{{a}}"))];
        assert_eq!(log_problems(&bad), ["x: 50%", "y: {{a}}"]);
    }
}
