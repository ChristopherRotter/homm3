//! The mapping between the wire and the engine (ADR-0018 point 5): the only code that knows
//! both. What leaves for a viewer is projected here by construction: [`observe`] and [`events`]
//! take the viewer and call the engine's `observe` and `project`, so no caller can send an
//! unprojected observation or event (ADR-0018 point 7).
//!
//! The matches over the engine's enums are in `from_engine.rs` (exhaustive: a new engine
//! variant fails the build until it is mapped, ADR-0018 point 5 as amended); everything else is
//! here.

pub(crate) mod from_engine;

use alloc::string::String;
use alloc::vec::Vec;

use h3_battle::{
    ActiveEffect, BATTLE_STATE_HASH_VERSION, BattleResult, BattleSide, BattleState, CastTarget,
    CastTargets, HeroView, Hex, HexSet, LegalCommands, Opening, OwnHeroView, SpellBonuses,
    SpellSet, SpellValues, StackFlags, StackId, StackTraits, StrikeFlags, TacticsPhase, Viewer,
};
use h3_content::{CreatureTypeId, Ruleset, SpellId};
use h3_contracts::{BallistaControl, DamageBonuses, Rate, TentControl};
use h3_kernel::hash::{HASH_ALGORITHM, HASH_VERSION};
use h3_kernel::rng::{RNG_ALGORITHM, RNG_VERSION};

pub use from_engine::{command, reject, target};

use crate::PROTOCOL_VERSION;
use crate::wire::{
    Algorithm, Ballista, BookSpell, Command, Defences, Effect, Event, Field, Hero, KeyTables,
    Manifest, Observation, Offer, OpeningSpell, Outcome, OwnHero, PackRef, Query,
    RemovableObstacle, Seat, Side, SideView, SpellBonus, SpellObstacle, Stack, Strike, Tactics,
    Target, WireU64,
};

/// A battle side on the wire.
pub fn side(side: BattleSide) -> Side {
    match side {
        BattleSide::Attacker => Side::Attacker,
        BattleSide::Defender => Side::Defender,
    }
}

/// A wire side in the engine.
pub fn side_to_engine(side: Side) -> BattleSide {
    match side {
        Side::Attacker => BattleSide::Attacker,
        Side::Defender => BattleSide::Defender,
    }
}

/// A battle's result on the wire.
pub fn outcome(result: BattleResult) -> Outcome {
    match result {
        BattleResult::Victory(s) => Outcome::Victory { side: side(s) },
        BattleResult::Draw => Outcome::Draw {},
        BattleResult::Retreat(s) => Outcome::Retreat { side: side(s) },
        BattleResult::Surrender(s) => Outcome::Surrender { side: side(s) },
    }
}

/// A strike's flags on the wire.
pub fn strike(flags: StrikeFlags) -> Strike {
    Strike {
        retaliation: flags.contains(StrikeFlags::RETALIATION),
        ranged: flags.contains(StrikeFlags::RANGED),
        secondary: flags.contains(StrikeFlags::SECONDARY),
        death_blow: flags.contains(StrikeFlags::DEATH_BLOW),
        lucky: flags.contains(StrikeFlags::LUCKY),
        double_damage: flags.contains(StrikeFlags::DOUBLE_DAMAGE),
    }
}

/// A set of hexes as their indices, ascending.
pub fn hexes(set: HexSet) -> Vec<u8> {
    set.iter().map(|h| h.0).collect()
}

/// A 64-bit value on the wire.
pub(crate) const fn wide(value: u64) -> WireU64 {
    WireU64(value)
}

/// A wire target in the engine.
pub fn target_to_engine(target: Target) -> CastTarget {
    match target {
        Target::None {} => CastTarget::None,
        Target::Stack { stack } => CastTarget::Stack(StackId(stack)),
        Target::Hex { hex } => CastTarget::Hex(Hex(hex)),
    }
}

