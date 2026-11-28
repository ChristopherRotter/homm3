//! The object value's visits and pickups (E31 T2h; survey §2.1): the experience and skill
//! objects, the morale, luck and movement objects, the finds and the weekly objects, the markets
//! and the flat values. Every class `super::value` does not send to `fights`.

use h3_adventure::{ObjectKind, ObjectTypeId, Pos};
use h3_heroes::ArtifactInstance;
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87, X87};

use super::super::artifacts;
use super::super::skills;
use super::super::towns::Asked;
use super::super::vision;
use super::{
    Asking, COVERED, Class, GEMS, SKILLS, WOOD, at, backpack_full, gold_weight, learnable_spell,
    level, low16, luck, luck_change, luck_gain, mean, morale_change, morale_gain, movement,
    next_level, of_strength, resource_weight, single, undead, weight,
};

/// A Keymaster's Tent's value while the player lacks its key (0x52847D: 5 000).
const KEY: i32 = 5000;

/// A Lighthouse's or a Shipyard's value while not the team's (0x529264: 1 000).
const OWNABLE: i32 = 1000;

/// The radius of the Redwood Observatory's and the Pillar of Fire's circle (0x52913B: 20).
const OBSERVATORY: i32 = 20;

/// The Library's condition: the level plus twice Diplomacy at least this (0x528F92: 10).
const LIBRARY: i32 = 10;

/// The Diplomacy skill (the original's skill 4).
const DIPLOMACY: h3_content::SkillId = h3_content::SkillId::from_raw(4);

/// The gold a School asks (0x529FB5, 0x52B7B9: 1 000).
const SCHOOL_GOLD: i32 = 1000;

/// The Tree of Knowledge's prices as the computer reads them (0x52B647: 10 gems, 0x52B67F:
/// 2 000 gold), by the drawn price's index (1 gold, 2 gems).
const TREE_GOLD: i32 = 2000;
const TREE_GEMS: i32 = 10;

/// The cost within which the Magic Well and Spring count wherever the hero is headed (0x52A57D:
/// 300).
const NEAR: u16 = 300;

/// The Swan Pond's points the hero may keep at most (0x528586: 200).
const SWAN_LEFT: i32 = 200;

/// The Stables' base values: its bonus off the cost, or covering it (0x52AB08: 50, 0x52AB17:
/// 10 000).
const STABLES: (i32, i32) = (50, 10_000);

/// The days of a week (the Stables' bonus is for the days left, 8 − the day).
const WEEK: i32 = 8;

/// The War Machine Factory's cost within which a machine's price counts nothing (0x5251D8: 500;
/// survey §5.5 #5, a slip ported as read).
const FACTORY_NEAR: i32 = 500;

/// The artifacts a full backpack holds for the Black Market's test (0x528403: exactly 64).
const BACKPACK: usize = 64;

/// The kind's once-mark and movement points, of a bonus visitable (none, 0 for every other).
pub(super) fn bonus(kind: &ObjectKind) -> (Option<u8>, u32) {
    match kind {
        ObjectKind::Bonus { mark, points, .. } => (Some(*mark), *points),
        _ => (None, 0),
    }
}

/// `n` as `fild` loads it (an integer into the FPU, exact).
fn int(n: i32) -> SignedX87 {
    SignedX87::int(n.into())
}

/// The double nearest `num / den`.
fn double(num: u32, den: u32) -> SignedX87 {
    X87::ratio(num, den, DOUBLE_BITS).map_or(SignedX87::ZERO, SignedX87::from)
}

/// The single nearest `num / den`.
fn float(num: u32, den: u32) -> SignedX87 {
    X87::ratio(num, den, SINGLE_BITS).map_or(SignedX87::ZERO, SignedX87::from)
}

/// The player's stock of `resource` (+0x9C).
fn stock(asked: Asked<'_, '_>, resource: h3_content::ResourceId) -> i32 {
    asked.treasury().get(resource.index()).copied().unwrap_or(0)
}

/// The player's gold (+0xB4: the market's cost resource), 0 without the market.
fn gold(asked: Asked<'_, '_>) -> i32 {
    let gold = asked
        .hero
        .rules
        .adventure()
        .params()
        .market()
        .map(|m| m.cost());
    gold.map_or(0, |g| stock(asked, g))
}

