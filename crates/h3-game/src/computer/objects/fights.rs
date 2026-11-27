//! The object value's classes that hold or fight creatures (E31 T2h; survey §2.1): the guarded
//! pickups (the artifact, the Spell Scroll, the resource pile), Pandora's Box, the banks and
//! crypts, the monster, the mine, the Prison, the Pyramid, the garrison, a hero, the Hill Fort,
//! the Refugee Camp, and T2g's dwelling and town values.

use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use h3_adventure::{HeroOnMap, Holds, ObjectKind, Pos, allied};
use h3_contracts::ArmyStack;
use h3_heroes::ArtifactInstance;
use h3_kernel::arith::{SINGLE_BITS, SignedX87, X87};

use super::super::armies::{self, Offer, Plan};
use super::super::artifacts;
use super::super::fight::{self, Fighter, Opponent};
use super::super::towns::{self, Asked};
use super::super::turn::{primary_sum_of, towns_of};
use super::{
    LOST, SKILLS, backpack_full, learnable_spell, level, low16, mean, movement, resources_value,
    single, weight,
};

/// An artifact's least value to a player (`ai_artifact_value_to_player` 0x433AA0's floor, which
/// 0x5280F8 applies again: 10).
const LEAST: i32 = 10;

/// A Spell Scroll's base value (0x52A8EA: 10).
const SCROLL_BASE: i32 = 10;

/// The original's Spell Scroll (the artifact 1).
const SCROLL: h3_content::ArtifactTypeId = h3_content::ArtifactTypeId::from_raw(1);

/// An enemy hero's worth beyond its threat when reached today (0x528C80: 10 000).
const HERO_BASE: i32 = 10_000;

/// A hero's lost fight in a town while its player owns no town (0x528CED: −1 250 000).
const LOST_WITHOUT_TOWN: i32 = -1_250_000;

/// The heroes a player holds at most (the Prison's test of the player's +1 against 8).
const HEROES: usize = 8;

/// The gold a prisoner is worth beside its army (0x6405B0: 2 500).
const PRISONER: i32 = 2500;

/// The Wisdom level from which the Pyramid counts its spells' mean gain (0x52A47F: Expert).
const PYRAMID_WISDOM: u8 = 3;

/// The level of the spells the Pyramid's mean counts (0x52A490: 5).
const PYRAMID_LEVEL: u8 = 5;

/// The cost beyond which the Hill Fort counts only when its upgrades are worth a third of the
/// army (0x528EFD: 500).
const FAR: i32 = 500;

/// The army slots (`army_fill_split` 0x44AAA0's seven).
const SLOTS: u32 = 7;

/// An army's slots (`army_can_take` 0x44AC50's seven).
const ARMY_SLOTS: usize = 7;

/// The game.
fn state<'a>(asked: Asked<'_, 'a>) -> &'a crate::GameState {
    asked.hero.state
}

/// `asked`'s fight estimate against `army` with no hero or town, on `pos`.
fn fight_against(asked: Asked<'_, '_>, army: &[ArmyStack], pos: Pos) -> i32 {
    let opponent = Opponent {
        hero: None,
        army,
        town: None,
    };
    fight::fight(
        state(asked),
        asked.hero.rules,
        asked.fighter(),
        opponent,
        Some(pos),
    )
}

/// `value` plus the fight against the guards of the object at `index` while they stand
/// (`ai_guarded_object_fight` 0x529890; CPU-OBJ-003).
// rule: CPU-OBJ-003
fn guarded(asked: Asked<'_, '_>, index: usize, pos: Pos, value: i32) -> i32 {
    let guards = state(asked).adventure().guards_left(index);
    if guards.is_empty() {
        value
    } else {
        fight_against(asked, guards, pos).wrapping_add(value)
    }
}

