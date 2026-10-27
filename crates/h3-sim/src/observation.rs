//! Flat observation encoding (version 22) of a per-viewer [`Observation`].
//!
//! Layout, in this order: [`GLOBALS`] global features; the public part of the two heroes,
//! [`HERO_PUBLIC_FEATURES`] each (the viewer's, then the opponent's); the private part of the
//! viewer's own hero, [`OWN_HERO_FEATURES`]; the battlefield, [`HEX_PLANES`] planes of
//! [`MAX_FIELD_HEXES`] features (one per hex index, as the action encoding's hex targets; plane
//! after plane); then [`STACK_FEATURES`] features for each of the [`STACK_CAPACITY`] stack slots
//! (zeros for unused slots); then the turn order, [`ORDER_FEATURES`] features for each stack slot
//! (v22, after the rest so that v21's layout is its prefix). Values are raw game quantities as `f32` (normalisation is the
//! trainer's business). Stacks, sides and obstacles are described relative to the viewer ("own"
//! flag, "viewer"/"opponent"), so one policy can play both sides; a spectator's observation
//! (SRV-Q4) is seen from the attacker's seat and has no private part. The encoding is complete
//! (E04b T9a): it holds everything of the [`Observation`] a rule reads, directly or through the
//! ruleset's data.
//!
//! Globals: `[round (0 in the tactics phase), viewer is attacker, viewer to act (the viewer
//! controls the active stack: a hypnotized stack's controller is the other side, BAT-MAG-026;
//! its "own" flag below stays its owner's), battle over,
//! viewer places in a tactics phase, opponent places in a tactics phase, the phase's zone in
//! columns, viewer may retreat, opponent may retreat (at a town the defender's option shows to
//! the defender only: 0 in the attacker's and a spectator's observation, BAT-SIE-024), viewer may
//! surrender at all, opponent may surrender at all, the spell the active stack drew as handle + 1
//! if the viewer controls it (0:
//! none or hidden), the round starts since the viewer's periodic casters cast, the round starts
//! since the opponent's (both at most 255, which every battle starts with), the round's wait
//! phase has begun, the side the speed tie-break counts as activated last (the last turn's,
//! the defender after the attacker's Catapult; 1 viewer, 2 opponent, 0 none yet), the
//! terrain's handle + 1 (0 none), the creature the viewer's Summon Elemental casts are bound to
//! as handle + 1 (0 none yet), the opponent's, a spectator's observation, the special
//! battlefield's handle + 1 (0 none), the battle is at an anti-magic garrison, the town the
//! battle is fought at as its faction's handle + 1 (0: not at a town), its fort level (0 none, 1
//! Fort, 2 Citadel, 3 Castle), its moat (0 absent, 1 harmless, 2 damaging), the gate is open
//! (or destroyed), the gate is destroyed, the game's difficulty (0 Easy to 4 Impossible), the
//! viewer's player is human, the opponent's, then the hit points of each of the
//! [`MAX_WALL_TARGETS`] wall-shot targets (0: destroyed, not one; −1: not set up by the fort
//! level)]` (BAT-TAC-001, BAT-FLE-001, BAT-FLE-002, BAT-MAG-049, BAT-MAG-053, BAT-MAG-039,
//! BAT-TER-002, BAT-MAG-061, BAT-SIE-001, BAT-SIE-003, BAT-SIE-010, BAT-SIE-012, BAT-SIE-023).
//! Public hero part, zeros for a side without a hero (which gets no lucky strikes in profiles
//! with quirk Q-0003): `[led by a hero, hero type handle, attack, defense, power, knowledge,
//! mana, max mana, commands its Ballista, commands its First Aid Tent, casts this round,
//! carries a spell book, the change of its spell costs from the armies, commands its Catapult]`
//! (BAT-MAG-054, BAT-SIE-011).
//! Private part of the viewer's hero (ADR-0013 point 9, ADR-0014 point 8, BAT-Q26), zeros
//! without one: [`OWN_HERO_VALUES`] values `[Tactics level, ranged bonus bp, melee bonus bp,
//! damage reduction bp, the commanded Ballista's shots per turn, its double-damage chance bp,
//! the commanded tent's heal bonus bp, surrender discount bp, funds, surrender price (0 if the
//! side may not surrender), the spell-damage bonus of every spell bp, the bonus of each spell
//! school bp (by school handle), its stacks' magic resistance bp, rounds added to its timed
//! spells, it forbids every cast, the highest spell level it allows + 1 (0: no limit), the
//! highest spell level its stacks are immune to (0: none), its stacks are immune to mind spells,
//! it negates immunities, its opening spells are still due, its Eagle Eye's chance in percent,
//! the commanded Catapult's Ballistics level (0: none), the commanded Ballista's Artillery level
//! (0: none)]` (BAT-MAG-004, BAT-MAG-008, HER-BAT-014, BAT-MAG-061 to BAT-MAG-066, BAT-MAG-068,
//! BAT-MAG-069, BAT-SIE-011, BAT-SIE-021),
//! [`MAX_SPELL_BONUSES`] spell bonuses `[spell handle + 1, damage bp, growth per level bp, the
//! level]`, [`MAX_OPENING_SPELLS`] opening spells `[spell handle + 1, mastery, power]`; then
//! four vectors of one feature per spell handle below [`MAX_SPELLS`]: in its spell book, its
//! mastery (0 none to 3 Expert, of every spell: Sacrifice reads Resurrection's under
//! BAT-QUIRK-5), its stacks are immune to it, its Eagle Eye may pick it (1) or picked it (2).
//! Battlefield planes: `[blocked by a natural obstacle, an anchor hex that belongs to its
//! obstacle (BAT-QUIRK-3), the natural obstacle Remove Obstacle finds here as index + 1
//! (BAT-MAG-034)]`, then for the viewer's spell obstacles and for the opponent's it sees
//! (BAT-MAG-031; a side has at most one on a hex) `[Force Field, Fire Wall, Quicksand, Land
//! Mine]` as the rounds left (−1: for the rest of the battle; 0: none) and their value (Fire
//! Wall's or a mine's damage value, BAT-MAG-028, BAT-MAG-033), then the viewer's spell obstacle
//! here is seen by the opponent; then the town's `[standing wall hex (the gate's included until
//! it is destroyed), moat hex now (the bridges only while the gate is closed)]` (BAT-SIE-002,
//! BAT-SIE-004, BAT-SIE-012; the town's mines are the defender's Land Mines above, BAT-SIE-007).
//! Per stack: `[present, alive, own, creature handle, count, top_hp, max_hp, attack, defense,
//! damage_min, damage_max, speed, x, y (its position: a two-hex stack's rear hex), acted,
//! waited, retaliated, defending, active, morale, luck, shots, retaliations left (255 =
//! unlimited), had good morale this round, two-hex,
//! flies, shooter, war machine, no distance penalty, shoots when adjacent, the hero's
//! regeneration in hit points, created in battle, clone, the clone's rounds left, casts left, the
//! spells the active stack's cast could draw for it, creatures at the start (the resurrection
//! limit), its turn has started this round, raised from a corpse, native, creatures raised for
//! this battle only, its corpse (0 none yet, −1 none left, else the order of its death), the
//! stack linked to it as id + 1 (a clone's original, its living clone, itself for an original
//! that became its own clone under BAT-QUIRK-7; 0 none), its binders as
//! bits of stack ids 0–20 and 21–41, the tenths Poison took, its hit points per creature at the
//! start if Poison or Age changed them (0 otherwise), a town's turret (off the field: its `x`,
//! `y` are the original's position 251, 254 or 255 as a hex index)]` (no distance penalty, shoots when
//! adjacent and regeneration can come from the side's hero, BAT-HER-006; created stacks and
//! clones: BAT-MAG-037, BAT-MAG-038; casts: BAT-MAG-048; the draw: BAT-MAG-050, for the viewer
//! that commands a helpful caster, else 0; BAT-DMG-008, BAT-TRN-001, BAT-MAG-052, BAT-TER-001,
//! BAT-MAG-035, BAT-MAG-036, BAT-MAG-045, BAT-MAG-044),
//! then its timed effects (BAT-MAG-009 to 026): `[melee attack bonus, ranged attack bonus,
//! blessed, cursed, the blessing's or curse's value, speed share in percent (100: not slowed),
//! number of effects, melee damage let through in percent, ranged damage let through, spell
//! damage let through per school handle, Slayer's attack bonus, Slayer's mastery, Fire Shield's
//! share in percent, Frenzy's share of the defense in percent, Anti-Magic's level, Magic
//! Mirror's chance in percent, forgetful, cannot shoot, loses its turns, an effect ends by
//! damage, berserk, hypnotized, an effect ends by its attack, petrified]` and
//! [`EFFECT_SLOTS`] triples `[spell handle + 1 (0: empty), rounds left (−1: for the rest of the
//! battle), the mastery]`, the stack's first effects by spell handle. Attack, defense, speed,
//! morale and luck are the values now, with the effects (attack without the melee or ranged
//! bonus; attack and defense may be below 0).
//! The turn order per stack slot (`h3_battle::turn_order`, BAT-TRN-001 to BAT-TRN-003: the turns
//! as they come, no wait, morale or death foreseen; the same for every viewer): `[its place in
//! the rest of the round's turns (1: the active stack; 0: none), its place among its side's turns
//! in the rest of the round (1: its side's next), its place in the next round's turns (0:
//! none)]`; in the tactics phase the round is the first.
//!
//! v3 (E03 T9): the two hero globals and the per-stack features from `morale` on. v4 (E04a
//! T11b): the tactics, retreat and surrender globals, the hero blocks (instead of the two hero
//! globals) and the last three per-stack features. v5 (E04b T2c): the hero's casts this round,
//! school masteries and spell book; 42 stack slots. v6 (E04b T3a): attack, defense and speed
//! with the timed effects, and the per-stack effect features. v7 (E04b T6b): created in battle,
//! clone and the clone's rounds left. v8 (E04b T7b1): casts left. v9 (E04b T7b2a): the drawn
//! spell global. v10 (E04b T7b2b): the hero's spell book and the stack's draw candidates. v11
//! (E04b T7b4): the two periodic-cast globals. v12 (E04b T8a): the book's features carry each
//! spell's mastery, the school masteries are gone (masteries are derived per spell, BAT-Q19).
//! v13 (E04b T9a): complete; the battlefield planes; the private part once, with the spell
//! bonuses and the hidden magic; the globals from the wait phase on, the cost change, the
//! per-stack features from the start count on, the materialised effects and each effect's
//! mastery. v14: a two-hex stack's `x`, `y` are its rear hex, its position as in the original
//! (BAT-GEO-002), no longer its front hex. v15 (E04c T2b): the special battlefield and the
//! anti-magic garrison as the last two globals. v16 (E04c T5): the town's four globals and its
//! two planes. v17 (E04c T6): the gate's destruction and the wall-shot targets' hit points as
//! globals, the heroes' command of their Catapults, the own hero's Ballistics level; the wall
//! plane shows the standing walls. v18 (E04c T7): the turret flag as the last per-stack value
//! (BAT-SIE-016). v19 (E04c T7d): the game's difficulty and the sides' human players as globals
//! before the wall-shot targets (ADR-0023 point 7). v20 (E04c T7e): the own hero's Artillery
//! level as its last scalar (ADR-0013 point 4). v21 (E04c T10b): the layout unchanged; at a town
//! the opponent-may-retreat global of the attacker's and a spectator's observation is 0, as the
//! defender's Escape Tunnel shows to the defender only (BAT-Q50 (4)). v22 (E06, the turn-order
//! probe): the turn order block after the stack slots, v21's layout unchanged before it.

