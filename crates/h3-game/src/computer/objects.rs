//! The computer's object value (E31 T2h; survey `docs/rules/surveys/E31-computer-player.md`
//! §2.1): `ai_object_value` 0x528040, what the object whose entrance is a tile is worth to one of
//! the computer's heroes, in the fight estimates' units, for the planning's candidates (T2i) and
//! the answers to a guarded object (`ai_fight_answer` 0x52BD10, increment 4). Its arguments are
//! the hero, the candidate's cost (an `i32` some classes lower or replace: the movement objects
//! and the Swan Pond) and the tile; nothing here changes the game.
//!
//! The original switches on the object's class (+0x1E, 4 to 113): 67 handlers for 73 classes,
//! every other class 0. The engine knows an object by its type's kind (ADR-0016 point 4), so the
//! class is read from the kind and its parameters ([`Class::of`], CPU-OBJ-001): a kind of one
//! class is that class; the bonus visitables, the once-per-hero gifts and the finds are told
//! apart by what they give. The per-class values are the original's, its constants included:
//! the computer's expectations of an object (the Flotsam's 175 gold and 5 wood, the Treasure
//! Chest's tiers) are its own, not the object's data. The creature-holding classes are in
//! `fights`, the visits and pickups in `visits`; the vision circle of the Redwood Observatory
//! and the Pillar of Fire is `super::vision`.
//!
//! Not here: the Hut of the Magi (37), the Obelisk (57) and the Seer's Hut (83), which the engine
//! does not have (`AI-STUB-20`: E12, E09 increment 8); the victory conditions' musts of the Mine
//! and the Monster (`AI-STUB-18`, E10); a campaign's garrisons (`AI-STUB-21`, E20); the artifact's
//! pickup conditions 1 to 6, which no map reaches (`E08-rewards.md` §3.3: the loader and the new
//! game leave the nibble 0); `ai_join_without_room` 0x52BC60, which goes with its
//! `ai_army_plan_run` 0x42D690 to increment 3.

mod fights;
mod visits;

#[cfg(test)]
mod tests;

use h3_adventure::{
    Find, Gives, HeroOnMap, MarketScreen, MonolithWay, ObjectKind, ObjectTypeId, Pos, Turn,
};
use h3_content::{CreatureFlag, ResourceId};
use h3_heroes::PrimarySkill;
use h3_kernel::arith::{SINGLE_BITS, SignedX87};

use super::artifacts::{self, SLIP};
use super::towns::Asked;
use super::turn::primary_sum_of;
use crate::{GameRuleset, GameState};

/// A fight lost by this much or more is returned alone (0x52864B and every guarded class's test:
/// −500 000 000).
const LOST: i32 = -500_000_000;

/// The artifacts a hero carries in its backpack at most (`hero_backpack_count` 0x4D90C0 against
/// 64).
const BACKPACK: usize = 64;

/// The skills a hero knows at most (+0x101 against 8).
const SKILLS: usize = 8;

/// The value of a movement object whose bonus covers the path (0x528A3C: 10 000), the cost then
/// 0.
const COVERED: i32 = 10_000;

/// The original's wood and gems (the resources 0 and 5: the base pack's handles are the
/// original's ids).
const WOOD: ResourceId = ResourceId::from_raw(0);
const GEMS: ResourceId = ResourceId::from_raw(5);

