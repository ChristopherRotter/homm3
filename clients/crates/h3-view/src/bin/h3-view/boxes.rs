//! The battle's message boxes (E17 M5 T4a; `docs/epics/E17-design.md` §7.19): what each asks
//! or tells and what its OK does, shared by the window's battle (`crate::battle`) and the capture
//! set's stills (`crate::spellbook::still`), which lay them out with `crate::log::Writer`.

use h3_protocol::wire::{Command, Observation, Side};
use h3_view::book;
use h3_view::message::{Picture, YES};
use h3_view::text::{Args, Catalogue};

/// What a message box asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Nothing: its OK alone closes it (the book's refusal or description, a spell that affects
    /// no one, the surrender refused).
    Told,
    /// Retreat (2002, `battle_retreat_button` 0x474FA1): its OK retreats.
    Retreat,
    /// Surrender (2001, `battle_surrender_button` 0x474FE9) at its price: its OK surrenders, or
    /// tells the side it cannot pay (GENRLTXT 30) when its funds are below the price.
    Surrender {
        /// The price.
        price: u32,
        /// The side's gold.
        funds: u32,
    },
    /// The battle's result window (E17 M5 T4b1, `crate::results`): its OK (Enter or Escape)
    /// closes the battle's screen.
    Result,
}

/// How the side to act may leave the battle by the panel's buttons: whether it may retreat (the
/// engine lists it), and its surrender's price, its gold and the other side's hero's name when it
/// may surrender at all (its own view prices it).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Leave {
    /// It may retreat.
    pub retreat: bool,
    /// Its surrender: the price, its gold, the other side's hero's name.
    pub surrender: Option<(u32, u32, String)>,
}

impl Leave {
    /// The side's ways to leave in its own view `own` with its legal commands `legal`, the
    /// heroes' names from the manifest's `hero_types` through `catalogue`.
    pub fn of(
        own: &Observation,
        legal: &[Command],
        hero_types: &[String],
        catalogue: &Catalogue,
    ) -> Self {
        let surrender = own
            .own_hero
            .as_ref()
            .and_then(|h| Some((h.surrender_price?, h.funds)))
            .map(|(price, funds)| {
                let other = 1 - usize::from(own.viewer == Side::Defender);
                let name = own.sides[other]
                    .hero
                    .as_ref()
                    .and_then(|h| hero_types.get(usize::from(h.hero_type)))
                    .map(|k| catalogue.text(&format!("hero.{k}"), &Args::new()))
                    .unwrap_or_default();
                (price, funds, name)
            });
        Self {
            retreat: legal.contains(&Command::Retreat {}),
            surrender,
        }
    }

    /// The question the panel's button `id` asks (GENRLTXT 29 to retreat; 33 with the other
    /// side's hero's name and the price to surrender) and what its OK does; none when the side
    /// may not leave so.
    pub fn ask(&self, catalogue: &Catalogue, id: u16) -> Option<(String, Asked)> {
        match id {
            h3_view::panel::RETREAT if self.retreat => Some((
                catalogue.text("battle.retreat.ask", &Args::new()),
                Asked::Retreat,
            )),
            h3_view::panel::SURRENDER => {
                let (price, funds, name) = self.surrender.as_ref()?;
                let args = Args::new()
                    .text("hero", name)
                    .number("price", i64::from(*price));
                Some((
                    catalogue.text("battle.surrender.ask", &args),
                    Asked::Surrender {
                        price: *price,
                        funds: *funds,
                    },
                ))
            }
            _ => None,
        }
    }
}

/// What a box's closing by its button `id` leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answered {
    /// Nothing more.
    Closed,
    /// The command its OK gives.
    Give(Command),
    /// Another box at once, telling this (the surrender refused, GENRLTXT 30: 0x475119).
    Tell(String),
}

