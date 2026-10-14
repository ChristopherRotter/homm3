//! Action encoding (docs/07-simulation-and-ai.md §1: "dense action encoding is defined by the
//! sim layer, not the domain"), factorized for casting (ADR-0014 point 9).
//!
//! An [`Action`] is an *entry* and three heads: a spell and two targets. Every command but a
//! cast is one entry of the dense list below with all three heads 0; a hero's cast is the entry
//! [`HERO_CAST`] with the spell's handle in the spell head and its target in the target head
//! (`0` none, `1 + h` hex `h < 255`, `256 + s` stack slot `s`); the second target has the same
//! layout: Teleport's destination hex (BAT-MAG-029), Sacrifice's victim stack (BAT-MAG-036, E04b
//! T6a; the layout is unchanged), 0 for every other spell. A stack's cast (BAT-MAG-048) is the
//! entry [`CREATURE_CAST`]: its command names only the target (ADR-0014 point 3), so the spell
//! head allows only the active stack's spell ([`h3_battle::stack_spell`]: a random caster's drawn
//! spell, BAT-MAG-049, whose area spells take a hex in the target head; only 0 for a spell drawn
//! at the cast, the Master Genie's, BAT-MAG-050) and the second target head only 0. The heads' masks
//! are conditioned on the earlier choices ([`legal_mask`], [`spell_mask`], [`target_mask`],
//! [`second_mask`]); after a legal non-cast entry each head allows exactly 0.
//!
//! Entries, version 13 (independent of the ruleset, so agents see a fixed-size action space):
//!
//! | entry | action |
//! |---|---|
//! | 0 | Wait |
//! | 1 | Defend |
//! | 2 + h, h < 255 | Move to the position `h` (a two-hex stack's rear hex, BAT-GEO-002) |
//! | 257 + 24·s + k | Attack stack `s` from its canonical attack position `k` (`Field::attack_positions` for the active stack's shape) |
//! | 1265 + 2·s + a | Shoot at stack `s`, aiming at its front hex (`a = 0`) or a two-hex stack's rear hex (`a = 1`) |
//! | 1349 + s | Heal stack `s` (the commanded First Aid Tent, BAT-WAR-007) |
//! | 1391 | End the tactics phase (BAT-TAC-002) |
//! | 1392 | Retreat (BAT-FLE-001) |
//! | 1393 | Surrender (BAT-FLE-002) |
//! | 1394 | The hero casts (heads: spell, target, second target; BAT-MAG-001) |
//! | 1395 | The active stack casts (heads: its spell or 0, the target, 0; BAT-MAG-048 to BAT-MAG-050) |
//! | 1396 + p, p < 8 | Shoot the walls at wall-shot target `p` (the Catapult, a Cyclops; BAT-SIE-011, BAT-SIE-013) |
//!
//! In the tactics phase `Move` entries place the active stack and Wait (0) passes to the next
//! stack (BAT-TAC-002).
//!
//! Version 2 (E03 T3) replaced the six neighbour directions of version 1 by the 24 canonical
//! attack positions, which cover two-hex attackers and targets; version 3 (E03 T4) added shots;
//! version 4 (E03 T8) grew the stack slots from 32 to 38 (`STACK_CAPACITY` includes war machines);
//! version 5 (E04a T7b) appended the heals, version 6 (E04a T8) `EndTactics`, version 7
//! (E04a T9a) Retreat and version 8 (E04a T9b) Surrender. Version 9 (E04b T2c) grew the stack
//! slots to 42 (21 per side, ADR-0014 point 6), appended the two cast entries and added the
//! spell and target heads. Version 10 (E04b T5a) uses the second target head (Teleport).
//! Version 11 (E04b T7b1) makes the stack's cast legal. Version 12 (the rear hex as a two-hex
//! stack's position, as in the original) names a two-hex stack's rear hex in the move entries and
//! the attack positions (whose slots keep their geometry); the shots keep the front hex at
//! `a = 0`. Version 13 (E04c T6) appends the wall shots.

use h3_battle::{
    ATTACK_SLOTS, BattleState, CastSpell, CastTarget, CastTargets, Command, Field, Hex,
    LegalCommands, STACK_CAPACITY, StackId, cast_targets, castable_spells, creature_cast_spell,
    creature_cast_targets, second_targets, stack_spell,
};
use h3_content::{MAX_SPELLS, SpellId};
use h3_content_battle::{BattleRuleset, MAX_FIELD_HEXES, MAX_WALL_TARGETS};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 13;

const MOVE_BASE: usize = 2;
const ATTACK_BASE: usize = MOVE_BASE + MAX_FIELD_HEXES;

const SHOOT_BASE: usize = ATTACK_BASE + ATTACK_SLOTS * STACK_CAPACITY;

const HEAL_BASE: usize = SHOOT_BASE + 2 * STACK_CAPACITY;

/// The entry that ends the tactics phase.
pub const END_TACTICS: usize = HEAL_BASE + STACK_CAPACITY;

/// The entry of a retreat.
pub const RETREAT: usize = END_TACTICS + 1;

/// The entry of a surrender.
pub const SURRENDER: usize = RETREAT + 1;

/// The entry of a hero's cast.
pub const HERO_CAST: usize = SURRENDER + 1;

/// The entry of the active stack's cast (BAT-MAG-048).
pub const CREATURE_CAST: usize = HERO_CAST + 1;

/// The first wall-shot entry (BAT-SIE-011, BAT-SIE-013).
pub const WALL_SHOT_BASE: usize = CREATURE_CAST + 1;

