//! The hero's spell book in the viewer (E17 M5 T3a; `docs/epics/E17-design.md` §7.17): the book's
//! hero from a battle's observation, its ruleset and its names, for the window and the capture
//! set's stills (`h3_view::book` the book itself).

use h3_content::SpellCategory;
use h3_content_battle::BattleRuleset;
use h3_kernel::Handle;
use h3_protocol::wire::{Command, Manifest, Observation, Side};
use h3_ui::text::{Args, Catalogue};
use h3_view::army_info::{Drawn, Hover};
use h3_view::book::{Hero, Kind, Memory, Open, Outcome, Spell};
use h3_view::keys::{Command as Do, Context, Keys};
use h3_view::layout::{BattleLayout, BookLayout};
use h3_view::message::{MessageBox, Picture, Style};
use h3_view::mouse::Legal;
use h3_view::shades::{Cells, Options as Shades};
use h3_view::sound::Tune;
use h3_view::stats::Memory as StatsMemory;
use h3_view::tables::Tables;

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
            (
                h3_view::panel::Kept::built(side),
                h3_view::panel::Autoplay::default(),
            ),
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
/// none of them names counts as none: *provisional*). A spell the ruleset or the manifest does not
/// know, or a creature's ability, is left out; one of none of the layout's schools shows on the tab
/// of all schools alone (R4-4). None without the own hero or its view.
///
/// The mastery and the cost copy the engine's (`crates/h3-battle/src/magic.rs` `mastery_at`,
/// `mana.rs` `cost`; `mastered_cost`'s tests pin the cost's cases): a change there is one here.
pub fn hero(
    observation: &Observation,
    (manifest, ruleset): (&Manifest, &BattleRuleset),
    (tables, catalogue): (&Tables, &Catalogue),
    layout: &BookLayout,
) -> Option<Hero> {
    let side = observation
        .sides
        .get(usize::from(observation.viewer == Side::Defender))?;
    let view = side.hero.as_ref()?;
    let own = observation.own_hero.as_ref()?;
    let core = &**ruleset;
    let battlefield = observation
        .battlefield
        .and_then(|b| ruleset.battlefield(Handle::from_raw(b)));
    let school_of = |school: h3_content::SchoolId| {
        let key = core.spell_schools().get(school.index())?.key.as_str();
        let name = key.rsplit(':').next().unwrap_or(key);
        // The book's four alone (its bits 1 to 8; `parse_battle` takes no other layout).
        layout.schools.iter().take(4).position(|s| s == name)
    };
    let mut schools = [0u8; 4];
    let mut spells = Vec::new();
    for entry in &own.spell_book {
        // A spell the ruleset or the manifest does not know: left out, the rest shown (R4-1).
        let (Some(spell), Some(key)) = (
            ruleset.spell(Handle::from_raw(entry.spell)),
            manifest.keys.spells.get(usize::from(entry.spell)),
        ) else {
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
            *level = (*level).max(entry.mastery.min(3));
        }
        let bits = indices.iter().fold(0u8, |b, &i| b | (1 << i));
        let raised = battlefield.is_some_and(|b| b.raises(&spell.schools));
        let mastery = if raised { 3 } else { entry.mastery.min(3) };
        let cost = mastered_cost(spell.cost[usize::from(mastery)], side.cost_change);
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

/// A spell's cost `base` at the caster's mastery changed by the armies' `change`: at least 1, a
/// free spell free (0x4E5240, BAT-MAG-054; [`hero`]'s). The engine's `cost`
/// (`crates/h3-battle/src/mana.rs`) copied: keep the two alike.
fn mastered_cost(base: u16, change: i8) -> u16 {
    if base == 0 {
        0
    } else {
        u16::try_from((i32::from(base) + i32::from(change)).max(1)).unwrap_or(u16::MAX)
    }
}

/// What `side`'s hero's cast of `spell` costs in the battle `observation` shows (E17 M5 T6b1: the
/// seat's battle on a server spends its mana by its casts, `h3_view::remote::seen`), as the book
/// prices it ([`hero`]): the observation's own hero's alone (another hero's book is unknown: 0).
pub fn cost(observation: &Observation, ruleset: &BattleRuleset, side: Side, spell: u16) -> u16 {
    let (Some(own), Some(view)) = (
        observation
            .own_hero
            .as_ref()
            .filter(|_| side == observation.viewer),
        observation.sides.get(usize::from(side == Side::Defender)),
    ) else {
        return 0;
    };
    let (Some(entry), Some(record)) = (
        own.spell_book.iter().find(|b| b.spell == spell),
        ruleset.spell(Handle::from_raw(spell)),
    ) else {
        return 0;
    };
    let raised = observation
        .battlefield
        .and_then(|b| ruleset.battlefield(Handle::from_raw(b)))
        .is_some_and(|b| b.raises(&record.schools));
    let mastery = if raised { 3 } else { entry.mastery.min(3) };
    mastered_cost(record.cost[usize::from(mastery)], view.cost_change)
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
    /// over the whole screen darkened, with its tune (E17 M5 T4b2b: its clip's).
    pub result: Option<(MessageBox, Tune)>,
    /// The mouse after the actions.
    pub mouse: [i32; 2],
    /// When the actions were made (ms after the opening: a situation's at once, 0); the battle
    /// behind the open book or box stands as it was then.
    pub at: u32,
    /// The tactics phase's message the battle opened with is still open (E17 M5 T4c2a, design
    /// §7.23): the battle behind it stands before its first turn.
    pub opening: bool,
    /// The message closed and the mouse not moved since: the battle's cursor is the pointer the
    /// first turn's start set (0x478321) and marks nothing (CLT-Q32: the turn's posted mouse move
    /// does not reach the battle).
    pub arrow: bool,
    /// Where a `move:` left the mouse over the battle, every dialog closed (none: the situation's
    /// `mouse`).
    pub over: Option<[i32; 2]>,
    /// The stats dialog a press on a stack to look at or the key T opened (E17 M6 T2a1, T2a2a),
    /// over the battle, with what the actions did to it.
    pub stats: Option<crate::stats::Open>,
    /// The message box is the right button's popup over the battle, held (E17 M6 T2b2: a
    /// turret's info): no cursor (`popup_run_held` 0x602A40).
    pub held: bool,
    /// The combat options dialog the key O or a click on the panel's Options opened (E17 M6
    /// T4a), with what the actions did to it; a box over it (its help, the no-sound box) is
    /// `message`.
    pub options: Option<h3_view::options::Open>,
    /// The options dialog closed by its key and the mouse not moved since: its arrow stays at the
    /// mouse, the battle's status line and marked hex as before (oddity 10).
    pub options_arrow: bool,
    /// The army info windows the actions left drawn over the battle (E17 M6 T4b2b1, design
    /// §12.15: shown at Creature Info's level 1 or 2), under every dialog.
    pub army: Vec<MessageBox>,
    /// Creature Info's level the battle last wrote its status line at (E17 M6 T4b2b2,
    /// `Hover::line_level`): the still's line is worded at it (the epic's review item 42).
    pub line_level: u8,
    /// The cursor shadow's list and the background as the actions left them, with the grid's and
    /// the shadows' options (E17 M6 T4c, design §12.16).
    pub cells: Cells,
}

impl Still {
    /// No action made yet, the mouse at `mouse`.
    pub const fn new(mouse: [i32; 2]) -> Self {
        Self {
            open: None,
            message: None,
            result: None,
            mouse,
            at: 0,
            opening: false,
            arrow: false,
            over: None,
            stats: None,
            held: false,
            options: None,
            options_arrow: false,
            army: Vec::new(),
            line_level: 0,
            cells: Cells::new(
                Shades {
                    grid: false,
                    cursor: false,
                    movement: false,
                },
                h3_render::Bugs::Original,
            ),
        }
    }

    /// Whether a dialog (the book, a box) is open over the battle.
    pub const fn dialog(&self) -> bool {
        self.open.is_some()
            || self.message.is_some()
            || self.result.is_some()
            || self.stats.is_some()
            || self.options.is_some()
    }
}

/// Lays out a message box: its text, its style, its picture (none without its fonts).
pub type Tell<'a> = dyn Fn(&str, Style, Option<&Picture>) -> Option<MessageBox> + 'a;

/// What a situation's message boxes need (E17 M5 T4a; `crate::boxes`): how a box is laid out
/// (its text, style and picture), how the side to act may leave, the box the aim's refusal of
/// each of its hero's spells (by handle) shows now (a stack spell's or Teleport's "affects no
/// one"; E17 M6 T5a, design §12.17: the others), the spells' pictures' DEF and size; the result
/// window the battle ends with after a command a box's OK gives (E17 M5 T4b1).
pub struct Boxes<'a> {
    /// Lays out a box (none without its fonts).
    pub tell: &'a Tell<'a>,
    /// How the side to act may leave.
    pub leave: Leave,
    /// The box the aim's refusal of the spell shows once the book closed on it, if any.
    pub refusal: &'a dyn Fn(u16) -> Option<String>,
    /// The spells' pictures' DEF and its size.
    pub pictures: (String, [i32; 2]),
    /// The result window after a command and its tune (none: the battle goes on, or no window).
    pub result: &'a dyn Fn(Command) -> Option<(MessageBox, Tune)>,
    /// The tactics phase's message the battle opens with (E17 M5 T4c2a; none: it opens without).
    pub opening: Option<String>,
    /// The stats dialog a press at a point over the battle opens in a memory's profile (E17 M6
    /// T2a1; none: the point shows no stack to look at).
    pub look: &'a dyn Fn([i32; 2], &mut StatsMemory) -> Option<crate::stats::Open>,
    /// The acting stack's stats dialog the key T opens, the mouse at a point (E17 M6 T2a2a; none
    /// in the tactics phase).
    pub acting: &'a dyn Fn([i32; 2], &mut StatsMemory) -> Option<crate::stats::Open>,
    /// The stats dialog the right button held at a point over the battle shows (E17 M6 T2b1;
    /// none: no stack on the hex there).
    pub held: &'a dyn Fn([i32; 2], &mut StatsMemory) -> Option<crate::stats::Open>,
    /// The turret's info a press at a point over the battle shows, by the left button or the
    /// right (E17 M6 T2b2; none: no turret's info there).
    pub turret: &'a dyn Fn([i32; 2], bool) -> Option<String>,
    /// The help of the panel's control the right button held at a point shows (E17 M6 T3a; none:
    /// no control there).
    pub help: &'a dyn Fn([i32; 2]) -> Option<String>,
    /// The hero's info window the right button held at a point shows (E17 M6 T3b1; none: no
    /// side's hero there).
    pub hero_info: &'a dyn Fn([i32; 2]) -> Option<MessageBox>,
    /// The combat options dialog the key O opens (none given) or a click at a point on the
    /// panel's Options (E17 M6 T4a; none: the battle takes no command, or no button there), at
    /// Creature Info's level then (E17 M6 T4b2b2).
    pub options: &'a dyn Fn(Option<[i32; 2]>, (u8, Shades)) -> Option<h3_view::options::Open>,
    /// The army info windows the battle's mouse shows (E17 M6 T4b2b1; none: none).
    pub army: Option<Army<'a>>,
}

/// What a still's army info windows need (E17 M6 T4b2b1, design §12.15): where the mouse finds
/// the heroes' rectangles, the battle at the opening, the level and each side's hero's window.
pub struct Army<'a> {
    /// The battle's layout.
    pub layout: &'a BattleLayout,
    /// The battle at the opening (turn 0).
    pub observation: &'a Observation,
    /// Its acting stack's legal commands, as the still's cursor reads them (E17 M6 T4b2b2: an
    /// attack's new slot writes the status line again).
    pub legal: &'a Legal,
    /// Creature Info's level the still starts at (`frames.toml`'s `army_info`, *provisional*:
    /// the harness's registry value; 0, `Settings::CAPTURE`'s, without it); the key F5 cycles it.
    pub level: u8,
    /// A window as the battle at the opening fills it: a side's hero's (none: no hero there) or,
    /// E17 M6 T4b2b2, a stack's (none: no rules or tables for it).
    pub window: &'a dyn Fn(Drawn) -> Option<MessageBox>,
    /// The grid's and the shadows' options the still starts at (E17 M6 T4c: `frames.toml`'s
    /// `grid`, `cursor_shadow` and `movement_shadow`, *provisional*: the harness's registry
    /// values; off without them); the keys F6 to F8 toggle them.
    pub shades: Shades,
}

