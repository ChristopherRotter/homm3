//! Tests of the simulated combat (E31 T2b, T2c; AI-Q34): a side built from its army, its hero
//! and the opponent (the factor, Tactics, the walls, the bonuses, the shares, the order, the
//! casting inputs) on the crate's test ruleset with fight values and AI kinds; the losses, the
//! damage, the stand-off, the lookahead and the run on sides built by hand; the casts (the
//! choice, the damage spells with their chains and areas, the mass damage and its carried
//! totals, the other spells and the value-per-hit-point slip, the raises). Their numbers come
//! from an IEEE double model of the reading (each case's comment gives its sums; `E31-log.md`
//! T2b, T2c).

use std::vec;
use std::vec::Vec;

use h3_content::{CreatureStat, HeroTypeId, SpellScope};
use h3_content_battle::{AbilityDef, SpellAi, SpellAiKind, SpellAiKinds};
use h3_contracts::{BattleMagic, CatapultControl, Rate};
use proptest::prelude::*;

use super::*;
use crate::tests::support::ruleset_from;

/// The kinds of `kinds` as a set.
pub(super) fn kinds(kinds: &[SpellAiKind]) -> SpellAiKinds {
    kinds
        .iter()
        .fold(SpellAiKinds::default(), |set, &k| set.with(k))
}

/// The crate's test ruleset (`t:faithful`) with Fight Values (`fixed` 100, `slow` 50, `tank`
/// 1 000, `archer` 150, `sniper` 400, `flyer` 120, `dwarf` 200, `golem` 300, `ember` 120,
/// `troglo` 80, `ghoul` 60, `pegasus` 90, `magus` 110, `familiar` 30), `no_wall_penalty` on the
/// sniper and the original's AI kinds on the spells the casts read. For the caster record's
/// tests (`caster/tests.rs`, E31 T2e) also AI values (`fixed` 7, `slow` 3, `tank` 50, `archer`
/// 11, `dwarf` 20, `wyrm` 10; the others 0) and `quake` free.
pub(super) fn rules() -> BattleRuleset {
    static RULES: std::sync::OnceLock<BattleRuleset> = std::sync::OnceLock::new();
    RULES.get_or_init(build_rules).clone()
}

/// [`rules`] compiled.
fn build_rules() -> BattleRuleset {
    ruleset_from(
        "t:faithful",
        |_| {},
        |core, battle| {
            let fights = [
                ("fixed", 100),
                ("slow", 50),
                ("tank", 1000),
                ("archer", 150),
                ("sniper", 400),
                ("flyer", 120),
                ("dwarf", 200),
                ("golem", 300),
                ("ember", 120),
                ("troglo", 80),
                ("ghoul", 60),
                ("pegasus", 90),
                ("magus", 110),
                ("familiar", 30),
            ];
            let ai = [
                ("fixed", 7),
                ("slow", 3),
                ("tank", 50),
                ("archer", 11),
                ("dwarf", 20),
                ("wyrm", 10),
            ];
            for def in &mut core.creatures {
                if let Some(&(_, v)) = fights.iter().find(|(n, _)| def.key.name() == *n) {
                    def.info.fight_value = v;
                }
                if let Some(&(_, v)) = ai.iter().find(|(n, _)| def.key.name() == *n) {
                    def.info.ai_value = v;
                }
            }
            for def in &mut core.spells {
                if def.key.name() == "quake" {
                    def.cost = [0; 4];
                }
            }
            for def in &mut battle.creatures {
                if def.key.name() == "sniper" {
                    def.abilities.push(AbilityDef::NoWallPenalty);
                }
            }
            use SpellAiKind::{Damage, DamageAll, DamageOnce, Effect, Special, Stacks};
            let table: [(&str, &[SpellAiKind], [u16; 4]); 15] = [
                ("bolt", &[Damage], [1; 4]),
                ("smite", &[Damage, Special], [1; 4]),
                ("chain", &[DamageOnce], [1; 4]),
                ("nova", &[Damage], [1; 4]),
                ("ring", &[Damage], [1; 4]),
                ("storm", &[DamageAll], [1; 4]),
                ("bless", &[Effect], [2, 2, 4, 4]),
                ("curse", &[Effect], [2, 2, 4, 4]),
                ("airward", &[Effect], [10, 10, 15, 20]),
                ("dispel", &[Effect], [10, 10, 20, 30]),
                ("cure", &[Effect], [5, 5, 10, 10]),
                ("rise", &[Stacks], [50, 50, 60, 120]),
                ("animate", &[Stacks], [60, 60, 60, 120]),
                ("offering", &[Stacks], [1; 4]),
                ("quake", &[Special], [1; 4]),
            ];
            for def in &mut battle.spells {
                if let Some(&(_, k, values)) = table.iter().find(|(n, _, _)| def.key.name() == *n) {
                    def.ai = SpellAi {
                        kinds: kinds(k),
                        values,
                    };
                }
            }
        },
    )
}

pub(super) fn creature(r: &BattleRuleset, name: &str) -> CreatureTypeId {
    r.creature_by_key(&std::format!("t:{name}")).unwrap()
}

pub(super) fn spell(r: &BattleRuleset, name: &str) -> SpellId {
    r.spell_by_key(&std::format!("t:{name}")).unwrap()
}

/// `(slot, creature, count)` as army stacks.
pub(super) fn army(r: &BattleRuleset, spec: &[(u8, &str, u32)]) -> Vec<ArmyStack> {
    spec.iter()
        .map(|&(slot, name, count)| ArmyStack {
            slot,
            creature: creature(r, name),
            count,
        })
        .collect()
}

/// A hero with no skills.
pub(super) fn bare_hero() -> BattleHero {
    BattleHero::new(HeroTypeId::from_index(0).unwrap())
}

/// A hero with Spell Power `power`, `mana` spell points and a spell book of `spells` at their
/// masteries.
pub(super) fn caster(r: &BattleRuleset, power: u8, mana: u16, spells: &[(&str, u8)]) -> BattleHero {
    let mut hero = bare_hero();
    hero.power = power;
    hero.magic = BattleMagic {
        spell_book: Some(spells.iter().map(|&(s, _)| spell(r, s)).collect()),
        masteries: spells
            .iter()
            .filter(|&&(_, m)| m > 0)
            .map(|&(s, m)| (spell(r, s), m))
            .collect(),
        mana,
        max_mana: mana,
        eagle_eye: None,
    };
    hero
}

/// The hero's book as its available spells.
pub(super) fn book(hero: &BattleHero) -> Vec<SpellId> {
    hero.magic.spell_book.clone().unwrap_or_default()
}

/// `v` as an x87 value exactly: a numerator over a power of two.
fn exact(num: u32, den: u32) -> X87 {
    X87::ratio(num, den, DOUBLE_BITS).unwrap()
}

fn fifth() -> X87 {
    exact(1, 5)
}

/// A creature of none of the test ruleset's rules (the hand-built sides cast nothing).
fn nobody() -> CreatureTypeId {
    CreatureTypeId::from_index(0).unwrap()
}