/// Number of entries.
pub const ACTION_COUNT: usize = WALL_SHOT_BASE + MAX_WALL_TARGETS;

/// [`HERO_CAST`] as an [`Action::entry`] value.
const HERO_CAST_ENTRY: u32 = HERO_CAST as u32;
/// [`CREATURE_CAST`] as an [`Action::entry`] value.
const CREATURE_CAST_ENTRY: u32 = CREATURE_CAST as u32;
const _: () = assert!(ACTION_COUNT <= u32::MAX as usize);

/// Size of the spell head: spells by handle (the content allows at most [`MAX_SPELLS`]).
pub const SPELL_COUNT: usize = MAX_SPELLS;

/// The target head's first stack slot (`0` is no target, `1 + h` hex `h`).
const TARGET_STACK_BASE: usize = 1 + MAX_FIELD_HEXES;

/// Size of the target heads: none, every hex, every stack slot.
pub const TARGET_COUNT: usize = TARGET_STACK_BASE + STACK_CAPACITY;

/// Legal-entry mask.
pub type ActionMask = [bool; ACTION_COUNT];

/// Spell-head mask.
pub type SpellMask = [bool; SPELL_COUNT];

/// Target-head mask (first and second target).
pub type TargetMask = [bool; TARGET_COUNT];

/// One action: an entry and the spell and target heads (all 0 for every entry but a cast).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Action {
    /// The entry (see the module documentation).
    pub entry: u32,
    /// A cast's spell handle.
    pub spell: u16,
    /// A cast's target.
    pub target: u16,
    /// A cast's second target (Teleport's destination, Sacrifice's victim; 0 for every other
    /// spell).
    pub second: u16,
}

impl Action {
    /// The action of a non-cast entry (heads 0); an entry beyond `u32` fails to decode.
    pub fn entry(entry: usize) -> Self {
        Self {
            entry: u32::try_from(entry).unwrap_or(u32::MAX),
            ..Self::default()
        }
    }
}

/// The target head's index of `target` (none for a hex beyond the field's limit, which would
/// otherwise take the first stack's index).
fn target_index(target: CastTarget) -> Option<u16> {
    let i = match target {
        CastTarget::None => 0,
        CastTarget::Hex(h) if h.index() < MAX_FIELD_HEXES => 1 + h.index(),
        CastTarget::Hex(_) => return None,
        CastTarget::Stack(s) => TARGET_STACK_BASE + s.index(),
    };
    (i < TARGET_COUNT).then(|| u16::try_from(i).ok()).flatten()
}

/// The target at target-head index `index`.
fn target_at(index: u16) -> Option<CastTarget> {
    match usize::from(index) {
        0 => Some(CastTarget::None),
        i if i < TARGET_STACK_BASE => u8::try_from(i - 1).ok().map(|h| CastTarget::Hex(Hex(h))),
        i if i < TARGET_COUNT => u8::try_from(i - TARGET_STACK_BASE)
            .ok()
            .map(|s| CastTarget::Stack(StackId(s))),
        _ => None,
    }
}

/// The canonical attack positions of the active stack against `target`.
fn positions(
    state: &BattleState,
    ruleset: &BattleRuleset,
    target: StackId,
) -> Option<[Option<Hex>; ATTACK_SLOTS]> {
    let field = Field::new(ruleset.battle().field);
    let attacker = state.stack(state.active()?)?;
    let victim = state.stack(target)?;
    Some(field.attack_positions(
        attacker.is_two_hex(),
        attacker.side,
        victim.footprint(field),
    ))
}

/// The dense entry of a command other than a cast.
fn encode_entry(state: &BattleState, ruleset: &BattleRuleset, command: Command) -> Option<usize> {
    match command {
        Command::Wait => Some(0),
        Command::Defend => Some(1),
        Command::Move { to } => (to.index() < MAX_FIELD_HEXES).then_some(MOVE_BASE + to.index()),
        Command::Attack { target, from } => {
            let k = positions(state, ruleset, target)?
                .iter()
                .position(|p| *p == Some(from))?;
            Some(ATTACK_BASE + ATTACK_SLOTS * target.index() + k)
        }
        Command::Shoot { target, at } => {
            let field = Field::new(ruleset.battle().field);
            let fp = state.stack(target)?.footprint(field);
            let aim = if at == fp.front {
                0
            } else if fp.rear == Some(at) {
                1
            } else {
                return None;
            };
            Some(SHOOT_BASE + 2 * target.index() + aim)
        }
        Command::Heal { target } => {
            (target.index() < STACK_CAPACITY).then_some(HEAL_BASE + target.index())
        }
        Command::EndTactics => Some(END_TACTICS),
        Command::Retreat => Some(RETREAT),
        Command::Surrender => Some(SURRENDER),
        Command::ShootWall { part } => {
            (usize::from(part) < MAX_WALL_TARGETS).then_some(WALL_SHOT_BASE + usize::from(part))
        }
        Command::Cast { .. } | Command::CreatureCast { .. } => None,
    }
}

/// The spell head of a stack's cast: the spell's handle, or 0 for a spell drawn at the cast (the
/// Master Genie, BAT-MAG-050; ADR-0014 point 9).
fn spell_head(spell: CastSpell) -> u16 {
    match spell {
        CastSpell::Named(id) => id.raw(),
        CastSpell::AtResolution | CastSpell::Raise => 0,
    }
}

