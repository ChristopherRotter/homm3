//! `h3_map_view::engine` (E24 M4 T3a): a whole game's record read and checked, and its start as a
//! capture's live part, on a game the engine builds on a writer's map with the repository's packs
//! (no original data): a random resource, monster and artifact, an owned sawmill, a pile, heroes
//! of three colours and an empty boat; then every refusal, and what `differences` finds. Since
//! E24 M4 T3b1 the panels: the kingdom, the status window, the kingdom's window and what choosing
//! each hero and town shows, on a map with Red's town (`PANELS`).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use h3_adventure::{ObjectKind, ObjectTypeId};
use h3_contracts::PlayerId;
use h3_game::{GameRuleset, GameState, Viewer};
use h3_map::writer::{self, ObjectTemplate};
use h3_map::{Limits, MAP_IMPORT_VERSION};
use h3_map_view::art::MapArt;
use h3_map_view::engine::{self, Refused};
use h3_map_view::hover::Facts;
use h3_map_view::picks;
use h3_map_view::record::Records;
use h3_map_view::situation::Situation;
use h3_map_view::tables::{self, TownArt};
use h3_protocol::RECORD_FORMAT_VERSION;
use h3_protocol::codec;
use h3_protocol::map::game::{manifest, observe};
use h3_protocol::wire::game::{
    Amount, GameObservation, GameRecordHeader, GarrisonStack, Guards, Known, KnownObject,
    MapIdentity, Object, ObjectRef, SkillLevel, VisitBonus,
};
use h3_protocol::wire::{ServerMessage, WireU64};
use toml::Value;

/// The map: Red's hero beside the objects, Blue's in its sight, Purple's far off; a water tile with
/// an empty boat; a random resource, monster, artifact and dwelling, Red's sawmill and a pile of
/// wood.
const MAP: &str = r#"
size = 36
terrain = "grass"
heroes = [
    { at = [10, 10], class = 2 },
    { at = [12, 10], owner = 1, class = 9 },
    { at = [34, 34], owner = 2 },
]
tiles = [{ at = [8, 12], terrain = "water", picture = 21 }]
boats = [{ at = [8, 12], kind = 1 }]
view = [0, 0, 0]

[[objects]]
def = "AVTrndm0.def"
at = [14, 10]

[[objects]]
def = "AVArand.def"
at = [8, 9]

[[objects]]
def = "AVMsawg0.def"
at = [6, 10]
owner = 0

[[objects]]
def = "AVTwood0.def"
at = [13, 12]
amount = 5

[[objects]]
def = "AVWmrnd0.def"
at = [10, 13]

[[objects]]
def = "avrcgen0.def"
at = [8, 15]
"#;

/// The map's templates as `Objects.txt` gives them, a boat's and a Castle's among them.
const LINES: [&str; 8] = [
    "AVCcasx0.def 000001110000011110001111111111111111111111111111 001000000000000000000000000000000000000000000000 011111111 011111111 98 0 1 0",
    "avrcgen0.def 001111110011111111111111111111111111111111111111 110000000000000000000000000000000000000000000000 111111111 011111111 216 0 0 0",
    "AVXboat1.def 101111111111111111111111111111111111111111111111 010000000000000000000000000000000000000000000000 100000000 100000000 8 1 0 0",
    "AVTrndm0.def 011111111111111111111111111111111111111111111111 100000000000000000000000000000000000000000000000 111111111 000000001 76 0 5 0",
    "AVArand.def 011111111111111111111111111111111111111111111111 100000000000000000000000000000000000000000000000 111111111 000000001 65 0 4 0",
    "AVMsawg0.def 000011110011111111111111111111111111111111111111 010000000000000000000000000000000000000000000000 111111111 000010100 53 0 0 0",
    "AVTwood0.def 011111111111111111111111111111111111111111111111 100000000000000000000000000000000000000000000000 111111111 000000001 79 0 5 0",
    "AVWmrnd0.def 011111111111111111111111111111111111111111111111 100000000000000000000000000000000000000000000000 111111111 000000001 71 0 2 0",
];

/// The game's templates: the map's, then a monster's of every creature (`M<creature>.def`).
/// A visit placed on object `object` of `class` on its own tile (E24 M5 T4c: `visits::lay`'s
/// `object_at`), from beside or not.
fn placed(
    v: &engine::Visit,
    object: usize,
    class: [u32; 2],
    beside: bool,
) -> engine::visits::Placed {
    engine::visits::Placed {
        at: v.at,
        object,
        class,
        word: None,
        beside,
    }
}

fn templates() -> Vec<ObjectTemplate> {
    let monster = |c: u32| {
        format!(
            "M{c:03}.def 0{} 1{} 111111111 000000001 54 {c} 2 0",
            "1".repeat(47),
            "0".repeat(47)
        )
    };
    LINES
        .iter()
        .map(|line| (*line).to_owned())
        .chain((0..160).map(monster))
        .map(|line| ObjectTemplate::from_objects_txt(&line).unwrap())
        .collect()
}

/// The repository's packs, Complete's content, the faithful profile.
fn rules() -> &'static GameRuleset {
    static RULES: OnceLock<GameRuleset> = OnceLock::new();
    RULES.get_or_init(|| {
        let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packs");
        let ids = ["base", "ab", "sod"].map(String::from);
        h3_files::adventure::load_rules(&packs, &ids, "base:faithful").unwrap()
    })
}

/// The handle of the packs' creature `key` (the engine's, by the packs' order).
fn handle(key: &str) -> u16 {
    rules().core().creature_by_key(key).unwrap().raw()
}

/// The committed tables' records (`[map.records]`): the classes the game loads the map's as.
fn records() -> &'static Records {
    static RECORDS: OnceLock<Records> = OnceLock::new();
    RECORDS.get_or_init(|| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml");
        let text = std::fs::read_to_string(path).unwrap();
        tables::load(&[tables::parse(&text).unwrap()], "base")
            .unwrap()
            .records
    })
}

/// A situation's map file, gzip-compressed as the engine reads it.
fn map_file(text: &str) -> Vec<u8> {
    let (map, extras) = Situation::parse(text)
        .unwrap()
        .build(&BTreeMap::new(), &templates())
        .unwrap();
    writer::gzip(&writer::write_with(&map, &extras).unwrap())
}

/// A game on the map: its file, its art, Red's first observation and the record's header.
struct Game {
    bytes: Vec<u8>,
    art: MapArt,
    observation: GameObservation,
    header: GameRecordHeader,
}

fn game() -> Game {
    game_on(map_file(MAP))
}

/// A game on the map file `bytes`.
fn game_on(bytes: Vec<u8>) -> Game {
    let rules = rules();
    let raw = h3_map::read(&bytes, &Limits::default()).unwrap();
    let setup = h3_map::import(&raw, rules).unwrap();
    let state = GameState::new(rules, &setup, 7).unwrap();
    Game {
        art: MapArt::from_raw(&raw).unwrap(),
        observation: observe(&state, rules, Viewer::Player(PlayerId::new(0))),
        header: header_on(&bytes),
        bytes,
    }
}

/// The header of a record of a game on the map file `bytes` under the packs' rules.
fn header_on(bytes: &[u8]) -> GameRecordHeader {
    let identity = MapIdentity {
        content_hash: WireU64(engine::content_hash(bytes)),
        import_version: MAP_IMPORT_VERSION,
    };
    GameRecordHeader {
        format: RECORD_FORMAT_VERSION,
        game: 0,
        player: 0,
        manifest: manifest(rules(), identity),
        first: 1,
    }
}

/// A record's text: the header, then each message, a line each (a blank line among them).
fn text(header: &GameRecordHeader, messages: &[ServerMessage]) -> String {
    let mut lines = vec![codec::encode_text(header), String::new()];
    lines.extend(messages.iter().map(codec::encode_text));
    lines.join("\n")
}

fn snapshot(observation: &GameObservation) -> ServerMessage {
    ServerMessage::GameSnapshot {
        n: 1,
        observation: observation.clone(),
    }
}

/// The start of `observation`'s record on the game's map.
fn start(g: &Game, observation: &GameObservation) -> Result<engine::Start, Refused> {
    let record = text(&g.header, &[snapshot(observation)]);
    let loaded = engine::load(&record, &g.bytes, rules())?;
    engine::start(
        &loaded,
        &g.art,
        &templates(),
        &town_art(),
        records(),
        rules(),
    )
}

/// Every faction's town pictures, named by their list and faction.
fn town_art() -> TownArt {
    let list = |word: &str| (0..9).map(|f| format!("{word}{f}.def")).collect();
    TownArt {
        village: list("village"),
        fort: list("fort"),
        capitol: list("capitol"),
    }
}

/// The index of the map's object whose template is `def`.
fn object(g: &Game, def: &str) -> usize {
    (0..g.art.objects.len())
        .find(|&i| g.art.templates[g.art.objects[i].template].def == def)
        .unwrap()
}

fn numbers(value: &Value) -> Vec<i64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_integer().unwrap())
        .collect()
}