/// An artifact's value (class 5; CPU-OBJ-003): 0 with a full backpack; else its value to the
/// player (`artifacts::value_to_player`, at least 10; the placed artifact, a random one's draw, 10
/// for none), plus the fight against its guards while they stand ([`guarded`]). Its pickup
/// conditions 1 to 6 are left out: no map reaches them (`E08-rewards.md` §3.3).
// rule: CPU-OBJ-003
pub(super) fn artifact(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    if backpack_full(asked) {
        return 0;
    }
    let state = state(asked);
    let item = state
        .adventure()
        .object(index)
        .and_then(|object| match &object.holds {
            Holds::Artifact(item) => Some(*item),
            _ => object.resolved.artifact().map(ArtifactInstance::from),
        });
    let heroes = artifacts::players_heroes(
        state,
        asked.hero.rules,
        (asked.record, asked.hero.weights),
        (asked.player(), &[]),
    );
    let value = item
        .map_or(LEAST, |item| artifacts::value_to_player(&heroes, item))
        .max(LEAST);
    guarded(asked, index, pos, value)
}

/// A Spell Scroll's value (class 93, `ai_spell_scroll_value` 0x52A8C0; CPU-OBJ-003): 0 with a
/// full backpack; else 10, plus the fight against its guards while they stand, plus, when the
/// hero does not have its spell available, the scroll's value to it as a candidate
/// (`artifacts::hero_value`, not counted).
// rule: CPU-OBJ-003
pub(super) fn scroll(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    if backpack_full(asked) {
        return 0;
    }
    let value = guarded(asked, index, pos, SCROLL_BASE);
    let spell = state(asked)
        .adventure()
        .object(index)
        .and_then(|object| match &object.holds {
            Holds::Artifact(item) => item.spell,
            _ => None,
        });
    spell
        .filter(|&spell| !asked.hero.has(spell))
        .map_or(value, |spell| {
            let item = ArtifactInstance {
                artifact: SCROLL,
                spell: Some(spell),
            };
            value.wrapping_add(artifacts::hero_value(asked.hero, item, false, false))
        })
}

/// A resource pile's value (class 79, `ai_resource_pile_value` 0x52A7C0; CPU-OBJ-003): its amount
/// (the gold's whole) times its resource's weight plus the fight against its guards while they
/// stand, in double precision, truncated.
// rule: CPU-OBJ-003
pub(super) fn resource(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    let state = state(asked);
    let guards = state.adventure().guards_left(index);
    let fought = if guards.is_empty() {
        0
    } else {
        fight_against(asked, guards, pos)
    };
    let contents = state
        .adventure()
        .object(index)
        .map_or(&[][..], |object| &object.contents[..]);
    contents.first().map_or(0, |&(resource, amount)| {
        SignedX87::int(i32::from_ne_bytes(amount.to_ne_bytes()).into())
            .times(super::resource_weight(asked, resource))
            .plus(SignedX87::int(fought.into()))
            .ftol()
    })
}

/// Pandora's Box's value (class 6; CPU-OBJ-004): the fight against its guards while they stand
/// (else 0); with experience above 0 as a signed value, that times the hero's weight plus the
/// fight, in single-precision operands, truncated; plus its resources' value
/// ([`super::resources_value`]); each primary point one level's experience ([`level`]); each skill
/// the levels it adds, a new one only below eight skills, a level each; its artifacts times the
/// player's mean (single-precision operands, truncated); with a spell book its spells'
/// ([`learnable_spell`]); its creatures' AI values times their counts where the army can take
/// them. Its mana, morale and luck count nothing.
// rule: CPU-OBJ-004
pub(super) fn pandora(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    let state = state(asked);
    let record = state
        .adventure()
        .object(index)
        .and_then(|object| match &object.holds {
            Holds::Record(record) => Some(&**record),
            _ => None,
        });
    record.map_or(0, |record| {
        let guards = state.adventure().guards_left(index);
        let mut v = if guards.is_empty() {
            0
        } else {
            fight_against(asked, guards, pos)
        };
        let experience = i32::from_ne_bytes(record.hero.experience.to_ne_bytes());
        if experience > 0 {
            v = single(experience.into())
                .times(weight(asked))
                .plus(single(v.into()))
                .ftol();
        }
        v = v.wrapping_add(resources_value(asked, &record.resources));
        let level = level(asked);
        for &gain in &record.hero.primary {
            if gain > 0 {
                v = v.wrapping_add(i32::from(gain).wrapping_mul(level));
            }
        }
        let details = asked.fighter().details;
        for &(skill, to) in &record.hero.skills {
            let now = asked.hero.skill(skill);
            let levels = match now {
                0 if details.skills.len() < SKILLS => i32::from(to),
                0 => 0,
                _ if now < to => i32::from(to - now),
                _ => 0,
            };
            v = v.wrapping_add(levels.wrapping_mul(level));
        }
        v = with_artifacts(asked, record.hero.artifacts.len(), v);
        if h3_heroes::wears_spell_book(details, asked.hero.rules.core()) {
            for &spell in &record.hero.spells {
                v = v.wrapping_add(learnable_spell(asked, spell));
            }
        }
        v.wrapping_add(takeable_value(asked, &record.creatures))
    })
}

