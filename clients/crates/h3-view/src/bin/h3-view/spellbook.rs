//! The hero's spell book in the viewer (E17 M5 T3a; `docs/epics/E17-design.md` §7.17): the book's
//! hero from a battle's observation, its ruleset and its names, for the window and the capture
//! set's stills (`h3_view::book` the book itself).

use h3_content::SpellCategory;
use h3_content_battle::BattleRuleset;
use h3_kernel::Handle;
use h3_protocol::wire::{Command, Manifest, Observation};
use h3_view::book::{Hero, Kind, Memory, Open, Outcome, Spell};
use h3_view::layout::BookLayout;
use h3_view::message::{MessageBox, Picture, Style};
use h3_view::tables::Tables;
use h3_view::text::{Args, Catalogue};

use crate::boxes::{Answered, Asked, Leave};

/// The forms of a spell's description by mastery (the catalogue's family `spell-text`).
const MASTERY_FORMS: [&str; 4] = ["none", "basic", "advanced", "expert"];

/// Whether the battle opens the acting hero's book in `observation`: the side to act is the
/// observation's own and the panel's spell book button is enabled (`battle_controls_state`: a human
/// side whose hero wears a book and has not cast this round, outside the tactics phase; E17 M3
/// T6a, `h3_view::panel`).
pub fn takes(observation: &Observation) -> bool {
    let Some(active) = observation.active else {
        return false;
    };
    let Some(stack) = observation.stacks.iter().find(|s| s.id == active) else {
        return false;
    };
    let side = stack.side;
    side == observation.viewer
        && !h3_view::panel::shown(
            observation,
            Some(side),
            h3_view::panel::Kept::built(side),
            "",
            [0, 1],
        )
        .state(h3_view::panel::SPELL_BOOK)
        .disabled
}

/// The book of the observation's own hero (the viewer's side's, which the battle asks to act):
/// its spells as the book shows them. The mastery is the hero's (the observation's, the hero's
/// own) unless the battlefield raises one of the spell's schools (Expert; 0x4E5080 with the field,
/// BAT-TER-005, as the engine's `mastery_at`); the cost the spell's at that mastery changed by the
/// armies' change (the observation's `cost_change`), at least 1, a free spell free (0x4E5240,
/// BAT-MAG-054, as the engine's `mana::cost`: *provisional*, the client's copy of that formula: the
/// M5 review list); the hero's mastery of each school its mastery of the book's spells of that
/// school alone (the school a spell of several shows under on the tab of all schools; a school
/// none of them names counts as none: *provisional*). None without the own hero or its view.
pub fn hero(
    observation: &Observation,
    (manifest, ruleset): (&Manifest, &BattleRuleset),
    (tables, catalogue): (&Tables, &Catalogue),
    layout: &BookLayout,
) -> Option<Hero> {
    let side = observation.sides.get(usize::from(
        observation.viewer == h3_protocol::wire::Side::Defender,
    ))?;
    let view = side.hero.as_ref()?;
    let own = observation.own_hero.as_ref()?;
    let core = &**ruleset;
    let battlefield = observation
        .battlefield
        .and_then(|b| ruleset.battlefield(Handle::from_raw(b)));
    let school_of = |school: h3_content::SchoolId| {
        let key = core.spell_schools().get(school.index())?.key.as_str();
        let name = key.rsplit(':').next().unwrap_or(key);
        layout.schools.iter().position(|s| s == name)
    };
    let mut schools = [0u8; 4];
    let mut spells = Vec::new();
    for entry in &own.spell_book {
        let Some(spell) = ruleset.spell(Handle::from_raw(entry.spell)) else {
            continue;
        };
        let kind = match spell.category {
            SpellCategory::Combat => Kind::Combat,
            SpellCategory::Adventure => Kind::Adventure,
            SpellCategory::Ability => continue,
        };
        let indices: Vec<usize> = spell.schools.iter().filter_map(|&s| school_of(s)).collect();
        if let [only] = indices[..]
            && let Some(level) = schools.get_mut(only)
        {
            *level = (*level).max(entry.mastery);
        }
        let bits = indices.iter().fold(0u8, |b, &i| b | (1 << i));
        let raised = battlefield.is_some_and(|b| b.raises(&spell.schools));
        let mastery = if raised { 3 } else { entry.mastery.min(3) };
        let base = spell.cost[usize::from(mastery)];
        let cost = if base == 0 {
            0
        } else {
            u16::try_from((i32::from(base) + i32::from(side.cost_change)).max(1))
                .unwrap_or(u16::MAX)
        };
        let key = manifest.keys.spells.get(usize::from(entry.spell))?;
        let original = tables.spells.get(key).map_or(0, |s| u32::from(s.original));
        let description = catalogue.text(
            &format!("spell-text.{key}"),
            &Args::new().context(MASTERY_FORMS[usize::from(mastery)]),
        );
        spells.push(Spell {
            handle: entry.spell,
            original,
            name: catalogue.text(&format!("spell.{key}"), &Args::new()),
            description,
            level: spell.level,
            schools: bits,
            kind,
            mastery,
            cost,
        });
    }
    Some(Hero {
        id: view.hero_type,
        mana: view.mana,
        schools,
        spells,
    })
}

