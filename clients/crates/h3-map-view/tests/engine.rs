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
    { at = [8, 10], class = 1, map_army = [[0, 15], [2, 5]] },
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
    // (MCL-Q36), its army by slot.
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
    assert_eq!(army[..3], [vec![0, 15], vec![2, 5], vec![-1, 0]]);
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
    town.garrison.army = Some(vec![h3_protocol::wire::game::ArmyStack {
        slot: 2,
        creature: 4,
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
    assert_eq!(numbers(&status["army"].as_array().unwrap()[2]), [4, 3]);
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
    session.record(PlayerId::new(0)).unwrap()
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
    // Each week's and month's kind but the normal ones, each with a turn's start.
    let starts = [
        (
            GameEvent::MonthStarted {
                month: 2,
                kind: Month::Plague {},
            },
            [2, 0],
        ),
        (
            GameEvent::MonthStarted {
                month: 3,
                kind: Month::Creature { creature: 12 },
            },
            [1, 12],
        ),
        (
            GameEvent::WeekStarted {
                week: 9,
                kind: Week::Creature { creature: 7 },
            },
            [1, 7],
        ),
        (
            GameEvent::WeekStarted {
                week: 10,
                kind: Week::Of {
                    creatures: [Some(30), Some(31)],
                    bonus: 5,
                },
            },
            [2, 0],
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
        assert_eq!(numbers(&table(&played.turns[i])[key]), want, "{start:?}");
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
    // The heroes' experience at the start, by their tiles.
    assert_eq!(
        played.experience,
        [(
            tile(
                u8::try_from(red.pos.x).unwrap(),
                u8::try_from(red.pos.y).unwrap()
            ),
            details.experience
        )]
    );
}

/// Another player's turn in a record (E24 M4 T4c, design §5.16): Blue's steps by hero, a boat
/// boarded and left, a step out of sight; Purple's turn with none seen no turn; a town's command
/// no walk.
#[test]
fn a_record_s_other_players_turns() {
    use h3_map_view::engine::{Command, OtherStep};
    use h3_map_view::live::{Facing, LiveBoat};
    use h3_map_view::timeline::Sea;
    use h3_protocol::wire::game::{AdventureEvent, GameCommand, GameEvent, Pos, TownCommand};
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
                    object_type: 79,
                    anchor: pos(13, 12),
                }),
                event(AdventureEvent::MovementChanged {
                    hero: blue,
                    movement: 0,
                    max_movement: 1500,
                }),
            ]),
            events(vec![
                GameEvent::TurnEnded { player: 1 },
                GameEvent::TurnStarted { player: 2 },
                // Purple's: an object taken with no step (E24 M4 T4c2, MCL-Q46: off the map as
                // the next turn table starts), one off the map's positions (nothing) and a
                // landing with no walk (nothing).
                event(AdventureEvent::ObjectRemoved {
                    object_type: 79,
                    anchor: pos(13, 12),
                }),
                event(AdventureEvent::ObjectRemoved {
                    object_type: 79,
                    anchor: pos(300, 12),
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
    assert_eq!(other.walks[2].steps, [seen(8, None, None)]);
    assert_eq!(other.walks[2].taken, [tile(13, 12)]);
    assert!(other.walks[0].taken.is_empty());
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
    // Purple's pickup without a step: given to Red's day 2's table.
    assert_eq!(played.taken, [(1, vec![tile(13, 12)])]);
}

#[test]
fn what_a_record_s_visits_tell() {
    use h3_map_view::asks::Ask;
    use h3_map_view::engine::visits::{self, Answered, Told};
    use h3_map_view::finds::{Find, Gain};
    use h3_protocol::wire::game::{
        AdventureEvent, Answer, ArenaChoice, ChestChoice, Consent, GameEvent, HeroChange,
        NothingWhy, Pos,
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
        events,
        answers: Vec::new(),
        battle: None,
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
    // An artifact added and taken; none without it; another class tells nothing.
    assert_eq!(
        find(found(5, vec![removed.clone(), artifact])).map(|f| (f.0, f.2)),
        Some((Find::Artifact { artifact: 108 }, true))
    );
    assert_eq!(found(5, vec![removed]), None);
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
        events,
        answers,
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
    // colour; unflagged (another's own visit), an abandoned mine's type: nothing.
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
    assert_eq!(told(&flags, 53, 7), None);
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
    let declined = visit(
        vec![flagged],
        vec![(
            Answer::Offer {
                consent: Consent::Decline,
            },
            Vec::new(),
        )],
    );
    assert_eq!(told(&declined, 20, 0), None);
    assert_eq!(told(&visit(Vec::new(), Vec::new()), 17, 0), None);
}

#[test]
fn a_record_s_boats() {
    use h3_map_view::live::{Facing, LiveBoat};
    use h3_map_view::timeline::Sea;
    use h3_protocol::wire::game::{AdventureCommand, AdventureEvent, GameCommand, GameEvent, Pos};
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
            events(vec![stepped(13, 10), landed(red, 9)]),
            // Another hero's boarding: no walk of its own open.
            events(vec![adventure(AdventureEvent::Boarded {
                hero: blue,
                boat: 0,
                at: pos(30, 30),
            })]),
        ],
    );
    let loaded = engine::load(&record, &g.bytes, rules()).unwrap();
    let played = engine::played(&loaded, &g.art, rules()).unwrap();
    let seas: Vec<_> = played.moves.iter().map(|m| m.sea).collect();
    let boat = LiveBoat {
        at: h3_map::model::Pos { x: 11, y: 10, z: 0 },
        kind: 1,
        facing: Facing::Left,
    };
    assert_eq!(seas, [Some(Sea::Board(2)), Some(Sea::Land(boat)), None]);
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
        events: Vec::new(),
        answers,
        battle: Some(engine::Battled { started: 4, ended }),
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
        guarded(vec![won.clone(), experience, flagged], vec![accept.clone()]),
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
