//! Mandatory invariant property tests (docs/05-testing.md "Invariants every context must
//! provide"): atomicity, legality consistency, ownership isolation, conservation, observation
//! hygiene, clone equivalence and determinism.
//!
//! Battles are generated from random armies; each step either plays a random legal command or
//! throws an arbitrary (usually invalid) command at the engine, possibly from the wrong actor.
//! The heroes cast, among the other spells, Berserk, Hypnotize and Blind (E04b T11b review,
//! sec F-4): a stack's controller is not always its side, and some turns are automatic. The
//! assertions are stated for the trusted actor and the controller (BAT-MAG-025, BAT-MAG-026).
//! Half the heroes have hidden magic in force: casting limits, spell defences, the negation of
//! immunities, opening spells, Eagle Eye (BAT-MAG-061 to BAT-MAG-069; private to their side,
//! BAT-Q26).

use std::num::NonZeroU32;
use std::vec::Vec;

use h3_content::{
    Ability, Affects, BattleParamsDef, ContentKey, CreatureFlag, CreatureSelector, CreatureStat,
    EffectDuration, Formation, ObstacleKind, ProfileDef, Ruleset, SchoolId, SpellCategory, SpellId,
    SpellScope, TimedChange,
};
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleEffect, BattleHero, BattleResult, BattleSetup,
    BattleSide, DamageBonuses, EagleEye, StackOrigin, StackRecord, StatOp, TentControl,
};
use h3_kernel::Rng;
use proptest::collection::{btree_map, vec};
use proptest::prelude::{
    Just, Strategy, any, prop_assert, prop_assert_eq, prop_assume, prop_oneof, proptest,
};
use proptest::test_runner::Config;

use super::support::{CREATURES, Rate, hero, masteries, ruleset_from, ruleset_with};
use crate::event::StrikeFlags;
use crate::{
    BattleState, CastTarget, CastTargets, Command, Event, EventKind, Hex, LegalCommands, SpellSet,
    StackId, Viewer, apply, cast_targets, castable_spells, creature_cast_targets, legal_commands,
    observe, observe_as, project, project_as, second_targets, validate,
};

/// The test ruleset with frequent good and bad morale and luck (a 4-sided die) and the quirk
/// Q-0003 on, so that generated games exercise them.
fn lively() -> Ruleset {
    lively_with("t:quirky")
}

/// The battle parameters of [`lively`].
fn enliven(b: &mut BattleParamsDef) {
    b.morale_dice = 4;
    b.good_morale_chances = std::vec![1, 2, 3];
    b.bad_morale_chances = std::vec![1, 2, 3];
    b.luck_dice = 4;
    b.good_luck_chances = std::vec![1, 2, 3];
    // Low enough for the casts of a battle to reach it (BAT-MAG-037).
    b.side_stack_limit = 12;
}

/// [`lively`] with the profile `profile`.
fn lively_with(profile: &str) -> Ruleset {
    ruleset_with(profile, enliven)
}

/// [`lively`] with a profile of its own that has the quirks `quirks` (their keys after
/// `quirk.battle.`) together: the test pack's profiles have one quirk each, a real pack's
/// several.
fn lively_mix(quirks: &[&str]) -> Ruleset {
    ruleset_from("t:mix", enliven, |pack| {
        pack.profiles.push(ProfileDef {
            key: ContentKey::new("t", "mix").unwrap(),
            quirks: quirks
                .iter()
                .map(|quirk| (std::format!("quirk.battle.{quirk}"), true))
                .collect(),
        });
    })
}

/// One step of a generated game.
#[derive(Debug, Clone)]
enum Step {
    /// Play the `n`-th legal command that does not leave the battle (modulo their number;
    /// leaving is exercised by arbitrary steps and the legality probes).
    Legal(u16),
    /// Try an arbitrary command, as the side to act or its opponent.
    Arbitrary { command: Command, wrong_actor: bool },
    /// Cast the `n`-th castable spell at its `n`-th target (modulo their numbers), if any; for an
    /// odd `n` (or no castable spell) the active stack casts at its `n`-th target instead, if it
    /// may (BAT-MAG-048).
    Cast(u16),
    /// Cast the `n`-th castable control spell (Berserk, Hypnotize, Blind: BAT-MAG-024 to
    /// BAT-MAG-026) at its `n`-th target, Berserk at a hex a living stack stands on if it may, so
    /// that stacks do go berserk; without a castable one, as `Legal(n)`.
    Control(u16),
    /// Cast Clone (BAT-MAG-038) at its `n`-th target, a two-hex stack if it may: one in a crowd
    /// becomes its own clone with quirk BAT-QUIRK-7 ([`crowd`]); without a castable Clone, as
    /// `Legal(n)`.
    Clone(u16),
}

/// The control spells of the test ruleset that a `Control` step casts.
const CONTROL_SPELLS: [&str; 3] = ["t:berserk", "t:hypnotize", "t:blind"];

/// An arbitrary stack id: mostly one a battle may have (it holds 42 stacks), sometimes any byte.
fn arb_stack() -> impl Strategy<Value = StackId> {
    prop_oneof![4 => 0u8..44, 1 => any::<u8>()].prop_map(StackId)
}

/// An arbitrary cast target.
fn arb_target() -> impl Strategy<Value = CastTarget> {
    prop_oneof![
        Just(CastTarget::None),
        arb_stack().prop_map(CastTarget::Stack),
        any::<u8>().prop_map(|h| CastTarget::Hex(Hex(h))),
    ]
}

/// An arbitrary command; leaving the battle is rare, so that games run long. A cast names any
/// spell of the test ruleset (it has 61) or none of it, and its second target may be a hex
/// (Teleport's, BAT-MAG-029) or a stack (Sacrifice's victim, BAT-MAG-036).
fn arb_command() -> impl Strategy<Value = Command> {
    prop_oneof![
        8 => Just(Command::Wait),
        8 => Just(Command::Defend),
        8 => Just(Command::EndTactics),
        8 => any::<u8>().prop_map(|h| Command::Move { to: Hex(h) }),
        8 => (arb_stack(), any::<u8>(), 0u8..4).prop_map(|(target, h, kind)| match kind {
            0 => Command::Shoot { target, at: Hex(h) },
            _ => Command::Attack { target, from: Hex(h) },
        }),
        8 => arb_stack().prop_map(|target| Command::Heal { target }),
        8 => (0u16..80, arb_target(), prop_oneof![2 => Just(CastTarget::None), 1 => arb_target()])
            .prop_map(|(spell, target, second)| Command::Cast {
                spell: SpellId::from_raw(spell),
                target,
                second,
            }),
        8 => arb_target().prop_map(|target| Command::CreatureCast { target }),
        1 => Just(Command::Retreat),
        1 => Just(Command::Surrender),
    ]
}

fn arb_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        6 => any::<u16>().prop_map(Step::Legal),
        2 => (arb_command(), any::<bool>()).prop_map(|(command, wrong_actor)| Step::Arbitrary { command, wrong_actor }),
        2 => any::<u16>().prop_map(Step::Cast),
        1 => any::<u16>().prop_map(Step::Control),
        1 => any::<u16>().prop_map(Step::Clone),
    ]
}

/// Indices of the test creatures that may be in an army (war machines may not).
fn army_creatures() -> Vec<u8> {
    (0u8..)
        .zip(CREATURES)
        .filter(|(_, c)| !c.7.contains(&"war_machine"))
        .map(|(i, _)| i)
        .collect()
}

fn arb_army() -> impl Strategy<Value = ArmySpec> {
    let pool = army_creatures();
    btree_map(0u8..7, (0..pool.len(), 1u32..60), 1..=7).prop_map(move |m| {
        m.into_iter()
            .map(|(slot, (c, n))| (slot, pool[c], n))
            .collect()
    })
}

/// `(slot, creature index, count)` per stack.
type ArmySpec = Vec<(u8, u8, u32)>;
/// Attacker, defender, obstacles, seed, steps, the hidden magic of the attacker's and the
/// defender's hero.
type Game = (ArmySpec, ArmySpec, Vec<u8>, u64, Vec<Step>, [Hidden; 2]);

/// The hidden magic of a generated hero (BAT-MAG-061 to BAT-MAG-069; private to its side,
/// BAT-Q26), each part as [`conceal`] reads it.
#[derive(Debug, Clone, Default)]
struct Hidden {
    /// No hero and no stack casts (BAT-MAG-061).
    no_casting: bool,
    /// The highest spell level the battle allows (BAT-MAG-062).
    level_limit: Option<u8>,
    /// A spell its stacks are immune to (BAT-MAG-063): 0 to 2 one of [`CONTROL_SPELLS`], else a
    /// spell of the ruleset by index, modulo their number.
    immune_to: Option<u16>,
    /// The spell level up to which its stacks are immune (BAT-MAG-064).
    immune_up_to: Option<u8>,
    /// Its stacks are immune to mind spells (BAT-MAG-065).
    mind_immunity: bool,
    /// It negates immunities (BAT-MAG-066).
    negates: bool,
    /// Its opening spells (BAT-MAG-068): an index into [`openings`], modulo their number, and
    /// the power.
    opening: Vec<(u16, u16)>,
    /// Its Eagle Eye (BAT-MAG-069): the chance (0, 40 or 100 %, by index) and the spells of its
    /// book it may pick, a bit each.
    eagle_eye: Option<(u8, u64)>,
}

/// A hero's hidden magic: none for half the heroes, so that most games play every spell; for
/// the others each part with a chance of its own (no casting at all is rare: it stops every cast
/// of its battle).
fn arb_hidden() -> impl Strategy<Value = Hidden> {
    use proptest::bool::weighted as with;
    use proptest::option::weighted as some;
    let parts = (
        with(0.1),
        some(0.2, 1u8..4),
        some(0.3, prop_oneof![0u16..3, any::<u16>()]),
        some(0.2, 1u8..3),
        with(0.2),
        with(0.2),
        prop_oneof![2 => Just(Vec::new()), 1 => vec((any::<u16>(), 1u16..4), 1..3)],
        some(0.3, (0u8..3, any::<u64>())),
    );
    let hidden = parts.prop_map(|parts| {
        let (no_casting, level_limit, immune_to, immune_up_to, mind_immunity, negates, ..) = parts;
        Hidden {
            no_casting,
            level_limit,
            immune_to,
            immune_up_to,
            mind_immunity,
            negates,
            opening: parts.6,
            eagle_eye: parts.7,
        }
    });
    prop_oneof![Just(Hidden::default()), hidden]
}

/// Every spell of the ruleset with a mastery at which a hero may have it as an opening spell
/// (BAT-MAG-068).
fn openings(r: &Ruleset) -> Vec<(SpellId, u8)> {
    (0..r.spells().len())
        .filter_map(SpellId::from_index)
        .flat_map(|id| (0..=3).map(move |mastery| (id, mastery)))
        .filter(|&(id, mastery)| {
            r.spell(id)
                .is_some_and(|t| crate::magic::opens_with(r, t, mastery))
        })
        .collect()
}

/// Gives `hero` the hidden magic `hidden` (BAT-MAG-061 to BAT-MAG-069).
fn conceal(r: &Ruleset, hero: &mut BattleHero, hidden: &Hidden) {
    if hidden.no_casting {
        hero.effects.push(BattleEffect::NoCasting);
    }
    if let Some(max_level) = hidden.level_limit {
        hero.effects
            .push(BattleEffect::SpellLevelLimit { max_level });
    }
    if let Some(i) = hidden.immune_to {
        let i = usize::from(i);
        let spell = CONTROL_SPELLS
            .get(i)
            .and_then(|key| r.spell_by_key(key))
            .or_else(|| SpellId::from_index(i % r.spells().len()))
            .unwrap();
        hero.effects.push(BattleEffect::SpellImmunity { spell });
    }
    if let Some(max_level) = hidden.immune_up_to {
        hero.effects
            .push(BattleEffect::SpellLevelImmunity { max_level });
    }
    if hidden.mind_immunity {
        hero.effects.push(BattleEffect::MindImmunity);
    }
    if hidden.negates {
        hero.effects.push(BattleEffect::NegateImmunities);
    }
    let openings = openings(r);
    for &(i, power) in &hidden.opening {
        let (spell, mastery) = openings[usize::from(i) % openings.len()];
        hero.effects.push(BattleEffect::OpeningSpell {
            spell,
            mastery,
            power,
        });
    }
    if let Some((chance, picks)) = hidden.eagle_eye {
        let book = hero.magic.spell_book.clone().unwrap_or_default();
        hero.magic.eagle_eye = Some(EagleEye {
            chance: Rate::bp([0, 4000, 10_000][usize::from(chance % 3)]),
            spells: (0..64)
                .zip(book)
                .filter(|(bit, _)| picks >> bit & 1 == 1)
                .map(|(_, spell)| spell)
                .collect(),
        });
    }
}

/// What the heroes of a setup do to all of its battle's magic, by the rules (BAT-MAG-061 to
/// BAT-MAG-066), read from the setup.
struct Limits {
    /// A hero forbids every cast (BAT-MAG-061).
    no_casting: bool,
    /// The highest spell level the heroes allow (BAT-MAG-062).
    max_level: u8,
    /// Per side, what the stacks its hero controls are immune to: the spells, the spell level up
    /// to which, and whether mind spells (BAT-MAG-063 to BAT-MAG-065).
    immune: [(Vec<SpellId>, Option<u8>, bool); 2],
    /// Per side, whether its hero negates immunities (BAT-MAG-066).
    negates: [bool; 2],
}