/// The spell head of the active stack's cast: its creature's spell (BAT-MAG-048 to BAT-MAG-050).
fn active_spell(state: &BattleState, ruleset: &BattleRuleset) -> Option<u16> {
    state
        .active()
        .and_then(|id| stack_spell(state, ruleset, id))
        .map(spell_head)
}

/// Encodes a command. Attacks are encoded relative to the target's position and the active
/// stack's shape, so encoding needs the state; returns `None` for commands that cannot be
/// expressed (no active stack, unknown target, a position that is not an attack position, a
/// spell or target beyond the heads).
pub fn encode(state: &BattleState, ruleset: &BattleRuleset, command: Command) -> Option<Action> {
    if let Command::Cast {
        spell,
        target,
        second,
    } = command
    {
        return (spell.index() < SPELL_COUNT).then_some(Action {
            entry: HERO_CAST_ENTRY,
            spell: spell.raw(),
            target: target_index(target)?,
            second: target_index(second)?,
        });
    }
    if let Command::CreatureCast { target } = command {
        return Some(Action {
            entry: CREATURE_CAST_ENTRY,
            spell: active_spell(state, ruleset)?,
            target: target_index(target)?,
            second: 0,
        });
    }
    encode_entry(state, ruleset, command).map(Action::entry)
}

/// Decodes the dense entry `entry` into a command for the current state.
fn decode_entry(state: &BattleState, ruleset: &BattleRuleset, entry: usize) -> Option<Command> {
    match entry {
        0 => Some(Command::Wait),
        1 => Some(Command::Defend),
        a if a < ATTACK_BASE => u8::try_from(a - MOVE_BASE)
            .ok()
            .map(|h| Command::Move { to: Hex(h) }),
        a if a < SHOOT_BASE => {
            let target = StackId(u8::try_from((a - ATTACK_BASE) / ATTACK_SLOTS).ok()?);
            let from = positions(state, ruleset, target)?[(a - ATTACK_BASE) % ATTACK_SLOTS]?;
            Some(Command::Attack { target, from })
        }
        a if a < HEAL_BASE => {
            let target = StackId(u8::try_from((a - SHOOT_BASE) / 2).ok()?);
            let field = Field::new(ruleset.battle().field);
            let fp = state.stack(target)?.footprint(field);
            let at = if (a - SHOOT_BASE).is_multiple_of(2) {
                fp.front
            } else {
                fp.rear?
            };
            Some(Command::Shoot { target, at })
        }
        a if a < END_TACTICS => u8::try_from(a - HEAL_BASE)
            .ok()
            .map(|t| Command::Heal { target: StackId(t) }),
        END_TACTICS => Some(Command::EndTactics),
        RETREAT => Some(Command::Retreat),
        SURRENDER => Some(Command::Surrender),
        a if (WALL_SHOT_BASE..ACTION_COUNT).contains(&a) => u8::try_from(a - WALL_SHOT_BASE)
            .ok()
            .map(|part| Command::ShootWall { part }),
        _ => None,
    }
}

/// Decodes an action into a command for the current state (`None` if an index is out of range,
/// a non-cast entry has a non-zero head, or the slot holds no attack position). The result
/// still has to pass validation.
pub fn decode(state: &BattleState, ruleset: &BattleRuleset, action: Action) -> Option<Command> {
    let entry = action.entry as usize;
    if entry == HERO_CAST {
        if usize::from(action.spell) >= SPELL_COUNT {
            return None;
        }
        return Some(Command::Cast {
            spell: SpellId::from_raw(action.spell),
            target: target_at(action.target)?,
            second: target_at(action.second)?,
        });
    }
    if entry == CREATURE_CAST {
        let spell = active_spell(state, ruleset)?;
        return (action.spell == spell && action.second == 0)
            .then(|| target_at(action.target))
            .flatten()
            .map(|target| Command::CreatureCast { target });
    }
    if (action.spell, action.target, action.second) != (0, 0, 0) {
        return None;
    }
    decode_entry(state, ruleset, entry)
}

/// Fills `mask` with the legal entries of `actor` (all false if it is not the actor's turn):
/// the entries of [`h3_battle::legal_commands`], [`HERO_CAST`] if the hero may cast a spell
/// now ([`castable_spells`]) and [`CREATURE_CAST`] if the active stack may cast at a target
/// ([`creature_cast_targets`]).
pub fn legal_mask(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    legal: &mut LegalCommands,
    mask: &mut ActionMask,
) {
    mask.fill(false);
    h3_battle::legal_commands(state, ruleset, actor, legal);
    for c in legal.iter() {
        if let Some(i) = encode_entry(state, ruleset, *c) {
            mask[i] = true;
        }
    }
    mask[HERO_CAST] = !castable_spells(state, ruleset, actor).is_empty();
    mask[CREATURE_CAST] = creature_cast_possible(state, ruleset, actor);
}

/// Whether the active stack of `actor` may cast at some target now (BAT-MAG-048).
fn creature_cast_possible(state: &BattleState, ruleset: &BattleRuleset, actor: BattleSide) -> bool {
    // Most stacks may not cast: skip the targets' buffer for them.
    creature_cast_spell(state, ruleset, actor).is_some_and(|_| {
        let mut targets = CastTargets::new();
        creature_cast_targets(state, ruleset, actor, &mut targets);
        !targets.is_empty()
    })
}

