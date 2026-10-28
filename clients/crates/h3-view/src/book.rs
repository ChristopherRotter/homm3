//! The hero's spell book in battle (E17 M5 T3a; `docs/epics/E17-design.md` §7.17): the dialog's
//! state (its tab, bookmark and page, and the memory of them between openings), the page a state
//! shows (`spell_book_page` 0x59C9A0), what its items show ([`Book::shown`]), and what a click on
//! an item or a key does (`spell_book_proc` 0x59D040). The dialog's items are the layout's
//! `[book]` (`h3_ui::ui`); the spells come from the battle (the caller's [`Hero`]), the words from
//! the catalogue.
//!
//! *Not modelled* (later tasks): the page turns' animation (the videos `PgTrnLft` and
//! `PgTrnRgh`, played only with the option "Animate SpellBook", 0x698788), the message boxes (a
//! spell above the hero's mana, an adventure spell's description, the right button's help), the
//! adventure map's book.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::layout::BookLayout;
use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Shown, State};

/// The book's picture (`Spelback.pcx`), in the hero's player's colours.
pub const BACKGROUND: u16 = 200;
/// The first place's mastery ribbon (201 to 212, place by place).
pub const RIBBONS: u16 = 201;
/// The first place's spell icon (`spells.def`, 213 to 224).
pub const ICONS: u16 = 213;
/// The last place's spell icon.
const LAST_ICON: u16 = ICONS + 11;
/// The tabs' picture (`SpelTab.def`, its frame the tab's school, 4 for all).
pub const TABS: u16 = 225;
/// The first tab's area: air (226), fire, water, earth, all schools (230).
pub const TAB_AREAS: u16 = 226;
/// The last tab's area.
const LAST_TAB: u16 = TAB_AREAS + 4;
/// The combat spells' bookmark.
pub const COMBAT: u16 = 231;
/// The adventure spells' bookmark.
pub const ADVENTURE: u16 = 232;
/// The hero's spell points.
pub const MANA: u16 = 233;
/// The previous page's corner (`SpelTrnL.pcx`).
pub const PREVIOUS: u16 = 234;
/// The next page's corner (`SpelTrnR.pcx`).
pub const NEXT: u16 = 235;
/// A school's emblem over a school's first page (`schools.def`).
pub const EMBLEM: u16 = 236;
/// The first place's text (237 to 248).
pub const TEXTS: u16 = 237;
/// The status line (`spelroll.pcx`).
pub const STATUS: u16 = 249;
/// The close area (the dialog's result 0x7801).
pub const CLOSE: u16 = 0x7801;
/// The places a page holds.
pub const PLACES: usize = 12;
/// The tab of all schools: every school bit.
pub const ALL: u8 = 0xF;

/// The schools' bits in the order `hero_best_school` (0x4E51C0) tries them, each with its index
/// (air 0, fire 1, water 2, earth 3: the ribbons', the emblems' and the tabs' frames).
const BEST_ORDER: [(u8, usize); 4] = [(1, 0), (2, 1), (8, 3), (4, 2)];
/// Each tab's place from the top (0x641DA4, by school index, all schools last): air, earth, fire,
/// water, all.
const TAB_PLACE: [usize; 5] = [0, 2, 3, 1, 4];
/// The tab at each place from the top (0x641DB8).
const TAB_AT: [usize; 5] = [0, 3, 1, 2, 4];

/// The spells a page lists: the bookmark (+0x74).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The combat spells (1; a spell's flag 1).
    Combat,
    /// The adventure spells (2; a spell's flag 2).
    Adventure,
}

/// A spell in the hero's book as the book shows it: the caller's, from the battle's ruleset, the
/// hero's masteries and the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spell {
    /// The engine's handle, which a choice gives back.
    pub handle: u16,
    /// The original's number: its icon's frame in `spells.def`.
    pub original: u32,
    /// Its name.
    pub name: String,
    /// Its description at the hero's mastery (the right click's text; its first line is the
    /// hint).
    pub description: String,
    /// Its level, 1 to 5.
    pub level: u8,
    /// Its schools' bits: air 1, fire 2, water 4, earth 8.
    pub schools: u8,
    /// Combat or adventure.
    pub kind: Kind,
    /// The hero's mastery of it on this battlefield, 0 (none) to 3 (expert; 0x4E5080).
    pub mastery: u8,
    /// Its cost to the hero (0x4E5240).
    pub cost: u16,
}