/// The shares of a walker (0.2 of melee, 1 of the stand-off, no shots).
fn walker(slot: u8, count: i32, speed: i32, contact: i32, unit: i32) -> Stack {
    Stack {
        slot,
        creature: nobody(),
        count,
        start: count,
        speed,
        melee: fifth(),
        hold: X87::ONE,
        shot: X87::ZERO,
        per_hp: SignedX87::ZERO,
        contact,
        unit,
        value: unit.wrapping_mul(count),
    }
}

/// A shooter without a hero (0.1 of melee, 0.7 of the stand-off, 0.2 of shots).
fn shooter(slot: u8, count: i32, speed: i32, unit: i32) -> Stack {
    Stack {
        melee: exact(1, 10),
        hold: exact(7, 10),
        shot: fifth(),
        ..walker(slot, count, speed, 0, unit)
    }
}

/// A side of `stacks` as the build leaves it without a hero: by unit value, ties in slot order,
/// the strength their sum.
fn side_of(mut stacks: Vec<Stack>) -> Side {
    let _ = sort_by_unique_key(&mut stacks, |s| (s.unit, s.slot));
    let strength = stacks.iter().map(|s| s.value).sum();
    Side {
        stacks,
        strength,
        mana: 0,
        data: Arc::new(Data {
            hero: None,
            may_cast: false,
            level: u8::MAX,
            ground: None,
            army: Vec::new(),
        }),
    }
}

/// The stacks' (slot, count, value), in the side's order.
fn left(side: &Side) -> Vec<(u8, i32, i32)> {
    side.stacks
        .iter()
        .map(|s| (s.slot, s.count, s.value))
        .collect()
}

/// The stacks' (slot, speed, contact, unit, value), in the side's order.
fn built(side: &Side) -> Vec<(u8, i32, i32, i32, i32)> {
    side.stacks
        .iter()
        .map(|s| (s.slot, s.speed, s.contact, s.unit, s.value))
        .collect()
}

/// A side of `army` without a hero, on ordinary ground.
fn plain(r: &BattleRuleset, army: &[ArmyStack]) -> Side {
    side(
        r,
        army,
        X87::ONE,
        (None, None),
        FortLevel::None,
        Ground::default(),
    )
}

/// A side of `army` with `hero` and its book, against no hero, on `ground`.
fn led(r: &BattleRuleset, army: &[ArmyStack], hero: &BattleHero, ground: Ground) -> Side {
    let spells = book(hero);
    let own = EstimateHero {
        hero,
        spells: &spells,
    };
    side(
        r,
        army,
        X87::ONE,
        (Some(own), None),
        FortLevel::None,
        ground,
    )
}

// --- the side (CPU-SIM-001) ---

// covers: CPU-SIM-001
#[test]
fn a_side_without_heroes_takes_the_fight_values_and_the_plain_shares() {
    let r = rules();
    let army = army(
        &r,
        &[
            (0, "fixed", 10),
            (2, "archer", 4),
            (1, "tank", 3),
            (3, "sniper", 2),
            (5, "slow", 20),
        ],
    );
    let side = plain(&r, &army);
    // The slow (speed 2: 16 / 2 = 8, at most 4), the fixed ((5 + 14) / 5 = 3), the archer and
    // the sniper (shooters), the tank ((4 + 14) / 4 = 4): weakest unit first; slot 4 is empty.
    assert_eq!(
        built(&side),
        vec![
            (5, 2, 4, 50, 1000),
            (0, 5, 3, 100, 1000),
            (2, 4, 0, 150, 600),
            (3, 4, 0, 400, 800),
            (1, 4, 4, 1000, 3000)
        ]
    );
    assert_eq!(side.strength(), 6400);
    let shares: Vec<_> = side
        .stacks
        .iter()
        .map(|s| (s.melee, s.hold, s.shot))
        .collect();
    // A shooter with the melee penalty 0.1 and 0.7; the sniper without it, striking twice.
    assert_eq!(shares[2], (exact(1, 10), exact(7, 10), fifth()));
    assert_eq!(shares[3], (fifth(), X87::ONE, fifth().plus(fifth())));
    assert_eq!(shares[0], (fifth(), X87::ONE, X87::ZERO));
    // A value per hit point: the unit value over the hit points (the tank's 1 000 / 100).
    assert_eq!(side.stacks[4].per_hp, int(10));
    assert_eq!(side.stacks[4].start, 3);
    // No hero: no spell points, no casting.
    assert_eq!((side.mana, side.data.may_cast), (0, false));
}

/// The hero of the skills case: Attack 5, Defense 3, Tactics 2, +2 hit points and +1 speed to
/// every creature, Archery's 25 %.
fn skilled() -> BattleHero {
    let mut hero = bare_hero();
    hero.attack = 5;
    hero.defense = 3;
    hero.tactics = 2;
    hero.damage.ranged = Rate {
        bp: 2500,
        single: None,
    };
    hero.effects = vec![
        BattleEffect::StackStat {
            creatures: CreatureSelector::All,
            stat: CreatureStat::HitPoints,
            op: StatOp::Add,
            amount: 2,
        },
        BattleEffect::StackStat {
            creatures: CreatureSelector::All,
            stat: CreatureStat::Speed,
            op: StatOp::Add,
            amount: 1,
        },
    ];
    hero
}

// covers: CPU-SIM-001
#[test]
fn a_heros_skills_bonuses_and_tactics_scale_its_side() {
    let r = rules();
    let army = army(&r, &[(0, "fixed", 10), (1, "archer", 4), (2, "tank", 3)]);
    let hero = skilled();
    let mut opponent = bare_hero();
    opponent.attack = 1;
    opponent.defense = 1;
    let heroes = (
        Some(EstimateHero {
            hero: &hero,
            spells: &[],
        }),
        Some(EstimateHero {
            hero: &opponent,
            spells: &[],
        }),
    );
    let side = side(
        &r,
        &army,
        X87::ONE,
        heroes,
        FortLevel::None,
        Ground::default(),
    );
    // The factor sqrt(1 + 0.05 × 4) × sqrt(1 + 0.05 × 2) = 1.148 912 529 307 605 7; the fixed
    // ftol(sqrt(12 / 10) × 100 × it) = 125, speed 6, (6 + 10) / 6 = 2 rounds; the archer 188;
    // the tank ftol(sqrt(102 / 100) × 1 000 × it) = 1 160, (5 + 10) / 5 = 3.
    assert_eq!(
        built(&side),
        vec![
            (0, 6, 2, 125, 1250),
            (1, 5, 0, 188, 752),
            (2, 5, 3, 1160, 3480)
        ]
    );
    // The archer's shots: Archery's 0.25 / 5.
    assert_eq!(side.stacks[1].shot, exact(1, 20));
    // Its value per hit point: 125 over 12.
    assert_eq!(side.stacks[0].per_hp, int(125).over(int(12)).unwrap());
}

