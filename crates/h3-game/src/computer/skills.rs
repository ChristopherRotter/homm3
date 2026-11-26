//! The computer's values of a hero's secondary skills (E31 T2e, AI-Q36; survey
//! `docs/rules/surveys/E31-computer-player.md` §2.4): `ai_secondary_skill_value` 0x524690 with
//! its flag set, whose switch is the level-up's evaluation in `h3_heroes::choice` (one
//! implementation with the setup's path, ADR-0030 point 6; CPU-SKL-001), fed by [`reads`]; a
//! magic school's value (`ai_school_value` 0x524D20, [`school_value`]) and a University's
//! (`ai_university_value` 0x525BF0, [`university_value`]). Nothing here changes the hero: the
//! original writes the school's level into it and back, the port values a changed copy (ADR-0030
//! point 4).

use h3_content::{CreatureFlag, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, SkillId};
use h3_content_battle::Ability;
use h3_heroes::{ComputerReads, computer_skill_value, computer_takes_new_skill, counted_worth};
use h3_kernel::arith::SignedX87;

use super::fight::Fighter;
use super::spells::{self, SpellGains};
use super::start::Record;
use crate::{GameRuleset, GameState};

/// The original's ids of the four magic schools, Fire to Earth (0x524690's cases 14 to 17).
const SCHOOLS: [u16; 4] = [14, 15, 16, 17];

/// The gold a University's value asks the player for when asked to (0x525C17: 2 000).
const UNIVERSITY_GOLD: i32 = 2000;

/// The value of the magic school `school` to `hero` (`ai_school_value` 0x524D20; CPU-SKL-002):
/// the best value of every kind of its spells (`CasterRecord::best`) with the school one level
/// higher, or at Expert when it lacks the school, less the best value now (32 bits). A new school
/// counts at Expert at once (survey §5.5 #17, a slip ported as read).
// rule: CPU-SKL-002
pub(super) fn school_value(
    state: &GameState,
    rules: &GameRuleset,
    hero: Fighter<'_>,
    school: SkillId,
) -> i32 {
    let battle = rules.battle();
    let all = spells::every_kind();
    let now = spells::record(state, rules, hero).best(battle, all);
    let mut raised = hero.details.clone();
    match raised.skills.iter_mut().find(|(skill, _)| *skill == school) {
        Some((_, level)) => *level = level.wrapping_add(1),
        None => raised.skills.push((school, MAX_SKILL_LEVEL)),
    }
    let raised = Fighter {
        on_map: hero.on_map,
        details: &raised,
    };
    spells::record(state, rules, raised)
        .best(battle, all)
        .wrapping_sub(now)
}

/// What the flag-set path of the skill value reads of `hero` (0x524690's flag set; CPU-SKL-001):
/// its army counted (`h3_heroes::counted_worth`), its spell values per unit `gains` (its +0x47E
/// and +0x486, as the computer last wrote them: [`super::start`]'s record), its owner's gold
/// weight (the start's `Record::weights` at the market's cost resource, the base pack's gold;
/// 0 without the market), each school's [`school_value`] where the switch reaches it (a school
/// below Expert, known or with a free skill slot; else 0, which nothing reads), the Ballista
/// (a worn war machine that shoots) and the First Aid Tent (one with `first_aid`) it wears, and
/// the map's terrain shares.
pub(super) fn reads(
    state: &GameState,
    rules: &GameRuleset,
    hero: Fighter<'_>,
    gains: &SpellGains,
    start: &Record,
) -> ComputerReads {
    let (core, battle) = (rules.core(), rules.battle());
    let details = hero.details;
    let gold = rules.adventure().params().market().map(|m| m.cost());
    let gold_weight = gold
        .and_then(|g| start.weights(rules).get(g.index()).copied())
        .unwrap_or(SignedX87::ZERO);
    let level = |skill: SkillId| {
        details
            .skills
            .iter()
            .find(|&&(s, _)| s == skill)
            .map_or(0, |&(_, l)| l)
    };
    let reached = |skill: SkillId| {
        let l = level(skill);
        l < MAX_SKILL_LEVEL && (l > 0 || details.skills.len() < MAX_HERO_SKILLS)
    };
    let schools = SCHOOLS.map(|raw| {
        let skill = SkillId::from_raw(raw);
        if reached(skill) {
            school_value(state, rules, hero, skill)
        } else {
            0
        }
    });
    let machines = h3_heroes::war_machines(details, core);
    let ballista = machines.iter().any(|&c| {
        battle
            .creature(c)
            .is_some_and(|c| c.flags.contains(CreatureFlag::Shooter))
    });
    let tent = machines.iter().any(|&c| {
        battle
            .abilities(c)
            .iter()
            .any(|a| matches!(a, Ability::FirstAid { .. }))
    });
    ComputerReads {
        worth: counted_worth(core, &hero.on_map.army),
        power_gain: gains.power,
        knowledge_gain: gains.knowledge,
        gold_weight,
        schools,
        ballista,
        tent,
        shares: state.adventure().map().terrain_shares(),
    }
}

/// A University's value to `hero` (`ai_university_value` 0x525BF0; CPU-SKL-003) offering
/// `skills`, with the flag-set path's `reads` ([`reads`]): 0 when it knows the most skills, or
/// with `gold_test` while its owner has less than 2 000 gold (the market's cost resource); else
/// the sum (32 bits) of the flag-set skill values (`h3_heroes::computer_skill_value`) of the
/// offered skills its class may get (a weight above 0), it lacks and it would take
/// (`h3_heroes::computer_takes_new_skill`). All four count though the gold may buy one (survey
/// §5.5 #20, a slip ported as read).
// rule: CPU-SKL-003
pub(super) fn university_value(
    state: &GameState,
    rules: &GameRuleset,
    hero: Fighter<'_>,
    reads: &ComputerReads,
    (skills, gold_test): ([SkillId; 4], bool),
) -> i32 {
    let (core, details) = (rules.core(), hero.details);
    if details.skills.len() >= MAX_HERO_SKILLS {
        return 0;
    }
    if gold_test {
        let gold = rules.adventure().params().market().map(|m| m.cost());
        let stock = gold
            .and_then(|g| {
                state
                    .treasury(hero.on_map.owner)
                    .and_then(|t| t.get(g.index()).copied())
            })
            .map_or(0, |amount| i32::from_ne_bytes(amount.to_ne_bytes()));
        if stock < UNIVERSITY_GOLD {
            return 0;
        }
    }
    let class = core
        .hero_type(details.hero_type)
        .and_then(|h| core.hero_class(h.class));
    let weighed = |skill: SkillId| {
        class.is_some_and(|c| c.skill_weights.iter().any(|&(s, w)| s == skill && w > 0))
    };
    let lacks = |skill: SkillId| details.skills.iter().all(|&(s, _)| s != skill);
    skills
        .into_iter()
        .filter(|&skill| {
            weighed(skill) && lacks(skill) && computer_takes_new_skill(details, core, reads, skill)
        })
        .fold(0_i32, |sum, skill| {
            sum.wrapping_add(computer_skill_value(details, core, reads, skill))
        })
}
