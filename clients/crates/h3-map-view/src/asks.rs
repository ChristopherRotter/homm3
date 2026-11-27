//! The questions a visit asks (E24 M4 T2b2 part A1, *provisional*; `docs/epics/E24-design.md`
//! §5.4): a box of style 2 (`message_box` 0x4F6570, `h3_ui::message::told` with its OK and
//! Cancel) that a visit shows a human before what it gives. [`Ask`] is a question's facts as the
//! viewer knows them ([`Ask::of`]), [`Ask::told`] its box's text and pictures through the string
//! catalogue (ADVEVENT's, GENRLTXT's and ARRAYTXT's records and the names' families), [`Ask::yes`]
//! what its OK shows where the viewer builds it: the Warrior's Tomb's find and a dwelling's
//! creatures joining free. A few boxes of the same handlers ask nothing (a looted bank's, the
//! Tree of Knowledge's visited and too poor): [`Ask::asks`] tells them.
//!
//! *Provisional* (the viewer has no engine): the facts are the viewer's at the sequence's 0 (the
//! object's class, subclass, entrance word and owner, the selected hero's mark, the kingdom's
//! resources and the hero's army, a pile's or an artifact's message from the map), a dwelling's a
//! new game's (its creature's week of growth to hire; its guards three weeks' growth of a creature
//! of level 5 or more: the dwellings' survey §2, the tables' [`AskArt`]), and the Tomb's morale is
//! lost at its first search (the hero's status bit 21 not known).
//!
//! Since part A2 (*provisional*): a wandering monster's offers (to join free or for gold, to flee:
//! [`Ask::of`] decides as `monster_visit_decision` 0x4A7740 does, in integers, from the hero's
//! [`power`], its creatures' sympathy and its Diplomacy), a Border Guard's passage, a
//! Cartographer's map and Pandora's Box; [`Ask::no`] what the Cancel does (a monster's flight
//! asked after its offer declined, the monster gone after its flight declined) and [`Reply`] an
//! answer's boxes, gain, next question, the object taken off the map and the terrains revealed.
//!
//! Since part B (*provisional*): the choices (design §5.4's choices' table), a box of style 7 or
//! 10 ([`Ask::style`]) whose pictures are chosen: a treasure chest's gold or experience with the
//! "or" between them ([`Ask::or`]), the Arena's +2 Attack or +2 Defense and a School of Magic's
//! +1 Spell Power or +1 Knowledge for its price; [`Ask::yes`] the first picture's answer,
//! [`Ask::second`] the second's, [`Ask::no`] the Cancel's (the Arena's the Attack: MCL-Q33, the
//! original's; the School's neither, unpaid and unmarked).
//!
//! *Not built:* every other answer (a battle, a level-up, the recruit dialog: [`Ask::yes`] none;
//! the engine's, M4 T5 and M5), a mine of another player with guards (ADVEVENT 187: none at a new
//! game), a guarded artifact's several stacks (", " and GENRLTXT 142 between them), a pile's and an
//! artifact's message without guards, a dwelling of class 20 (ADVEVENT 36) and the Refugee Camp
//! (112), a free Tree (148: its level-up at once); since part A2 a monster's message and treasure
//! (its word's bit 31), its fight and its insult (87), a join without room (the army dialog), a
//! Border Guard without the key, a Cartographer visited (24) or too dear (28), Pandora's Box with
//! a message or guards (16) or empty (15); since part B a chest's artifact (bit 10), its
//! Learning (the experience the gold less 500 alone; the levels it reaches are T2c's, the
//! timeline's: `crate::levels`), the Arena visited, a School visited (ADVEVENT 72) or too dear
//! (73), the School of War (158, its code the Magic's: not recorded).
//!
//! Since E24 M4 T2d1 (design §5.6, *provisional*): a dwelling of another team without guards is
//! the visitor's first ([`Ask::flags`]), then asks its hire as the visitor's own does.
//!
//! Since E24 M5 T4c (design §5.28, *provisional*): live, a decision the view does not ask is a
//! notice in the client's words ([`Ask::Notice`]: a box of style 1, its OK the host's closing
//! answer).

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_map::model::{Pos, Rewards};
use h3_ui::message::Style;
use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

use crate::finds::{self, FindArt, Gain, Reward, Told};
use crate::hover::SIZES;
use crate::live::Stack;
use crate::notice::Notice;

/// The questions' numbers (the tables' `[map.asks]`; design §5.4). Test tooling's alone since E24
/// M5 T4c (design §5.29, the facts audit): only [`Ask::of`] reads them (a recording's or a
/// situation's question at its 0); play's questions and their facts are the engine's decision and
/// record's (`crate::engine::visits::opened`, `live`), and [`Ask::yes`] takes no tables.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArt {
    /// Each creature's level, 1 to 7, by its number (`creature_table` 0x6703B8's +4, plus 1; 0
    /// where no creature has the number).
    pub levels: Vec<u8>,
    /// A dwelling's guards at a new game: the least level a creature guards its dwelling from (5)
    /// and the weeks of its growth they are (3; the dwellings' survey §2).
    pub guards: [i32; 2],
    /// The Tree of Knowledge's prices 1 and 2 (`visit_tree_of_knowledge` 0x4A6710, its word's bits
    /// 13 to 15), each `[resource, amount]`: 2 000 gold, 10 gems.
    pub tree_prices: Vec<[i32; 2]>,
    /// The morale the Warrior's Tomb's search costs (−3, `visit_warriors_tomb` 0x4A7C30).
    pub tomb_morale: i8,
    /// A Cartographer's price, `[resource, amount]` (1 000 gold: `visit_object` 0x4A8A14, paid
    /// at 0x4A8B60; E24 M4 T2b2 part A2).
    pub maps_price: [i32; 2],
    /// The terrains each Cartographer's kind reveals (sea, land, underground: a bit per terrain,
    /// `reveal_terrain_mask` 0x4CA240's mask, set at a new game: 0x100, 0xBF, 0x40; the objects'
    /// survey §4).
    pub reveals: Vec<u32>,
    /// A monster's picture while a hero visits it (`avwattak.def`, loaded into the adventure
    /// manager's +0xE0 at 0x407247: two frames a creature, facing left and right;
    /// [`crate::live::Attacking`]).
    pub attacking: String,
    /// A treasure chest's gold by a step of its word's bits 11 to 14 (500; the rewards' survey
    /// §2) and what the experience offered is less than the gold (500: `chest_gold_or_experience`
    /// 0x4A6440, before the hero's Learning; part B).
    pub chest: [i32; 2],
    /// The Arena's gain to the skill chosen (2: `visit_arena` 0x49E7D0; part B).
    pub arena: i8,
    /// A School of Magic's gain to the skill chosen (1) and its price, `[resource, amount]`
    /// (1 000 gold: `visit_school_of_magic` 0x4A33E0; part B).
    pub school: i8,
    /// The School's price.
    pub school_price: [i32; 2],
}