impl Limits {
    fn of(setup: &BattleSetup) -> Self {
        let effects = |side: usize| setup.heroes[side].iter().flat_map(|h| h.effects.iter());
        let all = || effects(0).chain(effects(1));
        let limit = |e: &BattleEffect| match e {
            BattleEffect::SpellLevelLimit { max_level } => Some(*max_level),
            _ => None,
        };
        let spell = |e: &BattleEffect| match e {
            BattleEffect::SpellImmunity { spell } => Some(*spell),
            _ => None,
        };
        let level = |e: &BattleEffect| match e {
            BattleEffect::SpellLevelImmunity { max_level } => Some(*max_level),
            _ => None,
        };
        Self {
            no_casting: all().any(|e| matches!(e, BattleEffect::NoCasting)),
            max_level: all().filter_map(limit).min().unwrap_or(u8::MAX),
            immune: [0, 1].map(|side| {
                (
                    effects(side).filter_map(spell).collect(),
                    effects(side).filter_map(level).max(),
                    effects(side).any(|e| matches!(e, BattleEffect::MindImmunity)),
                )
            }),
            negates: [0, 1]
                .map(|side| effects(side).any(|e| matches!(e, BattleEffect::NegateImmunities))),
        }
    }
}

/// The setup of a generated game: [`make_setup`] of its armies and obstacles from the word
/// `machines` (its seed), each hero with its hidden magic.
fn game_setup(
    r: &Ruleset,
    (attacker, defender, obstacles): (&ArmySpec, &ArmySpec, &[u8]),
    machines: u64,
    hidden: &[Hidden; 2],
) -> BattleSetup {
    let mut setup = make_setup(r, attacker, defender, obstacles, machines);
    for (hero, hidden) in setup.heroes.iter_mut().zip(hidden) {
        hero.iter_mut().for_each(|hero| conceal(r, hero, hidden));
    }
    setup
}

/// Up to eight obstacle hexes away from the edge columns (x in 2..=14 of the 17-wide field).
fn arb_obstacles() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::btree_set((2u8..15, 0u8..11).prop_map(|(x, y)| x + 17 * y), 0..8)
        .prop_map(|s| s.into_iter().collect())
}

/// The obstacles over `hexes`, one per hex: by the hex modulo 3 without an anchor, anchored on
/// itself, or anchored on its left neighbour (which it does not block, BAT-QUIRK-3; perhaps
/// another obstacle's hex).
fn pieces(hexes: &[u8]) -> Vec<h3_contracts::Obstacle> {
    hexes
        .iter()
        .map(|&h| h3_contracts::Obstacle {
            hexes: std::vec![h],
            anchor: [None, Some(h), Some(h - 1)][usize::from(h % 3)],
        })
        .collect()
}

fn arb_game() -> impl Strategy<Value = Game> {
    (
        arb_army(),
        arb_army(),
        arb_obstacles(),
        any::<u64>(),
        vec(arb_step(), 1..160),
        [arb_hidden(), arb_hidden()],
    )
}

/// A setup from generated armies and obstacles; bits 0–5 of `machines` choose the attacker's
/// and the defender's war machines (ballista, ammo cart, first aid tent), bits 6–7 whether the
/// attacker and the defender stand in tight formation, bit 8 whether the attacker's hero
/// commands its Ballista (bit 9: two shots; bits 10–23: the double-damage chance), bit 24
/// whether it commands its First Aid Tent (bits 25–40: the heal bonus), bits 41–42 the attacker
/// hero's Tactics level if bit 43 is set, bit 44 whether the defender is led by a hero too, with
/// Tactics level bits 45–46 (so that some games start with a tactics phase, BAT-TAC-001), bit
/// 47 whether the attacker's hero wears Shackles of War (no retreat, BAT-FLE-001), bits 48–55
/// and 56–63 the attacker's and the defender's funds in hundreds of gold (BAT-FLE-002). Both
/// heroes have spell books (38 combat spells of the test ruleset, the control spells Berserk,
/// Hypnotize and Blind among them: BAT-MAG-024 to BAT-MAG-026) and the attacker's some
/// spell bonuses; their mana comes from bits 0–5 and 6–11 of `machines` rotated by 13
/// (BAT-MAG-001 to BAT-MAG-004), the battlefield's terrain from its bits 14–15 (none, the plain
/// or lava, so that some stacks are native: BAT-TER-001, BAT-MAG-031). The attacker's air mastery
/// is its bits 12–13 and its fire mastery Basic; of `machines` rotated by 29, bits 0–1 give the
/// attacker's hero power 10 instead of 2 one time in four and bits 2–3 the defender's 6 instead
/// of 1 (Hypnotize takes a stack of at most power × 25 + 10 to 50 hit points), bits 4–5 and 6–7
/// the defender's air and fire masteries (Berserk's area and Blind's retaliation are by mastery).
fn make_setup(
    r: &Ruleset,
    attacker: &[(u8, u8, u32)],
    defender: &[(u8, u8, u32)],
    obstacles: &[u8],
    machines: u64,
) -> BattleSetup {
    let pick = |bits: u64| -> Vec<_> {
        ["ballista", "cart", "tent"]
            .iter()
            .enumerate()
            .filter(|(i, _)| bits & (1 << i) != 0)
            .map(|(_, k)| r.creature_by_key(&std::format!("t:{k}")).unwrap())
            .collect()
    };
    let war_machines = [pick(machines & 7), pick((machines >> 3) & 7)];
    let army = |spec: &[(u8, u8, u32)]| Army {
        stacks: spec
            .iter()
            .map(|&(slot, c, count)| ArmyStack {
                slot,
                creature: r
                    .creature_by_key(&std::format!("t:{}", CREATURES[usize::from(c)].0))
                    .unwrap(),
                count,
            })
            .collect(),
    };
    let mut led = hero(r);
    if machines & (1 << 8) != 0 {
        led.war_machines.ballista = Some(BallistaControl {
            shots: 1 + u8::from(machines & (1 << 9) != 0),
            double_damage_chance_bp: u16::try_from((machines >> 10) % 16_384)
                .unwrap()
                .min(10_000),
        });
    }
    if machines & (1 << 24) != 0 {
        led.war_machines.tent = Some(TentControl {
            heal_bonus: Rate::bp(u32::try_from((machines >> 25) & 0xffff).unwrap()),
        });
    }
    if machines & (1 << 43) != 0 {
        led.tactics = u8::try_from((machines >> 41) & 3).unwrap();
    }
    if machines & (1 << 47) != 0 {
        led.effects.push(BattleEffect::NoRetreat);
    }
    let magic = machines.rotate_left(13);
    let book: Vec<SpellId> = [
        "bolt",
        "smite",
        "ward",
        "nova",
        "portal",
        "chain",
        "storm",
        "ring",
        "haste",
        "sloth",
        "prayer",
        "weaken",
        "ray",
        "skin",
        "lust",
        "aim",
        "bless",
        "curse",
        "mirth",
        "sorrow",
        "fortune",
        "misfortune",
        "dispel",
        "wall",
        "firewall",
        "teleport",
        "quicksand",
        "landmine",
        "remove",
        "rise",
        "animate",
        "offering",
        "copy",
        "sprites",
        "serfs",
        "berserk",
        "hypnotize",
        "blind",
    ]
    .iter()
    .map(|k| r.spell_by_key(&std::format!("t:{k}")).unwrap())
    .collect();
    let lift = machines.rotate_left(29);
    let two_bits = |word: u64, shift: u32| u8::try_from((word >> shift) & 3).unwrap();
    led.magic.spell_book = Some(book.clone());
    led.magic.mana = u16::try_from(magic & 63).unwrap();
    led.magic.masteries = masteries(r, [two_bits(magic, 12), 1, 0, 0]);
    led.power = if two_bits(lift, 0) == 3 { 10 } else { 2 };
    led.effects.extend([
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate: Rate::bp(1000),
        },
        BattleEffect::SpellSpecialty {
            spell: book[0],
            bp_per_level: 300,
            level: 5,
        },
        BattleEffect::MagicResistance {
            rate: Rate::bp(500),
        },
        BattleEffect::SpellDuration { rounds: 1 },
    ]);
    let defender_hero = (machines & (1 << 44) != 0).then(|| {
        let mut h = hero(r);
        h.tactics = u8::try_from((machines >> 45) & 3).unwrap();
        h.magic.spell_book = Some(book.clone());
        h.magic.mana = u16::try_from((magic >> 6) & 63).unwrap();
        h.magic.masteries = masteries(r, [two_bits(lift, 4), two_bits(lift, 6), 0, 0]);
        if two_bits(lift, 2) == 3 {
            h.power = 6;
        }
        h
    });
    BattleSetup {
        attacker: army(attacker),
        defender: army(defender),
        obstacles: pieces(obstacles),
        heroes: [Some(led), defender_hero],
        terrain: match (magic >> 14) & 3 {
            1 => r.terrain_by_key("t:plain"),
            2 => r.terrain_by_key("t:lava"),
            _ => None,
        },
        war_machines,
        funds: [48, 56].map(|bit| u32::try_from((machines >> bit) & 0xff).unwrap() * 100),
        formations: [6, 7].map(|bit| {
            if machines & (1 << bit) == 0 {
                Formation::Loose
            } else {
                Formation::Tight
            }
        }),
    }
}

/// The cast of `side`'s castable `spell` at the `n`-th of its targets that `prefer` accepts, of
/// all its targets if it accepts none.
fn aimed(
    state: &BattleState,
    r: &Ruleset,
    (side, spell): (BattleSide, SpellId),
    n: u16,
    prefer: impl Fn(&CastTarget) -> bool,
) -> Command {
    let mut targets = CastTargets::new();
    cast_targets(state, r, side, spell, &mut targets);
    let preferred: Vec<CastTarget> = targets.iter().copied().filter(|t| prefer(t)).collect();
    let pool = if preferred.is_empty() {
        targets.as_slice()
    } else {
        &preferred
    };
    Command::Cast {
        spell,
        target: pool[usize::from(n) % pool.len()],
        second: CastTarget::None,
    }
}

/// Walls in a living two-hex stack of `state` that is no war machine (the first whose side has a
/// hero, else the first):
/// every free hex up to two hexes from it is blocked, but the one ahead of its head. Clone, cast
/// at it, then finds no place for a clone before the stack's own hexes, which count as free with
/// quirk BAT-QUIRK-7: the stack becomes its own clone (BAT-MAG-038).
fn crowd(state: &mut BattleState, r: &Ruleset) {
    let field = crate::Field::new(r.battle().field);
    let wide = |led: bool| {
        let fits = |s: &&crate::Stack| {
            let clonable =
                s.is_alive() && s.is_two_hex() && !s.has(crate::StackTraits::WAR_MACHINE);
            clonable && (!led || state.hero(s.side).is_some())
        };
        state.stacks().iter().find(fits).copied()
    };
    let Some(stack) = wide(true).or_else(|| wide(false)) else {
        return;
    };
    let own = stack.footprint(field);
    let ahead = field.neighbor(own.head, crate::Field::head_direction(stack.side));
    let taken = state.living_hexes(field);
    for hex in (0..=u8::MAX).map(Hex).take(field.hex_count()) {
        let near = own.hexes().any(|h| field.distance(h, hex) <= 2);
        if near && Some(hex) != ahead && field.is_standable(hex) && !taken.contains(hex) {
            state.natural.block(hex);
        }
    }
}

/// Resolves a step into `(actor, command)` for the current state. The side to act is the active
/// stack's controller (BAT-MAG-026): a step that is not from the wrong actor is its command, also
/// for a stack of the other side that it hypnotized.
fn resolve(
    state: &BattleState,
    r: &Ruleset,
    step: &Step,
    legal: &mut LegalCommands,
) -> (BattleSide, Command) {
    let side = state.side_to_act().unwrap_or(BattleSide::Attacker);
    match step {
        Step::Control(n) => {
            let castable = castable_spells(state, r, side);
            let controls: Vec<SpellId> = CONTROL_SPELLS
                .iter()
                .filter_map(|key| r.spell_by_key(key))
                .filter(|&spell| castable.contains(spell))
                .collect();
            let Some(&spell) = controls.get(usize::from(*n) % controls.len().max(1)) else {
                return resolve(state, r, &Step::Legal(*n), legal);
            };
            let on_stack = |t: &CastTarget| match t {
                CastTarget::Hex(h) => state.stacks().iter().any(|s| s.is_alive() && s.hex == *h),
                _ => true,
            };
            (side, aimed(state, r, (side, spell), *n, on_stack))
        }
        Step::Clone(n) => {
            let clone = r.spell_by_key("t:copy");
            let Some(spell) =
                clone.filter(|&spell| castable_spells(state, r, side).contains(spell))
            else {
                return resolve(state, r, &Step::Legal(*n), legal);
            };
            let wide = |t: &CastTarget| {
                t.stack()
                    .is_some_and(|id| state.stacks()[id.index()].is_two_hex())
            };
            (side, aimed(state, r, (side, spell), *n, wide))
        }
        Step::Legal(n) => {
            legal_commands(state, r, side, legal);
            // The commands that leave the battle come last.
            let staying = legal.iter().take_while(|c| !c.leaves_battle()).count();
            let cmd = legal
                .get(usize::from(*n) % staying.max(1))
                .copied()
                .unwrap_or(Command::Wait);
            (side, cmd)
        }
        Step::Arbitrary {
            command,
            wrong_actor,
        } => (if *wrong_actor { side.opponent() } else { side }, *command),
        Step::Cast(n) => {
            let spells: Vec<SpellId> = castable_spells(state, r, side).iter().collect();
            let mut creature = CastTargets::new();
            creature_cast_targets(state, r, side, &mut creature);
            if !creature.is_empty() && (n % 2 == 1 || spells.is_empty()) {
                let target = creature.as_slice()[usize::from(*n) % creature.len()];
                return (side, Command::CreatureCast { target });
            }
            let Some(&spell) = spells.get(usize::from(*n) % spells.len().max(1)) else {
                return (side, Command::Wait);
            };
            let mut targets = CastTargets::new();
            cast_targets(state, r, side, spell, &mut targets);
            let target = targets.as_slice()[usize::from(*n) % targets.len()];
            second_targets(state, r, side, (spell, target), &mut targets);
            let second = targets.as_slice()[usize::from(*n) % targets.len()];
            (
                side,
                Command::Cast {
                    spell,
                    target,
                    second,
                },
            )
        }
    }
}