/// The hero whose book it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hero {
    /// Its identity: the book's memory is the last hero's (+0x1A).
    pub id: u16,
    /// Its spell points.
    pub mana: u16,
    /// Its mastery of each school (air, fire, water, earth): a spell of several schools shows the
    /// one it knows best on the tab of all schools (0x4E51C0).
    pub schools: [u8; 4],
    /// The spells in its book.
    pub spells: Vec<Spell>,
}

/// What the book remembers between openings (`spell_book_memory` 0x684B40 and the tab 0x6A34F4):
/// the hero, its last page, its last bookmark and tab. One per game (the client's session).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Memory {
    hero: Option<u16>,
    page: u16,
    last: Option<Kind>,
    tab: u8,
    /// The item the mouse was last over in any opening (0x684B4C, never reset: none at first, as
    /// the executable's -1).
    hovered: Option<u16>,
}

/// The book open: its tab (school bits), bookmark and page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Book {
    /// The tab's school bits ([`ALL`] for all schools).
    pub tab: u8,
    /// The bookmark.
    pub kind: Kind,
    /// The page, from 0.
    pub page: u16,
}

/// A place's spell as a page shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The spell's handle.
    pub handle: u16,
    /// Its icon's frame.
    pub original: u32,
    /// The school whose ribbon shows (its index: air 0, fire 1, water 2, earth 3).
    pub school: usize,
    /// The ribbon's frame: the hero's mastery.
    pub mastery: u8,
    /// The text under the icon.
    pub text: String,
    /// The text is greyed: the spell costs more than the hero's spell points.
    pub greyed: bool,
    /// The status line's hint over it.
    pub hint: String,
}

/// A page as it shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Page {
    /// The places' spells, row by row, the left page first.
    pub places: [Option<Place>; PLACES],
    /// The school whose emblem covers places 0 and 1 (a school's first page).
    pub emblem: Option<usize>,
    /// The previous page's corner shows (a page before this one).
    pub previous: bool,
    /// The next page's corner shows (spells after this page).
    pub next: bool,
}

/// What a click or a key did to the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing.
    Nothing,
    /// The book turned to another page, tab or bookmark: draw it again.
    Turned,
    /// The book closed without a spell.
    Closed,
    /// The book closed on a spell the hero can pay: its handle.
    Chosen(u16),
    /// The spell costs more than the hero's spell points (GENRLTXT 207's message box; not drawn).
    Refused {
        /// The spell's cost.
        cost: u16,
        /// The hero's spell points.
        mana: u16,
    },
    /// An adventure spell clicked in battle: its description in a message box (not drawn).
    Described(u16),
}

impl Book {
    /// Opens the book of `hero` in battle (the constructor 0x59BDF0 with the kind 0): a hero other
    /// than the last forgets the page, bookmark and tab; the combat bookmark shows, at the last
    /// tab and page if the book closed on combat spells last, else on page 0 of all schools.
    pub fn open(memory: &mut Memory, hero: u16) -> Self {
        if memory.hero != Some(hero) {
            *memory = Memory {
                hero: Some(hero),
                hovered: memory.hovered,
                ..Memory::default()
            };
        }
        if memory.last == Some(Kind::Combat) {
            Self {
                tab: memory.tab,
                kind: Kind::Combat,
                page: memory.page,
            }
        } else {
            memory.last = Some(Kind::Combat);
            memory.tab = ALL;
            memory.page = 0;
            Self {
                tab: ALL,
                kind: Kind::Combat,
                page: 0,
            }
        }
    }