/// The value of the visit or pickup of class `class` at `pos` (CPU-OBJ-009 to -011): the object
/// at `index`, of type `ty` and kind `kind`, `cost` the candidate's cost; every class the
/// switch does not list 0.
pub(super) fn value(
    asking: Asking<'_, '_>,
    (index, ty, kind, class): (usize, ObjectTypeId, &ObjectKind, Class),
    pos: Pos,
    cost: &mut i32,
) -> i32 {
    let asked = asking.asked;
    let adventure = asked.state().adventure();
    let hero = asked.fighter().on_map.id;
    let player = asked.player();
    let marked = adventure.is_marked(hero, index);
    let known = adventure.knows(player, index);
    let (mark, points) = bonus(kind);
    let marks = adventure.visit_marks(hero);
    let bonus_marked = mark.is_some_and(|m| marks.bonuses().has_mark(m));
    let timed = marks.timed().binary_search(&ty).is_ok();
    let reachable = *cost <= movement(asked);
    let gains = asked.values().spells;
    match class {
        Class::Arena if !marked => single(next_level(asked).wrapping_mul(2).into())
            .times(weight(asked))
            .ftol(),
        Class::OneLevel if !marked => level(asked),
        Class::Scholar => level(asked),
        Class::GardenOfRevelation if !marked => gains.knowledge,
        Class::StarAxis if !marked => gains.power,
        Class::Library if !marked => library(asked),
        Class::LearningStone if !marked => weight(asked).times(float(1000, 1)).ftol(),
        Class::SchoolOfMagic if !marked && gold(asked) >= SCHOOL_GOLD => {
            int(gains.power.max(gains.knowledge))
                .minus(gold_weight(asked).times(double(1000, 1)))
                .ftol()
        }
        Class::SchoolOfWar if !marked && gold(asked) >= SCHOOL_GOLD => {
            single(next_level(asked).into())
                .times(weight(asked))
                .minus(gold_weight(asked).times(double(1000, 1)))
                .ftol()
        }
        Class::Tree if !marked => tree(asked, index, known),
        Class::University => university(asked, index),
        Class::WitchHut => witch_hut(asked, index, known),
        Class::Shrine => drawn_spell(asked, index).map_or(0, |s| learnable_spell(asked, s)),
        Class::Buoy if !bonus_marked && reachable => morale_gain(asked, 1),
        Class::FaerieRing if !bonus_marked && reachable => luck_gain(asked, 1),
        Class::SwanPond if !bonus_marked && reachable => {
            let left = movement(asked);
            if cost.wrapping_add(SWAN_LEFT) < left {
                return 0;
            }
            *cost = (*cost).max(left);
            luck_gain(asked, 2)
        }
        Class::Oasis if !bonus_marked => oasis(asked, points_off(points), cost),
        Class::RallyFlag if reachable && !bonus_marked => rally(asked, points_off(points), cost),
        Class::Temple if !bonus_marked && reachable => morale_gain(asked, 2),
        Class::Stables => stables(asked, (kind, timed), cost),
        Class::Sirens => sirens(asked),
        Class::MagicWell if !timed => near(asking, gains.refilled, *cost),
        Class::MagicSpring if adventure.emptied_springs().binary_search(&pos).is_err() => {
            near(asking, gains.doubled, *cost)
        }
        Class::Observatory => {
            vision::value(asked.state(), asked.hero.rules, player, OBSERVATORY, pos)
        }
        Class::Keymaster => match kind {
            ObjectKind::KeyTent { key } if !adventure.has_key(player, *key) => KEY,
            _ => 0,
        },
        Class::Lighthouse | Class::Shipyard => {
            if adventure
                .owner(index)
                .is_some_and(|o| adventure.allied(o, player))
            {
                0
            } else {
                OWNABLE
            }
        }
        Class::Campfire => campfire(asked, index),
        Class::Flotsam => gold_weight(asked)
            .times(double(175, 1))
            .plus(resource_weight(asked, WOOD).times(double(5, 1)))
            .ftol(),
        Class::SeaChest => {
            let gold = gold_weight(asked).times(double(1200, 1));
            if backpack_full(asked) {
                gold.ftol()
            } else {
                mean(asked)
                    .over(float(10, 1))
                    .unwrap_or(SignedX87::ZERO)
                    .plus(gold)
                    .ftol()
            }
        }
        Class::Survivor if !backpack_full(asked) => mean(asked).ftol(),
        Class::WarriorsTomb if !known && !backpack_full(asked) => mean(asked).ftol(),
        Class::Corpse if !known => {
            if backpack_full(asked) {
                gold_weight(asked).times(double(200, 1)).ftol()
            } else {
                mean(asked)
                    .over(float(5, 1))
                    .unwrap_or(SignedX87::ZERO)
                    .ftol()
            }
        }
        Class::LeanTo if !known => asked.record.mean.wrapping_mul(3),
        Class::Wagon if !known => {
            let resources = asked.record.mean.wrapping_mul(7) / 4;
            mean(asked)
                .plus(mean(asked))
                .over(float(5, 1))
                .unwrap_or(SignedX87::ZERO)
                .plus(single(resources.into()))
                .ftol()
        }
        Class::TreasureChest => treasure_chest(asked),
        Class::MysticalGarden => adventure.weekly(index).map_or(0, |weekly| {
            if weekly.amount == 0 {
                return 0;
            }
            gold_weight(asked)
                .times(double(500, 1))
                .plus(resource_weight(asked, GEMS).times(double(5, 1)))
                .over(double(2, 1))
                .unwrap_or(SignedX87::ZERO)
                .ftol()
        }),
        Class::Windmill => {
            let empty = adventure.weekly(index).is_some_and(|w| w.amount == 0);
            if adventure.visited_by(index, player) && empty {
                0
            } else {
                asked.record.mean.wrapping_mul(9) / 2
            }
        }
        Class::WaterWheel => {
            if adventure.visited_by(index, player) {
                let amount = adventure.weekly(index).map_or(0, |w| w.amount);
                int(low16(i32::from_ne_bytes(amount.to_ne_bytes())).into())
                    .times(gold_weight(asked))
                    .ftol()
            } else {
                gold_weight(asked).times(double(1000, 1)).ftol()
            }
        }
        Class::BlackMarket => black_market(asked, (index, kind)),
        Class::WarMachineFactory => match kind {
            ObjectKind::WarMachineFactory(factory) => {
                factory.machines.iter().fold(0_i32, |sum, &machine| {
                    sum.wrapping_add(war_machine(asked, machine, *cost))
                })
            }
            _ => 0,
        },
        _ => 0,
    }
}

