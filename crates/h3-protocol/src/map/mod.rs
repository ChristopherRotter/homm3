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
    BattleResult, BattleSide, BattleState, CastTarget, CastTargets, HeroView, Hex, HexSet,
    LegalCommands, OwnHeroView, StackFlags, StackId, StackTraits, StrikeFlags,
};
use h3_content::{CreatureTypeId, Ruleset, SpellId};

pub use from_engine::{command, reject, target};

use crate::wire::{
    Ballista, BookSpell, Command, Effect, Event, Field, Hero, KeyTables, Observation, Outcome,
    OwnHero, Query, RemovableObstacle, Side, SideView, SpellObstacle, Stack, Strike, Tactics,
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

/// The battle as `viewer` sees it, on the wire.
pub fn observe(state: &BattleState, ruleset: &Ruleset, viewer: BattleSide) -> Observation {
    observation(&h3_battle::observe(state, ruleset, viewer))
}

/// The events `viewer` may see, in order, on the wire; the others are dropped.
pub fn events(events: &[h3_battle::Event], viewer: BattleSide) -> Vec<Event> {
    events
        .iter()
        .filter_map(|e| h3_battle::project(e, viewer))
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

/// The battlefield's size.
pub fn field(ruleset: &Ruleset) -> Field {
    let field = &ruleset.battle().field;
    Field {
        width: field.width,
        height: field.height,
    }
}

/// An engine observation on the wire.
pub(crate) fn observation(o: &h3_battle::Observation) -> Observation {
    let side_view = |s: BattleSide| {
        let i = s.index();
        SideView {
            may_retreat: o.may_retreat[i],
            may_surrender: o.surrender_possible[i],
            periodic_rounds: o.periodic_rounds[i],
            hero: o.heroes[i].map(hero),
        }
    };
    Observation {
        viewer: side(o.viewer),
        round: o.round,
        tactics: o.tactics.map(|t| Tactics {
            side: side(t.side),
            columns: t.columns,
        }),
        active: o.active.map(|a| a.0),
        last_side_acted: o.last_side_acted.map(side),
        result: o.result.map(outcome),
        drawn: o.drawn.map(SpellId::raw),
        terrain: o.terrain.map(h3_content::TerrainId::raw),
        sides: [
            side_view(BattleSide::Attacker),
            side_view(BattleSide::Defender),
        ],
        own_hero: o.own_hero.as_ref().map(own_hero),
        stacks: o
            .stacks
            .iter()
            .enumerate()
            .map(|(i, s)| stack(o, i, s))
            .collect(),
        obstacles: hexes(o.obstacles),
        obstacle_anchors: hexes(o.obstacle_anchors),
        removable_obstacles: o
            .removable_obstacles
            .iter()
            .map(|r| RemovableObstacle {
                anchor: r.anchor().0,
                hexes: r.hexes().iter().map(|h| h.0).collect(),
            })
            .collect(),
        spell_obstacles: o
            .spell_obstacles
            .iter()
            .map(|s| SpellObstacle {
                spell: s.spell.raw(),
                kind: from_engine::obstacle_kind(s.kind),
                side: side(s.side),
                hexes: hexes(s.hexes),
                rounds: s.rounds,
                value: wide(s.value),
                revealed: s.revealed,
            })
            .collect(),
    }
}

/// A hero's public part on the wire.
fn hero(h: HeroView) -> Hero {
    Hero {
        hero_type: h.hero_type.raw(),
        attack: h.attack,
        defense: h.defense,
        power: h.power,
        knowledge: h.knowledge,
        mana: h.mana,
        max_mana: h.max_mana,
        casts: h.casts,
        commands_ballista: h.commands_ballista,
        commands_tent: h.commands_tent,
        spell_book: h.spell_book,
    }
}

/// The viewer's own hero's private part on the wire.
fn own_hero(h: &OwnHeroView) -> OwnHero {
    OwnHero {
        tactics: h.tactics,
        ranged_bonus_bp: h.damage.ranged.bp,
        melee_bonus_bp: h.damage.melee.bp,
        damage_reduction_bp: h.damage.reduction.bp,
        ballista: h.ballista.map(|b| Ballista {
            shots: b.shots,
            double_damage_chance_bp: b.double_damage_chance_bp,
        }),
        tent_heal_bonus_bp: h.tent.map(|t| t.heal_bonus.bp),
        surrender_discount_bp: h.surrender_discount_bp,
        funds: h.funds,
        surrender_price: h.surrender_price,
        spell_book: h
            .spell_book
            .iter()
            .map(|spell| BookSpell {
                spell: spell.raw(),
                mastery: h.masteries.get(spell),
            })
            .collect(),
    }
}

/// Stack `i` of an observation on the wire: the values the sim's observation encoding exposes.
fn stack(o: &h3_battle::Observation, i: usize, s: &h3_battle::Stack) -> Stack {
    let spells = s.spells;
    Stack {
        id: u8::try_from(i).unwrap_or(u8::MAX),
        side: side(s.side),
        controller: side(s.controller()),
        slot: s.slot,
        creature: s.creature.raw(),
        hex: s.hex.0,
        alive: s.is_alive(),
        count: s.count,
        top_hp: s.top_hp,
        max_hp: s.max_hp,
        attack: s.attack_now(false) - i32::from(spells.melee_attack),
        defense: s.defense_now(),
        damage_min: s.damage_min,
        damage_max: s.damage_max,
        speed: s.speed_now(),
        morale: s.morale,
        luck: s.luck,
        shots: s.shots,
        retaliations_left: s.retaliations_left,
        regeneration: s.regeneration,
        acted: s.flags.contains(StackFlags::ACTED),
        waited: s.flags.contains(StackFlags::WAITED),
        retaliated: s.flags.contains(StackFlags::RETALIATED),
        defending: s.flags.contains(StackFlags::DEFENDING),
        had_morale: s.flags.contains(StackFlags::HAD_MORALE),
        two_hex: s.has(StackTraits::TWO_HEX),
        flies: s.has(StackTraits::FLIES),
        shooter: s.has(StackTraits::SHOOTER),
        war_machine: s.has(StackTraits::WAR_MACHINE),
        no_distance_penalty: s.has(StackTraits::NO_DISTANCE_PENALTY),
        shoots_when_adjacent: s.has(StackTraits::SHOOTS_WHEN_ADJACENT),
        created: s.is_created(),
        clone: s.has(StackTraits::CLONE),
        clone_rounds: o.clone_rounds[i],
        casts_left: o.casts_left[i],
        cast_candidates: o.cast_candidates[i],
        melee_attack_bonus: spells.melee_attack,
        ranged_attack_bonus: spells.ranged_attack,
        damage_roll: from_engine::damage_roll(spells.damage),
        speed_percent: spells.speed_percent,
        effects: o
            .effects
            .iter()
            .filter(|e| usize::from(e.stack.0) == i)
            .map(|e| Effect {
                spell: e.spell.raw(),
                rounds: e.rounds,
            })
            .collect(),
    }
}
