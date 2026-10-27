use super::*;

use alloc::vec;

use crate::live::Facing;

const SEQUENCES: &str = include_str!("../../../../map/sequences.toml");

fn one(rest: &str) -> Sequence {
    let text = format!(
        "[[sequence]]\nid = \"s\"\nsituation = \"seq-scroll\"\nms = 100\nwhy = \"a test\"\n{rest}"
    );
    Sequences::parse(&text).unwrap().sequence.remove(0)
}

#[test]
fn reads_the_recorded_sequences() {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    assert_eq!(sequences.sequence.len(), 13);
    let scroll = sequences.get("scroll-2").unwrap();
    assert_eq!((scroll.situation.as_str(), scroll.ms), ("seq-scroll", 3100));
    assert_eq!(scroll.scroll_speed(), Ok(2));
    let actions = scroll.actions().unwrap();
    assert_eq!(actions[&100], [Action::Scroll(Direction::North)]);
    assert_eq!(actions[&300], [Action::Scroll(Direction::NorthEast)]);
    assert_eq!(actions[&1500], [Action::Scroll(Direction::NorthWest)]);
    // A key's release is no action.
    assert!(!actions.contains_key(&160));
    assert_eq!(actions[&2300], [Action::Pointer([2, 300])]);
    assert!(sequences.get("idle").is_none());
    assert_eq!(sequences.get("idle-cycles").unwrap().scroll_speed(), Ok(1));
}

#[test]
fn the_keys_and_the_clicks() {
    let s = one(
        "inputs = [{ at = 1, keydown = \"Home\" }, { at = 1, keyup = \"Home\" }, \
         { at = 2, keydown = \"ctrl+End\" }, { at = 3, click = [300, 200] }, \
         { at = 3, pointer = [700, 300] }]",
    );
    assert_eq!(
        s.actions().unwrap(),
        BTreeMap::from([
            (1, vec![Action::Step(Direction::NorthWest)]),
            (2, vec![Action::Scroll(Direction::SouthWest)]),
            (
                3,
                vec![Action::Click([300, 200]), Action::Pointer([700, 300])]
            ),
        ])
    );
    assert!(s.walks());
    assert!(!one("inputs = [{ at = 2, keydown = \"ctrl+Up\" }]").walks());
    assert!(one("hero_speed = 2").walks());
    assert!(one("points = 0").walks());
    assert!(one("inputs = [{ at = 2, click = [1, 1] }]").walks());
}

#[test]
fn refuses_what_breaks_a_rule() {
    for (inputs, message) in [
        (
            "{ at = 1, keydown = \"Up\", keyup = \"Up\" }",
            "s: the input at 1: not exactly one of keydown, keyup, pointer, click",
        ),
        (
            "{ at = 1 }",
            "s: the input at 1: not exactly one of keydown, keyup, pointer, click",
        ),
        (
            "{ at = 0, pointer = [1, 1] }",
            "s: the input at 0: out of order or outside 1 to 100",
        ),
        (
            "{ at = 101, pointer = [1, 1] }",
            "s: the input at 101: out of order or outside 1 to 100",
        ),
        (
            "{ at = 5, pointer = [1, 1] }, { at = 4, pointer = [1, 1] }",
            "s: the input at 4: out of order or outside 1 to 100",
        ),
        (
            "{ at = 5, keyup = \"Up\" }",
            "s: the input at 5: `Up` released, never pressed",
        ),
        (
            "{ at = 5, keydown = \"ctrl+a\" }",
            "s: the input at 5: `ctrl+a`: no key of the map's",
        ),
    ] {
        let s = one(&format!("inputs = [{inputs}]"));
        assert_eq!(s.actions(), Err(message.to_string()), "{inputs}");
    }
    assert_eq!(
        one("scroll_speed = 3").scroll_speed(),
        Err("s: the scroll speed 3 is not 0 to 2".to_string())
    );
    assert_eq!(one("").hero_speed(), Ok(2));
    assert_eq!(one("hero_speed = 4").hero_speed(), Ok(4));
    for speed in [0, 5] {
        assert_eq!(
            one(&format!("hero_speed = {speed}")).hero_speed(),
            Err(format!("s: the hero speed {speed} is not 1 to 4"))
        );
    }
    assert!(Sequences::parse("[[sequence]]\nid = 1").is_err());
}

#[test]
fn the_paths_the_clicks_showed() {
    let s = one(
        "inputs = [{ at = 10, click = [1, 1] }, { at = 20, click = [1, 1] }, \
         { at = 30, keydown = \"Up\" }, { at = 40, click = [5, 5] }]",
    );
    let recorded = "0 view 1 2 0\n10 arrows 3,4,0,12 4,4,0,1\n20 arrows 4,4,0,1\n21 arrows\n\
                    45 screen 1,1,2,2\n50 arrows 7,7,1,26\n";
    let path = |list: &[(u8, u8, u8, u16)]| -> Vec<Arrow> {
        list.iter()
            .map(|&(x, y, z, arrow)| Arrow {
                at: Pos { x, y, z },
                arrow,
            })
            .collect()
    };
    // Each click's first arrows after it, once: 10's, 20's (the rest of 10's), 40's (at 50).
    assert_eq!(
        s.paths(recorded),
        Ok(vec![
            path(&[(3, 4, 0, 12), (4, 4, 0, 1)]),
            path(&[(4, 4, 0, 1)]),
            path(&[(7, 7, 1, 26)]),
        ])
    );
    let twice = one("inputs = [{ at = 10, click = [1, 1] }, { at = 10, click = [1, 1] }]");
    assert_eq!(twice.paths(recorded).unwrap().len(), 1);
    assert_eq!(one("").paths(recorded), Ok(Vec::new()));
    for bad in [
        "x arrows 1,1,0,1",
        "5 arrows 1,1,0",
        "5 arrows 1,1,0,z",
        "5 arrows 300,1,0,1",
    ] {
        assert_eq!(
            s.paths(bad),
            Err(format!("s: the line `{bad}` does not read")),
            "{bad}"
        );
    }
}

#[test]
fn the_selected_hero() {
    let hero = |x, owner, facing| LiveHero {
        at: Pos { x, y: 3, z: 0 },
        class: 2,
        owner,
        facing,
        boat: None,
    };
    let heroes = [hero(1, 1, Facing::Up), hero(2, 0, Facing::DownLeft)];
    assert_eq!(
        selected(&heroes, 0),
        Ok(Hero {
            at: Pos { x: 2, y: 3, z: 0 },
            facing: Direction::SouthWest
        })
    );
    assert_eq!(
        selected(&heroes, 3),
        Err("0 heroes of the viewer's colour 3: a walk needs one, its selected hero".to_string())
    );
    assert_eq!(selected_index(&heroes, 0), Ok(1));
    let two = [hero(1, 0, Facing::Up), hero(2, 0, Facing::Up)];
    assert!(selected(&two, 0).is_err());
    assert!(selected_index(&two, 0).is_err());
}
