//! The hero's search and what it reaches (E31 T2i; survey §1.9, §1.11): its mode-4 jumps as
//! `pathfinder_run` 0x56B440 sets them (CPU-PLN-007), the search itself with the danger map and
//! the fight answers (`ai_hero_reach_claims` 0x42F570), its colleagues' claims (CPU-PLN-002) and
//! the candidates (`ai_hero_candidates` 0x42EDD0, CPU-PLN-003) with the claimed classes
//! (`ai_class_tables_init` 0x428070).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureSpell, ComputerSearch, NoAsks, ObjectKind, Passage, Pos, SearchAsks, SearchFog,
    SearchJumps, SearchMode, SearchStart,
};
use h3_content::SpellId;
use h3_heroes::SchoolRaise;

use super::super::fight::{self, Fighter};
use super::super::memory::Plans;
use super::super::objects::{self, Asking, At, Class};
use super::{Candidate, Planner, entrance};
use crate::{GameRuleset, GameState};

/// The eight steps clockwise from up (`step_directions` 0x678150: (dx, dy) and two bytes each).
pub(super) const STEPS: [(i8, i8); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// What the mana must hold beyond a jump's cost (0x56B539, 0x56B68E, 0x56B6E9: 20).
const MANA_MARGIN: i32 = 20;

/// A Dimension Door leg's price (300) and at Expert (200).
const DOOR_PRICES: (u32, u32) = (300, 200);

/// The levels a jump's spell is cast at: Expert (3), the level of Angel Wings and the Boots of
/// Levitation (0x56B4B8), and from which Summon Boat makes a boat (Advanced, 2: 0x4E5686).
const EXPERT: u8 = 3;
const ADVANCED: u8 = 2;

/// The value of an unexplored tile while the player owns a town, and under the turn's flag
/// (0x42F16A..0x42F176: 100, 100 000).
const UNEXPLORED: (i32, i32) = (100, 100_000);

/// The classes `ai_claimed_classes` 0x693718 marks: `ai_class_tables_init` 0x428070 writes 1 at
/// the indices 0, 1, 2 … one per entry of `ai_claimed_class_list` 0x660540 (37 entries, its zero
/// at 0x6605D4), so the classes 0 to 36, not the 37 listed (survey §5.5 #1, a slip ported as
/// read).
const CLAIMED: u8 = 37;

/// Whether the claims apply to class `class` ([`CLAIMED`]; CPU-PLN-002).
pub(super) const fn claimed(class: u8) -> bool {
    class < CLAIMED
}

/// A class past the claimed ones: the original's class of an object the values count as
/// [`Class::Other`] and of no class below 37.
const UNCLAIMED: u8 = u8::MAX;

/// The original's class (map item +0x1E) of what stands at a tile, as far as the claims tell it
/// apart (the agent's mapping, CPU-PLN-002; change freely): none 0 (no object: 0, a coast's 3, an
/// event's 26, all below 37); a hero 34, a boat 8; an object [`Class::number`] of its
/// class, but where one class of the values stands for classes both sides of 37: a bank by its
/// battle and search (the Shipwreck 85 on a deck at the coast, the Derelict Ship 24 on a deck, the
/// Crypt 84 searched, else a creature bank 16 or the Dragon Utopia 25), one level's experience by
/// its skill (the Mercenary Camp's Attack 51, else the Marletto Tower's Defense 23), one luck by
/// its ground (the Mermaids on water 52, else the Faerie Ring 28), one morale and points by them
/// (the Oasis's 800 points 56, else the Fountain of Youth 31, the Watering Hole's 110 read as it),
/// a border guard 9; any other of [`Class::Other`] past 36.
pub(super) fn class_number(state: &GameState, rules: &GameRuleset, pos: Pos) -> u8 {
    let Some(at) = objects::at(state, rules, pos) else {
        return 0;
    };
    let water = || water(state, rules, pos);
    match at {
        At::Hero(_) => Class::Hero.number(),
        At::Boat => Class::Boat.number(),
        At::Object { kind, class, .. } => match (class, kind) {
            (Class::Bank, ObjectKind::Bank(bank)) => match (bank.deck, bank.coast) {
                (true, true) => 85,
                (true, false) => 24,
                _ if bank.looted.is_some() => 84,
                _ => Class::Bank.number(),
            },
            (Class::OneLevel, ObjectKind::Gift(gift)) => match gift.gives {
                h3_adventure::Gives::Primary([attack, ..]) if attack > 0 => 51,
                _ => Class::OneLevel.number(),
            },
            (Class::FaerieRing, _) if water() => 52,
            (Class::Oasis, ObjectKind::Bonus { points, .. }) if *points >= 800 => 56,
            (Class::Other, ObjectKind::KeyGuard { .. }) => 9,
            (Class::Other, _) => UNCLAIMED,
            (class, _) => class.number(),
        },
    }
}

/// Whether `pos`'s terrain is water (map item +4 = 8).
pub(super) fn water(state: &GameState, rules: &GameRuleset, pos: Pos) -> bool {
    let tile = state.adventure().map().tile(pos);
    let passage = tile.and_then(|tile| rules.adventure().passage(tile.terrain));
    matches!(passage, Some(Passage::Water(_)))
}

/// The spell of the adventure effect `effect` in the core (the original's spells 0, 6, 7 and 8
/// by the effect the adventure part gives them).
fn spell_of(rules: &GameRuleset, effect: AdventureSpell) -> Option<SpellId> {
    (0..rules.core().spells().len())
        .filter_map(SpellId::from_index)
        .find(|&spell| rules.adventure().adventure_spell(spell) == Some(effect))
}

/// The raise of `pos`'s special ground (`hero_tile_spell_field` 0x4E4FA0): its battlefield's
/// schools at Expert.
fn raise_at(state: &GameState, rules: &GameRuleset, pos: Pos) -> SchoolRaise {
    let field = fight::ground(state, rules, pos).battlefield;
    field
        .and_then(|id| rules.battle().battlefield(id))
        .map_or(SchoolRaise::None, |field| {
            SchoolRaise::Schools(field.expert_schools.iter().copied().collect())
        })
}

/// Mode 4's jumps (`pathfinder_run` 0x56B440 at 0x56B4FC..0x56B731; survey §1.11 and §5.5 #79;
/// CPU-PLN-007) on the hero's spells and masteries (ADR-0013 point 2: `h3_heroes`' queries, on
/// its tile's ground): a jump's spell castable when the hero has it (+0x430, the available
/// spells: `h3_heroes::available_spells`) and its cost there plus 20 is within its mana (a
/// signed word). Dimension Door so, its leg 300 points, 200 at Expert. Summon Boat when it
/// would bring a boat ([`summons`]) and no boat's entrance is beside the hero. Fly by Angel Wings
/// (its level 3) or so (its level its mastery there, castable or not); Water Walk by the Boots of
/// Levitation or so, alike; each level lowered to the hero's own of the day's spell when that is
/// set and lower.
// rule: CPU-PLN-007
pub(super) fn jumps(state: &GameState, rules: &GameRuleset, hero: Fighter<'_>) -> SearchJumps {
    let core = rules.core();
    let details = hero.details;
    let raise = raise_at(state, rules, hero.on_map.pos);
    let available = h3_heroes::available_spells(details, core).unwrap_or_default();
    let masteries = h3_heroes::spell_masteries_on(details, core, raise).unwrap_or_default();
    let mana = i32::from(i16::from_ne_bytes(details.mana.to_ne_bytes()));
    let mastery = |spell: Option<SpellId>| {
        masteries
            .iter()
            .find(|&&(s, _)| Some(s) == spell)
            .map_or(0, |&(_, level)| level)
    };
    let castable = |spell: Option<SpellId>| {
        spell.filter(|s| available.contains(s)).is_some_and(|s| {
            h3_heroes::map_spell_cost(details, core, s, raise)
                .is_ok_and(|cost| i32::from(cost) + MANA_MARGIN <= mana)
        })
    };
    let [summon, fly, walk, door] = [
        AdventureSpell::SummonBoat,
        AdventureSpell::Fly,
        AdventureSpell::WaterWalk,
        AdventureSpell::DimensionDoor,
    ]
    .map(|effect| spell_of(rules, effect));
    let worn = h3_heroes::movement(details, core);
    let own = state.adventure().day_spells(hero.on_map.id);
    let level = |wears: bool, spell, own: Option<u8>| {
        let level = if wears { EXPERT } else { mastery(spell) };
        own.filter(|&own| own < level).unwrap_or(level)
    };
    let beside_boat = STEPS.iter().any(|&(dx, dy)| {
        let pos = hero.on_map.pos;
        pos.x
            .checked_add_signed(dx.into())
            .zip(pos.y.checked_add_signed(dy.into()))
            .is_some_and(|(x, y)| {
                let tile = Pos::new(x, y, pos.level);
                state.adventure().is_empty_boat(tile)
            })
    });
    SearchJumps {
        summon_boat: summons(state, rules, hero, (summon, raise)) && !beside_boat,
        door: castable(door),
        fly: worn.flying.is_some() || castable(fly),
        water_walk: worn.water_walking.is_some() || castable(walk),
        door_price: if mastery(door) >= EXPERT {
            DOOR_PRICES.1
        } else {
            DOOR_PRICES.0
        },
        fly_level: Some(level(worn.flying.is_some(), fly, own.fly)),
        water_walk_level: Some(level(worn.water_walking.is_some(), walk, own.water_walk)),
    }
}

/// Whether the hero's Summon Boat would bring a boat (`hero_summon_boat_possible` 0x4E5550,
/// read at T2i; the agent's mapping onto the engine's boats, change freely): the spell
/// available, its cost on the hero's ground (at least 1) within the mana; then a boat
/// `summon_boat_choice` 0x4E54A0 would pick (an empty boat whose last captain is the hero, or of
/// the player or of nobody, on any level: the original's, ADV-SPL-003's faithful profile), or,
/// with the school's mastery off the ground (0x4E557A) at least Advanced, room for a new boat
/// (`boat_free_record` 0x4BB170: fewer boats in use than the part's most).
fn summons(
    state: &GameState,
    rules: &GameRuleset,
    hero: Fighter<'_>,
    (spell, raise): (Option<SpellId>, SchoolRaise),
) -> bool {
    let core = rules.core();
    let details = hero.details;
    let Some(spell) = spell.filter(|s| {
        h3_heroes::available_spells(details, core).is_ok_and(|spells| spells.contains(s))
    }) else {
        return false;
    };
    let mana = i32::from(i16::from_ne_bytes(details.mana.to_ne_bytes()));
    let cost = h3_heroes::map_spell_cost(details, core, spell, raise).map_or(i32::MAX, i32::from);
    if mana < cost {
        return false;
    }
    let adventure = state.adventure();
    let (id, player) = (hero.on_map.id, hero.on_map.owner);
    let mut boats = adventure
        .boats()
        .filter(|(_, boat)| !boat.aboard && !boat.gone);
    if boats.any(|(_, b)| b.captain == Some(id) || b.owner.is_none_or(|o| o == player)) {
        return true;
    }
    let plain = h3_heroes::spell_masteries_on(details, core, SchoolRaise::None)
        .unwrap_or_default()
        .into_iter()
        .find(|&(s, _)| s == spell)
        .map_or(0, |(_, level)| level);
    let in_use = adventure.boats().filter(|(_, boat)| !boat.gone).count();
    let most = rules.adventure().params().spells();
    plain >= ADVANCED && most.is_some_and(|params| in_use < usize::from(params.boats_most.get()))
}

/// The hero's search's asks: the turn's danger map and the fight answers of the object value
/// with the hero's kept destination (`ai_fight_answer` 0x52BD10, `objects::fight_answer`); no
/// boat marks (`AI-STUB-2`: the walk's, T2j).
struct Asks<'p, 'a> {
    planner: &'p Planner<'a>,
    asking: Asking<'p, 'a>,
}

impl SearchAsks for Asks<'_, '_> {
    fn danger(&self, pos: Pos) -> i32 {
        self.planner.danger.at(pos)
    }

    fn fight_answer(&self, pos: Pos) -> i32 {
        objects::fight_answer(self.asking, pos)
    }
}

impl Planner<'_> {
    /// The hero's search (`ai_hero_reach_claims` 0x42F570 at 0x42F6B5; CPU-PLN-002): mode 4 from
    /// its tile with the budget as its bound, its points left as its points, in a boat when it
    /// is, its jumps, the danger map and the fight answers of `asking`; none for a hero not on
    /// the map.
    pub(super) fn search(&self, budget: i32, asking: Asking<'_, '_>) -> Option<ComputerSearch> {
        let (state, rules) = (self.state, self.rules);
        let hero = self.on_map();
        let start = SearchStart {
            player: hero.owner,
            hero: hero.id,
            from: hero.pos,
            boat: hero.boat.is_some(),
            bound: u32::try_from(budget).unwrap_or(0),
            points: hero.movement,
            mode: SearchMode::Plan,
            fog: SearchFog::Computer,
            window: None,
            jumps: self.jumps,
            open: false,
        };
        let asks = Asks {
            planner: self,
            asking,
        };
        let adventure = state.adventure();
        adventure.computer_search(
            rules.core(),
            rules.adventure(),
            state.heroes(),
            &start,
            &asks,
        )
    }

    /// The colleagues' claims (`ai_hero_reach_claims` 0x42F570 from 0x42F6BA; CPU-PLN-002): a
    /// word per tile of every level, 0xFFFF, lowered by each other hero of the player whose
    /// tile's place `search` reached: its search in mode 2 from its kept destination (else its
    /// tile, the offset 0), its day's maximum (+0x49) as its bound and points, in a boat when the
    /// start tile is water, no danger (its own pathfinder's buffer none, 0x4B13AE); each place it
    /// reached (inside its patrol, `AI-STUB-5`: every tile) lowers its tile's word to its cost
    /// plus the offset (16 bits, kept when unsigned less). The offset: the kept cost beyond less
    /// its points left (16 bits) when that cost is not above them (a signed 32-bit compare),
    /// else 0 (survey §5.5 #2, a slip ported as read: a node nearer than that difference wraps
    /// and claims nothing).
    fn claims(&self, search: &ComputerSearch, plans: &Plans) -> Vec<u16> {
        let (state, rules) = (self.state, self.rules);
        let adventure = state.adventure();
        let hero = self.on_map();
        let mut claims = vec![u16::MAX; self.tiles()];
        let colleagues = adventure
            .heroes()
            .iter()
            .filter(|other| other.owner == hero.owner && other.id != hero.id)
            .filter(|other| search.node(other.pos).is_some());
        for other in colleagues {
            let points = i32::from_ne_bytes(other.movement.to_ne_bytes());
            let (from, offset) = plans.get(other.id).map_or((other.pos, 0), |plan| {
                let [low, high, ..] = points.to_le_bytes();
                let offset = if i32::from(plan.beyond) <= points {
                    plan.beyond.wrapping_sub(u16::from_le_bytes([low, high]))
                } else {
                    0
                };
                (plan.destination, offset)
            });
            let start = SearchStart {
                player: hero.owner,
                hero: other.id,
                from,
                boat: water(state, rules, from),
                bound: other.max_movement,
                points: other.max_movement,
                mode: SearchMode::Claims,
                fog: SearchFog::Computer,
                window: None,
                jumps: SearchJumps::default(),
                open: false,
            };
            let reach = adventure.computer_search(
                rules.core(),
                rules.adventure(),
                state.heroes(),
                &start,
                &NoAsks,
            );
            // stub: AI-STUB-5
            for node in reach.iter().flat_map(ComputerSearch::reached) {
                let claim = node.cost.wrapping_add(offset);
                let slot = &mut claims[self.index(node.pos)];
                *slot = (*slot).min(claim);
            }
        }
        claims
    }

    /// The hero's candidates (`ai_hero_candidates` 0x42EDD0 but its Grail step; survey §1.9;
    /// CPU-PLN-003) from its search `search`, with its colleagues' claims ([`Self::claims`]) and
    /// the object value's question `asking`; with the danger of its tile, which the original's
    /// search returns (`ai_hero_reach_claims` 0x42F5EE..0x42F63C). On a town's entrance that two
    /// or more enemy heroes threaten (the record's count, town +0x3) none; one: only the enemy
    /// heroes it reaches with its points left, each a must-go. Else each place of the reached
    /// list from its end (the mode flag is set from the search and walk: its other test is the
    /// walk's, T2j): an entrance (explored or not), or an unexplored tile while the player owns
    /// a town; inside the patrol (`AI-STUB-5`: every tile); not the hero's own tile; of a claimed
    /// class ([`claimed`] of [`class_number`]) only when no colleague claims it at less cost.
    /// Its cost the node's second cost (the third argument 0 from 0x42E0B0, 0x42E11F); its value
    /// for an unexplored tile while the player owns a town 100, or 100 000 under the turn's flag,
    /// else the object value (`ai_object_value` 0x528040, `objects::value`, which may change the
    /// cost); kept above 0, or at 0 while the hero's tile is in danger. The Grail step
    /// (0x42F35A..0x42F54E, the player's puzzle target +0x39) is not ported: the engine has no
    /// Grail on the map nor obelisks (`AI-STUB-15`).
    // rule: CPU-PLN-003
    pub(super) fn candidates(
        &self,
        search: &ComputerSearch,
        plans: &Plans,
        asking: Asking<'_, '_>,
    ) -> (i32, Vec<Candidate>) {
        let (state, rules) = (self.state, self.rules);
        let hero = self.on_map();
        let danger = self.danger.at(hero.pos);
        let claims = self.claims(search, plans);
        let adventure = state.adventure();
        let threats = adventure.map().town_at(hero.pos).map_or(0, |town| {
            let towns = &self.record.towns;
            towns
                .binary_search_by_key(&town, |&(t, _)| t)
                .map_or(0, |at| towns[at].1)
        });
        if threats >= 2 {
            return (danger, Vec::new());
        }
        let owns_town = self.owns_town();
        let points = self.points();
        let mut list = Vec::new();
        for node in search.reached().rev() {
            let pos = node.pos;
            let at = objects::at(state, rules, pos);
            let explored = self.explored(pos);
            if !entrance(at.as_ref()) && (explored || !owns_town) {
                continue;
            }
            // stub: AI-STUB-5
            let mut must_go = false;
            if threats == 1 {
                let enemy = matches!(at, Some(At::Hero(other)) if !adventure.allied(other.owner, hero.owner));
                if !enemy || i32::from(node.cost) > points {
                    continue;
                }
                must_go = true;
            }
            if pos == hero.pos
                || (claimed(class_number(state, rules, pos)) && node.cost > claims[self.index(pos)])
            {
                continue;
            }
            let mut cost = i32::from(node.cost2);
            let value = if !explored && owns_town {
                if self.flag {
                    UNEXPLORED.1
                } else {
                    UNEXPLORED.0
                }
            } else {
                objects::value(asking, pos, &mut cost)
            };
            if value > 0 || (value == 0 && danger < 0) {
                list.push(Candidate {
                    pos: Some(pos),
                    value,
                    cost,
                    near: false,
                    must_go,
                });
            }
        }
        // stub: AI-STUB-15
        (danger, list)
    }
}