// covers: CPU-SIM-001
#[test]
fn an_opponent_without_a_hero_takes_nothing_and_one_with_more_tactics_takes_all() {
    let r = rules();
    let army = army(&r, &[(0, "fixed", 10)]);
    let hero = skilled();
    let mut better = bare_hero();
    better.tactics = 3;
    let own = EstimateHero {
        hero: &hero,
        spells: &[],
    };
    // Without an opponent the hero's own skills: sqrt(1.25) × sqrt(1.15).
    let alone = side(
        &r,
        &army,
        X87::ONE,
        (Some(own), None),
        FortLevel::None,
        Ground::default(),
    );
    let factor = skill_root(5).times(skill_root(3));
    let root = X87::int(12).over(X87::int(10)).unwrap().sqrt();
    let unit = SignedX87::from(root.times(X87::int(100)).times(factor)).ftol();
    assert_eq!(alone.stacks[0].unit, unit);
    // Tactics 2 against 3: none (16 / 6 = 2 rounds with Tactics 2, (6 + 14) / 6 = 3 without).
    let opponent = EstimateHero {
        hero: &better,
        spells: &[],
    };
    let side = side(
        &r,
        &army,
        X87::ONE,
        (Some(own), Some(opponent)),
        FortLevel::None,
        Ground::default(),
    );
    assert_eq!(side.stacks[0].contact, 3);
}

// covers: CPU-SIM-001
#[test]
fn a_share_of_hit_points_is_rounded_down_and_a_skill_above_99_counts_99() {
    let r = rules();
    let mut hero = bare_hero();
    hero.attack = 120;
    // The Elixir's quarter of a living creature's hit points: 10 × 2 500 / 10 000 = 2.
    hero.effects = vec![BattleEffect::StackStat {
        creatures: CreatureSelector::Flag(CreatureFlag::Alive),
        stat: CreatureStat::HitPoints,
        op: StatOp::PercentBpDown,
        amount: 2500,
    }];
    let reads = HeroReads::of(
        EstimateHero {
            hero: &hero,
            spells: &[],
        },
        &r,
    );
    assert_eq!(reads.health_bonus(&r, creature(&r, "dwarf"), 10), 2);
    assert_eq!(reads.health_bonus(&r, creature(&r, "fixed"), 10), 0);
    assert_eq!((counted(120), counted(200), counted(7)), (99, 0, 7));
}

// covers: CPU-SIM-001
#[test]
fn the_walls_are_the_forts_less_ballistics_none_with_earthquake() {
    let r = rules();
    let mut gunner = bare_hero();
    gunner.war_machines.catapult = Some(CatapultControl { level: 3 });
    let gunner = HeroReads::of(
        EstimateHero {
            hero: &gunner,
            spells: &[],
        },
        &r,
    );
    let quake = [spell(&r, "quake")];
    let shaker = bare_hero();
    let shaker = HeroReads::of(
        EstimateHero {
            hero: &shaker,
            spells: &quake,
        },
        &r,
    );
    assert_eq!(walls(FortLevel::None, None), (false, 0));
    assert_eq!(walls(FortLevel::Fort, None), (true, 4));
    assert_eq!(walls(FortLevel::Castle, None), (true, 4));
    // A Fort less Ballistics 3 is 1: at least 2; a Citadel's 5 less 3 is 2.
    assert_eq!(walls(FortLevel::Fort, Some(&gunner)), (true, 2));
    assert_eq!(walls(FortLevel::Citadel, Some(&gunner)), (true, 2));
    assert_eq!(walls(FortLevel::None, Some(&gunner)), (false, 0));
    assert_eq!(walls(FortLevel::Castle, Some(&shaker)), (false, 0));
}

// covers: CPU-SIM-001
#[test]
fn a_walker_waits_for_the_walls_a_flyer_does_not_and_the_shots_are_halved() {
    let r = rules();
    let army = army(
        &r,
        &[
            (0, "fixed", 1),
            (1, "flyer", 1),
            (2, "archer", 1),
            (3, "sniper", 1),
        ],
    );
    let side = side(
        &r,
        &army,
        X87::ONE,
        (None, None),
        FortLevel::Castle,
        Ground::default(),
    );
    let rows: Vec<_> = side
        .stacks
        .iter()
        .map(|s| (s.slot, s.contact, s.shot))
        .collect();
    // The fixed 3 rounds raised to the walls' 4; the flyer (7 + 14) / 7 = 3; the archer's shots
    // halved, the sniper's (`no_wall_penalty`) not.
    assert_eq!(
        rows,
        vec![
            (0, 4, X87::ZERO),
            (1, 3, X87::ZERO),
            (2, 0, exact(1, 10)),
            (3, 0, fifth().plus(fifth()))
        ]
    );
}

// covers: CPU-SIM-001
#[test]
fn a_specialists_archery_is_the_originals_single_precision_rate() {
    let r = rules();
    let mut hero = bare_hero();
    // The single-precision value of a grown 10.5 %: 0x3DD70A3D.
    hero.damage.ranged = Rate {
        bp: 1050,
        single: NonZeroU32::new(0x3DD7_0A3D),
    };
    let reads = HeroReads::of(
        EstimateHero {
            hero: &hero,
            spells: &[],
        },
        &r,
    );
    assert_eq!(reads.archery, X87::from_f32_bits(0x3DD7_0A3D));
}

// covers: CPU-SIM-001
#[test]
fn a_creature_of_speed_0_is_four_rounds_away_and_a_huge_stack_wraps() {
    let r = rules();
    let army = army(&r, &[(0, "gadget", 1), (1, "tank", 3_000_000)]);
    let side = plain(&r, &army);
    assert_eq!(side.stacks[0].contact, FAR);
    // 1 000 × 3 000 000 in 32 bits.
    let value = 1000_i32.wrapping_mul(3_000_000);
    assert_eq!(side.stacks[1].value, value);
    assert_eq!(side.strength(), value);
}

// covers: CPU-SIM-006
#[test]
fn a_side_may_cast_with_a_book_unless_a_hero_or_the_ground_forbids() {
    let r = rules();
    let army = army(&r, &[(0, "fixed", 1)]);
    let hero = caster(&r, 5, 20, &[("bolt", 1), ("portal", 0)]);
    let one = led(&r, &army, &hero, Ground::default());
    assert!(one.data.may_cast);
    assert_eq!(one.mana, 20);
    // The adventure spell is not among the casts'.
    let cast = one.data.hero.as_ref().unwrap();
    assert_eq!(
        cast.spells.iter().collect::<Vec<_>>(),
        vec![spell(&r, "bolt")]
    );
    assert_eq!((cast.power, cast.duration), (5, 5));
    // An anti-magic garrison.
    let blocked = Ground {
        no_casting: true,
        ..Ground::default()
    };
    assert!(!led(&r, &army, &hero, blocked).data.may_cast);
    // No book.
    let mut unread = hero.clone();
    unread.magic.spell_book = None;
    assert!(!led(&r, &army, &unread, Ground::default()).data.may_cast);
    // The Orb of Inhibition on either hero; Recanter's Cloak on the opponent and Cursed Ground
    // lower the level.
    let mut orb = hero.clone();
    orb.effects.push(BattleEffect::NoCasting);
    assert!(!led(&r, &army, &orb, Ground::default()).data.may_cast);
    let mut cloak = bare_hero();
    cloak.effects = vec![
        BattleEffect::NoCasting,
        BattleEffect::SpellLevelLimit { max_level: 2 },
    ];
    let spells = book(&hero);
    let heroes = (
        Some(EstimateHero {
            hero: &hero,
            spells: &spells,
        }),
        Some(EstimateHero {
            hero: &cloak,
            spells: &[],
        }),
    );
    let against = side(
        &r,
        &army,
        X87::ONE,
        heroes,
        FortLevel::None,
        Ground::default(),
    );
    assert!(!against.data.may_cast);
    assert_eq!(against.data.level, 2);
    let cursed = Ground {
        battlefield: r.battlefield_by_key("t:cursed"),
        ..Ground::default()
    };
    assert_eq!(led(&r, &army, &hero, cursed).data.level, 1);
    // The Power power is clamped to 1..99 and the Conjuring artifacts add rounds.
    let mut weak = hero.clone();
    weak.power = 0;
    weak.effects.push(BattleEffect::SpellDuration { rounds: 2 });
    let weak = led(&r, &army, &weak, Ground::default());
    let cast = weak.data.hero.as_ref().unwrap();
    assert_eq!((cast.power, cast.duration), (1, 3));
}