/// The timed changes `spell` puts on a stack.
fn changes(r: &Ruleset, spell: SpellId) -> impl Iterator<Item = TimedChange> + '_ {
    r.spell(spell)
        .into_iter()
        .flat_map(|s| s.magic.effects.iter().filter_map(|e| e.timed()))
}

/// Whether `spell` puts the timed change `change` on a stack.
fn has_change(r: &Ruleset, spell: SpellId, change: TimedChange) -> bool {
    changes(r, spell).any(|c| c == change)
}

/// Whether `spell`'s effects last for the rest of the battle: such an effect stays on a stack
/// that dies (BAT-MAG-010).
fn is_permanent(r: &Ruleset, spell: SpellId) -> bool {
    r.spell(spell)
        .is_some_and(|s| s.magic.duration == EffectDuration::Permanent)
}

/// Timed effects on stacks, each as its stack and spell: the state's list (ADR-0014 point 5), or
/// what a command's events tell of it.
type Tracked = Vec<(StackId, SpellId)>;

/// The effect list of `state`.
fn tracked(state: &BattleState) -> Tracked {
    state.effects().iter().map(|e| (e.stack, e.spell)).collect()
}

/// Whether an effect of `list` on stack `id` is of a spell with a timed change that `test`
/// accepts.
fn under(
    r: &Ruleset,
    list: &[(StackId, SpellId)],
    id: StackId,
    test: fn(TimedChange) -> bool,
) -> bool {
    list.iter()
        .any(|&(stack, spell)| stack == id && changes(r, spell).any(test))
}

/// Whether Berserk is among `list`'s effects on stack `id` (BAT-MAG-025).
fn berserk(r: &Ruleset, list: &[(StackId, SpellId)], id: StackId) -> bool {
    under(r, list, id, |c| c == TimedChange::Berserk)
}

/// The side that commands stack `id`, by the rule (BAT-MAG-026): its own, or the other side while
/// a Hypnotize is active on it; read from the effect list, not from what the stack keeps.
fn controller_by_rule(state: &BattleState, r: &Ruleset, id: StackId) -> BattleSide {
    let side = state.stacks()[id.index()].side;
    let hypnotized = state
        .effects()
        .iter()
        .any(|e| e.stack == id && has_change(r, e.spell, TimedChange::Hypnotize));
    if hypnotized { side.opponent() } else { side }
}

/// Plays a game, returning the final state, the event log and the state hash after every step.
fn play(
    r: &Ruleset,
    setup: &BattleSetup,
    seed: u64,
    steps: &[Step],
) -> (BattleState, Vec<Event>, Vec<u64>) {
    let mut state = BattleState::new(r, setup, seed).unwrap();
    let mut events = Vec::new();
    let mut hashes = Vec::new();
    let mut legal = LegalCommands::new();
    for step in steps {
        let (actor, cmd) = resolve(&state, r, step, &mut legal);
        let _ = apply(&mut state, r, actor, cmd, &mut events);
        hashes.push(state.state_hash());
    }
    (state, events, hashes)
}