/// The movement objects' cost off: half their points (the Fountain of Youth's, the Watering
/// Hole's and the Rally Flag's 200, the Oasis's 400; 0x529106, 0x529480, 0x52A60E).
fn points_off(points: u32) -> i32 {
    i32::from_ne_bytes((points / 2).to_ne_bytes())
}

/// The Library of Enlightenment's value (class 41, 0x528F72; CPU-OBJ-009): 0 below its condition
/// (the hero's level plus twice its Diplomacy level under 10); else twice its Spell Power's and
/// Knowledge's values plus its army's value times 4 (32 bits) over 40.
// rule: CPU-OBJ-009
fn library(asked: Asked<'_, '_>) -> i32 {
    let details = asked.fighter().details;
    let reach = i32::from(details.level) + 2 * i32::from(asked.hero.skill(DIPLOMACY));
    if reach < LIBRARY {
        return 0;
    }
    let gains = asked.values().spells;
    let army = super::super::fight::value_of(asked.hero.rules, &asked.fighter().on_map.army);
    (army.wrapping_mul(4) / 40)
        .wrapping_add(gains.power.wrapping_add(gains.knowledge).wrapping_mul(2))
}

/// The Tree of Knowledge's value (class 102, `ai_tree_of_knowledge_value` 0x52B5A0;
/// CPU-OBJ-009): one level's experience less its price: known to the player, the drawn price's
/// (its index 1 2 000 gold, 2 10 gems: 0 when the player cannot pay, else the level less its
/// weight, truncated; any other free); unknown, the expected price, a third of both prices'
/// weights, at most two thirds of the level.
// rule: CPU-OBJ-009
fn tree(asked: Asked<'_, '_>, index: usize, known: bool) -> i32 {
    let v = level(asked);
    let gold = gold_weight(asked).times(double(2000, 1));
    let gems = resource_weight(asked, GEMS).times(double(10, 1));
    if !known {
        let cap = gold
            .plus(gems)
            .over(double(3, 1))
            .unwrap_or(SignedX87::ZERO)
            .ftol();
        return v.wrapping_sub(cap.min(v.wrapping_mul(2) / 3));
    }
    let price = asked
        .state()
        .adventure()
        .object(index)
        .map_or(0, |object| match object.resolved {
            h3_adventure::Resolved::Price(price) => price,
            _ => 0,
        });
    match price {
        1 if self::gold(asked) < TREE_GOLD => 0,
        1 => int(v).minus(gold).ftol(),
        2 if stock(asked, GEMS) < TREE_GEMS => 0,
        2 => int(v).minus(gems).ftol(),
        _ => v,
    }
}