impl AskArt {
    /// The level of creature `creature` (0: none known).
    #[must_use]
    pub fn level(&self, creature: u16) -> u8 {
        self.levels.get(usize::from(creature)).copied().unwrap_or(0)
    }
}

/// The string catalogue's keys the questions show (`clients/text/keys.toml`); the names are the
/// families `creature`, `artifact`, `bank` and `dwelling1`'s.
pub const KEYS: [&str; 44] = [
    "map.ask.size.0",
    "map.ask.size.1",
    "map.ask.size.2",
    "map.ask.size.3",
    "map.ask.size.4",
    "map.ask.size.5",
    "map.ask.size.6",
    "map.ask.size.7",
    "map.ask.size.8",
    "map.ask.artifact",
    "map.ask.guarded",
    "map.ask.mine",
    "map.ask.bank",
    "map.ask.bank-looted",
    "map.ask.derelict-ship",
    "map.ask.crypt",
    "map.ask.shipwreck",
    "map.ask.tree-gold",
    "map.ask.tree-gems",
    "map.ask.tree-visited",
    "map.ask.tree-poor-gold",
    "map.ask.tree-poor-gems",
    "map.ask.pyramid",
    "map.ask.tomb",
    "map.ask.tomb-found",
    "map.ask.tomb-empty",
    "map.ask.hire",
    "map.ask.joined",
    "map.ask.none",
    "map.ask.no-room",
    "map.ask.join",
    "map.ask.join-one",
    "map.ask.join-gold",
    "map.ask.join-price",
    "map.ask.flee",
    "map.ask.pass",
    "map.ask.maps-sea",
    "map.ask.maps-land",
    "map.ask.maps-underground",
    "map.ask.open-box",
    "map.ask.or",
    "map.ask.chest",
    "map.ask.arena",
    "map.ask.school",
];

/// The classes a question is asked at (as the game holds them): an artifact, a resource pile, a
/// spell scroll, a mine (the abandoned one its subclass 7), a creature bank, the search kinds
/// (the Derelict Ship, the Crypt, the Shipwreck), the Tree of Knowledge, the Pyramid, the
/// Warrior's Tomb, a dwelling of one creature.
const ARTIFACT: u32 = 5;
const PILE: u32 = 79;
const SCROLL: u32 = 93;
const MINE: u32 = 53;
const ABANDONED: u32 = 7;
const BANK: u32 = 16;
const DERELICT_SHIP: u32 = 24;
const CRYPT: u32 = 84;
const SHIPWRECK: u32 = 85;
const TREE: u32 = 102;
const PYRAMID: u32 = 63;
const TOMB: u32 = 108;
const DWELLING: u32 = 17;

/// A bank's word's looted bit (25, `visit_creature_bank` 0x4A15A0).
const LOOTED: u32 = 25;

/// Part A2's classes: a wandering monster (its creature the subclass), a Border Guard (its
/// colour the subclass), a Cartographer (its kind the subclass: sea, land, underground) and
/// Pandora's Box.
const MONSTER: u32 = 54;
const BORDER_GUARD: u32 = 9;
const CARTOGRAPHER: u32 = 13;
const PANDORA: u32 = 6;

/// Part B's classes: a treasure chest, the Arena, a School of Magic.
const CHEST: u32 = 101;
const ARENA: u32 = 4;
const SCHOOL: u32 = 47;

/// A chest's word (the rewards' survey §2): an artifact's bit 10, its gold's steps in bits 11 to
/// 14.
const CHEST_ARTIFACT: u32 = 10;
const CHEST_GOLD: u32 = 11;

/// The primary skills' picture types (31 to 34: Attack, Defense, Spell Power, Knowledge).
const PRIMARY: u32 = 31;

/// A monster's word (`map_item` +0x0 at the load, the battles' survey §1.2): its count in bits 0
/// to 11, its aggression in 12 to 16 (signed), "never flees" bit 17, "has a message" bit 31.
const COUNT: u32 = 0xFFF;
const AGGRESSION: u32 = 12;
const NEVER_FLEES: u32 = 17;
const MESSAGE: u32 = 31;

/// The creatures below this number pair a creature with its upgrade (2 n and 2 n + 1: the eight
/// towns' fourteen each), the related type a monster's sympathy counts (*provisional*:
/// `monster_sympathy` 0x4A75C0 finds it through the town creature table by faction and level;
/// Conflux's pairs not built, none for the neutral creatures).
const PAIRED: u16 = 112;

/// The gold's resource and picture type (6), the price's picture.
const GOLD: usize = 6;
const GOLD_PICTURE: u32 = 6;

/// The picture types the questions and their answers show: the experience's (17, subtype −1 its
/// "+1 Level"; a chest's its amount), an artifact's (8), the bad morale's (16).
const EXPERIENCE: (u32, i32) = (17, -1);
const ARTIFACT_PICTURE: u32 = 8;
const BAD_MORALE: (u32, i32) = (16, 0);

/// What the viewer knows of an object a question's facts come from (*provisional*: the live
/// part's at the sequence's 0, a dwelling's creature by the tables).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seen {
    /// Its class and subclass as the game holds them.
    pub class: [u32; 2],
    /// Its entrance tile's word.
    pub word: Option<u32>,
    /// Its owner's colour.
    pub owner: Option<u8>,
    /// The selected hero's mark on it (a Tree's).
    pub visited: bool,
    /// A pile's, a scroll's or an artifact's message on the map: its text (empty: none) and its
    /// guards' stacks in slot order (none: no guards).
    pub message: Option<(String, Vec<Stack>)>,
    /// A dwelling's creature (`dwelling_creatures` 0x63D570 by the subclass) and its growth.
    pub dwelling: Option<(u16, i32)>,
    /// A wandering monster's creature's numbers (part A2).
    pub monster: Option<Traits>,
    /// A Border Guard's: the viewer holds its colour's key (game +0x4E364; *provisional*: a
    /// Keymaster's Tent of its colour with the viewer's mark).
    pub key: bool,
    /// Pandora's Box's rewards (its message in [`Seen::message`]).
    pub rewards: Option<Rewards>,
}