proptest! {
    #![proptest_config(Config { cases: 96, ..Config::default() })]

    /// Atomicity, legality consistency, ownership isolation and conservation, checked at every
    /// step, by the trusted actor and the active stack's controller: who commands and whose hero
    /// casts (BAT-MAG-026), which turns are automatic (a berserk stack's, BAT-MAG-025; a war
    /// machine's, BAT-WAR-002), and what a command or such a turn may move and strike; and the
    /// battle's end (BAT-END-001, BAT-END-002). Half the games with quirk BAT-QUIRK-7 as well,
    /// one of their two-hex stacks in a crowd: Clone turns it into its own clone, which may
    /// vanish as its side's last stack (BAT-MAG-038).
    #[test]
    fn every_step_preserves_the_invariants(
        (att, def, obstacles, seed, steps, hidden) in arb_game(),
        own_clone: bool,
    ) {
        let r = if own_clone {
            lively_mix(&["no_luck_without_hero", "clone_on_its_original"])
        } else {
            lively()
        };
        let setup = game_setup(&r, (&att, &def, &obstacles), seed, &hidden);
        let limits = Limits::of(&setup);
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        if own_clone {
            crowd(&mut state, &r);
        }
        let mut legal = LegalCommands::new();
        for (turn, step) in steps.iter().enumerate() {
            let (actor, cmd) = resolve(&state, &r, step, &mut legal);

            // Legality consistency: the legal set is exactly what apply accepts. Every listed
            // command is accepted; exactly the side that controls the active stack has commands
            // (BAT-MAG-026): not its own side while the other side has hypnotized it.
            legal_commands(&state, &r, actor, &mut legal);
            prop_assert_eq!(legal.is_empty(), state.side_to_act() != Some(actor));
            for c in legal.iter() {
                let mut probe = state.clone();
                prop_assert!(apply(&mut probe, &r, actor, *c, &mut crate::NoopSink).is_ok(), "legal {c:?} rejected");
            }
            let mut other = LegalCommands::new();
            legal_commands(&state, &r, actor.opponent(), &mut other);
            prop_assert_eq!(other.is_empty(), state.side_to_act() != Some(actor.opponent()));
            // The same for casts: every listed spell, target and second target is accepted
            // (BAT-MAG-001, BAT-MAG-029). Each passes the validation `apply` runs before it
            // changes anything, and is applied to a copy of the state; of the casts aimed at a
            // hex (an area spell has 165 targets, Teleport as many places for each stack) every
            // second one is applied, by turns from step to step.
            let castable = castable_spells(&state, &r, actor);
            let mut targets = CastTargets::new();
            let mut seconds = CastTargets::new();
            // No hero casts while a hero forbids casting, and none a spell above the heroes'
            // level limit (BAT-MAG-061, BAT-MAG-062).
            prop_assert!(!limits.no_casting || castable.is_empty());
            for spell in castable.iter() {
                prop_assert!(r.spell(spell).unwrap().level <= limits.max_level);
                cast_targets(&state, &r, actor, spell, &mut targets);
                prop_assert!(!targets.is_empty());
                for target in targets.iter() {
                    second_targets(&state, &r, actor, (spell, *target), &mut seconds);
                    prop_assert!(!seconds.is_empty());
                    for second in seconds.iter() {
                        let c = Command::Cast { spell, target: *target, second: *second };
                        prop_assert!(validate(&state, &r, actor, c).is_ok(), "legal {c:?} rejected");
                        let hex = target.hex().or(second.hex());
                        if hex.is_none_or(|h| (usize::from(h.0) + turn) % 2 == 0) {
                            let mut probe = state.clone();
                            prop_assert!(apply(&mut probe, &r, actor, c, &mut crate::NoopSink).is_ok(), "legal {c:?} rejected");
                        }
                    }
                }
            }
            prop_assert!(castable_spells(&state, &r, actor.opponent()).is_empty() || state.side_to_act() == Some(actor.opponent()));
            // And for the active stack's casts (BAT-MAG-048).
            creature_cast_targets(&state, &r, actor, &mut targets);
            for target in targets.iter() {
                let mut probe = state.clone();
                let c = Command::CreatureCast { target: *target };
                prop_assert!(apply(&mut probe, &r, actor, c, &mut crate::NoopSink).is_ok(), "legal {c:?} rejected");
            }
            creature_cast_targets(&state, &r, actor.opponent(), &mut targets);
            prop_assert!(targets.is_empty() || state.side_to_act() == Some(actor.opponent()));
            let listed = |c: Command| match c {
                Command::Cast { spell, target, second } => {
                    let mut t = CastTargets::new();
                    cast_targets(&state, &r, actor, spell, &mut t);
                    let first = castable.contains(spell) && t.iter().any(|x| *x == target);
                    second_targets(&state, &r, actor, (spell, target), &mut t);
                    first && t.iter().any(|x| *x == second)
                }
                Command::CreatureCast { target } => {
                    let mut t = CastTargets::new();
                    creature_cast_targets(&state, &r, actor, &mut t);
                    t.iter().any(|x| *x == target)
                }
                other => legal.iter().any(|c| *c == other),
            };
            let cmd_listed = listed(cmd);

            let before = state.clone();
            let before_hash = state.state_hash();
            let mut events = Vec::new();
            let result = apply(&mut state, &r, actor, cmd, &mut events);
            match result {
                Err(_) => {
                    // Atomicity: a rejected command changes nothing.
                    prop_assert_eq!(&state, &before);
                    prop_assert_eq!(state.state_hash(), before_hash);
                    prop_assert!(events.is_empty());
                    // ... and it was not in the legal set.
                    prop_assert!(!cmd_listed, "legal {cmd:?} rejected");
                }
                Ok(()) => {
                    prop_assert!(cmd_listed, "illegal {cmd:?} accepted");
                    // A side leaves the battle only by its own command: the trusted actor's
                    // side, whoever owns the active stack (BAT-FLE-001, BAT-FLE-002).
                    let left = match state.result() {
                        Some(BattleResult::Retreat(side) | BattleResult::Surrender(side)) => Some(side),
                        _ => None,
                    };
                    prop_assert_eq!(left, cmd.leaves_battle().then_some(actor));
                    // Mana is spent only by casts, by the spell's cost, at most once per round
                    // and side (BAT-MAG-001, BAT-MAG-002); besides, it comes only from channels
                    // and goes only to drains, as their events report (BAT-MAG-055,
                    // BAT-MAG-056); a cast keeps the turn ...
                    let per_round = r.battle().hero_casts_per_round;
                    for side in BattleSide::BOTH {
                        let (b, a) = (before.hero(side), state.hero(side));
                        if let (Some(b), Some(a)) = (b, a) {
                            let (mut gained, mut drained) = (0u32, 0u32);
                            for e in &events {
                                match e.kind {
                                    EventKind::ManaChanneled { side: s, mana } if s == side => {
                                        gained += u32::from(mana);
                                    }
                                    EventKind::ManaDrained { side: s, mana, .. } if s == side => {
                                        drained += u32::from(mana);
                                    }
                                    _ => {}
                                }
                            }
                            let (a_mana, b_mana) = (u32::from(a.mana) + drained, u32::from(b.mana) + gained);
                            prop_assert!(a_mana <= b_mana && a.casts <= per_round);
                            let cast_by = matches!(cmd, Command::Cast { .. }) && side == actor;
                            prop_assert!(cast_by || a_mana == b_mana);
                        }
                    }
                    // The hero that casts is the trusted actor's, also during the turn of a stack
                    // of the other side that the actor controls (BAT-MAG-026); nothing else is a
                    // hero's cast.
                    let casters: Vec<BattleSide> = events
                        .iter()
                        .filter_map(|e| match e.kind {
                            EventKind::SpellCast { side, .. } => Some(side),
                            _ => None,
                        })
                        .collect();
                    let cast_now = matches!(cmd, Command::Cast { .. });
                    prop_assert_eq!(&casters, &[actor][..usize::from(cast_now)]);
                    // The casting limits hold whoever casts (BAT-MAG-061, BAT-MAG-062). While a
                    // hero forbids casting no hero and no stack casts or raises, on command or on
                    // its own, but a stack whose creature ignores that; the opening spells and
                    // the strikes' spells are no casts. No spell above the heroes' level limit
                    // is cast or placed (and none takes effect on a stack: in the walk below).
                    for e in &events {
                        let ignores = |stack: StackId| r.abilities(state.stacks()[stack.index()].creature).contains(&Ability::IgnoresMagicBlock);
                        let (cast, spell) = match e.kind {
                            EventKind::SpellCast { spell, .. } | EventKind::PeriodicCast { spell, .. } => (true, Some(spell)),
                            EventKind::StackCast { stack, spell, .. } => (!ignores(stack), Some(spell)),
                            EventKind::CreatureRaise { .. } => (true, None),
                            EventKind::OpeningSpell { spell, .. } | EventKind::ObstaclePlaced { spell, .. } => (false, Some(spell)),
                            _ => (false, None),
                        };
                        prop_assert!(!(cast && limits.no_casting), "a cast while a hero forbids casting: {:?}", e.kind);
                        prop_assert!(spell.is_none_or(|s| r.spell(s).unwrap().level <= limits.max_level), "above the level limit: {:?}", e.kind);
                    }
                    let mover = before.active().unwrap();
                    // ... unless it killed the active stack, whose turn then ends (BAT-MAG-004,
                    // BAT-MAG-036), or made it berserk: it takes its automatic turn at once
                    // (BAT-MAG-025).
                    if cast_now {
                        let killed = !state.stacks()[mover.index()].is_alive();
                        let enraged = events.iter().any(|e| matches!(e.kind,
                            EventKind::EffectApplied { target, spell, .. } if target == mover && has_change(&r, spell, TimedChange::Berserk)));
                        prop_assert!(state.is_over() || state.active() == Some(mover) || killed || enraged);
                    }
                    // A stack's cast uses one of its casts and nothing else does (BAT-MAG-048)
                    // but its rebirths (BAT-MAG-059): one cast each time it comes back, which
                    // may be more than once in a command (its own action and the automatic
                    // turns after it: two berserk stacks may kill it one after the other), and
                    // one more if a draw of none left it dead; a slot a created stack or a Pit
                    // Lord's Demons take over starts afresh (BAT-MAG-037, BAT-MAG-052).
                    let used = usize::from(matches!(cmd, Command::CreatureCast { .. }));
                    for (i, a) in state.stacks().iter().enumerate() {
                        let id = StackId(u8::try_from(i).unwrap());
                        let spent = usize::from(id == mover) * used;
                        let reused = events.iter().any(|e| matches!(e.kind, EventKind::StackCreated { stack, .. } | EventKind::RoseFromCorpse { stack, .. } if stack == id));
                        let reborn = events.iter().filter(|e| matches!(e.kind, EventKind::Reborn { stack, .. } if stack == id)).count();
                        let none_drawn = usize::from(a.has(crate::StackTraits::REBIRTH) && !a.is_alive());
                        let (was, is) = (usize::from(before.casts_left(id)), usize::from(state.casts_left(id)));
                        prop_assert!(reused || was == is + spent + reborn || was == is + spent + reborn + none_drawn);
                    }
                    let target = match cmd {
                        Command::Attack { target, .. } | Command::Shoot { target, .. } => Some(target),
                        _ => None,
                    };
                    // Who acts when. The command's own action is the active stack's, which the
                    // actor controls (BAT-MAG-026); every turn that follows within the command
                    // starts with `TurnStarted` and is automatic: a war machine's (BAT-WAR-002)
                    // or a berserk stack's (BAT-MAG-025), until a stack awaits its side's
                    // command. The walk follows the timed effects by the events alone: who is
                    // berserk and who loses its turns at each event.
                    let side_of = |id: StackId| state.stacks()[id.index()].side;
                    let machine = |id: StackId| state.stacks()[id.index()].has(crate::StackTraits::WAR_MACHINE);
                    let stunned = |live: &Tracked, id: StackId| under(&r, live, id, TimedChange::takes_turns);
                    let moves = matches!(cmd, Command::Move { .. } | Command::Attack { .. });
                    let mut live = tracked(&before);
                    // The side that controls a stack under the effects of a list (BAT-MAG-026).
                    let led = |list: &Tracked, id: StackId| {
                        let other = under(&r, list, id, |c| c == TimedChange::Hypnotize);
                        if other { side_of(id).opponent() } else { side_of(id) }
                    };
                    // The effects as the last cast found them (who controlled each stack then),
                    // the side that cast (a stack casts for the side that controls it), and
                    // for a strike's spell the stack it landed on.
                    let mut at_cast = live.clone();
                    let (mut caster, mut by_strike) = (None, None);
                    // The stack whose turn it is, whether it is still the turn the actor commands,
                    // the stack it last struck, the berserk stack that has yet to act, and the
                    // one that has attacked: its own attack ends its Berserk before the next turn
                    // starts, unless it died in it (BAT-MAG-025).
                    let (mut acting, mut commanded, mut struck) = (mover, true, None);
                    let (mut due, mut attacked) = (None, None);
                    let calmed = |live: &Tracked, stack: Option<StackId>| {
                        stack.is_none_or(|s| !berserk(&r, live, s) || !state.stacks()[s.index()].is_alive())
                    };
                    for (k, e) in events.iter().enumerate() {
                        let next = |n: usize| events.get(k + n).map(|e| e.kind);
                        let cast = match e.kind {
                            EventKind::SpellCast { side, .. } | EventKind::OpeningSpell { side, .. } => Some((side, None)),
                            EventKind::StackCast { stack, .. } | EventKind::PeriodicCast { stack, .. } => Some((led(&live, stack), None)),
                            EventKind::StrikeSpell { striker, target, spell } => Some((led(&live, striker), Some((target, spell)))),
                            _ => None,
                        };
                        if let Some((side, strike)) = cast {
                            at_cast.clone_from(&live);
                            (caster, by_strike) = (Some(side), strike);
                        }
                        match e.kind {
                            EventKind::TurnStarted { stack } => {
                                prop_assert_eq!(due, None, "a berserk stack's turn left no trace");
                                prop_assert!(calmed(&live, attacked), "{attacked:?} attacked and is still berserk");
                                (acting, commanded, attacked) = (stack, false, None);
                                due = Some(stack).filter(|&s| berserk(&r, &live, s));
                            }
                            // No spell takes effect on a stack against the heroes' magic in
                            // force, whoever casts it: none above the level limit (BAT-MAG-062);
                            // none its controlling hero's immunities name (BAT-MAG-063), though
                            // it skip the immunities otherwise; none up to that hero's immune
                            // level (BAT-MAG-064) and no mind spell if that hero's stacks are
                            // immune to them (BAT-MAG-065), unless the spell skips the
                            // immunities (BAT-MAG-016), and the mind spell also when the hero of
                            // the side that casts or the controlling hero negates immunities
                            // (BAT-MAG-066: not the other side's hero for a side's spell at a
                            // stack it controls). The controlling hero is that of the side that
                            // controlled the stack when the spell was cast (BAT-MAG-026). Only
                            // what a strike puts on a stack without a test asks nothing (Bind,
                            // Acid Breath: BAT-MAG-045, BAT-MAG-046).
                            EventKind::EffectApplied { target, spell, .. } => {
                                let cast = r.spell(spell).unwrap();
                                let skips = cast.magic.ignores_immunity;
                                let untested = skips && by_strike == Some((target, spell));
                                let hero = led(&at_cast, target);
                                let (spells, level, mind) = &limits.immune[hero.index()];
                                let negated = [caster, Some(hero)].into_iter().flatten().any(|side| limits.negates[side.index()]);
                                prop_assert!(untested || cast.level <= limits.max_level, "{spell:?} is above the level limit");
                                prop_assert!(untested || !spells.contains(&spell), "{target:?} is immune to {spell:?}");
                                prop_assert!(skips || level.is_none_or(|l| cast.level > l), "{target:?} is immune to the level of {spell:?}");
                                prop_assert!(skips || negated || !(cast.magic.mind && *mind), "{target:?} is immune to mind spells");
                                // A cast that makes the active stack berserk lets it take its
                                // automatic turn at once (BAT-MAG-025).
                                if !live.contains(&(target, spell)) {
                                    live.push((target, spell));
                                }
                                if commanded && target == acting && has_change(&r, spell, TimedChange::Berserk) {
                                    due = Some(target);
                                }
                            }
                            // A stack that is no longer berserk awaits its side's command.
                            EventKind::EffectEnded { target, spell } => {
                                live.retain(|&effect| effect != (target, spell));
                                due = due.filter(|&s| berserk(&r, &live, s));
                            }
                            // A stack that comes back keeps only what lasts the whole battle, and a
                            // new one in a dead stack's slot starts without effects (BAT-MAG-010).
                            EventKind::Reborn { stack, .. } => live.retain(|&(s, spell)| s != stack || is_permanent(&r, spell)),
                            EventKind::StackCreated { stack, .. } | EventKind::RoseFromCorpse { stack, .. } => live.retain(|&(s, _)| s != stack),
                            // A stack moves on its own turn only: by the actor's command to move or
                            // to attack, or on its own while it is berserk (no war machine moves).
                            EventKind::Moved { stack, .. } => {
                                prop_assert_eq!(stack, acting);
                                prop_assert!(berserk(&r, &live, stack) || (commanded && moves), "{stack:?} moved on its own");
                                due = due.filter(|&s| s != stack);
                            }
                            // Teleport is its side's hero's cast on a stack of that side, whoever
                            // controls the stack (BAT-MAG-029, BAT-MAG-026).
                            EventKind::Teleported { stack, .. } => {
                                prop_assert!(cast_now && side_of(stack) == actor);
                            }
                            EventKind::Struck { attacker, target: hit, flags, .. } if !flags.contains(StrikeFlags::SECONDARY) => {
                                if flags.contains(StrikeFlags::RETALIATION) {
                                    // The struck stack retaliates against the striker (BAT-RET-001).
                                    prop_assert_eq!((Some(attacker), hit), (struck, acting));
                                } else {
                                    // A stack strikes on its own turn only, and another stack.
                                    prop_assert_eq!(attacker, acting);
                                    prop_assert!(hit != attacker);
                                    struck = Some(hit);
                                    if berserk(&r, &live, attacker) {
                                        // A berserk stack takes any other stack (BAT-MAG-025).
                                        due = due.filter(|&s| s != attacker);
                                        attacked = Some(attacker);
                                    } else if commanded {
                                        // The commanded strikes hit the commanded target, a stack
                                        // of the side the actor fights, whoever controls it: never
                                        // one of the actor's own side (BAT-ATK-001, BAT-MAG-026).
                                        prop_assert_eq!(Some(hit), target);
                                        prop_assert!(side_of(hit) != actor);
                                    } else {
                                        // The automatic Ballista shoots the other side's stacks,
                                        // whoever controls them (BAT-WAR-002).
                                        prop_assert!(machine(attacker) && side_of(hit) != side_of(attacker), "{attacker:?} struck on its own");
                                    }
                                }
                            }
                            // A turn passes without an action for a stack that loses its turns
                            // (BAT-MAG-024, BAT-MAG-043) and for a berserk one with no stack to
                            // attack or unable to move (BAT-MAG-025).
                            EventKind::TurnPassed { stack } => {
                                prop_assert_eq!(stack, acting);
                                prop_assert!(stunned(&live, stack) || berserk(&r, &live, stack));
                                due = due.filter(|&s| s != stack);
                            }
                            // Bad morale and Fear freeze a stack as its turn starts, a berserk one
                            // too (BAT-MOR-004, BAT-MAG-060).
                            EventKind::BadMorale { stack } | EventKind::Frightened { stack } => {
                                prop_assert!(!commanded && stack == acting);
                                due = due.filter(|&s| s != stack);
                            }
                            // Only the actor's command makes a stack wait, defend, cast or raise:
                            // an automatic turn does none of these (BAT-MAG-025, BAT-MAG-048).
                            EventKind::Waited { stack } | EventKind::Defended { stack } | EventKind::StackCast { stack, .. } | EventKind::CreatureRaise { stack, .. } => {
                                prop_assert!(commanded && stack == acting);
                            }
                            // Good morale gives the stack that acted another turn at once
                            // (BAT-MOR-003: `GoodMorale`, then `TurnStarted` for it), but not while
                            // it loses its turns: blind, petrified or paralysed by a retaliation's
                            // spell, it is passed over (BAT-MAG-024). Fear may freeze the turn
                            // (BAT-MAG-060). A turn it takes awaits its side's command, which ends
                            // the command, unless it is still berserk: then the turn is automatic
                            // too (BAT-MAG-025). The round's mark stays until a new round clears
                            // it (BAT-TRN-003).
                            EventKind::GoodMorale { stack } => {
                                prop_assert_eq!(stack, acting);
                                let turn = next(1) == Some(EventKind::TurnStarted { stack });
                                prop_assert_eq!(turn, !stunned(&live, stack));
                                let awaits = turn && next(2) != Some(EventKind::Frightened { stack }) && !berserk(&r, &live, stack);
                                prop_assert!(!awaits || state.active() == Some(stack));
                                let new_round = events.iter().skip(k).any(|e| matches!(e.kind, EventKind::RoundStarted { .. }));
                                prop_assert!(!(awaits && new_round));
                                prop_assert!(state.stacks()[stack.index()].flags.contains(crate::StackFlags::HAD_MORALE) || new_round);
                            }
                            _ => {}
                        }
                    }
                    prop_assert_eq!(due, None, "a berserk stack's turn left no trace");
                    prop_assert!(calmed(&live, attacked), "{attacked:?} attacked and is still berserk");
                    // The events told every change of the effect list (`EffectApplied`,
                    // `EffectEnded`; BAT-MAG-009, BAT-MAG-010) but a death's: a dead stack keeps
                    // only what lasts the whole battle.
                    live.retain(|&(s, spell)| state.stacks()[s.index()].is_alive() || is_permanent(&r, spell));
                    live.sort_unstable();
                    prop_assert_eq!(&live, &tracked(&state));
                    let mentioned = |i: usize, f: fn(&EventKind, usize) -> bool| events.iter().any(|e| f(&e.kind, i));
                    for (i, (b, a)) in before.stacks().iter().zip(state.stacks()).enumerate() {
                        // A created stack, or a stack raised from a corpse, may take a dead
                        // created stack's slot (BAT-MAG-037, BAT-MAG-052).
                        if mentioned(i, |k, i| matches!(k, EventKind::StackCreated { stack, .. } | EventKind::RoseFromCorpse { stack, .. } if stack.index() == i)) {
                            prop_assert!(!b.is_alive() && b.is_created() && a.side == b.side);
                            prop_assert!(a.is_created() != a.has(crate::StackTraits::RAISED));
                            continue;
                        }
                        // Conservation: creatures only ever die, except that life drain and the
                        // raising spells bring back up to the starting count (BAT-DMG-008,
                        // BAT-MAG-035); the creatures raised for the battle only are among them.
                        let drained = mentioned(i, |k, i| matches!(k, EventKind::LifeDrained { stack, .. } | EventKind::SpellRaised { target: stack, .. } | EventKind::Reborn { stack, .. } if stack.index() == i));
                        prop_assert!(a.count <= b.count || (drained && a.count <= a.base_count));
                        let id = StackId(u8::try_from(i).unwrap());
                        prop_assert!(state.temporary(id) <= a.base_count && (state.temporary(id) == before.temporary(id) || drained));
                        prop_assert!(a.is_alive() || a.top_hp == 0);
                        prop_assert!(!a.is_alive() || (1..=a.max_hp).contains(&a.top_hp));
                        // Morale and luck are materialised and change only with the stack's timed
                        // effects (BAT-MAG-011); they stay in range.
                        prop_assert!((a.morale, a.luck) == (b.morale, b.luck) || a.spells != b.spells);
                        prop_assert!((-3..=3).contains(&a.morale) && (0..=3).contains(&a.luck));
                        // Ownership isolation: no command re-stats a stack; a stack is moved only as
                        // a `Moved` or `Teleported` event reports (above: on its own turn, which is
                        // the actor's command or automatic, or by its own side's hero), so the
                        // actor moves no stack of the other side but one it controls; and a stack
                        // of the other side changes only as an event that names it reports: struck
                        // or hurt by a spell, regenerating at the start of its turn
                        // (BAT-TRN-005), and, striking on the actor's command while hypnotized or
                        // on its own while berserk, burnt by a Fire Shield (BAT-MAG-018) or
                        // draining life (BAT-DMG-008).
                        prop_assert_eq!((a.creature, a.side, a.slot, a.attack, a.defense, a.speed), (b.creature, b.side, b.slot, b.attack, b.defense, b.speed));
                        let moved = mentioned(i, |k, i| matches!(k, EventKind::Moved { stack, .. } | EventKind::Teleported { stack, .. } if stack.index() == i));
                        prop_assert!(a.hex == b.hex || moved);
                        if b.side != actor {
                            let explained = mentioned(i, |k, i| matches!(k,
                                EventKind::Struck { target, .. } | EventKind::DeathStared { target, .. } | EventKind::SpellDamage { target, .. } | EventKind::StrikeDamage { target, .. } if target.index() == i))
                                || mentioned(i, |k, i| matches!(k, EventKind::Regenerated { stack, .. } | EventKind::Vanished { stack, .. } | EventKind::Reflected { stack, .. } | EventKind::LifeDrained { stack, .. } if stack.index() == i))
                                || mentioned(i, |k, i| matches!(k, EventKind::Healed { target, .. } if target.index() == i))
                                // The other side's periodic Cure at its caster's activation after
                                // the command (BAT-MAG-053).
                                || mentioned(i, |k, i| matches!(k, EventKind::SpellHealed { target, .. } if target.index() == i))
                                // Poison and Age change the hit points per creature (BAT-MAG-044).
                                || mentioned(i, |k, i| matches!(k, EventKind::Poisoned { stack, .. } | EventKind::EffectEnded { target: stack, .. } if stack.index() == i));
                            if !explained {
                                prop_assert_eq!((a.count, a.top_hp), (b.count, b.top_hp));
                            }
                        }
                    }
                }
            }
            // The effect list (ADR-0014 point 5): sorted by stack and spell, each once, timed
            // spells only, never at 0 rounds, on living stacks and, those without a duration, on
            // the dead stacks that died with them (BAT-MAG-010; review r2a F-3); each stack's
            // values are those of its entries, a dead stack's too (BAT-MAG-009 to 011).
            let effects = state.effects();
            prop_assert!(effects.windows(2).all(|w| (w[0].stack, w[0].spell) < (w[1].stack, w[1].spell)));
            for e in effects {
                prop_assert!(state.stacks()[e.stack.index()].is_alive() || e.rounds.is_none());
                prop_assert!(e.rounds != Some(0));
                prop_assert!(r.spell(e.spell).unwrap().magic.is_timed());
            }
            for i in 0..state.stacks().len() {
                let mut fresh = state.clone();
                crate::effects::recompute(&mut fresh, &r, StackId(u8::try_from(i).unwrap()));
                prop_assert_eq!(fresh.stacks()[i].spells, state.stacks()[i].spells);
            }
            // A battle that is over has no active stack, and vice versa.
            prop_assert_eq!(state.is_over(), state.active().is_none());
            // The battle's end (BAT-END-001, BAT-END-002). While it goes on, each side has a
            // living stack that is no war machine, also after a round's start took one: with
            // quirk BAT-QUIRK-7 a stack that became its own clone vanishes there, perhaps its
            // side's last (BAT-MAG-038; review r4a F-3). Once it ended without a side leaving
            // it, a side has none; the other side has won exactly if a stack of its army lives,
            // not only stacks created in the battle (BAT-MAG-037).
            let fights = |side: BattleSide, army: bool| {
                state.stacks().iter().any(|s| {
                    s.side == side && s.is_alive() && !s.has(crate::StackTraits::WAR_MACHINE) && !(army && s.is_created())
                })
            };
            let wins = |side: BattleSide| !fights(side.opponent(), false) && fights(side, true);
            let [attacker, defender] = BattleSide::BOTH.map(wins);
            match state.result() {
                None => prop_assert!(BattleSide::BOTH.into_iter().all(|side| fights(side, false))),
                Some(BattleResult::Victory(side)) => prop_assert!(wins(side)),
                Some(BattleResult::Draw) => {
                    prop_assert!(BattleSide::BOTH.into_iter().any(|side| !fights(side, false)));
                    prop_assert!(!attacker && !defender);
                }
                Some(BattleResult::Retreat(_) | BattleResult::Surrender(_)) => {}
            }
            // Who commands (BAT-MAG-026): a stack's controller is its side, or the other side
            // exactly while a Hypnotize is active on it; the side to act is the active stack's
            // controller; and the stack that awaits a command is not berserk (BAT-MAG-025: a
            // berserk stack's turns are automatic). Berserk and Hypnotize end each other, and
            // neither takes a war machine.
            let listed = tracked(&state);
            for (i, s) in state.stacks().iter().enumerate() {
                let id = StackId(u8::try_from(i).unwrap());
                prop_assert_eq!(s.controller(), controller_by_rule(&state, &r, id));
                let (wild, led) = (berserk(&r, &listed, id), s.controller() != s.side);
                prop_assert!(!(wild && led), "{id:?} is berserk and hypnotized");
                prop_assert!(!((wild || led) && s.has(crate::StackTraits::WAR_MACHINE)));
            }
            prop_assert_eq!(state.side_to_act(), state.active().map(|id| controller_by_rule(&state, &r, id)));
            prop_assert!(state.active().is_none_or(|id| !berserk(&r, &listed, id)));
            // Living stacks never share a hex (BAT-GEO-002; a clone neither, BAT-QUIRK-7); a side
            // has at most the stack limit of stacks; created stacks leave no corpse
            // (BAT-MAG-037). Rounds are left to exactly the living clones and, with quirk
            // BAT-QUIRK-7, the living stacks that became their own clones: created, no clones,
            // linked to themselves; any other link joins a living clone and its living original
            // (BAT-MAG-038).
            let field = crate::geometry::Field::new(r.battle().field);
            let mut taken = crate::geometry::HexSet::new();
            let links = state.created.links();
            for (i, s) in state.stacks().iter().enumerate() {
                let id = StackId(u8::try_from(i).unwrap());
                if s.is_alive() {
                    for h in s.footprint(field).hexes() {
                        prop_assert!(!taken.contains(h), "two stacks on {h:?}");
                        taken.insert(h);
                    }
                }
                let clone = s.has(crate::StackTraits::CLONE);
                let own = links[i] == Some(id);
                prop_assert_eq!(s.is_alive() && (clone || own), state.clone_rounds(id) > 0);
                prop_assert!(!own || (own_clone && s.is_created() && !clone));
                if let Some(other) = links[i].filter(|_| !own) {
                    let linked = state.stacks()[other.index()];
                    prop_assert_eq!(links[other.index()], Some(id));
                    prop_assert!(s.is_alive() && linked.is_alive() && clone != linked.has(crate::StackTraits::CLONE));
                }
                prop_assert!(s.is_alive() || !s.is_created() || state.corpse(id) == crate::NO_CORPSE);
            }
            for side in BattleSide::BOTH {
                let n = state.stacks().iter().filter(|s| s.side == side).count();
                prop_assert!(n <= usize::from(r.battle().side_stack_limit));
            }
        }
    }

    /// Observation hygiene (ADR-0013 point 9): hidden information does not show. The RNG shows
    /// to neither side nor a spectator; the private part of a side's hero (Tactics level, damage
    /// bonuses, how its war machines work, surrender discount, funds, its hidden magic: BAT-Q26)
    /// only to that side, and so does the spell its random caster drew (ADR-0014 point 8,
    /// BAT-MAG-049); a spectator sees neither side's (SRV-Q4). Of a hero's hidden magic in force
    /// the other side's observation shows nothing but what its own caster may cast (BAT-Q26: it
    /// infers that magic from the casts its commands may not make).
    #[test]
    fn observations_do_not_depend_on_hidden_state(
        (att, def, obstacles, seed, steps, hidden) in arb_game(),
        other_seed: u64,
        damage: [u16; 2],
        (reduction, chance, discount) in (0u16..=10_000, 0u16..=10_000, 0u16..=10_000),
        (shots, heal, funds) in (1u8..=3, any::<u32>(), any::<u32>()),
        (rounds, level, eagle) in (any::<u16>(), 1u8..=5, any::<u8>()),
    ) {
        let r = lively();
        let setup = game_setup(&r, (&att, &def, &obstacles), seed, &hidden);
        let (state, _, _) = play(&r, &setup, seed, &steps);
        let spectator = observe_as(&state, &r, Viewer::Spectator);
        let mut twin = state.clone();
        twin.rng = Rng::from_seed(other_seed);
        for viewer in BattleSide::BOTH {
            prop_assert_eq!(observe(&state, &r, viewer), observe(&twin, &r, viewer));
        }
        prop_assert_eq!(&spectator, &observe_as(&twin, &r, Viewer::Spectator));
        // What goes with the command goes to the side that commands the active stack, whose the
        // stack may not be (BAT-MAG-026): the spell it drew and its cast's candidates
        // (BAT-MAG-049, BAT-MAG-050); the other side, the stack's own while it is hypnotized,
        // sees neither.
        let drawn = state.casts.drawn.filter(|&(s, _)| state.active() == Some(s)).map(|(_, spell)| spell);
        let commander = state.active().map(|id| controller_by_rule(&state, &r, id));
        for viewer in BattleSide::BOTH {
            let seen = observe(&state, &r, viewer);
            let commands = commander == Some(viewer);
            prop_assert_eq!(seen.drawn, drawn.filter(|_| commands));
            prop_assert!(commands || seen.cast_candidates == [0; crate::STACK_CAPACITY]);
        }
        for side in BattleSide::BOTH {
            let mut twin = state.clone();
            for hero in twin.heroes[side.index()].iter_mut() {
                hero.tactics = 3 - hero.tactics;
                hero.damage = DamageBonuses {
                    ranged: Rate::bp(damage[0].into()),
                    melee: Rate::bp(damage[1].into()),
                    reduction: Rate::bp(reduction.into()),
                };
                let ballista = BallistaControl { shots, double_damage_chance_bp: chance };
                hero.ballista = hero.ballista.and(Some(ballista));
                hero.tent = hero.tent.and(Some(TentControl { heal_bonus: Rate::bp(heal )}));
                hero.surrender_discount_bp = discount;
                hero.funds = funds;
                hero.magic.book = SpellSet::default();
                hero.magic.masteries = crate::magic::Masteries::of(&masteries(&r, [3; 4]));
                hero.magic.bonuses.all = Rate::bp(discount.into());
                // The hidden magic no rule reads at this state (BAT-Q26): how long its spells
                // last, its opening spells, its Eagle Eye and what it picked, its stacks'
                // resistance (no cast of the viewer's that an observation counts is hostile).
                let m = &mut hero.magic;
                m.resistance = u64::from(reduction);
                m.duration_bonus = rounds;
                m.opening_due = !m.opening_due;
                m.eagle_chance = eagle;
                m.eagle_spells = SpellSet::default();
                m.learned.insert(SpellId::from_raw(u16::from(level)));
                if m.opening.is_empty() {
                    let opening = crate::Opening { spell: SpellId::from_raw(0), mastery: 3, power: rounds };
                    m.opening.push(opening).unwrap();
                } else {
                    m.opening.clear();
                }
            }
            // covers: BAT-MAG-049
            twin.casts.drawn = twin.casts.drawn.map(|(s, spell)| {
                let other = SpellId::from_raw(spell.raw() ^ 1);
                (s, if twin.at(s).controller() == side { other } else { spell })
            });
            let viewer = side.opponent();
            let seen = observe(&state, &r, viewer);
            prop_assert_eq!(&seen, &observe(&twin, &r, viewer));
            prop_assert_eq!(&spectator, &observe_as(&twin, &r, Viewer::Spectator));
            // The hidden magic in force (BAT-MAG-061 to BAT-MAG-066): the hero's casting limits,
            // its stacks' spell defences and its negation of immunities change, and the battle's
            // limits with them. The other side's observation shows of it only what its own
            // caster may cast at each stack (BAT-MAG-050): the casts its commands may not make
            // (BAT-Q26). A spectator sees none of it.
            for hero in twin.heroes[side.index()].iter_mut() {
                let m = &mut hero.magic;
                m.no_casting = !m.no_casting;
                m.level_limit = level;
                m.defences.up_to_level = Some(level);
                m.defences.mind = !m.defences.mind;
                m.defences.spells.insert(SpellId::from_raw(u16::from(level)));
                m.negates = !m.negates;
            }
            twin.limits = crate::magic::MagicLimits::of(&twin.heroes);
            let mut other = observe(&twin, &r, viewer);
            prop_assert!(commander == Some(viewer) || other.cast_candidates == [0; crate::STACK_CAPACITY]);
            other.cast_candidates = seen.cast_candidates;
            prop_assert_eq!(&seen, &other);
            prop_assert_eq!(&spectator, &observe_as(&twin, &r, Viewer::Spectator));
        }
        // What a spectator sees is what both sides see: a side's view without its private part
        // and its own hidden obstacles the other side does not see (SRV-Q4).
        let mut seat = observe(&state, &r, BattleSide::Attacker);
        seat.spectator = true;
        seat.own_hero = None;
        seat.drawn = None;
        seat.cast_candidates = [0; crate::STACK_CAPACITY];
        seat.spell_obstacles.retain(|o| o.revealed);
        prop_assert_eq!(&spectator, &seat);
    }

    /// Observation hygiene through whole games for the two parts of a hero's hidden magic that
    /// are no limit of a cast (BAT-Q26). Its Eagle Eye never shows (BAT-MAG-069): with a chance
    /// of 0 and of 100 % (no draw either way) every command has the same result and the same
    /// events, the other side and a spectator observe the same, its own side sees only its picks
    /// differ, and with 100 % it has picked every spell the other hero cast. Its opening spells
    /// show only when its side's first activation casts them (BAT-MAG-068: `OpeningSpell` tells
    /// both sides): until then every result and every event is the same with them and without,
    /// and so is what the other side and a spectator observe.
    #[test]
    fn a_heros_eagle_eye_and_unopened_spells_do_not_show(
        (att, def, obstacles, seed, steps, hidden) in arb_game(),
        side in prop_oneof![Just(BattleSide::Attacker), Just(BattleSide::Defender)],
        opening in vec((any::<u16>(), 1u16..4), 1..3),
    ) {
        let r = lively();
        // Both sides are led (bit 44 of the setup's word).
        let setup = game_setup(&r, (&att, &def, &obstacles), seed | 1 << 44, &hidden);
        let viewer = side.opponent();
        let start = |change: &dyn Fn(&mut BattleHero)| {
            let mut setup = setup.clone();
            setup.heroes[side.index()].iter_mut().for_each(change);
            let mut events = Vec::new();
            let state = BattleState::start(&r, &setup, seed, &mut events).unwrap();
            (state, events)
        };
        let mut legal = LegalCommands::new();

        // covers: BAT-MAG-069
        let book = setup.heroes[side.index()].as_ref().and_then(|h| h.magic.spell_book.clone()).unwrap();
        let eagle = |chance: u32| start(&|hero| {
            hero.magic.eagle_eye = Some(EagleEye { chance: Rate::bp(chance), spells: book.clone() });
        });
        let ((mut x, ex), (mut y, ey)) = (eagle(0), eagle(10_000));
        prop_assert_eq!(ex, ey);
        let mut cast = SpellSet::default();
        for step in &steps {
            let (actor, cmd) = resolve(&x, &r, step, &mut legal);
            let (mut ex, mut ey) = (Vec::new(), Vec::new());
            prop_assert_eq!(apply(&mut x, &r, actor, cmd, &mut ex), apply(&mut y, &r, actor, cmd, &mut ey));
            prop_assert_eq!(&ex, &ey);
            for e in &ex {
                if let EventKind::SpellCast { side: caster, spell, .. } = e.kind && caster == viewer {
                    cast.insert(spell);
                }
            }
            prop_assert_eq!(observe(&x, &r, viewer), observe(&y, &r, viewer));
            prop_assert_eq!(observe_as(&x, &r, Viewer::Spectator), observe_as(&y, &r, Viewer::Spectator));
            let (mut own, other) = (observe(&x, &r, side), observe(&y, &r, side));
            if let (Some(a), Some(b)) = (own.own_hero.as_mut(), other.own_hero.as_ref()) {
                (a.learned, a.eagle_chance) = (b.learned, b.eagle_chance);
            }
            prop_assert_eq!(own, other);
        }
        let learned = |state: &BattleState| state.heroes[side.index()].map(|h| h.magic.learned);
        prop_assert_eq!(learned(&x), Some(SpellSet::default()));
        prop_assert_eq!(learned(&y), Some(cast));

        // covers: BAT-MAG-068
        let list = openings(&r);
        let spells: Vec<BattleEffect> = opening.iter().map(|&(i, power)| {
            let (spell, mastery) = list[usize::from(i) % list.len()];
            BattleEffect::OpeningSpell { spell, mastery, power }
        }).collect();
        let opened = |with: bool| start(&|hero| {
            hero.effects.retain(|e| !matches!(e, BattleEffect::OpeningSpell { .. }));
            hero.effects.extend(spells.iter().filter(|_| with).cloned());
        });
        // The events before the hero's first opening spell, and whether it came.
        let before = |events: &[Event]| {
            let cast = events.iter().position(|e| matches!(e.kind, EventKind::OpeningSpell { side: s, .. } if s == side));
            (cast.unwrap_or(events.len()), cast.is_some())
        };
        let ((mut x, mut ex), (mut y, mut ey)) = (opened(true), opened(false));
        let mut steps = steps.iter();
        loop {
            let (k, cast) = before(&ex);
            prop_assert_eq!(Some(&ex[..k]), ey.get(..k));
            if cast {
                break;
            }
            prop_assert_eq!(observe(&x, &r, viewer), observe(&y, &r, viewer));
            prop_assert_eq!(observe_as(&x, &r, Viewer::Spectator), observe_as(&y, &r, Viewer::Spectator));
            let Some(step) = steps.next() else {
                break;
            };
            let (actor, cmd) = resolve(&x, &r, step, &mut legal);
            (ex, ey) = (Vec::new(), Vec::new());
            prop_assert_eq!(apply(&mut x, &r, actor, cmd, &mut ex), apply(&mut y, &r, actor, cmd, &mut ey));
        }
    }

    /// Observation hygiene for hidden obstacles (ADR-0014 point 7, BAT-MAG-031): what a side
    /// does not see changes neither its observation nor what it may do (its legal commands,
    /// castable spells and their targets, its active stack's cast and its targets); nor what
    /// its commands report to it while no hidden obstacle shows itself (a pit it runs into, a
    /// mine that goes off: BAT-MAG-032, BAT-MAG-033; a native it raises gives its side sight
    /// without quirk BAT-QUIRK-1, BAT-MAG-031), its hero's and its stack's casts included:
    /// without quirk BAT-QUIRK-4 a random obstacle spell draws among the hexes its side sees
    /// free (BAT-MAG-032), and Remove Obstacle finds only what the caster sees (BAT-MAG-034).
    /// In four profiles: the natives' sight fixed at the start (BAT-QUIRK-1), anchors
    /// (BAT-QUIRK-3), neither of the two, and the two together, as a pack's faithful profile has
    /// them.
    #[test]
    fn hidden_obstacles_do_not_show(
        (att, def, obstacles, seed, steps, hidden) in arb_game(),
        profile in 0usize..4,
        extra in proptest::collection::btree_set((1u8..16, 0u8..11), 0..6),
    ) {
        let quirks = ["natives_sight_at_start", "obstacle_anchors"];
        let r = match profile {
            0 => lively_with("t:sight"),
            1 => lively_with("t:anchors"),
            2 => lively_with("t:quirky"),
            _ => lively_mix(&quirks),
        };
        // The one property without Berserk in the heroes' books (the others play it). A berserk
        // stack's automatic turn is planned with the pits its own side sees, so its walk, which
        // both sides are told, can show the other side where its own side's hidden pits lie
        // (the E04b T11b review's F-sec-1 part a and r3 F-11; by the reviewers' reading the
        // original does the same). What the engine should do there is open: BAT-Q31.
        let mut setup = game_setup(&r, (&att, &def, &obstacles), seed, &hidden);
        let berserk = r.spell_by_key("t:berserk").unwrap();
        for book in setup.heroes.iter_mut().flatten().filter_map(|h| h.magic.spell_book.as_mut()) {
            book.retain(|&spell| spell != berserk);
        }
        let (mut state, _, _) = play(&r, &setup, seed, &steps);
        // More pits and mines of either side, so that most games have hidden ones.
        let kinds = [("t:quicksand", ObstacleKind::Pit), ("t:landmine", ObstacleKind::Mine)];
        for (i, (x, y)) in extra.into_iter().enumerate() {
            let side = BattleSide::BOTH[i % 2];
            let (key, kind) = kinds[i / 2 % 2];
            let id = r.spell_by_key(key).unwrap();
            let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
            let hex = [Hex(x + 17 * y)];
            let placed = (id, kind, &shape);
            crate::obstacles::place(&mut state, placed, side, (&hex, 40), &mut Vec::new());
        }
        // Observations find the natural obstacle on a hex as Remove Obstacle does (BAT-MAG-034).
        let seen = observe(&state, &r, BattleSide::Attacker);
        for hex in (0..=u8::MAX).map(Hex) {
            prop_assert_eq!(seen.removable_at(hex), state.natural.removable_at(hex));
        }
        // A spectator sees only the hidden obstacles both sides see (SRV-Q4).
        let mut twin = state.clone();
        let both: Vec<_> = crate::obstacles::seen_by(&state, BattleSide::Attacker)
            .into_iter()
            .filter(|o| crate::obstacles::seen_by(&state, BattleSide::Defender).contains(o))
            .collect();
        twin.spell_obstacles.retain(|o| both.contains(o));
        prop_assert_eq!(
            observe_as(&state, &r, Viewer::Spectator),
            observe_as(&twin, &r, Viewer::Spectator)
        );
        for viewer in BattleSide::BOTH {
            let mut twin = state.clone();
            let seen: Vec<_> = crate::obstacles::seen_by(&state, viewer);
            twin.spell_obstacles.retain(|o| seen.contains(o));
            prop_assert_eq!(observe(&state, &r, viewer), observe(&twin, &r, viewer));
            let (mut a, mut b) = (LegalCommands::new(), LegalCommands::new());
            legal_commands(&state, &r, viewer, &mut a);
            legal_commands(&twin, &r, viewer, &mut b);
            prop_assert_eq!(a.as_slice(), b.as_slice());
            let mut commands: Vec<Command> = a.as_slice().to_vec();
            let spells = castable_spells(&state, &r, viewer);
            prop_assert_eq!(spells, castable_spells(&twin, &r, viewer));
            for spell in spells.iter() {
                let (mut ta, mut tb) = (CastTargets::new(), CastTargets::new());
                cast_targets(&state, &r, viewer, spell, &mut ta);
                cast_targets(&twin, &r, viewer, spell, &mut tb);
                prop_assert_eq!(ta.as_slice(), tb.as_slice());
                // Casts too: the first, middle and last target, each with its first second
                // target (legal commands list no casts).
                let n = ta.len();
                let mut picks = std::vec![0, n / 2, n.saturating_sub(1)];
                picks.dedup();
                for target in picks.into_iter().filter_map(|i| ta.get(i).copied()) {
                    let (mut sa, mut sb) = (CastTargets::new(), CastTargets::new());
                    second_targets(&state, &r, viewer, (spell, target), &mut sa);
                    second_targets(&twin, &r, viewer, (spell, target), &mut sb);
                    prop_assert_eq!(sa.as_slice(), sb.as_slice());
                    if let Some(&second) = sa.get(0) {
                        commands.push(Command::Cast { spell, target, second });
                    }
                }
            }
            // And the active stack's cast at each of its targets (BAT-MAG-048 to BAT-MAG-052).
            let (mut ta, mut tb) = (CastTargets::new(), CastTargets::new());
            creature_cast_targets(&state, &r, viewer, &mut ta);
            creature_cast_targets(&twin, &r, viewer, &mut tb);
            prop_assert_eq!(ta.as_slice(), tb.as_slice());
            commands.extend(ta.iter().map(|&target| Command::CreatureCast { target }));
            for &command in &commands {
                let (mut x, mut y) = (state.clone(), twin.clone());
                let (mut ex, mut ey) = (Vec::new(), Vec::new());
                prop_assert!(apply(&mut x, &r, viewer, command, &mut ex).is_ok());
                prop_assert!(apply(&mut y, &r, viewer, command, &mut ey).is_ok());
                if ex.iter().any(|e| matches!(e.kind, EventKind::ObstacleRevealed { .. })) {
                    continue;
                }
                let told = |events: &[Event]| -> Vec<Event> {
                    events.iter().filter_map(|e| project(e, viewer)).collect()
                };
                // A native the command raises gives its side sight again (BAT-MAG-031 without
                // BAT-QUIRK-1): the hidden obstacles it now sees show themselves.
                let unseen: Vec<_> = state
                    .spell_obstacles()
                    .iter()
                    .filter(|o| !seen.contains(o))
                    .map(|o| (o.side, o.hexes))
                    .collect();
                let sight = crate::obstacles::seen_by(&x, viewer);
                if sight.iter().any(|o| unseen.contains(&(o.side, o.hexes))) {
                    continue;
                }
                prop_assert_eq!(told(&ex), told(&ey));
                // A spectator is told the public events only (SRV-Q4).
                let public = |events: &[Event]| -> Vec<Event> {
                    events.iter().filter_map(|e| project_as(e, Viewer::Spectator)).collect()
                };
                let shown: Vec<Event> = ex
                    .iter()
                    .filter(|e| e.visibility == crate::Visibility::Public)
                    .copied()
                    .collect();
                prop_assert_eq!(public(&ex), shown);
                prop_assert_eq!(observe(&x, &r, viewer), observe(&y, &r, viewer));
            }
        }
    }

    /// Clone equivalence: a clone taken mid-game evolves identically to the original.
    #[test]
    fn clones_evolve_identically((att, def, obstacles, seed, steps, hidden) in arb_game(), split in 0usize..160) {
        let r = lively();
        let setup = game_setup(&r, (&att, &def, &obstacles), seed, &hidden);
        let split = split.min(steps.len());
        let (mut original, _, _) = play(&r, &setup, seed, &steps[..split]);
        let mut clone = original.clone();
        let mut legal = LegalCommands::new();
        for step in &steps[split..] {
            let (actor, cmd) = resolve(&original, &r, step, &mut legal);
            let a = apply(&mut original, &r, actor, cmd, &mut crate::NoopSink);
            let b = apply(&mut clone, &r, actor, cmd, &mut crate::NoopSink);
            prop_assert_eq!(a, b);
            prop_assert_eq!(original.state_hash(), clone.state_hash());
        }
        prop_assert_eq!(original, clone);
    }

    /// Determinism: the same inputs give the same events and state hashes.
    #[test]
    fn replays_are_deterministic((att, def, obstacles, seed, steps, hidden) in arb_game()) {
        let r = lively();
        let setup = game_setup(&r, (&att, &def, &obstacles), seed, &hidden);
        let first = play(&r, &setup, seed, &steps);
        let second = play(&r, &setup, seed, &steps);
        prop_assert_eq!(first.1, second.1);
        prop_assert_eq!(first.2, second.2);
        prop_assert_eq!(first.0, second.0);
    }
}