    /// The spells the page lists (0x59CA58): the hero's of the tab's schools and the bookmark's
    /// kind, each with the school it shows under (the tab's, or on the tab of all schools the one
    /// the hero knows best), sorted by level, then school bit, then name (0x59DBE0; the name's
    /// letters A to Z as a to z; the handle last, never needed).
    #[must_use]
    pub fn entries<'h>(&self, hero: &'h Hero) -> Vec<(&'h Spell, u8)> {
        let mut list: Vec<(&Spell, u8)> = hero
            .spells
            .iter()
            .filter(|s| s.schools & self.tab != 0 && s.kind == self.kind)
            .map(|s| {
                let school = if self.tab == ALL {
                    best_school(hero.schools, s.schools)
                } else {
                    self.tab
                };
                (s, school)
            })
            .collect();
        list.sort_by(order);
        list
    }

    /// The page this book shows of `hero`'s spells (`spell_book_page` 0x59C9A0), its words from
    /// `catalogue`: twelve places from the page's first entry (a school's later pages start two
    /// earlier, its first page's emblem taking places 0 and 1), the corners where there is a page
    /// before or entries after.
    #[must_use]
    pub fn page(&self, hero: &Hero, catalogue: &Catalogue) -> Page {
        let entries = self.entries(hero);
        let school_tab = self.tab != ALL;
        let mut start = usize::from(self.page) * PLACES;
        if self.page > 0 && school_tab {
            start -= 2;
        }
        let mut page = Page::default();
        let mut first = 0;
        if self.page == 0 && school_tab {
            page.emblem = Some(index_of(self.tab));
            first = 2;
        }
        let mut next = start;
        for place in &mut page.places[first..] {
            let Some(&(spell, school)) = entries.get(next) else {
                break;
            };
            *place = Some(Place {
                handle: spell.handle,
                original: spell.original,
                school: index_of(school),
                mastery: spell.mastery,
                text: place_text(spell, catalogue),
                greyed: spell.cost > hero.mana,
                hint: hint(spell, catalogue),
            });
            next += 1;
        }
        page.previous = self.page > 0;
        page.next = next < entries.len();
        page
    }

    /// What the dialog's items show for `page`: the picture in player `player`'s colours, each
    /// place's ribbon (its school's DEF of `ribbons`, its frame the mastery), icon and text
    /// (disabled when greyed) or nothing, the emblem, the tabs' frame, the corners, the spell
    /// points (0x59C638: `%d`) and the status line's `status`.
    #[must_use]
    pub fn shown(
        &self,
        page: &Page,
        hero: &Hero,
        (ribbons, player): (&[String], u8),
        status: &str,
    ) -> Shown {
        let hidden = State {
            hidden: true,
            ..State::default()
        };
        let mut shown = Shown::default();
        shown.players.insert(BACKGROUND, player);
        for (k, place) in page.places.iter().enumerate() {
            let k = u16::try_from(k).unwrap_or(u16::MAX);
            let (ribbon, icon, text) = (RIBBONS + k, ICONS + k, TEXTS + k);
            match place {
                Some(p) => {
                    if let Some(def) = ribbons.get(p.school) {
                        shown.defs.insert(ribbon, def.clone());
                    }
                    shown.frames.insert(ribbon, u32::from(p.mastery));
                    shown.frames.insert(icon, p.original);
                    shown.texts.insert(text, p.text.clone());
                    if p.greyed {
                        shown.states.insert(
                            text,
                            State {
                                disabled: true,
                                ..State::default()
                            },
                        );
                    }
                }
                None => {
                    shown.states.insert(ribbon, hidden);
                    shown.states.insert(icon, hidden);
                }
            }
        }
        match page.emblem {
            Some(school) => {
                shown
                    .frames
                    .insert(EMBLEM, u32::try_from(school).unwrap_or(u32::MAX));
            }
            None => {
                shown.states.insert(EMBLEM, hidden);
            }
        }
        shown
            .frames
            .insert(TABS, u32::try_from(index_of(self.tab)).unwrap_or(u32::MAX));
        if !page.previous {
            shown.states.insert(PREVIOUS, hidden);
        }
        if !page.next {
            shown.states.insert(NEXT, hidden);
        }
        shown.texts.insert(MANA, hero.mana.to_string());
        shown.texts.insert(STATUS, status.into());
        shown
    }

    /// A left press on item `id` (0x59D3E3; the caller found the item that takes it): a place
    /// gives its spell, or the refusal when it costs more than the hero's spell points (an
    /// adventure spell its description); a tab, a bookmark or a corner turns the book; the close
    /// area closes it.
    pub fn click(&mut self, memory: &mut Memory, id: u16, (hero, page): (&Hero, &Page)) -> Outcome {
        match id {
            ICONS..=LAST_ICON if self.kind == Kind::Adventure => place_of(page, id - ICONS)
                .map_or(Outcome::Nothing, |p| Outcome::Described(p.handle)),
            ICONS..=LAST_ICON => match place_of(page, id - ICONS) {
                Some(p) => {
                    let cost = hero
                        .spells
                        .iter()
                        .find(|s| s.handle == p.handle)
                        .map_or(0, |s| s.cost);
                    if cost > hero.mana {
                        Outcome::Refused {
                            cost,
                            mana: hero.mana,
                        }
                    } else {
                        Outcome::Chosen(p.handle)
                    }
                }
                None => Outcome::Nothing,
            },
            TAB_AREAS..=LAST_TAB => self.turn_to_tab(memory, usize::from(id - TAB_AREAS)),
            COMBAT => self.turn_to_kind(memory, Kind::Combat),
            ADVENTURE => self.turn_to_kind(memory, Kind::Adventure),
            PREVIOUS => self.turn_to_page(memory, self.page.saturating_sub(1)),
            NEXT => self.turn_to_page(memory, self.page + 1),
            CLOSE => Outcome::Closed,
            _ => Outcome::Nothing,
        }
    }

    /// A key down (0x59D1B4, by the screens' scan codes): Escape and Enter close the book, A and C
    /// take a bookmark, the left and right arrows turn a page where the corner shows, the up and
    /// down arrows take the tab above or below (round to the other end).
    pub fn key(&mut self, memory: &mut Memory, key: &str, page: &Page) -> Outcome {
        match key {
            "Escape" | "Return" => Outcome::Closed,
            "A" => self.turn_to_kind(memory, Kind::Adventure),
            "C" => self.turn_to_kind(memory, Kind::Combat),
            "Left" if page.previous => self.turn_to_page(memory, self.page - 1),
            "Right" if page.next => self.turn_to_page(memory, self.page + 1),
            "Up" | "Down" => {
                let at = TAB_PLACE[index_of(self.tab)];
                let to = if key == "Up" {
                    at.checked_sub(1).unwrap_or(4)
                } else {
                    (at + 1) % 5
                };
                self.turn_to_tab(memory, TAB_AT[to])
            }
            _ => Outcome::Nothing,
        }
    }

    /// The tab of school `index` (4: all), page 0 (0x59D513).
    fn turn_to_tab(&mut self, memory: &mut Memory, index: usize) -> Outcome {
        let tab = if index < 4 { 1 << index } else { ALL };
        if tab == self.tab {
            return Outcome::Nothing;
        }
        self.tab = tab;
        memory.tab = tab;
        self.turn_to_page(memory, 0)
    }

    /// The bookmark `kind`, page 0 (0x59D5A2, 0x59D5E8).
    fn turn_to_kind(&mut self, memory: &mut Memory, kind: Kind) -> Outcome {
        if kind == self.kind {
            return Outcome::Nothing;
        }
        self.kind = kind;
        memory.last = Some(kind);
        self.turn_to_page(memory, 0)
    }

    /// Page `page`, remembered (0x59CFC1).
    fn turn_to_page(&mut self, memory: &mut Memory, page: u16) -> Outcome {
        self.page = page;
        memory.page = page;
        Outcome::Turned
    }
}

