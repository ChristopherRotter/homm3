//! The creatures that change the heroes' mana (E04b T7c1; BAT-MAG-054 to BAT-MAG-056), after the
//! original's code (`docs/formats/installation.md` §7.21, "The mana creatures, re-read for
//! T7c1"): the armies change each hero's spell costs (ability `spell_cost`: the Pegasi, the
//! Mages), a side's living Familiars give its hero a share of the other hero's cost
//! (`mana_channel`), and a Wraith takes mana from the enemy hero at its activations
//! (`mana_drain`).

use h3_content::MAX_ARMY_SLOTS;
use h3_content_battle::BattleSpell;
use h3_content_battle::{Ability, AuraTarget, BattleRuleset};
use h3_contracts::{BattleSetup, BattleSide};

use crate::event::{Event, EventKind, Sink};
use crate::state::{BattleState, StackId, StackTraits};

/// The change of the spell costs of `side`'s hero from the creatures in the armies (BAT-MAG-054):
/// the largest increase plus the largest reduction among the `spell_cost` abilities of the
/// creatures in both armies that affect that hero (its own army's for `allies`, the other's for
/// `enemies`). The original's 0x4E5240 reads the armies, which keep their creatures until the
/// battle's end, not the battle's stacks: a stack that died still counts and a stack created in
/// the battle never does (BAT-QUIRK-15, the rule).
// rule: BAT-MAG-054
pub(crate) fn cost_change(ruleset: &BattleRuleset, setup: &BattleSetup, side: BattleSide) -> i8 {
    let (mut up, mut down) = (0i8, 0i8);
    for army in BattleSide::BOTH {
        let own = army == side;
        for stack in &setup.army(army).stacks {
            for ability in ruleset.abilities(stack.creature) {
                if let Ability::SpellCost { amount, affects } = *ability
                    && (affects == AuraTarget::Allies) == own
                {
                    up = up.max(amount);
                    down = down.min(amount);
                }
            }
        }
    }
    up.saturating_add(down)
}

/// The change of the spell costs of `side`'s hero from its own army alone (the original's
/// 0x4E5240 given no enemy army, as the adventure AI's value of a spell calls it: the computer's
/// odds, BAT-AI-008, `installation.md` §7.39): [`cost_change`]'s largest increase plus largest
/// reduction among the `spell_cost` abilities for allies of the creatures in the side's army
/// slots, dead or alive (the army keeps its creatures, BAT-QUIRK-15), created and raised stacks
/// (a Pit Lord's Demons, new records) and war machines left out.
// rule: BAT-AI-008
pub(crate) fn own_cost_change(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
) -> i8 {
    let (mut up, mut down) = (0i8, 0i8);
    let army = state.stacks.iter().filter(|s| {
        s.side == side && usize::from(s.slot) < MAX_ARMY_SLOTS && !s.has(StackTraits::RAISED)
    });
    for stack in army {
        for ability in ruleset.abilities(stack.creature) {
            if let Ability::SpellCost { amount, affects } = *ability
                && affects == AuraTarget::Allies
            {
                up = up.max(amount);
                down = down.min(amount);
            }
        }
    }
    up.saturating_add(down)
}

/// The mana a hero pays for `spell` at `mastery` (BAT-MAG-054; the original's 0x4E5240): its cost
/// at that mastery changed by the armies' `change` ([`cost_change`]), at least 1; a spell that
/// costs nothing (Titan's Lightning Bolt) stays free.
// rule: BAT-MAG-054
pub(crate) fn cost(spell: &BattleSpell, mastery: u8, change: i8) -> u16 {
    let base = spell.cost[usize::from(mastery)];
    if base == 0 {
        return 0;
    }
    let changed = (i32::from(base) + i32::from(change)).max(1);
    u16::try_from(changed).unwrap_or(u16::MAX)
}

/// After a hero of `caster`'s side cast a spell that cost `cost` (BAT-MAG-055; the original's
/// 0x5A2170, after the spell's effects): if the other side has a hero and controls a living stack
/// with `mana_channel`, that hero gains the largest such percent of the cost, rounded down (the
/// Familiars' 20 %: `cost div 5`, nothing below 5), once, with no cap at its maximum. Every such
/// stack counts, a hypnotized one for its controller, wherever its record lies (the original's
/// loop misses some: BAT-QUIRK-16, always fixed).
// rule: BAT-MAG-055
pub(crate) fn channel<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (caster, cost): (BattleSide, u16),
    sink: &mut S,
) {
    let side = caster.opponent();
    let percent = state
        .stacks
        .iter()
        .filter(|s| s.is_alive() && s.controller() == side)
        .flat_map(|s| ruleset.abilities(s.creature))
        .filter_map(|a| match *a {
            Ability::ManaChannel { percent } => Some(percent),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let mana = u16::try_from(u32::from(cost) * u32::from(percent) / 100).unwrap_or(u16::MAX);
    let Some(hero) = state.heroes[side.index()].as_mut().filter(|_| mana > 0) else {
        return;
    };
    let before = hero.view.mana;
    hero.view.mana = before.saturating_add(mana);
    sink.emit(Event::public(EventKind::ManaChanneled {
        side,
        mana: hero.view.mana - before,
    }));
}

/// At an activation of stack `id` for a turn (BAT-MAG-056; the original's 0x465548): unless the
/// round's wait phase has begun (the waited stacks act), a stack with `mana_drain` takes its
/// amount from the mana of the hero of the side opposite its controller, at least 0, if that hero
/// has any: at its first turn of the round, at a good-morale turn of the normal phase and,
/// berserk, before its automatic action.
// rule: BAT-MAG-056
pub(crate) fn drain<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) {
    let stack = state.at(id);
    // Most stacks drain nothing: test that first.
    if !stack.has(StackTraits::MANA_DRAIN) || state.wait_phase {
        return;
    }
    let (side, creature) = (stack.controller().opponent(), stack.creature);
    // A draining stack has its ability (the trait comes from it).
    let amount = ruleset
        .abilities(creature)
        .iter()
        .find_map(|a| match *a {
            Ability::ManaDrain { amount } => Some(amount),
            _ => None,
        })
        .unwrap_or(0);
    let Some(hero) = state.heroes[side.index()]
        .as_mut()
        .filter(|h| h.view.mana > 0)
    else {
        return;
    };
    let mana = hero.view.mana.min(u16::from(amount));
    hero.view.mana -= mana;
    sink.emit(Event::public(EventKind::ManaDrained {
        stack: id,
        side,
        mana,
    }));
}
