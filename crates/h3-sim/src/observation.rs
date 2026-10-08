//! Flat observation encoding (version 7) of a per-viewer [`Observation`].
//!
//! Layout: [`GLOBALS`] global features, then two hero blocks of [`HERO_FEATURES`] features (the
//! viewer's, then the opponent's), then [`STACK_FEATURES`] features for each of the
//! [`STACK_CAPACITY`] stack slots (zeros for unused slots). Values are raw game quantities as
//! `f32` (normalisation is the trainer's business). Stacks and sides are described relative to
//! the viewer ("own" flag, "viewer"/"opponent"), so one policy can play both sides.
//!
//! Globals: `[round (0 in the tactics phase), viewer is attacker, viewer to act (the viewer
//! controls the active stack: a hypnotized stack's controller is the other side, BAT-MAG-026;
//! its "own" flag below stays its owner's), battle over,
//! viewer places in a tactics phase, opponent places in a tactics phase, the phase's zone in
//! columns, viewer may retreat, opponent may retreat, viewer may surrender at all, opponent may
//! surrender at all]` (BAT-TAC-001, BAT-FLE-001, BAT-FLE-002).
//! Hero block, zeros for a side without a hero (which gets no lucky strikes in profiles with
//! quirk Q-0003): the [`HERO_PUBLIC_FEATURES`] public features `[led by a hero, hero type handle,
//! attack, defense, power, knowledge, mana, max mana, commands its Ballista, commands its First
//! Aid Tent, casts this round]`, then the private ones, zeros in the opponent's block (ADR-0013
//! point 9, ADR-0014 point 8): `[Tactics level, ranged bonus bp, melee bonus bp, damage
//! reduction bp, the commanded Ballista's shots per turn, its double-damage chance bp, the
//! commanded tent's heal bonus bp, surrender discount bp, funds, surrender price (0 if the side
//! may not surrender)]`, the mastery of each of the [`MAX_SPELL_SCHOOLS`] schools by handle,
//! and the spell book: one feature per spell handle below [`MAX_SPELLS`], 1 if the book has it.
//! Per stack: `[present, alive, own, creature handle, count, top_hp, max_hp, attack, defense,
//! damage_min, damage_max, speed, x, y, acted, waited, retaliated, defending, active, morale,
//! luck, shots, retaliations left (255 = unlimited), had good morale this round, two-hex,
//! flies, shooter, war machine, no distance penalty, shoots when adjacent, the hero's
//! regeneration in hit points, created in battle, clone, the clone's rounds left]` (no distance
//! penalty, shoots when adjacent and regeneration can come from the side's hero, BAT-HER-006;
//! created stacks and clones: BAT-MAG-037, BAT-MAG-038),
//! then its timed effects (BAT-MAG-009 to 011): `[melee attack bonus, ranged attack bonus,
//! blessed, cursed, the blessing's or curse's value, speed share in percent (100: not slowed),
//! number of effects]` and [`EFFECT_SLOTS`] pairs `[spell handle + 1 (0: empty), rounds left
//! (−1: for the rest of the battle)]`, the stack's first effects by spell handle. Attack,
//! defense and speed are the values now, with the effects (attack without the melee or ranged
//! bonus; attack and defense may be below 0).
//!
//! v3 (E03 T9): the two hero globals and the per-stack features from `morale` on. v4 (E04a
//! T11b): the tactics, retreat and surrender globals, the hero blocks (instead of the two hero
//! globals) and the last three per-stack features. v5 (E04b T2c): the hero's casts this round,
//! school masteries and spell book; 42 stack slots. v6 (E04b T3a): attack, defense and speed
//! with the timed effects, and the per-stack effect features. v7 (E04b T6b): created in battle,
//! clone and the clone's rounds left.