/// The original's object classes the computer's values tell apart (`ai_object_value` 0x528040's
/// switch and `ai_class_scout_values` 0x6925AC; `docs/formats/h3m.md`'s class numbers), each its
/// class number, one variant for classes of one value (the dwellings 17 and 20, the
/// observatories 58 and 60, the Shrines 88 to 90, the banks 16, 24, 25, 84 and 85, the Marletto
/// Tower 23 and the Mercenary Camp 51, the Faerie Ring 28 and the Mermaids 52, the Fountain of
/// Youth 31, the Oasis 56 and the Watering Hole 110). [`Class::Other`] is every class worth 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Class {
    /// Any class the switch values at 0 and the scouting does not list.
    Other = 0,
    Altar = 2,
    Arena = 4,
    Artifact = 5,
    Pandora = 6,
    BlackMarket = 7,
    Boat = 8,
    Keymaster = 10,
    Buoy = 11,
    Campfire = 12,
    Cartographer = 13,
    SwanPond = 14,
    /// A creature bank, the Derelict Ship, the Dragon Utopia, the Crypt, the Shipwreck.
    Bank = 16,
    Dwelling = 17,
    Corpse = 22,
    /// The Marletto Tower and the Mercenary Camp: one level's experience.
    OneLevel = 23,
    /// The Faerie Ring and the Mermaids: one luck.
    FaerieRing = 28,
    Flotsam = 29,
    FountainOfFortune = 30,
    GardenOfRevelation = 32,
    Garrison = 33,
    Hero = 34,
    HillFort = 35,
    IdolOfFortune = 38,
    LeanTo = 39,
    Library = 41,
    Lighthouse = 42,
    MonolithEntrance = 43,
    MonolithTwoWay = 45,
    SchoolOfMagic = 47,
    MagicSpring = 48,
    MagicWell = 49,
    Mine = 53,
    Monster = 54,
    MysticalGarden = 55,
    /// The Fountain of Youth, the Oasis and the Watering Hole: one morale and the cost lowered.
    Oasis = 56,
    Observatory = 58,
    StarAxis = 61,
    Prison = 62,
    Pyramid = 63,
    RallyFlag = 64,
    RefugeeCamp = 78,
    Resource = 79,
    Scholar = 81,
    SeaChest = 82,
    Survivor = 86,
    Shipyard = 87,
    Shrine = 88,
    Sirens = 92,
    Scroll = 93,
    Stables = 94,
    Temple = 96,
    Town = 98,
    LearningStone = 100,
    TreasureChest = 101,
    Tree = 102,
    University = 104,
    Wagon = 105,
    WarMachineFactory = 106,
    SchoolOfWar = 107,
    WarriorsTomb = 108,
    WaterWheel = 109,
    Whirlpool = 111,
    Windmill = 112,
    WitchHut = 113,
}