/// The book open over the battle (E17 M5 T3a): the book, its hero, the page it shows, the status
/// line and the cursor the mouse's moves over it set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The book.
    pub book: Book,
    /// Its hero.
    pub hero: Hero,
    /// The page shown.
    pub page: Page,
    /// The status line's text: empty as the book opens (0x59C728).
    pub status: String,
    /// The default cursor's frame the book set (`crdeflt.def`, `SetCursor(frame, 0)` at 0x59D712):
    /// 1 over an item, 0 elsewhere; none until the mouse enters another item than the last.
    pub cursor: Option<u32>,
}

impl Open {
    /// The book of `hero` opened ([`Book::open`]), its page made.
    pub fn new(memory: &mut Memory, hero: Hero, catalogue: &Catalogue) -> Self {
        let book = Book::open(memory, hero.id);
        let page = book.page(&hero, catalogue);
        Self {
            book,
            hero,
            page,
            status: String::new(),
            cursor: None,
        }
    }

    /// What the dialog's items show ([`Book::shown`]): the book's picture in player `player`'s
    /// colours.
    #[must_use]
    pub fn shown(&self, layout: &BookLayout, player: u8) -> Shown {
        self.book.shown(
            &self.page,
            &self.hero,
            (&layout.ribbons, player),
            &self.status,
        )
    }