use h3_battle::{
    DamageRoll, HeroView, Observation, OwnHeroView, STACK_CAPACITY, StackFlags, StackTraits,
};
use h3_content::{MAX_SPELL_SCHOOLS, MAX_SPELLS};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above (and bump the C ABI
/// version with it, ADR-0011).
pub const OBSERVATION_ENCODING_VERSION: u32 = 7;

/// Global features.
pub const GLOBALS: usize = 11;
/// The public features at the start of a hero block; the rest are private to the hero's side.
pub const HERO_PUBLIC_FEATURES: usize = 11;
/// The private features before the school masteries.
const HERO_PRIVATE_VALUES: usize = 10;
/// Offset of the school masteries in a hero block.
const HERO_SCHOOLS: usize = HERO_PUBLIC_FEATURES + HERO_PRIVATE_VALUES;
/// Offset of the spell book in a hero block.
const HERO_BOOK: usize = HERO_SCHOOLS + MAX_SPELL_SCHOOLS;
/// Features per hero block.
pub const HERO_FEATURES: usize = HERO_BOOK + MAX_SPELLS;
/// Effects per stack the encoding shows.
pub const EFFECT_SLOTS: usize = 8;
/// The per-stack features before the effect features.
const STACK_VALUES: usize = 34;
/// The effect values before the effect slots.
const EFFECT_VALUES: usize = 7;
/// Features per stack slot.
pub const STACK_FEATURES: usize = STACK_VALUES + EFFECT_VALUES + 2 * EFFECT_SLOTS;
/// Offset of the viewer's hero block; the opponent's follows it.
pub const HEROES: usize = GLOBALS;
/// Offset of the first stack slot.
pub const STACKS: usize = HEROES + 2 * HERO_FEATURES;
/// Length of the encoded observation.
pub const OBSERVATION_LEN: usize = STACKS + STACK_FEATURES * STACK_CAPACITY;

/// Hex columns of the encoded field, used to derive `x`/`y` from the hex index.
fn coords(hex: u8, width: u8) -> (f32, f32) {
    let w = width.max(1);
    (f32::from(hex % w), f32::from(hex / w))
}

fn flag(on: bool) -> f32 {
    if on { 1.0 } else { 0.0 }
}

/// Writes a hero block into `out`: the public part `view` and, for the viewer's own hero only,
/// the private part `own`.
fn encode_hero(view: Option<HeroView>, own: Option<OwnHeroView>, out: &mut [f32]) {
    if let Some(h) = view {
        out[..HERO_PUBLIC_FEATURES].copy_from_slice(&[
            1.0,
            f32::from(h.hero_type.raw()),
            f32::from(h.attack),
            f32::from(h.defense),
            f32::from(h.power),
            f32::from(h.knowledge),
            f32::from(h.mana),
            f32::from(h.max_mana),
            flag(h.commands_ballista),
            flag(h.commands_tent),
            f32::from(h.casts),
        ]);
    }
    if let Some(h) = own {
        out[HERO_PUBLIC_FEATURES..HERO_SCHOOLS].copy_from_slice(&[
            f32::from(h.tactics),
            h.damage.ranged.bp as f32,
            h.damage.melee.bp as f32,
            h.damage.reduction.bp as f32,
            h.ballista.map_or(0.0, |b| f32::from(b.shots)),
            h.ballista
                .map_or(0.0, |b| f32::from(b.double_damage_chance_bp)),
            h.tent.map_or(0.0, |t| t.heal_bonus.bp as f32),
            f32::from(h.surrender_discount_bp),
            h.funds as f32,
            h.surrender_price.map_or(0.0, |p| p as f32),
        ]);
        for (o, level) in out[HERO_SCHOOLS..HERO_BOOK].iter_mut().zip(h.schools) {
            *o = f32::from(level);
        }
        for spell in h.spell_book.iter() {
            out[HERO_BOOK + spell.index()] = 1.0;
        }
    }
}

/// Encodes `obs` into `out`. Returns `Err(required_len)` if `out` is too short. `field_width`
/// (from the ruleset) turns hex indices into coordinates.
pub fn encode(obs: &Observation, field_width: u8, out: &mut [f32]) -> Result<(), usize> {
    let out = out.get_mut(..OBSERVATION_LEN).ok_or(OBSERVATION_LEN)?;
    out.fill(0.0);
    // The active stack's controller acts (a hypnotized stack's is the other side, BAT-MAG-026).
    let to_act = obs
        .active
        .and_then(|id| obs.stacks.get(id.index()))
        .map(|s| s.controller());
    let (own, other) = (obs.viewer.index(), obs.viewer.opponent().index());
    let phase = obs.tactics;
    out[..GLOBALS].copy_from_slice(&[
        f32::from(obs.round),
        flag(obs.viewer == BattleSide::Attacker),
        flag(to_act == Some(obs.viewer)),
        flag(obs.result.is_some()),
        flag(phase.is_some_and(|p| p.side == obs.viewer)),
        flag(phase.is_some_and(|p| p.side != obs.viewer)),
        phase.map_or(0.0, |p| f32::from(p.columns)),
        flag(obs.may_retreat[own]),
        flag(obs.may_retreat[other]),
        flag(obs.surrender_possible[own]),
        flag(obs.surrender_possible[other]),
    ]);
    let (viewer, opponent) = out[HEROES..STACKS].split_at_mut(HERO_FEATURES);
    encode_hero(obs.heroes[own], obs.own_hero, viewer);
    encode_hero(obs.heroes[other], None, opponent);
    for (i, s) in obs.stacks.iter().enumerate() {
        let f = &mut out[STACKS + i * STACK_FEATURES..STACKS + (i + 1) * STACK_FEATURES];
        let (x, y) = coords(s.hex.0, field_width);
        let values = [
            1.0,
            flag(s.is_alive()),
            flag(s.side == obs.viewer),
            f32::from(s.creature.raw()),
            s.count as f32,
            s.top_hp as f32,
            s.max_hp as f32,
            (s.attack_now(false) - i32::from(s.spells.melee_attack)) as f32,
            s.defense_now() as f32,
            f32::from(s.damage_min),
            f32::from(s.damage_max),
            f32::from(s.speed_now()),
            x,
            y,
            flag(s.flags.contains(StackFlags::ACTED)),
            flag(s.flags.contains(StackFlags::WAITED)),
            flag(s.flags.contains(StackFlags::RETALIATED)),
            flag(s.flags.contains(StackFlags::DEFENDING)),
            flag(obs.active.is_some_and(|a| a.index() == i)),
            f32::from(s.morale),
            f32::from(s.luck),
            f32::from(s.shots),
            f32::from(s.retaliations_left),
            flag(s.flags.contains(StackFlags::HAD_MORALE)),
            flag(s.has(StackTraits::TWO_HEX)),
            flag(s.has(StackTraits::FLIES)),
            flag(s.has(StackTraits::SHOOTER)),
            flag(s.has(StackTraits::WAR_MACHINE)),
            flag(s.has(StackTraits::NO_DISTANCE_PENALTY)),
            flag(s.has(StackTraits::SHOOTS_WHEN_ADJACENT)),
            f32::from(s.regeneration),
            flag(s.is_created()),
            flag(s.has(StackTraits::CLONE)),
            f32::from(obs.clone_rounds[i]),
        ];
        f[..STACK_VALUES].copy_from_slice(&values);
        let sp = s.spells;
        let (blessed, cursed, change) = match sp.damage {
            DamageRoll::Random => (0.0, 0.0, 0),
            DamageRoll::Maximum(v) => (1.0, 0.0, v),
            DamageRoll::Minimum(v) => (0.0, 1.0, v),
        };
        let effects = obs.effects.iter().filter(|e| e.stack.index() == i);
        f[STACK_VALUES..STACK_VALUES + EFFECT_VALUES].copy_from_slice(&[
            f32::from(sp.melee_attack),
            f32::from(sp.ranged_attack),
            blessed,
            cursed,
            f32::from(change),
            f32::from(sp.speed_percent),
            effects.clone().count() as f32,
        ]);
        let slots = f[STACK_VALUES + EFFECT_VALUES..].chunks_exact_mut(2);
        for (slot, e) in slots.zip(effects) {
            slot.copy_from_slice(&[
                f32::from(e.spell.raw()) + 1.0,
                e.rounds.map_or(-1.0, f32::from),
            ]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_battle::{BattleState, observe};

    /// The features of stack `i` in an encoded observation.
    fn stack(out: &[f32], i: usize) -> &[f32] {
        &out[STACKS + i * STACK_FEATURES..STACKS + (i + 1) * STACK_FEATURES]
    }

    /// The viewer's (0) or the opponent's (1) hero block.
    fn hero(out: &[f32], block: usize) -> &[f32] {
        &out[HEROES + block * HERO_FEATURES..HEROES + (block + 1) * HERO_FEATURES]
    }

    #[test]
    fn encodes_globals_and_stacks() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let obs = observe(&s, &r, BattleSide::Defender);
        let mut out = vec![9.0; OBSERVATION_LEN + 3];
        encode(&obs, r.battle().field.width, &mut out).unwrap();
        // Round 1, the defender's view, the attacker to act; no tactics phase, no heroes, so no
        // retreat or surrender.
        assert_eq!(
            out[..GLOBALS],
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        assert!(out[HEROES..STACKS].iter().all(|v| *v == 0.0));
        // Stack 0: the attacker's two-hex brute ×8, head at (2, 5) (tail in column 1), active.
        let s0 = stack(&out, 0);
        assert_eq!(s0[0..3], [1.0, 1.0, 0.0]);
        assert_eq!(s0[4], 8.0);
        assert_eq!((s0[12], s0[13]), (2.0, 5.0));
        assert_eq!(s0[18], 1.0);
        // Morale +1 (one-faction army, BAT-MOR-002), luck 0, no shots, one retaliation, two-hex
        // walker, nothing from a hero.
        assert_eq!(
            s0[19..31],
            [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        // No timed effects: no bonuses, not slowed, no entries.
        // Not created in battle, no clone (BAT-MAG-037, BAT-MAG-038).
        assert_eq!(s0[31..34], [0.0, 0.0, 0.0]);
        assert_eq!(s0[34..41], [0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 0.0]);
        assert!(s0[41..].iter().all(|v| *v == 0.0));
        // Unused slots are zero; the tail beyond OBSERVATION_LEN is untouched.
        assert!(
            out[STACKS + 2 * STACK_FEATURES..OBSERVATION_LEN]
                .iter()
                .all(|v| *v == 0.0)
        );
        assert_eq!(out[OBSERVATION_LEN], 9.0);
        let mut short = vec![0.0; 3];
        assert_eq!(encode(&obs, 17, &mut short), Err(OBSERVATION_LEN));
        assert_eq!(OBSERVATION_LEN, 11 + 2 * (11 + 10 + 4 + 128) + 57 * 42);
        assert_eq!(coords(20, 0), (0.0, 20.0));
        // A clone with three rounds left (BAT-MAG-038).
        let mut obs = obs;
        let clone = &mut obs.stacks.as_mut_slice()[1];
        clone.traits = clone
            .traits
            .with(StackTraits::SUMMONED, true)
            .with(StackTraits::CLONE, true);
        obs.clone_rounds[1] = 3;
        encode(&obs, r.battle().field.width, &mut out).unwrap();
        assert_eq!(stack(&out, 1)[31..34], [1.0, 1.0, 3.0]);
    }

    #[test]
    fn encodes_the_heroes_and_shows_the_private_part_only_to_its_side() {
        use BattleSide::{Attacker, Defender};
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut out = vec![0.0; OBSERVATION_LEN];
        // Both sides are led by a hero (`full.toml`): Brea's Tactics give her a tactics phase
        // with a five-column zone, and both sides may retreat and surrender.
        let handle = |key| f32::from(r.hero_type_by_key(key).unwrap().raw());
        let brea = [
            1.0,
            handle("synthetic:brea"),
            2.0,
            2.0,
            2.0,
            1.0,
            40.0,
            40.0,
            1.0,
            1.0,
            0.0,
        ];
        let corwin = [
            1.0,
            handle("synthetic:corwin"),
            1.0,
            3.0,
            3.0,
            1.0,
            40.0,
            40.0,
            1.0,
            0.0,
            0.0,
        ];
        // Masteries by school handle (storm, stone) and the books by spell handle (spark,
        // cinders, arc, blight, ward, farsight, glare, hasten, drag, fury, zeal, gloom, boon,
        // bane, corrode, aegis, hallow, smiting, riposte, purge, mend, blaze, rage, veil, glass,
        // haze, dazzle, rampage, charm, bulwark, scorch, blink, mire, snare, uproot).
        let schools = [[1.0, 0.0, 0.0, 0.0], [0.0, 3.0, 0.0, 0.0]];
        let books: [&[usize]; 2] = [
            &[
                0, 1, 2, 3, 4, 7, 8, 9, 17, 18, 20, 21, 24, 25, 28, 30, 31, 33, 35, 36,
            ],
            &[
                0, 2, 3, 10, 11, 12, 13, 14, 15, 16, 19, 22, 23, 26, 27, 29, 32, 34, 35,
            ],
        ];
        let price = |side| s.surrender_price(&r, side).unwrap() as f32;
        assert!(price(Attacker) > 0.0 && price(Defender) > 0.0);
        let brea_private = [
            2.0,
            1000.0,
            1000.0,
            500.0,
            1.0,
            5000.0,
            10_000.0,
            0.0,
            10_000.0,
            price(Attacker),
        ];
        let corwin_private = [
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            5000.0,
            0.0,
            1000.0,
            10_000.0,
            price(Defender),
        ];
        for (viewer, globals, own, other, private, schools, book) in [
            (
                Attacker,
                [0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 5.0, 1.0, 1.0, 1.0, 1.0],
                brea,
                corwin,
                brea_private,
                schools[0],
                books[0],
            ),
            (
                Defender,
                [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0, 1.0, 1.0, 1.0, 1.0],
                corwin,
                brea,
                corwin_private,
                schools[1],
                books[1],
            ),
        ] {
            encode(&observe(&s, &r, viewer), 17, &mut out).unwrap();
            assert_eq!(out[..GLOBALS], globals);
            assert_eq!(hero(&out, 0)[..HERO_PUBLIC_FEATURES], own);
            assert_eq!(hero(&out, 0)[HERO_PUBLIC_FEATURES..HERO_SCHOOLS], private);
            assert_eq!(hero(&out, 0)[HERO_SCHOOLS..HERO_BOOK], schools);
            let known: Vec<usize> = (0..MAX_SPELLS)
                .filter(|i| hero(&out, 0)[HERO_BOOK + i] == 1.0)
                .collect();
            assert_eq!(known, book);
            assert_eq!(hero(&out, 1)[..HERO_PUBLIC_FEATURES], other);
            // The opponent's private part never shows.
            assert!(
                hero(&out, 1)[HERO_PUBLIC_FEATURES..]
                    .iter()
                    .all(|v| *v == 0.0)
            );
        }
        // Without an enemy hero: no surrender, a zero block; without a Ballista, zero control.
        let mut obs = observe(&s, &r, Attacker);
        obs.heroes[1] = None;
        obs.may_retreat[1] = false;
        obs.surrender_possible = [false, false];
        let own = obs.own_hero.as_mut().unwrap();
        own.surrender_price = None;
        own.ballista = None;
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(out[7..GLOBALS], [1.0, 0.0, 0.0, 0.0]);
        assert!(hero(&out, 1).iter().all(|v| *v == 0.0));
        assert_eq!(hero(&out, 0)[15..17], [0.0, 0.0]);
        assert_eq!(hero(&out, 0)[HERO_SCHOOLS - 1], 0.0);
    }

    #[test]
    fn encodes_morale_luck_shots_war_machines_and_traits_from_heroes() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&observe(&s, &r, BattleSide::Attacker), 17, &mut out).unwrap();
        // Ids: attacker army 0..=6, its bolt thrower 7 and aid tent 8; defender army 9..=15,
        // its bolt thrower 16 and supply wagon 17.
        let kinds: Vec<[f32; 4]> = (0..18)
            .map(|i| {
                let f = stack(&out, i);
                [f[24], f[25], f[26], f[27]]
            })
            .collect();
        assert_eq!(kinds[2], [0.0, 1.0, 1.0, 0.0], "wisp: flies and shoots");
        assert_eq!(kinds[7], [1.0, 0.0, 1.0, 1.0], "bolt thrower");
        assert_eq!(kinds[8], [1.0, 0.0, 0.0, 1.0], "aid tent");
        assert_eq!(kinds[17], [0.0, 0.0, 0.0, 1.0], "supply wagon");
        assert_eq!(stack(&out, 7)[21], 12.0, "the bolt thrower's shots");
        // Morale, luck, the good-morale flag and the hero's traits come from the stacks.
        let mut obs = observe(&s, &r, BattleSide::Attacker);
        let st = &mut obs.stacks.as_mut_slice()[0];
        st.morale = 2;
        st.luck = -1;
        st.flags.set(StackFlags::HAD_MORALE, true);
        st.traits = st
            .traits
            .with(StackTraits::NO_DISTANCE_PENALTY, true)
            .with(StackTraits::SHOOTS_WHEN_ADJACENT, true);
        st.regeneration = 50;
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(stack(&out, 0)[19..24], [2.0, -1.0, 0.0, 1.0, 1.0]);
        assert_eq!(stack(&out, 0)[28..31], [1.0, 1.0, 50.0]);
    }

    #[test]
    fn encodes_the_timed_effects_of_stacks() {
        use h3_battle::{ActiveEffect, StackId};
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut obs = observe(&s, &r, BattleSide::Attacker);
        let spell = |k: &str| r.spell_by_key(k).unwrap();
        let entry = |stack, spell, rounds| ActiveEffect {
            stack: StackId(stack),
            spell,
            rounds,
            value: 1,
            mastery: 0,
            attack: 0,
            defense: 0,
            speed: 0,
        };
        obs.effects = std::vec![
            entry(0, spell("synthetic:hasten"), Some(3)),
            entry(0, spell("synthetic:corrode"), None),
        ];
        let st = &mut obs.stacks.as_mut_slice()[0];
        st.spells.attack = -2;
        st.spells.defense = -40;
        st.spells.speed = 1;
        st.spells.melee_attack = 3;
        st.spells.ranged_attack = 4;
        st.spells.damage = DamageRoll::Minimum(2);
        st.spells.speed_percent = 50;
        let (attack, defense) = (f32::from(st.attack) - 2.0, f32::from(st.defense) - 40.0);
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&obs, 17, &mut out).unwrap();
        let f = stack(&out, 0);
        assert_eq!([f[7], f[8], f[11]], [attack, defense, 1.0]);
        assert_eq!(f[34..41], [3.0, 4.0, 0.0, 1.0, 2.0, 50.0, 2.0]);
        let hasten = f32::from(spell("synthetic:hasten").raw()) + 1.0;
        let corrode = f32::from(spell("synthetic:corrode").raw()) + 1.0;
        assert_eq!(f[41..45], [hasten, 3.0, corrode, -1.0]);
        assert!(f[45..].iter().all(|v| *v == 0.0));
        obs.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Maximum(1);
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(stack(&out, 0)[36..39], [1.0, 0.0, 1.0]);
        assert!(stack(&out, 1)[41..].iter().all(|v| *v == 0.0));
    }
}