// --- the losses (CPU-SIM-002) ---

/// The imp, the archer and the gog of T2b's cases (units 100, 150, 300).
fn three() -> Side {
    side_of(vec![
        walker(0, 10, 5, 3, 100),
        shooter(2, 4, 4, 150),
        walker(1, 3, 2, 4, 300),
    ])
}

// covers: CPU-SIM-002
#[test]
fn a_loss_spreads_by_value_and_a_group_takes_its_share_first() {
    // 500 over 2 500: the imp ftol(1000 × 500 / 2500) = 200 (count 8), the archer
    // ftol(600 × 300 / 1500) = 120 (480: 4), the gog the 180 left (720: 3).
    let mut side = three();
    assert_eq!(side.spread(500, 0, FAR), 0);
    assert_eq!(left(&side), vec![(0, 8, 800), (2, 4, 480), (1, 3, 720)]);
    // 1 500 on the stacks in contact by round 3 (the imp alone): it dies, 500 left.
    let mut side = three();
    assert_eq!(side.spread(1500, 1, 3), 500);
    assert_eq!(left(&side), vec![(0, 0, 0), (2, 4, 600), (1, 3, 900)]);
    // Nothing in the range: the loss stays.
    assert_eq!(three().spread(700, 1, 2), 700);
}

// covers: CPU-SIM-002
#[test]
fn a_side_takes_damage_on_its_stacks_in_contact_then_on_all_or_is_wiped() {
    // 1 500 in round 3: the imp's 1 000, then 500 over the rest (the dead imp skipped): the
    // archer ftol(600 × 500 / 1500) = 200 (400: 3), the gog 300 (600: 2).
    let mut side = three();
    side.take_damage(1500, 3);
    assert_eq!(side.strength, 1000);
    assert_eq!(left(&side), vec![(0, 0, 0), (2, 3, 400), (1, 2, 600)]);
    // Its whole strength: wiped.
    let mut side = three();
    side.take_damage(2500, 0);
    assert_eq!(side.strength, 0);
    assert_eq!(left(&side), vec![(0, 0, 0), (2, 0, 0), (1, 0, 0)]);
}

// covers: CPU-SIM-002
#[test]
fn the_survivors_keep_their_slots_and_a_beaten_side_none() {
    let stack = |slot, count| ArmyStack {
        slot,
        creature: nobody(),
        count,
    };
    let army = [stack(0, 10), stack(1, 3), stack(2, 4), stack(7, 9)];
    let mut side = three();
    side.take_damage(1500, 3);
    // The stack of slot 0 died; slot 7 is not simulated and keeps its count.
    assert_eq!(
        side.survivors(&army),
        vec![stack(1, 2), stack(2, 3), stack(7, 9)]
    );
    side.wipe();
    assert_eq!(side.survivors(&army), vec![]);
}

// covers: CPU-SIM-002, CPU-SIM-007
#[test]
fn a_count_is_the_ceiling_of_the_value_left_and_a_take_at_most_the_value() {
    assert_eq!(ceiling(401, 100), 5);
    assert_eq!(ceiling(400, 100), 4);
    assert_eq!(ceiling(0, 100), 0);
    assert_eq!(ceiling(7, 0), 0);
    let mut stack = walker(0, 10, 5, 1, 100);
    assert_eq!(take_value(&mut stack, 250), 250);
    assert_eq!((stack.count, stack.value), (8, 750));
    assert_eq!(take_value(&mut stack, 900), 750);
    assert_eq!((stack.count, stack.value), (0, 0));
}

// --- the damage and the stand-off (CPU-SIM-003) ---

// covers: CPU-SIM-003
#[test]
fn a_sides_damage_and_hold_value_truncate_after_each_stack() {
    let side = three();
    // Shots alone: 4 × 150 × 0.2 = 120; with the imp in contact by round 3: + 10 × 100 × 0.2;
    // all in melee: 0.2 × 1 000 + 0.1 × 600 + 0.2 × 900 = 440.
    assert_eq!(side.damage(0, false), 120);
    assert_eq!(side.damage(3, false), 320);
    assert_eq!(side.damage(FAR, true), 440);
    // 1 000 + 0.7 × 600 + 900.
    assert_eq!(side.hold(), 2320);
    // The truncation after each stack: 2 × 0.7 three times gives 1, 2, 3, not 4.
    let thin = side_of(vec![
        shooter(0, 2, 4, 1),
        shooter(1, 2, 4, 1),
        shooter(2, 2, 4, 1),
    ]);
    assert_eq!(thin.hold(), 3);
}

// covers: CPU-SIM-003
#[test]
fn the_stand_off_wipes_the_weaker_and_costs_the_winner() {
    // 1 420 against 900: the winner loses ftol(f32(900 / 1420 + 0.05) × 900) = 615.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 10, 5, 3, 100), shooter(2, 4, 4, 150)]),
        side_of(vec![walker(1, 3, 2, 4, 300)]),
    );
    finish(&mut a, &mut b);
    assert_eq!((a.strength, b.strength), (985, 0));
    assert_eq!(left(&a), vec![(0, 7, 616), (2, 3, 369)]);
    // 200 against 900: the first wiped, the second loses ftol(f32(200 / 900 + 0.05) × 200) = 54.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 2, 5, 3, 100)]),
        side_of(vec![walker(1, 3, 2, 4, 300)]),
    );
    finish(&mut a, &mut b);
    assert_eq!((a.strength, b.strength), (0, 846));
    // Equal: the first is wiped, and the second loses ftol(f32(1.05) × 300) = 314, more than its
    // 300.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 3, 5, 3, 100)]),
        side_of(vec![walker(1, 1, 2, 4, 300)]),
    );
    finish(&mut a, &mut b);
    assert_eq!((a.strength, b.strength), (0, 0));
    // A side without a hold value: no stand-off.
    let (mut a, mut b) = (side_of(vec![]), three());
    finish(&mut a, &mut b);
    assert_eq!(b, three());
}

// --- the lookahead (CPU-SIM-004) and the run (CPU-SIM-005) ---

/// Two walkers (26 × 50 in 2 rounds, 25 × 100 in 3) against 4 × 150 in 1.
fn walkers_and_one() -> (Side, Side) {
    (
        side_of(vec![walker(0, 26, 6, 2, 50), walker(1, 25, 6, 3, 100)]),
        side_of(vec![walker(0, 4, 3, 1, 150)]),
    )
}