/// The visiting player's facts a question reads (*provisional*: the live part's at 0).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Visitor {
    /// Its colour.
    pub colour: u8,
    /// Each colour's team.
    pub teams: [u8; 8],
    /// The kingdom's resources, in the resource bar's order.
    pub resources: [i32; 7],
    /// The selected hero's army (the status window's).
    pub army: [Option<Stack>; 7],
    /// The selected hero's power against a monster ([`power`]; part A2).
    pub power: i64,
    /// Its Diplomacy's level as the monster's decision takes it, 0 to 3 (hero +0xCD, one higher
    /// on Easy for a human: 0x4A787E; *provisional*: the binary's, the difficulty Normal's).
    pub diplomacy: u8,
    /// Its name (Pandora's Box's boxes', `grant_rewards` 0x49FA90's `%s`).
    pub hero: String,
}

impl Visitor {
    /// Whether the kingdom holds `amount` of `resource` (E24 M4 T6, the reviewer's R2-9: a Tree's,
    /// a Cartographer's and a School's price); none of a resource past the bar.
    #[must_use]
    pub fn has(&self, [resource, amount]: [i32; 2]) -> bool {
        let have = usize::try_from(resource)
            .ok()
            .and_then(|r| self.resources.get(r))
            .copied()
            .unwrap_or(0);
        have >= amount
    }

    /// Whether the hero's army has room for `creature` (R2-9: a dwelling's and a monster's join):
    /// an empty slot, or a stack of it.
    #[must_use]
    pub fn room(&self, creature: u16) -> bool {
        self.army
            .iter()
            .any(|s| s.is_none_or(|s| s.creature == creature))
    }
}

/// A creature's numbers a monster's offer reads (`CRTRAITS.TXT`'s, as the creature record holds
/// them: its gold cost +0x38, its AI value +0x40).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Traits {
    /// Its gold cost.
    pub gold: i32,
    /// Its AI value.
    pub value: i32,
}

/// A wandering monster's offer (`monster_visit_decision` 0x4A7740; the battles' survey §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offer {
    /// To join for free (`monster_join_free` 0x4A7000: ADVEVENT 86).
    Join,
    /// To join for gold, its price (`monster_join_for_gold` 0x4A7250: the creature's gold cost
    /// times the count; ADVEVENT 88 for one creature, else 89 then 90).
    Gold(i32),
    /// To flee (`monster_flee_offer` 0x4A6DF0: ADVEVENT 91).
    Flee,
}

/// How an object taken off the map is told (`hero_loss_effect` 0x4ACBB0's argument): a pickup's
/// sound (1; the pile's and the artifact's own) or `killfade.82M` (0), then the dissolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fade {
    /// A pickup's sound.
    Pickup,
    /// `killfade.82M`.
    Killfade,
}

/// What an answer does where the viewer builds it ([`Ask::yes`], [`Ask::no`]; *provisional*: the
/// engine's in time): its boxes (style 1, or `grant_rewards`' `message_box_pictures`), each with
/// what the panels show as it opens; what they show after the last; the status window built
/// again (`status_window_choose` 0x415DE0 with 1, 1, 1: a monster's join); the question asked
/// next; the object taken off the map and (E24 M4 T2c) what the panels show at its dissolve's
/// end; the terrains revealed (a bit per terrain: then the view and the minimap drawn again,
/// `visit_object`'s common end 0x4A9AD3). Its default ends the visit with nothing shown (A1's
/// every Cancel).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reply {
    /// The boxes.
    pub boxes: Vec<Reward>,
    /// They are `grant_rewards`' (`message_box_pictures` 0x4F7690), not `message_box`'s.
    pub granted: bool,
    /// What the panels show after the last.
    pub gain: Gain,
    /// The status window built again after the gain.
    pub chooses: bool,
    /// The question asked next.
    pub then: Option<Box<Ask>>,
    /// The object taken off the map, and how.
    pub taken: Option<Fade>,
    /// What the panels show at the taken object's dissolve's end (E24 M4 T2c, design §5.5: a
    /// chest's experience, whose levels follow the chest's removal, `visit_treasure_chest`
    /// 0x4A6520's `hero_level_up`).
    pub after: Gain,
    /// What it reveals (since E24 M5 T4c a recording's terrains or play's tiles, [`Reveal`]).
    pub reveal: Reveal,
    /// A Cancel's answer marks the find answered as an OK's does (part B: the Arena's, which
    /// gives the Attack: MCL-Q33).
    pub marks: bool,
}

/// What an answer reveals to the viewer (E24 M5 T4c, design §5.29, *provisional*): a recording's
/// terrains (test tooling's: a Cartographer's kind's by the tables, [`Ask::of`]) or play's tiles
/// (the record's: those its answer's `GameReveal` explored); then the view and the minimap drawn
/// again.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Reveal {
    /// Nothing.
    #[default]
    None,
    /// Every tile of these terrains on both levels (a bit per terrain code,
    /// `reveal_terrain_mask` 0x4CA240).
    Terrains(u32),
    /// These tiles.
    Tiles(Vec<Pos>),
}

/// A hero's power against a wandering monster (`hero_army_power` 0x427650): its army's value
/// (`army_value` 0x44AC80: each stack's AI value by `value` times its count) times
/// √((1 + 0.05 Defense)(1 + 0.05 Attack)) (`hero_attack_defense_factor` 0x4E5400, the skills
/// held in 0 to 99), truncated. *Provisional:* in integers, exact where the game's float factor
/// and its 32-bit sum are not (√ of value² (20 + D)(20 + A), over 20).
#[must_use]
pub fn power(
    army: &[Option<Stack>; 7],
    value: impl Fn(u16) -> i32,
    [attack, defense]: [i8; 2],
) -> i64 {
    let worth = army
        .iter()
        .flatten()
        .map(|s| i128::from(value(s.creature)) * i128::from(s.count))
        .fold(0i128, i128::saturating_add)
        .max(0);
    let factor = |skill: i8| 20 + i128::from(skill.clamp(0, 99));
    let squared = worth
        .saturating_mul(worth)
        .saturating_mul(factor(attack))
        .saturating_mul(factor(defense));
    let root = u128::try_from(squared).map_or(0, u128::isqrt);
    i64::try_from(root / 20).unwrap_or(i64::MAX)
}

