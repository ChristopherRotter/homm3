//! The computer's values of dwellings and towns (E31 T2g; survey
//! `docs/rules/surveys/E31-computer-player.md` §2.3, §2.4): a dwelling's value to a hero
//! (`ai_dwelling_value` 0x529A30, [`dwelling_value`], CPU-TWN-001), the value of taking another's
//! town (`ai_town_capture_value` 0x529CB0, [`capture_value`], CPU-TWN-002), the army part of its
//! own town's (`ai_own_town_army_value` 0x52B090, [`own_army_value`], CPU-TWN-003), a town's
//! visit value (`ai_town_building_value` 0x52B1E0, [`visit_value`], CPU-TWN-004) and a town's
//! value (`ai_town_value` 0x52AB80, [`town_value`], CPU-TWN-005), all in the fight estimates' units
//! on the army plan's values (`super::armies`) and the fight estimate (`super::fight`). Nothing
//! here changes the game. Each is asked for one of the computer's own heroes in its own turn (the
//! object value's, T2h): the original's current player is the hero's owner.
//!
//! Not here: the teams and the victory and loss conditions they read (an ally's dwelling and
//! town, the dwellings to flag, the capture's loss of its town, the Grail to build, the artifact
//! to bring, the town to capture and the strongest hero's share: `AI-STUB-18`, E10); the buildings a hero visits once and its marks
//! (the Wall of Knowledge, the Order of Fire, the Academy of Battle Scholars, the Hall of
//! Valhalla, the Cage of Warlords: `AI-STUB-19`, E09 increment 7). The fight against a hero
//! visiting another's town, whose result the original drops, is not run (it changes nothing and
//! draws nothing).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{ObjectKind, Pos};
use h3_content::{ArtifactTypeId, CreatureTypeId, FactionId, SpellId};
use h3_contracts::{ArmyStack, PlayerId, TownId};
use h3_heroes::ArtifactInstance;
use h3_kernel::arith::{SINGLE_BITS, SignedX87};
use h3_towns::{BuildingRoles, CREATURE_INDEXES, Town, TownRecord, index_creature};

use super::armies::{self, Offer, Plan};
use super::artifacts::{self, Valued};
use super::fight::{self, Fighter, Opponent};
use super::skills;
use super::spells;
use super::start::Record;
use super::turn::{primary_sum_of, towns_of};
use crate::GameState;

#[cfg(test)]
mod tests;

/// A must: the value of a capture without a town of one's own, its share by the map's towns.
const MUST: i32 = 5_000_000;

/// A fight lost by this much or more (0x529B01, 0x529D06: −500 000 000) is the value.
const LOST: i32 = -500_000_000;

/// A capture's lost fight while the player owns no town (0x529D1E: −2 500 000).
const LOST_WITHOUT_TOWN: i32 = -2_500_000;

/// The candidate's cost from which a purchase counts only when it adds a third of the army
/// (0x529B89, 0x52B168: 400).
const FAR: i32 = 400;

/// What a hero's own town's value is at most (0x52B063: 5 000 000).
const MOST: i32 = 5_000_000;

/// The day of the week from which a capture counts a week's growth (0x529DEF: 7).
const WEEK_ROLL: i32 = 7;

/// A capture's income days (0x63AC28: the f64 3.0, and the Silo's 3 at 0x529DC7).
const INCOME_DAYS: i64 = 3;

/// The spell book's price the guild's value asks of the player (0x52B267: 500 gold) and what a
/// book is worth then (0x52B277: 1 000).
const BOOK: (i32, i32) = (500, 1000);

/// The spell levels a hero learns without Wisdom (0x52B28A: 2).
const LEVELS_WITHOUT_WISDOM: u8 = 2;

/// The Grail (the original's artifact 2).
const GRAIL: ArtifactTypeId = ArtifactTypeId::from_raw(2);

/// The Statue of Legion's parts (0x640558: 118 to 122).
const LEGION_PARTS: [u16; 5] = [118, 119, 120, 121, 122];

/// The Wisdom skill (the original's skill 7).
const WISDOM: h3_content::SkillId = h3_content::SkillId::from_raw(7);

/// What the values read of the hero and its owner: the hero's reads (`artifacts::Valued`: the
/// hero, its army's value, its spell gains, its owner's resource weights) and the owner's turn
/// start's record.
#[derive(Clone, Copy)]
pub(super) struct Asked<'v, 'a> {
    /// The hero's reads.
    pub(super) hero: &'v Valued<'a>,
    /// Its owner's turn start's record.
    pub(super) record: &'v Record,
}