// covers: CPU-SIM-004
#[test]
fn a_side_engages_only_with_a_walker_in_reach_and_when_the_lookahead_says_now() {
    let r = rules();
    let (a, b) = walkers_and_one();
    // Round 1: none of the walkers reaches.
    assert!(!engages(&r, &a, &b, 1));
    // Round 3: its walkers reach and engaging now scores best; round 2 waits.
    assert!(!engages(&r, &a, &b, 2));
    assert!(engages(&r, &a, &b, 3));
    // The lone walker reaches from round 1 and would lose: it waits.
    assert!(!engages(&r, &b, &a, 1));
    // Dead walkers reach nothing.
    let mut dead = a.clone();
    dead.wipe();
    assert!(!engages(&r, &dead, &b, 3));
}

// covers: CPU-SIM-005
#[test]
fn the_run_shoots_while_neither_engages_and_fights_in_contact_when_the_attacker_does() {
    // Rounds 1 and 2 nobody engages (no shooter: no damage), round 3 the attacker does: it
    // deals 0.2 × its stacks in contact, takes 0.2 × 600 = 120 on them first; the defender is
    // wiped (no stand-off).
    let r = rules();
    let (mut a, mut b) = walkers_and_one();
    run(&r, &mut a, &mut b);
    assert_eq!(a.strength, 3680);
    assert_eq!(left(&a), vec![(0, 26, 1259), (1, 25, 2421)]);
    assert_eq!(b.strength, 0);
}

// covers: CPU-SIM-005
#[test]
fn the_defender_engaging_alone_fights_both_sides_with_all_their_stacks() {
    // Round 3 only the defender engages (0x426D17): both deal 0.2 of all their stacks.
    let r = rules();
    let (mut a, mut b) = (
        side_of(vec![walker(0, 28, 9, 1, 100)]),
        side_of(vec![walker(0, 18, 4, 1, 100), walker(1, 29, 4, 1, 300)]),
    );
    run(&r, &mut a, &mut b);
    assert_eq!(a.strength, 0);
    assert_eq!(b.strength, 9856);
    assert_eq!(left(&b), vec![(0, 17, 1690), (1, 28, 8166)]);
}

// covers: CPU-SIM-005
#[test]
fn both_sides_engaging_fight_with_all_their_stacks() {
    let r = rules();
    let (mut a, mut b) = (
        side_of(vec![walker(1, 2, 5, 1, 50), shooter(0, 3, 5, 150)]),
        side_of(vec![walker(1, 14, 7, 1, 50), shooter(0, 15, 7, 300)]),
    );
    run(&r, &mut a, &mut b);
    assert_eq!(a.strength, 0);
    assert_eq!(b.strength, 5135);
    assert_eq!(left(&b), vec![(1, 14, 692), (0, 15, 4443)]);
}

// covers: CPU-SIM-005, CPU-SIM-003
#[test]
fn three_rounds_end_in_the_stand_off() {
    // Rounds 1 and 2 the shots alone, round 3 the attacker engages; both still stand: the
    // stand-off (8 600 held against ftol(0.7 × 1 193) + 1 277 = 2 112) wipes the defender.
    let r = rules();
    let (mut a, mut b) = (
        side_of(vec![walker(1, 17, 6, 1, 100), walker(0, 28, 6, 3, 300)]),
        side_of(vec![shooter(0, 21, 6, 100), walker(1, 15, 6, 1, 150)]),
    );
    run(&r, &mut a, &mut b);
    assert_eq!(a.strength, 7976);
    assert_eq!(left(&a), vec![(1, 14, 1344), (0, 23, 6632)]);
    assert_eq!(b.strength, 0);
}

// --- the casts (CPU-SIM-006 to -010) ---

/// The bolt's caster: Spell Power 5, 20 points, Basic Air (the bolt's 10 + 10 × 5 = 60).
fn bolter(r: &BattleRuleset) -> BattleHero {
    caster(r, 5, 20, &[("bolt", 1)])
}

// covers: CPU-SIM-006, CPU-SIM-007
#[test]
fn the_faster_side_casts_first_and_a_bolt_hits_the_stack_it_takes_most_from() {
    let r = rules();
    let (slow, fast) = (
        side_of(vec![walker(0, 1, 3, 1, 10)]),
        side_of(vec![walker(0, 1, 7, 1, 10)]),
    );
    assert_eq!((slow.max_speed(), fast.max_speed()), (3, 7));
    let mut dead = fast.clone();
    dead.wipe();
    assert_eq!(dead.max_speed(), 0);
    // Sides without heroes cast nothing, in either order.
    let (mut a, mut b) = (slow.clone(), fast.clone());
    casts(&r, &mut a, &mut b, 1);
    casts(&r, &mut b, &mut a, 1);
    assert_eq!((a, b), (slow, fast));
    // The bolt's 60 is 600 of value on the fixed (10 a hit point) and on the tank (10): the tie
    // goes to the later stack, the tank; it costs 4 of 20 at Basic.
    let hero = bolter(&r);
    let mut caster = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    let mut enemy = plain(&r, &army(&r, &[(0, "fixed", 10), (1, "tank", 3)]));
    cast(&r, &mut caster, &mut enemy, 1);
    assert_eq!(caster.mana, 16);
    assert_eq!(left(&enemy), vec![(0, 10, 1000), (1, 3, 2400)]);
    assert_eq!(enemy.strength, 3400);
    // Slower than its enemy, it casts second; on the flyer (8 hit points: 15 a hit point) 900.
    let mut quick = plain(&r, &army(&r, &[(0, "flyer", 10)]));
    let mut slower = led(&r, &army(&r, &[(0, "slow", 1)]), &hero, Ground::default());
    casts(&r, &mut quick, &mut slower, 1);
    assert_eq!(quick.strength, 1200 - 900);
}

// covers: CPU-SIM-006
#[test]
fn a_side_casts_only_with_strength_points_and_a_spell_it_affords_up_to_its_level() {
    let r = rules();
    let hero = bolter(&r);
    let enemy = plain(&r, &army(&r, &[(0, "fixed", 10)]));
    let own = army(&r, &[(0, "fixed", 1)]);
    let check = |mut caster: Side| {
        let mut target = enemy.clone();
        cast(&r, &mut caster, &mut target, 1);
        target.strength
    };
    assert_eq!(check(led(&r, &own, &hero, Ground::default())), 400);
    // No strength, no points, no casting, too dear (5 points at none for a hero without the
    // school: 4 < 5), above the level (Cursed Ground's 1 bars the rise), no kind (the quake).
    let mut beaten = led(&r, &own, &hero, Ground::default());
    beaten.strength = 0;
    assert_eq!(check(beaten), 1000);
    let mut empty = led(&r, &own, &hero, Ground::default());
    empty.mana = 0;
    assert_eq!(check(empty), 1000);
    let blocked = Ground {
        no_casting: true,
        ..Ground::default()
    };
    assert_eq!(check(led(&r, &own, &hero, blocked)), 1000);
    let poor = caster(&r, 5, 4, &[("bolt", 0)]);
    assert_eq!(check(led(&r, &own, &poor, Ground::default())), 1000);
    let cursed = Ground {
        battlefield: r.battlefield_by_key("t:cursed"),
        ..Ground::default()
    };
    let lofty = caster(&r, 5, 20, &[("quake", 1), ("smite", 1), ("offering", 1)]);
    assert_eq!(check(led(&r, &own, &lofty, cursed)), 1000);
}