/// `v` with `n` artifacts at the player's mean: f32(n) times the mean plus f32(v), truncated.
fn with_artifacts(asked: Asked<'_, '_>, n: usize, v: i32) -> i32 {
    single(i64::try_from(n).unwrap_or(i64::MAX))
        .times(mean(asked))
        .plus(single(v.into()))
        .ftol()
}

/// The AI value of `creatures` the hero's army can take (`army_can_take` 0x44AC50: a stack of the
/// creature or a free slot), each times its count.
fn takeable_value(asked: Asked<'_, '_>, creatures: &[ArmyStack]) -> i32 {
    let army = &asked.fighter().on_map.army;
    let core = asked.hero.rules.core();
    creatures
        .iter()
        .filter(|stack| {
            army.len() < ARMY_SLOTS || army.iter().any(|s| s.creature == stack.creature)
        })
        .fold(0_i32, |sum, stack| {
            let ai = core.creature(stack.creature).map_or(0, |c| c.ai_value);
            sum.wrapping_add(word(ai).wrapping_mul(word(stack.count)))
        })
}

/// A 32-bit unsigned value as the original's signed register holds it.
const fn word(value: u32) -> i32 {
    i32::from_ne_bytes(value.to_ne_bytes())
}

/// A bank's or a crypt's value (classes 16, 24, 25, 84, 85; 0x528626, `ai_crypt_value` 0x529920;
/// CPU-OBJ-005): 0 once looted; else the fight against its guards (as they stand), returned when
/// −500 000 000 or less; plus the value of its level's resources ([`super::resources_value`]),
/// its creature's AI value times its count, and its artifacts times the player's mean
/// (single-precision operands, truncated).
// rule: CPU-OBJ-005
pub(super) fn bank(asked: Asked<'_, '_>, (index, kind): (usize, &ObjectKind), pos: Pos) -> i32 {
    let adventure = state(asked).adventure();
    if adventure.is_emptied(index) {
        return 0;
    }
    let fought = fight_against(asked, adventure.guards_left(index), pos);
    if fought <= LOST {
        return fought;
    }
    let drawn = adventure
        .object(index)
        .and_then(|object| object.resolved.bank());
    let (level, artifacts) = drawn.unwrap_or((0, &[]));
    let level = match kind {
        ObjectKind::Bank(bank) => bank.levels.get(usize::from(level)),
        _ => None,
    };
    let resources: Vec<_> = level.map_or(Vec::new(), |l| {
        l.resources.iter().map(|&(r, a)| (r, word(a))).collect()
    });
    let mut v = fought.wrapping_add(resources_value(asked, &resources));
    let core = asked.hero.rules.core();
    if let Some((creature, count)) = level.and_then(|l| l.creature) {
        let ai = core.creature(creature).map_or(0, |c| c.ai_value);
        v = v.wrapping_add(word(ai).wrapping_mul(i32::from(count)));
    }
    with_artifacts(asked, artifacts.len(), v)
}

/// A wandering monster's value (class 54, `ai_monster_value` 0x52A140; CPU-OBJ-006): the fight
/// against its creature with its count's low 12 bits as it stands (the word's), spread over the
/// seven slots (`army_fill_split` 0x44AAA0: slot k the rest over 7 − k). The "defeat a monster"
/// condition's 5 000 000 waits for the conditions (`AI-STUB-18`).
// rule: CPU-OBJ-006
pub(super) fn monster(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    let adventure = state(asked).adventure();
    let creature = adventure
        .object(index)
        .and_then(|object| object.monster.as_ref())
        .map(|m| m.creature);
    let count = adventure.monster_count(index).unwrap_or(0) & 0xFFF;
    let army = creature.map_or(Vec::new(), |creature| fill_split(creature, count));
    // stub: AI-STUB-18
    fight_against(asked, &army, pos)
}