    /// The dialog's draws over the battle, the players' palette `players`.
    #[must_use]
    pub fn draws(&self, layout: &BookLayout, players: &str, player: u8) -> Vec<crate::scene::Draw> {
        layout
            .dialog
            .draws(&self.shown(layout, player), players)
            .into_iter()
            .map(crate::scene::Draw::Dialog)
            .collect()
    }

    /// The cursor the book set, at the mouse's `point` (none before it set one): its frame of the
    /// layout's cursor, the canvas's corner at the mouse less the frame's hot spot.
    #[must_use]
    pub fn cursor_draw(&self, layout: &BookLayout, point: [i32; 2]) -> Option<crate::scene::Draw> {
        let frame = self.cursor?;
        let hot = layout
            .hot
            .get(usize::try_from(frame).unwrap_or(usize::MAX))
            .copied()
            .unwrap_or_default();
        Some(crate::scene::Draw::Cursor {
            def: layout.cursor.clone(),
            frame,
            x: point[0],
            y: point[1],
            hot,
        })
    }

    /// The item that takes the mouse at `point` (`ItemAtPosition` 0x5FF3E0), if any.
    fn item(&self, layout: &BookLayout, point: [i32; 2]) -> Option<u16> {
        let shown = self.shown(layout, 0);
        layout
            .dialog
            .item_taking(point, &shown)
            .map(h3_ui::ui::Item::id)
    }

    /// A left press at `point` ([`Book::click`] on the item that takes it); a turned book's page is
    /// made again.
    pub fn press(
        &mut self,
        memory: &mut Memory,
        (layout, catalogue): (&BookLayout, &Catalogue),
        point: [i32; 2],
    ) -> Outcome {
        let Some(id) = self.item(layout, point) else {
            return Outcome::Nothing;
        };
        let outcome = self.book.click(memory, id, (&self.hero, &self.page));
        self.turned(outcome, catalogue)
    }

    /// A key down ([`Book::key`]); a turned book's page is made again.
    pub fn key(&mut self, memory: &mut Memory, key: &str, catalogue: &Catalogue) -> Outcome {
        let outcome = self.book.key(memory, key, &self.page);
        self.turned(outcome, catalogue)
    }

    fn turned(&mut self, outcome: Outcome, catalogue: &Catalogue) -> Outcome {
        if outcome == Outcome::Turned {
            self.page = self.book.page(&self.hero, catalogue);
        }
        outcome
    }