/// Plays `state` on with random legal commands, attacks first, until the battle is over (or a
/// step limit): with `leave` the commands that leave the battle are among them.
fn finish(state: &mut BattleState, r: &Ruleset, seed: u64, leave: bool) {
    let mut agent = Rng::for_stream(seed, h3_kernel::StreamId::AGENT);
    let mut legal = LegalCommands::new();
    for _ in 0..5_000 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        legal_commands(state, r, side, &mut legal);
        let allowed = |c: &&Command| leave || !c.leaves_battle();
        // Prefer attacks so that random games end quickly.
        let attacks: Vec<Command> = legal
            .iter()
            .filter(allowed)
            .copied()
            .filter(|c| matches!(c, Command::Attack { .. }))
            .collect();
        let pool: Vec<Command> = if attacks.is_empty() {
            legal.iter().filter(allowed).copied().collect()
        } else {
            attacks
        };
        let cmd = pool[agent.below(pool.len() as u32) as usize];
        apply(state, r, side, cmd, &mut crate::NoopSink).unwrap();
    }
}

proptest! {
    #![proptest_config(Config { cases: 48, ..Config::default() })]

    /// Conservation with casualties accounted (E04a exit criterion 4, BAT-END-003): a finished
    /// battle's outcome lists every stack once, in stack order per side, with its start and end
    /// counts; the survivors and war machines are exactly the records with creatures left.
    #[test]
    fn outcomes_account_for_every_stack((att, def, obstacles, seed, steps, hidden) in arb_game()) {
        let r = lively();
        let setup = game_setup(&r, (&att, &def, &obstacles), seed, &hidden);
        let (mut state, _, _) = play(&r, &setup, seed, &steps);
        finish(&mut state, &r, seed, false);
        // A random game may never end (no rule ends a battle in which, say, a petrifying striker
        // keeps a stack frozen while a First Aid Tent heals what it deals): it tells nothing
        // about the outcome.
        prop_assume!(state.is_over());
        let o = state.outcome().unwrap();
        prop_assert_eq!(Some(o.result), state.result());
        prop_assert_eq!(o.rounds, state.round());
        for side in BattleSide::BOTH {
            // Every stack is reported in stack order, by origin (BAT-MAG-037, BAT-Q13).
            let stacks: Vec<_> = (0u8..).zip(state.stacks()).filter(|(_, s)| s.side == side).collect();
            let records = &o.stacks[side.index()];
            prop_assert_eq!(records.len(), stacks.len());
            for (record, &(id, s)) in records.iter().zip(&stacks) {
                let temporary = state.temporary(StackId(id));
                prop_assert!(temporary <= s.base_count);
                prop_assert_eq!((record.creature, record.start), (s.creature, s.base_count));
                let origin = if s.is_created() {
                    StackOrigin::Created
                } else if s.has(crate::StackTraits::WAR_MACHINE) {
                    StackOrigin::WarMachine
                } else if s.has(crate::StackTraits::RAISED) {
                    StackOrigin::Raised(s.slot)
                } else {
                    StackOrigin::Army(s.slot)
                };
                prop_assert_eq!(record.origin, origin);
                let army = matches!(origin, StackOrigin::Army(_) | StackOrigin::Raised(_));
                // A created stack that vanished keeps the creatures it vanished with (BAT-Q13).
                let count = s.count.max(state.created.vanished[usize::from(id)]);
                // Creatures raised for the battle only are left out, but an army whose living
                // creatures are all such keeps one in its last living stack (BAT-MAG-035).
                let kept = record.end == 1 && count.saturating_sub(temporary) == 0 && s.is_alive() && army;
                prop_assert!(record.end == count.saturating_sub(temporary) || kept);
                prop_assert!(record.end <= record.start);
            }
            let army = |rec: &&StackRecord| matches!(rec.origin, StackOrigin::Army(_) | StackOrigin::Raised(_));
            let any_army_alive = stacks.iter().any(|(_, s)| s.is_alive() && !s.has(crate::StackTraits::WAR_MACHINE) && !s.is_created());
            prop_assert_eq!(records.iter().filter(army).any(|rec| rec.end > 0), any_army_alive);
            let in_army = |rec: &&StackRecord| rec.end > 0 && army(rec);
            let machine = |rec: &&StackRecord| rec.end > 0 && rec.origin == StackOrigin::WarMachine;
            prop_assert_eq!(o.survivors[side.index()].stacks.len(), records.iter().filter(in_army).count());
            prop_assert_eq!(o.war_machines[side.index()].len(), records.iter().filter(machine).count());
            prop_assert_eq!(o.survivors[side.index()].stacks.iter().map(|s| s.count).sum::<u32>(), records.iter().filter(in_army).map(|rec| rec.end).sum::<u32>());
            // One survivor per army slot, in slot order (a raised stack takes a dead stack's
            // slot, BAT-MAG-052).
            prop_assert!(o.survivors[side.index()].stacks.windows(2).all(|w| w[0].slot < w[1].slot));
        }
    }
}