/// Whether `entry` is a legal non-cast entry of `actor`.
fn legal_entry(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    entry: usize,
    legal: &mut LegalCommands,
) -> bool {
    h3_battle::legal_commands(state, ruleset, actor, legal);
    legal
        .iter()
        .any(|c| encode_entry(state, ruleset, *c) == Some(entry))
}

/// Fills `mask` with the legal spells of `actor` after choosing `entry`: the castable spells
/// for [`HERO_CAST`], the active stack's spell for a legal [`CREATURE_CAST`], only 0 for another
/// legal entry, nothing otherwise.
pub fn spell_mask(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    entry: usize,
    legal: &mut LegalCommands,
    mask: &mut SpellMask,
) {
    mask.fill(false);
    if entry == HERO_CAST {
        for spell in castable_spells(state, ruleset, actor).iter() {
            mask[spell.index()] = true;
        }
    } else if entry == CREATURE_CAST {
        if let Some(spell) = creature_cast_spell(state, ruleset, actor)
            .filter(|_| creature_cast_possible(state, ruleset, actor))
        {
            mask[usize::from(spell_head(spell))] = true;
        }
    } else if legal_entry(state, ruleset, actor, entry, legal) {
        mask[0] = true;
    }
}

/// Fills `mask` with the legal targets of `actor` after choosing `entry` and `spell`: the
/// targets of the cast for [`HERO_CAST`] ([`cast_targets`]), for [`CREATURE_CAST`] with the
/// active stack's spell those of its cast ([`creature_cast_targets`]), only 0 for another legal
/// entry with spell 0, nothing otherwise.
pub fn target_mask(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    (entry, spell): (usize, u16),
    legal: &mut LegalCommands,
    mask: &mut TargetMask,
) {
    mask.fill(false);
    if entry == HERO_CAST {
        let mut targets = CastTargets::new();
        if usize::from(spell) < SPELL_COUNT {
            cast_targets(
                state,
                ruleset,
                actor,
                SpellId::from_raw(spell),
                &mut targets,
            );
        }
        for t in targets.iter() {
            if let Some(i) = target_index(*t) {
                mask[usize::from(i)] = true;
            }
        }
    } else if entry == CREATURE_CAST {
        let mut targets = CastTargets::new();
        if creature_cast_spell(state, ruleset, actor).is_some_and(|s| spell_head(s) == spell) {
            creature_cast_targets(state, ruleset, actor, &mut targets);
        }
        for t in targets.iter() {
            if let Some(i) = target_index(*t) {
                mask[usize::from(i)] = true;
            }
        }
    } else if spell == 0 && legal_entry(state, ruleset, actor, entry, legal) {
        mask[0] = true;
    }
}