/// `count` of `creature` over the seven slots (`army_fill_split` 0x44AAA0): slot k takes the
/// rest over 7 − k while any is left.
pub(super) fn fill_split(creature: h3_content::CreatureTypeId, count: u32) -> Vec<ArmyStack> {
    let mut left = count;
    (0..SLOTS)
        .filter_map(|k| {
            let part = left / (SLOTS - k);
            left -= part;
            (part > 0).then(|| ArmyStack {
                slot: u8::try_from(k).unwrap_or(0),
                creature,
                count: part,
            })
        })
        .collect()
}

/// A mine's value (class 53, `ai_mine_value` 0x52A010; CPU-OBJ-006): 0 when its owner is of the
/// player's team; else the fight against its guards when it has some, returned when
/// −500 000 000 or less; plus two days of its income at its resource's weight (truncated), times
/// 1 plus the blend of its owner (`fight::blend`), in single-precision operands, truncated. The
/// "flag all mines" condition's share waits for the conditions (`AI-STUB-18`; survey §5.5 #6).
// rule: CPU-OBJ-006
pub(super) fn mine(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    let state = state(asked);
    let adventure = state.adventure();
    let part = asked.hero.rules.adventure();
    let owner = adventure.owner(index);
    if owner.is_some_and(|o| allied(o, asked.player())) {
        return 0;
    }
    let guards = adventure.mine_guards(part, index);
    let mut v = 0;
    if !guards.is_empty() {
        v = fight_against(asked, &guards, pos);
        if v <= LOST {
            return v;
        }
    }
    let income = adventure
        .production(part, index)
        .map_or(0, |(resource, daily)| {
            SignedX87::int(word(daily).into())
                .times(super::resource_weight(asked, resource))
                .times(SignedX87::int(2))
                .ftol()
        });
    let blend = fight::blend(
        adventure.difficulty(),
        owner.map(|o| state.human(o).unwrap_or(false)),
    );
    let gained = blend
        .plus(SignedX87::ONE)
        .times(single(income.into()))
        .ftol();
    // stub: AI-STUB-18
    v.wrapping_add(gained)
}

/// A Prison's value (class 62, `ai_prison_value` 0x52A3A0; CPU-OBJ-006): 0 while the player holds
/// eight heroes; else 2 500 gold at its weight plus the prisoner's army's value, in double
/// precision, truncated.
// rule: CPU-OBJ-006
pub(super) fn prison(asked: Asked<'_, '_>, index: usize) -> i32 {
    let adventure = state(asked).adventure();
    if adventure.hero_list(asked.player()).len() >= HEROES {
        return 0;
    }
    let army = adventure
        .object(index)
        .and_then(|object| object.resolved.prisoner())
        .map_or(&[][..], |(_, army, _)| army);
    super::gold_weight(asked)
        .times(SignedX87::int(PRISONER.into()))
        .plus(SignedX87::int(
            fight::value_of(asked.hero.rules, army).into(),
        ))
        .ftol()
}