impl Class {
    /// The class of an object of kind `kind` (CPU-OBJ-001): a kind of one class is it; a bonus
    /// visitable by its gift (draining the points: the Swan Pond; points and luck: the Rally
    /// Flag; points alone: the Oasis's three; a week of changing gifts: the Idol with luck, the
    /// Temple without; else one morale, the Buoy, or one luck, the Faerie Ring and the
    /// Mermaids); a once-per-hero gift by what it gives (a free choice: the Arena; a paid one: a
    /// School, of Magic when it offers Spell Power or Knowledge; the Library's condition; a
    /// primary skill: Spell Power the Star Axis, Knowledge the Garden, Attack or Defense one
    /// level; experience the Learning Stone; a level the Tree); a find by its shape (a question
    /// or a bonus: the Warrior's Tomb; one that leaves the map, the sea's: the Survivor loses an
    /// artifact, the Sea Chest gives gold, the Flotsam holds none; else the Wagon's drawn
    /// resources, the Corpse's gold, the Lean-To without an artifact). The classes the
    /// original's switch values at 0 and its scouting does not list are [`Class::Other`].
    // rule: CPU-OBJ-001
    pub(super) fn of(kind: &ObjectKind) -> Self {
        match kind {
            ObjectKind::Pickup(h3_adventure::Pickup::Paired { .. }) => Self::Campfire,
            ObjectKind::Pickup(_) => Self::Resource,
            ObjectKind::RevealTerrain { .. } => Self::Cartographer,
            ObjectKind::RevealArea { .. } => Self::Observatory,
            ObjectKind::KeyTent { .. } => Self::Keymaster,
            ObjectKind::Monster { .. } => Self::Monster,
            ObjectKind::Chest(_) => Self::TreasureChest,
            ObjectKind::Artifact { .. } => Self::Artifact,
            ObjectKind::Scroll => Self::Scroll,
            ObjectKind::Gift(gift) => Self::of_gift(gift),
            ObjectKind::Shrine { .. } => Self::Shrine,
            ObjectKind::WitchHut { .. } => Self::WitchHut,
            ObjectKind::Scholar => Self::Scholar,
            ObjectKind::University { .. } => Self::University,
            ObjectKind::Find(find) => Self::of_find(find),
            ObjectKind::PandoraBox => Self::Pandora,
            ObjectKind::Pyramid(_) => Self::Pyramid,
            ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. } => Self::Mine,
            ObjectKind::Lighthouse { .. } => Self::Lighthouse,
            ObjectKind::Shipyard { .. } => Self::Shipyard,
            ObjectKind::Windmill { .. } => Self::Windmill,
            ObjectKind::WaterWheel { .. } => Self::WaterWheel,
            ObjectKind::Garden { .. } => Self::MysticalGarden,
            ObjectKind::Dwelling { .. } => Self::Dwelling,
            ObjectKind::RefugeeCamp { .. } => Self::RefugeeCamp,
            ObjectKind::Town { .. } => Self::Town,
            ObjectKind::Monolith { way, .. } => match way {
                MonolithWay::Entrance => Self::MonolithEntrance,
                MonolithWay::TwoWay => Self::MonolithTwoWay,
                MonolithWay::Exit => Self::Other,
            },
            ObjectKind::Whirlpool { .. } => Self::Whirlpool,
            ObjectKind::Garrison { .. } => Self::Garrison,
            ObjectKind::Bonus {
                days,
                points,
                drain,
                ..
            } => Self::of_bonus(days, *points, *drain),
            ObjectKind::FountainOfFortune { .. } => Self::FountainOfFortune,
            ObjectKind::MagicWell => Self::MagicWell,
            ObjectKind::MagicSpring => Self::MagicSpring,
            ObjectKind::Stables { .. } => Self::Stables,
            ObjectKind::Sirens { .. } => Self::Sirens,
            ObjectKind::Bank(_) => Self::Bank,
            ObjectKind::Market(market) if market.shows(MarketScreen::Buy) => Self::BlackMarket,
            ObjectKind::Altar(_) => Self::Altar,
            ObjectKind::HillFort(_) => Self::HillFort,
            ObjectKind::WarMachineFactory(_) => Self::WarMachineFactory,
            ObjectKind::Prison => Self::Prison,
            ObjectKind::KeyGuard { .. }
            | ObjectKind::KeyGate { .. }
            | ObjectKind::Ground { .. }
            | ObjectKind::Event
            | ObjectKind::Message { .. }
            | ObjectKind::SeaGround
            | ObjectKind::Hole
            | ObjectKind::RandomDwelling
            | ObjectKind::SubterraneanGate
            | ObjectKind::Tavern
            | ObjectKind::DenOfThieves
            | ObjectKind::Market(_)
            | ObjectKind::Empty => Self::Other,
            // stub: AI-STUB-20
        }
    }

    /// A bonus visitable's class ([`Class::of`]).
    fn of_bonus(days: &[(i8, i8)], points: u32, drain: bool) -> Self {
        let luck = days.iter().any(|&(_, l)| l != 0);
        let changes = days.windows(2).any(|pair| pair[0] != pair[1]);
        match (drain, points > 0, changes, luck) {
            (true, ..) => Self::SwanPond,
            (false, true, _, true) => Self::RallyFlag,
            (false, true, _, false) => Self::Oasis,
            (false, false, true, true) => Self::IdolOfFortune,
            (false, false, true, false) => Self::Temple,
            (false, false, false, false) => Self::Buoy,
            (false, false, false, true) => Self::FaerieRing,
        }
    }

    /// A once-per-hero gift's class ([`Class::of`]).
    fn of_gift(gift: &h3_adventure::Gift) -> Self {
        let only = |p: [u8; 4], skill: usize| {
            p.iter()
                .enumerate()
                .all(|(at, &gain)| (at == skill) == (gain > 0))
        };
        match gift.gives {
            Gives::Experience(_) => Self::LearningStone,
            Gives::Level => Self::Tree,
            Gives::Choice { .. } if gift.prices.is_empty() => Self::Arena,
            Gives::Choice { skills, .. } => {
                if skills
                    .iter()
                    .any(|&s| matches!(s, PrimarySkill::Power | PrimarySkill::Knowledge))
                {
                    Self::SchoolOfMagic
                } else {
                    Self::SchoolOfWar
                }
            }
            Gives::Primary(_) if gift.admits.is_some() => Self::Library,
            Gives::Primary(p) if only(p, 2) => Self::StarAxis,
            Gives::Primary(p) if only(p, 3) => Self::GardenOfRevelation,
            Gives::Primary([_, _, 0, 0]) => Self::OneLevel,
            Gives::Primary(_) => Self::Other,
        }
    }

    /// A find's class ([`Class::of`]).
    fn of_find(find: &Find) -> Self {
        if find.question || find.bonus.is_some() {
            return Self::WarriorsTomb;
        }
        match (find.leaves, &find.full_backpack) {
            (true, Some(Turn::Lost | Turn::Stays)) => Self::Survivor,
            (true, Some(_)) => Self::SeaChest,
            (true, None) => Self::Flotsam,
            (false, Some(Turn::Drawn)) => Self::Wagon,
            (false, Some(_)) => Self::Corpse,
            (false, None) => Self::LeanTo,
        }
    }
}