/// A client's command in the engine. The engine validates the values against the state and the
/// seat (ADR-0008).
pub fn command_to_engine(command: Command) -> h3_battle::Command {
    use h3_battle::Command as C;
    match command {
        Command::Move { to } => C::Move { to: Hex(to) },
        Command::Attack { target, from } => C::Attack {
            target: StackId(target),
            from: Hex(from),
        },
        Command::Shoot { target, at } => C::Shoot {
            target: StackId(target),
            at: Hex(at),
        },
        Command::Wait {} => C::Wait,
        Command::Defend {} => C::Defend,
        Command::Heal { target } => C::Heal {
            target: StackId(target),
        },
        Command::EndTactics {} => C::EndTactics,
        Command::Retreat {} => C::Retreat,
        Command::Surrender {} => C::Surrender,
        Command::Cast {
            spell,
            target,
            second,
        } => C::Cast {
            spell: SpellId::from_raw(spell),
            target: target_to_engine(target),
            second: target_to_engine(second),
        },
        Command::CreatureCast { target } => C::CreatureCast {
            target: target_to_engine(target),
        },
    }
}

/// The battle as `viewer`, a side or a spectator (E16 M2 T4), sees it, on the wire.
pub fn observe(state: &BattleState, ruleset: &Ruleset, viewer: impl Into<Viewer>) -> Observation {
    observation(&h3_battle::observe_as(state, ruleset, viewer.into()))
}

/// The events `viewer`, a side or a spectator, may see, in order, on the wire; the others are
/// dropped.
pub fn events(events: &[h3_battle::Event], viewer: impl Into<Viewer>) -> Vec<Event> {
    let viewer = viewer.into();
    events
        .iter()
        .filter_map(|e| h3_battle::project_as(e, viewer))
        .map(|e| from_engine::event(&e.kind))
        .collect()
}

/// The legal commands of `actor` now (casts are listed by [`castable`]).
pub fn legal_commands(state: &BattleState, ruleset: &Ruleset, actor: BattleSide) -> Vec<Command> {
    let mut legal = LegalCommands::new();
    h3_battle::legal_commands(state, ruleset, actor, &mut legal);
    legal.iter().map(|&c| command(c)).collect()
}

/// The handles of the spells `actor`'s hero may cast now.
pub fn castable(state: &BattleState, ruleset: &Ruleset, actor: BattleSide) -> Vec<u16> {
    h3_battle::castable_spells(state, ruleset, actor)
        .iter()
        .map(SpellId::raw)
        .collect()
}

/// The answer to `query` for `actor`: the targets the server's rules allow.
pub fn query(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    query: Query,
) -> Vec<Target> {
    let mut out = CastTargets::new();
    match query {
        Query::CastTargets { spell } => {
            h3_battle::cast_targets(state, ruleset, actor, SpellId::from_raw(spell), &mut out);
        }
        Query::SecondTargets { spell, target } => h3_battle::second_targets(
            state,
            ruleset,
            actor,
            (SpellId::from_raw(spell), target_to_engine(target)),
            &mut out,
        ),
    }
    out.iter().map(|&t| target(t)).collect()
}

/// The ruleset's content keys by handle (the manifest's tables, ADR-0018 point 10).
pub fn key_tables(ruleset: &Ruleset) -> KeyTables {
    let texts = |keys: &mut dyn Iterator<Item = &h3_content::ContentKey>| {
        keys.map(|k| String::from(k.as_str())).collect()
    };
    KeyTables {
        creatures: (0..ruleset.creature_count())
            .map(|i| {
                CreatureTypeId::from_index(i)
                    .and_then(|id| ruleset.creature_key(id))
                    .map_or_else(String::new, |k| String::from(k.as_str()))
            })
            .collect(),
        spells: texts(&mut ruleset.spells().iter().map(|s| &s.key)),
        skills: texts(&mut ruleset.skills().iter().map(|s| &s.key)),
        hero_types: texts(&mut ruleset.hero_types().iter().map(|h| &h.key)),
        artifacts: texts(&mut ruleset.artifacts().iter().map(|a| &a.key)),
        factions: texts(&mut ruleset.factions().iter().map(|f| &f.key)),
        terrains: texts(&mut ruleset.terrains().iter().map(|t| &t.key)),
    }
}

/// What a game on `ruleset` is made of (ADR-0018 point 10): the replay header's fields
/// (`h3_sim::ReplayHeader`), the field's size and the key tables.
pub fn manifest(ruleset: &Ruleset) -> Manifest {
    Manifest {
        protocol: PROTOCOL_VERSION,
        engine: String::from(env!("CARGO_PKG_VERSION")),
        packs: ruleset
            .packs()
            .iter()
            .map(|(id, version)| PackRef {
                id: id.clone(),
                version: version.clone(),
            })
            .collect(),
        profile: String::from(ruleset.profile().as_str()),
        ruleset_hash: wide(ruleset.content_hash()),
        rng: Algorithm {
            name: String::from(RNG_ALGORITHM),
            version: RNG_VERSION,
        },
        hash: Algorithm {
            name: String::from(HASH_ALGORITHM),
            version: HASH_VERSION,
        },
        battle_state_hash_version: BATTLE_STATE_HASH_VERSION,
        field: field(ruleset),
        keys: key_tables(ruleset),
    }
}