impl<'a> Asked<'_, 'a> {
    /// The game.
    fn state(&self) -> &'a GameState {
        self.hero.state
    }

    /// The hero.
    fn fighter(&self) -> Fighter<'a> {
        self.hero.hero
    }

    /// The hero's owner, the player whose turn it is.
    fn player(&self) -> PlayerId {
        self.hero.hero.on_map.owner
    }

    /// The hero's own morale as the values ask it (`hero_morale` 0x4E39B0 with its clamp: −3 to
    /// 3, a word).
    fn morale(&self) -> i16 {
        let (morale, _) =
            artifacts::own_morale_and_luck(self.state(), self.hero.rules, self.fighter());
        i16::try_from(morale.clamp(-3, 3)).unwrap_or(0)
    }

    /// The Alliance's factions when any of the player's heroes, on the map or in a town's
    /// garrison, wears the Angelic Alliance (`player_hero_wears_artifact` 0x4BACB0 with 129), else
    /// none.
    fn join(&self, factions: &'a [FactionId]) -> &'a [FactionId] {
        let (state, player) = (self.state(), self.player());
        let adventure = state.adventure();
        let on_map = adventure
            .heroes()
            .iter()
            .filter(|h| h.owner == player)
            .map(|h| h.id);
        let garrisons =
            towns_of(state, player).filter_map(|town| adventure.town_hero(town).map(|h| h.id));
        let wears = on_map.chain(garrisons).any(|hero| {
            state.hero(hero).is_some_and(|d| {
                d.worn
                    .iter()
                    .flatten()
                    .any(|i| i.artifact == armies::ANGELIC_ALLIANCE)
            })
        });
        if wears { factions } else { &[] }
    }

    /// The player's resources (+0x9C) by resource, as the original's `i32`.
    fn treasury(&self) -> Vec<i32> {
        let core = self.hero.rules.core();
        let treasury = self.state().treasury(self.player()).unwrap_or_default();
        (0..core.resources().len())
            .map(|r| i32::from_ne_bytes(treasury.get(r).copied().unwrap_or(0).to_ne_bytes()))
            .collect()
    }

    /// The player's stock of the towns' gold.
    fn gold(&self) -> i32 {
        let gold = self.hero.rules.towns().params().map(|p| p.gold);
        gold.and_then(|g| self.treasury().get(g.index()).copied())
            .unwrap_or(0)
    }

    /// The hero's values per unit at the turn start (its experience weight and spell gains), the
    /// default for a hero the record does not list.
    fn values(&self) -> super::start::HeroValues {
        let id = self.fighter().on_map.id;
        self.record
            .heroes
            .binary_search_by_key(&id, |&(h, _)| h)
            .map_or(super::start::HeroValues::default(), |at| {
                self.record.heroes[at].1
            })
    }
}

/// Whether the hero `details` carries `artifact` (`hero_has_artifact` 0x4D91B0): worn or in its
/// backpack.
fn carries(details: &h3_heroes::HeroDetails, artifact: ArtifactTypeId) -> bool {
    details
        .worn
        .iter()
        .flatten()
        .chain(&details.backpack)
        .any(|i| i.artifact == artifact)
}