/// The Pyramid's value (class 63, `ai_pyramid_value` 0x52A410; CPU-OBJ-006): 0 when the player
/// knows it looted; else the fight against one stack of its first creature and count and one of
/// its second with its stacks' counts summed (the original's 40 Gold and 20 Diamond Golems), plus
/// with Expert Wisdom the mean, over every spell of level 5, of the gains (`spells::gain`) of
/// those the hero neither has available nor has learned, with a spell book (0 for the others).
/// Its own spell is never read.
// rule: CPU-OBJ-006
pub(super) fn pyramid(asked: Asked<'_, '_>, (index, kind): (usize, &ObjectKind), pos: Pos) -> i32 {
    let adventure = state(asked).adventure();
    if adventure.knows(asked.player(), index) && adventure.is_emptied(index) {
        return 0;
    }
    let army: Vec<ArmyStack> = match kind {
        ObjectKind::Pyramid(pyramid) => {
            let (second, counts) = &pyramid.fixed;
            vec![
                ArmyStack {
                    slot: 0,
                    creature: pyramid.split.0,
                    count: pyramid.split.1,
                },
                ArmyStack {
                    slot: 1,
                    creature: *second,
                    count: counts.iter().fold(0_u32, |sum, &c| sum.wrapping_add(c)),
                },
            ]
        }
        _ => Vec::new(),
    };
    let rules = asked.hero.rules;
    let core = rules.core();
    let mut mean = 0;
    if super::wisdom(asked) >= PYRAMID_WISDOM {
        let book = h3_heroes::wears_spell_book(asked.fighter().details, core);
        let (sum, count) = core
            .spells()
            .iter()
            .enumerate()
            .filter(|(_, spell)| spell.level == PYRAMID_LEVEL)
            .fold((0_i32, 0_i32), |(sum, count), (at, _)| {
                let spell =
                    h3_content::SpellId::from_index(at).unwrap_or(h3_content::SpellId::from_raw(0));
                let gain = if !asked.hero.has(spell) && !asked.hero.learned(spell) && book {
                    super::super::spells::gain(rules, &asked.hero.caster, spell)
                } else {
                    0
                };
                (sum.wrapping_add(gain), count + 1)
            });
        mean = sum.checked_div(count).unwrap_or(0);
    }
    fight_against(asked, &army, pos).wrapping_add(mean)
}

/// A garrison's value (class 33; CPU-OBJ-007): the player's own with removable troops, the
/// exchange of the hero's army with it (`armies::Plan::exchange_value` on a cleared plan, the
/// Alliance's flag, the hero's morale, its primary sum the advantage); a team's (its own with
/// troops kept) 0; another's or a neutral one the fight against its army. A campaign's garrison
/// waits for the campaigns (`AI-STUB-21`).
// rule: CPU-OBJ-007
pub(super) fn garrison(asked: Asked<'_, '_>, index: usize, pos: Pos) -> i32 {
    let adventure = state(asked).adventure();
    let player = asked.player();
    let owner = adventure.owner(index);
    let army = adventure.garrison_army(index);
    if owner == Some(player) && adventure.garrison_removable(index).unwrap_or(false) {
        // stub: AI-STUB-21
        let rules = asked.hero.rules;
        let fighter = asked.fighter();
        let factions = armies::alliance_factions(rules);
        let advantage = primary_sum_of(fighter.details, rules.core());
        let mut plan = Plan::new(rules, asked.hero.weights, Vec::new());
        return plan.exchange_value(
            (&fighter.on_map.army, army),
            (asked.join(&factions), asked.morale(), short(advantage)),
        );
    }
    if owner.is_some_and(|o| allied(o, player)) {
        return 0;
    }
    fight_against(asked, army, pos)
}

/// A primary sum as the original's signed word.
fn short(sum: u16) -> i16 {
    i16::from_ne_bytes(sum.to_ne_bytes())
}