/// An offer of the server (E16 M2 T1): the game configuration `id`, the scenario `scenario` on
/// `ruleset`, with the server's bot in `bots`; what the manifest names, before anyone joins.
pub fn offer(id: &str, scenario: &str, ruleset: &Ruleset, bots: Vec<Seat>) -> Offer {
    let Manifest {
        packs,
        profile,
        ruleset_hash,
        ..
    } = manifest(ruleset);
    Offer {
        id: String::from(id),
        scenario: String::from(scenario),
        packs,
        profile,
        ruleset_hash,
        bots,
    }
}

/// The battlefield's size.
pub fn field(ruleset: &Ruleset) -> Field {
    let field = &ruleset.battle().field;
    Field {
        width: field.width,
        height: field.height,
    }
}

/// An engine observation on the wire. Every field of the engine's observation, and of the parts
/// it is made of, is named in a pattern here: a field the engine adds fails the build until it is
/// mapped or left out on purpose with its reason (E16 M2 T4b), as a variant does (ADR-0018 point
/// 5 as amended). Only the removable obstacles are read through their accessors (their fields
/// are private).
pub(crate) fn observation(o: &h3_battle::Observation) -> Observation {
    let h3_battle::Observation {
        viewer,
        spectator,
        round,
        tactics,
        may_retreat,
        surrender_possible,
        wait_phase,
        active,
        last_side_acted,
        result,
        stacks,
        clone_rounds,
        clone_links,
        summoned,
        temporary,
        corpses,
        binders,
        poison,
        start_hp,
        casts_left,
        drawn,
        cast_candidates,
        periodic_rounds,
        obstacles,
        removable_obstacles,
        obstacle_anchors,
        obstacles_cleared,
        heroes,
        cost_change,
        own_hero: own,
        terrain,
        effects,
        spell_obstacles,
    } = o;
    let side_view = |s: BattleSide| {
        let i = s.index();
        SideView {
            may_retreat: may_retreat[i],
            may_surrender: surrender_possible[i],
            periodic_rounds: periodic_rounds[i],
            summoned: summoned[i].map(CreatureTypeId::raw),
            cost_change: cost_change[i],
            hero: heroes[i].map(hero),
        }
    };
    let per_stack = |i: usize| PerStack {
        clone_rounds: clone_rounds[i],
        clone_link: clone_links[i],
        temporary: temporary[i],
        corpse: corpses[i],
        binders: binders[i],
        poison: poison[i],
        start_hp: start_hp[i],
        casts_left: casts_left[i],
        cast_candidates: cast_candidates[i],
    };
    Observation {
        viewer: side(*viewer),
        spectator: *spectator,
        round: *round,
        tactics: tactics.map(|t| {
            let TacticsPhase { side: s, columns } = t;
            Tactics {
                side: side(s),
                columns,
            }
        }),
        wait_phase: *wait_phase,
        active: active.map(|a| a.0),
        last_side_acted: last_side_acted.map(side),
        result: result.map(outcome),
        drawn: drawn.map(SpellId::raw),
        terrain: terrain.map(h3_content::TerrainId::raw),
        sides: [
            side_view(BattleSide::Attacker),
            side_view(BattleSide::Defender),
        ],
        own_hero: own.as_ref().map(own_hero),
        stacks: stacks
            .iter()
            .enumerate()
            .map(|(i, s)| stack(i, s, per_stack(i), effects))
            .collect(),
        obstacles: hexes(*obstacles),
        obstacle_anchors: hexes(*obstacle_anchors),
        removable_obstacles: removable_obstacles
            .iter()
            .map(|r| RemovableObstacle {
                anchor: r.anchor().0,
                hexes: r.hexes().iter().map(|h| h.0).collect(),
            })
            .collect(),
        obstacles_cleared: hexes(*obstacles_cleared),
        spell_obstacles: spell_obstacles.iter().map(spell_obstacle).collect(),
    }
}