#[test]
fn a_start_as_the_original_s_live_part() {
    let g = game();
    let start = start(&g, &g.observation).unwrap();
    let t = &start.table;
    assert_eq!(t["viewer"], Value::Integer(0));
    let defs = t["defs"].as_array().unwrap();
    let classes = t["classes"].as_array().unwrap();
    let owners = t["owners"].as_array().unwrap();
    // The random resource became the pile of its drawn resource, named as the original's copy.
    let resource = object(&g, "AVTrndm0.def");
    let pile = numbers(&classes[resource]);
    assert_eq!(pile[0], 79);
    let names = [
        "avtwood0.def",
        "avtmerc0.def",
        "avtore0.def",
        "avtsulf0.def",
        "avtcrys0.def",
        "avtgems0.def",
        "avtgold0.def",
    ];
    assert_eq!(
        defs[resource].as_str(),
        Some(names[usize::try_from(pile[1]).unwrap()])
    );
    // The random monster the last template of its creature; the random artifact its own
    // (MCL-STUB-1); the others their templates.
    let monster = object(&g, "AVWmrnd0.def");
    let creature = numbers(&classes[monster]);
    assert_eq!(creature[0], 54);
    assert_eq!(
        defs[monster].as_str().unwrap(),
        format!("M{:03}.def", creature[1])
    );
    let artifact = object(&g, "AVArand.def");
    assert_eq!(defs[artifact].as_str(), Some("AVArand.def"));
    assert_eq!(numbers(&classes[artifact]), [65, 0]);
    // The owners by colour; every object here in sight.
    assert_eq!(owners[object(&g, "AVMsawg0.def")], Value::Integer(0));
    assert_eq!(owners[object(&g, "AVTwood0.def")], Value::Integer(-1));
    for def in [
        "AVTrndm0.def",
        "AVArand.def",
        "AVMsawg0.def",
        "AVTwood0.def",
    ] {
        assert!(start.seen[object(&g, def)], "{def}");
    }
    // Red's hero and Blue's in sight, facing right on land; Purple's out of sight.
    let heroes: Vec<Vec<i64>> = t["heroes"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(
        heroes,
        [
            vec![10, 10, 0, 2, 2, 0, -1, -1],
            vec![12, 10, 0, 9, 2, 1, -1, -1]
        ]
    );
    let boats: Vec<Vec<i64>> = t["boats"].as_array().unwrap().iter().map(numbers).collect();
    assert_eq!(boats.len(), 1);
    assert_eq!(boats[0][..4], [8, 12, 0, 1]);
    let rows = t["explored"].as_array().unwrap();
    assert_eq!(rows.len(), 36);
    assert_eq!(rows[10].as_str().unwrap().as_bytes()[10], b'1');
    assert_eq!(rows[34].as_str().unwrap().as_bytes()[34], b'0');
    // The same start shows no difference.
    assert!(engine::differences(&start, t).is_empty());
}

#[test]
fn a_later_start_draws_no_object_taken_off_the_map() {
    // E24 M5 T4c, T4's game: the hot seat's view set up again from a later snapshot showed a pile
    // taken the day before standing again. An object of the import's that the view does not show
    // where the viewer explored a tile it covers is gone; under the fog it stands as the file has
    // it. A new game's start has none (no `gone`).
    let g = game();
    assert!(
        !start(&g, &g.observation)
            .unwrap()
            .table
            .contains_key("gone")
    );
    let wood = object(&g, "AVTwood0.def");
    let at = g.art.objects[wood].anchor;
    let without = |observation: &GameObservation| {
        let mut o = observation.clone();
        o.adventure.objects.retain(|v| {
            (v.anchor.x, v.anchor.y, v.anchor.level) != (u16::from(at.x), u16::from(at.y), at.z)
        });
        assert_eq!(
            o.adventure.objects.len() + 1,
            observation.adventure.objects.len()
        );
        o
    };
    let taken = without(&g.observation);
    let t = start(&g, &taken).unwrap().table;
    assert_eq!(
        t["gone"],
        Value::Array(vec![Value::Integer(i64::try_from(wood).unwrap())])
    );
    // The same with its tile unexplored: standing, nothing gone.
    let mut fogged = taken.clone();
    let a = &mut fogged.adventure;
    let row = usize::from(at.z) * usize::from(a.width) + usize::from(at.y);
    let word = row * usize::from(a.stride) + usize::from(at.x) / 64;
    a.explored[word].0 &= !(1 << (at.x % 64));
    assert!(!start(&g, &fogged).unwrap().table.contains_key("gone"));
}

#[test]
fn a_hero_in_a_boat_and_an_odd_pile() {
    let g = game();
    let mut observation = g.observation.clone();
    observation.adventure.heroes[0].boat = Some(2);
    // A random resource the view shows as no pile keeps its random template.
    let resource = object(&g, "AVTrndm0.def");
    let artifact_type = observation
        .adventure
        .objects
        .iter()
        .find(|o| (o.anchor.x, o.anchor.y) == (8, 9))
        .unwrap()
        .object_type;
    let shown = observation
        .adventure
        .objects
        .iter_mut()
        .find(|o| (o.anchor.x, o.anchor.y) == (14, 10))
        .unwrap();
    shown.object_type = artifact_type;
    let start = start(&g, &observation).unwrap();
    let row = numbers(&start.table["heroes"].as_array().unwrap()[0]);
    assert_eq!(row[6..], [2, 0]);
    let defs = start.table["defs"].as_array().unwrap();
    assert_eq!(defs[resource].as_str(), Some("AVTrndm0.def"));
}

#[test]
fn a_random_dwelling_moved_right() {
    let g = game();
    let dwelling = object(&g, "avrcgen0.def");
    let at = |o: &Object| (o.anchor.x, o.anchor.y) == (8, 15);
    let start_at = |shift: u16| {
        let mut observation = g.observation.clone();
        let shown = observation
            .adventure
            .objects
            .iter_mut()
            .find(|o| at(o))
            .unwrap();
        shown.anchor.x += shift;
        start(&g, &observation).unwrap()
    };
    for shift in [0, 1] {
        let start = start_at(shift);
        assert!(start.seen[dwelling]);
        let defs = start.table["defs"].as_array().unwrap();
        assert_eq!(defs[dwelling].as_str(), Some("avrcgen0.def"));
    }
}

#[test]
fn what_differs_from_a_capture() {
    let g = game();
    let ours = start(&g, &g.observation).unwrap();
    let sawmill = object(&g, "AVMsawg0.def");
    let wood = object(&g, "AVTwood0.def");
    let mut theirs = ours.table.clone();
    // A DEF's case and a hero out of sight change nothing.
    let lower = Value::String("avmsawg0.def".to_owned());
    theirs["defs"].as_array_mut().unwrap()[sawmill] = lower;
    let far = Value::Array([30, 30, 0, 0, 2, 2, -1, -1].map(Value::Integer).to_vec());
    let heroes = theirs["heroes"].as_array_mut().unwrap();
    heroes.push(far);
    // Neither does a row that is not one, or one off the map.
    heroes.push(Value::String("x".to_owned()));
    heroes.push(Value::Array([99, 0, 0].map(Value::Integer).to_vec()));
    assert!(engine::differences(&ours, &theirs).is_empty());
    theirs["viewer"] = Value::Integer(1);
    theirs["defs"].as_array_mut().unwrap()[wood] = Value::String("AVTgold0.def".to_owned());
    theirs["owners"].as_array_mut().unwrap()[sawmill] = Value::Integer(3);
    theirs["heroes"].as_array_mut().unwrap().remove(0);
    theirs.remove("boats");
    let row = &mut theirs["explored"].as_array_mut().unwrap()[0];
    *row = Value::String("1".repeat(36));
    assert_eq!(
        engine::differences(&ours, &theirs),
        [
            "`viewer` differs".to_owned(),
            "`explored` differs".to_owned(),
            format!("object {wood}: `defs` differ"),
            format!("object {sawmill}: `owners` differ"),
            "`heroes` on explored tiles differ".to_owned(),
            "`boats` on explored tiles differ".to_owned(),
            // The chosen hero's row is another hero's once the first is gone (E24 M4 T3b1).
            "`chosen.heroes` 0: its tile or path differs".to_owned(),
        ]
    );
    // A capture without the objects' keys differs for every object in sight.
    theirs.remove("defs");
    let found = engine::differences(&ours, &theirs);
    let seen = ours.seen.iter().filter(|s| **s).count();
    assert_eq!(
        found
            .iter()
            .filter(|f| f.ends_with("`defs` differ"))
            .count(),
        seen
    );
}

#[test]
fn records_it_refuses() {
    let g = game();
    let good = text(&g.header, &[snapshot(&g.observation)]);
    let refused = |text: &str, bytes: &[u8]| engine::load(text, bytes, rules()).unwrap_err();
    assert_eq!(engine::parse(" \n"), Err(Refused::Empty));
    assert_eq!(refused("", &g.bytes), Refused::Empty);
    assert!(matches!(
        engine::parse("nonsense"),
        Err(Refused::Line(1, _))
    ));
    let mut header = g.header.clone();
    header.format += 1;
    let format = RECORD_FORMAT_VERSION + 1;
    assert_eq!(
        engine::parse(&text(&header, &[])),
        Err(Refused::Format(format))
    );
    let mut header = g.header.clone();
    header.manifest.protocol += 1;
    assert!(matches!(
        engine::parse(&text(&header, &[])),
        Err(Refused::Protocol(_))
    ));
    let bad = format!("{good}\nnonsense");
    assert!(matches!(engine::parse(&bad), Err(Refused::Line(4, _))));
    // The map is not the file; the rules not the viewer's.
    assert!(matches!(refused(&good, b"another file"), Refused::Map(..)));
    let mut header = g.header.clone();
    header.manifest.engine = "another".to_owned();
    assert_eq!(refused(&text(&header, &[]), &g.bytes), Refused::Rules);
    // A file that is no map; a map the import refuses (MCL-Q21 (b)).
    let junk = b"not a map".as_slice();
    assert!(matches!(
        refused(&text(&header_on(junk), &[]), junk),
        Refused::Read(_)
    ));
    // An owned mine entered from water (`Unsupported::OwnedOnWater`).
    let wet = MAP.replace(
        "tiles = [",
        "tiles = [{ at = [20, 20], terrain = \"water\", picture = 21 }, ",
    ) + "\n[[objects]]\ndef = \"AVMsawg0.def\"\nat = [21, 20]\nowner = 0\n";
    let odd = map_file(&wet);
    assert!(matches!(
        refused(&text(&header_on(&odd), &[]), &odd),
        Refused::Import(_)
    ));
    // No snapshot first.
    let turn = ServerMessage::GameTurn {
        n: 1,
        legal: Vec::new(),
        castable: Vec::new(),
    };
    let loaded = engine::load(&text(&g.header, &[turn]), &g.bytes, rules()).unwrap();
    assert_eq!(
        engine::start(&loaded, &g.art, &[], &town_art(), records(), rules()),
        Err(Refused::NoStart)
    );
    // A record its bound cut, beginning at a later snapshot (E24 M4 T6, R4-4): not a new game's
    // start; its walks are not played either.
    let mut cut = g.header.clone();
    cut.first = 40;
    let loaded = engine::load(&text(&cut, &[snapshot(&g.observation)]), &g.bytes, rules()).unwrap();
    assert_eq!(
        engine::start(&loaded, &g.art, &[], &town_art(), records(), rules()),
        Err(Refused::Cut(40))
    );
    assert_eq!(
        engine::played(&loaded, &g.art, rules()).map(|_| ()),
        Err(Refused::Cut(40))
    );
    // Another map's art; an object, a hero or a player the map or the view does not hold.
    let loaded = engine::load(&good, &g.bytes, rules()).unwrap();
    let other = MapArt::from_raw(&h3_map::read(&g.bytes, &Limits::default()).unwrap())
        .map(|mut art| {
            let first = art.objects[3];
            art.objects.push(first);
            art
        })
        .unwrap();
    assert_eq!(
        engine::start(&loaded, &other, &[], &town_art(), records(), rules()),
        Err(Refused::Setup)
    );
    // An art with a hero more before the objects: the static ones as many, their indices past the
    // file's objects (E24 M4 T6, R4-11).
    let shifted = MapArt::from_raw(&h3_map::read(&g.bytes, &Limits::default()).unwrap())
        .map(|mut art| {
            let hero = (0..art.objects.len())
                .find(|&i| art.templates[art.objects[i].template].class == 34)
                .unwrap();
            let copy = art.objects[hero];
            art.objects.insert(0, copy);
            art
        })
        .unwrap();
    assert_eq!(
        engine::start(&loaded, &shifted, &[], &town_art(), records(), rules()),
        Err(Refused::Setup)
    );
    // As many objects, a static one a hero in the art: the static ones fewer than the import's.
    let swapped = MapArt::from_raw(&h3_map::read(&g.bytes, &Limits::default()).unwrap())
        .map(|mut art| {
            let class = |art: &MapArt, i: usize| art.templates[art.objects[i].template].class;
            let hero = (0..art.objects.len())
                .find(|&i| class(&art, i) == 34)
                .unwrap();
            let other = (0..art.objects.len())
                .find(|&i| ![34, 70, 214, 8].contains(&class(&art, i)))
                .unwrap();
            art.objects[other] = art.objects[hero];
            art
        })
        .unwrap();
    assert_eq!(
        engine::start(&loaded, &swapped, &[], &town_art(), records(), rules()),
        Err(Refused::Setup)
    );
    let changed = |change: &dyn Fn(&mut GameObservation)| {
        let mut observation = g.observation.clone();
        change(&mut observation);
        start(&g, &observation).unwrap_err()
    };
    // The random artifact moved: no object of the file there.
    let moved = changed(&|o| o.adventure.objects[0].anchor.x += 1);
    assert!(matches!(moved, Refused::Object(..)));
    assert_eq!(changed(&|o| o.heroes.clear()), Refused::Hero(0));
    assert_eq!(
        changed(&|o| o.adventure.objects[0].owner = Some(9)),
        Refused::Player(9)
    );
    assert_eq!(
        changed(&|o| o.adventure.heroes[0].owner = 9),
        Refused::Player(9)
    );
    assert_eq!(changed(&|o| o.adventure.viewer = 9), Refused::Player(9));
}

#[test]
fn each_refusal_says_why() {
    let identity = MapIdentity {
        content_hash: WireU64(1),
        import_version: 2,
    };
    let all = [
        Refused::Empty,
        Refused::Line(3, "bad".to_owned()),
        Refused::Format(9),
        Refused::Protocol(9),
        Refused::Map(identity, identity),
        Refused::Rules,
        Refused::Read("short".to_owned()),
        Refused::Import("odd".to_owned()),
        Refused::NoStart,
        Refused::Setup,
        Refused::Object(4, [1, 2, 0]),
        Refused::Hero(5),
        Refused::Player(9),
        Refused::Cut(40),
    ];
    let texts: Vec<String> = all.iter().map(ToString::to_string).collect();
    assert!(texts.iter().all(|t| !t.is_empty()));
    assert_eq!(texts[1], "line 3: bad");
    assert!(texts[4].contains("0x0000000000000001"));
    assert!(texts[10].contains("(1, 2, 0)"));
    assert!(texts[13].contains("message 40, not at the game's start"));
}

/// The panels' map (E24 M4 T3b1): Red's two heroes out of their types' order (Astral named by
/// the map, its army and Rion's given), Red's town named by the map, Blue's hero far off.
const PANELS: &str = r#"
size = 36
terrain = "grass"
heroes = [
    { at = [12, 12], class = 5, name = "Quillon", map_army = [[28, 20]] },
    { at = [8, 10], class = 1, map_army = [[0, 15], [139, 5]] },
    { at = [33, 33], owner = 1, class = 6 },
]
towns = [{ at = [16, 8], faction = 0, owner = 0, buildings = "fort", name = "Brightmoor" }]
view = [0, 0, 0]
"#;

/// A game on `PANELS`' map, with the map's teams when given.
fn panels_game(teams: Option<[u8; 8]>) -> Game {
    let (mut map, extras) = Situation::parse(PANELS)
        .unwrap()
        .build(&BTreeMap::new(), &templates())
        .unwrap();
    map.teams = teams.map(|of_player| h3_map::model::Teams {
        count: of_player.iter().max().unwrap() + 1,
        of_player,
    });
    game_on(writer::gzip(&writer::write_with(&map, &extras).unwrap()))
}

fn table(value: &Value) -> &toml::Table {
    value.as_table().unwrap()
}

#[test]
fn the_panels_of_a_start() {
    let g = panels_game(None);
    let start = start(&g, &g.observation).unwrap();
    let t = &start.table;
    let o = &g.observation;
    let own = |id: u16| {
        let hero = o.adventure.heroes.iter().find(|h| h.id == id).unwrap();
        hero.own.clone().unwrap()
    };
    let info = |id: u16| o.heroes.iter().find(|h| h.id == id).unwrap().clone();
    // The map's heroes: Astral (40) first in the file, Rion (8) second; the list by type.
    let ids: Vec<u16> = o.adventure.heroes.iter().map(|h| h.id).collect();
    let (astral, rion) = (ids[0], ids[1]);
    assert_eq!((info(rion).hero_type, info(astral).hero_type), (8, 40));
    let mana = |id: u16| i64::from(info(id).details.unwrap().mana);
    let points = |id: u16| i64::from(own(id).movement);
    let kingdom = table(&t["kingdom"]);
    let rows: Vec<Vec<i64>> = kingdom["heroes"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(
        rows,
        [
            vec![0, 1, 8, points(rion), mana(rion)],
            vec![0, 1, 40, points(astral), mana(astral)],
        ]
    );
    let towns: Vec<Vec<i64>> = kingdom["towns"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(towns, [vec![0, 1, 0]]);
    assert_eq!(numbers(&kingdom["selected"]), [1, 0]);
    assert_eq!(numbers(&kingdom["tops"]), [0, 0]);
    assert_eq!(kingdom["quests"], Value::Integer(0));
    let treasury: Vec<i64> = o.treasury.iter().map(|&a| i64::from(a)).collect();
    assert_eq!(numbers(&kingdom["resources"]), treasury);
    assert_eq!(numbers(&kingdom["date"]), [1, 1, 1]);
    // Rion's status window: its type, no name of the map's, its base skills, morale and luck 0
    // (MCL-Q36), its army by slot (the map's Peasants, AB's 139, by their number: the import's
    // handle 140, E24 M5 T4c).
    let status = table(&t["status"]);
    assert_eq!(status["kind"], Value::Integer(3));
    assert_eq!(status["hero"], Value::Integer(8));
    assert_eq!(status["own_name"], Value::Integer(0));
    let primary = info(rion).details.unwrap().primary.map(i64::from);
    assert_eq!(numbers(&status["skills"]), primary);
    assert_eq!(
        (&status["morale"], &status["luck"]),
        (&Value::Integer(0), &Value::Integer(0))
    );
    let army: Vec<Vec<i64>> = status["army"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(army[..3], [vec![0, 15], vec![139, 5], vec![-1, 0]]);
    // The kingdom's window: one village, Red against Blue (no teams).
    let realm = table(&t["realm"]);
    assert_eq!(numbers(&realm["halls"]), [1, 0, 0, 0]);
    assert_eq!(numbers(&realm["allies"]), [0]);
    assert_eq!(numbers(&realm["enemies"]), [1]);
    // Each one chosen: the heroes by their places among the view's, Astral named by the map;
    // the town at its entrance, named by the map, its Fort and Village Hall, its 500 gold.
    let chosen = table(&t["chosen"]);
    let heroes = chosen["heroes"].as_array().unwrap();
    assert_eq!(heroes[0]["hero"], Value::Integer(1));
    assert_eq!(heroes[1]["hero"], Value::Integer(0));
    assert_eq!(heroes[1]["status"]["own_name"], Value::Integer(1));
    assert_eq!(heroes[0]["path"], Value::Array(Vec::new()));
    let town = &chosen["towns"].as_array().unwrap()[0];
    assert_eq!(numbers(&town["at"]), [14, 8, 0]);
    let status = table(&town["status"]);
    assert_eq!(status["name"], Value::String("Brightmoor".into()));
    assert_eq!(numbers(&status["buildings"]), [7, 10]);
    assert_eq!(status["garrison"], Value::Integer(0));
    assert_eq!(numbers(&status["silo"]), Vec::<i64>::new());
    assert_eq!(status["income"], Value::Integer(500));
    assert!(engine::differences(&start, t).is_empty());
}

#[test]
fn a_town_s_window_selected_or_nothing_selected() {
    let g = panels_game(Some([0, 0, 1, 1, 1, 1, 1, 1]));
    let mut o = g.observation.clone();
    // Red's heroes gone: the town selected, its window (a nameless Citadel with a silo of wood
    // and ore, a garrison hero's army; no hall known).
    let red = o.adventure.viewer;
    o.adventure.heroes.retain(|h| h.owner != red);
    let town = &mut o.towns[0];
    town.name = None;
    town.fort = 2;
    let facts = town.owner_facts.as_mut().unwrap();
    facts.garrison_hero = true;
    facts.income = [(0, 1), (2, 1), (5, 0), (6, 1250)]
        .map(|(resource, amount)| Amount { resource, amount })
        .to_vec();
    // A war machine by its number (the Ballista, 146: handle 119; E24 M5 T4c).
    town.garrison.army = Some(vec![h3_protocol::wire::game::ArmyStack {
        slot: 2,
        creature: handle("base:ballista"),
        count: 3,
    }]);
    let selected = start(&g, &o).unwrap();
    let t = &selected.table;
    assert_eq!(numbers(&t["kingdom"]["selected"]), [2, 0]);
    let status = table(&t["status"]);
    assert_eq!(status["kind"], Value::Integer(4));
    assert_eq!(status["name"], Value::Integer(0));
    assert_eq!(numbers(&status["buildings"]), [8, 10]);
    assert_eq!(status["garrison"], Value::Integer(1));
    assert_eq!(numbers(&status["army"].as_array().unwrap()[2]), [146, 3]);
    assert_eq!(numbers(&status["silo"]), [0, 2]);
    assert_eq!(status["income"], Value::Integer(1250));
    // Blue on Red's team: an ally.
    assert_eq!(numbers(&t["realm"]["allies"]), [0, 1]);
    // A town the file holds none of at its anchor: its anchor for its entrance; no facts known.
    let town = &mut o.towns[0];
    town.anchor.x = 3;
    town.owner_facts = None;
    town.fort = 0;
    let start_moved = start(&g, &o).unwrap();
    let chosen = &start_moved.table["chosen"]["towns"].as_array().unwrap()[0];
    assert_eq!(numbers(&chosen["at"]), [3, 8, 0]);
    let status = table(&chosen["status"]);
    assert_eq!(numbers(&status["buildings"]), Vec::<i64>::new());
    assert_eq!(numbers(&start_moved.table["realm"]["halls"]), [0; 4]);
    // No town either: nothing selected, no window.
    o.towns.clear();
    let none = start(&g, &o).unwrap();
    assert_eq!(numbers(&none.table["kingdom"]["selected"]), [0, 0]);
    assert!(!none.table.contains_key("status"));
}

#[test]
fn a_hero_off_its_file_tile_shows_its_type_s_portrait() {
    let g = panels_game(None);
    let mut o = g.observation.clone();
    o.adventure.heroes[1].pos.x = 20;
    o.adventure.heroes[1].own.as_mut().unwrap().movement = 0;
    let start = start(&g, &o).unwrap();
    let row = numbers(&start.table["kingdom"]["heroes"].as_array().unwrap()[0]);
    // Rion (type 8) unmoved by the map's portrait (none given), not able to move.
    assert_eq!(row[..3], [0, 0, 8]);
}

#[test]
fn what_differs_in_the_panels() {
    let g = panels_game(None);
    let ours = start(&g, &g.observation).unwrap();
    let mut theirs = ours.table.clone();
    // A town's buildings compare by the hall and fort its window draws.
    let town = &mut theirs["chosen"]["towns"].as_array_mut().unwrap()[0]["status"];
    town["buildings"] = Value::Array([5, 7, 10, 30].map(Value::Integer).to_vec());
    assert!(engine::differences(&ours, &theirs).is_empty());
    let kingdom = theirs["kingdom"].as_table_mut().unwrap();
    kingdom.insert(
        "date".into(),
        Value::Array([1, 1, 2].map(Value::Integer).to_vec()),
    );
    theirs["status"]["morale"] = Value::Integer(1);
    theirs["status"]["army"] = Value::Integer(0);
    theirs["realm"]["halls"] = Value::Integer(0);
    let chosen = theirs["chosen"].as_table_mut().unwrap();
    chosen["heroes"].as_array_mut().unwrap()[0]["path"] = Value::Array(vec![Value::Integer(0)]);
    chosen["heroes"].as_array_mut().unwrap()[1]["hero"] = Value::Integer(9);
    chosen["towns"].as_array_mut().unwrap()[0]["at"] = Value::Integer(0);
    chosen["towns"].as_array_mut().unwrap()[0]["status"]["buildings"] =
        Value::Array(vec![Value::Integer(11)]);
    assert_eq!(
        engine::differences(&ours, &theirs),
        [
            "`kingdom.date` differs",
            "`status.army` differs",
            "`status.morale` differs (MCL-Q36: the record has none)",
            "`realm.halls` differs",
            "`chosen.heroes` 0: its tile or path differs",
            "`chosen.heroes` 1: its tile or path differs",
            "`chosen.towns` 0: its tile differs",
            "`chosen.towns 0.buildings` differs",
        ]
    );
    // A list longer on one side, a window that is no table, a kingdom or a window missing.
    let mut theirs = ours.table.clone();
    let chosen = theirs["chosen"].as_table_mut().unwrap();
    chosen["heroes"].as_array_mut().unwrap().pop();
    chosen["towns"]
        .as_array_mut()
        .unwrap()
        .push(Value::Integer(0));
    theirs.insert("realm".into(), Value::Integer(2));
    theirs.remove("kingdom");
    theirs.remove("status");
    assert_eq!(
        engine::differences(&ours, &theirs),
        [
            "`kingdom` differs",
            "`status` differs",
            "`realm` differs",
            "`chosen.heroes` differ in number",
            "`chosen.towns` differ in number",
        ]
    );
    // A chosen hero's row missing: the capture's `heroes` gone, or not a list.
    let mut theirs = ours.table.clone();
    let rows = theirs.remove("heroes").unwrap();
    let mut mine = ours.clone();
    mine.table.insert("heroes".into(), Value::Integer(0));
    let missing = [
        "`heroes` on explored tiles differ",
        "`chosen.heroes` 0: its tile or path differs",
        "`chosen.heroes` 1: its tile or path differs",
    ];
    assert_eq!(engine::differences(&ours, &theirs), missing);
    theirs.insert("heroes".into(), rows);
    assert_eq!(engine::differences(&mine, &theirs), missing);
    // A capture without `[chosen]` compares no choice; nor do two without a kingdom or a window.
    let mut theirs = ours.table.clone();
    theirs.remove("chosen");
    let mut mine = ours.clone();
    for key in ["kingdom", "status"] {
        mine.table.remove(key);
        theirs.remove(key);
    }
    assert!(engine::differences(&mine, &theirs).is_empty());
}

/// E24 M4 T3b2: the map of the mouse-over's facts: Red's hero among objects of every case the
/// facts write (each a one-tile template of its own, `HOVER_LINES`), Red's named Castle, a neutral
/// Rampart, a random monster, Blue's hero far off and a Hill Fort out of sight; since E24 M4 T6
/// (R4-1) an abandoned mine of class 220 and a vertical garrison (219).
const HOVER: &str = r#"
size = 36
terrain = "grass"
heroes = [
    { at = [12, 12], class = 2 },
    { at = [34, 34], owner = 1, class = 6 },
]
towns = [
    { at = [12, 7], faction = 0, owner = 0, buildings = "fort", name = "Brightmoor" },
    { at = [18, 7], faction = 1, buildings = "village" },
]
view = [0, 0, 0]

[[objects]]
def = "hv33.def"
at = [9, 10]

[[objects]]
def = "hv16.def"
at = [10, 10]

[[objects]]
def = "hv88.def"
at = [11, 10]

[[objects]]
def = "hv89.def"
at = [13, 10]

[[objects]]
def = "hv102.def"
at = [14, 10]

[[objects]]
def = "hv102.def"
at = [15, 10]

[[objects]]
def = "hv102.def"
at = [9, 12]

[[objects]]
def = "hv113.def"
at = [10, 12]

[[objects]]
def = "hv113.def"
at = [11, 12]

[[objects]]
def = "hv83.def"
at = [14, 12]
quest = { level = 10 }

[[objects]]
def = "hv10.def"
at = [15, 12]

[[objects]]
def = "hv30.def"
at = [9, 14]

[[objects]]
def = "hv49.def"
at = [10, 14]

[[objects]]
def = "hv22.def"
at = [11, 14]

[[objects]]
def = "hv112.def"
at = [12, 14]

[[objects]]
def = "hv112.def"
at = [13, 14]

[[objects]]
def = "hv35.def"
at = [14, 14]

[[objects]]
def = "hv105.def"
at = [15, 14]

[[objects]]
def = "hv537.def"
at = [9, 16]

[[objects]]
def = "hv53.def"
at = [10, 16]

[[objects]]
def = "hv53.def"
at = [11, 16]
owner = 0

[[objects]]
def = "AVWmrnd0.def"
at = [13, 16]

[[objects]]
def = "hv35.def"
at = [30, 30]

[[objects]]
def = "hv14.def"
at = [14, 16]

[[objects]]
def = "hv92.def"
at = [15, 16]

[[objects]]
def = "hv2207.def"
at = [12, 16]

[[objects]]
def = "hv219.def"
at = [12, 10]
"#;

/// The templates of `HOVER`'s objects: one tile, visited there, by class and subclass; a
/// Rampart's town.
fn hover_templates() -> Vec<ObjectTemplate> {
    let one = |def: &str, class: u32, subclass: u32| {
        format!(
            "{def} 0{} 1{} 111111111 000000001 {class} {subclass} 2 0",
            "1".repeat(47),
            "0".repeat(47)
        )
    };
    let mut lines: Vec<String> = [
        ("hv33.def", 33, 0),
        ("hv16.def", 16, 0),
        ("hv88.def", 88, 0),
        ("hv89.def", 89, 0),
        ("hv102.def", 102, 0),
        ("hv113.def", 113, 0),
        ("hv83.def", 83, 0),
        ("hv10.def", 10, 2),
        ("hv30.def", 30, 0),
        ("hv49.def", 49, 0),
        ("hv22.def", 22, 0),
        ("hv112.def", 112, 0),
        ("hv35.def", 35, 0),
        ("hv105.def", 105, 0),
        ("hv537.def", 53, 7),
        ("hv53.def", 53, 2),
        ("hv14.def", 14, 0),
        ("hv92.def", 92, 0),
        // The classes the game loads as others (E24 M4 T6, R4-1): the abandoned mine 220.7 as
        // 53, the vertical garrison 219 as 33.
        ("hv2207.def", 220, 7),
        ("hv219.def", 219, 0),
    ]
    .iter()
    .map(|&(def, class, subclass)| one(def, class, subclass))
    .collect();
    lines.push(
        LINES[0]
            .replace("AVCcasx0.def", "AVCramx0.def")
            .replace(" 98 0 ", " 98 1 "),
    );
    let mut all: Vec<ObjectTemplate> = lines
        .iter()
        .map(|line| ObjectTemplate::from_objects_txt(line).unwrap())
        .collect();
    all.extend(templates());
    all
}

/// A game on `HOVER`'s map.
fn hover_game() -> Game {
    let (map, extras) = Situation::parse(HOVER)
        .unwrap()
        .build(&BTreeMap::new(), &hover_templates())
        .unwrap();
    game_on(writer::gzip(&writer::write_with(&map, &extras).unwrap()))
}

/// The index of the map's object at `at` whose template is `def`.
fn object_at(g: &Game, def: &str, at: [u8; 2]) -> usize {
    (0..g.art.objects.len())
        .find(|&i| {
            let o = &g.art.objects[i];
            g.art.templates[o.template].def == def && [o.anchor.x, o.anchor.y] == at
        })
        .unwrap()
}

/// The view's object of the map's object `index` (its type and anchor).
fn view_of<'a>(g: &Game, o: &'a mut GameObservation, index: usize) -> &'a mut Object {
    let a = g.art.objects[index].anchor;
    o.adventure
        .objects
        .iter_mut()
        .find(|v| (v.anchor.x, v.anchor.y, v.anchor.level) == (u16::from(a.x), u16::from(a.y), a.z))
        .unwrap()
}

#[test]
fn the_hover_s_facts_of_a_start() {
    let g = hover_game();
    let start = start(&g, &g.observation).unwrap();
    let hover = table(&start.table["hover"]);
    let rows = |key: &str| -> Vec<Vec<i64>> {
        hover[key].as_array().unwrap().iter().map(numbers).collect()
    };
    let castle = object_at(&g, "AVCcasx0.def", [12, 7]);
    let rampart = object_at(&g, "AVCramx0.def", [18, 7]);
    let monster = object_at(&g, "AVWmrnd0.def", [13, 16]);
    let far = object_at(&g, "hv35.def", [30, 30]);
    let words = numbers(&hover["words"]);
    // A town's word its index among the towns written; a monster's the first count of its word;
    // an object out of sight none.
    assert_eq!((words[castle], words[rampart], words[far]), (0, 1, -1));
    let word = g
        .observation
        .adventure
        .objects
        .iter()
        .find_map(|o| o.monster)
        .unwrap();
    let first = [1, 5, 10, 20, 50, 100, 250, 500, 1000][usize::from(word.count_word)];
    assert_eq!(words[monster], first);
    assert_eq!(
        hover["towns"],
        toml::from_str::<toml::Table>(r#"t = [["Brightmoor", 0], [16, 1]]"#).unwrap()["t"]
    );
    assert_eq!(numbers(&hover["town_list"]), [0]);
    assert_eq!(rows("hero_ids"), [vec![16, 0]]);
    assert_eq!(rows("hero_list"), [vec![16, 2, 0]]);
    assert_eq!(numbers(&hover["teams"]), [0, 1, 2, 3, 4, 5, 6, 7]);
    // MCL-Q37: no guarded tile, no reach.
    assert!(
        hover["guarded"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| !r.as_str().unwrap().contains('1'))
    );
    assert!(rows("reach").is_empty());
    let own = g.observation.adventure.heroes[0].own.clone().unwrap();
    assert_eq!(
        hover["day_points"],
        Value::Integer(i64::from(own.max_movement))
    );
    // The mines: the abandoned one's name its own, a neutral one's resource its subclass, Red's its
    // production's.
    let mines = |i: usize| {
        hover["mines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r.as_array().unwrap()[0] == Value::Integer(i64::try_from(i).unwrap()))
            .cloned()
            .unwrap()
    };
    let ore = object_at(&g, "hv53.def", [10, 16]);
    let owned = object_at(&g, "hv53.def", [11, 16]);
    assert_eq!(mines(ore).as_array().unwrap()[1], Value::Integer(2));
    assert_eq!(mines(owned).as_array().unwrap()[1], Value::Integer(2));
    // The abandoned mine of class 220 a mine by the class the game loads (53, E24 M4 T6, R4-1):
    // its row as the one of class 53's, its resource the game's draw (none written).
    let abandoned = object_at(&g, "hv2207.def", [12, 16]);
    let abandoned53 = object_at(&g, "hv537.def", [9, 16]);
    assert!(start.seen[abandoned]);
    assert_eq!(
        mines(abandoned).as_array().unwrap()[1],
        mines(abandoned53).as_array().unwrap()[1]
    );
    assert_eq!(mines(abandoned).as_array().unwrap()[1], Value::Integer(0));
    // The live part keeps the map's class (the sound reads it: 220's own entrance sound).
    assert_eq!(
        numbers(&start.table["classes"].as_array().unwrap()[abandoned]),
        [220, 7]
    );
    // The map's own town names, in its order of towns (an empty one none).
    assert_eq!(g.art.town_names, [Some("Brightmoor".to_string()), None]);
    let mut raw = h3_map::read(&g.bytes, &Limits::default()).unwrap();
    for object in &mut raw.objects {
        if let h3_map::model::Body::Town(town) = &mut object.body {
            town.name = Some(h3_map::model::Text(Vec::new()));
        }
    }
    assert_eq!(MapArt::from_raw(&raw).unwrap().town_names, [None, None]);
    // Read as a capture's: the facts' table as the viewer takes it.
    let heroes = start.table["heroes"].as_array().unwrap().len();
    let facts = Facts::read(&start.table["hover"], g.art.objects.len(), heroes, 36, 1).unwrap();
    assert_eq!(facts.words[castle], Some(0));
    assert!(facts.contents.contains_key(&ore));
    assert!(facts.contents.contains_key(&abandoned));
}

#[test]
fn the_hover_s_facts_of_what_the_player_learnt() {
    let g = hover_game();
    let rules = rules();
    let mut o = g.observation.clone();
    let index = |def: &str, at: [u8; 2]| object_at(&g, def, at);
    let (shrine, shrine2) = (index("hv88.def", [11, 10]), index("hv89.def", [13, 10]));
    let (gems, gold, plain) = (
        index("hv102.def", [14, 10]),
        index("hv102.def", [15, 10]),
        index("hv102.def", [9, 12]),
    );
    let (hut, hut2) = (index("hv113.def", [10, 12]), index("hv113.def", [11, 12]));
    let (bank, fortune, well, corpse) = (
        index("hv16.def", [10, 10]),
        index("hv30.def", [9, 14]),
        index("hv49.def", [10, 14]),
        index("hv22.def", [11, 14]),
    );
    let (mill, mill2, tent, garrison) = (
        index("hv112.def", [12, 14]),
        index("hv112.def", [13, 14]),
        index("hv10.def", [15, 12]),
        index("hv33.def", [9, 10]),
    );
    let (ore, seer, castle) = (
        index("hv53.def", [10, 16]),
        index("hv83.def", [14, 12]),
        index("AVCcasx0.def", [12, 7]),
    );
    let reference = |o: &mut GameObservation, i: usize| {
        let v = view_of(&g, o, i);
        ObjectRef {
            object_type: v.object_type,
            anchor: v.anchor,
        }
    };
    let learnt = |o: &mut GameObservation, i: usize, known: Known| {
        let r = reference(o, i);
        o.adventure.known.push(KnownObject {
            object_type: r.object_type,
            anchor: r.anchor,
            known,
        });
    };
    let (spell, skill) = (3u16, 4u16);
    learnt(&mut o, shrine, Known::Spell { spell: Some(spell) });
    learnt(&mut o, hut, Known::Skill { skill: Some(skill) });
    let price = |resource| {
        vec![Amount {
            resource,
            amount: 1,
        }]
    };
    learnt(&mut o, gems, Known::Price { price: price(5) });
    learnt(&mut o, gold, Known::Price { price: price(6) });
    learnt(&mut o, bank, Known::Emptied { emptied: true });
    learnt(&mut o, fortune, Known::Visited);
    learnt(&mut o, corpse, Known::Emptied { emptied: true });
    let shrine_type = reference(&mut o, shrine).object_type;
    let hut_type = reference(&mut o, hut).object_type;
    o.adventure.kinds = vec![shrine_type, hut_type];
    o.adventure.kinds.sort_unstable();
    o.adventure.keys = 1 << 2;
    view_of(&g, &mut o, mill).visited = true;
    view_of(&g, &mut o, ore).guards = Some(Guards {
        creature: None,
        count_word: 0,
        army: None,
    });
    // The selected hero's marks: the Tree's, the Well's day, the Fountain's once-mark; its spell
    // and skill; it visits Red's town.
    let tree = reference(&mut o, gems);
    let well_type = reference(&mut o, well).object_type;
    let fortune_type = reference(&mut o, fortune).object_type;
    let mark = match &rules
        .adventure()
        .object_type(ObjectTypeId::from_raw(fortune_type))
        .unwrap()
        .kind
    {
        ObjectKind::FountainOfFortune { mark, .. } => *mark,
        kind => panic!("{kind:?}"),
    };
    let hero = &mut o.adventure.heroes[0];
    let own = hero.own.as_mut().unwrap();
    own.marks.push(tree);
    own.timed.push(well_type);
    own.bonuses.once.push(VisitBonus {
        mark,
        morale: 0,
        luck: 1,
    });
    hero.pos.x = 10;
    hero.pos.y = 7;
    let id = hero.id;
    let details = o
        .heroes
        .iter_mut()
        .find(|h| h.id == id)
        .and_then(|h| h.details.as_mut())
        .unwrap();
    details.spells.push(spell);
    details.skills.push(SkillLevel { skill, level: 1 });
    // The vertical garrison (219) a garrison by the class the game loads (33, E24 M4 T6, R4-1).
    let vertical = index("hv219.def", [12, 10]);
    let anchors = [
        reference(&mut o, garrison).anchor,
        reference(&mut o, vertical).anchor,
    ];
    for g in &mut o.garrisons {
        if anchors.contains(&g.anchor) {
            g.garrison.stacks = vec![GarrisonStack {
                creature: 1,
                count_word: None,
                count: None,
            }];
        }
    }
    let learnt_start = start(&g, &o).unwrap();
    let hover = table(&learnt_start.table["hover"]);
    let row_of = |key: &str, i: usize| -> Vec<Value> {
        hover[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_array().unwrap().clone())
            .find(|r| r[0] == Value::Integer(i64::try_from(i).unwrap()))
            .unwrap()
    };
    let ints = |r: Vec<Value>| -> Vec<i64> { r.iter().filter_map(Value::as_integer).collect() };
    let s = i64::from(spell);
    let k = i64::from(skill);
    assert_eq!(
        ints(row_of("shrines", shrine)),
        [i64::try_from(shrine).unwrap(), 1, s, 1]
    );
    // A Shrine of another level: its kind as its type's mark, nothing learnt.
    let other = reference(&mut o, shrine2).object_type;
    let known = i64::from(o.adventure.kinds.contains(&other));
    assert_eq!(ints(row_of("shrines", shrine2))[1..], [known, -1, 0]);
    assert_eq!(ints(row_of("witch_huts", hut))[1..], [1, k, 1]);
    // Another Witch Hut of its type: its kind known, its skill not learnt.
    assert_eq!(ints(row_of("witch_huts", hut2))[1..], [1, -1, 0]);
    assert_eq!(ints(row_of("trees", gems))[1..], [0, 2, 1]);
    assert_eq!(ints(row_of("trees", gold))[1..], [0, 1, 0]);
    assert_eq!(ints(row_of("trees", plain))[1..], [0, -1, 0]);
    assert_eq!(ints(row_of("banks", bank))[1..], [1]);
    // The Seer's Hut: the engine's views show none yet (no row; its hint its name).
    assert!(!learnt_start.seen[seer]);
    let several = row_of("mines", ore);
    assert_eq!(
        several[2],
        Value::Array(vec![Value::Array(vec![
            Value::Integer(i64::from(u32::MAX)),
            Value::Integer(1)
        ])])
    );
    let visits: Vec<Vec<i64>> = hover["visits"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(visits[fortune], [1, 1]);
    assert_eq!(visits[well], [0, 1]);
    assert_eq!(visits[corpse], [0, 1]);
    assert_eq!(
        (visits[mill].clone(), visits[mill2].clone()),
        (vec![0, 1], vec![0, -1])
    );
    assert_eq!(visits[tent], [0, 1]);
    let armed = numbers(&hover["armed"]);
    assert_eq!((armed[garrison], armed[vertical], armed[castle]), (1, 1, 1));
    // A Swan Pond's and the Sirens' once-marks the hero does not hold.
    let (pond, sirens) = (index("hv14.def", [14, 16]), index("hv92.def", [15, 16]));
    assert_eq!(
        (visits[pond].clone(), visits[sirens].clone()),
        (vec![0, 0], vec![0, 0])
    );
    // A town with a Capitol: its picture the Capitol's.
    let mut capitol = o.clone();
    capitol.towns[0].capitol = true;
    let with = start(&g, &capitol).unwrap();
    let defs = with.table["defs"].as_array().unwrap();
    assert_eq!(defs[castle].as_str(), Some("capitol0.def"));
    // Without a hero of the viewer's: no mark a hero keeps, no day's points.
    let mut none = o.clone();
    none.adventure.heroes.retain(|h| h.owner != 0);
    let alone = start(&g, &none).unwrap();
    let hover = table(&alone.table["hover"]);
    let visits: Vec<Vec<i64>> = hover["visits"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(
        (visits[well].clone(), visits[corpse].clone()),
        (vec![0, -1], vec![0, -1])
    );
    assert_eq!(hover["day_points"], Value::Integer(0));
    assert!(numbers(&row_of_none(hover, gems))[3] == -1);
}

/// The row of `trees` of the object `i` in `hover`.
fn row_of_none(hover: &toml::Table, i: usize) -> Value {
    hover["trees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r.as_array().unwrap()[0] == Value::Integer(i64::try_from(i).unwrap()))
        .cloned()
        .unwrap()
}

/// E24 M4 T6 (R4-7): a random town in Red's sight and a neutral Rampart with a Capitol out of it.
const TOWNS: &str = r#"
size = 36
terrain = "grass"
heroes = [
    { at = [12, 12], class = 2 },
    { at = [34, 34], owner = 1, class = 6 },
]
towns = [{ at = [30, 24], faction = 1, buildings = "capitol" }]
view = [0, 0, 0]

[[objects]]
def = "AVCranx0.def"
at = [12, 7]
"#;

#[test]
fn a_random_town_in_sight_and_a_town_out_of_it() {
    let random = LINES[0]
        .replace("AVCcasx0.def", "AVCranx0.def")
        .replace(" 98 0 ", " 77 0 ");
    let mut lines = hover_templates();
    lines.push(ObjectTemplate::from_objects_txt(&random).unwrap());
    let (map, extras) = Situation::parse(TOWNS)
        .unwrap()
        .build(&BTreeMap::new(), &lines)
        .unwrap();
    let g = game_on(writer::gzip(&writer::write_with(&map, &extras).unwrap()));
    let start = start(&g, &g.observation).unwrap();
    let defs = start.table["defs"].as_array().unwrap();
    let classes = start.table["classes"].as_array().unwrap();
    // The random town a town of the faction the game drew, by its view's fort and Capitol
    // (design §5.11); its word, by the town class's rule, its index among the towns written.
    let random = object_at(&g, "AVCranx0.def", [12, 7]);
    assert!(start.seen[random]);
    let view = g
        .observation
        .adventure
        .objects
        .iter()
        .find(|o| (o.anchor.x, o.anchor.y) == (12, 7))
        .unwrap();
    let (at, town) = g
        .observation
        .towns
        .iter()
        .enumerate()
        .find(|(_, t)| Some(t.town) == view.town)
        .unwrap();
    let list = match (town.capitol, town.fort > 0) {
        (true, _) => "capitol",
        (false, true) => "fort",
        (false, false) => "village",
    };
    assert_eq!(
        defs[random].as_str(),
        Some(format!("{list}{}.def", town.faction).as_str())
    );
    assert_eq!(numbers(&classes[random]), [98, i64::from(town.faction)]);
    let words = numbers(&start.table["hover"]["words"]);
    assert_eq!(words[random], i64::try_from(at).unwrap());
    // The Rampart out of sight: its new game's picture by the map's buildings, a Capitol's.
    let rampart = object_at(&g, "AVCramx0.def", [30, 24]);
    assert!(!start.seen[rampart]);
    assert_eq!(defs[rampart].as_str(), Some("capitol1.def"));
    assert_eq!(words[rampart], -1);
}

/// A game of `MAP` hosted (`GameSession`), Red's hero walking two tiles north, then every
/// player ending its turn for `days` days; Red's record.
fn session_record(g: &Game, days: u32) -> String {
    session_records(g, days).swap_remove(0)
}

/// As [`session_record`], each player's record (E24 M4 T6), by the players' order.
fn session_records(g: &Game, days: u32) -> Vec<String> {
    use h3_protocol::session::game::{Clock, CommandBounds, GameOptions, GameSession, Spectators};
    use h3_protocol::wire::game::{AdventureCommand, GameCommand, Pos};
    let rules = rules();
    let raw = h3_map::read(&g.bytes, &Limits::default()).unwrap();
    let setup = h3_map::import(&raw, rules).unwrap();
    let options = GameOptions {
        game: 3,
        checkpoint_every: 0,
        record_bytes: usize::MAX,
        clock: Clock::UNLIMITED,
        commands: CommandBounds::NONE,
        spectators: Spectators::NONE,
    };
    let manifest = g.header.manifest.clone();
    let mut out = Vec::new();
    let mut session = GameSession::start(rules, &setup, 7, manifest, options, &mut out).unwrap();
    let hero = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let path = [9, 8].map(|y| Pos { x: 10, y, level: 0 }).to_vec();
    let walk = GameCommand::Adventure {
        command: AdventureCommand::MoveHero { hero, path },
    };
    let mut seq = 1;
    session.play(PlayerId::new(0), seq, walk, &mut out).unwrap();
    for _ in 0..days {
        for player in 0..3 {
            seq += 1;
            session
                .play(
                    PlayerId::new(player),
                    seq,
                    GameCommand::EndTurn {},
                    &mut out,
                )
                .unwrap();
        }
    }
    (0..3)
        .map(|p| session.record(PlayerId::new(p)).unwrap())
        .collect()
}

#[test]
fn a_record_of_a_player_not_first() {
    // E24 M4 T6 (the reviewers' R1-1, R4-7 (a)): a map whose second player is Green (colour 3, so
    // its id and its colour differ); its record starts at the game's start in Red's turn: it
    // waits, Red's walk in its sight is a computer's turn shown first, then its own first day
    // (its colour, the date's window, day 1), every later day after the turn before it; the
    // record ends in Red's turn (open), so that its last End Turn has a turn to show.
    let g = game_on(map_file(&MAP.replace(
        "{ at = [12, 10], owner = 1, class = 9 }",
        "{ at = [12, 10], owner = 3, class = 9 }",
    )));
    let colours: Vec<u8> = g.observation.players.iter().map(|p| p.colour).collect();
    let green = colours.iter().position(|&c| c == 3).unwrap();
    assert_ne!(green, 3);
    assert!(green > 0);
    let records = session_records(&g, 2);
    let loaded = engine::load(&records[green], &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    assert!(played.waits);
    let player = |i: usize| table(&played.turns[i])["player"].as_integer();
    let kind = |i: usize| table(&table(&played.turns[i])["status"])["kind"].as_integer();
    let day = |i: usize| numbers(&table(&table(&played.turns[i])["kingdom"])["date"]);
    // Red's walk (two steps north of its hero, beside Green's) shown first, as Red's turn.
    let red = &played.others[0];
    assert_eq!((red.turn, red.player, red.open), (0, 0, false));
    assert_eq!(red.walks[0].steps.len(), 2);
    assert_eq!((player(0), kind(0)), (Some(0), Some(5)));
    // Green's first day, then its second after Red's (no walk of Red's: none shown).
    assert_eq!((player(1), kind(1), day(1)[2]), (Some(3), Some(1), 1));
    assert_eq!((player(2), kind(2), day(2)[2]), (Some(3), Some(1), 2));
    // The record's end in Red's day 3: a turn of Red's, open, without a walk.
    let last = played.others.last().unwrap();
    assert_eq!(
        (last.turn, last.player, last.open, last.walks.is_empty()),
        (played.turns.len() - 1, 0, true, true)
    );
    assert_eq!((player(last.turn), kind(last.turn)), (Some(0), Some(5)));
    // Green's list at each of its days: its one hero.
    let hero = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == u8::try_from(green).unwrap())
        .unwrap()
        .id;
    assert_eq!(played.lists[1], [hero]);
    assert_eq!(
        played.commands,
        [engine::Command::EndTurn, engine::Command::EndTurn]
    );
}

#[test]
fn a_record_s_walks_and_turns() {
    let g = game();
    let text = session_record(&g, 7);
    let loaded = engine::load(&text, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    // The walk: its tiles, its points at its last step, its whole reveal at its first (MCL-Q39
    // (c)): the rows above the explored circle.
    let [walk] = &played.moves[..] else {
        panic!("one walk: {:?}", played.moves);
    };
    let at = |x, y| h3_map::model::Pos { x, y, z: 0 };
    assert_eq!(walk.tiles, [at(10, 9), at(10, 8)]);
    assert_eq!(walk.points[0], None);
    assert!(walk.points[1].is_some_and(|p| p > 0 && p < 2000));
    assert!(!walk.revealed[0].is_empty() && walk.revealed[1].is_empty());
    assert!(walk.revealed[0].iter().all(|t| t.y < 8));
    assert_eq!(played.ends, [Some(at(10, 8))]);
    // Red's turns' starts, days 2 to 8: the date's window, the panels' kingdom with the day, the
    // week's kind and its client's name (MCL-Q40 (b): week 2 the names' second).
    assert_eq!(played.turns.len(), 7);
    let turn = |i: usize| table(&played.turns[i]);
    for (i, day) in (2..=8).enumerate() {
        let t = turn(i);
        assert_eq!(t["player"].as_integer(), Some(0));
        assert_eq!(table(&t["status"])["kind"].as_integer(), Some(1));
        let date = numbers(&table(&t["kingdom"])["date"]);
        assert_eq!(
            date,
            [1, i64::from(day > 7) + 1, (day - 1) % 7 + 1],
            "day {day}"
        );
        assert!(!t.contains_key("realm") && !t.contains_key("chosen"));
    }
    // The walker's points at its turn's start the new day's (E24 M4 T4b2: told after
    // `TurnStarted`), never the walk's leftover.
    let leftover = i64::from(walk.points[1].unwrap());
    let rows = table(&turn(0)["kingdom"])["heroes"]
        .as_array()
        .unwrap()
        .clone();
    assert!(rows.iter().all(|r| numbers(r)[3] != leftover));
    assert_eq!(numbers(&turn(0)["week"]), [0, 0]);
    assert_eq!(numbers(&turn(6)["week"]), [0, 1]);
    assert_eq!(numbers(&turn(6)["month"]), [0, 0]);
    // Each reads as a recording's `[[turn]]` table.
    assert!(
        played
            .turns
            .iter()
            .all(|t| h3_map_view::live::Turn::read(t).is_some())
    );
}

#[test]
fn a_record_s_rarer_lines() {
    use h3_protocol::wire::game::{AdventureCommand, GameCommand, GameEvent, Month, Pos, Week};
    let g = game();
    let record = session_record(&g, 0);
    let mut lines: Vec<String> = record.lines().map(String::from).collect();
    let line = |m: &ServerMessage| codec::encode_text(m);
    let moved = |path: Vec<Pos>| ServerMessage::GameAccepted {
        n: 90,
        seq: 9,
        command: GameCommand::Adventure {
            command: AdventureCommand::MoveHero { hero: 0, path },
        },
    };
    // A walk without a tile, a month's start with Red's turn, a fresh snapshot, another message,
    // then a walk the record ends in.
    lines.push(line(&moved(Vec::new())));
    lines.push(line(&ServerMessage::GameTurn {
        n: 91,
        legal: Vec::new(),
        castable: Vec::new(),
    }));
    // Each week's and month's kind but the normal ones, each with a turn's start; a creature by
    // the original's number (E24 M5 T4c: AB's Peasants 139 and Pixies 118, the Ballista 146).
    let starts = [
        (
            GameEvent::MonthStarted {
                month: 2,
                kind: Month::Plague {},
            },
            &[2, 0][..],
        ),
        (
            GameEvent::MonthStarted {
                month: 3,
                kind: Month::Creature {
                    creature: handle("ab:peasant"),
                },
            },
            &[1, 139][..],
        ),
        (
            GameEvent::WeekStarted {
                week: 9,
                kind: Week::Creature {
                    creature: handle("ab:pixie"),
                },
            },
            &[1, 118][..],
        ),
        (
            GameEvent::WeekStarted {
                week: 10,
                kind: Week::Of {
                    creatures: [Some(30), Some(31)],
                    bonus: 5,
                },
            },
            // Its creatures and growth the engine's (E24 M5 T4c).
            &[2, 30, 31, 5][..],
        ),
        (
            GameEvent::WeekStarted {
                week: 11,
                kind: Week::Of {
                    creatures: [Some(handle("base:ballista")), None],
                    bonus: 5,
                },
            },
            &[2, 146, -1, 5][..],
        ),
    ];
    for (start, _) in &starts {
        lines.push(line(&ServerMessage::GameEvents {
            n: 92,
            events: vec![start.clone(), GameEvent::TurnStarted { player: 0 }],
        }));
    }
    lines.push(line(&snapshot(&g.observation)));
    lines.push(line(&ServerMessage::MatchCancelled {}));
    lines.push(line(&moved(vec![Pos {
        x: 11,
        y: 10,
        level: 0,
    }])));
    let record = lines.join("\n");
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    assert_eq!(played.moves.len(), 3);
    assert!(played.moves[1].tiles.is_empty() && played.moves[1].points.is_empty());
    let last = &played.moves[2];
    assert_eq!(
        (last.points.as_slice(), last.revealed.as_slice()),
        (&[None][..], &[Vec::new()][..])
    );
    for (i, (start, want)) in starts.iter().enumerate() {
        let key = if matches!(start, GameEvent::WeekStarted { .. }) {
            "week"
        } else {
            "month"
        };
        assert_eq!(numbers(&table(&played.turns[i])[key]), *want, "{start:?}");
    }
    // A record without its first snapshot has no start.
    let no_start = text(&g.header, &[ServerMessage::MatchCancelled {}]);
    let loaded = engine::load(&no_start, &g.bytes, rules()).unwrap();
    assert_eq!(
        engine::played(&loaded, &g.art, rules()),
        Err(Refused::NoStart)
    );
    // A path's tile past a byte refuses the record.
    let mut bad: Vec<String> = record.lines().map(String::from).collect();
    bad.push(line(&moved(vec![Pos {
        x: 1,
        y: 300,
        level: 0,
    }])));
    let loaded = engine::load(&bad.join("\n"), &g.bytes, rules()).unwrap();
    assert!(matches!(
        engine::played(&loaded, &g.art, rules()),
        Err(Refused::Line(_, why)) if why.contains("off the map")
    ));
}

/// Every creature the import knows by the original's number (E24 M5 T4c, design §5.29): a
/// monster of each number a map file holds (to 144, `writer::CREATURE_COUNT`) on a map, imported
/// to its handle (the packs' order: AB's from 123), numbered back as its number; the war machines
/// (handles 118 to 122) by their keys; every handle of the packs numbered, one past them none.
#[test]
fn every_creature_by_the_original_s_number() {
    // The numbers no creature has: the original's unused 122, 124, 126, 128.
    let known: Vec<u16> = (0..writer::CREATURE_COUNT)
        .filter(|n| ![122, 124, 126, 128].contains(n))
        .collect();
    let at = |i: usize| (2 + i % 16 * 2, 2 + i / 16 * 3);
    let mut text = String::from(
        "size = 36\nterrain = \"grass\"\nheroes = [{ at = [1, 34] }, { at = [34, 34], owner = 1 }]\nview = [0, 0, 0]\n",
    );
    for (i, n) in known.iter().enumerate() {
        let (x, y) = at(i);
        text.push_str(&format!(
            "[[objects]]\ndef = \"M{n:03}.def\"\nat = [{x}, {y}]\nmonster = {{ count = 1, disposition = 0 }}\n"
        ));
    }
    let bytes = map_file(&text);
    let raw = h3_map::read(&bytes, &Limits::default()).unwrap();
    let setup = h3_map::import(&raw, rules()).unwrap();
    let creatures = engine::Creatures::new(rules());
    let numbered: Vec<(u16, u16)> = setup
        .map
        .objects
        .iter()
        .filter_map(|o| {
            let handle = o.monster.as_ref()?.creature?.raw();
            let i = (0..known.len())
                .find(|&i| at(i) == (usize::from(o.anchor.x), usize::from(o.anchor.y)))?;
            Some((known[i], creatures.number(handle)))
        })
        .collect();
    assert_eq!(numbered.len(), known.len());
    for (number, numbered) in numbered {
        assert_eq!(numbered, number);
    }
    let count = u16::try_from(rules().core().creature_count()).unwrap();
    assert!((0..count).all(|h| creatures.number(h) != engine::NONE));
    assert_eq!(creatures.number(count), engine::NONE);
    let machines = [
        "base:catapult",
        "base:ballista",
        "base:first_aid_tent",
        "base:ammo_cart",
        "base:arrow_tower",
    ]
    .map(handle);
    assert_eq!(machines, [118, 119, 120, 121, 122]);
    assert_eq!(
        machines.map(|h| creatures.number(h)),
        [145, 146, 147, 148, 149]
    );
    // The handles past the original's numbers.
    let handles = ["ab:pixie", "ab:peasant"].map(handle);
    assert_eq!(handles, [123, 140]);
    assert_eq!(handles.map(|h| creatures.number(h)), [118, 139]);
}

/// A start's creatures by the original's numbers (E24 M5 T4c, design §5.29): a hero's army, a
/// mine's guards, a random monster's picture and class (the mouse-over's hint by it).
#[test]
fn a_start_s_creatures_by_the_original_s_numbers() {
    use h3_protocol::wire::game::ArmyStack;
    let g = hover_game();
    let mut o = g.observation.clone();
    let [peasant, pixie] = ["ab:peasant", "ab:pixie"].map(handle);
    let viewer = o.adventure.viewer;
    let hero = o
        .adventure
        .heroes
        .iter_mut()
        .find(|h| h.owner == viewer)
        .unwrap();
    hero.own.as_mut().unwrap().army = vec![
        ArmyStack {
            slot: 0,
            creature: peasant,
            count: 10,
        },
        ArmyStack {
            slot: 1,
            creature: pixie,
            count: 5,
        },
    ];
    let ore = object_at(&g, "hv53.def", [10, 16]);
    view_of(&g, &mut o, ore).guards = Some(Guards {
        creature: Some(peasant),
        count_word: 0,
        army: None,
    });
    let monster = object_at(&g, "AVWmrnd0.def", [13, 16]);
    view_of(&g, &mut o, monster)
        .monster
        .as_mut()
        .unwrap()
        .creature = pixie;
    let start = start(&g, &o).unwrap();
    let t = &start.table;
    let army: Vec<Vec<i64>> = t["status"]["army"]
        .as_array()
        .unwrap()
        .iter()
        .map(numbers)
        .collect();
    assert_eq!(army[..3], [vec![139, 10], vec![118, 5], vec![-1, 0]]);
    let mine = t["hover"]["mines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r.as_array().unwrap()[0] == Value::Integer(i64::try_from(ore).unwrap()))
        .unwrap();
    assert_eq!(
        mine[2],
        Value::Array(vec![Value::Array(vec![139.into(), 1.into()])])
    );
    assert_eq!(
        numbers(&t["classes"].as_array().unwrap()[monster]),
        [54, 118]
    );
    assert_eq!(
        t["defs"].as_array().unwrap()[monster].as_str(),
        Some("M118.def")
    );
}

/// A visit's events with their creatures by the original's numbers (E24 M5 T4c, design §5.29):
/// the stacks a hero's army and its dialog tell, the creatures joined, refused and none to
/// recruit, the command's and an answer's; every other event the record's.
#[test]
fn a_visit_s_creatures_by_the_original_s_numbers() {
    use h3_protocol::wire::game::{
        AdventureCommand, AdventureEvent, Answer, ArmyRef, ArmySlot, ArmyStack, Consent,
        GameCommand, GameEvent, Pos,
    };
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let [peasant, pixie, ballista] = ["ab:peasant", "ab:pixie", "base:ballista"].map(handle);
    let stack = |creature| ArmyStack {
        slot: 1,
        creature,
        count: 4,
    };
    let at = ArmySlot {
        army: ArmyRef::Hero { hero: red },
        slot: 1,
    };
    let joined = |creature| GameEvent::Adventure {
        event: AdventureEvent::CreaturesJoined {
            hero: red,
            creature,
            count: 4,
        },
    };
    let anchor = Pos {
        x: 11,
        y: 10,
        level: 0,
    };
    let walked = |c: [u16; 4]| {
        vec![
            GameEvent::StackChanged {
                player: 0,
                at,
                stack: Some(stack(c[0])),
            },
            GameEvent::StackChanged {
                player: 0,
                at,
                stack: None,
            },
            GameEvent::Adventure {
                event: AdventureEvent::JoinRefused {
                    hero: red,
                    creature: c[1],
                    count: 4,
                },
            },
            GameEvent::Adventure {
                event: AdventureEvent::NoneToRecruit {
                    object_type: 7,
                    anchor,
                    creature: c[2],
                },
            },
            GameEvent::CreaturesRaised {
                player: 0,
                hero: red,
                creature: c[3],
                count: 4,
            },
        ]
    };
    let answered = |c: [u16; 2]| {
        vec![
            GameEvent::ArmyDialogOpened {
                player: 0,
                hero: red,
                stack: stack(c[0]),
            },
            joined(c[1]),
        ]
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            accepted(GameCommand::Adventure {
                command: AdventureCommand::MoveHero {
                    hero: red,
                    path: vec![anchor],
                },
            }),
            events(walked([peasant, pixie, ballista, peasant])),
            accepted(GameCommand::Answer {
                answer: Answer::Offer {
                    consent: Consent::Accept,
                },
            }),
            events(answered([pixie, peasant])),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let [visit] = &played.visits[..] else {
        panic!("one visit: {:?}", played.visits);
    };
    // A creature raised is no reading of a visit's: the record's handle.
    assert_eq!(visit.events, walked([139, 118, 146, peasant]));
    assert_eq!(visit.answers[0].1, answered([118, 139]));
    let gain = engine::visits::gain(&visit.events);
    assert_eq!(gain.army[1].map(|s| s.creature), Some(139));
}

/// A message of Red's command `command` accepted.
fn accepted(command: h3_protocol::wire::game::GameCommand) -> ServerMessage {
    ServerMessage::GameAccepted {
        n: 50,
        seq: 5,
        command,
    }
}

/// A message of `events`.
fn events(events: Vec<h3_protocol::wire::game::GameEvent>) -> ServerMessage {
    ServerMessage::GameEvents { n: 51, events }
}

#[test]
fn a_record_s_walks_short_of_their_paths() {
    // E24 M4 T6: a walk an event stops after its first step (its points there, its visit where it
    // stopped), then one whose last node is visited from beside (its points at its last, its
    // visit on the node); one taken whole stops nowhere.
    use h3_protocol::wire::game::{AdventureCommand, AdventureEvent, GameCommand, GameEvent, Pos};
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let p = |x, y| Pos { x, y, level: 0 };
    let walk = |path| {
        accepted(GameCommand::Adventure {
            command: AdventureCommand::MoveHero { hero: red, path },
        })
    };
    let step = |from, to| GameEvent::Adventure {
        event: AdventureEvent::HeroStepped {
            hero: red,
            from: Some(from),
            to: Some(to),
        },
    };
    let points = |movement| GameEvent::Adventure {
        event: AdventureEvent::MovementChanged {
            hero: red,
            movement,
            max_movement: 2000,
        },
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            walk(vec![p(10, 9), p(10, 8), p(10, 7)]),
            events(vec![step(p(10, 10), p(10, 9)), points(1800)]),
            walk(vec![p(10, 8), p(10, 7)]),
            events(vec![step(p(10, 9), p(10, 8)), points(1600)]),
            walk(vec![p(10, 7)]),
            events(vec![step(p(10, 8), p(10, 7)), points(1500)]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let at = |x, y| h3_map::model::Pos { x, y, z: 0 };
    let stops: Vec<_> = played.moves.iter().map(|m| m.stops).collect();
    assert_eq!(stops, [Some(1), Some(1), None]);
    assert_eq!(played.moves[0].points, [Some(1800), None, None]);
    assert_eq!(played.moves[1].points, [None, Some(1600)]);
    let visited: Vec<_> = played.visits.iter().map(|v| v.at).collect();
    assert_eq!(visited, [at(10, 9), at(10, 7), at(10, 7)]);
}

#[test]
fn a_record_s_visits_and_level_ups() {
    use h3_protocol::wire::game::{
        AdventureCommand, AdventureEvent, Answer, ChestChoice, GameCommand, GameEvent, HeroChange,
        Pos, PrimarySkill, SkillOffer,
    };
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .clone();
    let details = g
        .observation
        .heroes
        .iter()
        .find(|i| i.id == red.id)
        .and_then(|i| i.details.clone())
        .unwrap();
    let known = details.skills[0];
    let blue = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 1)
        .unwrap()
        .id;
    let moved = |x, y| {
        accepted(GameCommand::Adventure {
            command: AdventureCommand::MoveHero {
                hero: red.id,
                path: vec![Pos { x, y, level: 0 }],
            },
        })
    };
    let answer = |answer| accepted(GameCommand::Answer { answer });
    let gold = GameEvent::ResourceGained {
        player: 0,
        resource: 6,
        amount: 500,
    };
    let opened = GameEvent::LevelUpOpened {
        player: 0,
        hero: red.id,
        level: 2,
        primary: PrimarySkill::Defense,
        offers: [known.skill, 20],
    };
    let gained = |level, skill| GameEvent::LevelGained {
        player: 0,
        hero: red.id,
        level,
        primary: PrimarySkill::Knowledge,
        skill,
    };
    let removed = GameEvent::Adventure {
        event: AdventureEvent::ObjectRemoved {
            object_type: 6,
            anchor: Pos {
                x: 11,
                y: 10,
                level: 0,
            },
        },
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            // A walk with a visit's events, an answer and its own, a level-up opened and taken.
            moved(11, 10),
            events(vec![removed.clone(), gold.clone(), opened]),
            answer(Answer::Chest {
                choice: ChestChoice::Gold,
            }),
            events(vec![gold.clone()]),
            answer(Answer::LevelUp {
                offer: SkillOffer::Second,
            }),
            events(vec![gained(2, Some(20))]),
            // A level without a decision (one offer, then none), an empty walk, End Turn: the
            // events after it belong to no visit; a level-up of a hero without details nothing.
            moved(12, 10),
            events(vec![gained(3, Some(known.skill)), gained(4, None)]),
            accepted(GameCommand::Adventure {
                command: AdventureCommand::MoveHero {
                    hero: red.id,
                    path: Vec::new(),
                },
            }),
            answer(Answer::Chest {
                choice: ChestChoice::Experience,
            }),
            accepted(GameCommand::EndTurn {}),
            events(vec![
                gold.clone(),
                GameEvent::HeroChanged {
                    player: 0,
                    hero: red.id,
                    change: HeroChange::GainExperience { amount: 5 },
                },
                gained(5, None),
                GameEvent::LevelGained {
                    player: 0,
                    hero: 999,
                    level: 2,
                    primary: PrimarySkill::Attack,
                    skill: None,
                },
                // Blue's hero (no details in Red's view), an offer past the skills' bytes.
                GameEvent::LevelUpOpened {
                    player: 1,
                    hero: blue,
                    level: 2,
                    primary: PrimarySkill::Attack,
                    offers: [1, 2],
                },
                GameEvent::LevelUpOpened {
                    player: 0,
                    hero: red.id,
                    level: 6,
                    primary: PrimarySkill::Attack,
                    offers: [300, 2],
                },
                // The other primary skills; another hero's step with no walk open.
                GameEvent::LevelGained {
                    player: 0,
                    hero: red.id,
                    level: 7,
                    primary: PrimarySkill::Attack,
                    skill: None,
                },
                GameEvent::LevelGained {
                    player: 0,
                    hero: red.id,
                    level: 8,
                    primary: PrimarySkill::Power,
                    skill: None,
                },
                GameEvent::Adventure {
                    event: AdventureEvent::HeroStepped {
                        hero: blue,
                        from: None,
                        to: Some(Pos {
                            x: 30,
                            y: 30,
                            level: 0,
                        }),
                    },
                },
            ]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let tile = |x, y| h3_map::model::Pos { x, y, z: 0 };
    let [first, second] = &played.visits[..] else {
        panic!("two visits: {:?}", played.visits);
    };
    assert_eq!(first.at, tile(11, 10));
    assert_eq!(first.events.len(), 3);
    assert_eq!(first.answers.len(), 2);
    assert_eq!(first.answers[0].1, [gold]);
    assert_eq!((second.at, second.events.len()), (tile(12, 10), 2));
    assert!(second.answers.is_empty());
    // Where each walk left its hero: none for those without a step (a visit from beside).
    assert_eq!(played.ends, [None, None, None]);
    // E24 M4 T4c: each walk's hero, the viewer's commands (an empty path no walk), Red's place in
    // its list, the heroes the start shows.
    assert_eq!(played.walkers, [red.id; 3]);
    use h3_map_view::engine::Command;
    assert_eq!(
        played.commands,
        [
            Command::Walk {
                hero: red.id,
                to: tile(11, 10)
            },
            Command::Answer(Answer::Chest {
                choice: ChestChoice::Gold
            }),
            Command::Answer(Answer::LevelUp {
                offer: SkillOffer::Second
            }),
            Command::Walk {
                hero: red.id,
                to: tile(12, 10)
            },
            Command::Other,
            Command::Answer(Answer::Chest {
                choice: ChestChoice::Experience
            }),
            Command::EndTurn,
        ]
    );
    assert_eq!(played.slots, [(red.id, 0)]);
    assert_eq!(played.shown.len(), g.observation.adventure.heroes.len());
    // The level-ups: the dialog's offers with the levels they reach, the record's choice.
    let levels: Vec<_> = played
        .levels
        .iter()
        .map(|l| {
            let offers: Vec<_> = l.up.offers.iter().map(|o| (o.skill, o.level)).collect();
            (l.up.level, l.up.primary, offers, l.chosen)
        })
        .collect();
    let skill = u8::try_from(known.skill).unwrap();
    assert_eq!(
        levels,
        [
            (2, 1, vec![(skill, known.level + 1), (20, 1)], Some(1)),
            (3, 3, vec![(skill, known.level + 1)], Some(0)),
            (4, 3, Vec::new(), None),
            (5, 3, Vec::new(), None),
            (7, 0, Vec::new(), None),
            (8, 2, Vec::new(), None),
        ]
    );
    assert!(
        played
            .levels
            .iter()
            .all(|l| u16::from(l.up.hero) == details.hero_type)
    );
    // E24 M4 T6 (the reviewer's R2-1, R2-3, R2-4): each visit's walk (the empty walk's none), its
    // hero, what it held, its own level-ups (the walk's and its answers', none after End Turn);
    // each level-up's class the record's rules'.
    assert_eq!((first.walk, second.walk), (0, 1));
    assert_eq!((first.hero, second.hero), (red.id, red.id));
    let hero_type = u8::try_from(details.hero_type).ok();
    assert_eq!((first.hero_type, second.hero_type), (hero_type, hero_type));
    assert_eq!(first.held.mana, details.mana);
    assert_eq!(
        (first.levels.as_slice(), second.levels.as_slice()),
        (&[0][..], &[1, 2][..])
    );
    let class = rules()
        .core()
        .hero_type(h3_kernel::Handle::from_raw(details.hero_type))
        .map(|t| u8::try_from(t.class.index()).unwrap())
        .unwrap();
    assert!(played.levels.iter().all(|l| l.class == class));
}

/// A visit through its army dialog (E24 M4 T6, the reviewer's R2-4: a reward's creatures with no
/// room, GAM-DEC-004): the dialog's commands keep the visit, its close's events (the experience,
/// the level-up) and the level-up's answer are the visit's; a command after the dialog's close
/// ends it.
#[test]
fn a_record_s_visit_through_its_army_dialog() {
    use h3_protocol::wire::game::{
        AdventureCommand, Answer, ArmyStack, Consent, GameCommand, GameEvent, HeroChange, Manage,
        Pos, PrimarySkill, SkillOffer,
    };
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .clone();
    let tactics = accepted(GameCommand::Manage {
        manage: Manage::Tactics {
            hero: red.id,
            on: true,
        },
    });
    let answer = |answer| accepted(GameCommand::Answer { answer });
    let level = |level| GameEvent::LevelGained {
        player: 0,
        hero: red.id,
        level,
        primary: PrimarySkill::Attack,
        skill: None,
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            accepted(GameCommand::Adventure {
                command: AdventureCommand::MoveHero {
                    hero: red.id,
                    path: vec![Pos {
                        x: 11,
                        y: 10,
                        level: 0,
                    }],
                },
            }),
            answer(Answer::Offer {
                consent: Consent::Accept,
            }),
            events(vec![GameEvent::ArmyDialogOpened {
                player: 0,
                hero: red.id,
                stack: ArmyStack {
                    slot: 0,
                    creature: 18,
                    count: 10,
                },
            }]),
            tactics.clone(),
            answer(Answer::Close {}),
            events(vec![
                GameEvent::DecisionClosed { player: 0 },
                GameEvent::HeroChanged {
                    player: 0,
                    hero: red.id,
                    change: HeroChange::GainExperience { amount: 1000 },
                },
                level(2),
            ]),
            answer(Answer::LevelUp {
                offer: SkillOffer::First,
            }),
            // The dialog closed: a management command ends the visit.
            tactics,
            events(vec![level(3)]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let [visit] = &played.visits[..] else {
        panic!("one visit: {:?}", played.visits);
    };
    let answers: Vec<Answer> = visit.answers.iter().map(|(a, _)| *a).collect();
    assert_eq!(
        answers,
        [
            Answer::Offer {
                consent: Consent::Accept
            },
            Answer::Close {},
            Answer::LevelUp {
                offer: SkillOffer::First
            },
        ]
    );
    assert_eq!(visit.answers[1].1.len(), 3);
    assert_eq!(visit.levels, [0]);
    assert_eq!(played.levels.len(), 2);
}

/// Another player's turn in a record (E24 M4 T4c, design §5.16): Blue's steps by hero, a boat
/// boarded and left, a step out of sight; Purple's turn with none seen no turn; a town's command
/// no walk.
#[test]
fn a_record_s_other_players_turns() {
    use h3_map_view::engine::{Command, OtherEnd, OtherStep};
    use h3_map_view::live::{Facing, LiveBoat};
    use h3_map_view::timeline::Sea;
    use h3_protocol::wire::game::{AdventureEvent, GameCommand, GameEvent, Pos, TownCommand};
    let g = game();
    // The pile's and the monster's types (E24 M4 T6, R1-5: a pickup dissolved, a monster not).
    let type_at = |x, y| {
        g.observation
            .adventure
            .objects
            .iter()
            .find(|o| (o.anchor.x, o.anchor.y) == (x, y))
            .unwrap()
            .object_type
    };
    let (pile, monster) = (type_at(13, 12), type_at(10, 13));
    assert!(engine::pickup(rules(), pile) && !engine::pickup(rules(), monster));
    let hero = |owner| {
        g.observation
            .adventure
            .heroes
            .iter()
            .find(|h| h.owner == owner)
            .unwrap()
            .id
    };
    let (red, blue) = (hero(0), hero(1));
    let pos = |x, y| Pos { x, y, level: 0 };
    let event = |event| GameEvent::Adventure { event };
    let step = |hero, from: Option<Pos>, to: Option<Pos>| {
        event(AdventureEvent::HeroStepped { hero, from, to })
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            accepted(GameCommand::Town {
                command: TownCommand::Build {
                    town: 0,
                    building: 0,
                },
            }),
            accepted(GameCommand::EndTurn {}),
            events(vec![
                GameEvent::TurnEnded { player: 0 },
                GameEvent::TurnStarted { player: 1 },
            ]),
            events(vec![
                step(blue, Some(pos(12, 10)), Some(pos(12, 9))),
                event(AdventureEvent::Boarded {
                    hero: blue,
                    boat: 1,
                    at: pos(12, 9),
                }),
                step(blue, Some(pos(12, 9)), Some(pos(12, 8))),
                event(AdventureEvent::Landed {
                    hero: blue,
                    boat: 1,
                    at: pos(12, 8),
                    facing: 0,
                }),
                // A boat of a hero not walking: nothing.
                event(AdventureEvent::Boarded {
                    hero: red,
                    boat: 0,
                    at: pos(1, 1),
                }),
                step(red, Some(pos(10, 10)), Some(pos(10, 9))),
                step(blue, Some(pos(12, 8)), None),
                // An object it takes, then another of its events: nothing of a walk.
                event(AdventureEvent::ObjectRemoved {
                    object_type: pile,
                    anchor: pos(13, 12),
                }),
                event(AdventureEvent::MovementChanged {
                    hero: blue,
                    movement: 0,
                    max_movement: 1500,
                }),
            ]),
            // E24 M4 T6 (R1-3, R1-5): Blue's next command, the same hero walking on: a move of
            // its own; the monster it beats off the map, the sawmill flagged, Red's hero lost;
            // then a command with no step taking an object (R1-6: off the map as the next turn
            // table after this turn starts, not as this one's).
            events(vec![
                step(blue, Some(pos(12, 8)), Some(pos(12, 7))),
                event(AdventureEvent::ObjectRemoved {
                    object_type: monster,
                    anchor: pos(10, 13),
                }),
                event(AdventureEvent::ObjectFlagged {
                    object_type: 0,
                    anchor: pos(6, 10),
                    owner: 1,
                }),
                event(AdventureEvent::HeroRemoved {
                    hero: red,
                    pos: pos(10, 9),
                }),
            ]),
            events(vec![event(AdventureEvent::ObjectRemoved {
                object_type: pile,
                anchor: pos(14, 10),
            })]),
            events(vec![
                GameEvent::TurnEnded { player: 1 },
                GameEvent::TurnStarted { player: 2 },
                // Purple's: an object taken with no step (E24 M4 T4c2, MCL-Q46: off the map as
                // the next turn table starts), one off the map's positions (nothing) and a
                // landing with no walk (nothing).
                event(AdventureEvent::ObjectRemoved {
                    object_type: pile,
                    anchor: pos(13, 12),
                }),
                event(AdventureEvent::ObjectRemoved {
                    object_type: 79,
                    anchor: pos(300, 12),
                }),
                event(AdventureEvent::ObjectFlagged {
                    object_type: 53,
                    anchor: pos(300, 12),
                    owner: 2,
                }),
                event(AdventureEvent::Landed {
                    hero: blue,
                    boat: 1,
                    at: pos(12, 8),
                    facing: 0,
                }),
            ]),
            events(vec![
                GameEvent::TurnEnded { player: 2 },
                GameEvent::DayStarted { day: 2 },
                GameEvent::TurnStarted { player: 0 },
            ]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    assert_eq!(played.commands, [Command::Other, Command::EndTurn]);
    let [other] = &played.others[..] else {
        panic!("one other turn: {:?}", played.others);
    };
    let tile = |x, y| h3_map::model::Pos { x, y, z: 0 };
    assert_eq!((other.player, other.turn, other.walks.len()), (1, 0, 3));
    let seen = |from, to: Option<u8>, sea| OtherStep {
        from: Some(tile(12, from)),
        to: to.map(|y| tile(12, y)),
        sea,
    };
    assert_eq!(other.walks[0].hero, blue);
    assert_eq!(
        other.walks[0].steps,
        [
            seen(10, Some(9), Some(Sea::Board(1))),
            seen(
                9,
                Some(8),
                Some(Sea::Land(LiveBoat {
                    at: tile(12, 8),
                    kind: 1,
                    facing: Facing::ALL[0],
                }))
            ),
        ]
    );
    assert_eq!(other.walks[1].hero, red);
    assert_eq!(
        other.walks[2].steps,
        [seen(8, None, None), seen(8, Some(7), None)]
    );
    // Its two moves (R1-3): the pile after the first, the second's changes after it (R1-5).
    assert_eq!(
        other.walks[2].ends,
        [
            OtherEnd {
                after: 1,
                taken: vec![tile(13, 12)],
                ..OtherEnd::default()
            },
            OtherEnd {
                after: 2,
                removed: vec![tile(10, 13)],
                flagged: vec![(tile(6, 10), 1)],
                lost: vec![red],
                ..OtherEnd::default()
            },
        ]
    );
    assert_eq!(
        other.walks[0].ends,
        [OtherEnd {
            after: 2,
            ..OtherEnd::default()
        }]
    );
    assert!(!other.open && !played.waits);
    // Blue's turn's table, then Red's day 2: kind 5 for Blue, its two walking heroes, no points.
    assert_eq!(played.turns.len(), 2);
    let t = table(&played.turns[0]);
    assert_eq!(t["player"].as_integer(), Some(1));
    let status = table(&t["status"]);
    assert_eq!(
        ["kind", "crest", "sand", "glass", "value"].map(|k| status[k].as_integer()),
        [Some(5), Some(1), Some(0), Some(0), Some(1000)]
    );
    let kingdom = table(&t["kingdom"]);
    assert_eq!(kingdom["heroes"].as_array().unwrap().len(), 2);
    assert_eq!(numbers(&kingdom["selected"]), [1, 0]);
    assert_eq!(table(&played.turns[1])["player"].as_integer(), Some(0));
    // Blue's and Purple's pickups without a step: given to Red's day 2's table, the table after
    // Blue's own (R1-6).
    let [(1, between)] = &played.between[..] else {
        panic!("one table's changes: {:?}", played.between);
    };
    assert_eq!(between.taken, [tile(14, 10), tile(13, 12)]);
    // Red's list at each table (R1-4): its hero lost in Blue's turn, gone from both.
    assert_eq!(played.lists, [Vec::<u16>::new(), Vec::new()]);
}

#[test]
fn what_a_record_s_visits_tell() {
    use h3_map_view::asks::Ask;
    use h3_map_view::engine::visits::{self, Answered, Told};
    use h3_map_view::finds::{Find, Gain};
    use h3_protocol::wire::game::{
        AdventureEvent, Answer, ArenaChoice, ChestChoice, Consent, GameEvent, HeroChange,
        NothingWhy, Pos, SchoolChoice,
    };
    let anchor = Pos {
        x: 5,
        y: 4,
        level: 0,
    };
    let adventure = |event| GameEvent::Adventure { event };
    let visited = adventure(AdventureEvent::NothingGained {
        hero: 0,
        object_type: 1,
        anchor,
        why: NothingWhy::Visited,
    });
    let full = adventure(AdventureEvent::NothingGained {
        hero: 0,
        object_type: 1,
        anchor,
        why: NothingWhy::Full,
    });
    let removed = adventure(AdventureEvent::ObjectRemoved {
        object_type: 1,
        anchor,
    });
    let emptied = adventure(AdventureEvent::ObjectEmpty {
        object_type: 1,
        anchor,
    });
    let mark = |held| {
        adventure(AdventureEvent::OnceMarkChanged {
            hero: 0,
            bonus: VisitBonus {
                mark: 8,
                morale: 1,
                luck: 2,
            },
            held,
        })
    };
    let changed = |change| GameEvent::HeroChanged {
        player: 0,
        hero: 0,
        change,
    };
    let primary = |primary| changed(HeroChange::GainPrimary { primary });
    let experience = changed(HeroChange::GainExperience { amount: 1000 });
    let resource = |resource, amount| GameEvent::ResourceGained {
        player: 0,
        resource,
        amount,
    };
    let artifact = changed(HeroChange::AddArtifact {
        artifact: h3_protocol::wire::game::Artifact {
            artifact: 108,
            spell: None,
        },
    });
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let visit = |events: Vec<GameEvent>| engine::Visit {
        at,
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        events,
        answers: Vec::new(),
        revealed: Vec::new(),
        battle: None,
        flag: None,
    };
    let found = |class, events| visits::told(&visit(events), [class, 0], "Rion");
    let find = |told: Option<Told>| match told {
        Some(Told::Find {
            find,
            gain,
            taken,
            flag: None,
        }) => Some((find, gain, taken)),
        _ => None,
    };
    let none = Gain::default();
    // A Temple: its mark's morale (the luck too, on any day), else visited; nothing else tells.
    let morale = Gain {
        morale: [1, 1],
        luck: 2,
        ..Gain::default()
    };
    assert_eq!(
        find(found(96, vec![mark(true)])),
        Some((Find::Temple { visited: false }, morale, false))
    );
    assert_eq!(
        find(found(96, vec![visited.clone()])),
        Some((Find::Temple { visited: true }, none, false))
    );
    assert_eq!(found(96, vec![mark(false), full]), None);
    // A Star Axis and a Learning Stone.
    let power = Gain {
        skills: [0, 0, 1, 0],
        ..Gain::default()
    };
    assert_eq!(
        find(found(61, vec![primary([0, 0, 1, 0])])),
        Some((Find::Axis { visited: false }, power, false))
    );
    assert_eq!(
        find(found(61, vec![visited.clone()])).map(|f| f.0),
        Some(Find::Axis { visited: true })
    );
    assert_eq!(found(61, Vec::new()), None);
    assert_eq!(
        find(found(100, vec![experience.clone()])).map(|f| (f.0, f.1.experience)),
        Some((Find::Stone { visited: false }, 1000))
    );
    assert_eq!(
        find(found(100, vec![visited.clone()])).map(|f| f.0),
        Some(Find::Stone { visited: true })
    );
    assert_eq!(found(100, Vec::new()), None);
    // A pile taken, a Windmill's resource or its emptiness; none for several or none.
    assert_eq!(
        find(found(79, vec![removed.clone(), resource(6, 1000)])).map(|f| (f.0, f.2)),
        Some((
            Find::Pile {
                resource: 6,
                amount: 1000
            },
            true
        ))
    );
    assert_eq!(
        find(found(112, vec![resource(1, 3)])).map(|f| f.0),
        Some(Find::Windmill {
            resource: 1,
            amount: 3
        })
    );
    assert_eq!(
        find(found(112, vec![emptied.clone()])).map(|f| f.0),
        Some(Find::Windmill {
            resource: 0,
            amount: 0
        })
    );
    assert_eq!(found(112, Vec::new()), None);
    assert_eq!(found(79, vec![emptied]), None);
    assert_eq!(found(79, vec![resource(1, 3), resource(2, 3)]), None);
    // An artifact added and taken; without it taken alone (E24 M4 T6); another class tells
    // nothing.
    assert_eq!(
        find(found(5, vec![removed.clone(), artifact.clone()])).map(|f| (f.0, f.2)),
        Some((Find::Artifact { artifact: 108 }, true))
    );
    assert_eq!(found(5, vec![removed.clone()]), Some(Told::Taken));
    // A random artifact's (65 to 69: the view draws its random template, MCL-STUB-1) as an
    // artifact's, by the record's `AddArtifact` (E24 M5 T4c, T4's game: no box before).
    for class in 65..=69 {
        assert_eq!(
            find(found(class, vec![removed.clone(), artifact.clone()])).map(|f| (f.0, f.2)),
            Some((Find::Artifact { artifact: 108 }, true))
        );
    }
    assert_eq!(found(69, vec![removed.clone()]), Some(Told::Taken));
    assert_eq!(found(70, vec![removed, artifact]), Some(Told::Taken));
    assert_eq!(found(91, vec![visited]), None);
    // The questions: no answer, none; the answer another class's, none.
    let asked = |class, answer, after: Vec<GameEvent>| {
        let mut v = visit(Vec::new());
        v.answers.push((answer, after));
        match visits::told(&v, [class, 0], "Rion") {
            Some(Told::Ask {
                ask,
                answered,
                flag: None,
            }) => Some((ask, answered)),
            _ => None,
        }
    };
    assert_eq!(found(4, Vec::new()), None);
    let attack = Answer::Arena {
        choice: ArenaChoice::Attack,
    };
    assert_eq!(asked(101, attack, Vec::new()), None);
    assert_eq!(
        asked(4, attack, vec![primary([2, 0, 0, 0])]),
        Some((Ask::Arena(2), Answered::Yes))
    );
    assert_eq!(
        asked(
            4,
            Answer::Arena {
                choice: ArenaChoice::Defense
            },
            vec![primary([0, 2, 0, 0])]
        ),
        Some((Ask::Arena(2), Answered::Second))
    );
    // Pandora's Box: its answer's gains its rewards; declined, none.
    let accept = Answer::Offer {
        consent: Consent::Accept,
    };
    let Some((Ask::OpenBox { rewards, hero }, Answered::Yes)) = asked(
        6,
        accept,
        vec![resource(6, 1500), experience.clone(), primary([1, 0, 0, 0])],
    ) else {
        panic!("Pandora's Box accepted");
    };
    assert_eq!(
        (rewards.resources[6], rewards.experience, rewards.primary),
        (1500, 1000, [1, 0, 0, 0])
    );
    assert_eq!(hero, "Rion");
    assert!(matches!(
        asked(
            6,
            Answer::Offer {
                consent: Consent::Decline
            },
            Vec::new()
        ),
        Some((Ask::OpenBox { .. }, Answered::No))
    ));
    // A chest: the part taken the answer's gain, the other none (MCL-Q41).
    assert_eq!(
        asked(
            101,
            Answer::Chest {
                choice: ChestChoice::Gold
            },
            vec![resource(6, 1500)]
        ),
        Some((
            Ask::Chest {
                gold: 1500,
                experience: 0
            },
            Answered::Yes
        ))
    );
    assert_eq!(
        asked(
            101,
            Answer::Chest {
                choice: ChestChoice::Experience
            },
            vec![experience]
        ),
        Some((
            Ask::Chest {
                gold: 0,
                experience: 1000
            },
            Answered::Second
        ))
    );
    // A School of Magic (E24 M4 T6, R2-2): a skill bought, its gain and price the record's; left,
    // or no price told, nothing.
    let school = |choice| Answer::School { choice };
    let paid = GameEvent::ResourcePaid {
        player: 0,
        resource: 6,
        amount: 1000,
    };
    let bought = Ask::School {
        gain: 1,
        price: [6, 1000],
    };
    assert_eq!(
        asked(
            47,
            school(SchoolChoice::First),
            vec![paid.clone(), primary([0, 0, 1, 0])]
        ),
        Some((bought.clone(), Answered::Yes))
    );
    assert_eq!(
        asked(
            47,
            school(SchoolChoice::Second),
            vec![paid, primary([0, 0, 0, 1])]
        ),
        Some((bought, Answered::Second))
    );
    assert_eq!(asked(47, school(SchoolChoice::Leave), Vec::new()), None);
    assert_eq!(
        asked(47, school(SchoolChoice::First), vec![primary([0, 0, 1, 0])]),
        None
    );
    // What the panels show: an unknown resource, another change and another event left out.
    let gain = visits::gain(&[
        resource(9, 5),
        resource(0, 5),
        changed(HeroChange::SetSpellPoints { points: 3 }),
        GameEvent::DayStarted { day: 2 },
    ]);
    assert_eq!(gain.resources, [5, 0, 0, 0, 0, 0, 0]);
    assert_eq!(gain.skills, [0; 4]);
}

/// What the record tells play (E24 M5 T4c, design §5.29, the facts audit): the resources paid
/// and lost taken off a gain, the hero's army's slots as `StackChanged` tells them, the object an
/// answer takes off the map by its anchor, an observatory's visit a find.
#[test]
fn what_the_record_tells_play() {
    use h3_map_view::engine::visits::{self, Told};
    use h3_map_view::finds::{Find, Gain};
    use h3_map_view::live::Stack;
    use h3_protocol::wire::game::{
        AdventureEvent, ArmyRef, ArmySlot, ArmyStack, GameEvent, Pos as WirePos,
    };
    let gained = GameEvent::ResourceGained {
        player: 0,
        resource: 6,
        amount: 1000,
    };
    let paid = |resource| GameEvent::ResourcePaid {
        player: 0,
        resource,
        amount: 10,
    };
    let lost = GameEvent::ResourceLost {
        player: 0,
        resource: 0,
        amount: 3,
    };
    let slot = |army, slot, stack| GameEvent::StackChanged {
        player: 0,
        at: ArmySlot { army, slot },
        stack,
    };
    let hero = ArmyRef::Hero { hero: 0 };
    let imps = |slot| {
        Some(ArmyStack {
            slot,
            creature: 42,
            count: 25,
        })
    };
    // The Tree's 10 gems paid (the bug the audit found: the bar kept them), a loss, an unknown
    // resource's price left out.
    let gain = visits::gain(&[gained, paid(5), paid(9), lost]);
    assert_eq!(gain.resources, [-3, 0, 0, 0, 0, -10, 1000]);
    assert!(!gain.slotted);
    // The slots the record names, the hero's alone; an emptied slot or one past seven not told.
    let gain = visits::gain(&[
        slot(hero, 3, imps(3)),
        slot(hero, 4, None),
        slot(hero, 9, imps(9)),
        slot(ArmyRef::Garrison { town: 0 }, 1, imps(1)),
    ]);
    let mut army = [None; 7];
    army[3] = Some(Stack {
        creature: 42,
        count: 25,
    });
    assert_eq!((gain.army, gain.slotted), (army, true));
    assert!(!visits::gain(&[slot(hero, 9, imps(9))]).slotted);
    // The object taken by its anchor; one past a byte none.
    let removed = |x, y| GameEvent::Adventure {
        event: AdventureEvent::ObjectRemoved {
            object_type: 1,
            anchor: WirePos { x, y, level: 1 },
        },
    };
    assert_eq!(
        visits::removed(&[GameEvent::DayStarted { day: 2 }, removed(7, 8)]),
        Some(h3_map::model::Pos { x: 7, y: 8, z: 1 })
    );
    assert_eq!(visits::removed(&[removed(300, 8)]), None);
    assert_eq!(visits::removed(&[removed(7, 300)]), None);
    assert!(visits::removes(&[removed(7, 8)]));
    // The Redwood Observatory's and the Pillar of Fire's visit a find by its `TilesRevealed`;
    // without one, nothing.
    let visit = |events| engine::Visit {
        at: h3_map::model::Pos { x: 5, y: 4, z: 0 },
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        events,
        answers: Vec::new(),
        revealed: Vec::new(),
        battle: None,
        flag: None,
    };
    let revealed = GameEvent::Adventure {
        event: AdventureEvent::TilesRevealed {
            object_type: 3,
            count: 0,
        },
    };
    for (class, fire) in [(58, false), (60, true)] {
        assert_eq!(
            visits::told(&visit(vec![revealed.clone()]), [class, 0], ""),
            Some(Told::Find {
                find: Find::Observatory { fire },
                gain: Gain::default(),
                taken: false,
                flag: None,
            })
        );
        assert_eq!(visits::told(&visit(Vec::new()), [class, 0], ""), None);
    }
}

/// Pandora's Box as a record tells it (E24 M4 T6, the reviewer's R2-4): every part its answers'
/// events tell (the experience given after the army dialog's close), the spell points and the
/// battle bonuses against what the hero held, brought on by the walk's own events; its guards
/// fought, nothing (not built).
#[test]
fn what_a_record_s_box_tells() {
    use h3_map::model::{Rewards, Stack};
    use h3_map_view::asks::Ask;
    use h3_map_view::engine::visits::{self, Answered, Told};
    use h3_protocol::wire::game::{
        AdventureEvent, Answer, ArmyStack, Artifact, Consent, GameEvent, HeroChange,
    };
    let changed = |change| GameEvent::HeroChanged {
        player: 0,
        hero: 0,
        change,
    };
    let bonus = |morale, luck| GameEvent::Adventure {
        event: AdventureEvent::BonusChanged {
            hero: 0,
            morale,
            luck,
        },
    };
    let opened = vec![
        GameEvent::ResourceGained {
            player: 0,
            resource: 6,
            amount: 1000,
        },
        GameEvent::ResourceLost {
            player: 0,
            resource: 0,
            amount: 5,
        },
        GameEvent::ResourceLost {
            player: 0,
            resource: 9,
            amount: 5,
        },
        changed(HeroChange::GainPrimary {
            primary: [0, 1, 0, 0],
        }),
        changed(HeroChange::RaiseSkill { skill: 6, level: 2 }),
        changed(HeroChange::RaiseSkill {
            skill: 300,
            level: 1,
        }),
        changed(HeroChange::SetSpellPoints { points: 25 }),
        changed(HeroChange::AddArtifact {
            artifact: Artifact {
                artifact: 7,
                spell: None,
            },
        }),
        changed(HeroChange::LearnSpell { spell: 15 }),
        changed(HeroChange::LearnSpell { spell: 300 }),
        GameEvent::Adventure {
            event: AdventureEvent::CreaturesJoined {
                hero: 0,
                creature: 18,
                count: 10,
            },
        },
        bonus(3, -1),
        GameEvent::ArmyDialogOpened {
            player: 0,
            hero: 0,
            stack: ArmyStack {
                slot: 0,
                creature: 20,
                count: 4,
            },
        },
        GameEvent::DayStarted { day: 2 },
    ];
    let visit = engine::Visit {
        at: h3_map::model::Pos { x: 5, y: 4, z: 0 },
        walk: 3,
        hero: 0,
        hero_type: Some(1),
        held: engine::Held {
            mana: 10,
            morale: 1,
            luck: 0,
        },
        levels: Vec::new(),
        // The walk's own events bring the morale on: what the Box changes is told against it.
        events: vec![bonus(2, 0)],
        answers: vec![
            (
                Answer::Offer {
                    consent: Consent::Accept,
                },
                opened,
            ),
            // The army dialog's close: the experience, given last.
            (
                Answer::Close {},
                vec![changed(HeroChange::GainExperience { amount: 2000 })],
            ),
        ],
        revealed: Vec::new(),
        battle: None,
        flag: None,
    };
    let Some(Told::Ask {
        ask: Ask::OpenBox { rewards, hero },
        answered: Answered::Yes,
        flag: None,
    }) = visits::told(&visit, [6, 0], "Rion")
    else {
        panic!("Pandora's Box opened");
    };
    assert_eq!(
        rewards,
        Rewards {
            experience: 2000,
            mana: 15,
            morale: 1,
            luck: -1,
            resources: [-5, 0, 0, 0, 0, 0, 1000],
            primary: [0, 1, 0, 0],
            skills: vec![(6, 2)],
            artifacts: vec![7],
            spells: vec![15],
            creatures: vec![
                Stack {
                    creature: 18,
                    count: 10
                },
                Stack {
                    creature: 20,
                    count: 4
                }
            ],
        }
    );
    assert_eq!(hero, "Rion");
    // Its guards fought: nothing (ADVEVENT 16, the battle, then its rewards: not built).
    let mut guarded = visit;
    guarded.battle = Some(engine::Battled {
        started: 2,
        ended: Vec::new(),
        tile: None,
    });
    assert_eq!(visits::told(&guarded, [6, 0], "Rion"), None);
}

/// A record's visits laid out (E24 M4 T6, the reviewers' R2-1, R2-6 and R2-8): one find a visit,
/// three of them on one Windmill, Pandora's Box declined then opened, each question's answers
/// kept (the record's, the Arena's Cancel with its Attack, a guarded mine's OK), the objects that
/// block their tiles, an artifact's primary skills, the level-ups; a visit on no object, or one
/// that tells nothing, left out.
#[test]
fn a_record_s_visits_laid_out_each_on_its_own() {
    use h3_map_view::engine::visits::{self, Answered, Kept, Laid, Told};
    use h3_map_view::finds::Find;
    use h3_protocol::wire::game::{
        AdventureEvent, Answer, ArenaChoice, Artifact, Consent, GameEvent, HeroChange, Pos,
    };
    use h3_protocol::wire::{Outcome, Side};
    let anchor = Pos {
        x: 5,
        y: 4,
        level: 0,
    };
    let wood = GameEvent::ResourceGained {
        player: 0,
        resource: 0,
        amount: 3,
    };
    let emptied = GameEvent::Adventure {
        event: AdventureEvent::ObjectEmpty {
            object_type: 1,
            anchor,
        },
    };
    let removed = GameEvent::Adventure {
        event: AdventureEvent::ObjectRemoved {
            object_type: 1,
            anchor,
        },
    };
    let primary = |primary| GameEvent::HeroChanged {
        player: 0,
        hero: 0,
        change: HeroChange::GainPrimary { primary },
    };
    let offer = |consent| Answer::Offer { consent };
    let won = GameEvent::BattleEnded {
        heroes: [Some(0), None],
        result: Outcome::Victory {
            side: Side::Attacker,
        },
    };
    let tile = |x| h3_map::model::Pos { x, y: 4, z: 0 };
    let visit = |walk: usize, x, events: Vec<GameEvent>, answers: Vec<(Answer, Vec<GameEvent>)>| {
        engine::Visit {
            at: tile(x),
            walk,
            hero: 0,
            hero_type: Some(1),
            held: engine::Held::default(),
            levels: Vec::new(),
            events,
            answers,
            revealed: Vec::new(),
            battle: None,
            flag: None,
        }
    };
    let mut visits = vec![
        // A Windmill's three visits: its wood, its wood a week later, empty the same week.
        visit(0, 5, vec![wood.clone()], Vec::new()),
        visit(1, 5, vec![wood.clone()], Vec::new()),
        visit(2, 5, vec![emptied], Vec::new()),
        // Pandora's Box declined, then opened.
        visit(
            3,
            6,
            Vec::new(),
            vec![(offer(Consent::Decline), Vec::new())],
        ),
        visit(
            5,
            6,
            Vec::new(),
            vec![(offer(Consent::Accept), vec![removed.clone()])],
        ),
        // The Arena: Attack (its Cancel kept, MCL-Q33), Defense.
        visit(
            6,
            7,
            Vec::new(),
            vec![(
                Answer::Arena {
                    choice: ArenaChoice::Attack,
                },
                vec![primary([2, 0, 0, 0])],
            )],
        ),
        visit(
            7,
            7,
            Vec::new(),
            vec![(
                Answer::Arena {
                    choice: ArenaChoice::Defense,
                },
                vec![primary([0, 2, 0, 0])],
            )],
        ),
        // A guarded mine's battle, a monster's.
        visit(8, 8, Vec::new(), vec![(offer(Consent::Accept), Vec::new())]),
        visit(9, 9, Vec::new(), Vec::new()),
        // An artifact picked up (R2-6: its primary skills, the tables').
        visit(
            10,
            10,
            vec![
                removed,
                GameEvent::HeroChanged {
                    player: 0,
                    hero: 0,
                    change: HeroChange::AddArtifact {
                        artifact: Artifact {
                            artifact: 7,
                            spell: None,
                        },
                    },
                },
            ],
            Vec::new(),
        ),
        // A Temple that tells nothing; a tile with no object.
        visit(11, 11, Vec::new(), Vec::new()),
        visit(12, 12, vec![wood], Vec::new()),
    ];
    for k in [7, 8] {
        visits[k].battle = Some(engine::Battled {
            started: 4,
            ended: vec![won.clone()],
            tile: None,
        });
    }
    visits[1].levels = vec![0, 1];
    let object_at = |v: &engine::Visit| match v.at.x {
        5 => Some(placed(v, 1, [112, 0], false)),
        6 => Some(placed(v, 2, [6, 0], false)),
        7 => Some(placed(v, 3, [4, 0], false)),
        8 => Some(placed(v, 4, [53, 0], false)),
        9 => Some(placed(v, 5, [54, 139], false)),
        10 => Some(placed(v, 6, [5, 0], false)),
        11 => Some(placed(v, 7, [96, 0], false)),
        _ => None,
    };
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml"),
    )
    .unwrap();
    let tables = tables::load(&[tables::parse(&text).unwrap()], "base").unwrap();
    let laid = visits::lay(&visits, None, object_at, &tables.map.finds, |hero| {
        format!("hero {hero:?}")
    });
    let walks: Vec<usize> = laid.iter().map(|l| l.walk).collect();
    assert_eq!(walks, [0, 1, 2, 3, 5, 6, 7, 8, 9, 10]);
    // The Windmill: each visit its own find, its own box and gain.
    let windmill: Vec<(Find, i32)> = laid[..3]
        .iter()
        .map(|l| match &l.told {
            Told::Find { find, gain, .. } => (find.clone(), gain.resources[0]),
            other => panic!("a find: {other:?}"),
        })
        .collect();
    let mill = |amount| Find::Windmill {
        resource: 0,
        amount,
    };
    assert_eq!(windmill, [(mill(3), 3), (mill(3), 3), (mill(0), 0)]);
    assert!(laid[..3].iter().all(|l| (l.object, l.at) == (1, tile(5))));
    assert_eq!(
        laid.iter().map(|l| l.levels.len()).collect::<Vec<_>>(),
        [0, 2, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    // The answers each question keeps, and the objects that block their tiles.
    let kept = |yes, second, no| Kept { yes, second, no };
    let shape: Vec<(Kept, bool)> = laid.iter().map(|l| (l.kept, l.blocks)).collect();
    assert_eq!(
        shape,
        [
            (Kept::default(), false),
            (Kept::default(), false),
            (Kept::default(), false),
            (kept(false, false, true), true),
            (kept(true, false, false), true),
            (kept(true, false, true), false),
            (kept(false, true, false), false),
            (kept(true, false, false), false),
            (Kept::default(), true),
            (Kept::default(), true),
        ]
    );
    let answered: Vec<Option<Answered>> = laid
        .iter()
        .map(|l| match l.told {
            Told::Ask { answered, .. } => Some(answered),
            _ => None,
        })
        .collect();
    assert_eq!(
        answered[3..7],
        [
            Some(Answered::No),
            Some(Answered::Yes),
            Some(Answered::Yes),
            Some(Answered::Second)
        ]
    );
    assert!(matches!(laid[7].told, Told::Guarded { .. }));
    assert!(matches!(laid[8].told, Told::Fight { .. }));
    // The Box's boxes name the visit's hero.
    assert!(matches!(
        &laid[4].told,
        Told::Ask {
            ask: h3_map_view::asks::Ask::OpenBox { hero, .. },
            ..
        } if hero == "hero Some(1)"
    ));
    // The artifact's primary skills on the window (the tables' Centaur's Axe: +2 Attack).
    let Laid {
        told: Told::Find { gain, taken, .. },
        ..
    } = &laid[9]
    else {
        panic!("an artifact's find");
    };
    assert_eq!((gain.skills, *taken), ([2, 0, 0, 0], true));
}

#[test]
fn a_record_s_object_taken_alone() {
    // E24 M4 T6: a visit that takes its object off the map and tells nothing else Part A builds
    // (a Scholar's, class 81) takes it alone, from beside, no box; an event's removal (class 26,
    // not drawn) tells nothing.
    use h3_map_view::engine::visits::{self, Kept, Laid, Told};
    use h3_protocol::wire::game::{AdventureEvent, GameEvent, Pos};
    let removed = GameEvent::Adventure {
        event: AdventureEvent::ObjectRemoved {
            object_type: 63,
            anchor: Pos {
                x: 5,
                y: 4,
                level: 0,
            },
        },
    };
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let visit = engine::Visit {
        at,
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        flag: None,
        events: vec![removed],
        answers: Vec::new(),
        revealed: Vec::new(),
        battle: None,
    };
    assert_eq!(visits::told(&visit, [81, 0], "Rion"), Some(Told::Taken));
    assert_eq!(visits::told(&visit, [26, 0], "Rion"), None);
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml"),
    )
    .unwrap();
    let tables = tables::load(&[tables::parse(&text).unwrap()], "base").unwrap();
    let laid = visits::lay(
        &[visit],
        None,
        |v| Some(placed(v, 3, [81, 0], false)),
        &tables.map.finds,
        |_| String::new(),
    );
    let [
        Laid {
            told: Told::Taken,
            object: 3,
            kept,
            blocks: true,
            ..
        },
    ] = &laid[..]
    else {
        panic!("taken alone: {laid:?}");
    };
    assert_eq!(*kept, Kept::default());
}

#[test]
fn what_a_record_s_flags_tell() {
    use h3_map_view::asks::Ask;
    use h3_map_view::engine::visits::{self, Answered, Told};
    use h3_map_view::finds::Find;
    use h3_map_view::live::Stack;
    use h3_protocol::wire::game::{AdventureEvent, Answer, Consent, GameEvent, Pos};
    let anchor = Pos {
        x: 5,
        y: 4,
        level: 0,
    };
    let flagged = GameEvent::Adventure {
        event: AdventureEvent::ObjectFlagged {
            object_type: 77,
            anchor,
            owner: 2,
        },
    };
    let joined = GameEvent::Adventure {
        event: AdventureEvent::CreaturesJoined {
            hero: 0,
            creature: 0,
            count: 14,
        },
    };
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let visit = |events: Vec<GameEvent>, answers: Vec<(Answer, Vec<GameEvent>)>| engine::Visit {
        at,
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        flag: events.contains(&flagged).then_some(2),
        events,
        answers,
        revealed: Vec::new(),
        battle: None,
    };
    let told = |v: &engine::Visit, class, subclass| visits::told(v, [class, subclass], "Rion");
    let flag = |told: Option<Told>| match told {
        Some(Told::Find {
            find, taken, flag, ..
        }) => Some((find, taken, flag)),
        _ => None,
    };
    // A mine of its type's resource and a Lighthouse flagged: their finds in the record's
    // colour (the visit's, as `played` gives it: 2 here); unflagged (another's own visit):
    // nothing; an abandoned mine's type (E24 M4 T6, the reviewers' R3-4): its flag alone.
    let flags = visit(vec![flagged.clone()], Vec::new());
    assert_eq!(
        flag(told(&flags, 53, 0)),
        Some((Find::Mine { resource: 0 }, false, Some(2)))
    );
    assert_eq!(
        flag(told(&flags, 42, 0)),
        Some((Find::Lighthouse, false, Some(2)))
    );
    assert_eq!(told(&visit(Vec::new(), Vec::new()), 53, 0), None);
    assert_eq!(told(&flags, 53, 7), Some(Told::Flagged { flag: 2 }));
    // A dwelling flagged, its hire accepted: the creatures joined free; one of the visitor's own
    // not flagged; declined, or none joined: nothing.
    let accept = Answer::Offer {
        consent: Consent::Accept,
    };
    let hire = |events, after| {
        let v = visit(events, vec![(accept, after)]);
        match told(&v, 17, 0) {
            Some(Told::Ask {
                ask,
                answered,
                flag,
            }) => Some((ask, answered, flag)),
            _ => None,
        }
    };
    let joins = Stack {
        creature: 0,
        count: 14,
    };
    let hired = |flag| Ask::Hire {
        dwelling: 0,
        joins,
        free: true,
        room: true,
        flag,
    };
    assert_eq!(
        hire(vec![flagged.clone()], vec![joined.clone()]),
        Some((hired(true), Answered::Yes, Some(2)))
    );
    assert_eq!(
        hire(Vec::new(), vec![joined.clone()]),
        Some((hired(false), Answered::Yes, None))
    );
    // Refused for want of room, none to join: their boxes' facts.
    let refused = GameEvent::Adventure {
        event: AdventureEvent::JoinRefused {
            hero: 0,
            creature: 0,
            count: 14,
        },
    };
    let none = GameEvent::Adventure {
        event: AdventureEvent::NoneToRecruit {
            object_type: 77,
            anchor,
            creature: 0,
        },
    };
    let Some((Ask::Hire { joins, room, .. }, ..)) = hire(Vec::new(), vec![refused]) else {
        panic!("refused");
    };
    assert_eq!((joins.count, room), (14, false));
    let Some((Ask::Hire { joins, room, .. }, ..)) = hire(Vec::new(), vec![none]) else {
        panic!("none");
    };
    assert_eq!((joins.count, room), (0, true));
    let other = GameEvent::DayStarted { day: 2 };
    assert_eq!(
        hire(vec![flagged.clone()], vec![flagged.clone(), other]),
        None
    );
    assert_eq!(hire(vec![flagged.clone()], Vec::new()), None);
    // Its hire not built, the flag raised alone (E24 M4 T6, the reviewers' R3-4): none told
    // after an accept, declined, a class-20 dwelling's; unflagged, nothing.
    let alone = Some(Told::Flagged { flag: 2 });
    let untold = visit(vec![flagged.clone()], vec![(accept, Vec::new())]);
    assert_eq!(told(&untold, 17, 0), alone);
    let declined = visit(
        vec![flagged.clone()],
        vec![(
            Answer::Offer {
                consent: Consent::Decline,
            },
            Vec::new(),
        )],
    );
    assert_eq!(told(&declined, 20, 0), alone);
    assert_eq!(told(&declined, 17, 0), alone);
    // A Shipyard's (87) flag likewise.
    assert_eq!(told(&flags, 87, 0), alone);
    assert_eq!(told(&visit(Vec::new(), Vec::new()), 17, 0), None);
    // A class-20 dwelling's hire accepted (E24 M4 T6, the reviewer's R2-5): nothing (ADVEVENT 36
    // and its names not built), where class 17's of the same subclass asks its hire.
    let accepted = visit(Vec::new(), vec![(accept, vec![joined])]);
    assert!(matches!(
        told(&accepted, 17, 0),
        Some(Told::Ask {
            ask: Ask::Hire { .. },
            ..
        })
    ));
    assert_eq!(told(&accepted, 20, 0), None);
}

#[test]
fn a_record_s_flags_in_their_owners_colours() {
    // E24 M4 T6 (the reviewers' R3-1, R3-4): a visit's flag is its `ObjectFlagged`'s owner, a
    // player id, drawn in that player's colour (here Red, id 0, colour 5, and Blue, id 1, colour
    // 3): in its command's events (the first only), an answer's or its battle's end; none
    // without one. A mine taken, then taken back by the other: each visit its own flag.
    use h3_map_view::engine::visits::{self, Told};
    use h3_map_view::finds::Find;
    use h3_protocol::wire::game::{
        AdventureCommand, AdventureEvent, Answer, Consent, GameCommand, GameEvent, Pos,
    };
    use h3_protocol::wire::{Command as BattleCommand, Outcome, Side};
    let g = game();
    let mut observation = g.observation.clone();
    observation.players[0].colour = 5;
    observation.players[1].colour = 3;
    let red = observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let pos = |x, y| Pos { x, y, level: 0 };
    let walk = |x, y| {
        accepted(GameCommand::Adventure {
            command: AdventureCommand::MoveHero {
                hero: red,
                path: vec![pos(x, y)],
            },
        })
    };
    let flagged = |owner| GameEvent::Adventure {
        event: AdventureEvent::ObjectFlagged {
            object_type: 53,
            anchor: pos(5, 4),
            owner,
        },
    };
    let record = text(
        &g.header,
        &[
            snapshot(&observation),
            walk(5, 4),
            events(vec![flagged(0), flagged(1)]),
            walk(6, 4),
            events(Vec::new()),
            accepted(GameCommand::Answer {
                answer: Answer::Offer {
                    consent: Consent::Accept,
                },
            }),
            events(vec![flagged(1)]),
            walk(7, 4),
            events(vec![GameEvent::BattleStarted {
                heroes: [Some(red), None],
                tile: pos(7, 4),
            }]),
            accepted(GameCommand::Battle {
                command: BattleCommand::Defend {},
            }),
            events(vec![
                GameEvent::BattleEnded {
                    heroes: [Some(red), None],
                    result: Outcome::Victory {
                        side: Side::Attacker,
                    },
                },
                flagged(0),
            ]),
            walk(8, 4),
            events(Vec::new()),
            // The first walk's mine visited again: the flag the record raises then, Blue's.
            walk(5, 4),
            events(vec![flagged(1)]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let flags: Vec<Option<u8>> = played.visits.iter().map(|v| v.flag).collect();
    assert_eq!(flags, [Some(5), Some(3), Some(5), None, Some(3)]);
    // Laid out on a mine (the first and the last walk's) and a Shipyard: each visit's own flag,
    // a Shipyard's alone.
    let object_at = |v: &engine::Visit| match v.at.x {
        5 => Some(placed(v, 0, [53, 0], false)),
        6 => Some(placed(v, 1, [87, 0], false)),
        _ => None,
    };
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml"),
    )
    .unwrap();
    let tables = tables::load(&[tables::parse(&text).unwrap()], "base").unwrap();
    let laid = visits::lay(&played.visits, None, object_at, &tables.map.finds, |_| {
        String::new()
    });
    let told: Vec<(usize, Told)> = laid.into_iter().map(|l| (l.walk, l.told)).collect();
    let mine = |flag| Told::Find {
        find: Find::Mine { resource: 0 },
        gain: h3_map_view::finds::Gain::default(),
        taken: false,
        flag: Some(flag),
    };
    assert_eq!(
        told,
        [(0, mine(5)), (1, Told::Flagged { flag: 3 }), (4, mine(3))]
    );
}

#[test]
fn an_answer_s_reveal_is_kept_by_its_answer() {
    // E24 M5 T4c (design §5.29): the tiles an answer's `GameReveal` explores (a Cartographer's
    // map) kept by that answer, those explored before left out; a walk's reveal is its walk's.
    use h3_protocol::wire::game::{
        AdventureCommand, AdventureReveal, Answer, Consent, GameCommand, GameReveal, Pos,
        RevealedArea, RevealedView, Tile,
    };
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let pos = |x, y| Pos { x, y, level: 0 };
    let seen = RevealedView {
        explored: true,
        view: Tile::Seen {
            terrain: 2,
            road: None,
            river: None,
            blocked: false,
            visitable: false,
            no_native: false,
            winds: false,
            hole: false,
        },
    };
    // A row of three from `corner`: the first two explored now, the third's sight unchanged.
    let reveal = |corner| ServerMessage::GameReveal {
        n: 52,
        reveal: GameReveal {
            adventure: AdventureReveal {
                viewer: 0,
                palette: vec![seen],
                areas: vec![RevealedArea {
                    corner,
                    width: 3,
                    tiles: vec![1, 1, 0],
                }],
                heroes: Vec::new(),
                objects: Vec::new(),
                boats: Vec::new(),
                towns: Vec::new(),
            },
            heroes: Vec::new(),
            towns: Vec::new(),
            garrisons: Vec::new(),
        },
    };
    let answer = accepted(GameCommand::Answer {
        answer: Answer::Offer {
            consent: Consent::Accept,
        },
    });
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            accepted(GameCommand::Adventure {
                command: AdventureCommand::MoveHero {
                    hero: red,
                    path: vec![pos(11, 10)],
                },
            }),
            events(Vec::new()),
            reveal(pos(20, 2)),
            answer.clone(),
            events(Vec::new()),
            reveal(pos(30, 2)),
            // A second answer's: one tile new, the other explored by the first.
            answer,
            events(Vec::new()),
            reveal(pos(31, 2)),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let at = |x, y| h3_map::model::Pos { x, y, z: 0 };
    assert_eq!(played.visits.len(), 1);
    assert_eq!(
        played.visits[0].revealed,
        vec![vec![at(30, 2), at(31, 2)], vec![at(32, 2)]]
    );
    assert_eq!(played.moves[0].revealed[0], vec![at(20, 2), at(21, 2)]);
}

#[test]
fn a_record_s_boats() {
    use h3_map_view::live::{Facing, LiveBoat};
    use h3_map_view::timeline::Sea;
    use h3_protocol::wire::game::{
        AdventureCommand, AdventureEvent, AdventureReveal, Answer, Boat, Consent, GameCommand,
        GameEvent, GameReveal, Pos,
    };
    let g = game();
    let hero = |owner| {
        g.observation
            .adventure
            .heroes
            .iter()
            .find(|h| h.owner == owner)
            .unwrap()
            .id
    };
    let (red, blue) = (hero(0), hero(1));
    let pos = |x, y| Pos { x, y, level: 0 };
    let moved = |x, y| {
        accepted(GameCommand::Adventure {
            command: AdventureCommand::MoveHero {
                hero: red,
                path: vec![pos(x, y)],
            },
        })
    };
    let adventure = |event| GameEvent::Adventure { event };
    let stepped = |x, y| {
        adventure(AdventureEvent::HeroStepped {
            hero: red,
            from: None,
            to: Some(pos(x, y)),
        })
    };
    let landed = |hero, facing| {
        adventure(AdventureEvent::Landed {
            hero,
            boat: 1,
            at: pos(11, 10),
            facing,
        })
    };
    let seen = |at, facing| Boat {
        tile: pos(at, at),
        kind: 0,
        facing,
    };
    let built = |at, boat_type| {
        adventure(AdventureEvent::BoatBuilt {
            tile: pos(at, at),
            boat_type,
        })
    };
    // A boat off the map's positions: left out.
    let off = Boat {
        tile: Pos {
            x: 300,
            y: 1,
            level: 0,
        },
        kind: 0,
        facing: 0,
    };
    let reveal = |boats| ServerMessage::GameReveal {
        n: 52,
        reveal: GameReveal {
            adventure: AdventureReveal {
                viewer: 0,
                palette: Vec::new(),
                areas: Vec::new(),
                heroes: Vec::new(),
                objects: Vec::new(),
                boats,
                towns: Vec::new(),
            },
            heroes: Vec::new(),
            towns: Vec::new(),
            garrisons: Vec::new(),
        },
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            // Boarded; then landed, another hero's landing beside it; a landing with no facing.
            moved(11, 10),
            events(vec![
                stepped(11, 10),
                adventure(AdventureEvent::Boarded {
                    hero: red,
                    boat: 2,
                    at: pos(11, 10),
                }),
            ]),
            moved(12, 10),
            events(vec![stepped(12, 10), landed(red, 6), landed(blue, 2)]),
            moved(13, 10),
            events(vec![
                stepped(13, 10),
                landed(red, 9),
                // One off the map's positions: none, the landing before kept.
                adventure(AdventureEvent::Landed {
                    hero: red,
                    boat: 1,
                    at: Pos {
                        x: 300,
                        y: 10,
                        level: 0,
                    },
                    facing: 0,
                }),
            ]),
            // Another hero's boarding: no walk of its own open.
            events(vec![adventure(AdventureEvent::Boarded {
                hero: blue,
                boat: 0,
                at: pos(30, 30),
            })]),
            // E24 M4 T6 (the reviewer's R3-3): a walk's reveal's boats (one with no facing left
            // out), the walk's; a boat built after it, the next walk's; one built with no walk
            // after it, the next turn table's.
            moved(14, 10),
            events(vec![stepped(14, 10)]),
            reveal(vec![seen(20, 3), seen(25, 9), off]),
            accepted(GameCommand::Answer {
                answer: Answer::Offer {
                    consent: Consent::Accept,
                },
            }),
            events(vec![built(21, 1)]),
            reveal(vec![seen(26, 0)]),
            moved(15, 10),
            events(vec![stepped(15, 10), built(27, 0)]),
            accepted(GameCommand::EndTurn {}),
            events(vec![
                built(22, 2),
                GameEvent::TurnEnded { player: 0 },
                GameEvent::TurnStarted { player: 1 },
            ]),
            // Blue's: one built at its move's end, one with no step the next table's.
            events(vec![
                adventure(AdventureEvent::HeroStepped {
                    hero: blue,
                    from: Some(pos(30, 30)),
                    to: Some(pos(31, 30)),
                }),
                built(23, 0),
            ]),
            // E24 M4 T6: a hero the start does not show lands in sight: its boat the next
            // table's too (one with no facing left out).
            events(vec![
                built(24, 1),
                adventure(AdventureEvent::HeroStepped {
                    hero: 99,
                    from: None,
                    to: Some(pos(32, 32)),
                }),
                adventure(AdventureEvent::Landed {
                    hero: 99,
                    boat: 2,
                    at: pos(33, 33),
                    facing: 6,
                }),
                adventure(AdventureEvent::Landed {
                    hero: 99,
                    boat: 2,
                    at: pos(34, 34),
                    facing: 9,
                }),
            ]),
            events(vec![
                GameEvent::TurnEnded { player: 1 },
                GameEvent::DayStarted { day: 2 },
                GameEvent::TurnStarted { player: 0 },
            ]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let seas: Vec<_> = played.moves.iter().map(|m| m.sea).collect();
    let tile = |x, y| h3_map::model::Pos { x, y, z: 0 };
    let boat = LiveBoat {
        at: tile(11, 10),
        kind: 1,
        facing: Facing::Left,
    };
    // E24 M4 T6 (the reviewer's R3-6): a landing with a facing past 7 refused where it is played.
    assert_eq!(
        seas,
        [
            Some(Sea::Board(2)),
            Some(Sea::Land(boat)),
            Some(Sea::Unfaced(9)),
            None,
            None
        ]
    );
    let boats: Vec<Vec<LiveBoat>> = played.moves.iter().map(|m| m.boats.clone()).collect();
    let empty = |at, kind, facing| LiveBoat { at, kind, facing };
    assert_eq!(
        boats,
        [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![empty(tile(20, 20), 0, Facing::DownRight)],
            // A built boat faces as the fold builds it; one revealed with no walk open, and one
            // built during the walk, the walk's too.
            vec![
                empty(tile(21, 21), 1, Facing::Right),
                empty(tile(26, 26), 0, Facing::Up),
                empty(tile(27, 27), 0, Facing::Right),
            ],
        ]
    );
    let between: Vec<(usize, Vec<LiveBoat>)> = played
        .between
        .iter()
        .map(|(turn, end)| (*turn, end.boats.clone()))
        .collect();
    // The viewer's before Blue's table, Blue's stepless one before the viewer's next.
    assert_eq!(
        between,
        [
            (0, vec![empty(tile(22, 22), 2, Facing::Right)]),
            (
                1,
                vec![
                    empty(tile(24, 24), 1, Facing::Right),
                    empty(tile(33, 33), 2, Facing::Left)
                ]
            )
        ]
    );
    let ends: Vec<Vec<LiveBoat>> = played.others[0].walks[0]
        .ends
        .iter()
        .map(|e| e.boats.clone())
        .collect();
    assert_eq!(ends, [vec![empty(tile(23, 23), 0, Facing::Right)]]);
}

#[test]
fn a_record_s_battles() {
    // E24 M4 T5b1 (design §5.20): a walk whose events begin a battle names their message; a
    // battle begun with no walk of the viewer's open (another player's) names none; an answer's is
    // a command after it. Since E24 M4 T6 (the reviewer's R5-6) a battle's commands, the viewer's
    // and the system's for it, are none of the map's.
    use h3_map_view::engine::Command;
    use h3_protocol::wire::game::{
        AdventureCommand, AdventureEvent, Answer, Consent, GameCommand, GameEvent, Pos,
        SystemCommand,
    };
    let battle_command = || {
        accepted(GameCommand::Battle {
            command: h3_protocol::wire::Command::Defend {},
        })
    };
    let system_command = || {
        accepted(GameCommand::System {
            command: SystemCommand::Battle {
                player: 1,
                command: h3_protocol::wire::Command::Wait {},
            },
        })
    };
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let pos = |x, y| Pos { x, y, level: 0 };
    let moved = |x, y| {
        accepted(GameCommand::Adventure {
            command: AdventureCommand::MoveHero {
                hero: red,
                path: vec![pos(x, y)],
            },
        })
    };
    let stepped = |x, y| GameEvent::Adventure {
        event: AdventureEvent::HeroStepped {
            hero: red,
            from: None,
            to: Some(pos(x, y)),
        },
    };
    let started = |x, y| GameEvent::BattleStarted {
        heroes: [Some(red), None],
        tile: pos(x, y),
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            // A walk into a battle, its start told twice (the first kept); a walk without one.
            moved(12, 10),
            ServerMessage::GameEvents {
                n: 61,
                events: vec![stepped(11, 10), started(12, 10)],
            },
            events(vec![started(12, 10)]),
            battle_command(),
            system_command(),
            moved(11, 11),
            events(vec![stepped(11, 11)]),
            ServerMessage::GameTurn {
                n: 70,
                legal: Vec::new(),
                castable: Vec::new(),
            },
            // No walk open: none.
            events(vec![started(20, 20)]),
            // An answer's (a guards' question accepted): a command of its own after it, once.
            accepted(GameCommand::Answer {
                answer: Answer::Offer {
                    consent: Consent::Accept,
                },
            }),
            events(vec![started(13, 13)]),
            battle_command(),
            events(vec![started(13, 13)]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let battles: Vec<_> = played.moves.iter().map(|m| m.battle).collect();
    assert_eq!(battles, [Some(61), None]);
    assert_eq!(played.commands.len(), 4);
    assert_eq!(
        played.commands[2..],
        [
            Command::Answer(Answer::Offer {
                consent: Consent::Accept
            }),
            Command::Battle(51)
        ]
    );
}

#[test]
fn a_visit_s_battle_in_a_record() {
    use h3_protocol::wire::game::{AdventureCommand, AdventureEvent, GameCommand, GameEvent, Pos};
    use h3_protocol::wire::{Outcome, Side};
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let tile = Pos {
        x: 11,
        y: 10,
        level: 0,
    };
    let started = GameEvent::BattleStarted {
        heroes: [Some(red), None],
        tile,
    };
    let ended = GameEvent::BattleEnded {
        heroes: [Some(red), None],
        result: Outcome::Victory {
            side: Side::Attacker,
        },
    };
    let removed = GameEvent::Adventure {
        event: AdventureEvent::ObjectRemoved {
            object_type: 30,
            anchor: tile,
        },
    };
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            // A walk's battle (E24 M4 T5b2): its end's events from `BattleEnded` on kept, after
            // a command of the battle's.
            accepted(GameCommand::Adventure {
                command: AdventureCommand::MoveHero {
                    hero: red,
                    path: vec![tile],
                },
            }),
            events(vec![started.clone()]),
            accepted(GameCommand::EndTurn {}),
            events(vec![removed.clone(), ended.clone(), removed.clone()]),
            // A battle no visit began (the turn's): none kept, its end nobody's.
            events(vec![started, ended.clone()]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let [visit] = &played.visits[..] else {
        panic!("one visit: {:?}", played.visits);
    };
    assert_eq!(
        visit.battle,
        Some(engine::Battled {
            started: 51,
            ended: vec![ended, removed],
            tile: Some(h3_map::model::Pos { x: 11, y: 10, z: 0 }),
        })
    );
}

#[test]
fn what_a_record_s_battles_tell() {
    use h3_map_view::engine::visits::{self, Told};
    use h3_protocol::wire::game::{AdventureEvent, Answer, Consent, GameEvent, HeroChange, Pos};
    use h3_protocol::wire::{Outcome, Side};
    let anchor = Pos {
        x: 5,
        y: 4,
        level: 0,
    };
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let end = |result| GameEvent::BattleEnded {
        heroes: [Some(0), None],
        result,
    };
    let adventure = |event| GameEvent::Adventure { event };
    let removed = adventure(AdventureEvent::ObjectRemoved {
        object_type: 30,
        anchor,
    });
    let gone = adventure(AdventureEvent::HeroRemoved {
        hero: 0,
        pos: anchor,
    });
    let flagged = adventure(AdventureEvent::ObjectFlagged {
        object_type: 84,
        anchor,
        owner: 0,
    });
    let experience = GameEvent::HeroChanged {
        player: 0,
        hero: 0,
        change: HeroChange::GainExperience { amount: 15 },
    };
    let visit = |ended: Vec<GameEvent>, answers: Vec<(Answer, Vec<GameEvent>)>| engine::Visit {
        at,
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        events: Vec::new(),
        answers,
        revealed: Vec::new(),
        flag: ended.contains(&flagged).then_some(0),
        battle: Some(engine::Battled {
            started: 4,
            ended,
            tile: None,
        }),
    };
    let fight =
        |ended: Vec<GameEvent>| match visits::told(&visit(ended, Vec::new()), [54, 139], "Rion") {
            Some(Told::Fight {
                battle,
                winner,
                taken,
                lost,
                experience,
            }) => Some((battle, winner, taken, lost, experience)),
            _ => None,
        };
    // A monster beaten and taken, the experience gained; the hero retreated and gone.
    let won = end(Outcome::Victory {
        side: Side::Attacker,
    });
    assert_eq!(
        fight(vec![won.clone(), experience.clone(), removed.clone()]),
        Some((4, 0, true, false, 15))
    );
    let retreated = end(Outcome::Retreat {
        side: Side::Attacker,
    });
    assert_eq!(
        fight(vec![retreated, gone.clone()]),
        Some((4, 1, false, true, 0))
    );
    // Every winner (a side's retreat or surrender the other's win; a draw none's).
    for (result, winner) in [
        (
            Outcome::Victory {
                side: Side::Defender,
            },
            1,
        ),
        (
            Outcome::Retreat {
                side: Side::Defender,
            },
            0,
        ),
        (
            Outcome::Surrender {
                side: Side::Attacker,
            },
            1,
        ),
        (
            Outcome::Surrender {
                side: Side::Defender,
            },
            0,
        ),
        (Outcome::Draw {}, -1),
    ] {
        assert_eq!(fight(vec![end(result)]).map(|f| f.1), Some(winner));
    }
    // Not ended yet, or another class's: no battle told.
    assert_eq!(fight(Vec::new()), None);
    assert!(!matches!(
        visits::told(&visit(vec![won.clone()], Vec::new()), [96, 0], "Rion"),
        Some(Told::Fight { .. })
    ));
    // A mine's guards fought after their question accepted: its flag and resource (MCL-Q48: the
    // subclass); without a flag none; declined, or not ended: nothing of the battle.
    let accept = (
        Answer::Offer {
            consent: Consent::Accept,
        },
        Vec::new(),
    );
    let guarded = |ended: Vec<GameEvent>, answers| match visits::told(
        &visit(ended, answers),
        [53, 7],
        "Rion",
    ) {
        Some(Told::Guarded {
            battle,
            winner,
            lost,
            flag,
            resource,
            experience,
        }) => Some((battle, winner, lost, flag, resource, experience)),
        _ => None,
    };
    assert_eq!(
        guarded(
            vec![won.clone(), experience, flagged.clone()],
            vec![accept.clone()]
        ),
        Some((4, 0, false, Some(0), 7, 15))
    );
    assert_eq!(
        guarded(vec![won.clone()], vec![accept.clone()]),
        Some((4, 0, false, None, 7, 0))
    );
    // E24 M4 T6 (the reviewers' R5-4, R3-2): beaten by the guards, the hero gone.
    let beaten = end(Outcome::Victory {
        side: Side::Defender,
    });
    assert_eq!(
        guarded(vec![beaten, gone.clone()], vec![accept.clone()]),
        Some((4, 1, true, None, 7, 0))
    );
    let decline = (
        Answer::Offer {
            consent: Consent::Decline,
        },
        Vec::new(),
    );
    assert_eq!(guarded(vec![won.clone()], vec![decline]), None);
    assert_eq!(guarded(Vec::new(), vec![accept]), None);
    assert_eq!(guarded(vec![won], Vec::new()), None);
}

/// `record`'s messages up to its message `n` folded into its first observation (as a seat's view
/// is kept: `GameObservation::fold`, `reveal`), and the record since: a snapshot of that view
/// numbered `n`, then the later messages (E24 M5 T3b: the hot seat's host's live start).
fn live_since(record: &engine::GameRecord, n: u32) -> engine::GameRecord {
    let number = |m: &ServerMessage| match m {
        ServerMessage::GameSnapshot { n, .. }
        | ServerMessage::GameEvents { n, .. }
        | ServerMessage::GameReveal { n, .. }
        | ServerMessage::GameTurn { n, .. }
        | ServerMessage::GameAccepted { n, .. }
        | ServerMessage::GameClock { n, .. } => *n,
        other => panic!("{other:?}"),
    };
    let mut view: Option<GameObservation> = None;
    for message in record.messages.iter().take_while(|m| number(m) <= n) {
        match message {
            ServerMessage::GameSnapshot { observation, .. } => view = Some(observation.clone()),
            ServerMessage::GameEvents { events, .. } => {
                let view = view.as_mut().unwrap();
                for event in events {
                    view.fold(event);
                }
            }
            ServerMessage::GameReveal { reveal, .. } => view.as_mut().unwrap().reveal(reveal),
            _ => {}
        }
    }
    let mut since = record.clone();
    since.messages.retain(|m| number(m) > n);
    since.messages.insert(
        0,
        ServerMessage::GameSnapshot {
            n,
            observation: view.unwrap(),
        },
    );
    since.header.first = n;
    since
}

#[test]
fn a_live_start_from_a_later_snapshot() {
    // E24 M5 T3b: the hot seat's host sets a view up from a later snapshot of it. A record's file
    // cut so is refused; a live start takes it, the facings the host saw (else right).
    let g = game();
    let text = session_record(&g, 10);
    let whole = engine::parse(&text).unwrap();
    // At the game's snapshot a live start is the record's start.
    let start_of = |loaded: &engine::Loaded| {
        engine::start(
            loaded,
            &g.art,
            &templates(),
            &town_art(),
            records(),
            rules(),
        )
        .unwrap()
    };
    let record = engine::check(whole.clone(), &g.bytes, rules()).unwrap();
    let at_one = engine::live(whole.clone(), &g.bytes, rules(), engine::Live::default()).unwrap();
    // A live start is checked as a record is: another map file's refused.
    let other = engine::live(whole.clone(), b"junk", rules(), engine::Live::default());
    assert!(matches!(other, Err(Refused::Map(..))));
    assert_eq!(start_of(&at_one), start_of(&record));
    // Red's view after its End Turn of day 9, its next turn day 10's (the second week's).
    let ends: Vec<u32> = whole
        .messages
        .iter()
        .filter_map(|m| match m {
            ServerMessage::GameAccepted {
                n,
                command: h3_protocol::wire::game::GameCommand::EndTurn {},
                ..
            } => Some(*n),
            _ => None,
        })
        .collect();
    let n = ends[8] + 1;
    let since = live_since(&whole, n);
    let cut = engine::check(since.clone(), &g.bytes, rules()).unwrap();
    assert_eq!(cut.first().map(|_| ()), Err(Refused::Cut(n)));
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap()
        .id;
    let facings = engine::Live {
        facings: vec![(red, 6)],
        picks: None,
    };
    let loaded = engine::live(since, &g.bytes, rules(), facings).unwrap();
    let start = start_of(&loaded);
    let first = loaded.first().unwrap();
    assert_eq!(first.day, 9);
    let heroes = start.table["heroes"].as_array().unwrap();
    for (hero, row) in first.adventure.heroes.iter().zip(heroes) {
        let facing = numbers(row)[4];
        assert_eq!(facing, if hero.id == red { 6 } else { 2 }, "{}", hero.id);
    }
    // Red's walk north is in the snapshot: its hero where the walk left it.
    assert_eq!(numbers(&heroes[0])[..2], [10, 8]);
    // Its next turn, day 10, as the whole record's: the week's and the month's numbers by the
    // start's day.
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    assert!(played.waits);
    // The day from the date's month, week and weekday.
    let day = |t: &Value| {
        let [m, w, d] = numbers(&table(&table(t)["kingdom"])["date"])[..] else {
            panic!("a date")
        };
        (m - 1) * 28 + (w - 1) * 7 + d
    };
    assert_eq!(day(&played.turns[0]), 10);
    let all = engine::played(&record, &g.art, rules()).unwrap();
    let same = all.turns.iter().find(|t| day(t) == 10).unwrap();
    assert_eq!(&played.turns[0], same);
    let later = all.turns.iter().filter(|t| day(t) >= 10).count();
    assert_eq!(played.turns.len(), later);
}

#[test]
fn the_kingdom_s_heroes_in_the_hero_list_s_order() {
    // E24 M5 T3b: the kingdom's heroes in the view's hero list's order (GAM-HER-003; a live
    // start's), at a new game the types' order.
    let g = panels_game(None);
    let mut o = g.observation.clone();
    let mine: Vec<u16> = o
        .adventure
        .heroes
        .iter()
        .filter(|h| h.owner == 0)
        .map(|h| h.id)
        .collect();
    assert_eq!(mine.len(), 2);
    let rows = |o: &GameObservation| -> Vec<i64> {
        let start = start(&g, o).unwrap();
        table(&start.table["kingdom"])["heroes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| numbers(r)[2])
            .collect()
    };
    // At the game's start the list is by type: Rion (8), then Astral (40).
    assert_eq!(rows(&o), [8, 40]);
    o.hero_list.reverse();
    assert_eq!(rows(&o), [40, 8]);
    // A hero the list lacks comes after the listed ones.
    o.hero_list.retain(|&id| id == mine[1]);
    assert_eq!(rows(&o).len(), 2);
}

#[test]
fn a_live_visit_asks_the_question_of_the_decision_held_open() {
    // E24 M5 T4b (design §5.27): the question's facts the engine's decision's, its answers the
    // host's.
    use h3_map::model::Rewards;
    use h3_map_view::asks::{Ask, Offer as AskOffer, Reveal};
    use h3_map_view::engine::visits::{self, Answered, Kept, Laid, Told};
    use h3_map_view::live::Stack;
    use h3_protocol::wire::game::{
        Choice, ChoiceKind, Decision, LevelUp, Offer, OfferKind, Pos as WirePos, PrimarySkill,
    };
    let anchor = WirePos {
        x: 5,
        y: 4,
        level: 0,
    };
    let choice = |kind| Decision::Choice {
        choice: Choice {
            hero: 0,
            object_type: 1,
            anchor,
            kind,
        },
    };
    let offer = |kind| Decision::Offer {
        offer: Offer {
            hero: 0,
            object_type: 1,
            anchor,
            kind,
        },
    };
    let gold = |amount| Amount {
        resource: 6,
        amount,
    };
    let chest = choice(ChoiceKind::Chest {
        gold: gold(1500),
        experience: 1000,
    });
    let primary = |amount, price: Vec<Amount>, leave| {
        choice(ChoiceKind::Primary {
            skills: [PrimarySkill::Attack, PrimarySkill::Defense],
            amount,
            price,
            leave,
        })
    };
    let pandora = offer(OfferKind::OpenBox {});
    let ask = |d: &Decision, class| visits::opened(d, [class, 9], Some(14), "Rion");
    // A chest's gold and experience, the Arena's gain, a School's gain and price (none: free),
    // Pandora's Box before its rewards.
    assert_eq!(
        ask(&chest, 101),
        Some(Ask::Chest {
            gold: 1500,
            experience: 1000
        })
    );
    assert_eq!(ask(&primary(2, Vec::new(), false), 4), Some(Ask::Arena(2)));
    assert_eq!(
        ask(&primary(1, vec![gold(1000)], true), 47),
        Some(Ask::School {
            gain: 1,
            price: [6, 1000]
        })
    );
    assert_eq!(
        ask(&primary(1, Vec::new(), true), 47),
        Some(Ask::School {
            gain: 1,
            price: [6, 0]
        })
    );
    assert_eq!(
        ask(&pandora, 6),
        Some(Ask::OpenBox {
            rewards: Rewards::default(),
            hero: "Rion".into()
        })
    );
    // The yes-or-no offers: the Tomb, the Tree's price in gold or gems, a Cartographer's kind
    // (its subclass), a Border Guard, the Pyramid; a monster's creature its subclass, its count
    // the view's word's, no flight of the client's.
    assert_eq!(
        ask(&offer(OfferKind::Search {}), 108),
        Some(Ask::Tomb {
            artifact: None,
            morale: 0
        })
    );
    let gift = |resource| {
        offer(OfferKind::Gift {
            price: vec![Amount {
                resource,
                amount: 10,
            }],
        })
    };
    for (resource, gems) in [(5, true), (6, false)] {
        assert_eq!(
            ask(&gift(resource), 102),
            Some(Ask::Tree {
                gems,
                visited: false,
                poor: false
            })
        );
    }
    let reveal = offer(OfferKind::Reveal {
        price: vec![gold(1000)],
    });
    // Its price the offer's (E24 M5 T4c), none: free; what it reveals the answer's.
    assert_eq!(
        ask(&reveal, 13),
        Some(Ask::Maps {
            kind: 9,
            price: [6, 1000],
            reveal: Reveal::None
        })
    );
    assert_eq!(
        ask(&offer(OfferKind::Reveal { price: Vec::new() }), 13),
        Some(Ask::Maps {
            kind: 9,
            price: [6, 0],
            reveal: Reveal::None
        })
    );
    assert_eq!(ask(&offer(OfferKind::Pass {}), 9), Some(Ask::Pass));
    assert_eq!(
        ask(&offer(OfferKind::SearchPyramid {}), 63),
        Some(Ask::Pyramid)
    );
    let monster = |offer| Ask::Monster {
        stack: Stack {
            creature: 9,
            count: 14,
        },
        offer,
        room: true,
        flees: false,
    };
    assert_eq!(
        ask(&offer(OfferKind::Join {}), 54),
        Some(monster(AskOffer::Join))
    );
    let for_gold = offer(OfferKind::JoinForGold {
        price: vec![gold(700)],
    });
    assert_eq!(ask(&for_gold, 54), Some(monster(AskOffer::Gold(700))));
    assert_eq!(
        ask(&offer(OfferKind::Pursue {}), 54),
        Some(monster(AskOffer::Flee))
    );
    let unseen = visits::opened(&offer(OfferKind::Join {}), [54, 9], None, "");
    assert!(matches!(
        unseen,
        Some(Ask::Monster {
            stack: Stack { count: 0, .. },
            ..
        })
    ));
    // Another object's class, another offer or decision: none.
    assert_eq!(ask(&chest, 4), None);
    assert_eq!(ask(&primary(2, Vec::new(), false), 101), None);
    assert_eq!(ask(&pandora, 101), None);
    assert_eq!(ask(&offer(OfferKind::Pass {}), 6), None);
    let level = Decision::LevelUp {
        level_up: LevelUp {
            hero: 0,
            level: 2,
            primary: PrimarySkill::Attack,
            offers: [0, 1],
        },
    };
    assert_eq!(ask(&level, 101), None);
    // Laid out: the walk's visit with no answer asks it, nothing kept (every answer the host's);
    // another walk's visit, unanswered, tells nothing (the record's: no answer).
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let visit = |walk| engine::Visit {
        at,
        walk,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        flag: None,
        events: Vec::new(),
        answers: Vec::new(),
        revealed: Vec::new(),
        battle: None,
    };
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml"),
    )
    .unwrap();
    let tables = tables::load(&[tables::parse(&text).unwrap()], "base").unwrap();
    let open = [(0, chest)];
    let laid = visits::lay(
        &[visit(0), visit(1)],
        Some(&open),
        |v| Some(placed(v, 3, [101, 0], false)),
        &tables.map.finds,
        |_| "Rion".into(),
    );
    let [
        Laid {
            told:
                Told::Ask {
                    ask: Ask::Chest { gold: 1500, .. },
                    answered: Answered::Open,
                    flag: None,
                },
            object: 3,
            walk: 0,
            kept,
            ..
        },
    ] = &laid[..]
    else {
        panic!("the open question alone: {laid:?}");
    };
    assert_eq!(*kept, Kept::default());
    // A visit that tells nothing but opens level-ups (the Tree's free level): live, its levels;
    // a record's, nothing.
    let leveled = engine::Visit {
        levels: vec![0],
        ..visit(1)
    };
    let lay = |open| {
        visits::lay(
            core::slice::from_ref(&leveled),
            open,
            |v| Some(placed(v, 3, [102, 0], false)),
            &tables.map.finds,
            |_| String::new(),
        )
    };
    let [
        Laid {
            told: Told::Levels { .. },
            levels,
            blocks: false,
            ..
        },
    ] = &lay(Some(&[]))[..]
    else {
        panic!("its levels");
    };
    assert_eq!(levels, &[0]);
    assert!(lay(None).is_empty());
}

#[test]
fn a_live_visit_s_notice_of_a_decision_the_view_does_not_ask() {
    // E24 M5 T4c (design §5.28): a fight's, a hire's, a window's notice, whatever the object, its
    // every answer its OK's; laid out from beside where the object blocks its tile, a meeting's on
    // no object; a town entered its screen's notice.
    use h3_map_view::asks::Ask;
    use h3_map_view::engine::visits::{self, Answered, Kept, Laid, NO_OBJECT, Told};
    use h3_map_view::notice::Notice;
    use h3_protocol::wire::game::{
        Answer, Consent, Decision, GameEvent, Offer, OfferKind, Pos as WirePos,
    };
    let offer = |kind| Decision::Offer {
        offer: Offer {
            hero: 0,
            object_type: 1,
            anchor: WirePos {
                x: 5,
                y: 4,
                level: 0,
            },
            kind,
        },
    };
    let fight = offer(OfferKind::Fight {});
    let meeting = Decision::Meeting { heroes: [0, 1] };
    for (decision, class, notice) in [
        (&fight, 79, Notice::Battle),
        (&fight, 54, Notice::Battle),
        (&meeting, 0, Notice::Meeting),
    ] {
        assert_eq!(
            visits::opened(decision, [class, 0], None, "Rion"),
            Some(Ask::Notice(notice))
        );
    }
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let visit = |walk, answers: Vec<(Answer, Vec<GameEvent>)>| engine::Visit {
        at,
        walk,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        flag: None,
        events: Vec::new(),
        answers,
        revealed: Vec::new(),
        battle: None,
    };
    let decline = Answer::Offer {
        consent: Consent::Decline,
    };
    let told = |v: &engine::Visit, d: &Decision| match visits::live(v, d, ([79, 0], None), "") {
        Some(Told::Ask { ask, answered, .. }) => Some((ask, answered)),
        _ => None,
    };
    let battle = Ask::Notice(Notice::Battle);
    assert_eq!(
        told(&visit(0, Vec::new()), &fight),
        Some((battle.clone(), Answered::Open))
    );
    assert_eq!(
        told(&visit(0, vec![(decline, Vec::new())]), &fight),
        Some((battle, Answered::Yes))
    );
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml"),
    )
    .unwrap();
    let tables = tables::load(&[tables::parse(&text).unwrap()], "base").unwrap();
    let lay = |visits: &[engine::Visit], open: Option<&[(usize, Decision)]>, beside| {
        visits::lay(
            visits,
            open,
            |v: &engine::Visit| match v.at.x {
                5 => Some(placed(v, 3, [79, 0], beside)),
                _ => None,
            },
            &tables.map.finds,
            |_| String::new(),
        )
    };
    // A guarded pile's fight: from beside where the walk stops one short of it; on its entrance,
    // not.
    let open = [(0, fight.clone())];
    for beside in [true, false] {
        let [
            Laid {
                told:
                    Told::Ask {
                        ask: Ask::Notice(Notice::Battle),
                        answered: Answered::Open,
                        ..
                    },
                object: 3,
                blocks,
                kept,
                ..
            },
        ] = &lay(&[visit(0, Vec::new())], Some(&open), beside)[..]
        else {
            panic!("the fight's notice");
        };
        assert_eq!((*blocks, *kept), (beside, Kept::default()));
    }
    // A monster's offer on the way (its zone entered): told where the engine's walk ended, on its
    // tile or from beside by the engine's walk.
    let zone = h3_map::model::Pos { x: 4, ..at };
    let join = [(0, offer(OfferKind::Join {}))];
    for beside in [false, true] {
        let laid = visits::lay(
            &[visit(0, Vec::new())],
            Some(&join),
            |_| {
                Some(visits::Placed {
                    at: zone,
                    object: 3,
                    class: [54, 9],
                    word: Some(14),
                    beside,
                })
            },
            &tables.map.finds,
            |_| String::new(),
        );
        let [
            Laid {
                at,
                blocks,
                told:
                    Told::Ask {
                        ask: Ask::Monster { .. },
                        ..
                    },
                ..
            },
        ] = &laid[..]
        else {
            panic!("the monster's offer");
        };
        assert_eq!((*at, *blocks), (zone, beside));
    }
    // A meeting on no object's entrance: on none, from beside; another walk's visit there, or a
    // record's, nothing.
    let elsewhere = h3_map::model::Pos { x: 9, ..at };
    let met = [engine::Visit {
        at: elsewhere,
        ..visit(1, Vec::new())
    }];
    let open = [(1, meeting.clone())];
    let [
        Laid {
            told:
                Told::Ask {
                    ask: Ask::Notice(Notice::Meeting),
                    ..
                },
            object: NO_OBJECT,
            blocks: true,
            ..
        },
    ] = &lay(&met, Some(&open), false)[..]
    else {
        panic!("the meeting's notice");
    };
    assert!(lay(&met, Some(&[(0, meeting)]), false).is_empty());
    assert!(lay(&met, None, false).is_empty());
    // A town entered (live, nothing else told): the town screen's notice; a record's, nothing.
    let town = |open| {
        visits::lay(
            &[visit(0, Vec::new())],
            open,
            |v| Some(placed(v, 2, [98, 1], false)),
            &tables.map.finds,
            |_| String::new(),
        )
    };
    let [
        Laid {
            told: Told::Notice(Notice::Town),
            blocks: false,
            kept,
            ..
        },
    ] = &town(Some(&[]))[..]
    else {
        panic!("the town's notice");
    };
    assert_eq!(*kept, Kept::default());
    assert!(town(None).is_empty());
}

#[test]
fn a_live_visit_s_answer_is_the_record_s() {
    // E24 M5 T4b (design §5.27): the button by the engine's answer, the question's facts brought
    // up to what the answer told.
    use h3_map_view::asks::{Ask, Offer as AskOffer, Reveal};
    use h3_map_view::engine::visits::{self, Answered, Told};
    use h3_map_view::live::Stack;
    use h3_protocol::wire::game::{
        AdventureEvent, Answer, ArenaChoice, Artifact, ChestChoice, Choice, ChoiceKind, Consent,
        Decision, GameEvent, HeroChange, Offer, OfferKind, Pos as WirePos, PrimarySkill,
        SchoolChoice,
    };
    let anchor = WirePos {
        x: 5,
        y: 4,
        level: 0,
    };
    let choice = |kind| Decision::Choice {
        choice: Choice {
            hero: 0,
            object_type: 1,
            anchor,
            kind,
        },
    };
    let offer = |kind| Decision::Offer {
        offer: Offer {
            hero: 0,
            object_type: 1,
            anchor,
            kind,
        },
    };
    let visit = |answers: Vec<(Answer, Vec<GameEvent>)>| engine::Visit {
        at: h3_map::model::Pos { x: 5, y: 4, z: 0 },
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        flag: None,
        events: Vec::new(),
        answers,
        revealed: Vec::new(),
        battle: None,
    };
    let told = |v: &engine::Visit, d: &Decision, class| match visits::live(
        v,
        d,
        ([class, 9], Some(14)),
        "Rion",
    ) {
        Some(Told::Ask { ask, answered, .. }) => Some((ask, answered)),
        _ => None,
    };
    let accept = Answer::Offer {
        consent: Consent::Accept,
    };
    let decline = Answer::Offer {
        consent: Consent::Decline,
    };
    // Each choice's button by its answer; an offer's accepted or declined.
    let chest = choice(ChoiceKind::Chest {
        gold: Amount {
            resource: 6,
            amount: 1000,
        },
        experience: 500,
    });
    let primary = |leave| {
        choice(ChoiceKind::Primary {
            skills: [PrimarySkill::Attack, PrimarySkill::Defense],
            amount: 2,
            price: Vec::new(),
            leave,
        })
    };
    let buttons = [
        (
            Answer::Chest {
                choice: ChestChoice::Gold,
            },
            &chest,
            101,
            Answered::Yes,
        ),
        (
            Answer::Chest {
                choice: ChestChoice::Experience,
            },
            &chest,
            101,
            Answered::Second,
        ),
        (
            Answer::Arena {
                choice: ArenaChoice::Attack,
            },
            &primary(false),
            4,
            Answered::Yes,
        ),
        (
            Answer::Arena {
                choice: ArenaChoice::Defense,
            },
            &primary(false),
            4,
            Answered::Second,
        ),
        (
            Answer::School {
                choice: SchoolChoice::First,
            },
            &primary(true),
            47,
            Answered::Yes,
        ),
        (
            Answer::School {
                choice: SchoolChoice::Second,
            },
            &primary(true),
            47,
            Answered::Second,
        ),
        (
            Answer::School {
                choice: SchoolChoice::Leave,
            },
            &primary(true),
            47,
            Answered::No,
        ),
        (accept, &offer(OfferKind::Pass {}), 9, Answered::Yes),
        (decline, &offer(OfferKind::Pass {}), 9, Answered::No),
    ];
    for (answer, decision, class, button) in buttons {
        let got = told(&visit(vec![(answer, Vec::new())]), decision, class);
        assert_eq!(got.map(|(_, b)| b), Some(button), "{answer:?}");
    }
    // Unanswered: open; a decision asking nothing there: none.
    let open = told(&visit(Vec::new()), &offer(OfferKind::Pass {}), 9);
    assert_eq!(open, Some((Ask::Pass, Answered::Open)));
    assert_eq!(
        told(&visit(Vec::new()), &offer(OfferKind::Pass {}), 6),
        None
    );
    // The Tomb's artifact found, or none.
    let found = GameEvent::HeroChanged {
        player: 0,
        hero: 0,
        change: HeroChange::AddArtifact {
            artifact: Artifact {
                artifact: 12,
                spell: None,
            },
        },
    };
    let tomb = offer(OfferKind::Search {});
    let searched = |events| told(&visit(vec![(accept, events)]), &tomb, 108);
    let gold = GameEvent::ResourceGained {
        player: 0,
        resource: 6,
        amount: 1500,
    };
    // Its morale lost the record's (E24 M5 T4c: the once-mark's `OnceMarkChanged`).
    let marked = GameEvent::Adventure {
        event: AdventureEvent::OnceMarkChanged {
            hero: 0,
            bonus: VisitBonus {
                mark: 3,
                morale: -3,
                luck: 0,
            },
            held: true,
        },
    };
    assert_eq!(
        searched(vec![gold.clone(), found, marked]),
        Some((
            Ask::Tomb {
                artifact: Some(12),
                morale: -3
            },
            Answered::Yes
        ))
    );
    assert_eq!(
        searched(Vec::new()),
        Some((
            Ask::Tomb {
                artifact: None,
                morale: 0
            },
            Answered::Yes
        ))
    );
    // A Cartographer's tiles the answer's reveal (E24 M5 T4c: `Visit::revealed`), none told:
    // none.
    let maps = offer(OfferKind::Reveal {
        price: vec![Amount {
            resource: 6,
            amount: 1000,
        }],
    });
    let tiles = vec![h3_map::model::Pos { x: 3, y: 4, z: 0 }];
    let revealed = engine::Visit {
        revealed: vec![tiles.clone()],
        ..visit(vec![(accept, Vec::new())])
    };
    let mapped = |reveal| {
        Some((
            Ask::Maps {
                kind: 9,
                price: [6, 1000],
                reveal,
            },
            Answered::Yes,
        ))
    };
    assert_eq!(told(&revealed, &maps, 13), mapped(Reveal::Tiles(tiles)));
    assert_eq!(
        told(&visit(vec![(accept, Vec::new())]), &maps, 13),
        mapped(Reveal::Tiles(Vec::new()))
    );
    // A monster's creatures joined (their count the record's); none joined: no room.
    let joined = GameEvent::Adventure {
        event: AdventureEvent::CreaturesJoined {
            hero: 0,
            creature: 9,
            count: 17,
        },
    };
    let join = offer(OfferKind::Join {});
    let monster = |events| told(&visit(vec![(accept, events)]), &join, 54);
    let stack = |count| Stack { creature: 9, count };
    assert_eq!(
        monster(vec![gold.clone(), joined]),
        Some((
            Ask::Monster {
                stack: stack(17),
                offer: AskOffer::Join,
                room: true,
                flees: false
            },
            Answered::Yes
        ))
    );
    assert_eq!(
        monster(Vec::new()),
        Some((
            Ask::Monster {
                stack: stack(14),
                offer: AskOffer::Join,
                room: false,
                flees: false
            },
            Answered::Yes
        ))
    );
    // Pandora's Box's rewards the answer's; what the answers give the panels.
    let opened = visit(vec![(accept, vec![gold])]);
    let Some((Ask::OpenBox { rewards, hero }, Answered::Yes)) =
        told(&opened, &offer(OfferKind::OpenBox {}), 6)
    else {
        panic!("Pandora's Box opened");
    };
    assert_eq!((rewards.resources[6], hero.as_str()), (1500, "Rion"));
    assert_eq!(visits::answered_gain(&opened).resources[6], 1500);
    assert!(!visits::removes(&[]));
}

#[test]
fn a_record_s_revisit_is_a_visit_past_the_walks() {
    // E24 M5 T4b (design §5.27): Space's `Revisit` a visit on the hero's tile, keyed past the
    // walks (its find never told at a walk's end), its events its own; one of a hero the record
    // does not show, none.
    use h3_protocol::wire::game::{AdventureCommand, GameCommand, GameEvent, Pos};
    let g = game();
    let red = g
        .observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.owner == 0)
        .unwrap();
    let (id, at) = (red.id, red.pos);
    let revisit = |hero| {
        accepted(GameCommand::Adventure {
            command: AdventureCommand::Revisit { hero },
        })
    };
    let gold = GameEvent::ResourceGained {
        player: 0,
        resource: 6,
        amount: 100,
    };
    let walk = accepted(GameCommand::Adventure {
        command: AdventureCommand::MoveHero {
            hero: id,
            path: vec![Pos {
                x: at.x + 1,
                y: at.y,
                level: at.level,
            }],
        },
    });
    let record = text(
        &g.header,
        &[
            snapshot(&g.observation),
            revisit(id),
            events(vec![gold.clone()]),
            revisit(9999),
            events(Vec::new()),
            walk,
            events(Vec::new()),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let keys: Vec<usize> = played.visits.iter().map(|v| v.walk).collect();
    assert_eq!(keys, [engine::REVISIT, 0]);
    let first = &played.visits[0];
    assert_eq!(
        (first.at.x, first.at.y, first.events.as_slice()),
        (
            u8::try_from(at.x).unwrap(),
            u8::try_from(at.y).unwrap(),
            &[gold][..]
        )
    );
}

#[test]
fn an_unnamed_town_named_by_the_client_s_picks_in_play() {
    // E24 M5 T4b (design §5.27): live with the client's picks' seed, an unnamed town's name its
    // draw (`picks::town_name`) in the mouse-over's towns and the status window; a record's its
    // faction's first.
    let g = panels_game(None);
    let mut o = g.observation.clone();
    for town in &mut o.towns {
        town.name = None;
    }
    let record = text(&g.header, &[snapshot(&o)]);
    let start_of = |loaded: &engine::Loaded| {
        engine::start(
            loaded,
            &g.art,
            &templates(),
            &town_art(),
            records(),
            rules(),
        )
        .unwrap()
    };
    let names = |start: &engine::Start| -> (Vec<Value>, Vec<Value>) {
        let hover = table(&start.table["hover"])["towns"].as_array().unwrap();
        let chosen = table(&start.table["chosen"])["towns"].as_array().unwrap();
        (
            hover
                .iter()
                .map(|t| t.as_array().unwrap()[0].clone())
                .collect(),
            chosen
                .iter()
                .map(|t| table(&table(t)["status"])["name"].clone())
                .collect(),
        )
    };
    let picked = engine::Live {
        facings: Vec::new(),
        picks: Some(5),
    };
    let live = engine::live(engine::parse(&record).unwrap(), &g.bytes, rules(), picked).unwrap();
    let (hover, status) = names(&start_of(&live));
    let drawn = |picks| -> Vec<Value> {
        o.towns
            .iter()
            .map(|t| Value::Integer(i64::from(picks::town_name(t.faction, t.town, picks))))
            .collect()
    };
    assert!(!o.towns.is_empty());
    assert_eq!(hover, drawn(Some(5)));
    assert!(!status.is_empty());
    assert!(status.iter().all(|n| drawn(Some(5)).contains(n)));
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    assert_eq!(names(&start_of(&loaded)).0, drawn(None));
}

#[test]
fn a_live_visit_s_battle_tells_its_fight_before_and_after_its_end() {
    // E24 M5 T6a (design §5.33): live, a monster's battle the walk began is its fight before the
    // record ends it (no result yet: the host's) and after (its result); from beside where the
    // engine's walk stops one short, else told where it stopped (a zone entered); a record's not
    // ended tells nothing. An answer's battle (its events hold its `BattleStarted`) is its
    // answer's fight, by the answer's place.
    use h3_map_view::engine::visits::{self, Laid, Told};
    use h3_protocol::wire::game::{AdventureEvent, Answer, Consent, GameEvent, Pos};
    use h3_protocol::wire::{Outcome, Side};
    let at = h3_map::model::Pos { x: 5, y: 4, z: 0 };
    let anchor = Pos {
        x: 5,
        y: 4,
        level: 0,
    };
    let won = GameEvent::BattleEnded {
        heroes: [Some(0), None],
        result: Outcome::Victory {
            side: Side::Attacker,
        },
    };
    let removed = GameEvent::Adventure {
        event: AdventureEvent::ObjectRemoved {
            object_type: 54,
            anchor,
        },
    };
    let gone = GameEvent::Adventure {
        event: AdventureEvent::HeroRemoved {
            hero: 0,
            pos: anchor,
        },
    };
    let started = GameEvent::BattleStarted {
        heroes: [Some(0), None],
        tile: anchor,
    };
    let visit = |ended: Vec<GameEvent>, answers: Vec<(Answer, Vec<GameEvent>)>| engine::Visit {
        at,
        walk: 0,
        hero: 0,
        hero_type: None,
        held: engine::Held::default(),
        levels: Vec::new(),
        events: Vec::new(),
        answers,
        revealed: Vec::new(),
        flag: None,
        battle: Some(engine::Battled {
            started: 4,
            ended,
            tile: Some(at),
        }),
    };
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../map/assets/base.toml"),
    )
    .unwrap();
    let tables = tables::load(&[tables::parse(&text).unwrap()], "base").unwrap();
    let lay = |v: &engine::Visit, open: Option<&[_]>, beside| {
        visits::lay(
            core::slice::from_ref(v),
            open,
            |v| Some(placed(v, 3, [54, 139], beside)),
            &tables.map.finds,
            |_| String::new(),
        )
    };
    let fight = |winner, taken, lost| Told::Fight {
        battle: 4,
        winner,
        taken,
        lost,
        experience: 0,
    };
    // Not ended: live its fight with no result, from beside or where the walk stopped.
    let open = visit(Vec::new(), Vec::new());
    for beside in [true, false] {
        let [Laid { told, blocks, .. }] = &lay(&open, Some(&[]), beside)[..] else {
            panic!("its fight");
        };
        assert_eq!((told, *blocks), (&fight(-1, false, false), beside));
    }
    assert!(lay(&open, None, true).is_empty());
    // Ended: its result, live and a record's alike (a record's from beside).
    let ended = visit(vec![won.clone(), removed.clone()], Vec::new());
    let [Laid { told, .. }] = &lay(&ended, Some(&[]), false)[..] else {
        panic!("its fight");
    };
    assert_eq!(told, &fight(0, true, false));
    let [Laid { blocks: true, .. }] = &lay(&ended, None, false)[..] else {
        panic!("a record's fight from beside");
    };
    // A battle an answer began is its answer's, not the walk's.
    let decline = Answer::Offer {
        consent: Consent::Decline,
    };
    let answered = visit(
        vec![won, gone],
        vec![(decline, Vec::new()), (decline, vec![started])],
    );
    assert_eq!(
        visits::answer_fight(&answered, 1),
        Some(fight(0, false, true))
    );
    let taken = engine::Visit {
        battle: Some(engine::Battled {
            started: 4,
            ended: vec![removed.clone()],
            tile: Some(at),
        }),
        ..answered.clone()
    };
    assert_eq!(
        visits::answer_fight(&taken, 1),
        Some(fight(-1, true, false))
    );
    assert_eq!(visits::answer_fight(&answered, 0), None);
    assert_eq!(visits::answer_fight(&answered, 2), None);
    let not_yet = engine::Visit {
        battle: None,
        ..answered.clone()
    };
    assert_eq!(visits::answer_fight(&not_yet, 1), None);
    let unended = visit(
        Vec::new(),
        vec![(
            decline,
            vec![GameEvent::BattleStarted {
                heroes: [Some(0), None],
                tile: anchor,
            }],
        )],
    );
    assert_eq!(
        visits::answer_fight(&unended, 0),
        Some(fight(-1, false, false))
    );
    // A visit with answers is no walk's fight; another class's none.
    let other = visit(Vec::new(), Vec::new());
    let mine = visits::lay(
        core::slice::from_ref(&other),
        Some(&[]),
        |v| Some(placed(v, 3, [53, 7], false)),
        &tables.map.finds,
        |_| String::new(),
    );
    assert!(mine.is_empty());
    assert!(
        lay(&unended, Some(&[]), false)
            .iter()
            .all(|l| !matches!(l.told, Told::Fight { .. }))
    );
}