/// What a box's click leads to in a still: another box (or none), or a command given.
enum Next {
    Box(Option<Box<Asking>>),
    Give(Command),
}

/// A box open in a still: its button states and what it asks.
struct Asking {
    open: h3_view::message::Open,
    asked: Asked,
}

/// A situation's `actions` played at its opening (`h3drive.py`'s `act` at the frozen clock, after
/// the mouse's move to `mouse`): `key:<k>` opens the book of the hero `hero` gives (while the
/// panel's spell book button takes it; made once, at the first C, a later C after the book closed
/// opens it again) by its key C, Surrender's or Retreat's question by S or R
/// (E17 M5 T4a, with `boxes`), or is the open box's key or the open book's; `click:<x>,<y>`
/// moves the mouse there, presses and releases (the open box's or book's) and parks it;
/// `move:<x>,<y>` moves it, over the open book its hover (0x59D6C4); `right:<x>,<y>` over the
/// battle (E17 M6 T2b1) moves it there and holds the right button down: the stack there shows its
/// stats dialog while held, else a turret's info its popup (E17 M6 T2b2), over the panel a
/// control's help (T3a; the entry's last action); a `click:` over the battle opens a stack's stats dialog or a turret's info box (E17 M6
/// T2a1, T2b2). The book's refusal and
/// description open their boxes over it, a spell the engine refuses its box over the battle (E17 M6
/// T5a: [`crate::boxes::after_book`]'s, the original's profile).
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
    // The stills compare with the original's: its hovered item is never forgotten (CLT-Q26),
    // nor the stats dialog's (CLT-Q38 (3)).
    let mut memory = Memory::new(h3_render::Bugs::Original);
    let mut stats_memory = StatsMemory::new(h3_render::Bugs::Original);
    // The hero made at the first opening, each later one (the book closed) a copy (R4-3).
    let mut make = Some(hero);
    let mut hero: Option<Hero> = None;
    let mut still = Still::new(mouse);
    let mut result: Option<(h3_view::message::Open, Tune)> = None;
    let mut asking: Option<Asking> = None;
    let opening = boxes.and_then(|b| b.opening.as_deref());
    let keys = Keys::original();
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
                let text = (boxes?.refusal)(spell)?;
                tell(&text, Style::Ok, None, Asked::Told)
            }
        }
    };
    // A box's button clicked: what follows (another box at once, a command, or nothing).
    let answered = |id: u16, asked: Asked| match crate::boxes::answered(asked, id, catalogue) {
        Answered::Tell(text) => Next::Box(tell(&text, Style::Ok, None, Asked::Told).map(Box::new)),
        Answered::Give(command) => Next::Give(command),
        Answered::Closed => Next::Box(None),
    };
    // The result window a command ends the battle with.
    let ended = |command: Command| -> Option<(h3_view::message::Open, Tune)> {
        (boxes?.result)(command).map(|(window, tune)| (h3_view::message::Open::new(window), tune))
    };
    // The tactics phase's message, open as the battle opens (E17 M5 T4c2a), the mouse over it.
    if let Some(text) = opening {
        asking = tell(text, Style::Ok, None, Asked::Told);
        still.opening = asking.is_some();
        if let Some(a) = asking.as_mut() {
            let _ = a.open.moved(mouse);
        }
    }
    // The army info windows (E17 M6 T4b2b1, *provisional*): the battle's mouse at the harness's
    // moves and presses, in the original's profile (its rectangles, CLT-Q21; its keys, CLT-Q49;
    // its held press, CLT-Q50). The harness's move at the opening reaches the battle unless the
    // tactics phase's message takes it (its close posts none that does, CLT-Q32).
    let army = boxes.and_then(|b| b.army.as_ref());
    let mut hover = Hover::new(army.map_or(0, |a| a.level));
    // The cursor shadow (E17 M6 T4c, design §12.16): the original's list, every move the battle
    // takes shading the hex under the mouse.
    let mut cells = Cells::new(
        army.map_or_else(Shades::default, |a| a.shades),
        h3_render::Bugs::Original,
    );
    // The opening's turn in the tactics phase: its shade in every composition (a block saved
    // from one keeps it).
    cells.phase(army.map_or_else(std::collections::BTreeSet::new, |a| {
        h3_view::tactics::shade(a.observation)
    }));
    let take = |hover: &mut Hover, cells: &mut Cells, to: [i32; 2]| {
        if let Some(a) = army {
            let spot = h3_view::mouse::spot(a.layout, to, true);
            cells.mouse(h3_view::mouse::aim::screen_hex(spot), false);
            hover.moved(a.layout, (a.observation, a.legal), (to, true));
        }
    };
    if !still.opening {
        take(&mut hover, &mut cells, mouse);
    }
    for action in actions {
        if still.held {
            return Err(format!(
                "`{action}`: the right button is held: nothing comes after `right:`"
            ));
        }
        // The stats dialog open (E17 M6 T2a2a): its moves, its OK's click and keys.
        if let Some(open) = still.stats.as_mut() {
            let tell = boxes.map(|b| b.tell);
            let closed = stats_action(
                open,
                &mut stats_memory,
                (action, &keys, tell),
                &mut still.mouse,
            )
            .map_err(|e| format!("`{action}`: {e}"))?;
            if let Some(rest) = closed {
                // Closed: the battle takes the mouse posted where it rests (0x476F3A), its hex
                // forgotten: the window under it comes back (E17 M6 T4b2b1).
                still.stats = None;
                still.over = Some(rest);
                still.arrow = false;
                hover.forget();
                take(&mut hover, &mut cells, rest);
            }
            continue;
        }
        // The combat options dialog open, no box over it (E17 M6 T4a): its moves, clicks, keys
        // and the right button's help.
        if let Some(open) = still.options.as_mut()
            && asking.is_none()
        {
            let outcome = options_action(open, (action, &keys), &mut still.mouse)
                .map_err(|e| format!("`{action}`: {e}"))?;
            match outcome {
                OptionsAction::Open => {}
                OptionsAction::Tell(key, style) => {
                    let text = catalogue.text(&key, &Args::new());
                    let asked = if style == Style::Popup {
                        Asked::Held
                    } else {
                        Asked::Told
                    };
                    asking = tell(&text, style, None, asked);
                    if style == Style::Popup {
                        still.held = asking.is_some();
                    }
                }
                OptionsAction::Closed { moved } => {
                    // Its level set Creature Info's (E17 M6 T4b2a): the windows as they are, the
                    // line rewritten only by the park's move (as F5, §12.14 oddity 6). The
                    // background composed again with its options, the cursor shadow's list kept
                    // (E17 M6 T4c, oddity 2: the original's).
                    hover.windows.level = open.settings.army_info;
                    cells.compose(Shades::of(&open.settings));
                    still.options = None;
                    // By a click the park's move reaches the battle; by a key the arrow stays.
                    still.options_arrow = !moved;
                    if moved {
                        still.over = Some(still.mouse);
                        take(&mut hover, &mut cells, still.mouse);
                    }
                }
            }
            continue;
        }
        // Its OK closed it: the first turn starts, the box's cursor left (CLT-Q32).
        if still.opening && asking.is_none() {
            still.opening = false;
            still.arrow = true;
        }
        let (kind, arg) = action.split_once(':').unwrap_or((action, ""));
        match kind {
            "key" => {
                // The harness's key names (xdotool's) as the adapter's: a letter upper-cased; what
                // it does the original's keys' (the table's defaults, E24 M3 T1b).
                let key = if arg.len() == 1 {
                    arg.to_ascii_uppercase()
                } else {
                    arg.to_owned()
                };
                let context = Context::of(
                    result.is_some(),
                    asking.is_some(),
                    (false, false),
                    still.open.is_some(),
                    false,
                );
                let command = keys.command(context, false, &key);
                // The keys C, S, R and O keep the army info windows in the original's profile
                // (CLT-Q49; the client's hides them, `Hover::key`).
                if let (Some(a), Context::Battle, Some(Do::Button(_))) = (army, context, command) {
                    hover.key(a.layout, still.mouse, h3_render::Bugs::Original);
                }
                match (command, still.open.as_mut()) {
                    (Some(Do::ResultOk), _) => {
                        if let Some((r, _)) = result.as_mut() {
                            // The result window's OK by its key: the window closed.
                            let _ = r.key(h3_view::message::Key::Ok, &key);
                            if let h3_view::message::Outcome::Closed(_) = r.key_up(&key) {
                                result = None;
                            }
                        }
                    }
                    (Some(Do::Message(k)), _) => {
                        if let Some(a) = asking.as_mut() {
                            // A key down and up: its button clicked.
                            let _ = a.open.key(k, &key);
                            if let h3_view::message::Outcome::Closed(id) = a.open.key_up(&key) {
                                let asked = a.asked;
                                match answered(id, asked) {
                                    Next::Box(next) => asking = next.map(|a| *a),
                                    Next::Give(command) => {
                                        asking = None;
                                        result = ended(command);
                                    }
                                }
                            }
                        }
                    }
                    (Some(Do::Book(k)), Some(open)) => {
                        let outcome = open.key(&mut memory, k, catalogue);
                        asking = booked(&mut still, outcome);
                    }
                    _ if context != Context::Battle => {}
                    // The acting stack's stats dialog (E17 M6 T2a2a), the mouse where it is.
                    (Some(Do::Stats), _) => {
                        let opened = boxes.and_then(|b| (b.acting)(still.mouse, &mut stats_memory));
                        if let Some(open) = opened {
                            // The army info windows hidden first (0x475D1E; E17 M6 T4b2b1).
                            hover.windows.hide();
                            still.stats = Some(open);
                        }
                    }
                    // Creature Info's level cycled (E17 M6 T4b2a; the windows as they are until
                    // a new hex, E17 M6 T4b2b1).
                    (Some(Do::ArmyInfo), _) => hover.cycle(),
                    // The grid or a shadow toggled (E17 M6 T4c, design §12.16): the shade off, the
                    // background composed again, the mouse's hex forgotten and the mouse posted
                    // where it rests (the opening's turn a human's).
                    (Some(Do::Shades(toggle)), _) => {
                        cells.set(cells.options().toggled(toggle));
                        hover.forget();
                        take(&mut hover, &mut cells, still.mouse);
                    }
                    // The combat options dialog (E17 M6 T4a), the mouse where it is.
                    (Some(Do::Button(h3_view::panel::OPTIONS)), _) => {
                        let level = hover.windows.level;
                        let shades = cells.options();
                        still.options = boxes.and_then(|b| (b.options)(None, (level, shades)));
                        still.options_arrow = false;
                    }
                    (Some(Do::Button(h3_view::panel::SPELL_BOOK)), _) => {
                        if let Some(f) = make.take() {
                            hero = f();
                        }
                        if let Some(h) = hero.clone() {
                            still.open = Some(Open::new(&mut memory, h, catalogue));
                        }
                    }
                    (Some(Do::Button(id)), _)
                        if boxes.is_some()
                            && (id == h3_view::panel::RETREAT
                                || id == h3_view::panel::SURRENDER) =>
                    {
                        let asked = boxes.and_then(|b| b.leave.ask(catalogue, id));
                        if let Some((text, asked)) = asked {
                            asking = tell(&text, Style::OkCancel, None, asked);
                        }
                    }
                    _ => return Err(format!("`{action}`: only the book's keys in a situation")),
                }
            }
            "click" => {
                let to = point(arg)?;
                moved(&mut still, &mut memory, to);
                // Over the battle (no box, no book): its move, then the left press over the
                // battlefield hiding the army info windows, the hex kept (E17 M6 T4b2b1).
                if result.is_none() && asking.is_none() && still.open.is_none() {
                    take(&mut hover, &mut cells, to);
                    if let Some(a) = army {
                        hover.pressed(a.layout, to);
                    }
                }
                if let Some((r, _)) = result.as_mut() {
                    let _ = r.press(to);
                    if let h3_view::message::Outcome::Closed(_) = r.release() {
                        result = None;
                    }
                } else if let Some(a) = asking.as_mut() {
                    let _ = a.open.press(to);
                    if let h3_view::message::Outcome::Closed(id) = a.open.release() {
                        let asked = a.asked;
                        match answered(id, asked) {
                            Next::Box(next) => asking = next.map(|a| *a),
                            Next::Give(command) => {
                                asking = None;
                                result = ended(command);
                            }
                        }
                    }
                } else if let Some(open) = still.open.as_mut() {
                    let outcome = open.press(&mut memory, (layout, catalogue), to);
                    asking = booked(&mut still, outcome);
                } else if let Some(mut stats) = boxes.and_then(|b| (b.look)(to, &mut stats_memory))
                {
                    // A stack to look at: its stats dialog, modal; the park's move reaches it
                    // over none of its items (below the dialog at its lowest).
                    still.mouse = crate::pointer::PARK;
                    stats.modal.hover(&mut stats_memory, crate::pointer::PARK);
                    still.stats = Some(stats);
                    continue;
                } else if let Some(text) = boxes.and_then(|b| (b.turret)(to, false)) {
                    // A turret's info in its box (E17 M6 T2b2), which takes the park's move; the
                    // battle's status line keeps the press's hint.
                    asking = tell(&text, Style::Ok, None, Asked::Turret);
                    still.over = Some(to);
                } else if let Some(open) = boxes
                    .and_then(|b| (b.options)(Some(to), (hover.windows.level, cells.options())))
                {
                    // The panel's Options (E17 M6 T4a): the dialog, modal, the park's move its.
                    still.options = Some(open);
                    still.options_arrow = false;
                    still.mouse = crate::pointer::PARK;
                    if let Some(o) = still.options.as_mut() {
                        o.moved(crate::pointer::PARK);
                    }
                    continue;
                }
                // The tactics phase's message closed: the first turn starts (CLT-Q32).
                if still.opening && asking.is_none() {
                    still.opening = false;
                    still.arrow = true;
                }
                // Parked: a box the click opened takes the move, the book behind it none (its
                // status line keeps the hint); over the battle the move reaches it.
                if let Some((r, _)) = result.as_mut() {
                    still.mouse = crate::pointer::PARK;
                    let _ = r.moved(crate::pointer::PARK);
                    continue;
                }
                match asking.as_mut() {
                    Some(a) => {
                        still.mouse = crate::pointer::PARK;
                        let _ = a.open.moved(crate::pointer::PARK);
                    }
                    None => {
                        moved(&mut still, &mut memory, crate::pointer::PARK);
                        if still.open.is_none() {
                            take(&mut hover, &mut cells, crate::pointer::PARK);
                        }
                        if still.arrow {
                            still.over = Some(crate::pointer::PARK);
                            still.arrow = false;
                        }
                    }
                }
            }
            // The right button held over the battle (E17 M6 T2b1, design §12.5): the stack on
            // the hex there shows its stats dialog while held, through the frames; a turret its
            // info (T2b2), a panel's control its help (T3a, §12.7), a hero its info window
            // (T3b1, §12.8).
            "right" if asking.is_none() && result.is_none() && still.open.is_none() => {
                let to = point(arg)?;
                moved(&mut still, &mut memory, to);
                still.over = Some(to);
                still.arrow = false;
                take(&mut hover, &mut cells, to);
                if let Some(open) = boxes.and_then(|b| (b.held)(to, &mut stats_memory)) {
                    still.stats = Some(open);
                } else if let Some(window) = boxes.and_then(|b| (b.hero_info)(to)) {
                    // Held through the frames; the original's shows nothing when its window is
                    // shown already at level 0 (CLT-Q50), the cursor hidden all the same (E17 M6
                    // T4b2b1).
                    let side = army.and_then(|a| hero_side(a, to));
                    let shows = side
                        .is_none_or(|s| (hover.windows).held_shows(s, h3_render::Bugs::Original));
                    if let Some(s) = side {
                        hover.windows.held(s, 0);
                    }
                    asking = shows.then(|| Asking {
                        open: h3_view::message::Open::new(window),
                        asked: Asked::Held,
                    });
                    still.held = true;
                } else if let Some(text) =
                    boxes.and_then(|b| (b.turret)(to, true).or_else(|| (b.help)(to)))
                {
                    // Else a turret's info in its popup (E17 M6 T2b2), or over the panel a
                    // control's help (T3a).
                    asking = tell(&text, Style::Popup, None, Asked::Held);
                    still.held = asking.is_some();
                }
            }
            // The right button pressed over a hero's rectangle and released at once (E17 M6
            // T4b2b1, *provisional*: the harness's `right-click:`): its window shown while held,
            // then kept at Creature Info's levels 1 and 2, hidden at 0 (design §12.15).
            "right-click" if asking.is_none() && result.is_none() && still.open.is_none() => {
                let to = point(arg)?;
                moved(&mut still, &mut memory, to);
                still.over = Some(to);
                still.arrow = false;
                take(&mut hover, &mut cells, to);
                let side = army
                    .and_then(|a| {
                        hero_side(a, to).filter(|&s| (a.window)(Drawn::Hero(s, 0)).is_some())
                    })
                    .ok_or_else(|| format!("`{action}`: only over a hero's rectangle"))?;
                hover.windows.held(side, 0);
            }
            "move" => {
                let to = point(arg)?;
                if let Some((r, _)) = result.as_mut() {
                    still.mouse = to;
                    let _ = r.moved(to);
                    continue;
                }
                match asking.as_mut() {
                    Some(a) => {
                        still.mouse = to;
                        let _ = a.open.moved(to);
                    }
                    None => {
                        moved(&mut still, &mut memory, to);
                        if still.open.is_none() {
                            still.over = Some(to);
                            still.arrow = false;
                            // The options dialog's arrow goes with the move (E17 M6 T4c: the
                            // dialog's witness of §12.16's oddity 2 moves after its key's close).
                            still.options_arrow = false;
                            take(&mut hover, &mut cells, to);
                        }
                    }
                }
            }
            _ => {
                return Err(format!(
                    "`{action}`: a situation's actions are key:, click:, move:, right: and \
                     right-click: (over the battle)"
                ));
            }
        }
    }
    if still.opening && asking.is_none() {
        still.opening = false;
        still.arrow = true;
    }
    still.message = asking.map(|a| a.open.message);
    still.result = result.map(|(r, tune)| (r.message, tune));
    // The army info windows the battle's mouse left shown, drawn while the level is not 0, and
    // the level the status line was last written at.
    if let Some(a) = army {
        still.army = hover.windows.drawn().filter_map(a.window).collect();
    }
    still.line_level = hover.line_level;
    still.cells = cells;
    Ok(still)
}