/// The skills' reads of the asked hero (`skills::reads`).
fn reads(asked: Asked<'_, '_>) -> h3_heroes::ComputerReads {
    let gains = asked.values().spells;
    skills::reads(
        asked.state(),
        asked.hero.rules,
        asked.fighter(),
        &gains,
        asked.record,
    )
}

/// A University's value (class 104; CPU-OBJ-009): `skills::university_value` (CPU-SKL-003) of its
/// drawn skills with the gold's test.
// rule: CPU-OBJ-009
fn university(asked: Asked<'_, '_>, index: usize) -> i32 {
    let adventure = asked.state().adventure();
    let offered = adventure
        .object(index)
        .map_or(&[][..], |object| object.resolved.skills());
    skills::university_value(
        asked.state(),
        asked.hero.rules,
        asked.fighter(),
        &reads(asked),
        (offered, true),
    )
}

/// A Witch Hut's value (class 113, `ai_witch_hut_value` 0x52B900; CPU-OBJ-009): its skill known to
/// the player: 0 with eight skills, without a skill, a skill the hero has or one the computer
/// would not take (`h3_heroes::computer_takes_new_skill`); else its value
/// (`h3_heroes::computer_skill_value`). Unknown: one level's experience.
// rule: CPU-OBJ-009
fn witch_hut(asked: Asked<'_, '_>, index: usize, known: bool) -> i32 {
    if !known {
        return level(asked);
    }
    let details = asked.fighter().details;
    let core = asked.hero.rules.core();
    let skill = asked
        .state()
        .adventure()
        .object(index)
        .and_then(|object| match object.resolved {
            h3_adventure::Resolved::Skill(skill) => Some(skill),
            _ => None,
        });
    match skill {
        Some(skill)
            if details.skills.len() < SKILLS
                && asked.hero.skill(skill) == 0
                && h3_heroes::computer_takes_new_skill(details, core, &reads(asked), skill) =>
        {
            h3_heroes::computer_skill_value(details, core, &reads(asked), skill)
        }
        _ => 0,
    }
}

/// A Shrine's spell (the map's or the build's draw).
fn drawn_spell(asked: Asked<'_, '_>, index: usize) -> Option<h3_content::SpellId> {
    asked
        .state()
        .adventure()
        .object(index)
        .and_then(|object| match object.resolved {
            h3_adventure::Resolved::Spell(spell) => Some(spell),
            _ => None,
        })
}

/// The Fountain of Youth's, the Oasis's and the Watering Hole's value (classes 31, 56, 110,
/// `ai_oasis_value` 0x52A1E0; CPU-OBJ-010), `off` the cost they take off: a cost of at least `off`
/// lowered by it and one morale's value (0 for an undead army); a smaller one set to 0, worth
/// 10 000.
// rule: CPU-OBJ-010
fn oasis(asked: Asked<'_, '_>, off: i32, cost: &mut i32) -> i32 {
    if *cost < off {
        *cost = 0;
        return COVERED;
    }
    *cost -= off;
    if undead(asked) {
        0
    } else {
        of_strength(asked, morale_change(asked, 1))
    }
}

/// The Rally Flag's value (class 64, `ai_rally_flag_value` 0x52A5F0; CPU-OBJ-010), `off` its cost
/// off: as [`oasis`]'s part, plus the bare morale and luck fractions of one point each, in
/// double precision, truncated.
// rule: CPU-OBJ-010
fn rally(asked: Asked<'_, '_>, off: i32, cost: &mut i32) -> i32 {
    let part = oasis(asked, off, cost);
    morale_change(asked, 1)
        .plus(luck_change(luck(asked), 1))
        .plus(int(part))
        .ftol()
}