/// What stands at a tile for the values (the original's map item +0x1E and its entrance bit
/// +0xD 0x10): a hero (the class 34 its tile takes), an empty boat (8), or the object visited from
/// it (the last one still on the map, as a visit finds it), with its type and class.
#[derive(Debug, Clone, Copy)]
pub(super) enum At<'a> {
    /// A hero stands there.
    Hero(&'a HeroOnMap),
    /// An empty boat lies there.
    Boat,
    /// The object visited from the tile.
    Object {
        /// Its index.
        index: usize,
        /// Its type.
        ty: ObjectTypeId,
        /// Its type's kind.
        kind: &'a ObjectKind,
        /// Its class.
        class: Class,
    },
}

impl Class {
    /// Its class number.
    pub(super) const fn number(self) -> u8 {
        self as u8
    }
}

impl At<'_> {
    /// Its class.
    pub(super) const fn class(&self) -> Class {
        match self {
            Self::Hero(_) => Class::Hero,
            Self::Boat => Class::Boat,
            Self::Object { class, .. } => *class,
        }
    }
}

/// What stands at `pos` (CPU-OBJ-001): a hero on it, else an empty boat, else the object a visit
/// from it meets (`AdventureState::tile_index`: an object added in play, else the map's last
/// still on it); none for a tile that is no entrance (or whose object is a blocker).
pub(super) fn at<'a>(state: &'a GameState, rules: &'a GameRuleset, pos: Pos) -> Option<At<'a>> {
    let adventure = state.adventure();
    if let Some(hero) = adventure.heroes().iter().find(|h| h.pos == pos) {
        return Some(At::Hero(hero));
    }
    if adventure.is_empty_boat(pos) {
        return Some(At::Boat);
    }
    let typed = adventure.tile_index(pos).and_then(|index| {
        adventure
            .object(index)
            .and_then(|object| object.object_type)
            .map(|ty| (index, ty))
    });
    typed.and_then(|(index, ty)| {
        rules.adventure().object_type(ty).map(|t| At::Object {
            index,
            ty,
            kind: &t.kind,
            class: Class::of(&t.kind),
        })
    })
}

/// What the object value asks of its hero (`ai_object_value`'s ecx and the reads it makes of it):
/// the hero's and its owner's reads ([`Asked`]) and its kept destination (+0x35 to +0x3D, the
/// planning's: T2i), which the Magic Well and Spring read.
#[derive(Clone, Copy)]
pub(super) struct Asking<'v, 'a> {
    /// The hero's and its owner's reads.
    pub(super) asked: Asked<'v, 'a>,
    /// The hero's kept destination, none.
    pub(super) destination: Option<Pos>,
}

/// The value to the asked hero of the object whose entrance is `pos` (`ai_object_value`
/// 0x528040; CPU-OBJ-001), `cost` the candidate's cost, which the movement objects lower or
/// replace: 0 for a tile that is no entrance; else by its class ([`At`], [`Class::of`]), every
/// class the switch does not list 0.
// rule: CPU-OBJ-001
pub(super) fn value(asking: Asking<'_, '_>, pos: Pos, cost: &mut i32) -> i32 {
    let asked = asking.asked;
    match at(asked.state(), asked.hero.rules, pos) {
        None | Some(At::Boat) => 0,
        Some(At::Hero(other)) => fights::hero(asked, other, (pos, *cost)),
        Some(At::Object {
            index,
            ty,
            kind,
            class,
        }) => match class {
            Class::Artifact => fights::artifact(asked, index, pos),
            Class::Pandora => fights::pandora(asked, index, pos),
            Class::Bank => fights::bank(asked, (index, kind), pos),
            Class::Dwelling => fights::dwelling(asked, index, (pos, *cost)),
            Class::Garrison => fights::garrison(asked, index, pos),
            Class::HillFort => fights::hill_fort(asked, kind, *cost),
            Class::Mine => fights::mine(asked, index, pos),
            Class::Monster => fights::monster(asked, index, pos),
            Class::Prison => fights::prison(asked, index),
            Class::Pyramid => fights::pyramid(asked, (index, kind), pos),
            Class::RefugeeCamp => fights::refugee_camp(asked, index),
            Class::Resource => fights::resource(asked, index, pos),
            Class::Scroll => fights::scroll(asked, index, pos),
            Class::Town => fights::town(asked, index, (pos, *cost)),
            class => visits::value(asking, (index, ty, kind, class), pos, cost),
        },
    }
}