/// The value of meeting the hero `other` on `pos` (class 34; CPU-OBJ-007), `cost` the
/// candidate's cost: its own player's ([`own_hero`]); an ally's 0 (`AI-STUB-18`); an enemy's the
/// fight against it and its army (no town), plus, when the cost's low word is within the hero's
/// points left, the blend of its owner times its threat (the turn start's +0x11D) plus 10 000,
/// in single-precision operands, truncated; standing at a town's entrance, plus that town's
/// capture value (`towns::capture_value` with the cost's low word), a hopeless fight
/// (−500 000 000 or less) then −1 250 000 when the player owns no town, else returned alone.
// rule: CPU-OBJ-007
pub(super) fn hero(asked: Asked<'_, '_>, other: &HeroOnMap, (pos, cost): (Pos, i32)) -> i32 {
    let state = state(asked);
    let rules = asked.hero.rules;
    let player = asked.player();
    state.hero(other.id).map_or(0, |details| {
        let other = Fighter {
            on_map: other,
            details,
        };
        if other.on_map.owner == player {
            return own_hero(asked, other, cost);
        }
        // stub: AI-STUB-18
        let adventure = state.adventure();
        let count = adventure.map().towns().len();
        let town = (0..=u16::MAX)
            .take(count)
            .map(h3_contracts::TownId::new)
            .find(|&town| adventure.map().town_entrance(town) == Some(pos));
        let opponent = Opponent {
            hero: Some(other),
            army: &other.on_map.army,
            town: None,
        };
        let mut v = fight::fight(state, rules, asked.fighter(), opponent, Some(pos));
        let cost = i32::from(low16(cost));
        if cost <= movement(asked) {
            let threat = asked
                .record
                .threats
                .binary_search_by_key(&other.on_map.id, |&(h, _)| h)
                .map_or(0, |at| asked.record.threats[at].1);
            let blend = fight::blend(
                adventure.difficulty(),
                Some(state.human(other.on_map.owner).unwrap_or(false)),
            );
            v = blend
                .times(single(threat.wrapping_add(HERO_BASE).into()))
                .plus(single(v.into()))
                .ftol();
        }
        town.map_or(v, |town| {
            if v <= LOST {
                if towns_of(state, player).next().is_some() {
                    return v;
                }
                v = LOST_WITHOUT_TOWN;
            }
            towns::capture_value(asked, town, (cost, pos)).wrapping_add(v)
        })
    })
}

/// The value of meeting the player's own hero `other` (0x528D0C; CPU-OBJ-007): 0 on Easy; when
/// `other`'s primary sum is below the hero's, half the exchange of `other`'s army into the hero's
/// (the Alliance's flag, the hero's morale, the sums' difference the advantage), kept when the
/// cost's low word is below the hero's day's maximum or the plan's gain is at least its army's
/// value, else 0; when above, the whole exchange from `other`'s side (its morale); equal, 0.
// rule: CPU-OBJ-007
fn own_hero(asked: Asked<'_, '_>, other: Fighter<'_>, cost: i32) -> i32 {
    let state = state(asked);
    if state.adventure().difficulty() == 0 {
        return 0;
    }
    let rules = asked.hero.rules;
    let core = rules.core();
    let hero = asked.fighter();
    let (own, theirs) = (
        primary_sum_of(hero.details, core),
        primary_sum_of(other.details, core),
    );
    let factions = armies::alliance_factions(rules);
    let mut plan = Plan::new(rules, asked.hero.weights, Vec::new());
    match theirs.cmp(&own) {
        Ordering::Less => {
            let v = plan.exchange_value(
                (&hero.on_map.army, &other.on_map.army),
                (asked.join(&factions), asked.morale(), short(own - theirs)),
            ) / 2;
            let day = i32::from_ne_bytes(hero.on_map.max_movement.to_ne_bytes());
            let army = fight::value_of(rules, &hero.on_map.army);
            if i32::from(low16(cost)) < day || plan.gain >= army {
                v
            } else {
                0
            }
        }
        Ordering::Greater => {
            let (morale, _) = artifacts::own_morale_and_luck(state, rules, other);
            let morale = i16::try_from(morale.clamp(-3, 3)).unwrap_or(0);
            plan.exchange_value(
                (&other.on_map.army, &hero.on_map.army),
                (asked.join(&factions), morale, short(theirs - own)),
            )
        }
        Ordering::Equal => 0,
    }
}