/// The value to the asked hero of the dwelling at object index `index` (`ai_dwelling_value`
/// 0x529A30; CPU-TWN-001), `cost` the candidate's cost and `tile` the tile it is fought on (an
/// ally's 0 with E10's teams, `AI-STUB-18`). Another's (or none's) with guards: the fight estimate against them
/// ([`fight::fight`]), returned when −500 000 000 or less. Plus the purchase: a plan of its
/// creatures with their counts (the first level's free and whole), [`Plan::purchase_value`] on
/// the hero's army with no spare, the player's resources, the Alliance's flag and the hero's own
/// morale; on its own dwelling with a cost of 400 or more it counts only when the AI value the
/// purchase put in is a third of the army's value or more. The victory to flag all dwellings
/// waits for the conditions (`AI-STUB-18`).
// rule: CPU-TWN-001
pub(super) fn dwelling_value(asked: Asked<'_, '_>, index: usize, (cost, tile): (i32, Pos)) -> i32 {
    let (state, rules) = (asked.state(), asked.hero.rules);
    let adventure = state.adventure();
    let player = asked.player();
    let own = adventure.owner(index) == Some(player);
    // stub: AI-STUB-18
    let mut value = 0;
    let guards = adventure.dwelling_guards(index);
    if !own && !guards.is_empty() {
        let opponent = Opponent {
            hero: None,
            army: &guards,
            town: None,
        };
        value = fight::fight(state, rules, asked.fighter(), opponent, Some(tile));
        if value <= LOST {
            return value;
        }
    }
    let kind = adventure
        .map()
        .objects()
        .get(index)
        .and_then(|o| o.object_type)
        .and_then(|t| rules.adventure().object_type(t))
        .map(|t| &t.kind);
    let creatures = match kind {
        Some(ObjectKind::Dwelling { creatures, .. }) => &creatures[..],
        _ => &[],
    };
    let counts = adventure.dwelling_counts(index).unwrap_or_default();
    let offers = creatures
        .iter()
        .zip(counts)
        .map(|(&creature, &count)| Offer {
            creature,
            count: short(count),
            free: rules
                .core()
                .creature(creature)
                .is_some_and(|c| c.level == 1),
        })
        .collect();
    let factions = armies::alliance_factions(rules);
    let mut plan = Plan::new(rules, asked.hero.weights, offers);
    let army = &asked.fighter().on_map.army;
    let purchase = plan.purchase_value(
        (army, None),
        asked.treasury(),
        (asked.join(&factions), asked.morale()),
    );
    // stub: AI-STUB-18
    if cost >= FAR && own && plan.gain < armies::Army::of(army).value(rules) / 3 {
        return value;
    }
    value.wrapping_add(purchase)
}

/// The low 16 bits of a count as the original's word holds it.
fn short(count: u32) -> i16 {
    let [low, high, _, _] = count.to_le_bytes();
    i16::from_le_bytes([low, high])
}

/// The low 16 bits of a signed value as the original's word holds it.
fn low16(value: i32) -> i16 {
    short(u32::from_ne_bytes(value.to_ne_bytes()))
}

/// `town`'s gold a day as the capture reads it (`town_gold_income` 0x5BF600 without the silo's
/// gold: its hall's, plus the Grail's with a building of role `grail` ever built; TWN-INC-001),
/// its low 16 bits.
fn town_gold(record: Option<&TownRecord>, town: &Town, rules: &crate::GameRuleset) -> i32 {
    let part = rules.towns();
    let params = part.params();
    let hall = h3_towns::hall_level(town, part);
    let hall_gold = params
        .and_then(|p| {
            p.hall_gold
                .get(usize::from(hall).saturating_sub(1))
                .copied()
        })
        .unwrap_or(0);
    let grail = ever_built(record, town, |r| r.grail);
    let grail_gold = params.map_or(0, |p| if grail { p.grail_gold } else { 0 });
    i32::from(short(hall_gold.wrapping_add(grail_gold)))
}

/// Whether `town` has ever built a building of `record` whose roles `role` names.
fn ever_built(
    record: Option<&TownRecord>,
    town: &Town,
    role: impl Fn(&BuildingRoles) -> bool,
) -> bool {
    record.is_some_and(|r| {
        town.built()
            .iter()
            .filter_map(|id| r.building(id))
            .any(|b| role(&b.roles))
    })
}

/// Whether a building of `record` whose roles `role` names stands in `town`.
fn standing(
    record: Option<&TownRecord>,
    town: &Town,
    role: impl Fn(&BuildingRoles) -> bool,
) -> bool {
    record.is_some_and(|r| {
        town.standing(r)
            .iter()
            .filter_map(|id| r.building(id))
            .any(|b| role(&b.roles))
    })
}

