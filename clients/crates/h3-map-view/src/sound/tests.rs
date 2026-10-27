use super::*;

use alloc::string::ToString;
use alloc::vec;

use crate::art::{COAST_BIT, ObjectArt, Piece, TemplateArt};
use crate::live::{Drawn, Explored, Facing, LiveHero, LiveObject, Moment, Walking};
use crate::objects::{Shape, Shapes};
use crate::tables;

/// The committed table's `[map.sound]`, as written.
fn art() -> SoundArt {
    let text = include_str!("../../../../map/assets/base.toml");
    tables::parse(text).unwrap().map.unwrap().sound.unwrap()
}

/// The committed tables' sound rules.
pub(crate) fn rules() -> SoundRules {
    let text = include_str!("../../../../map/assets/base.toml");
    tables::load(&[tables::parse(text).unwrap()], "base")
        .unwrap()
        .sound
        .unwrap()
}

/// Tiles of a `size` map on one level: each tile's terrain and sound by its coordinates, no hero.
pub(crate) fn tiles(
    size: i32,
    terrain: impl Fn(i32, i32) -> u8,
    sound: impl Fn(i32, i32) -> Option<u8>,
) -> TileSounds {
    let mut terrains = Vec::new();
    let mut sounds = Vec::new();
    for y in 0..size {
        for x in 0..size {
            terrains.push(terrain(x, y));
            sounds.push(sound(x, y));
        }
    }
    TileSounds {
        size,
        levels: 1,
        terrains,
        sounds,
        heroes: Vec::new(),
    }
}

/// A sound state over `tiles` with the committed rules.
pub(crate) fn sound(tiles: TileSounds) -> Sound {
    Sound::new(Arc::new(rules()), Arc::new(tiles))
}