/// The Hill Fort's value (class 35, 0x528DF0; CPU-OBJ-008), `cost` the candidate's cost: on a copy
/// of the player's resources, for each stack of the hero's army in slot order whose creature has
/// an upgrade (the core's first; RoE's elementals have none in its ruleset), its price
/// (`upgrade_cost` 0x54E750: by resource the upgrade's cost less the creature's, at least 0, times
/// the count; the gold times the fort's percent of the creature's level, single-precision
/// operands, truncated), bought when every resource covers it: the AI value it adds summed, the
/// price paid from the copy. Beyond a cost of 500 it counts only when three times the sum reaches
/// the army's value.
// rule: CPU-OBJ-008
pub(super) fn hill_fort(asked: Asked<'_, '_>, kind: &ObjectKind, cost: i32) -> i32 {
    let rules = asked.hero.rules;
    let core = rules.core();
    let percents = match kind {
        ObjectKind::HillFort(fort) => &fort.gold[..],
        _ => &[],
    };
    let gold = rules.adventure().params().market().map(|m| m.cost());
    let mut resources = asked.treasury();
    let army = &asked.fighter().on_map.army;
    let mut sum = 0_i32;
    for stack in army {
        let from = core.creature(stack.creature);
        let Some(to) = core
            .upgrades(stack.creature)
            .first()
            .and_then(|&u| core.creature(u))
        else {
            continue;
        };
        let base = |r: h3_content::ResourceId| {
            from.and_then(|info| info.cost.iter().find(|&&(c, _)| c == r))
                .map_or(0, |&(_, each)| each)
        };
        let percent = from
            .and_then(|info| usize::from(info.level).checked_sub(1))
            .and_then(|at| percents.get(at))
            .map_or(100, |&p| u32::from(p));
        let factor = X87::ratio(percent, 100, SINGLE_BITS).unwrap_or(X87::ONE);
        let price: Vec<i32> = (0..=u16::MAX)
            .take(resources.len())
            .map(h3_content::ResourceId::from_raw)
            .map(|r| {
                let each = to
                    .cost
                    .iter()
                    .find(|&&(c, _)| c == r)
                    .map_or(0, |&(_, each)| each);
                let more = word(each).wrapping_sub(word(base(r))).max(0);
                let whole = more.wrapping_mul(word(stack.count));
                if Some(r) == gold {
                    single(whole.into()).times(factor.into()).ftol()
                } else {
                    whole
                }
            })
            .collect();
        if price.iter().zip(&resources).any(|(p, have)| p > have) {
            continue;
        }
        let gained = word(to.ai_value).wrapping_sub(from.map_or(0, |f| word(f.ai_value)));
        sum = sum.wrapping_add(gained.wrapping_mul(word(stack.count)));
        for (have, p) in resources.iter_mut().zip(&price) {
            *have = have.wrapping_sub(*p);
        }
    }
    if cost > FAR && sum.wrapping_mul(3) < fight::value_of(rules, army) {
        return 0;
    }
    sum
}

/// The Refugee Camp's value (class 78, `ai_refugee_camp_value` 0x52A700 and its purchase
/// 0x52A710; CPU-OBJ-008): a plan of its week's creature and count, not free (the one creature's
/// offer, `ai_army_plan_init_creature` 0x42CF50: [`Plan::new`] with one [`Offer`]), valued by the
/// purchase (`Plan::purchase_value`) on the hero's army with no spare, the player's resources,
/// the Alliance's flag and the hero's morale.
// rule: CPU-OBJ-008
pub(super) fn refugee_camp(asked: Asked<'_, '_>, index: usize) -> i32 {
    let rules = asked.hero.rules;
    state(asked)
        .adventure()
        .camp(index)
        .map_or(0, |(creature, count)| {
            let offer = Offer {
                creature,
                count: i16::from_le_bytes(u16::try_from(count & 0xFFFF).unwrap_or(0).to_le_bytes()),
                free: false,
            };
            let factions = armies::alliance_factions(rules);
            let mut plan = Plan::new(rules, asked.hero.weights, vec![offer]);
            plan.purchase_value(
                (&asked.fighter().on_map.army, None),
                asked.treasury(),
                (asked.join(&factions), asked.morale()),
            )
        })
}

/// A dwelling's value (classes 17, 20; CPU-OBJ-008): `towns::dwelling_value` (CPU-TWN-001) with
/// the candidate's cost.
// rule: CPU-OBJ-008
pub(super) fn dwelling(asked: Asked<'_, '_>, index: usize, (pos, cost): (Pos, i32)) -> i32 {
    towns::dwelling_value(asked, index, (cost, pos))
}

/// A town's value (class 98; CPU-OBJ-008): `towns::town_value` (CPU-TWN-005) with the cost's low
/// word.
// rule: CPU-OBJ-008
pub(super) fn town(asked: Asked<'_, '_>, index: usize, (pos, cost): (Pos, i32)) -> i32 {
    state(asked)
        .adventure()
        .map()
        .town_id(index)
        .map_or(0, |town| {
            towns::town_value(asked, town, (i32::from(low16(cost)), pos))
        })
}