// covers: CPU-SIM-006
#[test]
fn the_armies_change_the_cost_and_familiars_return_a_fifth() {
    let r = rules();
    let hero = bolter(&r);
    // The enemy's Pegasi +2, the own Magi -2: 4.
    let mut caster = led(&r, &army(&r, &[(0, "magus", 1)]), &hero, Ground::default());
    let mut enemy = plain(&r, &army(&r, &[(0, "pegasus", 10), (1, "fixed", 10)]));
    cast(&r, &mut caster, &mut enemy, 1);
    assert_eq!(caster.mana, 16);
    // The Pegasi alone: 6; an enemy hero with living Familiars gains 6 / 5 = 1.
    let mut caster = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    let familiars = army(&r, &[(0, "familiar", 5), (1, "pegasus", 10)]);
    let keeper = caster_of_nothing();
    let mut enemy = led(&r, &familiars, &keeper, Ground::default());
    cast(&r, &mut caster, &mut enemy, 1);
    assert_eq!((caster.mana, enemy.mana), (14, 3 + 1));
    // Dead Familiars return nothing; without a hero neither.
    let mut caster = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    let mut enemy = led(&r, &familiars, &keeper, Ground::default());
    enemy.stacks[0].count = 0;
    cast(&r, &mut caster, &mut enemy, 1);
    assert_eq!(enemy.mana, 3);
    let mut caster = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    let mut enemy = plain(&r, &familiars);
    cast(&r, &mut caster, &mut enemy, 1);
    assert_eq!((caster.mana, enemy.mana), (14, 0));
}

/// A hero with 3 spell points and no spells.
fn caster_of_nothing() -> BattleHero {
    let mut hero = bare_hero();
    hero.magic.mana = 3;
    hero
}

// covers: CPU-SIM-007, BAT-MAG-005, BAT-MAG-008, BAT-MAG-066
#[test]
fn a_bolts_loss_takes_the_creatures_factor_its_damage_factors_and_the_casters_bonuses() {
    let r = rules();
    let bolt = spell(&r, "bolt");
    let spell = (bolt, r.spell(bolt).unwrap());
    let none = HeroMagic::default();
    let enemy = plain(
        &r,
        &army(
            &r,
            &[
                (0, "dwarf", 5),
                (1, "golem", 2),
                (2, "ember", 10),
                (3, "troglo", 4),
            ],
        ),
    );
    let of = |name: &str| {
        enemy
            .stacks
            .iter()
            .find(|s| s.creature == creature(&r, name))
            .unwrap()
    };
    // The dwarf: ftol(0.8 × 60) = 48 at 20 a hit point, 960 of its 1 000; the golem: 60 less
    // 75 % is 15 at 30, 450; the ember: 60 doubled at 12, 1 440, at most its 1 200; the troglo
    // is immune.
    let loss = |name: &str, heroes| loss(&r, spell, 60, of(name), heroes);
    assert_eq!(loss("dwarf", (&none, None)), 960);
    assert_eq!(loss("golem", (&none, None)), 450);
    assert_eq!(loss("ember", (&none, None)), 1200);
    assert_eq!(loss("troglo", (&none, None)), 0);
    // The Orb of Vulnerability lifts the immunity (60 at 8, at most the troglo's 320) and the
    // resistance; the stack's hero's
    // Resistance 10 % lowers the share (0.7 × 60 is 42 in doubles, at 20); the Power of the Dragon
    // Father on its hero bars the level-1 bolt.
    let magic = |effects: Vec<BattleEffect>| {
        let mut hero = bare_hero();
        hero.effects = effects;
        HeroMagic::of(&hero, false).unwrap()
    };
    let orb = magic(vec![BattleEffect::NegateImmunities]);
    assert_eq!(loss("troglo", (&orb, None)), 320);
    assert_eq!(loss("dwarf", (&orb, None)), 1000);
    let resisting = magic(vec![BattleEffect::MagicResistance {
        rate: Rate {
            bp: 1000,
            single: None,
        },
    }]);
    assert_eq!(loss("dwarf", (&none, Some(&resisting))), 840);
    let father = magic(vec![BattleEffect::SpellLevelImmunity { max_level: 4 }]);
    assert_eq!(loss("golem", (&none, Some(&father))), 0);
    // Sorcery's 50 %: 90 on the golem, 22 at 30, 660 of its 600: 600.
    let sorcery = magic(vec![BattleEffect::SpellDamage {
        scope: SpellScope::All,
        rate: Rate {
            bp: 5000,
            single: None,
        },
    }]);
    assert_eq!(loss("golem", (&sorcery, None)), 600);
    // A dead stack loses nothing.
    let mut dead = of("dwarf").clone();
    dead.value = 0;
    assert_eq!(super::loss(&r, spell, 60, &dead, (&none, None)), 0);
}

/// An enemy of the slow (20 × 50, 10 a hit point), the fixed (10 × 100) and the tank (3 × 1 000) in
/// slots 0, 1, 2.
fn row(r: &BattleRuleset) -> Side {
    plain(
        r,
        &army(r, &[(0, "slow", 20), (1, "fixed", 10), (2, "tank", 3)]),
    )
}

// covers: CPU-SIM-007
#[test]
fn a_bolt_that_takes_nothing_is_not_cast_and_an_area_hits_up_to_its_radius() {
    let r = rules();
    let hero = bolter(&r);
    let mut own = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    let mut immune = plain(&r, &army(&r, &[(0, "troglo", 4)]));
    cast(&r, &mut own, &mut immune, 1);
    assert_eq!((own.mana, immune.strength), (20, 320));
    // An area of radius 2 on the fixed (slot 1): it, then the slow (slot 0) and the tank (slot
    // 2), 600 each.
    let nova = spell(&r, "nova");
    let mut wide = r.spell(nova).unwrap().clone();
    wide.magic.area = Some(h3_content_battle::SpellArea {
        inner: 0,
        radius: [2; 4],
    });
    let c = Candidate {
        spell: nova,
        mastery: 1,
        power: 5,
        duration: 5,
        target: Some(1),
        second: None,
        value: 1800,
    };
    let mut enemy = row(&r);
    damage_apply(&r, &mut enemy, (&HeroMagic::default(), (nova, &wide)), &c);
    assert_eq!(left(&enemy), vec![(0, 8, 400), (1, 4, 400), (2, 3, 2400)]);
    assert_eq!(enemy.strength, 5000 - 1800);
}