/// The Stables' value (class 94, `ai_stables_value` 0x52AAC0; CPU-OBJ-010), `timed` whether the
/// hero holds its week's mark: without it, half its points for the week's days left (8 − the
/// day) off the cost (a word), the base 50, or the cost set to 0 and the base 10 000 when that
/// covers it; with it, the base 0. Plus, with a stack of the creature it upgrades, its AI gain
/// times the number of such stacks (`army_count_type` 0x4E2340 counts stacks: survey §5.5 #72, a
/// slip ported as read), times 1.2 (truncated) when a stack of the upgrade is there too.
// rule: CPU-OBJ-010
fn stables(asked: Asked<'_, '_>, (kind, timed): (&ObjectKind, bool), cost: &mut i32) -> i32 {
    let ObjectKind::Stables { points, from, to } = kind else {
        return 0;
    };
    let mut base = 0;
    if !timed {
        let day = i32::from(asked.state().weekday(asked.hero.rules).get());
        let bonus = WEEK
            .wrapping_sub(day)
            .wrapping_mul(i32::from_ne_bytes(points.to_ne_bytes()))
            / 2;
        let bonus = i32::from(low16(bonus));
        if *cost >= bonus {
            *cost -= bonus;
            base = STABLES.0;
        } else {
            *cost = 0;
            base = STABLES.1;
        }
    }
    let army = &asked.fighter().on_map.army;
    let stacks = |creature| {
        army.iter()
            .filter(|s| s.creature == creature && s.count > 0)
            .fold(0_i32, |n, _| n + 1)
    };
    let n = stacks(*from);
    if n == 0 {
        return base;
    }
    let core = asked.hero.rules.core();
    let ai = |c| {
        core.creature(c)
            .map_or(0, |info| i32::from_ne_bytes(info.ai_value.to_ne_bytes()))
    };
    let mut up = ai(*to).wrapping_sub(ai(*from)).wrapping_mul(n);
    if stacks(*to) > 0 {
        up = int(up).times(double(12, 10)).ftol();
    }
    up.wrapping_add(base)
}

/// The Sirens' value (class 92, `ai_sirens_value` 0x52A960; CPU-OBJ-010): on a copy of the army,
/// each stack above 1 keeps `trunc(f32(count) × 0.7)` (a word); the lost creatures' hit points
/// are the experience, grown by the hero's Learning; the strength lost is the army's value lost
/// times 40 plus the primary sum, over 40. With a weight of 0, less that; else the experience
/// at the value of the army left (`fight::experience_value`) less the strength lost, in
/// single-precision operands, truncated. No mark is read: a hero that visited them values them
/// alike (survey §5.5 #73, a slip ported as read).
// rule: CPU-OBJ-010
fn sirens(asked: Asked<'_, '_>) -> i32 {
    let rules = asked.hero.rules;
    let fighter = asked.fighter();
    let army = &fighter.on_map.army;
    let before = super::super::fight::value_of(rules, army);
    let mut left = army.clone();
    let mut lost = 0_i32;
    for stack in &mut left {
        let count = i32::from_ne_bytes(stack.count.to_ne_bytes());
        if count > 1 {
            let keep = i32::from(low16(single(count.into()).times(double(7, 10)).ftol()));
            let hit_points = rules
                .battle()
                .creature(stack.creature)
                .map_or(0, |c| c.hit_points);
            lost = lost.wrapping_add(
                i32::from_ne_bytes(hit_points.to_ne_bytes()).wrapping_mul(count - keep),
            );
            stack.count = u32::from_ne_bytes(keep.to_ne_bytes());
        }
    }
    let experience = h3_heroes::gained_experience(
        fighter.details,
        rules.core(),
        u32::from_ne_bytes(lost.to_ne_bytes()),
    );
    let after = super::super::fight::value_of(rules, &left);
    let sum = super::super::turn::primary_sum_of(fighter.details, rules.core());
    let lost_strength = before.wrapping_sub(after).wrapping_mul(40 + i32::from(sum)) / 40;
    if weight(asked) == SignedX87::ZERO {
        return lost_strength.wrapping_neg();
    }
    let next = h3_heroes::next_level_experience(fighter.details, rules.core());
    super::super::fight::experience_value(after, next).map_or(i32::MIN, |value| {
        value
            .times(single(i64::from(experience)))
            .minus(single(lost_strength.into()))
            .ftol()
    })
}