/// The strength step of a hero's `power` over a monster's `strength` (0x4A780E to 0x4A7879, the
/// ratio r: 11 at 7 or more; ⌊2r − 2⌋ at 1 or more; −1 above 0.5, −2 above 0.333, else −3).
/// *Provisional:* in integers, exact where the game's single-precision ratio is not.
fn step(power: i64, strength: i64) -> i32 {
    let (p, s) = (i128::from(power), i128::from(strength.max(0)));
    if s == 0 || p >= 7 * s {
        11
    } else if p >= s {
        i32::try_from(2 * p / s - 2).unwrap_or(11)
    } else if 2 * p > s {
        -1
    } else if 1000 * p > 333 * s {
        -2
    } else {
        -3
    }
}

/// A hero's `army`'s sympathy for a monster of `creature` (`monster_sympathy` 0x4A75C0): 0 with
/// none of its type or the related one ([`PAIRED`]), 2 with more than half of its creatures, else
/// 1.
fn sympathy(army: &[Option<Stack>; 7], creature: u16) -> i32 {
    let related = (creature < PAIRED).then_some(creature ^ 1);
    let (similar, total) = army
        .iter()
        .flatten()
        .fold((0i64, 0i64), |(similar, total), s| {
            let count = i64::from(s.count);
            let like = s.creature == creature || Some(s.creature) == related;
            (similar + if like { count } else { 0 }, total + count)
        });
    match similar {
        0 => 0,
        _ if 2 * similar > total => 2,
        _ => 1,
    }
}

/// A question's facts (design §5.4's table; *provisional*: [`Ask::of`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ask {
    /// A guarded pile's, scroll's or artifact's with the map's text (`visit_guarded_resource`
    /// 0x4A4780, `visit_guarded_artifact` 0x49F070): the text; its OK the battle.
    Text(String),
    /// A guarded artifact's without a text (`visit_guarded_artifact` 0x49F070): GENRLTXT 421 with
    /// its one stack's size word (`creature_quantity_word` 0x44ADE0's form 2) and its name twice.
    Artifact(Stack),
    /// A neutral abandoned mine's (`visit_mine` 0x4A39A0: ADVEVENT 84, its Troglodytes in the
    /// text).
    Mine,
    /// A creature bank's (`visit_creature_bank` 0x4A15A0): its kind (the subclass) and whether it
    /// is looted (ADVEVENT 33, a box; else 32, the question).
    Bank {
        /// The kind.
        kind: u32,
        /// Looted.
        looted: bool,
    },
    /// A search kind's (`visit_search_bank` 0x4AC490: its class's first text, looted or not).
    Search(u32),
    /// The Tree of Knowledge's (`visit_tree_of_knowledge` 0x4A6710): its price in gems (else in
    /// gold), the hero's mark (ADVEVENT 147), the kingdom too poor (150, 152); else the price's
    /// question (149, 151) with the experience's picture.
    Tree {
        /// The price is in gems.
        gems: bool,
        /// The hero learned there.
        visited: bool,
        /// The kingdom cannot pay.
        poor: bool,
    },
    /// The Pyramid's (`visit_pyramid` 0x4A4230: ADVEVENT 105 at every visit).
    Pyramid,
    /// The Warrior's Tomb's (`visit_warriors_tomb` 0x4A7C30: ADVEVENT 161): its artifact (the
    /// word's bits 13 to 22 while its bit 0 is set), none; since E24 M5 T4c (design §5.29) the
    /// morale its search costs, a fact of its own (a recording's the tables' −3, [`Ask::of`];
    /// play's the record's `OnceMarkChanged`).
    Tomb {
        /// The artifact found.
        artifact: Option<u16>,
        /// The morale its search changes.
        morale: i8,
    },
    /// A dwelling's guards (`visit_dwelling` 0x4A18B0: GENRLTXT 422): the dwelling (the subclass)
    /// and its guards.
    Guarded {
        /// The dwelling.
        dwelling: u32,
        /// Its guards.
        guards: Stack,
    },
    /// A dwelling of the visitor's team (`visit_dwelling` 0x4A18B0: ADVEVENT 35): the dwelling,
    /// its creatures to hire, whether they join free (its own player's, level 1) and whether the
    /// hero's army has room for them. Since E24 M4 T2d1 (design §5.6) a dwelling of another team
    /// without guards too, its flag raised first (`set_dwelling_owner` 0x4C67B0 at 0x4A1B00, the
    /// visitor's then: `flag-dwelling`'s Pikemen free).
    Hire {
        /// The dwelling.
        dwelling: u32,
        /// Its creatures.
        joins: Stack,
        /// They join free.
        free: bool,
        /// The army has a slot of their kind or an empty one.
        room: bool,
        /// The visit raises the visitor's flag before it asks (E24 M4 T2d1).
        flag: bool,
    },
    /// A wandering monster's offer (part A2; `monster_visit_decision` 0x4A7740): its stack, the
    /// offer, whether the hero's army has room for it (`army_add_stack` 0x44ACE0: a slot of its
    /// kind or an empty one) and whether its flight is offered after the offer declined (its
    /// aggression not the decision's C, and it flees; else the insult and the fight).
    Monster {
        /// Its creature and count.
        stack: Stack,
        /// The offer.
        offer: Offer,
        /// The army has room for it.
        room: bool,
        /// Its flight follows a declined offer.
        flees: bool,
    },
    /// A Border Guard's passage, its colour's key held (part A2; `visit_object` 0x4A8734:
    /// ADVEVENT 17).
    Pass,
    /// A Cartographer's map (part A2; `visit_object` 0x4A8A14: ADVEVENT 25 + its kind): its kind
    /// (0 sea, 1 land, 2 underground); since E24 M5 T4c (design §5.29) its price and what it
    /// reveals, facts of their own (a recording's the tables', [`Ask::of`]; play's the engine's
    /// offer and the record's tiles).
    Maps {
        /// The kind.
        kind: u32,
        /// The price, `[resource, amount]`.
        price: [i32; 2],
        /// What it reveals.
        reveal: Reveal,
    },
    /// Pandora's Box (part A2; `visit_pandora_box` 0x4A0C50: ADVEVENT 14): what it gives and the
    /// hero its boxes name.
    OpenBox {
        /// What it gives.
        rewards: Rewards,
        /// The hero's name.
        hero: String,
    },
    /// A treasure chest's choice (part B; `chest_gold_or_experience` 0x4A6440: ADVEVENT 146,
    /// style 7 with the "or"): its gold (its word's bit 10 clear) or the experience, the gold
    /// less the tables' 500 (*provisional*: the hero's Learning not known, none).
    Chest {
        /// The gold.
        gold: i32,
        /// The experience.
        experience: i32,
    },
    /// The Arena's (part B; `visit_arena` 0x49E7D0: ADVEVENT 0, style 10), not visited by the
    /// hero: the tables' gain to Attack or to Defense; its Cancel the Attack's (MCL-Q33).
    Arena(i8),
    /// A School of Magic's (part B; `visit_school_of_magic` 0x4A33E0: ADVEVENT 71, style 10), not
    /// visited by the hero and its price held: the gain to Spell Power or to Knowledge for the
    /// price, or neither (the Cancel).
    School {
        /// The gain.
        gain: i8,
        /// The price, `[resource, amount]`.
        price: [i32; 2],
    },
    /// Live, the notice of a decision the view does not ask (E24 M5 T4c, design §5.28,
    /// `MCL-STUB-7`; [`crate::notice`]): a box of style 1 in the client's own words, its OK the
    /// host's closing answer (no answer of the client's: [`Ask::yes`] none).
    Notice(Notice),
}