// covers: CPU-SIM-007
#[test]
fn a_chain_hops_to_the_weaker_stacks_at_half_the_damage_and_only_in_round_one() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("chain", 1)]);
    let mut own = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    // The tank 600, then the fixed at 30: 300, the slow at 15: 150; the hop at 7 finds none.
    let mut enemy = row(&r);
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!(left(&enemy), vec![(0, 17, 850), (1, 7, 700), (2, 3, 2400)]);
    assert_eq!(enemy.strength, 5000 - 1050);
    // Round 2: the chain is the first round's.
    let mut enemy = row(&r);
    cast(&r, &mut own, &mut enemy, 2);
    assert_eq!(enemy.strength, 5000);
    // From the weakest stack it hops up the order.
    let mut aimed = row(&r);
    aimed.stacks[2].value = 0;
    aimed.stacks[1].value = 0;
    let bolt = spell(&r, "chain");
    let spell = (bolt, r.spell(bolt).unwrap());
    let none = HeroMagic::default();
    let mut c = Candidate {
        spell: bolt,
        mastery: 1,
        power: 5,
        duration: 5,
        target: None,
        second: None,
        value: 0,
    };
    let mut up = row(&r);
    up.stacks[0].count = 20;
    damage_value(&r, &aimed, (&none, spell), &mut c);
    assert_eq!((c.target, c.value), (Some(0), 600));
    assert_eq!(
        next_in_chain(&r, &up, spell, (1, 0, 30), (&none, None)),
        Some(1)
    );
    assert_eq!(
        next_in_chain(&r, &up, spell, (1, 0, 0), (&none, None)),
        None
    );
}

// covers: CPU-SIM-007
#[test]
fn an_area_spell_adds_the_stacks_in_the_slots_beside_its_target() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("nova", 1)]);
    let mut own = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    // The tank (slot 2) and its one neighbour, the fixed (slot 1): 1 200.
    let mut enemy = row(&r);
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!(left(&enemy), vec![(0, 20, 1000), (1, 4, 400), (2, 3, 2400)]);
    // A neighbour the fire spell cannot touch (the ember) does not count: the target alone.
    let mut enemy = plain(&r, &army(&r, &[(0, "ember", 10), (1, "tank", 3)]));
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!(left(&enemy), vec![(0, 10, 1200), (1, 3, 2400)]);
    // The value counts as many neighbours as the radius and stops there.
    let nova = spell(&r, "nova");
    let spell = (nova, r.spell(nova).unwrap());
    let none = HeroMagic::default();
    let mut c = Candidate {
        spell: nova,
        mastery: 1,
        power: 5,
        duration: 5,
        target: None,
        second: None,
        value: 0,
    };
    let enemy = plain(
        &r,
        &army(&r, &[(0, "slow", 20), (1, "tank", 3), (2, "fixed", 10)]),
    );
    area_value(&r, &enemy, spell, (2, 60, 1), &mut c, (&none, None));
    assert_eq!(c.value, 600);
    area_value(&r, &enemy, spell, (2, 60, 2), &mut c, (&none, None));
    assert_eq!(c.value, 1800);
}

// covers: CPU-SIM-008
#[test]
fn a_mass_spell_counts_both_sides_and_carries_its_totals_from_stack_to_stack() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("storm", 1)]);
    // Own: the fixed 600 and the tank 600, 1 200 of 6 000; the enemy's 1 800: worth 600.
    let mut own = led(
        &r,
        &army(&r, &[(0, "fixed", 10), (1, "tank", 5)]),
        &hero,
        Ground::default(),
    );
    let mut enemy = row(&r);
    cast(&r, &mut own, &mut enemy, 1);
    // Own from the last: the tank loses 600, then the fixed the running 1 200 of its 1 000 (it
    // dies, the total becomes its 1 000): 6 000 - 600 - 1 000.
    assert_eq!(left(&own), vec![(0, 0, 0), (1, 5, 4400)]);
    assert_eq!(own.strength, 4400);
    // The enemy's: the tank 600, the fixed 600 + 600 (dies: 1 000 taken), the slow 1 000 + 600
    // (dies: 1 000).
    assert_eq!(left(&enemy), vec![(0, 0, 0), (1, 0, 0), (2, 3, 2400)]);
    assert_eq!(enemy.strength, 2400);
    // Worth nothing when the own loss reaches the enemy's.
    let mut own = led(
        &r,
        &army(&r, &[(0, "fixed", 10), (1, "tank", 5)]),
        &hero,
        Ground::default(),
    );
    let mut enemy = plain(&r, &army(&r, &[(0, "fixed", 10), (1, "tank", 3)]));
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!(enemy.strength, 4000);
}

// covers: CPU-SIM-009
#[test]
fn a_blessing_raises_its_best_stack_and_turns_its_value_per_hit_point() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("bless", 1)]);
    let mut own = led(
        &r,
        &army(&r, &[(0, "fixed", 10), (1, "tank", 5)]),
        &hero,
        Ground::default(),
    );
    let mut enemy = row(&r);
    // The tank: ftol(f32(2 × 5 × 5 000) × 1 / 500) = 100 (the fixed 20).
    cast(&r, &mut own, &mut enemy, 1);
    let tank = &own.stacks[1];
    assert_eq!((tank.count, tank.unit, tank.value), (5, 1020, 5100));
    // The slip (#65): 5 100 / (5 000 × 10) = 0.102, not 10.2; the strength stays.
    assert_eq!(tank.per_hp.times(int(1_000_000)).ftol(), 102_000);
    assert_eq!(own.strength, 6000);
    assert_eq!(enemy.strength, 5000);
}

// covers: CPU-SIM-009
#[test]
fn a_curse_lowers_the_enemys_best_stack_and_wards_need_an_enemy_that_casts() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("curse", 1), ("airward", 1)]);
    let mut own = led(&r, &army(&r, &[(0, "fixed", 1)]), &hero, Ground::default());
    let mut enemy = row(&r);
    // The curse on the tank: 2 × 5 × 3 000 / 500 = 60 (the ward is worth nothing: the enemy has
    // no hero); its unit 1 000 - 1 000 × 60 / 3 000 = 980.
    cast(&r, &mut own, &mut enemy, 1);
    let tank = &enemy.stacks[2];
    assert_eq!((tank.count, tank.unit, tank.value), (3, 980, 2940));
    assert_eq!(tank.per_hp.times(int(1_000_000)).ftol(), 98_000);
    // Against a casting enemy the ward counts: 10 × 5 × 1 × 1 000... on the own fixed (100):
    // ftol(10 × 5 × 100 / 500) = 10, below the curse's 60.
    let warded = caster(&r, 5, 20, &[("airward", 1)]);
    let mut own = led(
        &r,
        &army(&r, &[(0, "fixed", 1)]),
        &warded,
        Ground::default(),
    );
    let foe = caster(&r, 1, 1, &[("bolt", 1)]);
    let mut enemy = led(&r, &army(&r, &[(0, "slow", 1)]), &foe, Ground::default());
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!((own.mana, own.stacks[0].value), (16, 110));
}