/// A Magic Well's or Spring's `value` (classes 49, 48; `ai_magic_well_value` 0x52A510,
/// `ai_magic_spring_value` 0x52B810; CPU-OBJ-010): kept within a cost of 300 (the cost's low word,
/// unsigned) or without a kept destination; farther, only while the hero is headed to a Well or
/// a Spring.
// rule: CPU-OBJ-010
fn near(asking: Asking<'_, '_>, value: i32, cost: i32) -> i32 {
    let asked = asking.asked;
    let far = u16::from_ne_bytes(low16(cost).to_ne_bytes()) > NEAR;
    match asking.destination {
        Some(destination) if far => {
            let headed = at(asked.state(), asked.hero.rules, destination)
                .is_some_and(|at| matches!(at.class(), Class::MagicWell | Class::MagicSpring));
            if headed { value } else { 0 }
        }
        _ => value,
    }
}

/// A Campfire's value (class 12, 0x52851E; CPU-OBJ-011): its amount of its resource and 100
/// times that in gold, at their weights, in double precision, truncated.
// rule: CPU-OBJ-011
fn campfire(asked: Asked<'_, '_>, index: usize) -> i32 {
    let gold = asked
        .hero
        .rules
        .adventure()
        .params()
        .market()
        .map(|m| m.cost());
    let contents = asked
        .state()
        .adventure()
        .object(index)
        .map_or(&[][..], |object| &object.contents[..]);
    contents
        .iter()
        .find(|&&(r, _)| Some(r) != gold)
        .map_or(0, |&(resource, amount)| {
            let amount = i32::from_ne_bytes(amount.to_ne_bytes());
            int(amount.wrapping_mul(100))
                .times(gold_weight(asked))
                .plus(int(amount).times(resource_weight(asked, resource)))
                .ftol()
        })
}

/// A Treasure Chest's value (class 101, `ai_treasure_chest_value` 0x52B4E0; CPU-OBJ-011): over its
/// three tiers the larger of the experience (160, 320, 465 times the hero's weight, single
/// operands) and the gold (320, 480, 620 at its weight), each truncated, plus a twentieth of the
/// player's mean artifact value, in single-precision operands, truncated (the chest's own
/// contents are not read).
// rule: CPU-OBJ-011
fn treasure_chest(asked: Asked<'_, '_>) -> i32 {
    let tiers = [(160, 320), (320, 480), (465, 620)];
    let sum = tiers.iter().fold(0_i32, |sum, &(experience, gold)| {
        let e = weight(asked).times(float(experience, 1)).ftol();
        let g = gold_weight(asked).times(double(gold, 1)).ftol();
        sum.wrapping_add(e.max(g))
    });
    mean(asked)
        .over(float(20, 1))
        .unwrap_or(SignedX87::ZERO)
        .plus(single(sum.into()))
        .ftol()
}

/// The Black Market's value (class 7, 0x5283FA; CPU-OBJ-011): 0 with a backpack of exactly 64;
/// else the sum over its stock's unsold entries of their values (`ai_black_market_artifact_value`
/// 0x529750: the price the computer would pay, [`market_price`], 0 when the player cannot; else
/// the artifact's value to the player less the price at its resource's weight, truncated, at
/// least 0).
// rule: CPU-OBJ-011
fn black_market(asked: Asked<'_, '_>, (index, kind): (usize, &ObjectKind)) -> i32 {
    let fighter = asked.fighter();
    if fighter.details.backpack.len() == BACKPACK {
        return 0;
    }
    let count = match kind {
        ObjectKind::Market(market) => market.count,
        _ => 0,
    };
    let adventure = asked.state().adventure();
    let heroes = artifacts::players_heroes(
        asked.state(),
        asked.hero.rules,
        (asked.record, asked.hero.weights),
        (asked.player(), &[]),
    );
    let entries = adventure
        .object(index)
        .map_or(&[][..], |object| object.resolved.stock());
    (0..=u8::MAX)
        .zip(entries)
        .filter(|&(entry, _)| !adventure.is_sold(index, entry))
        .fold(0_i32, |sum, (_, &artifact)| {
            let value = market_price(asked, artifact, count).map_or(0, |(price, resource)| {
                if price > stock(asked, resource) {
                    return 0;
                }
                let worth = artifacts::value_to_player(&heroes, ArtifactInstance::from(artifact));
                int(worth)
                    .minus(int(price).times(resource_weight(asked, resource)))
                    .ftol()
                    .max(0)
            });
            sum.wrapping_add(value)
        })
}