/// Random legal play always terminates in a decided battle eventually (sanity check of the turn
/// machinery; not a game rule: battles have no round limit).
#[test]
fn random_legal_play_finishes_battles() {
    let r = lively();
    let setup = make_setup(
        &r,
        &[(0, 5, 20), (1, 6, 8), (2, 10, 2)],
        &[(0, 7, 15), (3, 5, 12), (4, 8, 6)],
        &[60, 77, 94],
        0b111_111,
    );
    let mut decided = 0;
    for seed in 0..20u64 {
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        finish(&mut state, &r, seed, true);
        decided += usize::from(state.is_over());
    }
    assert_eq!(decided, 20);
}

/// The Ballista's target by the full ranking of BAT-WAR-002 (VCMI's order, every key evaluated
/// eagerly): not a war machine, then could attack next turn, then value, then the lower id.
fn reference_target(state: &BattleState, r: &Ruleset, shooter: StackId) -> Option<StackId> {
    let field = crate::Field::new(r.battle().field);
    let side = state.stacks()[shooter.index()].side;
    (0u8..)
        .zip(state.stacks())
        .filter(|(_, s)| s.is_alive() && s.side != side)
        .map(|(i, s)| {
            let id = StackId(i);
            let creature = !s.has(crate::StackTraits::WAR_MACHINE);
            let dangerous = crate::war::can_attack_next_turn(state, field, id);
            (
                id,
                creature,
                dangerous,
                crate::war::value(state, r, shooter, id),
            )
        })
        .max_by(|a, b| {
            (a.1, a.2)
                .cmp(&(b.1, b.2))
                .then_with(|| h3_kernel::arith::cmp_fractions(a.3, b.3))
                .then_with(|| b.0.cmp(&a.0))
        })
        .map(|c| c.0)
}