/// The computer's answer to a guarded object's question (`ai_fight_answer` 0x52BD10; CPU-OBJ-001):
/// the object value of `pos` with a scratch cost of 0, as it is. Its callers are the answers
/// (increment 4).
// rule: CPU-OBJ-001
pub(super) fn fight_answer(asking: Asking<'_, '_>, pos: Pos) -> i32 {
    let mut cost = 0;
    value(asking, pos, &mut cost)
}

/// `n` as `fild` then a store to `f32` loads it.
fn single(n: i64) -> SignedX87 {
    SignedX87::int(n).round(SINGLE_BITS)
}

/// The hero's experience weight (+0x109, an f32: the turn start's, `start::HeroValues`).
fn weight(asked: Asked<'_, '_>) -> SignedX87 {
    SignedX87::from_f32_bits(asked.values().experience)
}

/// The player's resource weight of `resource` (+0x128, f64: the turn start's), 0 for one it
/// does not weigh.
fn resource_weight(asked: Asked<'_, '_>, resource: ResourceId) -> SignedX87 {
    asked
        .hero
        .weights
        .get(resource.index())
        .copied()
        .unwrap_or(SignedX87::ZERO)
}

/// The player's gold weight (+0x158: the market's cost resource, the base pack's gold), 0
/// without the market.
fn gold_weight(asked: Asked<'_, '_>) -> SignedX87 {
    let gold = asked
        .hero
        .rules
        .adventure()
        .params()
        .market()
        .map(|m| m.cost());
    gold.map_or(SignedX87::ZERO, |g| resource_weight(asked, g))
}

/// The player's mean artifact value (+0x164, an f32: the turn start's, CPU-STA-008).
fn mean(asked: Asked<'_, '_>) -> SignedX87 {
    SignedX87::from_f32_bits(asked.record.artifacts)
}

/// The value of `amounts` (by resource) to the player (`ai_resources_value` 0x526C70;
/// CPU-OBJ-002): over the resources in order, the running total plus the amount times its
/// weight, in double precision, truncated at each step.
// rule: CPU-OBJ-002
fn resources_value(asked: Asked<'_, '_>, amounts: &[(ResourceId, i32)]) -> i32 {
    let count = asked.hero.rules.core().resources().len();
    (0..=u16::MAX)
        .take(count)
        .map(ResourceId::from_raw)
        .fold(0_i32, |total, resource| {
            let amount = amounts
                .iter()
                .find(|&&(r, _)| r == resource)
                .map_or(0, |&(_, a)| a);
            SignedX87::int(amount.into())
                .times(resource_weight(asked, resource))
                .plus(SignedX87::int(total.into()))
                .ftol()
        })
}

/// One level's experience to the hero (CPU-OBJ-002): the experience to its next level as an f32
/// times its weight, truncated (`experience_to_next_level` 0x4DA420, `h3_heroes::
/// next_level_experience`).
// rule: CPU-OBJ-002
fn level(asked: Asked<'_, '_>) -> i32 {
    single(next_level(asked).into()).times(weight(asked)).ftol()
}

/// The experience to the hero's next level, as the original's `i32`.
fn next_level(asked: Asked<'_, '_>) -> i32 {
    let fighter = asked.fighter();
    let next = h3_heroes::next_level_experience(fighter.details, asked.hero.rules.core());
    i32::from_ne_bytes(next.to_ne_bytes())
}

/// The hero's army's strength (CPU-OBJ-002): its value times 40 plus its primary sum
/// (`hero_primary_sum` 0x4E5960), in 32 bits, over 40: each primary point 2.5 % of the army.
// rule: CPU-OBJ-002
fn strength(asked: Asked<'_, '_>) -> i32 {
    let fighter = asked.fighter();
    let sum = primary_sum_of(fighter.details, asked.hero.rules.core());
    let army = super::fight::value_of(asked.hero.rules, &fighter.on_map.army);
    army.wrapping_mul(40 + i32::from(sum)) / 40
}