/// What a capture-set situation's actions leave of the spell book over its opening (E17 M5 T3a):
/// the book open, if any, a message box over it and the battle (E17 M5 T4a), and where the mouse
/// is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Still {
    /// The book left open.
    pub open: Option<Open>,
    /// The message box left open (E17 M5 T4a).
    pub message: Option<MessageBox>,
    /// The result window a box's OK ended the battle with (E17 M5 T4b1: Retreat's, Surrender's),
    /// over the whole screen darkened.
    pub result: Option<MessageBox>,
    /// The mouse after the actions.
    pub mouse: [i32; 2],
    /// When the actions were made (ms after the opening: a situation's at once, 0); the battle
    /// behind the open book or box stands as it was then.
    pub at: u32,
}

impl Still {
    /// Whether a dialog (the book, a box) is open over the battle.
    pub const fn dialog(&self) -> bool {
        self.open.is_some() || self.message.is_some() || self.result.is_some()
    }
}

/// Lays out a message box: its text, its style, its picture (none without its fonts).
pub type Tell<'a> = dyn Fn(&str, Style, Option<&Picture>) -> Option<MessageBox> + 'a;

/// What a situation's message boxes need (E17 M5 T4a; `crate::boxes`): how a box is laid out
/// (its text, style and picture), how the side to act may leave, which of its hero's spells (by
/// handle) no hex takes now (a stack spell's or Teleport's "affects no one"), the spells'
/// pictures' DEF and size; the result window the battle ends with after a command a box's OK
/// gives (E17 M5 T4b1).
pub struct Boxes<'a> {
    /// Lays out a box (none without its fonts).
    pub tell: &'a Tell<'a>,
    /// How the side to act may leave.
    pub leave: Leave,
    /// Whether the aim's entry tells that the spell affects no one.
    pub no_one: &'a dyn Fn(u16) -> bool,
    /// The spells' pictures' DEF and its size.
    pub pictures: (String, [i32; 2]),
    /// The result window after a command (none: the battle goes on, or no window).
    pub result: &'a dyn Fn(Command) -> Option<MessageBox>,
}

/// What a box's click leads to in a still: another box (or none), or a command given.
enum Next {
    Box(Option<Asking>),
    Give(Command),
}

/// A box open in a still: its button states and what it asks.
struct Asking {
    open: h3_view::message::Open,
    asked: Asked,
}