use h3_battle::{
    DamageRoll, HeroView, MAX_OPENING_SPELLS, MAX_SPELL_BONUSES, Observation, OwnHeroView,
    STACK_CAPACITY, SpellObstacle, StackFlags, StackTraits, Status, turn_order,
};
use h3_content::{MAX_SPELL_SCHOOLS, MAX_SPELLS};
use h3_content_battle::{MAX_FIELD_HEXES, MAX_WALL_TARGETS, ObstacleKind};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above (and bump the C ABI
/// version with it, ADR-0011).
pub const OBSERVATION_ENCODING_VERSION: u32 = 22;

/// Global features.
pub const GLOBALS: usize = 30 + MAX_WALL_TARGETS;
/// The public features of a hero.
pub const HERO_PUBLIC_FEATURES: usize = 14;
/// The values of the viewer's hero's private part before its spell bonuses.
const OWN_HERO_SCALARS: usize = 11 + MAX_SPELL_SCHOOLS + 11;
/// The values of the viewer's hero's private part before its per-spell vectors.
pub const OWN_HERO_VALUES: usize =
    OWN_HERO_SCALARS + 4 * MAX_SPELL_BONUSES + 3 * MAX_OPENING_SPELLS;
/// Features of the viewer's hero's private part.
pub const OWN_HERO_FEATURES: usize = OWN_HERO_VALUES + 4 * MAX_SPELLS;
/// Planes of the battlefield.
pub const HEX_PLANES: usize = SIEGE_PLANES + 2;
/// The plane of the viewer's spell obstacles the opponent sees.
const REVEALED_PLANE: usize = 3 + 2 * OWNER_PLANES;
/// The first of the town's planes.
const SIEGE_PLANES: usize = REVEALED_PLANE + 1;
/// The planes of one side's spell obstacles: one per kind, then the value.
const OWNER_PLANES: usize = ObstacleKind::COUNT + 1;
/// Effects per stack the encoding shows.
pub const EFFECT_SLOTS: usize = 8;
/// The per-stack features before the effect features.
const STACK_VALUES: usize = 48;
/// The effect values before the effect slots.
const EFFECT_VALUES: usize = 9 + MAX_SPELL_SCHOOLS + 6 + STATUSES.len();
/// Features per effect slot.
const SLOT_FEATURES: usize = 3;
/// Features per stack slot.
pub const STACK_FEATURES: usize = STACK_VALUES + EFFECT_VALUES + SLOT_FEATURES * EFFECT_SLOTS;
/// Offset of the viewer's public hero part; the opponent's follows it.
pub const HEROES: usize = GLOBALS;
/// Offset of the viewer's hero's private part.
pub const OWN_HERO: usize = HEROES + 2 * HERO_PUBLIC_FEATURES;
/// Offset of the battlefield planes.
pub const FIELD: usize = OWN_HERO + OWN_HERO_FEATURES;
/// Offset of the first stack slot.
pub const STACKS: usize = FIELD + HEX_PLANES * MAX_FIELD_HEXES;
/// Features per stack slot of the turn order.
pub const ORDER_FEATURES: usize = 3;
/// Offset of the turn order, after the stack slots: the length of v21's layout.
pub const ORDER: usize = STACKS + STACK_FEATURES * STACK_CAPACITY;
/// Length of the encoded observation.
pub const OBSERVATION_LEN: usize = ORDER + ORDER_FEATURES * STACK_CAPACITY;

/// The statuses of a stack's effects, in their encoded order.
const STATUSES: [Status; 8] = [
    Status::FORGETFUL,
    Status::NO_SHOOTING,
    Status::NO_TURNS,
    Status::DAMAGE_ENDS,
    Status::BERSERK,
    Status::HYPNOTIZED,
    Status::ATTACK_ENDS,
    Status::PETRIFIED,
];

/// Hex columns of the encoded field, used to derive `x`/`y` from the hex index.
fn coords(hex: u8, width: u8) -> (f32, f32) {
    let w = width.max(1);
    (f32::from(hex % w), f32::from(hex / w))
}

fn flag(on: bool) -> f32 {
    if on { 1.0 } else { 0.0 }
}

/// A handle or id + 1, 0 for none.
fn handle(raw: Option<u16>) -> f32 {
    raw.map_or(0.0, |h| f32::from(h) + 1.0)
}

/// Rounds left: −1 for the rest of the battle.
fn rounds(left: Option<u16>) -> f32 {
    left.map_or(-1.0, f32::from)
}

/// Writes the public part `view` of a hero into `out`, with its side's `cost_change`.
fn encode_hero(view: Option<HeroView>, cost_change: i8, out: &mut [f32]) {
    if let Some(h) = view {
        out.copy_from_slice(&[
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
            flag(h.spell_book),
            f32::from(cost_change),
            flag(h.commands_catapult),
        ]);
    }
}

/// Writes the private part `h` of the viewer's hero into `out`.
fn encode_own_hero(h: &OwnHeroView, out: &mut [f32]) {
    let b = &h.spell_bonuses;
    let d = &h.defences;
    let (scalars, rest) = out.split_at_mut(OWN_HERO_SCALARS);
    let mut values = [
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
        b.all.bp as f32,
    ]
    .into_iter()
    .chain(b.school.iter().map(|r| r.bp as f32))
    .chain([
        h.magic_resistance_bp as f32,
        f32::from(h.duration_bonus),
        flag(h.no_casting),
        h.level_limit.map_or(0.0, |l| f32::from(l) + 1.0),
        d.up_to_level.map_or(0.0, f32::from),
        flag(d.mind),
        flag(h.negates),
        flag(h.opening_due),
        f32::from(h.eagle_chance),
        h.catapult.map_or(0.0, |c| f32::from(c.level)),
        h.ballista.map_or(0.0, |b| f32::from(b.level)),
    ]);
    scalars.fill_with(|| values.next().unwrap_or_default());
    let (bonuses, rest) = rest.split_at_mut(4 * MAX_SPELL_BONUSES);
    for (slot, s) in bonuses.chunks_exact_mut(4).zip(b.spells.iter()) {
        slot.copy_from_slice(&[
            f32::from(s.spell.raw()) + 1.0,
            s.damage_bp as f32,
            s.per_level_bp as f32,
            f32::from(s.level),
        ]);
    }
    let (opening, spells) = rest.split_at_mut(3 * MAX_OPENING_SPELLS);
    for (slot, o) in opening.chunks_exact_mut(3).zip(h.opening.iter()) {
        slot.copy_from_slice(&[
            f32::from(o.spell.raw()) + 1.0,
            f32::from(o.mastery),
            f32::from(o.power),
        ]);
    }
    let (book, rest) = spells.split_at_mut(MAX_SPELLS);
    let (mastery, rest) = rest.split_at_mut(MAX_SPELLS);
    let (immune, eagle) = rest.split_at_mut(MAX_SPELLS);
    for (i, spell) in (0..MAX_SPELLS).map(|i| (i, h3_content::SpellId::from_raw(i as u16))) {
        book[i] = flag(h.spell_book.contains(spell));
        mastery[i] = f32::from(h.masteries.get(spell));
        immune[i] = flag(d.spells.contains(spell));
        eagle[i] = if h.learned.contains(spell) {
            2.0
        } else {
            flag(h.eagle_spells.contains(spell))
        };
    }
}

/// Writes the battlefield planes of `obs` into `out`.
fn encode_field(obs: &Observation, out: &mut [f32]) {
    let mut plane = |p: usize, hex: h3_battle::Hex, value: f32| {
        out[p * MAX_FIELD_HEXES..(p + 1) * MAX_FIELD_HEXES]
            .get_mut(hex.index())
            .into_iter()
            .for_each(|v| *v = value);
    };
    for hex in obs.obstacles.iter() {
        plane(0, hex, 1.0);
    }
    for hex in obs.obstacle_anchors.iter() {
        plane(1, hex, 1.0);
    }
    // rule: BAT-MAG-034
    let removable = obs.removable_obstacles.iter();
    for hex in removable.flat_map(|o| o.hexes().iter().copied().chain([o.anchor()])) {
        plane(2, hex, handle(obs.removable_at(hex).map(|i| i as u16)));
    }
    for o in &obs.spell_obstacles {
        encode_obstacle(obs.viewer, o, &mut plane);
    }
    // rule: BAT-SIE-002, BAT-SIE-004
    if let Some(s) = &obs.siege {
        s.walls()
            .iter()
            .for_each(|hex| plane(SIEGE_PLANES, hex, 1.0));
        s.moat()
            .iter()
            .for_each(|hex| plane(SIEGE_PLANES + 1, hex, 1.0));
    }
}

/// Writes spell obstacle `o` as `viewer` sees it with `plane`.
fn encode_obstacle(
    viewer: BattleSide,
    o: &SpellObstacle,
    plane: &mut impl FnMut(usize, h3_battle::Hex, f32),
) {
    let own = o.side == viewer;
    let base = 3 + if own { 0 } else { OWNER_PLANES };
    let kind = o.kind as usize;
    let left = rounds((o.rounds != 0).then_some(u16::from(o.rounds)));
    for hex in o.hexes.iter() {
        plane(base + kind, hex, left);
        plane(base + ObstacleKind::COUNT, hex, o.value as f32);
        if own {
            plane(REVEALED_PLANE, hex, flag(o.revealed));
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
    let last = obs
        .last_side_acted
        .map_or(0.0, |s| if s == obs.viewer { 1.0 } else { 2.0 });
    let (globals, walls) = out[..GLOBALS].split_at_mut(GLOBALS - MAX_WALL_TARGETS);
    globals.copy_from_slice(&[
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
        handle(obs.drawn.map(|spell| spell.raw())),
        f32::from(obs.periodic_rounds[own]),
        f32::from(obs.periodic_rounds[other]),
        flag(obs.wait_phase),
        last,
        handle(obs.terrain.map(|t| t.raw())),
        handle(obs.summoned[own].map(|c| c.raw())),
        handle(obs.summoned[other].map(|c| c.raw())),
        flag(obs.spectator),
        handle(obs.battlefield.map(|b| b.raw())),
        flag(obs.anti_magic),
        handle(obs.siege.map(|s| s.faction.raw())),
        obs.siege.map_or(0.0, |s| f32::from(s.fort as u8)),
        obs.siege.map_or(0.0, |s| f32::from(s.moat_kind as u8)),
        flag(obs.siege.is_some_and(|s| s.gate_open)),
        flag(obs.siege.is_some_and(|s| s.gate_destroyed())),
        // rule: BAT-SIE-023
        f32::from(obs.difficulty),
        flag(obs.human[own]),
        flag(obs.human[other]),
    ]);
    // rule: BAT-SIE-010
    for (t, v) in (0u8..).zip(walls.iter_mut()) {
        *v = obs.siege.map_or(0.0, |s| match (s.present(t), s.kind(t)) {
            (true, _) => f32::from(s.parts[usize::from(t)]),
            (false, Some(_)) => -1.0,
            (false, None) => 0.0,
        });
    }
    let (viewer, opponent) = out[HEROES..OWN_HERO].split_at_mut(HERO_PUBLIC_FEATURES);
    encode_hero(obs.heroes[own], obs.cost_change[own], viewer);
    encode_hero(obs.heroes[other], obs.cost_change[other], opponent);
    if let Some(h) = &obs.own_hero {
        encode_own_hero(h, &mut out[OWN_HERO..FIELD]);
    }
    encode_field(obs, &mut out[FIELD..STACKS]);
    for (i, s) in obs.stacks.iter().enumerate() {
        let f = &mut out[STACKS + i * STACK_FEATURES..STACKS + (i + 1) * STACK_FEATURES];
        let (x, y) = coords(s.hex.0, field_width);
        let corpse = match obs.corpses[i] {
            h3_battle::NO_CORPSE => -1.0,
            order => f32::from(order),
        };
        let binders = obs.binders[i];
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
            f32::from(obs.casts_left[i]),
            f32::from(obs.cast_candidates[i]),
            s.base_count as f32,
            flag(s.flags.contains(StackFlags::STARTED)),
            flag(s.has(StackTraits::RAISED)),
            flag(s.has(StackTraits::NATIVE)),
            obs.temporary[i] as f32,
            corpse,
            handle(obs.clone_links[i].map(|l| u16::from(l.0))),
            (binders & 0x1F_FFFF) as f32,
            (binders >> 21) as f32,
            f32::from(obs.poison[i]),
            obs.start_hp[i] as f32,
            // rule: BAT-SIE-016
            flag(s.is_turret()),
        ];
        f[..STACK_VALUES].copy_from_slice(&values);
        let sp = s.spells;
        let (blessed, cursed, change) = match sp.damage {
            DamageRoll::Random => (0.0, 0.0, 0),
            DamageRoll::Maximum(v) => (1.0, 0.0, v),
            DamageRoll::Minimum(v) => (0.0, 1.0, v),
        };
        let effects = obs.effects.iter().filter(|e| e.stack.index() == i);
        let mut values = [
            f32::from(sp.melee_attack),
            f32::from(sp.ranged_attack),
            blessed,
            cursed,
            f32::from(change),
            f32::from(sp.speed_percent),
            effects.clone().count() as f32,
            f32::from(sp.melee_protection),
            f32::from(sp.ranged_protection),
        ]
        .into_iter()
        .chain(sp.school_protection.map(f32::from))
        .chain([
            f32::from(sp.slayer.0),
            f32::from(sp.slayer.1),
            f32::from(sp.fire_shield),
            f32::from(sp.frenzy),
            f32::from(sp.spell_immunity),
            f32::from(sp.mirror),
        ])
        .chain(STATUSES.map(|st| flag(sp.status.contains(st))));
        f[STACK_VALUES..STACK_VALUES + EFFECT_VALUES]
            .fill_with(|| values.next().unwrap_or_default());
        let slots = f[STACK_VALUES + EFFECT_VALUES..].chunks_exact_mut(SLOT_FEATURES);
        for (slot, e) in slots.zip(effects) {
            slot.copy_from_slice(&[
                f32::from(e.spell.raw()) + 1.0,
                rounds(e.rounds),
                f32::from(e.mastery),
            ]);
        }
    }
    encode_order(obs, &mut out[ORDER..]);
    Ok(())
}

/// Writes each stack's places in the turn order into `out` (the order block).
// rule: BAT-TRN-001, BAT-TRN-002, BAT-TRN-003
fn encode_order(obs: &Observation, out: &mut [f32]) {
    let order = turn_order(obs);
    let mut sides = [0u8; 2];
    for (place, id) in (1u8..).zip(order.round.iter()) {
        let side = obs.stacks.as_slice()[id.index()].side.index();
        sides[side] += 1;
        let f = &mut out[id.index() * ORDER_FEATURES..];
        f[0] = f32::from(place);
        f[1] = f32::from(sides[side]);
    }
    for (place, id) in (1u8..).zip(order.next.iter()) {
        out[id.index() * ORDER_FEATURES + 2] = f32::from(place);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_battle::{BattleState, observe};

    /// The features of stack `i` in an encoded observation.
    fn stack(out: &[f32], i: usize) -> &[f32] {
        &out[STACKS + i * STACK_FEATURES..STACKS + (i + 1) * STACK_FEATURES]
    }

    /// The viewer's (0) or the opponent's (1) public hero part.
    fn hero(out: &[f32], block: usize) -> &[f32] {
        &out[HEROES + block * HERO_PUBLIC_FEATURES..HEROES + (block + 1) * HERO_PUBLIC_FEATURES]
    }

    /// The private part of the viewer's hero.
    fn own(out: &[f32]) -> &[f32] {
        &out[OWN_HERO..FIELD]
    }

    /// Battlefield plane `p`.
    fn plane(out: &[f32], p: usize) -> &[f32] {
        &out[FIELD + p * MAX_FIELD_HEXES..FIELD + (p + 1) * MAX_FIELD_HEXES]
    }

    /// The offset of a stack's effect slots.
    const SLOTS: usize = STACK_VALUES + EFFECT_VALUES;
    /// The offset of the private part's per-spell vectors.
    const SPELLS: usize = OWN_HERO_VALUES;

    /// E06 T5a: the entity network reads this encoding by `python/h3env/train/layout.py`
    /// (design §11): its sizes are these, and its stack columns hold what it reads them as.
    #[test]
    fn the_python_layout_is_the_encodings() {
        let python = crate::python_layout;
        assert_eq!(
            python("OBSERVATION_ENCODING"),
            OBSERVATION_ENCODING_VERSION as usize
        );
        for (name, value) in [
            ("GLOBALS", GLOBALS),
            ("HERO_PUBLIC_FEATURES", HERO_PUBLIC_FEATURES),
            ("OWN_HERO_FEATURES", OWN_HERO_FEATURES),
            ("HEX_PLANES", HEX_PLANES),
            ("MAX_FIELD_HEXES", MAX_FIELD_HEXES),
            ("STACK_FEATURES", STACK_FEATURES),
            ("STACK_CAPACITY", STACK_CAPACITY),
            ("ORDER_FEATURES", ORDER_FEATURES),
            ("ORDER_ROUND", 0),
            ("ORDER_SIDE", 1),
            ("ORDER_NEXT", 2),
        ] {
            assert_eq!(python(name), value, "{name}");
        }
        // The offsets follow from the sizes in the same order there; v21's layout is v22's
        // prefix, the turn order last.
        assert_eq!(
            (HEROES, OWN_HERO, FIELD),
            (GLOBALS, GLOBALS + 28, GLOBALS + 28 + OWN_HERO_FEATURES)
        );
        assert_eq!(ORDER, 8_882);
        assert_eq!(OBSERVATION_LEN, ORDER + ORDER_FEATURES * STACK_CAPACITY);
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_DUEL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let obs = observe(&s, &r, BattleSide::Defender);
        let width = r.battle().field.width;
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&obs, width, &mut out).unwrap();
        let column = |name| python(name);
        for (i, st) in obs.stacks.iter().enumerate() {
            let f = stack(&out, i);
            let (x, y) = coords(st.hex.0, width);
            assert_eq!(f[column("PRESENT")], 1.0);
            assert_eq!(f[column("CREATURE")], f32::from(st.creature.raw()));
            assert_eq!((f[column("X")], f[column("Y")]), (x, y));
            let active = obs.active.is_some_and(|a| a.index() == i);
            assert_eq!(f[column("ACTIVE")], if active { 1.0 } else { 0.0 });
        }
        assert_eq!(stack(&out, obs.stacks.len())[column("PRESENT")], 0.0);
        // E06 T6a: the behaviour analysis's columns (`h3env.train.inspect`), after a Wait and a
        // Defend so that the turn flags differ between stacks.
        let mut s = s;
        for command in [h3_battle::Command::Wait, h3_battle::Command::Defend] {
            let o = observe(&s, &r, BattleSide::Attacker);
            let side = o
                .stacks
                .get(o.active.unwrap().index())
                .unwrap()
                .controller();
            h3_battle::apply(&mut s, &r, side, command, &mut h3_battle::NoopSink).unwrap();
        }
        let obs = observe(&s, &r, BattleSide::Defender);
        encode(&obs, width, &mut out).unwrap();
        assert_eq!(out[column("ROUND")], f32::from(obs.round));
        let mut flags = [0usize; 4];
        for (i, st) in obs.stacks.iter().enumerate() {
            let f = stack(&out, i);
            let is = |set: bool| if set { 1.0 } else { 0.0 };
            let turn = [
                StackFlags::ACTED,
                StackFlags::WAITED,
                StackFlags::STARTED,
                StackFlags::DEFENDING,
            ];
            for ((name, bit), seen) in ["ACTED", "WAITED", "STARTED", "DEFENDING"]
                .iter()
                .zip(turn)
                .zip(&mut flags)
            {
                assert_eq!(f[column(name)], is(st.flags.contains(bit)), "{name}");
                *seen += usize::from(st.flags.contains(bit));
            }
            for (name, value) in [
                ("ALIVE", is(st.is_alive())),
                ("OWN", is(st.side == BattleSide::Defender)),
                ("COUNT", st.count as f32),
                ("TOP_HP", st.top_hp as f32),
                ("MAX_HP", st.max_hp as f32),
                ("RETALIATIONS_LEFT", f32::from(st.retaliations_left)),
                ("SHOOTER", is(st.has(StackTraits::SHOOTER))),
                ("WAR_MACHINE", is(st.has(StackTraits::WAR_MACHINE))),
                ("BASE_COUNT", st.base_count as f32),
                ("TURRET", is(st.is_turret())),
            ] {
                assert_eq!(f[column(name)], value, "{name}");
            }
        }
        // The Wait and the Defend set the flags on stacks of their own.
        assert!(flags.iter().all(|&n| n >= 1), "{flags:?}");
        let view = HeroView {
            hero_type: h3_content::HeroTypeId::from_index(0).unwrap(),
            attack: 1,
            defense: 2,
            power: 3,
            knowledge: 4,
            mana: 5,
            max_mana: 6,
            casts: 7,
            commands_ballista: false,
            commands_tent: false,
            commands_catapult: false,
            spell_book: true,
            sex: None,
        };
        let mut block = [0.0; HERO_PUBLIC_FEATURES];
        encode_hero(Some(view), 0, &mut block);
        for (name, value) in [
            ("HERO_MANA", 5.0),
            ("HERO_MAX_MANA", 6.0),
            ("HERO_CASTS", 7.0),
        ] {
            assert_eq!(block[column(name)], value, "{name}");
        }
    }

    #[test]
    fn encodes_globals_and_stacks() {
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_DUEL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let obs = observe(&s, &r, BattleSide::Defender);
        let mut out = vec![9.0; OBSERVATION_LEN + 3];
        encode(&obs, r.battle().field.width, &mut out).unwrap();
        // Round 1, the defender's view, the attacker to act; no tactics phase, no heroes, so no
        // retreat or surrender; no periodic cast yet on either side (BAT-MAG-053).
        // Not in the wait phase, no turn taken yet, the meadow, no summons, a side's view, no
        // special battlefield, no anti-magic garrison, not at a town.
        let meadow = f32::from(r.terrain_by_key("synthetic:meadow").unwrap().raw()) + 1.0;
        assert_eq!(
            out[..GLOBALS],
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 0.0,
                meadow, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            ]
        );
        assert!(out[HEROES..FIELD].iter().all(|v| *v == 0.0));
        // The random natural obstacles (BAT-GEO-004), none removed, no spell obstacle.
        let blocked: Vec<usize> = (0..MAX_FIELD_HEXES)
            .filter(|&h| plane(&out, 0)[h] == 1.0)
            .collect();
        let expected: Vec<usize> = obs.obstacles.iter().map(|h| h.index()).collect();
        assert_eq!(blocked, expected);
        for h in 0..MAX_FIELD_HEXES {
            let at = obs.removable_at(h3_battle::Hex(h as u8));
            assert_eq!(plane(&out, 2)[h], at.map_or(0.0, |i| i as f32 + 1.0));
        }
        assert!(
            out[FIELD + 3 * MAX_FIELD_HEXES..STACKS]
                .iter()
                .all(|v| *v == 0.0)
        );
        // Stack 0: the attacker's two-hex brute ×8 at (1, 5) (its front hex in column 2),
        // active.
        let s0 = stack(&out, 0);
        assert_eq!(s0[0..3], [1.0, 1.0, 0.0]);
        assert_eq!(s0[4], 8.0);
        assert_eq!((s0[12], s0[13]), (1.0, 5.0));
        assert_eq!(s0[18], 1.0);
        // Morale +1 (one-faction army, BAT-MOR-002), luck 0, no shots, one retaliation, two-hex
        // walker, nothing from a hero.
        assert_eq!(
            s0[19..31],
            [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        // Not created in battle, no clone (BAT-MAG-037, BAT-MAG-038), one cast: its rebirth's
        // (BAT-MAG-048, BAT-MAG-059).
        assert_eq!(s0[31..36], [0.0, 0.0, 0.0, 1.0, 0.0]);
        // 8 at the start, its turn started (the active stack), not raised, native to the
        // meadow (BAT-TER-001); no raise, corpse, link, binder or poison; no turret.
        assert_eq!(
            s0[36..STACK_VALUES],
            [8.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        // No timed effects: no bonuses, not slowed, no entries, all damage let through.
        assert_eq!(
            s0[STACK_VALUES..STACK_VALUES + 13],
            [
                0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 0.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0
            ]
        );
        assert!(s0[STACK_VALUES + 13..].iter().all(|v| *v == 0.0));
        // Unused slots are zero; the tail beyond OBSERVATION_LEN is untouched.
        assert!(
            out[STACKS + 2 * STACK_FEATURES..ORDER]
                .iter()
                .all(|v| *v == 0.0)
        );
        assert_eq!(out[OBSERVATION_LEN], 9.0);
        // The turn order (BAT-TRN-001): the active brute first, then the defender's stack, each
        // its side's next; the next round's places as the order tells them.
        let next = turn_order(&obs).next;
        let place = |i: u8| {
            (1u8..)
                .zip(next.iter())
                .find(|(_, id)| id.0 == i)
                .map_or(0.0, |(p, _)| f32::from(p))
        };
        assert_eq!(
            out[ORDER..ORDER + 2 * ORDER_FEATURES],
            [1.0, 1.0, place(0), 2.0, 1.0, place(1)]
        );
        assert_eq!(place(0) + place(1), 3.0);
        assert!(
            out[ORDER + 2 * ORDER_FEATURES..OBSERVATION_LEN]
                .iter()
                .all(|v| *v == 0.0)
        );
        let mut short = vec![0.0; 3];
        assert_eq!(encode(&obs, 17, &mut short), Err(OBSERVATION_LEN));
        assert_eq!(
            OBSERVATION_LEN,
            38 + 2 * 14 + (66 + 4 * 128) + 16 * 255 + (48 + 27 + 3 * 8) * 42 + 3 * 42
        );
        assert_eq!(coords(20, 0), (0.0, 20.0));
        // A clone with three rounds left (BAT-MAG-038).
        let mut obs = obs;
        let clone = &mut obs.stacks.as_mut_slice()[1];
        clone.traits = clone
            .traits
            .with(StackTraits::SUMMONED, true)
            .with(StackTraits::CLONE, true);
        obs.clone_rounds[1] = 3;
        obs.casts_left[1] = 2;
        // A drawn spell (BAT-MAG-049): its handle + 1.
        obs.drawn = Some(h3_content::SpellId::from_raw(4));
        // The spells a helpful caster's draw could pick for it (BAT-MAG-050).
        obs.cast_candidates[1] = 5;
        // The round starts since each side's periodic cast, the viewer's first (BAT-MAG-053).
        obs.periodic_rounds = [2, 0];
        encode(&obs, r.battle().field.width, &mut out).unwrap();
        assert_eq!(stack(&out, 1)[31..36], [1.0, 1.0, 3.0, 2.0, 5.0]);
        assert_eq!(out[11..14], [5.0, 0.0, 2.0]);
    }

    #[test]
    fn encodes_the_heroes_and_shows_the_private_part_only_to_its_side() {
        use BattleSide::{Attacker, Defender};
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
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
            1.0,
            -2.0,
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
            1.0,
            0.0,
            0.0,
        ];
        // The books by spell handle (spark, cinders, arc, blight, ward, farsight, glare, hasten,
        // drag, fury, zeal, gloom, boon, bane, corrode, aegis, hallow, smiting, riposte, purge,
        // mend, blaze, rage, veil, glass, haze, dazzle, rampage, charm, bulwark, scorch, blink,
        // mire, snare, uproot, revive, offer, copy, conjure), each as 1 + its mastery: Brea's
        // storm spells at Basic (six at Advanced since E04b T9b: spark, cinders, haze, charm,
        // scorch, snare), her stone ones (blight, revive) at none; Corwin's stone spells at
        // Expert, his storm ones (spark, arc) and hallow at none.
        let books: [&[(usize, f32)]; 2] = [
            &[
                (0, 3.0),
                (1, 3.0),
                (2, 2.0),
                (3, 1.0),
                (4, 2.0),
                (7, 2.0),
                (8, 2.0),
                (9, 2.0),
                (17, 2.0),
                (18, 2.0),
                (20, 2.0),
                (21, 2.0),
                (24, 2.0),
                (25, 3.0),
                (28, 3.0),
                (30, 3.0),
                (31, 2.0),
                (33, 3.0),
                (35, 1.0),
                (36, 2.0),
                (37, 2.0),
            ],
            &[
                (0, 1.0),
                (2, 1.0),
                (3, 4.0),
                (10, 4.0),
                (11, 4.0),
                (12, 4.0),
                (13, 4.0),
                (14, 4.0),
                (15, 4.0),
                (16, 1.0),
                (19, 4.0),
                (22, 4.0),
                (23, 4.0),
                (26, 4.0),
                (27, 4.0),
                (29, 4.0),
                (32, 4.0),
                (34, 4.0),
                (35, 4.0),
                (38, 4.0),
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
        let handle = |key| f32::from(r.spell_by_key(key).unwrap().raw()) + 1.0;
        let (hasten, boon) = (handle("synthetic:hasten"), handle("synthetic:boon"));
        let (drag, bane) = (handle("synthetic:drag"), handle("synthetic:bane"));
        // The casting workload's hidden magic (E04b T9b): resistance, Eagle Eye's chance, the
        // opening spells and the Pendant's spell.
        let brea_magic = (
            0.0,
            40.0,
            [hasten, 3.0, 3.0, 0.0, 0.0, 0.0],
            "synthetic:spark",
        );
        let corwin_magic = (
            1000.0,
            50.0,
            [drag, 3.0, 2.0, bane, 3.0, 2.0],
            "synthetic:cinders",
        );
        for (viewer, globals, mine, other, private, book, (rounds, specialty), magic) in [
            (
                Attacker,
                [
                    0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 5.0, 1.0, 1.0, 1.0, 1.0, 0.0, 255.0, 255.0, 0.0,
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                ],
                brea,
                corwin,
                brea_private,
                books[0],
                (0.0, [hasten, 0.0, 300.0, 6.0]),
                brea_magic,
            ),
            (
                Defender,
                [
                    0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0, 1.0, 1.0, 1.0, 1.0, 0.0, 255.0, 255.0, 0.0,
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                ],
                corwin,
                brea,
                corwin_private,
                books[1],
                (2.0, [boon, 0.0, 300.0, 1.0]),
                corwin_magic,
            ),
        ] {
            let obs = observe(&s, &r, viewer);
            encode(&obs, 17, &mut out).unwrap();
            assert_eq!(out[..GLOBALS], globals);
            assert_eq!(hero(&out, 0), mine);
            assert_eq!(hero(&out, 1), other);
            assert_eq!(own(&out)[..10], private);
            // The book as flags, each spell's mastery (of every spell: BAT-QUIRK-5).
            let in_book: Vec<(usize, f32)> = (0..MAX_SPELLS)
                .filter(|&i| own(&out)[SPELLS + i] == 1.0)
                .map(|i| (i, own(&out)[SPELLS + MAX_SPELLS + i] + 1.0))
                .collect();
            assert_eq!(in_book, book);
            let masteries = obs.own_hero.unwrap().masteries;
            for i in 0..MAX_SPELLS {
                let spell = h3_content::SpellId::from_raw(i as u16);
                assert_eq!(
                    own(&out)[SPELLS + MAX_SPELLS + i],
                    f32::from(masteries.get(spell))
                );
            }
            // The spell bonuses and the hidden magic: Brea's specialty in hasten (3 % a level at
            // level 6); Corwin's in boon (level 1) and his two extra rounds for timed spells
            // (HER-BAT-014); since E04b T9b the resistance, the opening spells (due), Eagle Eye
            // and the Pendant. The opponent's never shows.
            let (resistance, eagle, openings, immune) = magic;
            let mut hidden = [0.0; OWN_HERO_VALUES - 10];
            hidden[15 - 10] = resistance;
            hidden[16 - 10] = rounds;
            hidden[22 - 10] = 1.0;
            hidden[23 - 10] = eagle;
            // The commanded Ballista's Artillery level (v20): both heroes' 1.
            hidden[OWN_HERO_SCALARS - 1 - 10] = 1.0;
            hidden[OWN_HERO_SCALARS - 10..OWN_HERO_SCALARS - 6].copy_from_slice(&specialty);
            let opening = OWN_HERO_SCALARS + 4 * MAX_SPELL_BONUSES - 10;
            hidden[opening..opening + 6].copy_from_slice(&openings);
            assert_eq!(own(&out)[10..SPELLS], hidden);
            let immunities = &own(&out)[SPELLS + 2 * MAX_SPELLS..SPELLS + 3 * MAX_SPELLS];
            let pendant = r.spell_by_key(immune).unwrap().index();
            assert!(
                immunities
                    .iter()
                    .enumerate()
                    .all(|(i, v)| *v == f32::from(i == pendant))
            );
            // The spells Eagle Eye may learn, as the setup lists them; none learned yet.
            let learnable = &own(&out)[SPELLS + 3 * MAX_SPELLS..];
            let listed = &setup.heroes[viewer.index()]
                .as_ref()
                .unwrap()
                .magic
                .eagle_eye;
            let listed = &listed.as_ref().unwrap().spells;
            assert_eq!(learnable.len(), MAX_SPELLS);
            for (i, v) in learnable.iter().enumerate() {
                let spell = h3_content::SpellId::from_raw(i as u16);
                assert_eq!(*v, f32::from(listed.contains(&spell)));
            }
        }
        // Without an enemy hero: no surrender, a zero block; without a Ballista, zero control.
        let mut obs = observe(&s, &r, Attacker);
        obs.heroes[1] = None;
        obs.may_retreat[1] = false;
        obs.surrender_possible = [false, false];
        let mut hero_part = obs.own_hero.unwrap();
        hero_part.surrender_price = None;
        hero_part.ballista = None;
        obs.own_hero = Some(hero_part);
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(out[7..14], [1.0, 0.0, 0.0, 0.0, 0.0, 255.0, 255.0]);
        assert!(hero(&out, 1).iter().all(|v| *v == 0.0));
        assert_eq!(own(&out)[4..6], [0.0, 0.0]);
        assert_eq!(own(&out)[9], 0.0);
        // A spectator: the flag, and no private part (SRV-Q4).
        let spectator = h3_battle::observe_as(&s, &r, h3_battle::Viewer::Spectator);
        encode(&spectator, 17, &mut out).unwrap();
        assert_eq!((out[1], out[19]), (1.0, 1.0));
        assert_eq!(hero(&out, 0), brea);
        assert!(own(&out).iter().all(|v| *v == 0.0));
        // A special battlefield and an anti-magic garrison (E04c T2b).
        let mut field = spectator;
        field.battlefield = h3_content_battle::BattlefieldId::from_index(1);
        field.anti_magic = true;
        encode(&field, 17, &mut out).unwrap();
        assert_eq!(out[20..22], [2.0, 1.0]);
    }

    #[test]
    fn encodes_the_town() {
        // The vale's Citadel with a damaging moat of two rows (E04c T5): the town's globals and
        // planes, the gate closed, then open.
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_SIEGE).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut obs = observe(&s, &r, BattleSide::Attacker);
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&obs, 17, &mut out).unwrap();
        let vale = f32::from(r.faction_by_key("synthetic:vale").unwrap().raw()) + 1.0;
        let globals = GLOBALS - MAX_WALL_TARGETS - 8;
        assert_eq!(out[globals..globals + 5], [vale, 2.0, 2.0, 0.0, 0.0]);
        // The game's difficulty and the human players (E04c T7d): the defaults, then the
        // viewer's side human on Hard.
        assert_eq!(out[globals + 5..globals + 8], [0.0, 0.0, 0.0]);
        let mut hard = obs.clone();
        hard.difficulty = 2;
        hard.human = [true, false];
        encode(&hard, 17, &mut out).unwrap();
        assert_eq!(out[globals + 5..globals + 8], [2.0, 1.0, 0.0]);
        encode(&obs, 17, &mut out).unwrap();
        // The wall-shot targets' hit points: a Citadel sets up the keep, not the towers
        // (BAT-SIE-010).
        assert_eq!(
            out[GLOBALS - MAX_WALL_TARGETS..GLOBALS],
            [-1.0, 2.0, 2.0, 2.0, 2.0, 2.0, -1.0, 2.0]
        );
        // The keep, the last stack, is a turret off the field (BAT-SIE-016): 254 is (16, 14).
        let keep = stack(&out, s.stacks().len() - 1);
        assert_eq!(
            (keep[12], keep[13], keep[STACK_VALUES - 1]),
            (16.0, 14.0, 1.0)
        );
        assert_eq!(stack(&out, 0)[STACK_VALUES - 1], 0.0);
        let on = |out: &[f32], p: usize| -> Vec<usize> {
            let plane = &out[FIELD + p * MAX_FIELD_HEXES..FIELD + (p + 1) * MAX_FIELD_HEXES];
            (0..MAX_FIELD_HEXES).filter(|&h| plane[h] == 1.0).collect()
        };
        assert_eq!(
            on(&out, SIEGE_PLANES),
            [12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182]
        );
        let moat = on(&out, SIEGE_PLANES + 1);
        assert_eq!(moat.len(), 22);
        assert!(moat.contains(&94) && moat.contains(&95));
        // An open gate lowers the drawbridge over the bridges' hexes.
        let mut town = obs.siege.unwrap();
        town.gate_open = true;
        obs.siege = Some(town);
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(out[globals + 3..globals + 5], [1.0, 0.0]);
        let moat = on(&out, SIEGE_PLANES + 1);
        assert_eq!(moat.len(), 20);
        assert!(!moat.contains(&94) && !moat.contains(&95));
    }

    #[test]
    fn encodes_morale_luck_shots_war_machines_and_traits_from_heroes() {
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
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
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
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
            ActiveEffect {
                mastery: 2,
                ..entry(0, spell("synthetic:corrode"), None)
            },
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
        let e = STACK_VALUES;
        assert_eq!(f[e..e + 7], [3.0, 4.0, 0.0, 1.0, 2.0, 50.0, 2.0]);
        let hasten = f32::from(spell("synthetic:hasten").raw()) + 1.0;
        let corrode = f32::from(spell("synthetic:corrode").raw()) + 1.0;
        assert_eq!(f[SLOTS..SLOTS + 6], [hasten, 3.0, 0.0, corrode, -1.0, 2.0]);
        assert!(f[SLOTS + 6..].iter().all(|v| *v == 0.0));
        obs.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Maximum(1);
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(stack(&out, 0)[e + 2..e + 5], [1.0, 0.0, 1.0]);
        assert!(stack(&out, 1)[SLOTS..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn encodes_the_battlefield() {
        use h3_battle::{Hex, HexSet, SpellObstacle};
        use h3_content_battle::ObstacleKind::{Fire, Mine, Pit, Wall};
        let (r, mut setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_DUEL).unwrap();
        // A removable obstacle on hexes 40 and 41 with its anchor 57, a fixed one on 60.
        setup.obstacles = std::vec![
            h3_contracts::Obstacle {
                hexes: std::vec![40, 41],
                anchor: Some(57),
                kind: None,
            },
            h3_contracts::Obstacle {
                hexes: std::vec![60],
                anchor: None,
                kind: None,
            },
        ];
        let s = BattleState::new(&r, &setup, 1).unwrap();
        use BattleSide::{Attacker, Defender};
        let mut obs = observe(&s, &r, Defender);
        let spell = r.spell_by_key("synthetic:mire").unwrap();
        let obstacle = |kind, side, hex, rounds, value, revealed| {
            let mut hexes = HexSet::new();
            hexes.insert(Hex(hex));
            SpellObstacle {
                spell,
                kind,
                side,
                hexes,
                rounds,
                value,
                revealed,
            }
        };
        obs.spell_obstacles = std::vec![
            obstacle(Wall, Defender, 70, 2, 0, true),
            obstacle(Fire, Attacker, 71, 1, 25, true),
            obstacle(Pit, Defender, 72, 0, 0, false),
            obstacle(Pit, Defender, 73, 0, 0, true),
            obstacle(Mine, Attacker, 74, 0, 40, true),
            obstacle(Fire, Defender, 74, 2, 30, true),
        ];
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&obs, 17, &mut out).unwrap();
        let on = |p: usize| -> Vec<(usize, f32)> {
            (0..MAX_FIELD_HEXES)
                .filter(|&h| plane(&out, p)[h] != 0.0)
                .map(|h| (h, plane(&out, p)[h]))
                .collect()
        };
        assert_eq!(on(0), [(40, 1.0), (41, 1.0), (60, 1.0)]);
        let anchors: Vec<(usize, f32)> = obs
            .obstacle_anchors
            .iter()
            .map(|h| (h.index(), 1.0))
            .collect();
        assert_eq!(on(1), anchors);
        // Remove Obstacle finds the first obstacle on its hexes, and on its anchor with
        // BAT-QUIRK-3 (BAT-MAG-034).
        let anchored: &[(usize, f32)] = if anchors.is_empty() {
            &[]
        } else {
            &[(57, 1.0)]
        };
        let mut removable = std::vec![(40, 1.0), (41, 1.0)];
        removable.extend_from_slice(anchored);
        removable.sort_by_key(|&(h, _)| h);
        assert_eq!(on(2), removable);
        // The viewer's (the defender's) obstacles, then the attacker's: rounds by kind, −1 for
        // the whole battle, and the value.
        let (own, theirs) = (3, 3 + OWNER_PLANES);
        assert_eq!(on(own), [(70, 2.0)]);
        assert_eq!(on(own + 1), [(74, 2.0)]);
        assert_eq!(on(own + 2), [(72, -1.0), (73, -1.0)]);
        assert!(on(own + 3).is_empty());
        assert_eq!(on(own + 4), [(74, 30.0)]);
        assert!(on(theirs).is_empty());
        assert_eq!(on(theirs + 1), [(71, 1.0)]);
        assert!(on(theirs + 2).is_empty());
        assert_eq!(on(theirs + 3), [(74, -1.0)]);
        assert_eq!(on(theirs + 4), [(71, 25.0), (74, 40.0)]);
        // Which of the viewer's the attacker sees: all but the hidden pit.
        assert_eq!(on(REVEALED_PLANE), [(70, 1.0), (73, 1.0), (74, 1.0)]);
    }

    #[test]
    fn encodes_the_hidden_magic_and_what_the_battle_keeps_per_stack() {
        use h3_battle::{Opening, SpellBonus, StackId, Status};
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut obs = observe(&s, &r, BattleSide::Defender);
        let spell = |k: &str| r.spell_by_key(k).unwrap();
        let mut h = obs.own_hero.unwrap();
        // From no hidden magic of the scenario's (Corwin has some since E04b T9b).
        h.opening.clear();
        h.defences = h3_battle::Defences::default();
        h.eagle_spells = Default::default();
        h.learned = Default::default();
        h.spell_bonuses.all = h3_contracts::Rate::bp(500);
        h.spell_bonuses.school[3] = h3_contracts::Rate::bp(700);
        h.spell_bonuses.spells.clear();
        h.spell_bonuses
            .spells
            .push(SpellBonus {
                spell: spell("synthetic:spark"),
                damage_bp: 5000,
                per_level_bp: 0,
                level: 0,
            })
            .unwrap();
        h.magic_resistance_bp = 2000;
        h.duration_bonus = 0;
        h.no_casting = true;
        h.level_limit = Some(2);
        h.defences.up_to_level = Some(3);
        h.defences.mind = true;
        h.defences.spells.insert(spell("synthetic:glare"));
        h.negates = true;
        h.opening_due = true;
        h.opening
            .push(Opening {
                spell: spell("synthetic:boon"),
                mastery: 3,
                power: 10,
            })
            .unwrap();
        h.eagle_chance = 40;
        h.eagle_spells.insert(spell("synthetic:spark"));
        h.eagle_spells.insert(spell("synthetic:arc"));
        h.learned.insert(spell("synthetic:arc"));
        obs.own_hero = Some(h);
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&obs, 17, &mut out).unwrap();
        let handle = |k: &str| f32::from(spell(k).raw()) + 1.0;
        assert_eq!(own(&out)[10..15], [500.0, 0.0, 0.0, 0.0, 700.0]);
        assert_eq!(
            own(&out)[15..OWN_HERO_SCALARS],
            [2000.0, 0.0, 1.0, 3.0, 3.0, 1.0, 1.0, 1.0, 40.0, 0.0, 1.0]
        );
        let bonus = OWN_HERO_SCALARS;
        assert_eq!(
            own(&out)[bonus..bonus + 8],
            [
                handle("synthetic:spark"),
                5000.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0
            ]
        );
        let opening = bonus + 4 * MAX_SPELL_BONUSES;
        assert_eq!(
            own(&out)[opening..opening + 4],
            [handle("synthetic:boon"), 3.0, 10.0, 0.0]
        );
        let immune = SPELLS + 2 * MAX_SPELLS;
        let glare = spell("synthetic:glare").index();
        assert_eq!(own(&out)[immune + glare], 1.0);
        assert_eq!(
            own(&out)[immune..immune + MAX_SPELLS].iter().sum::<f32>(),
            1.0
        );
        let eagle = &own(&out)[SPELLS + 3 * MAX_SPELLS..];
        let picks: Vec<(usize, f32)> = (0..MAX_SPELLS)
            .filter(|&i| eagle[i] != 0.0)
            .map(|i| (i, eagle[i]))
            .collect();
        let mut expected = std::vec![
            (spell("synthetic:spark").index(), 1.0),
            (spell("synthetic:arc").index(), 2.0)
        ];
        expected.sort_by_key(|&(i, _)| i);
        assert_eq!(picks, expected);
        // Per stack: what the battle keeps (public).
        obs.temporary[0] = 4;
        obs.corpses[0] = 3;
        obs.corpses[1] = h3_battle::NO_CORPSE;
        obs.clone_links[0] = Some(StackId(5));
        obs.binders[0] = (1 << 41) | (1 << 20) | 1;
        obs.poison[0] = 2;
        obs.start_hp[0] = 30;
        obs.summoned = [Some(r.creature_by_key("synthetic:wisp").unwrap()), None];
        obs.wait_phase = true;
        obs.last_side_acted = Some(BattleSide::Attacker);
        let st = &mut obs.stacks.as_mut_slice()[0];
        st.spells.melee_protection = 70;
        st.spells.ranged_protection = 75;
        st.spells.school_protection = [50, 100, 100, 70];
        st.spells.slayer = (8, 3);
        st.spells.fire_shield = 20;
        st.spells.frenzy = 150;
        st.spells.spell_immunity = 4;
        st.spells.mirror = 40;
        st.spells.status = Status::default()
            .with(Status::HYPNOTIZED, true)
            .with(Status::PETRIFIED, true);
        encode(&obs, 17, &mut out).unwrap();
        let wisp = f32::from(r.creature_by_key("synthetic:wisp").unwrap().raw()) + 1.0;
        assert_eq!(out[14..19], [1.0, 2.0, 0.0, 0.0, wisp]);
        let f = stack(&out, 0);
        assert_eq!(
            f[40..STACK_VALUES],
            [4.0, 3.0, 6.0, 1_048_577.0, 1_048_576.0, 2.0, 30.0, 0.0]
        );
        assert_eq!(stack(&out, 1)[41], -1.0);
        let e = STACK_VALUES + 7;
        assert_eq!(
            f[e..SLOTS],
            [
                70.0, 75.0, 50.0, 100.0, 100.0, 70.0, 8.0, 3.0, 20.0, 150.0, 4.0, 40.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0, 0.0, 1.0
            ]
        );
    }
}