/// The value to the asked hero of taking another's `town` (`ai_town_capture_value` 0x529CB0;
/// CPU-TWN-002), `cost` the candidate's cost (its low 16 bits) and `tile` the tile it is fought
/// on: r, the fight estimate against the town's upper bar with its garrison hero
/// ([`fight::fight`]); r of −500 000 000 or less is the value while the player owns a town, else
/// r counts −2 500 000. V = trunc(gold a day × the gold weight × 3.0) (doubles), plus with a
/// Resource Silo standing 3 × its resources weighted (each step truncated); plus over the town's
/// fourteen indexes its available count, with a week's growth when (cost − points left) /
/// the day's maximum + the day of the week reaches 7, times the creature's AI value less its
/// weighted cost where that is positive; V = trunc((1 + the owner's difficulty weight) × f32(V)),
/// the weight `fight::blend`'s; plus 5 000 000 without a town of the player's, else 5 000 000 over
/// the map's towns. Returns r + V. The loss condition's town waits for the conditions
/// (`AI-STUB-18`).
// rule: CPU-TWN-002
pub(super) fn capture_value(asked: Asked<'_, '_>, town: TownId, (cost, tile): (i32, Pos)) -> i32 {
    let (state, rules) = (asked.state(), asked.hero.rules);
    let adventure = state.adventure();
    let hero = asked.fighter();
    let garrison = adventure.town_hero(town).and_then(|on_map| {
        state
            .hero(on_map.id)
            .map(|details| Fighter { on_map, details })
    });
    let opponent = Opponent {
        hero: garrison,
        army: adventure.upper_bar(town),
        town: Some(town),
    };
    let fought = fight::fight(state, rules, hero, opponent, Some(tile));
    let owns = towns_of(state, asked.player()).next().is_some();
    let fought = match (fought <= LOST, owns) {
        (true, true) => return fought,
        (true, false) => LOST_WITHOUT_TOWN,
        _ => fought,
    };
    let income = state
        .towns()
        .get(town)
        .map_or(0, |t| captured_income(asked, (town, t), cost));
    let index = adventure.map().town_object(town);
    let owner = index.and_then(|i| adventure.owner(i));
    let blend = fight::blend(
        adventure.difficulty(),
        owner.map(|o| state.human(o).unwrap_or(false)),
    );
    let value = blend
        .plus(SignedX87::ONE)
        .times(SignedX87::int(income.into()).round(SINGLE_BITS))
        .ftol();
    let towns = i32::try_from(adventure.map().towns().len()).unwrap_or(i32::MAX);
    let share = if owns {
        MUST.checked_div(towns).unwrap_or(0)
    } else {
        MUST
    };
    // stub: AI-STUB-18
    fought.wrapping_add(value.wrapping_add(share))
}

/// What a capture of `town` (`record_town` its record) gains before its owner's weight
/// ([`capture_value`]'s V; 0x529D25 to 0x529EC9), `cost` the candidate's cost.
fn captured_income(asked: Asked<'_, '_>, (town, record_town): (TownId, &Town), cost: i32) -> i32 {
    let (state, rules) = (asked.state(), asked.hero.rules);
    let (core, part) = (rules.core(), rules.towns());
    let hero = asked.fighter();
    let weights = asked.hero.weights;
    let weight = |r: usize| weights.get(r).copied().unwrap_or(SignedX87::ZERO);
    let record = part.town(record_town.faction());
    let gold = part.params().map_or(0, |p| p.gold.index());
    let mut value = SignedX87::int(town_gold(record, record_town, rules).into())
        .times(weight(gold))
        .times(SignedX87::int(INCOME_DAYS))
        .ftol();
    if standing(record, record_town, |r| r.silo) {
        let silo = record.map_or(&[][..], |r| &r.silo[..]);
        let mut amounts = vec![0_i32; core.resources().len()];
        for &(resource, amount) in silo {
            amounts.get_mut(resource.index()).into_iter().for_each(|a| {
                *a = i32::from_ne_bytes(amount.to_ne_bytes());
            });
        }
        let weighted = armies::weighted_cost(weights, &amounts);
        value = value.wrapping_add(weighted.wrapping_mul(3));
    }
    let on_map = hero.on_map;
    let days = i32::from(low16(cost))
        .wrapping_sub(i32::from_ne_bytes(on_map.movement.to_ne_bytes()))
        .checked_div(i32::from_ne_bytes(on_map.max_movement.to_ne_bytes()))
        .unwrap_or(0);
    let weekday = i32::from(state.weekday(rules).get());
    let grows = days.wrapping_add(weekday) >= WEEK_ROLL;
    let inputs = if grows {
        state.growth_inputs(rules)
    } else {
        Vec::new()
    };
    let inputs = inputs.get(town.index()).copied().unwrap_or_default();
    for index in 0..CREATURE_INDEXES {
        let mut count = i32::from(short(record_town.available(index).into()));
        if grows {
            let growth = h3_towns::growth(record_town, part, core, index, &inputs);
            count = count.wrapping_add(short(growth.into()).into());
        }
        let creature = record.and_then(|r| index_creature(r, index));
        if count > 0 {
            creature.into_iter().for_each(|creature: CreatureTypeId| {
                let costs = armies::creature_costs(rules, creature);
                let ai = core
                    .creature(creature)
                    .map_or(0, |c| i32::from_ne_bytes(c.ai_value.to_ne_bytes()));
                let net = ai.wrapping_sub(armies::weighted_cost(weights, &costs));
                if net > 0 {
                    value = value.wrapping_add(net.wrapping_mul(count));
                }
            });
        }
    }
    value
}