/// A situation's `actions` played at its opening (`h3drive.py`'s `act` at the frozen clock, after
/// the mouse's move to `mouse`): `key:<k>` opens the book of the hero `hero` gives (while the
/// panel's spell book button takes it) by its key C, Surrender's or Retreat's question by S or R
/// (E17 M5 T4a, with `boxes`), or is the open box's key or the open book's; `click:<x>,<y>`
/// moves the mouse there, presses and releases (the open box's or book's) and parks it;
/// `move:<x>,<y>` moves it, over the open book its hover (0x59D6C4). The book's refusal and
/// description open their boxes over it, a spell no hex takes "affects no one" over the battle.
/// A box's OK that gives a command (Retreat's, Surrender's) ends the battle in its result window
/// (E17 M5 T4b1), which a later key or click takes as the box's. A move to the park point after a
/// click is a move as the harness's is. Nothing else may come in a situation's actions.
pub fn still(
    actions: &[String],
    mouse: [i32; 2],
    hero: impl FnOnce() -> Option<Hero>,
    (layout, catalogue): (&BookLayout, &Catalogue),
    boxes: Option<&Boxes<'_>>,
) -> Result<Still, String> {
    let mut memory = Memory::default();
    let mut hero = Some(hero);
    let mut still = Still {
        open: None,
        message: None,
        result: None,
        mouse,
        at: 0,
    };
    let mut result: Option<h3_view::message::Open> = None;
    let mut asking: Option<Asking> = None;
    let point = |arg: &str| -> Result<[i32; 2], String> {
        let (x, y) = arg
            .split_once(',')
            .ok_or_else(|| format!("`{arg}`: not x,y"))?;
        let parse = |v: &str| v.trim().parse::<i32>().map_err(|e| format!("`{arg}`: {e}"));
        Ok([parse(x)?, parse(y)?])
    };
    let moved = |still: &mut Still, memory: &mut Memory, to: [i32; 2]| {
        still.mouse = to;
        if let Some(open) = still.open.as_mut() {
            open.hover(memory, (layout, catalogue), to);
        }
    };
    // A box laid out and shown.
    let tell = |text: &str, style, picture: Option<&Picture>, asked| -> Option<Asking> {
        let message = (boxes?.tell)(text, style, picture)?;
        Some(Asking {
            open: h3_view::message::Open::new(message),
            asked,
        })
    };
    // What the book's outcome leaves: the book, a box over it, or a box over the battle.
    let booked = |still: &mut Still, outcome: Outcome| -> Option<Asking> {
        match outcome {
            Outcome::Nothing | Outcome::Turned => None,
            Outcome::Refused { .. } | Outcome::Described(_) => {
                let boxes = boxes?;
                let open = still.open.as_ref()?;
                let (def, size) = &boxes.pictures;
                let (text, picture) =
                    crate::boxes::book_box(outcome, &open.hero, catalogue, (def, *size))?;
                tell(&text, Style::Ok, picture.as_ref(), Asked::Told)
            }
            Outcome::Closed => {
                still.open = None;
                None
            }
            Outcome::Chosen(spell) => {
                still.open = None;
                let boxes = boxes?;
                (boxes.no_one)(spell)
                    .then(|| {
                        tell(
                            &crate::boxes::no_one(catalogue),
                            Style::Ok,
                            None,
                            Asked::Told,
                        )
                    })
                    .flatten()
            }
        }
    };
    // A box's button clicked: what follows (another box at once, a command, or nothing).
    let answered = |id: u16, asked: Asked| match crate::boxes::answered(asked, id, catalogue) {
        Answered::Tell(text) => Next::Box(tell(&text, Style::Ok, None, Asked::Told)),
        Answered::Give(command) => Next::Give(command),
        Answered::Closed => Next::Box(None),
    };
    // The result window a command ends the battle with.
    let ended = |command: Command| -> Option<h3_view::message::Open> {
        (boxes?.result)(command).map(h3_view::message::Open::new)
    };
    for action in actions {
        let (kind, arg) = action.split_once(':').unwrap_or((action, ""));
        match kind {
            "key" => {
                // The harness's key names (xdotool's) as the book's: a letter upper-cased.
                let key = if arg.len() == 1 {
                    arg.to_ascii_uppercase()
                } else {
                    arg.to_owned()
                };
                if let Some(r) = result.as_mut() {
                    // The result window's OK by its key: the window closed.
                    let _ = r.key(&key);
                    if let h3_view::message::Outcome::Closed(_) = r.key_up(&key) {
                        result = None;
                    }
                    continue;
                }
                if let Some(a) = asking.as_mut() {
                    // A key down and up: its button clicked.
                    let _ = a.open.key(&key);
                    if let h3_view::message::Outcome::Closed(id) = a.open.key_up(&key) {
                        let asked = a.asked;
                        match answered(id, asked) {
                            Next::Box(next) => asking = next,
                            Next::Give(command) => {
                                asking = None;
                                result = ended(command);
                            }
                        }
                    }
                    continue;
                }
                match still.open.as_mut() {
                    Some(open) => {
                        let outcome = open.key(&mut memory, &key, catalogue);
                        asking = booked(&mut still, outcome);
                    }
                    None if key == "C" => {
                        if let Some(h) = hero.take().and_then(|f| f()) {
                            still.open = Some(Open::new(&mut memory, h, catalogue));
                        }
                    }
                    None if (key == "R" || key == "S") && boxes.is_some() => {
                        let id = if key == "R" {
                            h3_view::panel::RETREAT
                        } else {
                            h3_view::panel::SURRENDER
                        };
                        let asked = boxes.and_then(|b| b.leave.ask(catalogue, id));
                        if let Some((text, asked)) = asked {
                            asking = tell(&text, Style::OkCancel, None, asked);
                        }
                    }
                    None => return Err(format!("`{action}`: only the book's keys in a situation")),
                }
            }
            "click" => {
                let to = point(arg)?;
                moved(&mut still, &mut memory, to);
                if let Some(r) = result.as_mut() {
                    let _ = r.press(to);
                    if let h3_view::message::Outcome::Closed(_) = r.release() {
                        result = None;
                    }
                } else if let Some(a) = asking.as_mut() {
                    let _ = a.open.press(to);
                    if let h3_view::message::Outcome::Closed(id) = a.open.release() {
                        let asked = a.asked;
                        match answered(id, asked) {
                            Next::Box(next) => asking = next,
                            Next::Give(command) => {
                                asking = None;
                                result = ended(command);
                            }
                        }
                    }
                } else if let Some(open) = still.open.as_mut() {
                    let outcome = open.press(&mut memory, (layout, catalogue), to);
                    asking = booked(&mut still, outcome);
                }
                // Parked: a box the click opened takes the move, the book behind it none (its
                // status line keeps the hint).
                if let Some(r) = result.as_mut() {
                    still.mouse = crate::pointer::PARK;
                    let _ = r.moved(crate::pointer::PARK);
                    continue;
                }
                match asking.as_mut() {
                    Some(a) => {
                        still.mouse = crate::pointer::PARK;
                        let _ = a.open.moved(crate::pointer::PARK);
                    }
                    None => moved(&mut still, &mut memory, crate::pointer::PARK),
                }
            }
            "move" => {
                let to = point(arg)?;
                if let Some(r) = result.as_mut() {
                    still.mouse = to;
                    let _ = r.moved(to);
                    continue;
                }
                match asking.as_mut() {
                    Some(a) => {
                        still.mouse = to;
                        let _ = a.open.moved(to);
                    }
                    None => moved(&mut still, &mut memory, to),
                }
            }
            _ => {
                return Err(format!(
                    "`{action}`: a situation's actions are key:, click: and move:"
                ));
            }
        }
    }
    still.message = asking.map(|a| a.open.message);
    still.result = result.map(|r| r.message);
    Ok(still)
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_view::layout::parse_battle;

    fn layout() -> BookLayout {
        parse_battle(include_str!("../../../../../layouts/battle.toml"))
            .unwrap()
            .book
    }

    fn hero() -> Hero {
        Hero {
            id: 1,
            mana: 10,
            schools: [0; 4],
            spells: vec![Spell {
                handle: 3,
                original: 3,
                name: "Dart".into(),
                description: "{Dart}\n\nIt hits.".into(),
                level: 1,
                schools: 1,
                kind: Kind::Combat,
                mastery: 0,
                cost: 5,
            }],
        }
    }

    fn played(actions: &[&str], hero: Option<Hero>) -> Result<Still, String> {
        let actions: Vec<String> = actions.iter().map(|a| (*a).to_owned()).collect();
        let catalogue = Catalogue::default();
        still(
            &actions,
            crate::pointer::PARK,
            || hero,
            (&layout(), &catalogue),
            None,
        )
    }

    #[test]
    fn a_situation_s_actions_open_turn_and_hover_the_book() {
        // C opens it; the mouse over place 0's icon: its hint (the catalogue's key, no pack here),
        // the book's cursor over an item.
        let still = played(&["key:c", "move:246,123"], Some(hero())).unwrap();
        assert_eq!(still.mouse, [246, 123]);
        let open = still.open.unwrap();
        assert_eq!(open.cursor, Some(1));
        assert!(open.status.starts_with("{Dart} ("));
        // The fire tab clicked: the book turned, the mouse parked; Down: the next tab.
        let still = played(&["key:c", "click:654,228", "key:Down"], Some(hero())).unwrap();
        assert_eq!(still.mouse, crate::pointer::PARK);
        assert_eq!(still.open.unwrap().book.tab, 4);
        // The close clicked, Escape, the place's spell chosen: the book closed.
        for actions in [
            &["key:c", "click:585,430"][..],
            &["key:c", "key:Escape"],
            &["key:c", "click:246,123"],
        ] {
            assert_eq!(
                played(actions, Some(hero())).unwrap().open,
                None,
                "{actions:?}"
            );
        }
        // No book to open: nothing open.
        assert_eq!(played(&["key:c"], None).unwrap().open, None);
    }

    /// A box of one button `id` at (10, 10), 64 × 30, with the hotkeys `keys`.
    fn one_button(id: u16, keys: &[&str]) -> MessageBox {
        use h3_view::ui::{Button, Dialog, Frames, Item, Shown};
        MessageBox {
            dialog: Dialog {
                place: [0, 0],
                shadow: None,
                items: vec![Item::Button(Button {
                    id,
                    def: "iOkay.def".into(),
                    at: [10, 10],
                    size: [64, 30],
                    frames: Frames::default(),
                    hint: None,
                    keys: keys.iter().map(|k| (*k).to_owned()).collect(),
                })],
            },
            shown: Shown::default(),
        }
    }

    #[test]
    fn a_box_s_command_ends_the_still_in_the_result_window() {
        // Retreat's question (its OK by Enter or a click) gives the command; the result window
        // follows, which a key or a click on its OK closes again.
        let tell = |_: &str, _, _: Option<&Picture>| {
            Some(one_button(h3_view::message::YES, &["Return"]))
        };
        let given = std::cell::RefCell::new(Vec::new());
        let result = |command| {
            given.borrow_mut().push(command);
            Some(one_button(h3_view::result::OK, &["Return", "Escape"]))
        };
        let boxes = Boxes {
            tell: &tell,
            leave: Leave {
                retreat: true,
                surrender: None,
            },
            no_one: &|_| false,
            pictures: (String::new(), [0, 0]),
            result: &result,
        };
        let catalogue = Catalogue::default();
        let played = |actions: &[&str]| {
            let actions: Vec<String> = actions.iter().map(|a| (*a).to_owned()).collect();
            still(
                &actions,
                crate::pointer::PARK,
                || None,
                (&layout(), &catalogue),
                Some(&boxes),
            )
            .unwrap()
        };
        let still = played(&["key:r", "key:Return"]);
        assert!(still.result.is_some() && still.message.is_none() && still.dialog());
        assert_eq!(*given.borrow(), [h3_protocol::wire::Command::Retreat {}]);
        assert!(played(&["key:r", "key:Return", "key:Escape"]).result.is_none());
        let still = played(&["key:r", "click:20,20", "move:30,30"]);
        assert_eq!(still.mouse, [30, 30]);
        assert!(still.result.is_some());
        let still = played(&["key:r", "click:20,20", "click:20,20"]);
        assert!(still.result.is_none() && !still.dialog());
        assert_eq!(still.mouse, crate::pointer::PARK);
    }

    #[test]
    fn a_situation_s_other_actions_are_errors() {
        assert!(played(&["key:w"], Some(hero())).is_err());
        assert!(played(&["hex:40"], Some(hero())).is_err());
        assert!(played(&["key:c", "click:1"], Some(hero())).is_err());
        assert!(played(&["move:a,2"], Some(hero())).is_err());
    }
}