proptest! {
    #![proptest_config(Config { cases: 32, ..Config::default() })]

    /// The lazy target choice of the Ballista (BAT-WAR-002) equals the eager reference ranking
    /// in every state of random games with ballistae on both sides.
    #[test]
    fn the_lazy_ballista_target_equals_the_reference((att, def, obstacles, seed, steps, hidden) in arb_game()) {
        let r = lively();
        // Ballistae on both sides (bits 0 and 3), the other machines and formations at random.
        let setup = game_setup(&r, (&att, &def, &obstacles), seed | 0b1001, &hidden);
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        let mut legal = LegalCommands::new();
        let field = crate::Field::new(r.battle().field);
        for step in &steps {
            for (i, s) in (0u8..).zip(state.stacks()) {
                if s.is_alive() && s.has(crate::StackTraits::WAR_MACHINE) && s.has(crate::StackTraits::SHOOTER) {
                    let id = StackId(i);
                    prop_assert_eq!(
                        crate::war::shooting_target(&state, &r, field, id),
                        reference_target(&state, &r, id)
                    );
                }
            }
            let (actor, cmd) = resolve(&state, &r, step, &mut legal);
            let _ = apply(&mut state, &r, actor, cmd, &mut crate::NoopSink);
        }
    }
}

/// One effect of an extreme hero: kind, selector (all, a flag, a creature), statistic,
/// operation, amount (also the morale/luck value and the regeneration), affected sides.
type ExtremeEffect = (u8, u8, usize, u8, i16, u8);

/// A battle effect from its generated parts; every kind except `NoRetreat` and the alignment
/// groups (both covered by `make_setup` and `tests/heroes.rs`).
fn extreme_effect(
    r: &Ruleset,
    &(kind, selector, creature, stat, amount, affects): &ExtremeEffect,
) -> BattleEffect {
    let creatures = match selector {
        0 => CreatureSelector::All,
        1 => CreatureSelector::Flag(if creature % 2 == 0 {
            CreatureFlag::Alive
        } else {
            CreatureFlag::Shooter
        }),
        _ => CreatureSelector::Creature(
            r.creature_by_key(&std::format!(
                "t:{}",
                CREATURES[creature % CREATURES.len()].0
            ))
            .unwrap(),
        ),
    };
    let stat = CreatureStat::ALL[usize::from(stat) % CreatureStat::ALL.len()];
    let affects = Affects::ALL[usize::from(affects) % 3];
    let op = [StatOp::Add, StatOp::PercentBp, StatOp::PercentBpDown][usize::from(kind) % 3];
    let value = i8::try_from(amount.clamp(i8::MIN.into(), i8::MAX.into())).unwrap();
    match kind {
        0..=2 => BattleEffect::StackStat {
            creatures,
            stat,
            op,
            amount,
        },
        3 => BattleEffect::Regeneration {
            creatures,
            hit_points: amount.unsigned_abs(),
        },
        4 => BattleEffect::NoRangePenalty { creatures },
        5 => BattleEffect::ShootWhenAdjacent { creatures },
        6 => BattleEffect::Morale {
            amount: value,
            affects,
        },
        7 => BattleEffect::Luck {
            amount: value,
            affects,
        },
        8 => BattleEffect::MoraleAtMost { value, affects },
        _ => BattleEffect::LuckAtMost { value, affects },
    }
}

/// A sound hero with values no real hero could have (BAT-HER-001, OPEN-QUESTIONS Q31): any
/// primary skills and damage bonuses, any Ballista and First Aid Tent control, any surrender
/// discount, any funds and up to 24 effects with extreme amounts.
fn arb_extreme_hero() -> impl Strategy<Value = Option<(BattleHero, u32)>> {
    let effect = (
        0u8..10,
        0u8..3,
        0usize..64,
        any::<u8>(),
        any::<i16>(),
        any::<u8>(),
    );
    proptest::option::weighted(
        0.9,
        (
            any::<[u8; 4]>(),
            0u8..=3,
            (any::<u16>(), any::<u16>(), 0u16..=10_000),
            proptest::option::of((1u8..=255, 0u16..=10_000)),
            proptest::option::of(any::<u32>()),
            0u16..=10_000,
            vec(effect, 0..24),
            any::<u32>(),
        ),
    )
    .prop_map(|raw| {
        raw.map(
            |(
                primary,
                tactics,
                (ranged_bp, melee_bp, reduction_bp),
                ballista,
                tent,
                discount,
                effects,
                funds,
            )| {
                let r = lively();
                let mut h = hero(&r);
                [h.attack, h.defense, h.power, h.knowledge] = primary;
                h.tactics = tactics;
                h.damage = DamageBonuses {
                    ranged: Rate::bp(ranged_bp.into()),
                    melee: Rate::bp(melee_bp.into()),
                    reduction: Rate::bp(reduction_bp.into()),
                };
                h.war_machines.ballista =
                    ballista.map(|(shots, double_damage_chance_bp)| BallistaControl {
                        shots,
                        double_damage_chance_bp,
                    });
                h.war_machines.tent = tent.map(|bp| TentControl {
                    heal_bonus: Rate::bp(bp),
                });
                h.surrender_discount_bp = discount;
                h.effects = effects.iter().map(|e| extreme_effect(&r, e)).collect();
                (h, funds)
            },
        )
    })
}

/// Armies with counts up to `u32::MAX` (above the death stare's roll limit too, KER-RNG-003).
fn arb_big_army() -> impl Strategy<Value = ArmySpec> {
    let pool = army_creatures();
    let count = prop_oneof![1u32..100, 60_000u32..200_000, 1u32..=u32::MAX];
    btree_map(0u8..7, (0..pool.len(), count), 1..=7).prop_map(move |m| {
        m.into_iter()
            .map(|(slot, (c, n))| (slot, pool[c], n))
            .collect()
    })
}