/// The army part of the asked hero's own `town`'s value (`ai_own_town_army_value` 0x52B090;
/// CPU-TWN-003), `cost` the candidate's cost (its low 16 bits): a plan of the town's available
/// creatures (0x42CE30: each index with a count, none free); A = [`Plan::exchange_value`] of the
/// hero's army and the town's upper bar, its advantage over the garrison hero's primary sum (none:
/// its own); B = [`Plan::purchase_value`] on the hero's army with the upper bar as the spare and the
/// player's resources, the same plan; 0 when the cost is 400 or more and the AI value both put in
/// is below a third of the army's value, else A / 2 + B.
// rule: CPU-TWN-003
pub(super) fn own_army_value(asked: Asked<'_, '_>, town: TownId, cost: i32) -> i32 {
    let (state, rules) = (asked.state(), asked.hero.rules);
    let (core, part) = (rules.core(), rules.towns());
    let adventure = state.adventure();
    let offers = state.towns().get(town).map_or(Vec::new(), |t| {
        let record = part.town(t.faction());
        (0..CREATURE_INDEXES)
            .filter_map(|index| {
                let count = short(t.available(index).into());
                record
                    .and_then(|r| index_creature(r, index))
                    .filter(|_| count > 0)
                    .map(|creature| Offer {
                        creature,
                        count,
                        free: false,
                    })
            })
            .collect()
    });
    let factions = armies::alliance_factions(rules);
    let join = asked.join(&factions);
    let morale = asked.morale();
    let hero = asked.fighter();
    let garrison_hero = adventure.town_hero(town);
    let theirs = garrison_hero
        .and_then(|h| state.hero(h.id))
        .map_or(0, |d| primary_sum_of(d, core));
    let advantage = i32::from(primary_sum_of(hero.details, core)) - i32::from(theirs);
    let advantage = i16::try_from(advantage).unwrap_or(0);
    let garrison: &[ArmyStack] = adventure.upper_bar(town);
    let army = &hero.on_map.army;
    let mut plan = Plan::new(rules, asked.hero.weights, offers);
    let exchange = plan.exchange_value((army, garrison), (join, morale, advantage));
    let purchase = plan.purchase_value((army, Some(garrison)), asked.treasury(), (join, morale));
    if i32::from(low16(cost)) >= FAR && plan.gain < armies::Army::of(army).value(rules) / 3 {
        return 0;
    }
    (exchange / 2).wrapping_add(purchase)
}

/// The asked hero's value of visiting `town` (`ai_town_building_value` 0x52B1E0; CPU-TWN-004),
/// one of its own (an ally's with E10's teams): with a building of role `university` ever built (the Conflux's
/// Magic University), its value (`skills::university_value` with the gold test) of the part's
/// four skills; with a guild ever built, without a spell book 1 000 when the player has 500 gold,
/// with one the gain (`spells::gain`) of each spell the guild shows at the levels up to 2 plus
/// its Wisdom that the hero has not learned. The building a hero visits once of its faction
/// waits for them (`AI-STUB-19`).
// rule: CPU-TWN-004
pub(super) fn visit_value(asked: Asked<'_, '_>, town: TownId) -> i32 {
    let state = asked.state();
    state
        .towns()
        .get(town)
        .map_or(0, |record_town| visit_of(asked, record_town))
}

/// [`visit_value`] of the town record `record_town`.
fn visit_of(asked: Asked<'_, '_>, record_town: &Town) -> i32 {
    let (state, rules) = (asked.state(), asked.hero.rules);
    let (core, part) = (rules.core(), rules.towns());
    let record = part.town(record_town.faction());
    let hero = asked.fighter();
    let mut value = 0_i32;
    if ever_built(record, record_town, |r| r.university) {
        let offered = part.params().map_or(&[][..], |p| &p.university_skills[..]);
        let gains = asked.values().spells;
        let reads = skills::reads(state, rules, hero, &gains, asked.record);
        value = skills::university_value(state, rules, hero, &reads, (offered, true));
    }
    if ever_built(record, record_town, |r| r.guild.is_some()) {
        if !h3_heroes::wears_spell_book(hero.details, core) {
            if asked.gold() >= BOOK.0 {
                value = value.wrapping_add(BOOK.1);
            }
        } else {
            let caster = spells::record(state, rules, hero);
            for spell in guild_spells(asked, record_town) {
                value = value.wrapping_add(spells::gain(rules, &caster, spell));
            }
        }
    }
    // stub: AI-STUB-19
    value
}