// covers: CPU-SIM-009
#[test]
fn dispel_values_the_own_stacks_then_from_advanced_the_enemys_and_at_expert_both() {
    let r = rules();
    let foe = caster(&r, 1, 1, &[("bolt", 1)]);
    let check = |mastery: u8| {
        let hero = caster(&r, 5, 20, &[("dispel", mastery)]);
        let mut own = led(&r, &army(&r, &[(0, "fixed", 10)]), &hero, Ground::default());
        let mut enemy = led(&r, &army(&r, &[(0, "tank", 3)]), &foe, Ground::default());
        cast(&r, &mut own, &mut enemy, 1);
        (own.stacks[0].value, enemy.stacks[0].value)
    };
    // Basic: the own fixed, ftol(10 × 5 × 1 000 / 500) = 100.
    assert_eq!(check(1), (1100, 3000));
    // Advanced: the enemy's tank (20 × 5 × 3 000 / 500 = 600) beats the own 200: on the enemy.
    assert_eq!(check(2), (1000, 2400));
    // Expert (a mass version): both, 30 × 5 × 1 000 / 500 = 300 up, 900 down.
    assert_eq!(check(3), (1300, 2100));
    // Advanced with the own stack the better: on the own.
    let hero = caster(&r, 5, 20, &[("dispel", 2)]);
    let mut own = led(&r, &army(&r, &[(0, "tank", 3)]), &hero, Ground::default());
    let mut enemy = led(&r, &army(&r, &[(0, "fixed", 10)]), &foe, Ground::default());
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!((own.stacks[0].value, enemy.stacks[0].value), (3600, 1000));
}

// covers: CPU-SIM-009
#[test]
fn a_cure_is_no_dispel_and_a_mass_version_skips_the_stacks_without_value() {
    let r = rules();
    // Cure at Expert (a mass version) against a casting enemy: every own stack with value,
    // 10 × 5 × v / 500 (the tank's 500); against one that cannot cast it is worth nothing.
    let hero = caster(&r, 5, 20, &[("cure", 3)]);
    let foe = caster(&r, 1, 1, &[("bolt", 1)]);
    let mut own = led(
        &r,
        &army(&r, &[(0, "fixed", 10), (1, "tank", 5)]),
        &hero,
        Ground::default(),
    );
    own.stacks[0].value = 0;
    let mut enemy = led(&r, &army(&r, &[(0, "slow", 1)]), &foe, Ground::default());
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!((own.stacks[0].value, own.stacks[1].value), (0, 5500));
    let mut own = led(&r, &army(&r, &[(0, "tank", 5)]), &hero, Ground::default());
    let mut enemy = row(&r);
    cast(&r, &mut own, &mut enemy, 1);
    assert_eq!(own.mana, 20);
    assert!(!is_dispel(r.spell(spell(&r, "cure")).unwrap()));
}

// covers: CPU-SIM-010
#[test]
fn a_raise_brings_back_the_creatures_the_best_stack_lost() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("rise", 1)]);
    let mut own = led(
        &r,
        &army(&r, &[(0, "dwarf", 10), (1, "ghoul", 10), (2, "tank", 1)]),
        &hero,
        Ground::default(),
    );
    // The dwarves lost 4 (value 1 200 of 2 000), the ghouls (not alive) 5.
    own.stacks[1].count = 6;
    own.stacks[1].value = 1200;
    own.stacks[0].count = 5;
    own.stacks[0].value = 300;
    let mut enemy = row(&r);
    cast(&r, &mut own, &mut enemy, 1);
    // 40 + 50 × 5 = 290 hit points at 20: 5 800 / 200 = 29 creatures, at most 4: back to 10.
    let dwarves = own
        .stacks
        .iter()
        .find(|s| s.creature == creature(&r, "dwarf"))
        .unwrap();
    assert_eq!((dwarves.count, dwarves.value), (10, 2000));
    // The ghouls stay; the strength too (#66).
    let ghouls = own
        .stacks
        .iter()
        .find(|s| s.creature == creature(&r, "ghoul"))
        .unwrap();
    assert_eq!(ghouls.count, 5);
    assert_eq!(own.strength, 2000 + 600 + 1000);
    // An undamaged side raises nothing: no cast.
    let mut whole = led(&r, &army(&r, &[(0, "dwarf", 10)]), &hero, Ground::default());
    let mut enemy = row(&r);
    cast(&r, &mut whole, &mut enemy, 1);
    assert_eq!(whole.mana, 20);
}

// covers: CPU-SIM-006
#[test]
fn a_spells_kind_is_one_ai_kind_alone() {
    let r = rules();
    let kind_of = |name: &str, round| kind(r.spell(spell(&r, name)).unwrap(), round);
    assert_eq!(kind_of("bolt", 2), Some(Kind::Damage));
    assert_eq!(kind_of("chain", 1), Some(Kind::Damage));
    assert_eq!(kind_of("chain", 2), None);
    assert_eq!(kind_of("storm", 1), Some(Kind::Mass));
    assert_eq!(kind_of("bless", 1), Some(Kind::Other));
    assert_eq!(kind_of("rise", 1), Some(Kind::Raise));
    assert_eq!(kind_of("offering", 1), None);
    assert_eq!(kind_of("smite", 1), None);
    assert_eq!(kind_of("quake", 1), None);
    // The ward counters casting, the bless does not.
    assert!(counters_casting(r.spell(spell(&r, "airward")).unwrap()));
    assert!(!counters_casting(r.spell(spell(&r, "bless")).unwrap()));
}

// covers: CPU-SIM-005, CPU-SIM-006
#[test]
fn a_run_with_casters_casts_in_the_lookahead_and_the_rounds() {
    let r = rules();
    let hero = caster(&r, 5, 40, &[("bolt", 1)]);
    let mut a = led(&r, &army(&r, &[(0, "tank", 5)]), &hero, Ground::default());
    let mut b = row(&r);
    let before = b.strength;
    run(&r, &mut a, &mut b);
    // Every round's bolt costs 4.
    assert!(a.mana < 40);
    assert!(b.strength < before);
}

fn arb_stack() -> impl Strategy<Value = Stack> {
    (0_u8..7, 1_i32..60, 1_i32..10, 0_i32..=FAR, 1_i32..400).prop_map(
        |(slot, count, speed, contact, unit)| {
            if contact == 0 {
                shooter(slot, count, speed, unit)
            } else {
                walker(slot, count, speed, contact, unit)
            }
        },
    )
}

fn arb_side() -> impl Strategy<Value = Side> {
    proptest::collection::vec(arb_stack(), 1..5).prop_map(|stacks| {
        let mut slots: Vec<Stack> = Vec::new();
        for stack in stacks {
            if slots.iter().all(|s| s.slot != stack.slot) {
                slots.push(stack);
            }
        }
        side_of(slots)
    })
}

proptest! {
    // covers: CPU-SIM-002, CPU-SIM-005
    /// A run is a function of its sides; it leaves each stack's count the ceiling of its value
    /// and never raises a value or a strength.
    #[test]
    fn a_run_is_repeatable_and_only_takes(a in arb_side(), b in arb_side()) {
        let r = rules();
        let (mut first, mut second) = (a.clone(), b.clone());
        run(&r, &mut first, &mut second);
        let (mut again, mut other) = (a.clone(), b.clone());
        run(&r, &mut again, &mut other);
        prop_assert_eq!(&first, &again);
        prop_assert_eq!(&second, &other);
        for (after, before) in [(&first, &a), (&second, &b)] {
            prop_assert!(after.strength >= 0 && after.strength <= before.strength);
            for (s, t) in after.stacks.iter().zip(&before.stacks) {
                prop_assert_eq!(s.count, ceiling(s.value, s.unit));
                prop_assert!(s.value >= 0 && s.value <= t.value);
            }
        }
    }
}