/// The side whose hero's rectangle `point` is over in `army`'s battle (the original's
/// rectangles).
fn hero_side(army: &Army<'_>, point: [i32; 2]) -> Option<Side> {
    match h3_view::mouse::spot(army.layout, point, true) {
        h3_view::mouse::Spot::Hero(side) => Some(side),
        _ => None,
    }
}

/// What one of a situation's actions did to the combat options dialog (E17 M6 T4a).
#[derive(Debug, Clone, PartialEq, Eq)]
enum OptionsAction {
    /// It stays open.
    Open,
    /// A box over it: its text's key and style (the no-sound box, an OK; a control's help, held).
    Tell(String, Style),
    /// The OK closed it, by a click (the park's move then reaches the battle) or by its key.
    Closed {
        /// The mouse moved as it closed.
        moved: bool,
    },
}

/// One of a situation's actions over the open combat options dialog `open` (E17 M6 T4a, design
/// §12.12): `move:<x>,<y>` moves the mouse there (a button held following); `click:<x>,<y>` moves
/// it, presses, releases and parks it; `key:<k>` presses the dialog's key (its OK's) and releases
/// it; `right:<x>,<y>` moves it there and holds the right button down over a control: its help,
/// held through the frames. The mouse is `mouse`.
fn options_action(
    open: &mut h3_view::options::Open,
    (action, keys): (&str, &Keys),
    mouse: &mut [i32; 2],
) -> Result<OptionsAction, String> {
    let (kind, arg) = action.split_once(':').unwrap_or((action, ""));
    let point = |arg: &str| -> Result<[i32; 2], String> {
        let (x, y) = arg.split_once(',').ok_or("not x,y")?;
        let parse = |v: &str| v.trim().parse::<i32>().map_err(|e| e.to_string());
        Ok([parse(x)?, parse(y)?])
    };
    let did = match kind {
        "move" => {
            *mouse = point(arg)?;
            open.moved(*mouse)
        }
        "click" => {
            *mouse = point(arg)?;
            open.moved(*mouse);
            open.press(*mouse);
            let did = open.release();
            *mouse = crate::pointer::PARK;
            if !did.closed {
                open.moved(*mouse);
            }
            if did.closed {
                return Ok(OptionsAction::Closed { moved: true });
            }
            did
        }
        "right" => {
            *mouse = point(arg)?;
            open.moved(*mouse);
            return Ok(match open.help_at(*mouse) {
                Some(key) => OptionsAction::Tell(key, Style::Popup),
                None => OptionsAction::Open,
            });
        }
        "key" => {
            let key = if arg.len() == 1 {
                arg.to_ascii_uppercase()
            } else {
                arg.to_owned()
            };
            if keys.command(Context::Options, false, &key) != Some(Do::OptionsOk) {
                return Err(String::from("not a key of the combat options dialog"));
            }
            open.key(&key);
            if open.key_up(&key).closed {
                return Ok(OptionsAction::Closed { moved: false });
            }
            return Ok(OptionsAction::Open);
        }
        _ => {
            return Err(String::from(
                "over the combat options dialog only key:, click:, move: and right:",
            ));
        }
    };
    Ok(if did.no_sound {
        OptionsAction::Tell(String::from(h3_view::options::NO_SOUND), Style::Ok)
    } else {
        OptionsAction::Open
    })
}