fn pos(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

fn names(cues: &[(u32, Cue)]) -> Vec<String> {
    cues.iter().map(|(t, c)| line(*t, c)).collect()
}

#[test]
fn the_tables_sounds_are_the_originals() {
    let r = rules();
    assert_eq!(r.loops.len(), 70);
    assert_eq!(
        (r.loops[0].as_str(), r.loops[69].as_str()),
        ("LoopAnim.wav", "loopocea.wav")
    );
    let music: Vec<(u8, &str)> = r.music.iter().map(|(c, n)| (*c, n.as_str())).collect();
    assert_eq!(
        music,
        [
            (0, "Dirt"),
            (1, "Sand"),
            (2, "Grass"),
            (3, "Snow"),
            (4, "Swamp"),
            (5, "Rough"),
            (6, "Underground"),
            (7, "Lava"),
            (8, "Water")
        ]
    );
    assert_eq!(r.steps.len(), 10);
    assert_eq!(
        (r.steps[&0].as_str(), r.steps[&9].as_str()),
        ("horse00.wav", "horse09.wav")
    );
    assert_eq!(r.flying, "horse10.wav");
    assert_eq!(r.wavs().len(), 81);
    let volumes: Vec<u8> = (0..5).map(|d| r.volume(d)).collect();
    assert_eq!(volumes, [32, 28, 20, 10, 0]);
    assert_eq!(
        (r.game_class(202), r.game_class(220), r.game_class(5)),
        (158, 53, 5)
    );
    // `tile_ambient_sound`'s switch: by class, subclass, a dwelling's creature, a mine's type.
    for (class, subclass, map_class, want) in [
        (4, 0, 4, Some(2)),
        (16, 6, 16, Some(23)),
        (16, 7, 16, None),
        (20, 0, 20, Some(43)),
        (20, 1, 20, Some(12)),
        (33, 1, 33, Some(25)),
        (99, 0, 221, Some(26)),
        (8, 0, 8, None),
        // A dwelling: subclass 56's creature 0 (Pikemen), 0's 106, 7's 112, 59's 118 (none), past
        // the table's 80.
        (17, 56, 17, Some(36)),
        (17, 0, 17, Some(33)),
        (17, 7, 17, Some(46)),
        (17, 59, 17, Some(42)),
        (17, 200, 17, Some(42)),
        // A mine by type, abandoned (RoE's subclass 7, AB's 220), a type past the table's.
        (53, 0, 53, Some(24)),
        (53, 6, 53, Some(29)),
        (53, 7, 53, Some(7)),
        (53, 0, 220, Some(7)),
        (53, 9, 53, None),
    ] {
        assert_eq!(
            r.entrance(class, subclass, map_class),
            want,
            "{class}, {subclass}"
        );
    }
}

#[test]
fn the_tables_sounds_break_no_rule() {
    let fails = |change: fn(&mut SoundArt)| {
        let mut a = art();
        change(&mut a);
        SoundRules::new(&a, BTreeMap::new(), BTreeMap::new()).unwrap_err()
    };
    assert_eq!(
        fails(|a| a.loops[0] = "LoopAnim".into()),
        "`LoopAnim` is not a .WAV name"
    );
    assert_eq!(
        fails(|a| a.flying = ".wav".into()),
        "`.wav` is not a .WAV name"
    );
    assert_eq!(
        fails(|a| a.creature_default = "nope.wav".into()),
        "`nope.wav` is not one of the loops"
    );
    assert_eq!(
        fails(|a| a.loops.push("LOOPANIM.WAV".into())),
        "`LoopAnim.wav` is named twice among the loops"
    );
    for change in [
        |a: &mut SoundArt| a.shore = "x.wav".into(),
        |a: &mut SoundArt| a.objects[0].sound = "x.wav".into(),
        |a: &mut SoundArt| a.mines[0] = "x.wav".into(),
        |a: &mut SoundArt| a.abandoned = "x.wav".into(),
        |a: &mut SoundArt| a.blocking[0].sound = "x.wav".into(),
        |a: &mut SoundArt| a.ground_loops[0].sound = "x.wav".into(),
        |a: &mut SoundArt| a.creatures[0].sound = "x.wav".into(),
    ] {
        assert_eq!(fails(change), "`x.wav` is not one of the loops");
    }
    assert_eq!(
        fails(|a| a.volumes = vec![1, 2, 3]),
        "3 volumes, not one for each distance 0 to 3"
    );
    assert_eq!(
        fails(|a| a.loops = (0..257).map(|i| format!("l{i}.wav")).collect()),
        "257 loops, more than 256"
    );
    // A step sound's name is checked too; a ground class without a loop is none.
    let mut steps = BTreeMap::new();
    steps.insert(0, "horse00".to_string());
    assert_eq!(
        SoundRules::new(&art(), BTreeMap::new(), steps).unwrap_err(),
        "`horse00` is not a .WAV name"
    );
}

#[test]
fn the_tables_with_and_without_sound() {
    let text = include_str!("../../../../map/assets/base.toml");
    let t = tables::load(&[tables::parse(text).unwrap()], "base").unwrap();
    let sounds = t.sounds();
    assert_eq!(sounds.len(), 81);
    assert!(sounds.contains("LOOPANIM") && sounds.contains("HORSE10"));
    assert_eq!(t.tracks().len(), 9);
    // Without `[map.sound]`, no sound; a river's music, an error.
    let without = text.replace("[map.sound]", "[unused]");
    let start = without.find("[unused]").unwrap();
    let end = without[start..].find("\n\n").unwrap() + start;
    let mut cut = without.clone();
    cut.replace_range(start..end, "");
    let t = tables::load(&[tables::parse(&cut).unwrap()], "base").unwrap();
    assert!(t.sound.is_none() && t.sounds().is_empty() && t.tracks().is_empty());
    let river = text.replacen(
        "def = \"clrrvr.def\"",
        "def = \"clrrvr.def\"\nmusic = \"Dirt\"",
        1,
    );
    assert_eq!(
        tables::load(&[tables::parse(&river).unwrap()], "base")
            .unwrap_err()
            .to_string(),
        "`river 1`: only a terrain has music and a step sound"
    );
    let broken = text.replacen("abandoned = \"LoopCave.wav\"", "abandoned = \"x.wav\"", 1);
    assert_eq!(
        tables::load(&[tables::parse(&broken).unwrap()], "base")
            .unwrap_err()
            .to_string(),
        "`sound`: `x.wav` is not one of the loops"
    );
}

/// A template of `class` and `subclass` whose cells `blocked` block and `visited` are its
/// entrances, its DEF named by its index.
fn template(
    index: usize,
    class: u32,
    subclass: u32,
    blocked: &[(u8, u8)],
    visited: &[(u8, u8)],
) -> TemplateArt {
    let mut passable = [0xff; 6];
    let mut visitable = [0; 6];
    for &(dx, dy) in blocked {
        passable[usize::from(5 - dy)] &= !(1 << (7 - dx));
    }
    for &(dx, dy) in visited {
        visitable[usize::from(5 - dy)] |= 1 << (7 - dx);
    }
    TemplateArt {
        def: format!("t{index}.def"),
        passable,
        visitable,
        class,
        subclass,
        priority: 0,
    }
}

/// An 8 × 8 grass map with these objects (template, anchor), every DEF one tile but `wide`'s two.
fn map_with(objects: &[(TemplateArt, (u8, u8))], wide: &[usize]) -> (MapArt, Live, Shapes) {
    let grass = TileArt {
        terrain: Piece {
            code: 2,
            picture: 0,
            mirror_x: false,
            mirror_y: false,
        },
        river: None,
        road: None,
        flags: 0,
    };
    let mut tiles = vec![grass; 64];
    for (i, picture) in [(0, 5), (1, 25), (2, 20)] {
        tiles[i].terrain.code = WATER;
        tiles[i].terrain.picture = picture;
    }
    tiles[63].flags = COAST_BIT;
    let mut shapes = Shapes::default();
    let mut art = MapArt {
        size: 8,
        levels: 1,
        tiles,
        templates: Vec::new(),
        objects: Vec::new(),
    };
    for (i, (t, (x, y))) in objects.iter().enumerate() {
        let width = if wide.contains(&i) { 2 } else { 1 };
        shapes.insert(
            &t.def,
            Shape {
                width,
                height: 1,
                colour: [0xff; 6],
                shadow: [0xff; 6],
                frames: 1,
            },
        );
        art.templates.push(t.clone());
        art.objects.push(ObjectArt {
            anchor: pos(*x, *y),
            template: i,
        });
    }
    let live = Live {
        objects: vec![
            LiveObject {
                standing: true,
                drawn: None,
                owner: None,
                phase: 0,
            };
            objects.len()
        ],
        heroes: Vec::new(),
        boats: Vec::new(),
        viewer: 0,
        explored: Explored::all(8, 1, true),
        path: Vec::new(),
        moment: Moment::default(),
        walking: Walking::default(),
    };
    (art, live, shapes)
}

#[test]
fn each_tiles_sound_is_its_recorded_objects() {
    let r = rules();
    let objects = [
        // 0: an arena's entrance at (3, 1).
        (template(0, 4, 0, &[], &[(0, 0)]), (3, 1)),
        // 1, 2: a campfire's entrance under a blocking decoration at (4, 1).
        (template(1, 12, 0, &[], &[(0, 0)]), (4, 1)),
        (template(2, 120, 0, &[(0, 0)], &[]), (4, 1)),
        // 3: a volcano as the map names it (202, loaded as 158) at (5, 1).
        (template(3, 202, 0, &[(0, 0)], &[]), (5, 1)),
        // 4, 5: a volcano at (6, 2) under a crystal mine's blocked cell, its entrance at (7, 2).
        (template(4, 202, 0, &[(0, 0)], &[]), (6, 2)),
        (template(5, 53, 4, &[(1, 0)], &[(0, 0)]), (7, 2)),
        // 6: cursed ground alone at (1, 3); 7: SoD's magic plains (230) at (2, 3); 8: clover
        // field (222, no loop) at (3, 3).
        (template(6, 21, 0, &[], &[]), (1, 3)),
        (template(7, 230, 0, &[], &[]), (2, 3)),
        (template(8, 222, 0, &[], &[]), (3, 3)),
        // 9, 10: cursed ground under a hole (175, loaded as 124), passable, at (4, 3).
        (template(9, 21, 0, &[], &[]), (4, 3)),
        (template(10, 175, 0, &[], &[]), (4, 3)),
        // 11: cursed ground on coastal land (7, 7).
        (template(11, 21, 0, &[], &[]), (7, 7)),
        // 12, 13, 14: buoys on the shore's water (0, 0), open water (1, 0), picture 20 (2, 0).
        (template(12, 11, 0, &[], &[(0, 0)]), (0, 0)),
        (template(13, 11, 0, &[], &[(0, 0)]), (1, 0)),
        (template(14, 11, 0, &[], &[(0, 0)]), (2, 0)),
        // 15: a random dwelling (216) drawn as Pikemen's (17, 56) at (5, 5); 16: one fallen at
        // (6, 5).
        (template(15, 216, 0, &[], &[(0, 0)]), (5, 5)),
        (template(16, 17, 0, &[], &[(0, 0)]), (6, 5)),
    ];
    let (art, mut live, shapes) = map_with(&objects, &[5]);
    live.objects[15].drawn = Some(Drawn {
        def: "pikes.def".into(),
        own: None,
        class: 17,
        subclass: 56,
    });
    live.objects[16].standing = false;
    live.heroes.push(LiveHero {
        at: pos(3, 1),
        class: 0,
        owner: 0,
        facing: Facing::Right,
        boat: None,
    });
    let lists = [DrawLists::build(&art, &live, &shapes, 0).unwrap()];
    let tiles = TileSounds::build(&art, &live, &lists, &r, Some(pos(3, 1)));
    let at = |x, y| tiles.sound([x, y, 0], None);
    assert_eq!(
        [
            at(3, 1),
            at(4, 1),
            at(5, 1),
            at(6, 2),
            at(7, 2),
            at(1, 3),
            at(2, 3),
            at(3, 3),
            at(4, 3),
            at(7, 7),
            at(0, 0),
            at(1, 0),
            at(2, 0),
            at(5, 5),
            at(6, 5),
            at(6, 6),
        ],
        [
            Some(2),
            Some(6),
            Some(45),
            None,
            Some(47),
            Some(48),
            Some(25),
            None,
            None,
            None,
            Some(69),
            Some(5),
            Some(69),
            Some(36),
            None,
            None
        ]
    );
    // The selected hero on its tile hides its sound; another hero's tile is always silent.
    assert_eq!(tiles.sound([3, 1, 0], Some(pos(3, 1))), None);
    let others = TileSounds::build(&art, &live, &lists, &r, None);
    assert_eq!(others.sound([3, 1, 0], None), None);
    // Terrains, and a tile off the map; a level without draw lists sounds as its terrain.
    assert_eq!(
        (
            tiles.terrain([0, 0, 0]),
            tiles.terrain([7, 7, 0]),
            tiles.terrain([8, 0, 0]),
            tiles.terrain([0, -1, 0]),
            tiles.terrain([0, 0, 1])
        ),
        (Some(WATER), Some(2), None, None, None)
    );
    let bare = TileSounds::build(&art, &live, &[], &r, None);
    assert_eq!(
        (bare.sound([3, 1, 0], None), bare.sound([0, 0, 0], None)),
        (None, Some(69))
    );
}

#[test]
fn the_ambient_slots_take_the_nearest_sounds() {
    // Around (10, 10): LoopAren on the middle, LoopDog and LoopCamp at distance 1, LoopBuoy at 2,
    // LoopCave at 3, LoopDead at 4; LoopDrag alone at (25, 25).
    let placed = [
        ((10, 10), 2),
        ((11, 10), 6),
        ((9, 9), 10),
        ((12, 12), 5),
        ((13, 10), 7),
        ((14, 10), 8),
        ((25, 25), 11),
    ];
    let mut s = sound(tiles(
        30,
        |_, _| 2,
        |x, y| placed.iter().find(|(p, _)| *p == (x, y)).map(|(_, s)| *s),
    ));
    s.start(0, [10, 10, 0], None);
    // The music, then four loops in the scan's order (a ring's tiles four at a time: the top
    // row's (9, 9) before the right column's (11, 10)); LoopCave at 3 finds no slot farther. Each
    // was found, so each takes its volume.
    assert_eq!(
        names(&s.take()),
        [
            "0 music Grass",
            "0 loop 0 LoopAren.wav 32",
            "0 loop 1 LoopDog.wav 28",
            "0 loop 2 LoopCamp.wav 28",
            "0 loop 3 LoopBuoy.wav 20",
            "0 loop-volume 0 32",
            "0 loop-volume 1 28",
            "0 loop-volume 2 28",
            "0 loop-volume 3 20",
        ]
    );
    // A step's end a tile east: the loops kept, each at its new distance's volume; LoopCave, now
    // at 2, finds no slot farther.
    s.step_end(1, [11, 10, 0], pos(11, 10), false, 0);
    assert_eq!(
        names(&s.take()),
        [
            "1 loop-volume 0 28",
            "1 loop-volume 1 20",
            "1 loop-volume 2 32",
            "1 loop-volume 3 20",
        ]
    );
    // Far away: the first slot of the farthest taken; then slot by slot, one not found stopped, a
    // found one's volume.
    s.step_end(2, [25, 25, 0], pos(25, 25), false, 0);
    assert_eq!(
        names(&s.take()),
        [
            "2 loop-stop 0",
            "2 loop 0 LoopDrag.wav 32",
            "2 loop-volume 0 32",
            "2 loop-stop 1",
            "2 loop-stop 2",
            "2 loop-stop 3",
        ]
    );
    // `hero_select` restarts every loop.
    s.select(3, [25, 25, 0], pos(25, 25), true);
    assert_eq!(
        names(&s.take()),
        [
            "3 loop-stop 0",
            "3 loop 0 LoopDrag.wav 32",
            "3 loop-volume 0 32"
        ]
    );
    // A tile that names none (x −1): the loops stopped, no scan.
    s.select(4, [NO_TILE, 5, 0], pos(25, 25), false);
    assert_eq!(names(&s.take()), ["4 loop-stop 0"]);
}

#[test]
fn two_sounds_32_apart_share_a_bit() {
    // MCL-Q8: sound 1 found again nearer, then sound 33 started: its XOR clears bit 1, so neither
    // takes a volume after the scan (1 keeps its distance 2's).
    let placed = [((7, 5), 1), ((9, 5), 33)];
    let mut s = sound(tiles(
        20,
        |_, _| 2,
        |x, y| placed.iter().find(|(p, _)| *p == (x, y)).map(|(_, s)| *s),
    ));
    s.start(0, [5, 5, 0], None);
    assert_eq!(
        names(&s.take())[1..],
        ["0 loop 0 LoopArch.wav 20", "0 loop-volume 0 20"]
    );
    s.step_end(1, [6, 5, 0], pos(6, 5), false, 0);
    assert_eq!(names(&s.take()), ["1 loop 1 LoopMons.wav 10"]);
    assert_eq!(bit(33), bit(1));
}

#[test]
fn the_music_and_the_step_sounds_follow_the_terrain() {
    // Grass but a dirt column at x 5, rock at x 6, an unknown terrain at x 7; no ambient sound.
    let mut s = sound(tiles(
        10,
        |x, _| match x {
            5 => 0,
            6 => 9,
            7 => 12,
            _ => 2,
        },
        |_, _| None,
    ));
    // Off the map, tile 0's terrain.
    s.start(0, [-3, 40, 0], None);
    s.select(1, [1, 1, 0], pos(1, 1), true);
    s.select(2, [5, 1, 0], pos(5, 1), false);
    s.select(3, [5, 1, 0], pos(5, 1), true);
    assert_eq!(names(&s.take()), ["0 music Grass", "3 music Dirt"]);
    // A walk: its start's sound, a step on the same terrain nothing, onto grass the music and the
    // sound at the frame's wrap, then rock (no music) and dirt again at frame 0 (the music asked
    // for already).
    s.walk_start(10, pos(5, 1));
    s.step_end(20, [5, 2, 0], pos(5, 2), false, 3);
    s.step_end(30, [4, 2, 0], pos(4, 2), false, 6);
    s.wrap(35);
    s.wrap(36);
    s.step_end(40, [6, 2, 0], pos(6, 2), false, 2);
    s.wrap(45);
    s.step_end(50, [5, 2, 0], pos(5, 2), false, 0);
    assert_eq!(
        names(&s.take()),
        [
            "10 step horse00.wav",
            "30 music Grass",
            "35 step-stop",
            "35 step horse02.wav",
            "45 step-stop",
            "45 step horse09.wav",
            "50 music Dirt",
            "50 step-stop",
            "50 step horse00.wav",
        ]
    );
    // The walk's last step onto grass: the sound stopped, the new terrain's started at frame 0,
    // `hero_walk_end` stopping it.
    s.step_end(60, [4, 2, 0], pos(4, 2), true, 0);
    s.walk_end(60);
    s.walk_end(61);
    assert_eq!(
        names(&s.take()),
        [
            "60 step-stop",
            "60 music Grass",
            "60 step horse02.wav",
            "60 step-stop"
        ]
    );
    // A terrain without a step sound plays none; a step's end queued and the walk ended drops it.
    s.walk_start(70, pos(7, 1));
    s.step_end(80, [5, 1, 0], pos(5, 1), false, 4);
    s.walk_end(85);
    s.wrap(90);
    assert_eq!(names(&s.take()), ["80 music Dirt"]);
    // Grass, rock (no music), grass again: the track asked for already plays on; a hero off the
    // map has no step sound.
    s.walk_start(100, pos(5, 2));
    s.step_end(110, [4, 2, 0], pos(4, 2), false, 0);
    s.step_end(120, [6, 2, 0], pos(6, 2), false, 0);
    s.step_end(130, [4, 2, 0], pos(4, 2), false, 0);
    s.walk_end(135);
    s.walk_start(140, pos(200, 1));
    assert_eq!(
        names(&s.take()),
        [
            "100 step horse00.wav",
            "110 music Grass",
            "110 step-stop",
            "110 step horse02.wav",
            "120 step-stop",
            "120 step horse09.wav",
            "130 step-stop",
            "130 step horse02.wav",
            "135 step-stop",
        ]
    );
    assert_eq!(
        line(1, &Cue::LoopStop { slot: 2 }),
        "1 loop-stop 2".to_string()
    );
}