/// A monster's aggression in its `word` (bits 12 to 16, signed: −16 to 15).
fn aggression(word: u32) -> i32 {
    let raw = i32::try_from((word >> AGGRESSION) & 0x1F).unwrap_or(0);
    if raw >= 16 { raw - 32 } else { raw }
}

impl Ask {
    /// The question an object of `seen` asks `visitor` (design §5.4), by the tables' `art`; none
    /// for another class or one this part does not build (*Not built* above).
    #[must_use]
    pub fn of(seen: &Seen, visitor: &Visitor, art: &AskArt) -> Option<Self> {
        let [class, subclass] = seen.class;
        let bit = |n: u32| seen.word.is_some_and(|w| (w >> n) & 1 == 1);
        Some(match class {
            ARTIFACT | PILE | SCROLL => {
                let (text, guards) = seen.message.as_ref()?;
                match guards.as_slice() {
                    [] => return None,
                    _ if !text.is_empty() => Self::Text(text.clone()),
                    [stack] if class == ARTIFACT => Self::Artifact(*stack),
                    _ => return None,
                }
            }
            MINE if subclass == ABANDONED && seen.owner.is_none() => Self::Mine,
            BANK => Self::Bank {
                kind: subclass,
                looted: bit(LOOTED),
            },
            DERELICT_SHIP | CRYPT | SHIPWRECK => Self::Search(class),
            TREE => {
                let price = seen.word.map_or(0, |w| (w >> 13) & 7);
                let cost = usize::try_from(price)
                    .ok()
                    .and_then(|p| art.tree_prices.get(p.checked_sub(1)?));
                match cost {
                    _ if seen.visited => Self::Tree {
                        gems: price == 2,
                        visited: true,
                        poor: false,
                    },
                    Some(&price_of) => Self::Tree {
                        gems: price == 2,
                        visited: false,
                        poor: !visitor.has(price_of),
                    },
                    None => return None,
                }
            }
            PYRAMID => Self::Pyramid,
            TOMB => Self::Tomb {
                artifact: seen
                    .word
                    .filter(|_| bit(0))
                    .and_then(|w| u16::try_from((w >> 13) & 0x3FF).ok()),
                morale: art.tomb_morale,
            },
            DWELLING => {
                let (creature, growth) = seen.dwelling?;
                let level = art.level(creature);
                let team = |c: u8| visitor.teams.get(usize::from(c)).copied();
                let ours = seen.owner.is_some_and(|o| team(o) == team(visitor.colour));
                let [least, weeks] = art.guards;
                // Another team's without guards (*provisional*: a new game's, a creature below
                // level 5) is the visitor's first (E24 M4 T2d1), then asks the same.
                let flag = !ours && i32::from(level) < least;
                if ours || flag {
                    let room = visitor.room(creature);
                    let own = flag || seen.owner == Some(visitor.colour);
                    Self::Hire {
                        dwelling: subclass,
                        joins: Stack {
                            creature,
                            count: growth,
                        },
                        free: own && level == 1,
                        room,
                        flag,
                    }
                } else {
                    Self::Guarded {
                        dwelling: subclass,
                        guards: Stack {
                            creature,
                            count: weeks.saturating_mul(growth),
                        },
                    }
                }
            }
            MONSTER if !bit(MESSAGE) => {
                let traits = seen.monster?;
                let word = seen.word.unwrap_or(0);
                let creature = u16::try_from(subclass).unwrap_or(u16::MAX);
                let stack = Stack {
                    creature,
                    count: i32::try_from(word & COUNT).unwrap_or(0),
                };
                Self::monster(stack, aggression(word), bit(NEVER_FLEES), traits, visitor)?
            }
            BORDER_GUARD if seen.key => Self::Pass,
            CARTOGRAPHER if !seen.visited => {
                let reveal = usize::try_from(subclass)
                    .ok()
                    .and_then(|k| art.reveals.get(k))?;
                if !visitor.has(art.maps_price) {
                    return None;
                }
                Self::Maps {
                    kind: subclass,
                    price: art.maps_price,
                    reveal: Reveal::Terrains(*reveal),
                }
            }
            CHEST if !bit(CHEST_ARTIFACT) => {
                let steps = seen.word.map_or(0, |w| (w >> CHEST_GOLD) & 0xF);
                let [step, less] = art.chest;
                let gold = i32::try_from(steps).unwrap_or(0).saturating_mul(step);
                Self::Chest {
                    gold,
                    experience: gold.saturating_sub(less),
                }
            }
            ARENA if !seen.visited => Self::Arena(art.arena),
            SCHOOL if !seen.visited => {
                if !visitor.has(art.school_price) {
                    return None;
                }
                Self::School {
                    gain: art.school,
                    price: art.school_price,
                }
            }
            PANDORA if seen.message.is_none() => {
                let rewards = seen.rewards.clone()?;
                if rewards == Rewards::default() {
                    return None;
                }
                Self::OpenBox {
                    rewards,
                    hero: visitor.hero.clone(),
                }
            }
            _ => return None,
        })
    }