/// A hero's public part on the wire.
fn hero(h: HeroView) -> Hero {
    let HeroView {
        hero_type,
        attack,
        defense,
        power,
        knowledge,
        mana,
        max_mana,
        casts,
        commands_ballista,
        commands_tent,
        spell_book,
    } = h;
    Hero {
        hero_type: hero_type.raw(),
        attack,
        defense,
        power,
        knowledge,
        mana,
        max_mana,
        casts,
        commands_ballista,
        commands_tent,
        spell_book,
    }
}

/// A rate on the wire: its basis points. The original's single-precision value is left out on
/// purpose: the rules compute with it (quirk Q-0005), a client shows the rate.
const fn bp(rate: Rate) -> u32 {
    let Rate { bp, single: _ } = rate;
    bp
}

/// A set of spells on the wire: their handles, ascending.
fn spells(set: SpellSet) -> Vec<u16> {
    set.iter().map(SpellId::raw).collect()
}

/// The viewer's own hero's private part on the wire, its hidden magic among it (BAT-Q26).
fn own_hero(h: &OwnHeroView) -> OwnHero {
    let OwnHeroView {
        tactics,
        damage,
        ballista,
        tent,
        surrender_discount_bp,
        funds,
        surrender_price,
        spell_book,
        masteries,
        spell_bonuses,
        magic_resistance_bp,
        duration_bonus,
        no_casting,
        level_limit,
        defences,
        negates,
        opening,
        opening_due,
        eagle_chance,
        eagle_spells,
        learned,
    } = h;
    let DamageBonuses {
        ranged,
        melee,
        reduction,
    } = *damage;
    let SpellBonuses {
        all,
        school,
        spells: single,
    } = spell_bonuses;
    let h3_battle::Defences {
        spells: immune,
        up_to_level,
        mind,
    } = *defences;
    OwnHero {
        tactics: *tactics,
        ranged_bonus_bp: bp(ranged),
        melee_bonus_bp: bp(melee),
        damage_reduction_bp: bp(reduction),
        ballista: ballista.map(|b| {
            let BallistaControl {
                shots,
                double_damage_chance_bp,
            } = b;
            Ballista {
                shots,
                double_damage_chance_bp,
            }
        }),
        tent_heal_bonus_bp: tent.map(|t| {
            let TentControl { heal_bonus } = t;
            bp(heal_bonus)
        }),
        surrender_discount_bp: *surrender_discount_bp,
        funds: *funds,
        surrender_price: *surrender_price,
        spell_book: spell_book
            .iter()
            .map(|spell| BookSpell {
                spell: spell.raw(),
                mastery: masteries.get(spell),
            })
            .collect(),
        spell_damage_bp: bp(*all),
        school_damage_bp: school.iter().copied().map(bp).collect(),
        spell_bonuses: single.iter().map(spell_bonus).collect(),
        magic_resistance_bp: *magic_resistance_bp,
        duration_bonus: *duration_bonus,
        no_casting: *no_casting,
        level_limit: *level_limit,
        defences: Defences {
            spells: spells(immune),
            up_to_level,
            mind,
        },
        negates: *negates,
        opening: opening.iter().map(opening_spell).collect(),
        opening_due: *opening_due,
        eagle_chance: *eagle_chance,
        eagle_spells: spells(*eagle_spells),
        learned: spells(*learned),
    }
}

/// A single spell's damage bonus on the wire.
fn spell_bonus(b: &h3_battle::SpellBonus) -> SpellBonus {
    let h3_battle::SpellBonus {
        spell,
        damage_bp,
        per_level_bp,
        level,
    } = *b;
    SpellBonus {
        spell: spell.raw(),
        damage_bp,
        per_level_bp,
        level,
    }
}

/// An opening spell on the wire.
fn opening_spell(o: &Opening) -> OpeningSpell {
    let Opening {
        spell,
        mastery,
        power,
    } = *o;
    OpeningSpell {
        spell: spell.raw(),
        mastery,
        power,
    }
}

/// A spell obstacle on the wire.
fn spell_obstacle(o: &h3_battle::SpellObstacle) -> SpellObstacle {
    let h3_battle::SpellObstacle {
        spell,
        kind,
        side: owner,
        hexes: set,
        rounds,
        value,
        revealed,
    } = *o;
    SpellObstacle {
        spell: spell.raw(),
        kind: from_engine::obstacle_kind(kind),
        side: side(owner),
        hexes: hexes(set),
        rounds,
        value: wide(value),
        revealed,
    }
}

