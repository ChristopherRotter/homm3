//! The battle's panel below the battlefield (E17 M3 T6a; `docs/epics/E17-design.md` §9.7): its
//! controls' states as the executable sets them when a stack's turn starts, from what a viewer
//! knows of the battle. The panel is a [`crate::ui::Dialog`] of the layout; this module names its
//! controls by the executable's ids and decides what each shows.
//!
//! The executable builds the panel once (`battle_panel_base` 0x46B610 and `battle_panel_create`
//! 0x46BC30): the picture in the colours of the side it is built for, every button enabled but
//! Auto Combat when both sides are human (0x46BB40), the log's scroll buttons disabled (0x46BE64).
//! At each turn's start `battle_controls_state` (0x4782D0) sets them again: for a side whose
//! player is human the picture takes that player's colours and each button is enabled or disabled
//! by the battle (below); for any other side 0x46BF90 disables the seven buttons and leaves the
//! rest as they were. Nothing else sets them while the battle runs (E17 M3 T6b, design §9.8: an
//! action and its animation leave the panel as its turn's start set it; the hotkeys Wait and
//! Defend are the buttons' own, outside the battle's key switch 0x4758FB, not read further): the
//! panel at a moment is its last turn's start's, the battle as that turn found it ([`turns`]),
//! with what it kept from the last human player's turn ([`kept`]). Not read: the tactics phase's
//! panel 0x46C050 and Auto Combat switched on (the queue's inbox).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use h3_protocol::wire::{Event, Observation, Side};

use crate::ui::{Shown, State};

/// The panel's picture (`cbar.pcx`).
pub const PICTURE: u16 = 2000;
/// Surrender (`icm001.def`).
pub const SURRENDER: u16 = 2001;
/// Retreat (`icm002.def`).
pub const RETREAT: u16 = 2002;
/// The options (`icm003.def`).
pub const OPTIONS: u16 = 2003;
/// Auto Combat (`icm004.def`).
pub const AUTO_COMBAT: u16 = 2004;
/// The battle log's item (§9.5).
pub const LOG: u16 = 2005;
/// The log's scroll button up (`ComSlide.def`).
pub const LOG_UP: u16 = 2006;
/// The log's scroll button down.
pub const LOG_DOWN: u16 = 2007;
/// The hero's spell book (`icm005.def`).
pub const SPELL_BOOK: u16 = 2008;
/// Wait (`icm006.def`).
pub const WAIT: u16 = 2009;
/// Defend (`icm007.def`).
pub const DEFEND: u16 = 2010;

/// The seven buttons a turn's start sets (0x46BF90 disables them all on a turn no human plays).
pub const BUTTONS: [u16; 7] = [
    SURRENDER,
    RETREAT,
    OPTIONS,
    AUTO_COMBAT,
    SPELL_BOOK,
    WAIT,
    DEFEND,
];

/// The battle as each turn's start finds it (E17 M3 T6b): for each `TurnStarted` of `events` (a
/// record's, in order) but a frozen stack's, the opening's `observation` with the events before it applied as far as
/// the panel reads them: the active stack, the round's wait phase (its stack waited this round:
/// only waited stacks act then), the heroes' casts this round and the tactics phase.
#[must_use]
pub fn turns(observation: &Observation, events: &[Event]) -> Vec<Observation> {
    let mut now = observation.clone();
    let mut waited = BTreeSet::new();
    let mut out = Vec::new();
    for (i, event) in events.iter().enumerate() {
        match *event {
            Event::RoundStarted { round } => {
                now.round = round;
                waited.clear();
                for side in &mut now.sides {
                    if let Some(hero) = side.hero.as_mut() {
                        hero.casts = 0;
                    }
                }
            }
            Event::Waited { stack } => {
                waited.insert(stack);
            }
            Event::SpellCast { side, .. } => {
                if let Some(hero) = now.sides[usize::from(side == Side::Defender)].hero.as_mut() {
                    hero.casts = hero.casts.saturating_add(1);
                }
            }
            Event::TacticsEnded {} => now.tactics = None,
            // A turn its pick freezes is none (E17 M3 T7b-2b-2b, as the timeline's).
            Event::TurnStarted { stack } if !crate::anim::freezes(events.get(i + 1), stack) => {
                now.active = Some(stack);
                now.wait_phase = waited.contains(&stack);
                out.push(now.clone());
            }
            _ => {}
        }
    }
    out
}

/// What the panel keeps from turn to turn (E17 M3 T6b): a turn no human plays leaves the
/// picture's colours and the scroll buttons as the last human player's turn set them (0x46BF90).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    /// The side whose player's colours the picture shows.
    pub colours: Side,
    /// The scroll buttons enabled: a human's turn outside the tactics phase enables them
    /// (0x4786CA); else they are as built, disabled.
    pub scroll: bool,
}