/// The price the computer would pay for `artifact` at a market of `count` Marketplaces
/// (`ai_artifact_market_price` 0x524550; CPU-OBJ-011): in gold (the market's cost resource) its
/// cost over the artifacts' rate (single operands, truncated: the human's rounds, survey §5.3);
/// then, for each other resource in order whose amount `2p / its value` the player holds, that
/// amount when its weighed worth is not above the cheapest so far (the later on ties). None
/// without the market's parameters.
// rule: CPU-OBJ-011
fn market_price(
    asked: Asked<'_, '_>,
    artifact: h3_content::ArtifactTypeId,
    count: u8,
) -> Option<(i32, h3_content::ResourceId)> {
    let rules = asked.hero.rules;
    rules.adventure().params().market().map(|market| {
        let gold = market.cost();
        let cost = rules
            .core()
            .artifact(artifact)
            .and_then(|a| a.cost.iter().find(|&&(r, _)| r == gold))
            .map_or(0, |&(_, c)| c);
        let rate = market
            .factor_bits(count)
            .map_or(SignedX87::ONE, |[_, artifacts, _]| {
                SignedX87::from_f32_bits(artifacts)
            });
        let p = single(i64::from(cost))
            .over(rate)
            .unwrap_or(SignedX87::ZERO)
            .ftol();
        let resources = rules.core().resources().len();
        let start = (int(p).times(gold_weight(asked)).ftol(), p, gold);
        let (_, price, resource) = (0..=u16::MAX)
            .take(resources)
            .map(h3_content::ResourceId::from_raw)
            .filter(|&r| r != gold)
            .fold(start, |best, resource| {
                let value = market.value(resource).map_or(0, i32::from);
                let amount = p.wrapping_mul(2).checked_div(value).unwrap_or(0);
                if amount > stock(asked, resource) {
                    return best;
                }
                let worth = int(amount).times(resource_weight(asked, resource)).ftol();
                if worth <= best.0 {
                    (worth, amount, resource)
                } else {
                    best
                }
            });
        (price, resource)
    })
}

/// A war machine's value at a War Machine Factory (`ai_war_machine_value` 0x525120;
/// CPU-OBJ-011), `creature` the machine and `artifact` its artifact (the kind's pair; the
/// original's `war_machine_creature` 0x550360 of the artifact): 0 when the hero carries it or on Easy; else
/// its value to the hero, counted (`artifacts::hero_value`), less its creature's cost weighed
/// (the running total of each resource's cost at its weight, truncated at each step; 0 when the
/// player cannot pay a resource of it, and nothing within a cost of 500: survey §5.5 #5, a slip
/// ported as read), 0 when the price is above the value.
// rule: CPU-OBJ-011
fn war_machine(
    asked: Asked<'_, '_>,
    (creature, artifact): (h3_content::CreatureTypeId, h3_content::ArtifactTypeId),
    cost: i32,
) -> i32 {
    let state = asked.state();
    let rules = asked.hero.rules;
    let details = asked.fighter().details;
    let carried = details
        .worn
        .iter()
        .flatten()
        .chain(&details.backpack)
        .any(|i| i.artifact == artifact);
    if carried || state.adventure().difficulty() == 0 {
        return 0;
    }
    let value = artifacts::hero_value(asked.hero, ArtifactInstance::from(artifact), false, true);
    let costs = super::super::armies::creature_costs(rules, creature);
    let treasury = asked.treasury();
    if costs.iter().zip(&treasury).any(|(c, have)| have < c) {
        return 0;
    }
    let mut price = super::super::armies::weighted_cost(asked.hero.weights, &costs);
    if cost <= FACTORY_NEAR {
        price = 0;
    }
    if price > value { 0 } else { value - price }
}