    /// A wandering monster's offer to `visitor` (`monster_visit_decision` 0x4A7740; the battles'
    /// survey §3.2): with C its strength step ([`step`] of the hero's power over the stack's AI
    /// value times its count) plus the sympathy Sy ([`sympathy`]) plus the Diplomacy L, an
    /// aggression above C fights; else at most Sy + L + 1 offers to join free, at most
    /// Sy + 2L + 1 for gold when the kingdom can pay; else the flight unless the aggression is C or
    /// the monster never flees (the fight). None for a fight (not built).
    fn monster(
        stack: Stack,
        aggression: i32,
        never_flees: bool,
        traits: Traits,
        visitor: &Visitor,
    ) -> Option<Self> {
        let strength = i64::from(traits.value) * i64::from(stack.count);
        let sy = sympathy(&visitor.army, stack.creature);
        let l = i32::from(visitor.diplomacy);
        let c = step(visitor.power, strength) + sy + l;
        if aggression > c {
            return None;
        }
        let flees = aggression != c && !never_flees;
        let price = traits.gold.saturating_mul(stack.count);
        let offer = if aggression <= sy + l + 1 {
            Offer::Join
        } else if aggression <= sy + 2 * l + 1 && visitor.resources[GOLD] >= price {
            Offer::Gold(price)
        } else if flees {
            Offer::Flee
        } else {
            return None;
        };
        let room = visitor.room(stack.creature);
        Some(Self::Monster {
            stack,
            offer,
            room,
            flees,
        })
    }

    /// Whether its box is a question (style 2, its OK and Cancel); a looted bank's and the Tree's
    /// visited and too poor ones are an OK's alone (style 1), as a notice's (E24 M5 T4c).
    #[must_use]
    pub const fn asks(&self) -> bool {
        !matches!(
            self,
            Self::Bank { looted: true, .. }
                | Self::Tree { visited: true, .. }
                | Self::Tree { poor: true, .. }
                | Self::Notice(_)
        )
    }

    /// Whether its handler builds the status window again before its box (`status_window_choose`
    /// 0x415DE0 with 0, 1, 1: the pile's, the artifact's, the mine's, the bank's, the dwelling's
    /// and the Pyramid's; *provisional* for a dwelling's hire, as its guards'; a notice's not, E24
    /// M5 T4c: no original).
    #[must_use]
    pub const fn chooses(&self) -> bool {
        !matches!(
            self,
            Self::Search(_)
                | Self::Tree { .. }
                | Self::Tomb { .. }
                | Self::Monster { .. }
                | Self::Pass
                | Self::Maps { .. }
                | Self::OpenBox { .. }
                | Self::Notice(_)
        )
    }

    /// Whether its object blocks its entrance (a pile, a scroll, an artifact; since part A2 a
    /// monster, a Border Guard, Pandora's Box; since part B a chest): a key's step onto it visits
    /// it from beside. A notice's by its tile (E24 M5 T4c: `crate::engine::visits::lay`), none
    /// here.
    #[must_use]
    pub const fn beside(&self) -> bool {
        matches!(
            self,
            Self::Text(_)
                | Self::Artifact(_)
                | Self::Monster { .. }
                | Self::Pass
                | Self::OpenBox { .. }
                | Self::Chest { .. }
        )
    }

    /// Whether the visit makes the object the visitor's before it asks (E24 M4 T2d1, design §5.6:
    /// a dwelling of another team without guards, `set_dwelling_owner` 0x4C67B0).
    #[must_use]
    pub const fn flags(&self) -> bool {
        matches!(self, Self::Hire { flag: true, .. })
    }

    /// Its box's style (part B): a chest's 7, the Arena's and a School's 10 (their pictures
    /// chosen), every other question's 2, a box that asks nothing 1 ([`Ask::asks`]).
    #[must_use]
    pub const fn style(&self) -> Style {
        match self {
            Self::Chest { .. } => Style::Choose,
            Self::Arena(_) | Self::School { .. } => Style::ChooseCancel,
            _ if self.asks() => Style::OkCancel,
            _ => Style::Ok,
        }
    }

    /// The "or" its box shows between its pictures (part B: a chest's, the call's flag; GENRLTXT
    /// 5) through `texts`; none for another.
    #[must_use]
    pub fn or(&self, texts: &Catalogue) -> Option<String> {
        matches!(self, Self::Chest { .. }).then(|| texts.text("map.ask.or", &Args::new()))
    }

    /// A monster's picture while the hero visits it (part A2): twice its creature, the attack
    /// DEF's frame with the hero not left of it ([`crate::live::Attacking`]); none for another
    /// question.
    #[must_use]
    pub fn attacker(&self) -> Option<u32> {
        match self {
            Self::Monster { stack, .. } => Some(u32::from(stack.creature) * 2),
            _ => None,
        }
    }