/// Whether the hero's army is undead (0x44AB20): every stack's creature undead, an empty army
/// too.
fn undead(asked: Asked<'_, '_>) -> bool {
    let core = asked.hero.rules.core();
    asked.fighter().on_map.army.iter().all(|stack| {
        core.creature(stack.creature)
            .is_some_and(|c| c.flags.contains(CreatureFlag::Undead))
    })
}

/// The hero's own luck as the values ask it (`hero_luck` 0x4E36C0 with its clamp: −3 to 3).
fn luck(asked: Asked<'_, '_>) -> i32 {
    let (_, luck) =
        artifacts::own_morale_and_luck(asked.state(), asked.hero.rules, asked.fighter());
    luck.clamp(-3, 3)
}

/// The bare value of a morale change by `c` (`ai_morale_change` 0x435830 from the hero's own
/// morale, a fraction).
fn morale_change(asked: Asked<'_, '_>, c: i32) -> SignedX87 {
    let m = i32::from(asked.morale());
    h3_battle::estimate::change_value(m, c, h3_battle::estimate::morale_values(), SLIP)
}

/// The bare value of a luck change by `c` from `l` (`ai_luck_change` 0x435960, a fraction).
fn luck_change(l: i32, c: i32) -> SignedX87 {
    h3_battle::estimate::change_value(l, c, h3_battle::estimate::luck_values(), SLIP)
}

/// `x` times the hero's strength, truncated (double precision).
fn of_strength(asked: Asked<'_, '_>, x: SignedX87) -> i32 {
    x.times(SignedX87::int(strength(asked).into())).ftol()
}

/// The value of `c` points of morale to the hero (`ai_morale_gain_value` 0x527CF0;
/// CPU-OBJ-002): 0 for an undead army; else the change from its own morale times its strength.
// rule: CPU-OBJ-002
fn morale_gain(asked: Asked<'_, '_>, c: i32) -> i32 {
    if undead(asked) {
        return 0;
    }
    of_strength(asked, morale_change(asked, c))
}

/// The value of `c` points of luck to the hero (`ai_luck_gain_value` 0x527D80; CPU-OBJ-002): when
/// its luck plus `c` passes 3, its luck is taken as 3 less its luck (survey §5.5 #4, a slip
/// ported as read: the change is not capped); the change times its strength. The original's 0
/// for a `c` not above 0 there is left out: with the luck held to 3 it cannot pass 3.
// rule: CPU-OBJ-002
fn luck_gain(asked: Asked<'_, '_>, c: i32) -> i32 {
    let mut l = luck(asked);
    if l + c > 3 {
        l = 3 - l;
    }
    of_strength(asked, luck_change(l, c))
}

/// The hero's movement points left (+0x4D) as the original's `i32`.
fn movement(asked: Asked<'_, '_>) -> i32 {
    i32::from_ne_bytes(asked.fighter().on_map.movement.to_ne_bytes())
}

/// Whether the hero carries a full backpack (`hero_backpack_count` 0x4D90C0 at 64 or more).
fn backpack_full(asked: Asked<'_, '_>) -> bool {
    asked.fighter().details.backpack.len() >= BACKPACK
}

/// The low 16 bits of `value` as the original's word holds it.
fn low16(value: i32) -> i16 {
    let [low, high, _, _] = value.to_le_bytes();
    i16::from_le_bytes([low, high])
}

/// The hero's Wisdom level (+0xD0: the hero parameters' Wisdom, the original's skill 7), 0
/// without the parameters.
fn wisdom(asked: Asked<'_, '_>) -> u8 {
    let params = asked.hero.rules.core().hero_params();
    params.map_or(0, |p| asked.hero.skill(p.wisdom))
}

/// The value to the hero of a spell offered to it (`ai_learnable_spell_value` 0x5298D0;
/// CPU-OBJ-009): 0 when its level is above the hero's Wisdom level ([`wisdom`]) plus 2, when it knows it or
/// when it wears no spell book; else its gain (`spells::gain`, CPU-SPL-007).
// rule: CPU-OBJ-009
fn learnable_spell(asked: Asked<'_, '_>, spell: h3_content::SpellId) -> i32 {
    let rules = asked.hero.rules;
    let core = rules.core();
    let level = core.spell(spell).map_or(u8::MAX, |s| s.level);
    if u16::from(level) > u16::from(wisdom(asked)) + 2
        || asked.hero.learned(spell)
        || !h3_heroes::wears_spell_book(asked.fighter().details, core)
    {
        return 0;
    }
    super::spells::gain(rules, &asked.hero.caster, spell)
}