    /// The mouse moved to `point` (0x59D6C4): when it is over another item than the last (the
    /// memory's, never reset: [`Memory`]), the cursor's frame and the status line follow it: a
    /// place's spell's hint, another item's own hint, nothing elsewhere. Whether anything changed.
    pub fn hover(
        &mut self,
        memory: &mut Memory,
        (layout, catalogue): (&BookLayout, &Catalogue),
        point: [i32; 2],
    ) -> bool {
        let item = self.item(layout, point);
        if item == memory.hovered {
            return false;
        }
        memory.hovered = item;
        self.cursor = Some(u32::from(item.is_some()));
        self.status = match item {
            Some(id @ ICONS..=LAST_ICON) => place_of(&self.page, id - ICONS)
                .map(|p| p.hint.clone())
                .unwrap_or_default(),
            Some(id) => layout
                .dialog
                .item(id)
                .and_then(h3_ui::ui::Item::hint)
                .map(|key| catalogue.text(key, &Args::new()))
                .unwrap_or_default(),
            None => String::new(),
        };
        true
    }
}

/// The school of `bits` the hero knows best (`hero_best_school` 0x4E51C0): air, fire, earth and
/// water tried in order, a later one only when known better; `bits` itself without any.
#[must_use]
pub fn best_school(levels: [u8; 4], bits: u8) -> u8 {
    let mut best: Option<u8> = None;
    let mut out = bits;
    for (bit, index) in BEST_ORDER {
        if bits & bit != 0 && best.is_none_or(|b| levels[index] > b) {
            best = Some(levels[index]);
            out = bit;
        }
    }
    out
}

/// The index of the first school bit of `bits` (air 0 to earth 3), 4 for all of them and 0 for
/// none (0x59CBDB, 0x59CCE0).
const fn index_of(bits: u8) -> usize {
    if bits == ALL {
        return 4;
    }
    let mut i = 0;
    while i < 4 {
        if bits & (1 << i) != 0 {
            return i;
        }
        i += 1;
    }
    0
}

/// The book's order of two entries (`spell_book_order` 0x59DBE0).
fn order(a: &(&Spell, u8), b: &(&Spell, u8)) -> Ordering {
    let name = |s: &Spell| -> Vec<u8> { s.name.bytes().map(|c| c.to_ascii_lowercase()).collect() };
    a.0.level
        .cmp(&b.0.level)
        .then(a.1.cmp(&b.1))
        .then_with(|| name(a.0).cmp(&name(b.0)))
        .then(a.0.handle.cmp(&b.0.handle))
}

/// The place `k`'s spell on `page`, if any.
fn place_of(page: &Page, k: u16) -> Option<&Place> {
    page.places.get(usize::from(k))?.as_ref()
}

/// A place's text (0x59CE19, 0x59CEE1): the name, the level with the mastery's short name when
/// the hero has one, the cost.
fn place_text(spell: &Spell, catalogue: &Catalogue) -> String {
    let level = level_word(spell, catalogue);
    let points = catalogue.text("book.points", &Args::new());
    let args = Args::new()
        .text("spell", &spell.name)
        .text("level", &level)
        .text("points", &points)
        .number("cost", i64::from(spell.cost));
    match spell.mastery {
        0 => catalogue.text("book.place-unskilled", &args),
        m => {
            let mastery = catalogue.text(
                &format!("book.mastery.{}", MASTERIES[usize::from(m.min(3)) - 1]),
                &Args::new(),
            );
            catalogue.text("book.place", &args.text("mastery", &mastery))
        }
    }
}

/// The masteries' names in the keys, basic to expert.
const MASTERIES: [&str; 3] = ["basic", "advanced", "expert"];

/// A spell's level's word (GENRLTXT 173 to 177).
fn level_word(spell: &Spell, catalogue: &Catalogue) -> String {
    catalogue.text(&format!("book.level.{}", spell.level), &Args::new())
}

/// The status line over a place (0x59BAA0 with its hint flag): the description's first line, then
/// the level in parentheses (the executable's own " (" and ")", 0x684B80 and 0x660350).
fn hint(spell: &Spell, catalogue: &Catalogue) -> String {
    let first = spell.description.split('\n').next().unwrap_or_default();
    format!("{first} ({})", level_word(spell, catalogue))
}

#[cfg(test)]
mod tests;