/// One of a situation's actions over an open stats dialog `open` (E17 M6 T2a2a): `move:<x>,<y>`
/// moves the mouse there (its hint, the OK held following); `click:<x>,<y>` moves it, presses,
/// releases and parks it; `key:<k>` presses the dialog's key (its OK's) and releases it;
/// `right:<x>,<y>` (E17 M6 T2a2b1) moves it there and holds the right button down: the item's
/// description in its popup (laid out by `tell`), held through the frames, so nothing may come
/// after it. Where the mouse rests once the OK closed the dialog (none: still open; the `mouse`
/// kept).
fn stats_action(
    open: &mut crate::stats::Open,
    memory: &mut StatsMemory,
    (action, keys, tell): (&str, &Keys, Option<&Tell<'_>>),
    mouse: &mut [i32; 2],
) -> Result<Option<[i32; 2]>, String> {
    use h3_view::message::Outcome;
    let modal = &mut open.modal;
    if modal.popup.is_some() || open.held {
        return Err(String::from(
            "the right button is held: nothing comes after `right:`",
        ));
    }
    let (kind, arg) = action.split_once(':').unwrap_or((action, ""));
    let point = |arg: &str| -> Result<[i32; 2], String> {
        let (x, y) = arg.split_once(',').ok_or("not x,y")?;
        let parse = |v: &str| v.trim().parse::<i32>().map_err(|e| e.to_string());
        Ok([parse(x)?, parse(y)?])
    };
    let mut to = |modal: &mut h3_view::stats::Modal, at: [i32; 2]| {
        *mouse = at;
        modal.hover(memory, at);
        modal.moved(at);
    };
    match kind {
        "move" => {
            to(modal, point(arg)?);
            Ok(None)
        }
        "click" => {
            let at = point(arg)?;
            to(modal, at);
            modal.press(at);
            if modal.release() == Outcome::Closed(h3_view::stats::OK) {
                // The battle takes the press's point posted, then the park's move.
                *mouse = crate::pointer::PARK;
                return Ok(Some(crate::pointer::PARK));
            }
            to(modal, crate::pointer::PARK);
            Ok(None)
        }
        "right" => {
            let at = point(arg)?;
            to(modal, at);
            if let Some((text, picture)) = modal.describe(at) {
                modal.popup = tell.and_then(|tell| tell(&text, Style::Popup, picture.as_ref()));
                // Held from the opening's frozen time through the frames.
                open.holds.push([0, u32::MAX]);
            }
            Ok(None)
        }
        "key" => {
            let key = if arg.len() == 1 {
                arg.to_ascii_uppercase()
            } else {
                arg.to_owned()
            };
            match keys.command(Context::Stats, false, &key) {
                Some(Do::StatsOk) => {
                    modal.key(&key);
                    Ok((modal.key_up(&key) == Outcome::Closed(h3_view::stats::OK))
                        .then_some(*mouse))
                }
                _ => Err(String::from("not a key of the stats dialog")),
            }
        }
        _ => Err(String::from(
            "over the stats dialog only key:, click:, move: and right:",
        )),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use h3_view::army_info::Kind as WindowKind;
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

    #[test]
    fn a_second_c_after_the_book_closed_opens_it_again() {
        // R4-3: the hero made once, at the first C; reopened, the book shows it again.
        let made = std::cell::Cell::new(0);
        let actions: Vec<String> = ["key:c", "key:Escape", "key:c", "key:Escape", "key:c"]
            .iter()
            .map(|a| (*a).to_owned())
            .collect();
        let catalogue = Catalogue::default();
        let make = || {
            made.set(made.get() + 1);
            Some(hero())
        };
        let still = still(
            &actions,
            crate::pointer::PARK,
            make,
            (&layout(), &catalogue),
            None,
        )
        .unwrap();
        assert_eq!(still.open.map(|o| o.hero), Some(hero()));
        assert_eq!(made.get(), 1);
    }

    #[test]
    fn a_cost_changed_stays_at_least_1_and_a_free_spell_free() {
        // R4-2: as the engine's `mana::cost` (BAT-MAG-054).
        assert_eq!(mastered_cost(0, -2), 0);
        assert_eq!(mastered_cost(0, 3), 0);
        assert_eq!(mastered_cost(3, -5), 1);
        assert_eq!(mastered_cost(5, 2), 7);
        assert_eq!(mastered_cost(5, 0), 5);
        assert_eq!(mastered_cost(u16::MAX, 1), u16::MAX);
    }

    /// The book situation's battle at its opening (`situations/book.toml`: Orrin's Champions to
    /// act, Red human): its ruleset, the attacker's own observation and its record's manifest.
    pub(crate) fn opening() -> (BattleRuleset, Observation, Manifest) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (ruleset, setup) =
            crate::scenario::battle(&dir.join("situations/book.toml"), &dir.join("../packs"), 1)
                .unwrap();
        let (observation, record) = {
            let mut live = crate::live::Live::new(&ruleset, setup, 1, [true, false]).unwrap();
            (live.own_view().unwrap(), live.record().unwrap())
        };
        let manifest = h3_view::record::parse(&record).unwrap().header.manifest;
        (ruleset, observation, manifest)
    }

    #[test]
    fn the_book_s_hero_is_the_own_hero_s_at_the_battlefield_s_masteries() {
        // R4-2: `hero` and `cost` on the book situation (its school skills Expert Air, Basic
        // Fire; 20 combat spells and two adventure ones).
        let (ruleset, mut observation, mut manifest) = opening();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let catalogue = Catalogue::default();
        let layout = layout();
        let handle = |key: &str| ruleset.spell_by_key(key).unwrap().raw();
        let (haste, slow, arrow) = (
            handle("base:haste"),
            handle("base:slow"),
            handle("base:magic_arrow"),
        );
        let made = |observation: &Observation, manifest: &Manifest| {
            super::hero(
                observation,
                (manifest, &ruleset),
                (&tables, &catalogue),
                &layout,
            )
        };
        let of = |hero: &Hero, handle: u16| -> Spell {
            hero.spells
                .iter()
                .find(|s| s.handle == handle)
                .unwrap()
                .clone()
        };
        let base = |handle: u16| ruleset.spell(Handle::from_raw(handle)).unwrap().cost;
        let attacker = Side::Attacker;
        let book = made(&observation, &manifest).unwrap();
        assert_eq!((book.mana, book.spells.len()), (20, 22));
        // The school levels from the spells of one school alone: Magic Arrow (all four, Expert)
        // raises none.
        assert_eq!(book.schools, [3, 1, 0, 0]);
        assert_eq!(
            (of(&book, arrow).schools, of(&book, arrow).mastery),
            (0xF, 3)
        );
        let kinds = |hero: &Hero, kind| hero.spells.iter().filter(|s| s.kind == kind).count();
        assert_eq!(kinds(&book, Kind::Adventure), 2);
        // Slow unskilled: its cost at no mastery, the armies' change added; `cost` the same.
        observation.sides[0].cost_change = 2;
        let book = made(&observation, &manifest).unwrap();
        let slowed = of(&book, slow);
        assert_eq!((slowed.mastery, slowed.schools), (0, 8));
        assert_eq!(slowed.cost, base(slow)[0] + 2);
        assert_eq!(cost(&observation, &ruleset, attacker, slow), slowed.cost);
        // Magic Plains raise every school: Slow at Expert, the school levels the hero's still.
        let plains = ruleset.battlefield_by_key("base:magic_plains").unwrap();
        observation.battlefield = Some(plains.raw());
        let raised = made(&observation, &manifest).unwrap();
        assert_eq!(of(&raised, slow).mastery, 3);
        assert_eq!(of(&raised, slow).cost, base(slow)[3] + 2);
        assert_eq!(
            cost(&observation, &ruleset, attacker, slow),
            base(slow)[3] + 2
        );
        assert_eq!(raised.schools, [3, 1, 0, 0]);
        // A mastery past Expert counts as Expert (the cost's and the description's index).
        observation.battlefield = None;
        let own = observation.own_hero.as_mut().unwrap();
        own.spell_book = vec![
            h3_protocol::wire::BookSpell {
                spell: slow,
                mastery: 9,
            },
            h3_protocol::wire::BookSpell {
                spell: haste,
                mastery: 3,
            },
        ];
        let past = made(&observation, &manifest).unwrap();
        assert_eq!(of(&past, slow).mastery, 3);
        assert_eq!(of(&past, slow).cost, base(slow)[3] + 2);
        assert_eq!(
            cost(&observation, &ruleset, attacker, slow),
            base(slow)[3] + 2
        );
        assert_eq!(past.schools, [3, 0, 0, 3]);
        // A creature's ability, a spell the ruleset does not know: left out.
        let gaze = handle("base:stone_gaze");
        let own = observation.own_hero.as_mut().unwrap();
        for spell in [gaze, u16::MAX] {
            own.spell_book
                .push(h3_protocol::wire::BookSpell { spell, mastery: 0 });
        }
        assert_eq!(made(&observation, &manifest).unwrap().spells.len(), 2);
        // R4-1: a spell the manifest does not know left out, its school none; the rest shown.
        assert!(haste < slow);
        manifest.keys.spells.truncate(usize::from(slow));
        let short = made(&observation, &manifest).unwrap();
        let handles: Vec<u16> = short.spells.iter().map(|s| s.handle).collect();
        assert_eq!((handles, short.schools), (vec![haste], [3, 0, 0, 0]));
        // The other side's hero's cost, a spell not in the book: unknown, 0.
        let defender = Side::Defender;
        assert_eq!(cost(&observation, &ruleset, defender, slow), 0);
        assert_eq!(cost(&observation, &ruleset, attacker, arrow), 0);
        // No own hero: no book.
        observation.own_hero = None;
        assert_eq!(made(&observation, &manifest), None);
    }

    /// A box of one button `id` at (10, 10), 64 × 30 (its keys the table's).
    fn one_button(id: u16) -> MessageBox {
        use h3_ui::ui::{Button, Dialog, Frames, Item, Shown};
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
                })],
            },
            shown: Shown::default(),
        }
    }

    #[test]
    fn a_box_s_command_ends_the_still_in_the_result_window() {
        // Retreat's question (its OK by Enter or a click) gives the command; the result window
        // follows, which a key or a click on its OK closes again.
        let tell = |_: &str, _, _: Option<&Picture>| Some(one_button(h3_view::message::YES));
        let given = std::cell::RefCell::new(Vec::new());
        let result = |command| {
            given.borrow_mut().push(command);
            Some((one_button(h3_view::result::OK), Tune::Retreat))
        };
        let boxes = Boxes {
            tell: &tell,
            leave: Leave {
                retreat: true,
                surrender: None,
            },
            refusal: &|_| None,
            pictures: (String::new(), [0, 0]),
            result: &result,
            opening: None,
            look: &|_, _| None,
            acting: &|_, _| None,
            held: &|_, _| None,
            turret: &|_, _| None,
            help: &|_| None,
            hero_info: &|_| None,
            options: &|_, _| None,
            army: None,
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
        assert_eq!(still.result.map(|(_, tune)| tune), Some(Tune::Retreat));
        assert_eq!(*given.borrow(), [Command::Retreat {}]);
        assert!(
            played(&["key:r", "key:Return", "key:Escape"])
                .result
                .is_none()
        );
        let still = played(&["key:r", "click:20,20", "move:30,30"]);
        assert_eq!(still.mouse, [30, 30]);
        assert!(still.result.is_some());
        let still = played(&["key:r", "click:20,20", "click:20,20"]);
        assert!(still.result.is_none() && !still.dialog());
        assert_eq!(still.mouse, crate::pointer::PARK);
    }

    #[test]
    fn a_turret_s_info_opens_by_a_click_or_held_by_the_right_button() {
        // E17 M6 T2b2: a click where the turret's info shows opens its box (the park's move its
        // own, the battle's hint the press's); the right button held its popup, no cursor, the
        // entry's last action. T3a: over a panel's control the right button holds its help, a
        // click there none.
        let tell = |_: &str, style, _: Option<&Picture>| {
            let id = if style == Style::Popup {
                0
            } else {
                h3_view::message::OK
            };
            Some(one_button(id))
        };
        let turret = |point: [i32; 2], _: bool| (point == [770, 250]).then(|| String::from("info"));
        let help = |point: [i32; 2]| (point == [700, 570]).then(|| String::from("help"));
        let boxes = Boxes {
            tell: &tell,
            leave: Leave::default(),
            refusal: &|_| None,
            pictures: (String::new(), [0, 0]),
            result: &|_| None,
            opening: None,
            look: &|_, _| None,
            acting: &|_, _| None,
            held: &|_, _| None,
            turret: &turret,
            help: &help,
            hero_info: &|_| None,
            options: &|_, _| None,
            army: None,
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
        };
        let clicked = played(&["click:770,250"]).unwrap();
        assert!(clicked.message.is_some() && !clicked.held);
        assert_eq!(
            (clicked.over, clicked.mouse),
            (Some([770, 250]), crate::pointer::PARK)
        );
        assert!(
            played(&["click:770,250", "click:20,20"])
                .unwrap()
                .message
                .is_none()
        );
        assert!(played(&["click:600,40"]).unwrap().message.is_none());
        let held = played(&["right:770,250"]).unwrap();
        assert!(held.message.is_some() && held.held);
        assert_eq!(held.over, Some([770, 250]));
        assert!(played(&["right:770,250", "move:5,5"]).is_err());
        let nothing = played(&["right:600,40"]).unwrap();
        assert!(nothing.message.is_none() && !nothing.held);
        let helped = played(&["right:700,570"]).unwrap();
        assert!(helped.message.is_some() && helped.held);
        assert_eq!(helped.over, Some([700, 570]));
        assert!(played(&["click:700,570"]).unwrap().message.is_none());
    }

    #[test]
    fn the_tactics_phase_s_message_is_open_at_the_opening_until_its_ok() {
        // E17 M5 T4c2a: the battle stands before its first turn behind it; its OK by Enter or a
        // click starts the turn, no result follows.
        let tell = |_: &str, _, _: Option<&Picture>| Some(one_button(h3_view::message::OK));
        let boxes = Boxes {
            tell: &tell,
            leave: Leave::default(),
            refusal: &|_| None,
            pictures: (String::new(), [0, 0]),
            result: &|_| None,
            opening: Some("{Tactics Phase}".into()),
            look: &|_, _| None,
            acting: &|_, _| None,
            held: &|_, _| None,
            turret: &|_, _| None,
            help: &|_| None,
            hero_info: &|_| None,
            options: &|_, _| None,
            army: None,
        };
        let catalogue = Catalogue::default();
        let played = |actions: &[&str]| {
            let actions: Vec<String> = actions.iter().map(|a| (*a).to_owned()).collect();
            still(
                &actions,
                [300, 300],
                || None,
                (&layout(), &catalogue),
                Some(&boxes),
            )
            .unwrap()
        };
        let open = played(&[]);
        assert!(open.opening && open.message.is_some() && open.dialog());
        assert_eq!(open.mouse, [300, 300]);
        for actions in [
            &["key:Return"][..],
            &["click:20,20"],
            &["key:Return", "move:5,5"],
        ] {
            let closed = played(actions);
            assert!(!closed.opening && !closed.dialog(), "{actions:?}");
        }
        // Closed, the battle's cursor stays the box's until the mouse moves (CLT-Q32).
        let closed = played(&["key:Return"]);
        assert!(closed.arrow && closed.over.is_none());
        let moved = played(&["key:Return", "move:5,5"]);
        assert_eq!(
            (moved.arrow, moved.over, moved.mouse),
            (false, Some([5, 5]), [5, 5])
        );
        // Its OK clicked: the park move after it reaches the battle.
        let clicked = played(&["click:20,20"]);
        assert_eq!(clicked.mouse, crate::pointer::PARK);
        assert_eq!(
            (clicked.arrow, clicked.over),
            (false, Some(crate::pointer::PARK))
        );
        // A key the box does not take leaves it open.
        assert!(played(&["key:Up"]).opening);
    }

    #[test]
    fn a_situation_s_other_actions_are_errors() {
        assert!(played(&["key:w"], Some(hero())).is_err());
        assert!(played(&["hex:40"], Some(hero())).is_err());
        assert!(played(&["key:c", "click:1"], Some(hero())).is_err());
        assert!(played(&["move:a,2"], Some(hero())).is_err());
    }

    #[test]
    fn a_still_shows_the_army_info_windows_at_its_level() {
        // E17 M6 T4b2b1 (design §12.15's stills): `hero-info`'s battle at the level the still
        // asks (`frames.toml`'s `army_info`); the windows by name: "A" and "D".
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let text = crate::scenario::record_of(
            &root.join("situations/hero-info.toml"),
            &root.join("../packs"),
            1,
            h3_protocol::wire::Seat::Attacker,
        )
        .unwrap();
        let record = h3_view::record::parse(&text).unwrap();
        let observation = record.first_snapshot().unwrap();
        let battle = parse_battle(include_str!("../../../../../layouts/battle.toml")).unwrap();
        // A stack's window (E17 M6 T4b2b2) by its id and kind: 100 + the id, 200 + it for kind 2.
        let stack_id = |id: u8, kind: WindowKind| {
            u16::from(id) + if kind == WindowKind::All { 100 } else { 200 }
        };
        let window = |drawn: Drawn| {
            Some(one_button(match drawn {
                Drawn::Hero(Side::Attacker, _) => 1,
                Drawn::Hero(Side::Defender, _) => 2,
                Drawn::Stack(s) => stack_id(s.stack, s.kind),
            }))
        };
        let hero_info = |point: [i32; 2]| match h3_view::mouse::spot(&battle, point, true) {
            h3_view::mouse::Spot::Hero(side) => window(Drawn::Hero(side, 0)),
            _ => None,
        };
        let tell = |_: &str, _, _: Option<&Picture>| None;
        let catalogue = Catalogue::default();
        let none = Legal::default();
        // The combat options dialog by the key O or the panel's Options, at the level then.
        let options = |point: Option<[i32; 2]>, (level, shades): (u8, Shades)| {
            let on_button = point.is_none_or(|p| {
                battle.panel_in(observation).button_at(p) == Some(h3_view::panel::OPTIONS)
            });
            on_button.then(|| {
                let mut settings = h3_view::options::Settings {
                    army_info: level,
                    ..h3_view::options::Settings::CAPTURE
                };
                shades.set_in(&mut settings);
                h3_view::options::Open::new(&battle.options, &catalogue, settings, (0, false))
            })
        };
        let played_with = |(level, shades): (u8, Shades), mouse: [i32; 2], actions: &[&str]| {
            let boxes = Boxes {
                tell: &tell,
                leave: Leave {
                    retreat: false,
                    surrender: None,
                },
                refusal: &|_| None,
                pictures: (String::new(), [0, 0]),
                result: &|_| None,
                opening: None,
                look: &|_, _| None,
                acting: &|_, _| None,
                held: &|_, _| None,
                turret: &|_, _| None,
                help: &|_| None,
                hero_info: &hero_info,
                options: &options,
                army: Some(Army {
                    layout: &battle,
                    observation,
                    legal: &none,
                    level,
                    window: &window,
                    shades,
                }),
            };
            let actions: Vec<String> = actions.iter().map(|a| (*a).to_owned()).collect();
            still(
                &actions,
                mouse,
                || None,
                (&layout(), &catalogue),
                Some(&boxes),
            )
        };
        let played = |level: u8, mouse: [i32; 2], actions: &[&str]| {
            played_with((level, Shades::default()), mouse, actions)
        };
        let ids = |still: &Still| -> Vec<u16> {
            (still.army.iter())
                .filter_map(|w| match w.dialog.items.first() {
                    Some(h3_ui::ui::Item::Button(b)) => Some(b.id),
                    _ => None,
                })
                .collect()
        };
        let (orrin, crag) = ([30, 70], [770, 70]);
        // Resting on a hero: its window at levels 1 and 2, none at 0.
        assert_eq!(ids(&played(1, orrin, &[]).unwrap()), [1]);
        assert_eq!(ids(&played(2, crag, &[]).unwrap()), [2]);
        assert_eq!(ids(&played(0, orrin, &[]).unwrap()), [0; 0]);
        // From the hero to an empty hex: none.
        assert_eq!(ids(&played(1, orrin, &["move:400,300"]).unwrap()), [0; 0]);
        // A left press over the enemy's hero (nothing; the click's park leaves the battlefield),
        // then the right press there: its window shown (the move back a new hex shows it by
        // itself) and kept after the release. The press showing again a window a left press hid
        // with the mouse still on the rectangle: `mouse::track`'s tests.
        let still = played(1, crag, &["click:770,70", "right-click:770,70"]).unwrap();
        assert_eq!(ids(&still), [2]);
        assert!(still.message.is_none() && !still.held);
        assert_eq!(ids(&played(1, crag, &["click:770,70"]).unwrap()), [0; 0]);
        assert_eq!(
            ids(&played(0, crag, &["right-click:770,70"]).unwrap()),
            [0; 0]
        );
        assert!(played(1, crag, &["right-click:400,300"]).is_err());
        // F5 to level 0 under Orrin's window: the original's right press shows nothing, held
        // (CLT-Q50); a window not shown before shows.
        let still = played(1, orrin, &["key:F5", "key:F5", "right:30,70"]).unwrap();
        assert_eq!(ids(&still), [0; 0]);
        assert!(still.message.is_none() && still.held);
        let still = played(0, orrin, &["right:30,70"]).unwrap();
        assert!(still.message.is_some() && still.held);
        // At level 1 the held window and the hover's: both.
        let still = played(1, orrin, &["right:30,70"]).unwrap();
        assert!(still.message.is_some() && ids(&still) == [1]);
        // E17 M6 T4b2b2: resting on a stack, its window by the level (kind 1 at 1, kind 2 at 2),
        // none at 0, the enemy's too; the status line worded at the level of the last new hex
        // (F5 after the move rewrites nothing: the window's kind kept as well).
        let centre = |id: u8| {
            let stack = observation.stacks.iter().find(|s| s.id == id).unwrap();
            let [x, y] = battle.hex_box(stack.hex / 17, stack.hex % 17);
            [x + 22, y + 26]
        };
        let ours = observation
            .stacks
            .iter()
            .find(|s| s.side == Side::Attacker)
            .unwrap();
        let theirs = observation
            .stacks
            .iter()
            .find(|s| s.side == Side::Defender)
            .unwrap();
        let on = centre(ours.id);
        let still = played(1, on, &[]).unwrap();
        assert_eq!(ids(&still), [stack_id(ours.id, WindowKind::All)]);
        assert_eq!(still.line_level, 1);
        let still = played(2, centre(theirs.id), &[]).unwrap();
        assert_eq!(ids(&still), [stack_id(theirs.id, WindowKind::Spells)]);
        assert_eq!(still.line_level, 2);
        let still = played(0, on, &[]).unwrap();
        assert_eq!((ids(&still), still.line_level), (Vec::new(), 0));
        let still = played(1, on, &["key:F5"]).unwrap();
        assert_eq!(ids(&still), [stack_id(ours.id, WindowKind::All)]);
        assert_eq!(still.line_level, 1);
        // From the stack to a hero and off the battlefield: the line's level each time.
        let still = played(1, on, &["key:F5", "move:30,70"]).unwrap();
        assert_eq!((ids(&still), still.line_level), (vec![1], 2));
        let still = played(1, on, &["key:F5", "move:400,590"]).unwrap();
        assert_eq!((ids(&still), still.line_level), (Vec::new(), 2));
        // The combat options dialog opens at the level then; its OK sets the level, the windows
        // as they are: closed by a click, the park's move writes the line at it; by its key
        // nothing until a new hex (the level-1 box at 421,379, the level-2 box at 421,409, the
        // OK at 566,467).
        let level = |still: &Still| still.options.as_ref().map(|o| o.settings.army_info);
        assert_eq!(level(&played(1, on, &["key:o"]).unwrap()), Some(1));
        assert_eq!(
            level(&played(1, on, &["key:F5", "key:o"]).unwrap()),
            Some(2)
        );
        let off = ["key:o", "click:421,379"];
        let still = played(1, on, &[&off[..], &["click:566,467"]].concat()).unwrap();
        assert_eq!((ids(&still), still.line_level), (Vec::new(), 0));
        let still = played(1, on, &[&off[..], &["key:Return"]].concat()).unwrap();
        assert_eq!((ids(&still), still.line_level), (Vec::new(), 1));
        let to_theirs = format!("move:{},{}", centre(theirs.id)[0], centre(theirs.id)[1]);
        let still = played(
            1,
            on,
            &[&off[..], &["key:Return", to_theirs.as_str()]].concat(),
        )
        .unwrap();
        assert_eq!((ids(&still), still.line_level), (Vec::new(), 0));
        let up = ["key:o", "click:421,409", "key:Return"];
        let still = played(0, on, &up).unwrap();
        assert_eq!((ids(&still), still.line_level), (Vec::new(), 0));
        let still = played(0, on, &[&up[..], &[to_theirs.as_str()]].concat()).unwrap();
        let spells = stack_id(theirs.id, WindowKind::Spells);
        assert_eq!((ids(&still), still.line_level), (vec![spells], 2));
        // E17 M6 T4c (design §12.16): the cursor shadow follows the battle's moves, F7 takes it
        // off, F6 composes the grid and shades the mouse's hex again; the dialog opens with the
        // still's options, and its close composes without the shade's call (oddity 2): the next
        // hex puts the old block back (the grid's box at 199,151).
        let cursor = Shades {
            cursor: true,
            ..Shades::default()
        };
        let hex = |still: &Still| {
            still
                .cells
                .drawn(&std::collections::BTreeSet::new())
                .overlays
        };
        let theirs_hex = theirs.hex;
        let still = played_with((0, cursor), on, &[]).unwrap();
        assert_eq!(hex(&still).cursor, [ours.hex]);
        let still = played_with((0, cursor), on, &[to_theirs.as_str()]).unwrap();
        assert_eq!(hex(&still).cursor, [theirs_hex]);
        let still = played_with((0, cursor), on, &["key:F7"]).unwrap();
        assert!(hex(&still).cursor.is_empty() && !still.cells.options().cursor);
        let still = played_with((0, cursor), on, &["key:F6"]).unwrap();
        assert_eq!(
            (hex(&still).grid, hex(&still).cursor),
            (true, vec![ours.hex])
        );
        let opened = played_with((0, cursor), on, &["key:o"]).unwrap();
        assert!(opened.options.is_some_and(|o| o.settings.cursor_shade));
        let dialog = ["key:o", "click:199,151", "key:Return", to_theirs.as_str()];
        let still = played_with((0, cursor), on, &dialog).unwrap();
        let overlays = hex(&still);
        assert_eq!((overlays.grid, overlays.cursor), (true, vec![theirs_hex]));
        assert_eq!(overlays.stale.len(), 1);
        assert!(!still.options_arrow);
    }
}