/// The spells of `town`'s guild the visit value counts for the asked hero (0x52B283 to
/// 0x52B312; CPU-TWN-004): those its guild shows (`h3_towns::shown`, the original's counts) at
/// each level from 1 up to 2 plus the hero's Wisdom, in level and slot order, that the hero has
/// not learned.
fn guild_spells(asked: Asked<'_, '_>, town: &Town) -> Vec<SpellId> {
    let details = asked.fighter().details;
    let part = asked.hero.rules.towns();
    let wisdom = details
        .skills
        .iter()
        .find(|&&(s, _)| s == WISDOM)
        .map_or(0, |&(_, l)| l);
    (1..=wisdom.saturating_add(LEVELS_WITHOUT_WISDOM))
        .flat_map(|level| h3_towns::shown(town.guild(), town, part, level))
        .filter(|spell| !details.spells.contains(spell))
        .collect()
}

/// The value to the asked hero of the town at `town` (`ai_town_value` 0x52AB80, the object value's
/// class 98; CPU-TWN-005), `cost` the candidate's cost and `tile` its entrance: another's or none's,
/// [`capture_value`] (an ally's [`visit_value`] with E10's teams, `AI-STUB-18`); its own, [`own_army_value`] plus
/// [`visit_value`], plus: with the Grail carried to a town that may build its building and has
/// not, the Grail's value to the player (`artifacts::value_to_player`); while enemy heroes would
/// take it (the start's threat count), 5 000 000 over the map's towns; for each part of the
/// Statue of Legion the hero carries, its value to the garrison hero as it would wear it, counted
/// (`artifacts::value_to_hero`); at most 5 000 000. The victory conditions' musts wait for them
/// (`AI-STUB-18`).
// rule: CPU-TWN-005
pub(super) fn town_value(asked: Asked<'_, '_>, town: TownId, (cost, tile): (i32, Pos)) -> i32 {
    let (state, rules) = (asked.state(), asked.hero.rules);
    let adventure = state.adventure();
    let Some(index) = adventure.map().town_object(town) else {
        return 0;
    };
    let player = asked.player();
    // stub: AI-STUB-18
    if adventure.owner(index) != Some(player) {
        return capture_value(asked, town, (cost, tile));
    }
    let hero = asked.fighter();
    let mut value = own_army_value(asked, town, cost).wrapping_add(visit_value(asked, town));
    let part = rules.towns();
    let grail_free = state.towns().get(town).is_some_and(|t| {
        part.town(t.faction()).is_some_and(|r| {
            r.buildings.iter().enumerate().any(|(id, b)| {
                b.roles.grail
                    && h3_towns::BuildingId::from_index(id)
                        .is_some_and(|id| !t.forbidden().contains(id) && !t.built().contains(id))
            })
        })
    });
    // stub: AI-STUB-18
    if carries(hero.details, GRAIL) && grail_free {
        let heroes = artifacts::players_heroes(
            state,
            rules,
            (asked.record, asked.hero.weights),
            (player, &[]),
        );
        value = value.wrapping_add(artifacts::value_to_player(
            &heroes,
            ArtifactInstance::from(GRAIL),
        ));
    }
    let threatened = asked
        .record
        .towns
        .binary_search_by_key(&town, |&(t, _)| t)
        .is_ok_and(|at| asked.record.towns[at].1 > 0);
    if threatened {
        let towns = i32::try_from(adventure.map().towns().len()).unwrap_or(i32::MAX);
        value = value.wrapping_add(MUST.checked_div(towns).unwrap_or(0));
    }
    let garrison = adventure.town_hero(town).and_then(|on_map| {
        state
            .hero(on_map.id)
            .map(|details| Fighter { on_map, details })
    });
    if let Some(garrison) = garrison {
        let caster = spells::record(state, rules, garrison);
        let reads = Valued::with(
            state,
            rules,
            (asked.record, asked.hero.weights),
            (garrison, caster),
        );
        for raw in LEGION_PARTS {
            let part = ArtifactTypeId::from_raw(raw);
            if carries(hero.details, part) {
                let worth = artifacts::value_to_hero(&reads, ArtifactInstance::from(part), true);
                value = value.wrapping_add(worth);
            }
        }
    }
    value.min(MOST)
}