/// Fills `mask` with the legal second targets of `actor` after choosing `entry`, `spell` and
/// `target`: for [`HERO_CAST`] those of the cast at that target ([`second_targets`]: Teleport's
/// destinations, Sacrifice's victims, only 0 for other spells); only 0 for another legal prefix
/// ([`CREATURE_CAST`] included); nothing otherwise.
pub fn second_mask(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    (entry, spell, target): (usize, u16, u16),
    legal: &mut LegalCommands,
    mask: &mut TargetMask,
) {
    mask.fill(false);
    if entry == HERO_CAST {
        let mut seconds = CastTargets::new();
        if let (true, Some(first)) = (usize::from(spell) < SPELL_COUNT, target_at(target)) {
            second_targets(
                state,
                ruleset,
                actor,
                (SpellId::from_raw(spell), first),
                &mut seconds,
            );
        }
        for t in seconds.iter() {
            if let Some(i) = target_index(*t) {
                mask[usize::from(i)] = true;
            }
        }
    } else {
        let mut targets = [false; TARGET_COUNT];
        target_mask(state, ruleset, actor, (entry, spell), legal, &mut targets);
        mask[0] = targets.get(usize::from(target)).copied().unwrap_or(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_battle::NoopSink;

    fn env() -> (BattleRuleset, BattleState) {
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_SKIRMISH).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        (r, s)
    }

    /// The action of entry `i`.
    fn at(i: usize) -> Action {
        Action::entry(i)
    }

    /// Decodes an entry that does not depend on the state.
    fn decode_index(action: usize) -> Option<Command> {
        let (r, s) = env();
        let command = decode(&s, &r, at(action));
        assert_eq!(
            command.and_then(|c| encode(&s, &r, c)),
            command.map(|_| at(action))
        );
        command
    }

    /// The full scenario played until the attacker's hero may cast (Brea, whose book has the
    /// most spells, Blink's teleport among them).
    fn until_a_cast() -> (BattleRuleset, BattleState, BattleSide) {
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
        let mut s = BattleState::new(&r, &setup, 3).unwrap();
        let mut legal = LegalCommands::new();
        for _ in 0..200 {
            let side = s.side_to_act().unwrap();
            if side == BattleSide::Attacker && !castable_spells(&s, &r, side).is_empty() {
                return (r, s, side);
            }
            h3_battle::legal_commands(&s, &r, side, &mut legal);
            // End the tactics phase at once, then defend.
            let c = *legal
                .iter()
                .find(|c| matches!(c, Command::EndTactics | Command::Defend))
                .unwrap();
            h3_battle::apply(&mut s, &r, side, c, &mut NoopSink).unwrap();
        }
        panic!("no cast in 200 steps");
    }

    #[test]
    fn layout() {
        assert_eq!(ACTION_COUNT, 2 + 255 + 24 * 42 + 2 * 42 + 42 + 3 + 2 + 8);
        assert_eq!((END_TACTICS, RETREAT, SURRENDER), (1391, 1392, 1393));
        assert_eq!((HERO_CAST, CREATURE_CAST, WALL_SHOT_BASE), (1394, 1395, 1396));
        assert_eq!((SPELL_COUNT, TARGET_COUNT), (128, 298));
        assert_eq!(ACTION_ENCODING_VERSION, 13);
        assert_eq!(decode_index(SURRENDER), Some(Command::Surrender));
        assert_eq!(decode_index(CREATURE_CAST), None);
        // The wall shots (BAT-SIE-011): one entry per wall-shot target, none beyond them.
        assert_eq!(
            decode_index(WALL_SHOT_BASE + 7),
            Some(Command::ShootWall { part: 7 })
        );
        let (r, s) = env();
        assert_eq!(encode(&s, &r, Command::ShootWall { part: 8 }), None);
        assert_eq!(Action::entry(usize::MAX).entry, u32::MAX);
    }

    #[test]
    fn every_legal_command_round_trips() {
        let (r, s) = env();
        let side = s.side_to_act().unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, side, &mut legal, &mut mask);
        let creature = usize::from(creature_cast_possible(&s, &r, side));
        assert_eq!(mask.iter().filter(|m| **m).count(), legal.len() + creature);
        assert_eq!(mask[CREATURE_CAST], creature == 1);
        for c in legal.iter() {
            let a = encode(&s, &r, *c).unwrap();
            assert!(mask[a.entry as usize]);
            assert_eq!((a.spell, a.target, a.second), (0, 0, 0));
            assert_eq!(decode(&s, &r, a), Some(*c));
        }
        legal_mask(&s, &r, side.opponent(), &mut legal, &mut mask);
        assert!(mask.iter().all(|m| !m));
    }

    #[test]
    fn non_cast_entries_have_zero_heads() {
        let (r, s) = env();
        let side = s.side_to_act().unwrap();
        let mut legal = LegalCommands::new();
        let mut spells = [false; SPELL_COUNT];
        let mut targets = [false; TARGET_COUNT];
        // Defend is legal: each head allows exactly 0.
        spell_mask(&s, &r, side, 1, &mut legal, &mut spells);
        assert_eq!(spells.iter().position(|m| *m), Some(0));
        assert_eq!(spells.iter().filter(|m| **m).count(), 1);
        target_mask(&s, &r, side, (1, 0), &mut legal, &mut targets);
        assert_eq!(targets.iter().filter(|m| **m).count(), 1);
        second_mask(&s, &r, side, (1, 0, 0), &mut legal, &mut targets);
        assert!(targets[0]);
        // A head beyond 0, an illegal entry, the other side, a creature cast: nothing.
        target_mask(&s, &r, side, (1, 3), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
        second_mask(&s, &r, side, (1, 0, 5), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
        second_mask(&s, &r, side, (1, 0, u16::MAX), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
        spell_mask(&s, &r, side, END_TACTICS, &mut legal, &mut spells);
        assert!(spells.iter().all(|m| !m));
        spell_mask(&s, &r, side.opponent(), 1, &mut legal, &mut spells);
        assert!(spells.iter().all(|m| !m));
        spell_mask(
            &s,
            &r,
            side.opponent(),
            CREATURE_CAST,
            &mut legal,
            &mut spells,
        );
        assert!(spells.iter().all(|m| !m));
        target_mask(&s, &r, side, (CREATURE_CAST, 0), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
        // No hero, no cast.
        spell_mask(&s, &r, side, HERO_CAST, &mut legal, &mut spells);
        assert!(spells.iter().all(|m| !m));
        target_mask(&s, &r, side, (HERO_CAST, 0), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
        // Non-zero heads on a non-cast entry do not decode.
        let defend = Action { spell: 1, ..at(1) };
        assert_eq!(decode(&s, &r, defend), None);
        assert_eq!(decode(&s, &r, Action { second: 1, ..at(1) }), None);
    }

    #[test]
    fn casts_are_chosen_head_by_head() {
        let (r, s, side) = until_a_cast();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, side, &mut legal, &mut mask);
        let creature = creature_cast_possible(&s, &r, side);
        assert!(mask[HERO_CAST] && mask[CREATURE_CAST] == creature);
        assert_eq!(
            mask.iter().filter(|m| **m).count(),
            legal.len() + 1 + usize::from(creature)
        );
        let mut spells = [false; SPELL_COUNT];
        spell_mask(&s, &r, side, HERO_CAST, &mut legal, &mut spells);
        let castable = castable_spells(&s, &r, side);
        for (i, allowed) in spells.iter().enumerate() {
            let id = SpellId::from_raw(u16::try_from(i).unwrap());
            assert_eq!(*allowed, castable.contains(id));
        }
        let mut targets = [false; TARGET_COUNT];
        let mut seconds = [false; TARGET_COUNT];
        let mut casts = 0;
        // Every spell of the book and every target: the masks allow exactly what `apply`
        // accepts, and an allowed tuple round-trips through the encoding.
        let book: Vec<u16> = (0..r.spells().len())
            .map(|i| u16::try_from(i).unwrap())
            .collect();
        let mut teleports = 0;
        for &spell in &book {
            target_mask(&s, &r, side, (HERO_CAST, spell), &mut legal, &mut targets);
            assert_eq!(targets.iter().any(|m| *m), spells[usize::from(spell)]);
            let teleport = r
                .spell(SpellId::from_raw(spell))
                .is_some_and(|sp| sp.magic.teleports());
            // A sacrifice's second target is a stack (BAT-MAG-036).
            let several = teleport
                || r.spell(SpellId::from_raw(spell))
                    .is_some_and(|sp| sp.magic.sacrifices());
            for (t, &allowed) in targets.iter().enumerate() {
                let target = u16::try_from(t).unwrap();
                second_mask(
                    &s,
                    &r,
                    side,
                    (HERO_CAST, spell, target),
                    &mut legal,
                    &mut seconds,
                );
                assert_eq!(seconds.iter().any(|m| *m), allowed);
                // Every second target of a teleport or a sacrifice; else 0 and 1.
                let tried = if allowed && several { TARGET_COUNT } else { 2 };
                for (k, &ok) in seconds.iter().enumerate().take(tried) {
                    let action = Action {
                        entry: u32::try_from(HERO_CAST).unwrap(),
                        spell,
                        target,
                        second: u16::try_from(k).unwrap(),
                    };
                    let command = decode(&s, &r, action).unwrap();
                    let mut fork = s.clone();
                    let applied = h3_battle::apply(&mut fork, &r, side, command, &mut NoopSink);
                    assert_eq!(applied.is_ok(), ok, "{command:?}");
                    if ok {
                        assert_eq!(encode(&s, &r, command), Some(action));
                        casts += 1;
                        teleports += usize::from(teleport);
                    }
                }
                if allowed && !several {
                    assert_eq!(seconds.iter().position(|m| *m), Some(0));
                    assert_eq!(seconds.iter().filter(|m| **m).count(), 1);
                }
            }
        }
        assert!(casts > 0 && teleports > 0);
        second_mask(&s, &r, side, (HERO_CAST, 200, 0), &mut legal, &mut seconds);
        assert!(seconds.iter().all(|m| !m));
        second_mask(&s, &r, side, (HERO_CAST, 0, 400), &mut legal, &mut seconds);
        assert!(seconds.iter().all(|m| !m));
        // Out of range heads.
        let cast = |spell, target, second| Action {
            entry: u32::try_from(HERO_CAST).unwrap(),
            spell,
            target,
            second,
        };
        assert_eq!(decode(&s, &r, cast(0, 0, 298)), None);
        // A stack as the second target (Sacrifice's victim, BAT-MAG-036) decodes as one.
        assert!(matches!(
            decode(&s, &r, cast(0, 3, 256 + 4)),
            Some(Command::Cast {
                second: CastTarget::Stack(StackId(4)),
                ..
            })
        ));
        assert_eq!(decode(&s, &r, cast(128, 0, 0)), None);
        assert_eq!(decode(&s, &r, cast(0, 298, 0)), None);
        assert_eq!(
            decode(&s, &r, cast(0, 3, 0)),
            Some(Command::Cast {
                spell: SpellId::from_raw(0),
                target: CastTarget::Hex(Hex(2)),
                second: CastTarget::None,
            })
        );
        assert_eq!(
            encode(
                &s,
                &r,
                Command::Cast {
                    spell: SpellId::from_raw(128),
                    target: CastTarget::None,
                    second: CastTarget::None,
                }
            ),
            None
        );
        assert_eq!(
            encode(
                &s,
                &r,
                Command::Cast {
                    spell: SpellId::from_raw(0),
                    target: CastTarget::Stack(StackId(42)),
                    second: CastTarget::None,
                }
            ),
            None
        );
        target_mask(&s, &r, side, (HERO_CAST, 200), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
    }

    #[test]
    fn shots_round_trip_including_tail_aims() {
        use h3_contracts::{Army, ArmyStack, BattleSetup};
        let r = h3_pack_battle::compile(&[h3_pack_battle::BASE_PACK], "base:faithful").unwrap();
        let stack = |key: &str, count| Army {
            stacks: vec![ArmyStack {
                slot: 0,
                creature: r.creature_by_key(key).unwrap(),
                count,
            }],
        };
        let setup = BattleSetup {
            // Magogs (speed 6) act before Hydras (speed 5, two hexes) and shoot.
            attacker: stack("base:magog", 10),
            defender: stack("base:hydra", 2),
            obstacles: vec![],
            heroes: [None, None],
            terrain: None,
            battlefield: None,
            anti_magic: false,
            siege: None,
            war_machines: [vec![], vec![]],
            formations: Default::default(),
            funds: [0; 2],
        };
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let side = s.side_to_act().unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, side, &mut legal, &mut mask);
        let shots: Vec<Command> = legal
            .iter()
            .copied()
            .filter(|c| matches!(c, Command::Shoot { .. }))
            .collect();
        assert_eq!(shots.len(), 2, "front and rear hex of the two-hex hydra");
        for c in shots {
            let a = encode(&s, &r, c).unwrap();
            assert!(mask[a.entry as usize]);
            assert_eq!(decode(&s, &r, a), Some(c));
        }
        let off = Command::Shoot {
            target: StackId(1),
            at: Hex(0),
        };
        assert_eq!(encode(&s, &r, off), None);
        // A one-hex target has no rear-hex aim.
        assert_eq!(decode(&s, &r, at(SHOOT_BASE + 1)), None);
    }

    /// The skirmish played (defending) until the active stack may cast (a Wisp's Hasten).
    fn until_a_stack_casts() -> (BattleRuleset, BattleState, BattleSide) {
        let (r, mut s) = env();
        let mut legal = LegalCommands::new();
        for _ in 0..200 {
            let side = s.side_to_act().unwrap();
            if creature_cast_possible(&s, &r, side) {
                return (r, s, side);
            }
            h3_battle::legal_commands(&s, &r, side, &mut legal);
            let c = *legal.iter().find(|c| matches!(c, Command::Defend)).unwrap();
            h3_battle::apply(&mut s, &r, side, c, &mut NoopSink).unwrap();
        }
        panic!("no stack's cast in 200 steps");
    }

    #[test]
    fn a_stacks_cast_is_chosen_head_by_head() {
        let (r, s, side) = until_a_stack_casts();
        let hasten = r.spell_by_key("synthetic:hasten").unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, side, &mut legal, &mut mask);
        assert!(mask[CREATURE_CAST]);
        // The spell head: the stack's spell only.
        let mut spells = [false; SPELL_COUNT];
        spell_mask(&s, &r, side, CREATURE_CAST, &mut legal, &mut spells);
        let allowed: Vec<usize> = (0..SPELL_COUNT).filter(|&i| spells[i]).collect();
        assert_eq!(allowed, std::vec![hasten.index()]);
        spell_mask(
            &s,
            &r,
            side.opponent(),
            CREATURE_CAST,
            &mut legal,
            &mut spells,
        );
        assert!(spells.iter().all(|m| !m));
        // The target head: the stacks of its cast; none for another spell.
        let mut targets = [false; TARGET_COUNT];
        target_mask(
            &s,
            &r,
            side,
            (CREATURE_CAST, hasten.raw()),
            &mut legal,
            &mut targets,
        );
        let mut listed = CastTargets::new();
        creature_cast_targets(&s, &r, side, &mut listed);
        assert!(!listed.is_empty());
        for t in listed.iter() {
            assert!(targets[usize::from(target_index(*t).unwrap())]);
        }
        assert_eq!(targets.iter().filter(|m| **m).count(), listed.len());
        let other = hasten.raw() + 1;
        let mut none = [false; TARGET_COUNT];
        target_mask(&s, &r, side, (CREATURE_CAST, other), &mut legal, &mut none);
        assert!(none.iter().all(|m| !m));
        // The second target head: only 0.
        let target = target_index(listed.as_slice()[0]).unwrap();
        let mut seconds = [false; TARGET_COUNT];
        let prefix = (CREATURE_CAST, hasten.raw(), target);
        second_mask(&s, &r, side, prefix, &mut legal, &mut seconds);
        assert_eq!(seconds.iter().position(|m| *m), Some(0));
        assert_eq!(seconds.iter().filter(|m| **m).count(), 1);
        // Encoding and decoding: the spell head carries the stack's spell.
        let command = Command::CreatureCast {
            target: listed.as_slice()[0],
        };
        let action = encode(&s, &r, command).unwrap();
        assert_eq!(
            action,
            Action {
                entry: CREATURE_CAST_ENTRY,
                spell: hasten.raw(),
                target,
                second: 0,
            }
        );
        assert_eq!(decode(&s, &r, action), Some(command));
        assert_eq!(
            decode(
                &s,
                &r,
                Action {
                    spell: other,
                    ..action
                }
            ),
            None
        );
        assert_eq!(
            decode(
                &s,
                &r,
                Action {
                    second: 1,
                    ..action
                }
            ),
            None
        );
        assert_eq!(
            decode(
                &s,
                &r,
                Action {
                    target: u16::MAX,
                    ..action
                }
            ),
            None
        );
        let target_max = Command::CreatureCast {
            target: CastTarget::Hex(Hex(255)),
        };
        assert_eq!(encode(&s, &r, target_max), None);
        // A stack that is no caster: nothing to encode or decode.
        let mut plain = s.clone();
        while plain
            .active()
            .is_some_and(|id| stack_spell(&plain, &r, id).is_some())
        {
            let side = plain.side_to_act().unwrap();
            h3_battle::apply(&mut plain, &r, side, Command::Defend, &mut NoopSink).unwrap();
        }
        assert_eq!(encode(&plain, &r, command), None);
        assert_eq!(decode(&plain, &r, action), None);
    }

    #[test]
    fn a_draw_at_the_cast_names_no_spell() {
        // The skirmish's Wisps as helpful casters (BAT-MAG-050): their spell head allows only 0.
        let battle = h3_pack_battle::SYNTHETIC_BATTLE.replace(
            "{ kind = \"spell_caster\", spell = \"hasten\", mastery = 2, power = 6 },",
            "{ kind = \"helpful_spell_caster\", mastery = 2, power = 6 }, \
             { kind = \"drawn_spell\", spell = \"hasten\", weight = 1 },",
        );
        assert_ne!(battle, h3_pack_battle::SYNTHETIC_BATTLE);
        let scenario = h3_pack_battle::parse_scenario(h3_pack_battle::SYNTHETIC_SKIRMISH).unwrap();
        let pack = (h3_pack_battle::SYNTHETIC_PACK.0, battle.as_str());
        let r = h3_pack_battle::compile(&[pack], &scenario.profile).unwrap();
        let mut s = BattleState::new(&r, &scenario.resolve(&r).unwrap(), 1).unwrap();
        let mut legal = LegalCommands::new();
        let side = loop {
            let side = s.side_to_act().unwrap();
            if creature_cast_possible(&s, &r, side) {
                break side;
            }
            h3_battle::apply(&mut s, &r, side, Command::Defend, &mut NoopSink).unwrap();
        };
        let mut spells = [false; SPELL_COUNT];
        spell_mask(&s, &r, side, CREATURE_CAST, &mut legal, &mut spells);
        assert_eq!(spells.iter().position(|m| *m), Some(0));
        assert_eq!(spells.iter().filter(|m| **m).count(), 1);
        let mut listed = CastTargets::new();
        creature_cast_targets(&s, &r, side, &mut listed);
        let mut targets = [false; TARGET_COUNT];
        target_mask(&s, &r, side, (CREATURE_CAST, 0), &mut legal, &mut targets);
        assert_eq!(targets.iter().filter(|m| **m).count(), listed.len());
        let hasten = r.spell_by_key("synthetic:hasten").unwrap().raw();
        let mut none = [false; TARGET_COUNT];
        target_mask(&s, &r, side, (CREATURE_CAST, hasten), &mut legal, &mut none);
        assert!(none.iter().all(|m| !m));
        let command = Command::CreatureCast {
            target: listed.as_slice()[0],
        };
        let action = encode(&s, &r, command).unwrap();
        assert_eq!(action.spell, 0);
        assert_eq!(decode(&s, &r, action), Some(command));
        let named = Action {
            spell: hasten,
            ..action
        };
        assert_eq!(decode(&s, &r, named), None);
    }

    #[test]
    fn heals_of_the_commanded_tent_round_trip() {
        // The full scenario's attacker commands its tent (BAT-WAR-007): play until it may heal.
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
        let mut s = BattleState::new(&r, &setup, 3).unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        let mut heals = 0;
        for step in 0..2000 {
            let Some(side) = s.side_to_act() else { break };
            legal_mask(&s, &r, side, &mut legal, &mut mask);
            let cast = usize::from(mask[HERO_CAST]) + usize::from(mask[CREATURE_CAST]);
            assert_eq!(mask.iter().filter(|m| **m).count(), legal.len() + cast);
            for c in legal.iter().filter(|c| matches!(c, Command::Heal { .. })) {
                let a = encode(&s, &r, *c).unwrap();
                let i = a.entry as usize;
                assert!(i >= HEAL_BASE && mask[i]);
                assert_eq!(decode(&s, &r, a), Some(*c));
                heals += 1;
            }
            // Stay in the battle: both sides may retreat and surrender.
            let staying: Vec<Command> = legal
                .iter()
                .copied()
                .filter(|c| !c.leaves_battle())
                .collect();
            let command = staying[step % staying.len()];
            h3_battle::apply(&mut s, &r, side, command, &mut NoopSink).unwrap();
        }
        assert!(heals > 0);
        assert_eq!(
            encode(
                &s,
                &r,
                Command::Heal {
                    target: StackId(42)
                }
            ),
            None
        );
        assert_eq!(
            decode(&s, &r, at(END_TACTICS - 1)),
            Some(Command::Heal {
                target: StackId(41)
            })
        );
    }

    #[test]
    fn tactics_commands_round_trip() {
        // The full scenario's attacker has Advanced Tactics, the defender none (BAT-TAC-001).
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 3).unwrap();
        assert!(s.tactics().is_some());
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, BattleSide::Attacker, &mut legal, &mut mask);
        assert_eq!(mask.iter().filter(|m| **m).count(), legal.len());
        // Brea may retreat in her tactics phase (BAT-FLE-001), but not cast (BAT-MAG-001).
        assert!(mask[0] && mask[END_TACTICS] && mask[RETREAT] && !mask[1] && !mask[HERO_CAST]);
        for c in legal.iter() {
            let a = encode(&s, &r, *c).unwrap();
            assert_eq!(decode(&s, &r, a), Some(*c));
        }
        assert_eq!(decode(&s, &r, at(END_TACTICS)), Some(Command::EndTactics));
        assert_eq!(decode(&s, &r, at(RETREAT)), Some(Command::Retreat));
    }

    #[test]
    fn unrepresentable_and_out_of_range() {
        let (r, s) = env();
        assert_eq!(encode(&s, &r, Command::Move { to: Hex(255) }), None);
        assert_eq!(
            encode(
                &s,
                &r,
                Command::Attack {
                    target: StackId(44),
                    from: Hex(0)
                }
            ),
            None
        );
        let far = Command::Attack {
            target: StackId(3),
            from: Hex(0),
        };
        assert_eq!(encode(&s, &r, far), None);
        assert_eq!(decode(&s, &r, at(ACTION_COUNT)), None);
        // Stack 31 does not exist in a 6-stack battle.
        assert_eq!(decode(&s, &r, at(ATTACK_BASE + ATTACK_SLOTS * 31)), None);
        assert_eq!(
            decode(&s, &r, at(2 + 17)),
            Some(Command::Move { to: Hex(17) })
        );
        // A one-hex target has no rear-hex slots.
        let one_hex = s.stacks().iter().position(|t| !t.is_two_hex()).unwrap();
        assert_eq!(
            decode(&s, &r, at(ATTACK_BASE + ATTACK_SLOTS * one_hex + 12)),
            None
        );
        // Stack 0 stands in the first standable column: some of its attack positions touch the
        // side column, so they decode, and validation rejects them later.
        let side = s.side_to_act().unwrap();
        let rejected = (0..ATTACK_SLOTS)
            .filter_map(|k| decode(&s, &r, at(ATTACK_BASE + k)))
            .filter(|c| h3_battle::validate(&s, &r, side, *c).is_err())
            .count();
        assert!(rejected > 0);
    }
}