proptest! {
    #![proptest_config(Config { cases: 48, ..Config::default() })]

    // covers: BAT-HER-001
    /// Robustness (E04a T12c review): battles led on both sides by sound heroes with extreme
    /// values (BAT-HER-001 accepts them) and armies of up to `u32::MAX` creatures start, play
    /// with every legal command accepted (leaving in a quarter of them), and report an outcome when over;
    /// no arithmetic overflows.
    #[test]
    fn extreme_heroes_and_counts_play_without_failures(
        att in arb_big_army(),
        def in arb_big_army(),
        heroes in (arb_extreme_hero(), arb_extreme_hero()),
        machines in any::<u64>(),
        seed in any::<u64>(),
    ) {
        let r = lively();
        let mut setup = make_setup(&r, &att, &def, &[], machines);
        let [a, d] = [heroes.0, heroes.1];
        setup.funds = [a.as_ref().map_or(0, |h| h.1), d.as_ref().map_or(0, |h| h.1)];
        setup.heroes = [a.map(|h| h.0), d.map(|h| h.0)];
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        // A quarter of the games may leave the battle (random agents leave early).
        finish(&mut state, &r, seed, seed % 4 == 0);
        for side in BattleSide::BOTH {
            let _ = observe(&state, &r, side);
        }
        prop_assert_eq!(state.outcome().is_some(), state.is_over());
    }
}

/// One of `items`, uniformly.
fn one_of<T: Copy>(rng: &mut Rng, items: &[T]) -> T {
    items[rng.below(items.len() as u32) as usize]
}

/// A rate of an extreme casting hero: any basis points and, one time in three, a
/// single-precision value of its own with any bit pattern (infinities, NaNs and negative values
/// among them: with quirk Q-0005 the battle reads it, OPEN-QUESTIONS Q45).
fn extreme_rate(rng: &mut Rng) -> Rate {
    let bp = match rng.below(6) {
        0 => 0,
        1 => 500,
        2 => 65_535,
        3 => u32::MAX,
        4 => rng.below(100_000),
        _ => rng.next_u32(),
    };
    let single = match rng.below(6) {
        0 => NonZeroU32::new(rng.next_u32()),
        1 => NonZeroU32::new(one_of(
            rng,
            &[0x7f80_0000, 0x7fc0_0000, 0xff80_0000, 1, 0x7f7f_ffff],
        )),
        _ => None,
    };
    Rate { bp, single }
}

/// A sound casting hero with values no real hero could have (BAT-HER-001): any primary skills,
/// mana and damage bonuses, a book of the hero spells of the test ruleset with any masteries,
/// sometimes Eagle Eye, and up to 40 effects of every kind but `NoRetreat` and the alignment
/// groups, the magic ones (E04b) with extreme rates, levels and powers (the spell damage
/// bonuses too, whatever the ruleset's rates: the spell damage saturates since the E04b T11b
/// review's finding F-robust-3 was fixed).
fn extreme_caster(r: &Ruleset, rng: &mut Rng) -> BattleHero {
    let spells: Vec<SpellId> = (0..r.spells().len())
        .filter_map(SpellId::from_index)
        .collect();
    let hero_spells: Vec<SpellId> = spells
        .iter()
        .copied()
        .filter(|&s| {
            r.spell(s)
                .is_some_and(|t| t.category != SpellCategory::Ability)
        })
        .collect();
    let openings = openings(r);
    let mut h = hero(r);
    [h.attack, h.defense, h.power, h.knowledge] =
        [0; 4].map(|_| one_of(rng, &[0, 1, 7, 30, 99, 255]));
    h.tactics = one_of(rng, &[0, 0, 0, 0, 1, 2, 3]);
    h.damage = DamageBonuses {
        ranged: extreme_rate(rng),
        melee: extreme_rate(rng),
        reduction: Rate::bp(rng.below(10_001)),
    };
    if rng.below(3) == 0 {
        h.war_machines.ballista = Some(BallistaControl {
            shots: one_of(rng, &[1, 2, 255]),
            double_damage_chance_bp: one_of(rng, &[0, 5000, 10_000]),
        });
    }
    if rng.below(3) == 0 {
        h.war_machines.tent = Some(TentControl {
            heal_bonus: extreme_rate(rng),
        });
    }
    h.surrender_discount_bp = one_of(rng, &[0, 2000, 10_000]);
    // One book in eight lacks half its spells; the masteries cover the abilities too.
    let all = rng.below(8) != 0;
    h.magic.spell_book = Some(
        hero_spells
            .iter()
            .copied()
            .filter(|_| all || rng.below(2) == 0)
            .collect(),
    );
    h.magic.masteries = spells
        .iter()
        .filter_map(|&s| {
            let mastery = u8::try_from(rng.below(4)).unwrap();
            (mastery > 0).then_some((s, mastery))
        })
        .collect();
    h.magic.mana = one_of(rng, &[3, 20, 200, 5000, u16::MAX]);
    h.magic.max_mana = one_of(rng, &[0, 10, 500, u16::MAX]);
    if rng.below(3) == 0 {
        h.magic.eagle_eye = Some(EagleEye {
            chance: Rate::bp(rng.below(10_001)),
            spells: hero_spells
                .iter()
                .copied()
                .filter(|_| rng.below(2) == 0)
                .collect(),
        });
    }
    let level = |rng: &mut Rng| one_of(rng, &[0u8, 1, 2, 3, 4, 5, 6, 100, 255]);
    // The spells with bonuses of their own (at most `MAX_SPELL_BONUSES`).
    let own: Vec<SpellId> = (0..crate::MAX_SPELL_BONUSES)
        .map(|_| one_of(rng, &spells))
        .collect();
    let mut opened = 0;
    h.effects.clear();
    for _ in 0..rng.below(41) {
        let effect = match rng.below(20) {
            0 | 1 => BattleEffect::SpellDamage {
                scope: SpellScope::All,
                rate: extreme_rate(rng),
            },
            2 | 3 => BattleEffect::SpellDamage {
                scope: SpellScope::School(
                    SchoolId::from_index(rng.below(r.spell_schools().len() as u32) as usize)
                        .unwrap(),
                ),
                rate: extreme_rate(rng),
            },
            4 => BattleEffect::SpellDamage {
                scope: SpellScope::Spell(one_of(rng, &own)),
                rate: extreme_rate(rng),
            },
            5 | 6 => BattleEffect::SpellSpecialty {
                spell: one_of(rng, &own),
                bp_per_level: one_of(rng, &[0, 1, 300, 500, 10_000, u16::MAX]),
                level: one_of(rng, &[0, 1, 5, 30, 74, 255]),
            },
            7 | 8 => BattleEffect::MagicResistance {
                rate: extreme_rate(rng),
            },
            9 => BattleEffect::SpellDuration {
                rounds: one_of(rng, &[0, 1, 3, 56, 255]),
            },
            // Rare: with it no hero casts (BAT-MAG-061).
            10 if rng.below(25) == 0 => BattleEffect::NoCasting,
            10 => BattleEffect::SpellLevelLimit {
                max_level: level(rng),
            },
            11 => BattleEffect::SpellImmunity {
                spell: one_of(rng, &spells),
            },
            12 if rng.below(3) == 0 => BattleEffect::SpellLevelImmunity {
                max_level: level(rng),
            },
            12 => BattleEffect::MindImmunity,
            13 => BattleEffect::NegateImmunities,
            14 | 15 if opened < crate::MAX_OPENING_SPELLS => {
                opened += 1;
                let (spell, mastery) = one_of(rng, &openings);
                BattleEffect::OpeningSpell {
                    spell,
                    mastery,
                    power: one_of(rng, &[1, 2, 8, 255, 256, 30_000, u16::MAX]),
                }
            }
            _ => extreme_effect(
                r,
                &(
                    u8::try_from(rng.below(10)).unwrap(),
                    u8::try_from(rng.below(3)).unwrap(),
                    rng.below(64) as usize,
                    u8::try_from(rng.below(256)).unwrap(),
                    one_of(rng, &[i16::MIN, -100, -1, 0, 1, 10, 1000, i16::MAX]),
                    u8::try_from(rng.below(3)).unwrap(),
                ),
            ),
        };
        h.effects.push(effect);
    }
    h
}

/// Plays `state` on for at most 400 steps with commands the battle lists: half the time the
/// hero's cast if a spell is castable (a spell of `castable_spells` at one of its `cast_targets`
/// and `second_targets`: each list has an entry for a castable spell, BAT-MAG-001), else often
/// the active stack's cast if it has a target (BAT-MAG-048), else a legal command (a strike one
/// time in four; with `leave` the commands that leave the battle too). Every command is accepted.
/// Returns the numbers of the heroes' and the stacks' casts.
fn play_casting(state: &mut BattleState, r: &Ruleset, seed: u64, leave: bool) -> (u32, u32) {
    let mut agent = Rng::for_stream(seed, h3_kernel::StreamId::AGENT);
    let mut legal = LegalCommands::new();
    let mut targets = CastTargets::new();
    let mut seconds = CastTargets::new();
    let mut casts = (0, 0);
    for _ in 0..400 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        let spells: Vec<SpellId> = castable_spells(state, r, side).iter().collect();
        creature_cast_targets(state, r, side, &mut targets);
        let roll = agent.below(100);
        let cmd = if !spells.is_empty() && roll < 50 {
            let spell = one_of(&mut agent, &spells);
            cast_targets(state, r, side, spell, &mut targets);
            assert!(!targets.is_empty(), "castable {spell:?} without a target");
            let target = one_of(&mut agent, targets.as_slice());
            second_targets(state, r, side, (spell, target), &mut seconds);
            assert!(
                !seconds.is_empty(),
                "{spell:?} at {target:?} without a second target"
            );
            casts.0 += 1;
            Command::Cast {
                spell,
                target,
                second: one_of(&mut agent, seconds.as_slice()),
            }
        } else if !targets.is_empty() && roll < 85 {
            casts.1 += 1;
            Command::CreatureCast {
                target: one_of(&mut agent, targets.as_slice()),
            }
        } else {
            legal_commands(state, r, side, &mut legal);
            let allowed = |c: &&Command| leave || !c.leaves_battle();
            let attacks: Vec<Command> = legal
                .iter()
                .filter(allowed)
                .copied()
                .filter(|c| matches!(c, Command::Attack { .. } | Command::Shoot { .. }))
                .collect();
            // Mostly no strike, so that the battles last and the casts are many.
            let pool: Vec<Command> = if attacks.is_empty() || agent.below(4) != 0 {
                legal.iter().filter(allowed).copied().collect()
            } else {
                attacks
            };
            one_of(&mut agent, &pool)
        };
        apply(state, r, side, cmd, &mut crate::NoopSink)
            .unwrap_or_else(|e| panic!("seed {seed}: the listed {cmd:?} is rejected: {e:?}"));
    }
    casts
}

// covers: BAT-HER-001, BAT-MAG-001, BAT-MAG-003, BAT-MAG-048
/// Robustness (E04b T11b review): battles led on both sides by sound casting heroes with extreme
/// values (BAT-HER-001 accepts them) start and play with the heroes' and the stacks' casts among
/// the commands, every listed command accepted, with armies of up to `u32::MAX` creatures, under
/// exact rates and under the original's quirks (single-precision rates, Q-0005); no arithmetic
/// overflows; an outcome when over. Fixed seeds: a failure reproduces.
///
/// Two findings of that review bounded it. The spell damage saturates now (F-robust-3, also
/// through a Fire Shield's reflection of a huge strike), so the armies and the spell damage
/// bonuses are the same in both profiles. One still bounds it, to be widened once it is fixed:
/// the natural obstacles are anchored on their own hex, so that no removal
/// leaves another without a hex (F-robust-2).
#[test]
fn extreme_casting_heroes_play_without_failures() {
    let pool = army_creatures();
    // The creatures that cast on command or raise (BAT-MAG-048 to BAT-MAG-052).
    let casters: Vec<u8> = pool
        .iter()
        .copied()
        .filter(|&c| {
            CREATURES[usize::from(c)]
                .7
                .iter()
                .any(|t| t.contains("spell_caster") || t.contains("raiser"))
        })
        .collect();
    let mut casts = (0, 0);
    for profile in ["t:quirky", "t:original"] {
        let r = lively_with(profile);
        for seed in 0..100u64 {
            let mut rng = Rng::from_seed(seed);
            let army = |rng: &mut Rng| -> ArmySpec {
                let mut slots: Vec<u8> = (0..7).collect();
                (0..=rng.below(7))
                    .map(|_| {
                        let slot = slots.remove(rng.below(slots.len() as u32) as usize);
                        let count = match rng.below(4) {
                            0 => 1 + rng.below(100),
                            1 => 60_000 + rng.below(140_000),
                            2 => 1 + rng.below(u32::MAX),
                            _ => u32::MAX,
                        };
                        let creature = if rng.below(3) == 0 {
                            one_of(rng, &casters)
                        } else {
                            one_of(rng, &pool)
                        };
                        (slot, creature, count)
                    })
                    .collect()
            };
            let (att, def) = (army(&mut rng), army(&mut rng));
            let mut setup = make_setup(&r, &att, &def, &[], rng.next_u64());
            // Up to four one-hex obstacles between the armies, each anchored on its own hex.
            setup.obstacles = pieces(&[58, 76, 94, 112][..rng.below(5) as usize]);
            setup.heroes = [
                Some(extreme_caster(&r, &mut rng)),
                (rng.below(8) != 0).then(|| extreme_caster(&r, &mut rng)),
            ];
            setup.funds = [rng.next_u32(), one_of(&mut rng, &[0, 1000, u32::MAX])];
            let mut state = BattleState::new(&r, &setup, seed).unwrap();
            // One game in eight may leave the battle.
            let (hero, stack) = play_casting(&mut state, &r, seed, seed % 8 == 7);
            casts = (casts.0 + hero, casts.1 + stack);
            for side in BattleSide::BOTH {
                let _ = observe(&state, &r, side);
            }
            assert_eq!(state.outcome().is_some(), state.is_over());
        }
    }
    // The battles did cast (about a thousand casts of the heroes and four hundred of the stacks).
    assert!(casts.0 > 500 && casts.1 > 200, "{casts:?}");
}