/// A timed effect on the wire.
fn effect(e: &ActiveEffect) -> Effect {
    let ActiveEffect {
        // The stack it is on: listed with it.
        stack: _,
        spell,
        rounds,
        value,
        mastery,
        attack,
        defense,
        speed,
    } = *e;
    Effect {
        spell: spell.raw(),
        rounds,
        mastery,
        value,
        attack,
        defense,
        speed,
    }
}

/// What the engine's observation keeps of one stack beside the stack, by stack id.
#[derive(Clone, Copy)]
struct PerStack {
    clone_rounds: u16,
    clone_link: Option<StackId>,
    temporary: u32,
    corpse: u16,
    binders: u64,
    poison: u8,
    start_hp: u32,
    casts_left: u8,
    cast_candidates: u8,
}

/// Stack `id` of an observation on the wire, with what the observation keeps of it beside it and
/// the timed effects on it.
fn stack(id: usize, s: &h3_battle::Stack, per: PerStack, effects: &[ActiveEffect]) -> Stack {
    let h3_battle::Stack {
        creature,
        side: owner,
        slot,
        hex,
        count,
        base_count,
        top_hp,
        // Left out: the values now are on the wire, the effects' shares in `effects`.
        attack: _,
        defense: _,
        damage_min,
        damage_max,
        max_hp,
        speed: _,
        shots,
        retaliations,
        retaliations_left,
        morale,
        luck,
        traits,
        regeneration,
        flags,
        spells,
    } = *s;
    let SpellValues {
        // In the values now.
        attack: _,
        defense: _,
        speed: _,
        melee_attack,
        ranged_attack,
        damage,
        speed_percent,
        // Left out: materialised from the effects (BAT-MAG-011), which the wire carries with
        // their spells, masteries and values.
        morale: _,
        luck: _,
        melee_protection: _,
        ranged_protection: _,
        school_protection: _,
        slayer: _,
        fire_shield: _,
        frenzy: _,
        spell_immunity: _,
        mirror: _,
        status: _,
    } = spells;
    let PerStack {
        clone_rounds,
        clone_link,
        temporary,
        corpse,
        binders,
        poison,
        start_hp,
        casts_left,
        cast_candidates,
    } = per;
    Stack {
        id: u8::try_from(id).unwrap_or(u8::MAX),
        side: side(owner),
        controller: side(s.controller()),
        slot,
        creature: creature.raw(),
        hex: hex.0,
        alive: s.is_alive(),
        count,
        base_count,
        top_hp,
        max_hp,
        attack: s.attack_now(false) - i32::from(melee_attack),
        defense: s.defense_now(),
        damage_min,
        damage_max,
        speed: s.speed_now(),
        morale,
        luck,
        shots,
        retaliations,
        retaliations_left,
        regeneration,
        acted: flags.contains(StackFlags::ACTED),
        waited: flags.contains(StackFlags::WAITED),
        retaliated: flags.contains(StackFlags::RETALIATED),
        defending: flags.contains(StackFlags::DEFENDING),
        had_morale: flags.contains(StackFlags::HAD_MORALE),
        started: flags.contains(StackFlags::STARTED),
        two_hex: traits.contains(StackTraits::TWO_HEX),
        flies: traits.contains(StackTraits::FLIES),
        shooter: traits.contains(StackTraits::SHOOTER),
        war_machine: traits.contains(StackTraits::WAR_MACHINE),
        no_distance_penalty: traits.contains(StackTraits::NO_DISTANCE_PENALTY),
        shoots_when_adjacent: traits.contains(StackTraits::SHOOTS_WHEN_ADJACENT),
        native: traits.contains(StackTraits::NATIVE),
        created: traits.contains(StackTraits::SUMMONED),
        clone: traits.contains(StackTraits::CLONE),
        raised: traits.contains(StackTraits::RAISED),
        clone_rounds,
        clone_link: clone_link.map(|l| l.0),
        temporary,
        corpse,
        bound_by: (0u8..64).filter(|&b| binders & (1 << b) != 0).collect(),
        poison,
        start_hp,
        casts_left,
        cast_candidates,
        melee_attack_bonus: melee_attack,
        ranged_attack_bonus: ranged_attack,
        damage_roll: from_engine::damage_roll(damage),
        speed_percent,
        effects: effects
            .iter()
            .filter(|e| usize::from(e.stack.0) == id)
            .map(effect)
            .collect(),
    }
}