    /// Its box's text and pictures (design §5.4's table) through `texts`.
    #[must_use]
    pub fn told(&self, texts: &Catalogue) -> Told {
        let alone = |text: String| Told {
            text,
            pictures: Vec::new(),
        };
        let plain = |key: &str| alone(texts.text(key, &Args::new()));
        match self {
            Self::Text(text) => alone(text.clone()),
            Self::Artifact(guards) => {
                let args = Args::new()
                    .text("size", &size(texts, guards.count))
                    .text("creature", &name(texts, guards.creature, 2));
                alone(texts.text("map.ask.artifact", &args))
            }
            Self::Mine => plain("map.ask.mine"),
            Self::Bank { kind, looted } => {
                let args = Args::new().text("bank", &family(texts, "bank", *kind));
                let key = if *looted {
                    "map.ask.bank-looted"
                } else {
                    "map.ask.bank"
                };
                alone(texts.text(key, &args))
            }
            Self::Search(class) => plain(match *class {
                DERELICT_SHIP => "map.ask.derelict-ship",
                CRYPT => "map.ask.crypt",
                _ => "map.ask.shipwreck",
            }),
            Self::Tree {
                gems,
                visited,
                poor,
            } => match (*visited, *poor, *gems) {
                (true, ..) => plain("map.ask.tree-visited"),
                (false, true, false) => plain("map.ask.tree-poor-gold"),
                (false, true, true) => plain("map.ask.tree-poor-gems"),
                (false, false, gems) => Told {
                    text: texts.text(
                        if gems {
                            "map.ask.tree-gems"
                        } else {
                            "map.ask.tree-gold"
                        },
                        &Args::new(),
                    ),
                    pictures: vec![EXPERIENCE],
                },
            },
            Self::Pyramid => plain("map.ask.pyramid"),
            Self::Tomb { .. } => plain("map.ask.tomb"),
            Self::Guarded { dwelling, guards } => {
                let args = Args::new()
                    .text("dwelling", &family(texts, "dwelling1", *dwelling))
                    .text("size", &size(texts, guards.count))
                    .text("creature", &name(texts, guards.creature, 2));
                alone(texts.text("map.ask.guarded", &args))
            }
            Self::Hire {
                dwelling, joins, ..
            } => {
                let args = Args::new()
                    .text("dwelling", &family(texts, "dwelling1", *dwelling))
                    .text("creature", &name(texts, joins.creature, 2));
                alone(texts.text("map.ask.hire", &args))
            }
            Self::Monster { stack, offer, .. } => {
                let plural = Args::new().text("creature", &name(texts, stack.creature, 2));
                match *offer {
                    Offer::Join => alone(texts.text("map.ask.join", &plural)),
                    Offer::Flee => alone(texts.text("map.ask.flee", &plural)),
                    Offer::Gold(price) => {
                        let gold = i64::from(price);
                        let text = if stack.count == 1 {
                            let args = Args::new()
                                .text("creature", &name(texts, stack.creature, 1))
                                .number("price", gold);
                            texts.text("map.ask.join-one", &args)
                        } else {
                            // 89 ends with its blank line, 90 follows it (`monster_join_for_gold`
                            // 0x4A7250).
                            let args = plural
                                .number("amount", i64::from(stack.count))
                                .number("price", gold);
                            let mut text = texts.text("map.ask.join-gold", &Args::new());
                            text.push_str(&texts.text("map.ask.join-price", &args));
                            text
                        };
                        Told {
                            text,
                            pictures: vec![(GOLD_PICTURE, price)],
                        }
                    }
                }
            }
            Self::Pass => plain("map.ask.pass"),
            Self::Maps { kind, .. } => plain(match *kind {
                0 => "map.ask.maps-sea",
                1 => "map.ask.maps-land",
                _ => "map.ask.maps-underground",
            }),
            Self::OpenBox { .. } => plain("map.ask.open-box"),
            Self::Chest { gold, experience } => Told {
                text: texts.text("map.ask.chest", &Args::new()),
                pictures: vec![(GOLD_PICTURE, *gold), (EXPERIENCE.0, *experience)],
            },
            Self::Arena(gain) => Told {
                text: texts.text("map.ask.arena", &Args::new()),
                pictures: vec![(PRIMARY, i32::from(*gain)), (PRIMARY + 1, i32::from(*gain))],
            },
            Self::School { gain, .. } => Told {
                text: texts.text("map.ask.school", &Args::new()),
                pictures: vec![
                    (PRIMARY + 2, i32::from(*gain)),
                    (PRIMARY + 3, i32::from(*gain)),
                ],
            },
            Self::Notice(notice) => plain(notice.key()),
        }
    }

    /// What its OK shows where the viewer builds it ([`Reply`]): A1's boxes (style 1) and what
    /// the panels show at the last one's close (recorded, `ask-tomb` and `ask-hire` at 1660), the
    /// artifacts' skills `art`'s. The Warrior's Tomb's: ADVEVENT 162 with its artifact (its
    /// skills, the artifact taken as worn), else 163 with three pictures of bad morale; its
    /// morale lost either way. A dwelling's creatures joining free (`visit_dwelling`
    /// 0x4A18B0's own path): GENRLTXT 424 with their count and name (one's the singular,
    /// *provisional*: not recorded) and the army joined, else 423 (none to hire) or 426 (no room).
    /// Part A2's: a monster's join (its price paid, the army joined, the status window built
    /// again, the monster taken with `killfade.82M`; recorded, `ask-join`, `ask-join-gold`), a
    /// Border Guard taken with a pickup's sound (`ask-guard`), a Cartographer's price paid and
    /// what it reveals (`ask-cartographer`), Pandora's Box's
    /// `grant_rewards` boxes without the map's text (`finds::rewards`), then the box taken with a
    /// pickup's sound (`ask-pandora`). Part B's: a choice's first picture chosen, a chest's gold
    /// given and the chest taken with a pickup's sound (`choose-chest-gold`), the Arena's gain to
    /// Attack, a School's to Spell Power for its price ([`Ask::chosen`]). None for every other
    /// answer (*Not built* above).
    ///
    /// Since E24 M5 T4c (design §5.29, the facts audit) every number it shows is the question's
    /// own fact (the Tomb's morale, a Cartographer's price and reveal: no [`AskArt`] here, so play,
    /// whose questions the engine's decision and record give, reads no table of the client's
    /// but the artifacts' skills, MCL-Q65).
    #[must_use]
    pub fn yes(&self, art: &FindArt, texts: &Catalogue) -> Option<Reply> {
        let told = |told: Told, gain: Gain| Reply {
            boxes: vec![Reward {
                told,
                shows: Gain::default(),
            }],
            gain,
            ..Reply::default()
        };
        match self {
            Self::Tomb {
                artifact: Some(artifact),
                morale,
            } => {
                let named = texts
                    .known(&format!("artifact.{artifact}"), &Args::new())
                    .unwrap_or_default();
                let args = Args::new().text("artifact", &named);
                Some(told(
                    Told {
                        text: texts.text("map.ask.tomb-found", &args),
                        pictures: vec![(ARTIFACT_PICTURE, i32::from(*artifact))],
                    },
                    Gain {
                        skills: art.skills_of(*artifact),
                        morale: [*morale; 2],
                        ..Gain::default()
                    },
                ))
            }
            Self::Tomb {
                artifact: None,
                morale,
            } => Some(told(
                Told {
                    text: texts.text("map.ask.tomb-empty", &Args::new()),
                    pictures: vec![BAD_MORALE; 3],
                },
                Gain {
                    morale: [*morale; 2],
                    ..Gain::default()
                },
            )),
            Self::Hire {
                joins,
                free: true,
                room,
                ..
            } => {
                let (key, count) = match (joins.count, room) {
                    (0, _) => ("map.ask.none", 2),
                    (_, false) => ("map.ask.no-room", 2),
                    (count, true) => ("map.ask.joined", count),
                };
                let args = Args::new()
                    .number("amount", i64::from(joins.count))
                    .text("creature", &name(texts, joins.creature, count));
                let mut gain = Gain::default();
                if key == "map.ask.joined" {
                    gain.army[0] = Some(*joins);
                }
                Some(told(
                    Told {
                        text: texts.text(key, &args),
                        pictures: Vec::new(),
                    },
                    gain,
                ))
            }
            Self::Monster {
                stack,
                offer,
                room: true,
                ..
            } => {
                let mut gain = Gain::default();
                match *offer {
                    Offer::Join => {}
                    Offer::Gold(price) => gain.resources[GOLD] = price.saturating_neg(),
                    Offer::Flee => return None,
                }
                gain.army[0] = Some(*stack);
                Some(Reply {
                    gain,
                    chooses: true,
                    taken: Some(Fade::Killfade),
                    ..Reply::default()
                })
            }
            Self::Pass => Some(Reply {
                taken: Some(Fade::Pickup),
                ..Reply::default()
            }),
            Self::Maps {
                price: [resource, amount],
                reveal,
                ..
            } => {
                let mut gain = Gain::default();
                if let Some(paid) = usize::try_from(*resource)
                    .ok()
                    .and_then(|r| gain.resources.get_mut(r))
                {
                    *paid = amount.saturating_neg();
                }
                Some(Reply {
                    gain,
                    reveal: reveal.clone(),
                    ..Reply::default()
                })
            }
            Self::OpenBox { rewards, hero } => {
                let (boxes, gain) = finds::rewards(rewards, texts, hero, "");
                Some(Reply {
                    boxes,
                    granted: true,
                    gain,
                    taken: Some(Fade::Pickup),
                    ..Reply::default()
                })
            }
            Self::Chest { .. } | Self::Arena(_) | Self::School { .. } => self.chosen(0),
            _ => None,
        }
    }