/// What the panel keeps after the turns `started` (in order: each one's acting side, the
/// stack's controller, and whether the tactics phase ran), built for the side `built`
/// (0x46B70F's side the manager holds then, not read; *provisional*: the opening's side): the
/// colours of the last side a human plays (0x4782D0 colours it), else `built`'s; the scroll
/// buttons enabled once a human's turn started outside the tactics phase.
#[must_use]
pub fn kept(observation: &Observation, started: &[(Side, bool)], built: Side) -> Kept {
    let human = |s: Side| observation.sides[usize::from(s == Side::Defender)].human;
    let colours = started
        .iter()
        .rev()
        .map(|&(s, _)| s)
        .find(|&s| human(s))
        .unwrap_or(built);
    let scroll = started.iter().any(|&(s, tactics)| human(s) && !tactics);
    Kept { colours, scroll }
}

/// What the panel shows when the stacks of `side` (the active stack's controller; none before a
/// turn starts) begin their turn in the battle `observation` shows, with what it `kept` from the
/// turns before (the picture's colours, the scroll buttons), its log showing `log`; each side's
/// player's colour is `players`' entry (the record names none: CLT-Q6).
#[must_use]
pub fn shown(
    observation: &Observation,
    side: Option<Side>,
    kept: Kept,
    log: &str,
    players: [u8; 2],
) -> Shown {
    let index = |s: Side| usize::from(s == Side::Defender);
    let human = |s: Side| observation.sides[index(s)].human;
    let disabled = State {
        disabled: true,
        ..State::default()
    };
    let enabled = |on: bool| if on { State::default() } else { disabled };
    let mut states = BTreeMap::new();
    // As built (0x46BB40, 0x46BE64): Auto Combat disabled when both sides are human, the scroll
    // buttons disabled until a human's turn enables them.
    states.insert(
        AUTO_COMBAT,
        enabled(!(human(Side::Attacker) && human(Side::Defender))),
    );
    states.insert(LOG_UP, enabled(kept.scroll));
    states.insert(LOG_DOWN, enabled(kept.scroll));
    let picture = players[index(kept.colours)];
    match side {
        Some(s) if human(s) => {
            let view = &observation.sides[index(s)];
            let other = &observation.sides[1 - index(s)];
            let at_town = observation.siege.is_some();
            let tactics = observation.tactics.is_some();
            // 0x478573: a hero on both sides, and not a town's defender.
            let surrender =
                view.hero.is_some() && other.hero.is_some() && !(at_town && s == Side::Defender);
            // 0x4785C7: a hero, and the attacker or no town; a town's defender only with its
            // Escape Tunnel, which the viewer knows through the side's own `may_retreat`
            // (*provisional*: sieges are a later milestone's).
            let retreat =
                view.hero.is_some() && (s == Side::Attacker || !at_town || view.may_retreat);
            // 0x4786EF: the hero wears a spell book and has not cast this round (+0x54B4; its
            // exception +0x13D74 is not read), outside the tactics phase.
            let book = view.hero.is_some_and(|h| h.spell_book && h.casts == 0) && !tactics;
            states.insert(SURRENDER, enabled(surrender));
            states.insert(RETREAT, enabled(retreat));
            states.insert(OPTIONS, State::default());
            // 0x47864A: no longer selected; disabled when both sides are human.
            states.insert(
                AUTO_COMBAT,
                enabled(!(human(Side::Attacker) && human(Side::Defender))),
            );
            states.insert(SPELL_BOOK, enabled(book));
            // 0x478755: Wait outside the round's wait phase and the tactics phase.
            states.insert(WAIT, enabled(!observation.wait_phase && !tactics));
            if tactics {
                // Disabled when the dialog is built in the tactics phase (0x472428 to 0x472454;
                // its own panel 0x46C050 is not read).
                states.insert(DEFEND, disabled);
            } else {
                // 0x4786CA: the scroll buttons enabled; 0x4787B6: Defend.
                states.insert(LOG_UP, State::default());
                states.insert(LOG_DOWN, State::default());
                states.insert(DEFEND, State::default());
            }
        }
        Some(_) => {
            for id in BUTTONS {
                states.insert(id, disabled);
            }
        }
        None => {}
    }
    let mut texts = BTreeMap::new();
    texts.insert(LOG, String::from(log));
    let mut players_of = BTreeMap::new();
    players_of.insert(PICTURE, picture);
    Shown {
        states,
        players: players_of,
        texts,
    }
}

#[cfg(test)]
mod tests;