/// What closing a box that asked `asked` by its button `id` leads to (0x474FD0, 0x4750E4).
pub fn answered(asked: Asked, id: u16, catalogue: &Catalogue) -> Answered {
    match asked {
        Asked::Retreat if id == YES => Answered::Give(Command::Retreat {}),
        Asked::Surrender { price, funds } if id == YES => {
            if funds >= price {
                Answered::Give(Command::Surrender {})
            } else {
                Answered::Tell(catalogue.text("battle.surrender.poor", &Args::new()))
            }
        }
        _ => Answered::Closed,
    }
}

/// The box the book's `outcome` shows over it, if any: a spell above the hero's spell points
/// (GENRLTXT 207 with the cost and the points, 0x59D462) or an adventure spell's description with
/// its picture (`spells`, its frame the spell's number, of `size`) and name (0x59D4DC).
pub fn book_box(
    outcome: book::Outcome,
    hero: &book::Hero,
    catalogue: &Catalogue,
    (spells, size): (&str, [i32; 2]),
) -> Option<(String, Option<Picture>)> {
    match outcome {
        book::Outcome::Refused { cost, mana } => {
            let args = Args::new()
                .number("cost", i64::from(cost))
                .number("mana", i64::from(mana));
            Some((catalogue.text("book.refused", &args), None))
        }
        book::Outcome::Described(handle) => {
            let spell = hero.spells.iter().find(|s| s.handle == handle)?;
            let picture = Picture {
                def: spells.to_owned(),
                frame: spell.original,
                size,
                caption: spell.name.clone(),
            };
            Some((spell.description.clone(), Some(picture)))
        }
        _ => None,
    }
}

/// The original's stack spells (15 to 19, 24 to 39, 41 to 58, 60 to 62) and Teleport (63): the
/// aim's entry tells "That spell will affect no one!" (GENRLTXT 186) when no hex takes them
/// (`enter_player_cast` 0x59F085).
pub fn affects_someone_only(original: u32) -> bool {
    matches!(original, 15..=19 | 24..=39 | 41..=58 | 60..=63)
}

/// The text of "That spell will affect no one!".
pub fn no_one(catalogue: &Catalogue) -> String {
    catalogue.text("aim.no-one", &Args::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_question_s_ok_leaves_or_tells() {
        let c = Catalogue::default();
        let leave = Leave {
            retreat: true,
            surrender: Some((300, 200, "Crag Hack".into())),
        };
        assert!(matches!(
            leave.ask(&c, h3_view::panel::RETREAT),
            Some((_, Asked::Retreat))
        ));
        assert!(matches!(
            leave.ask(&c, h3_view::panel::SURRENDER),
            Some((
                _,
                Asked::Surrender {
                    price: 300,
                    funds: 200
                }
            ))
        ));
        assert_eq!(leave.ask(&c, h3_view::panel::AUTO_COMBAT), None);
        assert_eq!(Leave::default().ask(&c, h3_view::panel::RETREAT), None);
        assert_eq!(Leave::default().ask(&c, h3_view::panel::SURRENDER), None);
        let no = h3_view::message::NO;
        assert_eq!(
            answered(Asked::Retreat, YES, &c),
            Answered::Give(Command::Retreat {})
        );
        assert_eq!(answered(Asked::Retreat, no, &c), Answered::Closed);
        let rich = Asked::Surrender {
            price: 300,
            funds: 300,
        };
        let poor = Asked::Surrender {
            price: 300,
            funds: 299,
        };
        assert_eq!(
            answered(rich, YES, &c),
            Answered::Give(Command::Surrender {})
        );
        assert!(matches!(answered(poor, YES, &c), Answered::Tell(_)));
        assert_eq!(answered(poor, no, &c), Answered::Closed);
        assert_eq!(
            answered(Asked::Told, h3_view::message::OK, &c),
            Answered::Closed
        );
        assert!(affects_someone_only(39) && affects_someone_only(63) && !affects_someone_only(40));
        assert!(!affects_someone_only(20) && !affects_someone_only(59));
    }
}