    /// What a choice's second picture chosen does (part B; 0x780A): a chest's experience (nothing
    /// the panels show; since E24 M4 T2c its levels' dialogs at the dissolve's end) and the chest
    /// taken with a pickup's sound (`choose-chest`, `level-up-twice`), the Arena's gain to Defense (`choose-arena`), a School's to Knowledge
    /// for its price (`choose-school`); none for a question.
    #[must_use]
    pub fn second(&self) -> Option<Reply> {
        self.chosen(1)
    }

    /// A choice's picture `n` (0 or 1) chosen (part B): the gain, a School's price paid, a
    /// chest's taken.
    fn chosen(&self, n: usize) -> Option<Reply> {
        let mut gain = Gain::default();
        let mut after = Gain::default();
        let mut taken = None;
        match *self {
            Self::Chest { gold, experience } => {
                if n == 0 {
                    gain.resources[GOLD] = gold;
                } else {
                    after.experience = experience;
                }
                taken = Some(Fade::Pickup);
            }
            Self::Arena(more) => gain.skills[n] = more,
            Self::School {
                gain: more,
                price: [resource, amount],
            } => {
                gain.skills[2 + n] = more;
                if let Some(paid) = usize::try_from(resource)
                    .ok()
                    .and_then(|r| gain.resources.get_mut(r))
                {
                    *paid = amount.saturating_neg();
                }
            }
            _ => return None,
        }
        Some(Reply {
            gain,
            taken,
            after,
            ..Reply::default()
        })
    }

    /// What its Cancel does (part A2): a monster's declined offer its flight's question when it
    /// flees (recorded, `ask-join-no`: 91 in the Cancel's millisecond), its declined flight the
    /// monster taken off the map without its treasure with `killfade.82M` (`monster_flee_offer`
    /// 0x4A6DF0); none where it fights (the insult at the decision's C, 87, then the battle: not
    /// built); every other question's ends the visit ([`Reply::default`]). Part B's: the Arena's
    /// Cancel gives the Attack as its first picture does and marks it (MCL-Q33: every answer but
    /// 0x780A; `choose-arena-cancel`), a School's leaves (`choose-school-leave`), a chest's box
    /// has none.
    #[must_use]
    pub fn no(&self) -> Option<Reply> {
        match self {
            Self::Arena(_) => self.chosen(0).map(|reply| Reply {
                marks: true,
                ..reply
            }),
            Self::Monster {
                offer: Offer::Flee, ..
            } => Some(Reply {
                taken: Some(Fade::Killfade),
                ..Reply::default()
            }),
            Self::Monster {
                stack,
                room,
                flees: true,
                ..
            } => Some(Reply {
                then: Some(Box::new(Self::Monster {
                    stack: *stack,
                    offer: Offer::Flee,
                    room: *room,
                    flees: true,
                })),
                ..Reply::default()
            }),
            Self::Monster { .. } => None,
            _ => Some(Reply::default()),
        }
    }
}

/// The size word of `count` in form 2 (`creature_quantity_word` 0x44ADE0: `army_size_words`
/// 0x6A5BB8's lower-cased form, ARRAYTXT's records 176 + 3 n).
fn size(texts: &Catalogue, count: i32) -> String {
    let count = u32::try_from(count).unwrap_or(0);
    let n = SIZES.iter().take_while(|&&s| count >= s).count();
    texts.text(&format!("map.ask.size.{n}"), &Args::new())
}

/// Creature `creature`'s name for `count` (the singular for exactly one, else the plural).
fn name(texts: &Catalogue, creature: u16, count: i32) -> String {
    let args = Args::new().count(if count == 1 { 1 } else { 2 });
    texts
        .known(&format!("creature.{creature}"), &args)
        .unwrap_or_default()
}

/// The `family`'s entry `n`'s name (empty past the family).
fn family(texts: &Catalogue, family: &str, n: u32) -> String {
    texts
        .known(&format!("{family}.{n}"), &Args::new())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
