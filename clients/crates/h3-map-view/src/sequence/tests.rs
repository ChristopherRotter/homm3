use super::*;

use alloc::vec;

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
    assert!(Sequences::parse("[[sequence]]\nid = 1").is_err());
}
