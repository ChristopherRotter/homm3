//! `h3_map_view::engine` (E24 M4 T3a): a whole game's record read and checked, and its start as a
//! capture's live part, on a game the engine builds on a writer's map with the repository's packs
//! (no original data): a random resource, monster and artifact, an owned sawmill, a pile, heroes
//! of three colours and an empty boat; then every refusal, and what `differences` finds. Since
//! E24 M4 T3b1 the panels: the kingdom, the status window, the kingdom's window and what choosing
//! each hero and town shows, on a map with Red's town (`PANELS`).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use h3_contracts::PlayerId;
use h3_game::{GameRuleset, GameState, Viewer};
use h3_map::writer::{self, ObjectTemplate};
use h3_map::{Limits, MAP_IMPORT_VERSION};
use h3_map_view::art::MapArt;
use h3_map_view::engine::{self, Refused};
use h3_map_view::situation::Situation;
use h3_protocol::RECORD_FORMAT_VERSION;
use h3_protocol::codec;
use h3_protocol::map::game::{manifest, observe};
use h3_protocol::wire::game::{GameObservation, GameRecordHeader, MapIdentity};
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
    engine::start(&loaded, &g.art, &templates(), rules())
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
    let at = |o: &h3_protocol::wire::game::Object| (o.anchor.x, o.anchor.y) == (8, 15);
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
        engine::start(&loaded, &g.art, &[], rules()),
        Err(Refused::NoStart)
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
        engine::start(&loaded, &other, &[], rules()),
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
    ];
    let texts: Vec<String> = all.iter().map(ToString::to_string).collect();
    assert!(texts.iter().all(|t| !t.is_empty()));
    assert_eq!(texts[1], "line 3: bad");
    assert!(texts[4].contains("0x0000000000000001"));
    assert!(texts[10].contains("(1, 2, 0)"));
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
        .map(|(resource, amount)| h3_protocol::wire::game::Amount { resource, amount })
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
    // A capture without `[chosen]` compares no choice; nor do two without a window.
    let mut theirs = ours.table.clone();
    theirs.remove("chosen");
    let mut mine = ours.clone();
    mine.table.remove("status");
    theirs.remove("status");
    assert!(engine::differences(&mine, &theirs).is_empty());
}
