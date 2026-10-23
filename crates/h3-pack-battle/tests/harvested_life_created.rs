//! Batch 2d's raised and created stacks on the harvested SoD packs (E04b T11b review, r4a
//! F-10): RA2, CL1 and SU a of `experiments/results/E04b-batch-2d.md` (SoD 4.0 Complete, plain HD,
//! 2026-10-08) replayed through real commands with the heroes the batch used, derived in the
//! Heroes context: Basic Resurrection and the army after the battle (BAT-MAG-035, BAT-END-003),
//! Clone (BAT-MAG-038), Summon Elemental and the stacks a battle creates (BAT-MAG-037,
//! BAT-MAG-039), the surrender price (BAT-FLE-002), the end of a side left with created stacks
//! only (BAT-END-001).

use h3_battle::{
    BattleResult, BattleState, CastTarget, Command, CommandError, Direction, Event, EventKind,
    Field, LegalCommands, NO_CORPSE, StackId, apply, castable_spells, legal_commands,
};
use h3_content::{Formation, SkillId, SpellId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide, StackOrigin, StackRecord};
use h3_heroes::{HeroDetails, derive_battle_hero, war_machines};

fn sod() -> BattleRuleset {
    h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
        "base:faithful",
    )
    .unwrap()
}

fn spell(r: &BattleRuleset, key: &str) -> SpellId {
    r.spell_by_key(key).unwrap()
}

/// `hero` at level 1 with attack and defense 0, `power` and `knowledge`, the skills `skills`,
/// wearing a Spell Book and `worn`, knowing `spells`, with full mana.
fn details(
    r: &BattleRuleset,
    hero: &str,
    (power, knowledge): (u8, u8),
    skills: Vec<(SkillId, u8)>,
    worn: &[&str],
    spells: &[&str],
) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [0, 0, power, knowledge],
        skills,
        worn: h3_heroes::wearing(
            r,
            &core::iter::once("base:spell_book")
                .chain(worn.iter().copied())
                .map(|k| r.artifact_by_key(k).unwrap().into())
                .collect::<Vec<_>>(),
        ),
        backpack: vec![],
        spells: spells.iter().map(|k| spell(r, k)).collect(),
        mana: u16::from(knowledge) * 10,
    }
}

fn army(r: &BattleRuleset, stacks: &[(&str, u32)]) -> Army {
    Army {
        stacks: stacks
            .iter()
            .enumerate()
            .map(|(slot, &(key, count))| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: r.creature_by_key(key).unwrap(),
                count,
            })
            .collect(),
    }
}

/// The battle of Red's `heroes[0]` with `attacker` and its worn war machines against Blue's
/// `heroes[1]` with `defender`, on `terrain`, without obstacles; each player has 30 000 gold (SU
/// c's record).
fn battle(
    r: &BattleRuleset,
    heroes: [&HeroDetails; 2],
    attacker: &[(&str, u32)],
    defender: &[(&str, u32)],
    terrain: Option<&str>,
) -> BattleState {
    let setup = BattleSetup {
        attacker: army(r, attacker),
        defender: army(r, defender),
        obstacles: vec![],
        heroes: heroes.map(|d| Some(derive_battle_hero(d, r).unwrap())),
        terrain: terrain.map(|t| r.terrain_by_key(t).unwrap()),
        battlefield: None,
        anti_magic: false,
        siege: None,
        war_machines: heroes.map(|d| war_machines(d, r)),
        formations: [Formation::Loose; 2],
        funds: [30_000; 2],
        ..BattleSetup::default()
    };
    BattleState::new(r, &setup, 1).unwrap()
}

/// Stack `stack`, whose turn it is, does `command` for the side that commands it; returns the
/// events.
fn turn(s: &mut BattleState, r: &BattleRuleset, stack: u8, command: Command) -> Vec<Event> {
    assert_eq!(s.active(), Some(StackId(stack)));
    let side = s.side_to_act().unwrap();
    let mut events = Vec::new();
    apply(s, r, side, command, &mut events).unwrap();
    events
}

fn cast(spell: SpellId, target: CastTarget) -> Command {
    Command::Cast {
        spell,
        target,
        second: CastTarget::None,
    }
}

fn stack(id: u8) -> CastTarget {
    CastTarget::Stack(StackId(id))
}

/// The active stack attacks stack `enemy` if it can, else walks as far towards the other
/// side's edge as it can.
fn advance(s: &mut BattleState, r: &BattleRuleset, enemy: u8) {
    let field = Field::new(r.battle().field);
    let side = s.side_to_act().unwrap();
    let mut legal = LegalCommands::new();
    legal_commands(s, r, side, &mut legal);
    let attack = legal
        .iter()
        .find(|c| matches!(c, Command::Attack { target, .. } if *target == StackId(enemy)));
    let walk = legal
        .iter()
        .filter(|c| matches!(c, Command::Move { .. }))
        .max_by_key(|c| match c {
            Command::Move { to } => field.coords(*to).0,
            _ => 0,
        });
    let command = *attack.or(walk).unwrap_or(&Command::Defend);
    apply(s, r, side, command, &mut Vec::<Event>::new()).unwrap();
}

/// The stacks `events` report as created: `(stack, count, column, row)`.
fn created(r: &BattleRuleset, events: &[Event]) -> Vec<(u8, u32, i32, i32)> {
    let field = Field::new(r.battle().field);
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::StackCreated {
                stack, count, hex, ..
            } => {
                let (x, y) = field.coords(hex);
                Some((stack.0, count, x, y))
            }
            _ => None,
        })
        .collect()
}

/// The spell damage `events` report, as `(target, damage, kills)`.
fn spell_damage(events: &[Event]) -> Vec<(u8, u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage {
                target,
                damage,
                kills,
                ..
            } => Some((target.0, damage, kills)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-035, BAT-MAG-004, BAT-END-003
/// Batch 2d RA2: Gelare (power 1, Basic Earth Magic, Resurrection) with ten Swordsmen against
/// Grindan (power 7, Lightning Bolt: 7 × 25 + 10 = 185) with ten Peasants. After the first bolt
/// five Swordsmen are left, the top one at 25; Basic Resurrection (50 + 40 = 90 hit points)
/// makes them eight; after the second bolt two are left. The Swordsmen then win, and Red's army
/// after the battle is one Swordsman: the three raised for the battle only do not stay, and an
/// army whose living creatures were all raised so keeps one in its last living stack. (RA3, the
/// same with a Ballista, left Red no army in the original: quirk BAT-QUIRK-6, fixed in every
/// profile.)
#[test]
fn batch_2d_ra2_basic_resurrection_raises_for_the_battle_only() {
    let r = sod();
    let (rise, bolt) = (
        spell(&r, "base:resurrection"),
        spell(&r, "base:lightning_bolt"),
    );
    let earth = vec![(r.skill_by_key("base:earth_magic").unwrap(), 1)];
    let gelare = details(&r, "ab:gelare", (1, 2), earth, &[], &["base:resurrection"]);
    let grindan = details(
        &r,
        "ab:grindan",
        (7, 2),
        vec![],
        &[],
        &["base:lightning_bolt"],
    );
    let mut s = battle(
        &r,
        [&gelare, &grindan],
        &[("base:swordsman", 10)],
        &[("ab:peasant", 10)],
        None,
    );
    // Stacks: 0 the Swordsmen (they act first), 1 the Peasants.
    let (swordsmen, peasants) = (0, 1);
    let left = |s: &BattleState| (s.stacks()[0].count, s.stacks()[0].top_hp);
    // Round 1: the Swordsmen Defend; Blue's Lightning Bolt.
    turn(&mut s, &r, swordsmen, Command::Defend);
    let events = turn(&mut s, &r, peasants, cast(bolt, stack(swordsmen)));
    assert_eq!(spell_damage(&events), [(swordsmen, 185, 5)]);
    assert_eq!(left(&s), (5, 25));
    turn(&mut s, &r, peasants, Command::Defend);
    // Round 2: Red's Resurrection, for the battle only; the second Lightning Bolt.
    let events = turn(&mut s, &r, swordsmen, cast(rise, stack(swordsmen)));
    let raised: Vec<(u64, u32, bool)> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellRaised {
                hit_points,
                creatures,
                temporary,
                ..
            } => Some((hit_points, creatures, temporary)),
            _ => None,
        })
        .collect();
    assert_eq!(raised, [(90, 3, true)]);
    assert_eq!((left(&s).0, s.temporary(StackId(swordsmen))), (8, 3));
    turn(&mut s, &r, swordsmen, Command::Defend);
    let events = turn(&mut s, &r, peasants, cast(bolt, stack(swordsmen)));
    assert_eq!(spell_damage(&events), [(swordsmen, 185, 6)]);
    assert_eq!(left(&s).0, 2);
    turn(&mut s, &r, peasants, Command::Defend);
    // From round 3: the Swordsmen walk to the Peasants and attack; the Peasants Defend.
    for _ in 0..40 {
        match s.active() {
            Some(StackId(0)) => advance(&mut s, &r, peasants),
            Some(_) => {
                turn(&mut s, &r, peasants, Command::Defend);
            }
            None => break,
        }
    }
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    let record = outcome.stacks[0][0];
    assert_eq!(
        (record.origin, record.start, record.end),
        (StackOrigin::Army(0), 10, 1)
    );
    let survivors: Vec<u32> = outcome.survivors[0]
        .stacks
        .iter()
        .map(|a| a.count)
        .collect();
    assert_eq!(survivors, [1]);
}

// covers: BAT-MAG-038, BAT-MAG-037, BAT-FLE-002
/// Batch 2d CL1: Aine (power 3, Clone) with ten Archers and ten Pikemen against Damacon (power
/// 2, Clone and Magic Arrow) with ten Pikemen and 100 Peasants. The surrender price is 800 before
/// and after Red's Clone. Blue's clone appears on the up-left neighbour of its Pikemen, Red's on
/// the up-right neighbour of the Archers; both act in round 1. The Archers have shot once (11
/// shots left); their clone has 12 shots, 11 after its shot. In round 2 Blue's Magic Arrow (30)
/// kills all ten Archers of the clone and leaves nothing; when round 3 starts Blue's clone (two
/// rounds) is gone, without a corpse.
#[test]
fn batch_2d_cl1_clones_places_turns_shots_and_ends() {
    let r = sod();
    let field = Field::new(r.battle().field);
    let (clone, arrow) = (spell(&r, "base:clone"), spell(&r, "base:magic_arrow"));
    let aine = details(&r, "base:aine", (3, 5), vec![], &[], &["base:clone"]);
    let damacon = details(
        &r,
        "base:damacon",
        (2, 5),
        vec![],
        &[],
        &["base:clone", "base:magic_arrow"],
    );
    let mut s = battle(
        &r,
        [&aine, &damacon],
        &[("base:archer", 10), ("base:pikeman", 10)],
        &[("base:pikeman", 10), ("ab:peasant", 100)],
        None,
    );
    // Stacks: 0 the Archers, 1 Red's Pikemen; 2 Blue's Pikemen, 3 the Peasants; the clones
    // become 4 (Blue's Pikemen) and 5 (Red's Archers).
    let price = |s: &BattleState| s.surrender_price(&r, BattleSide::Attacker);
    let shoot = |s: &BattleState, target: u8| Command::Shoot {
        target: StackId(target),
        at: s.stacks()[usize::from(target)].hex,
    };
    let beside = |s: &BattleState, id: u8, d| {
        let hex = field.neighbor(s.stacks()[usize::from(id)].hex, d).unwrap();
        field.coords(hex)
    };
    // Round 1, the Archers' turn (Red's first): the price; they shoot the Peasants.
    assert_eq!(price(&s), Some(800));
    let shot = shoot(&s, 3);
    turn(&mut s, &r, 0, shot);
    assert_eq!(s.stacks()[0].shots, 11);
    // Blue's Pikemen's turn: Blue's Clone on them.
    let events = turn(&mut s, &r, 2, cast(clone, stack(2)));
    let (x, y) = beside(&s, 2, Direction::TopLeft);
    assert_eq!(created(&r, &events), [(4, 10, x, y)]);
    turn(&mut s, &r, 2, Command::Defend);
    // Red's Pikemen's turn: Red's Clone on the Archers; the price again.
    let events = turn(&mut s, &r, 1, cast(clone, stack(0)));
    let (x, y) = beside(&s, 0, Direction::TopRight);
    assert_eq!(created(&r, &events), [(5, 10, x, y)]);
    assert_eq!(price(&s), Some(800));
    turn(&mut s, &r, 1, Command::Defend);
    // Both clones get a turn in round 1: the Archers' clone shoots with all its shots, Blue's
    // clone and the Peasants Defend.
    let mut acted = vec![];
    while s.round() == 1 {
        let active = s.active().unwrap().0;
        acted.push(active);
        if active == 5 {
            assert_eq!(s.stacks()[5].shots, 12);
            let shot = shoot(&s, 3);
            turn(&mut s, &r, 5, shot);
            assert_eq!(s.stacks()[5].shots, 11);
        } else {
            turn(&mut s, &r, active, Command::Defend);
        }
    }
    acted.sort_unstable();
    assert_eq!(acted, [3, 4, 5]);
    // Round 2, the first turn of a Blue stack: Magic Arrow at the Archers' clone.
    while s.side_to_act() != Some(BattleSide::Defender) {
        let active = s.active().unwrap().0;
        turn(&mut s, &r, active, Command::Defend);
    }
    assert_eq!(s.round(), 2);
    let active = s.active().unwrap().0;
    let events = turn(&mut s, &r, active, cast(arrow, stack(5)));
    assert_eq!(spell_damage(&events), [(5, 30, 10)]);
    assert_eq!(s.stacks()[5].count, 0);
    // Round 3 starts: Blue's clone is gone, and nothing lies where it stood.
    assert!(s.stacks()[4].is_alive());
    while s.round() < 3 {
        let active = s.active().unwrap().0;
        turn(&mut s, &r, active, Command::Defend);
    }
    assert_eq!((s.stacks()[4].count, s.corpse(StackId(4))), (0, NO_CORPSE));
    assert_eq!(s.corpse(StackId(5)), NO_CORPSE);
}

/// The stacks of batch 2d SU's battle: 0 to 6 Red's Peasants, 7 the First Aid Tent, 8 the Ammo
/// Cart (the original's record order, D-23), 9 Blue's Skeletons; from 10 on the elementals.
const SKELETONS: u8 = 9;

/// The active stack Defends.
fn defend(s: &mut BattleState, r: &BattleRuleset) {
    let active = s.active().unwrap().0;
    turn(s, r, active, Command::Defend);
}

/// Red's hero summons Air Elementals in the active stack's turn; the new stack as
/// `(stack, count, column, row)`.
fn summon(s: &mut BattleState, r: &BattleRuleset) -> (u8, u32, i32, i32) {
    let active = s.active().unwrap().0;
    let air = spell(r, "base:air_elemental");
    let made = created(r, &turn(s, r, active, cast(air, CastTarget::None)));
    assert_eq!(made.len(), 1);
    made[0]
}

/// `key`'s record with `origin`, `start` and `end`.
fn record(
    r: &BattleRuleset,
    origin: StackOrigin,
    key: &str,
    (start, end): (u32, u32),
) -> StackRecord {
    StackRecord {
        origin,
        creature: r.creature_by_key(key).unwrap(),
        start,
        end,
    }
}

/// Batch 2d SU's first two rounds, which its three variants share, on grass: Octavia (power 1,
/// 350 spell points, Air Elemental, an Ammo Cart and a First Aid Tent) with seven single
/// Peasants against Calid (power 1, Death Ripple, Implosion) with ten Skeletons. The surrender
/// price is 910 before and after the first summon. E1 (two Air Elementals with attack 10,
/// defense 10, speed 8) appears in column 1 and acts in round 1; E2 follows in round 2, in which
/// the Peasants die of Death Ripple. Returns the battle at the start of round 3 and the rows of
/// E1 and E2.
fn su_after_two_rounds(r: &BattleRuleset) -> (BattleState, [i32; 2]) {
    let ripple = spell(r, "base:death_ripple");
    let octavia = details(
        r,
        "base:octavia",
        (1, 35),
        vec![],
        &["base:ammo_cart", "base:first_aid_tent"],
        &["base:air_elemental"],
    );
    let calid = details(
        r,
        "base:calid",
        (1, 10),
        vec![],
        &[],
        &["base:death_ripple", "base:implosion"],
    );
    let mut s = battle(
        r,
        [&octavia, &calid],
        &[("ab:peasant", 1); 7],
        &[("base:skeleton", 10)],
        Some("base:grass"),
    );
    assert_eq!(s.stacks().len(), 10);
    let price = |s: &BattleState| s.surrender_price(r, BattleSide::Attacker);
    // Round 1: the Skeletons act first and Defend. The first Peasant's turn: the price, E1,
    // the price again.
    turn(&mut s, r, SKELETONS, Command::Defend);
    assert_eq!((s.active(), price(&s)), (Some(StackId(0)), Some(910)));
    let e1 = summon(&mut s, r);
    assert_eq!((e1.0, e1.1, e1.2), (10, 2, 1));
    assert_eq!(price(&s), Some(910));
    let elementals = s.stacks()[10];
    assert_eq!(
        (elementals.attack, elementals.defense, elementals.speed),
        (10, 10, 8)
    );
    // Every Peasant Defends; E1 gets its turn in round 1.
    let mut acted = vec![];
    while s.round() == 1 {
        acted.push(s.active().unwrap().0);
        defend(&mut s, r);
    }
    assert_eq!(acted, [0, 10, 1, 2, 3, 4, 5, 6]);
    // Round 2, E1's turn: E2, on the last free row of column 1 beside the Peasants and the war
    // machines (the batch saw E1 in row 7 and E2 in row 3).
    assert_eq!(s.active(), Some(StackId(10)));
    let e2 = summon(&mut s, r);
    assert_eq!((e2.0, e2.1, e2.2), (11, 2, 1));
    let mut rows = [e1.3, e2.3];
    rows.sort_unstable();
    assert_eq!(rows, [3, 7]);
    defend(&mut s, r);
    defend(&mut s, r);
    // The Skeletons' turn: Death Ripple (5 + 10 = 15) kills the seven Peasants. It spares the
    // undead and war machines only (BAT-MAG-005): the elementals take it too and lose nobody
    // (the batch did not look at them).
    let events = turn(&mut s, r, SKELETONS, cast(ripple, CastTarget::None));
    let struck: Vec<(u8, u64, u32)> = (0..7).map(|p| (p, 15, 1)).collect();
    assert_eq!(
        spell_damage(&events),
        [struck, vec![(10, 15, 0), (11, 15, 0)]].concat()
    );
    assert!(s.stacks()[..7].iter().all(|p| !p.is_alive()));
    assert_eq!((s.stacks()[10].count, s.stacks()[11].count), (2, 2));
    turn(&mut s, r, SKELETONS, Command::Defend);
    assert_eq!(s.round(), 3);
    (s, [e1.3, e2.3])
}

// covers: BAT-MAG-037, BAT-MAG-039, BAT-FLE-002, BAT-END-001, BAT-END-003, BAT-TER-001
/// Batch 2d SU, variant a: after the first two rounds ([`su_after_two_rounds`]) Implosion kills
/// E1 in round 3 and E3 takes its record, so in round 4 E3 acts before E2. One summon a round
/// goes on until twelve have succeeded: ten in column 1 (every row but the war machines' rows 1
/// and 9, and dead E1's row a second time), then two in column 2. With twenty records in use
/// (seven Peasants, two war machines, eleven for the elementals) the thirteenth is refused, and
/// still after another elemental stack died. At the end the elementals kill the Skeletons and
/// both sides have lost; the casualties are seven Peasants and two Air Elementals (the stack
/// that died last: E1's losses went with its record) and ten Skeletons. The rows themselves are
/// drawn: the batch saw 7, 3, 2, 0, 6, 8, 7, 5, 10, 4 and then 3 and 8.
#[test]
fn batch_2d_su_summons_until_the_limit_and_a_side_of_elementals_loses() {
    let r = sod();
    let field = Field::new(r.battle().field);
    let (air, implosion) = (spell(&r, "base:air_elemental"), spell(&r, "base:implosion"));
    let (mut s, [row1, row2]) = su_after_two_rounds(&r);
    let is_elemental = |s: &BattleState| s.active().is_some_and(|id| id.0 >= 10);
    // Round 3: E1 and E2 Wait; Implosion (75 + 100) kills E1; back in the wait phase, E2's
    // turn: E3 takes E1's record.
    turn(&mut s, &r, 10, Command::Wait);
    turn(&mut s, &r, 11, Command::Wait);
    let events = turn(&mut s, &r, SKELETONS, cast(implosion, stack(10)));
    assert_eq!(spell_damage(&events), [(10, 175, 2)]);
    assert!(!s.stacks()[10].is_alive());
    turn(&mut s, &r, SKELETONS, Command::Defend);
    assert_eq!(s.active(), Some(StackId(11)));
    let e3 = summon(&mut s, &r);
    assert_eq!((e3.0, e3.1, e3.2), (10, 2, 1));
    while s.round() == 3 {
        defend(&mut s, &r);
    }
    // Round 4: E3, in the earlier record, acts before E2. From now on one summon a round, on
    // the first elemental's turn, until the book refuses it.
    assert_eq!((s.round(), s.active()), (4, Some(StackId(10))));
    let mut places = vec![(1, row1), (1, row2), (e3.2, e3.3)];
    let refused = loop {
        let round = s.round();
        assert!(is_elemental(&s));
        if !castable_spells(&s, &r, BattleSide::Attacker).contains(air) {
            break round;
        }
        let made = summon(&mut s, &r);
        places.push((made.2, made.3));
        while s.round() == round {
            defend(&mut s, &r);
        }
    };
    assert_eq!((refused, places.len()), (13, 12));
    // Ten in column 1: its rows 1 and 9 hold the war machines, the Peasants' corpses are in
    // nobody's way, and E1's row was free again after its death (no corpse). Then column 2.
    let columns: Vec<i32> = places.iter().map(|p| p.0).collect();
    assert_eq!(columns, [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2]);
    // The two-hex tent's position in the side column (its front hex on (1, 9)), the cart's hex.
    let machines = [s.stacks()[7].hex, s.stacks()[8].hex].map(|h| field.coords(h));
    assert_eq!(machines, [(0, 9), (1, 1)]);
    let mut rows: Vec<i32> = places[..10].iter().map(|p| p.1).collect();
    rows.sort_unstable();
    let mut expected = vec![0, 2, 3, 4, 5, 6, 7, 8, 10, row1];
    expected.sort_unstable();
    assert_eq!(rows, expected);
    assert_ne!(places[10].1, places[11].1);
    let red = |s: &BattleState| {
        s.stacks()
            .iter()
            .filter(|t| t.side == BattleSide::Attacker)
            .count()
    };
    assert_eq!(red(&s), 20);
    let side = s.side_to_act().unwrap();
    assert_eq!(
        apply(
            &mut s,
            &r,
            side,
            cast(air, CastTarget::None),
            &mut Vec::<Event>::new()
        ),
        Err(CommandError::InvalidSpellTarget)
    );
    // The Skeletons' turn: Implosion at the last elemental stack. In the next round the summon
    // is still refused.
    let last = u8::try_from(s.stacks().len() - 1).unwrap();
    while s.active() != Some(StackId(SKELETONS)) {
        defend(&mut s, &r);
    }
    turn(&mut s, &r, SKELETONS, cast(implosion, stack(last)));
    assert!(!s.stacks()[usize::from(last)].is_alive());
    turn(&mut s, &r, SKELETONS, Command::Defend);
    while s.round() == 13 {
        defend(&mut s, &r);
    }
    assert!(is_elemental(&s));
    assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(air));
    assert_eq!(s.hero(BattleSide::Attacker).unwrap().mana, 350 - 12 * 25);
    // The end: the elementals attack the Skeletons until none are left; the Skeletons Defend.
    for _ in 0..400 {
        match s.active() {
            Some(StackId(SKELETONS)) => {
                turn(&mut s, &r, SKELETONS, Command::Defend);
            }
            Some(_) => advance(&mut s, &r, SKELETONS),
            None => break,
        }
    }
    // Nobody wins, and neither army has survivors (both heroes were gone from the map).
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Draw);
    assert!(outcome.survivors.iter().all(|army| army.stacks.is_empty()));
    // Red's records: the seven Peasants, the two war machines, then eleven created stacks (E1's
    // record went to E3). The casualties as the result window listed them: seven Peasants and
    // two Air Elementals, those of the second Implosion; ten Skeletons.
    let mut expected: Vec<StackRecord> = (0..7)
        .map(|slot| record(&r, StackOrigin::Army(slot), "ab:peasant", (1, 0)))
        .collect();
    expected.push(record(
        &r,
        StackOrigin::WarMachine,
        "base:first_aid_tent",
        (1, 1),
    ));
    expected.push(record(
        &r,
        StackOrigin::WarMachine,
        "base:ammo_cart",
        (1, 1),
    ));
    expected.extend([record(&r, StackOrigin::Created, "base:air_elemental", (2, 2)); 10]);
    expected.push(record(
        &r,
        StackOrigin::Created,
        "base:air_elemental",
        (2, 0),
    ));
    assert_eq!(outcome.stacks[0], expected);
    assert_eq!(
        outcome.stacks[1],
        [record(&r, StackOrigin::Army(0), "base:skeleton", (10, 0))]
    );
}

// covers: BAT-END-001, BAT-END-003, BAT-FLE-001, BAT-FLE-002
/// Batch 2d SU, variants b and c: after the same two rounds Red has only its elementals and war
/// machines left. Round 3: the elementals Defend, and in the Skeletons' turn Blue retreats (b)
/// or surrenders (c) for 300, half the cost of ten Skeletons. The result names Blue's leaving;
/// Red, whose army has no survivors, has lost as well (its hero was gone from the map both
/// times) and is still paid the surrender's price (Red's gold rose by 300). The casualties are
/// the seven Peasants, and none of Blue's.
#[test]
fn batch_2d_su_a_side_of_elementals_loses_too_when_the_enemy_leaves() {
    let r = sod();
    let blue = BattleSide::Defender;
    for (command, result, cost) in [
        (Command::Retreat, BattleResult::Retreat(blue), None),
        (Command::Surrender, BattleResult::Surrender(blue), Some(300)),
    ] {
        let (mut s, _) = su_after_two_rounds(&r);
        defend(&mut s, &r);
        defend(&mut s, &r);
        assert_eq!(s.surrender_price(&r, blue), Some(300));
        turn(&mut s, &r, SKELETONS, command);
        let outcome = s.outcome().unwrap();
        assert_eq!((outcome.result, outcome.surrender_cost), (result, cost));
        assert!(outcome.survivors[0].stacks.is_empty());
        let mut expected: Vec<StackRecord> = (0..7)
            .map(|slot| record(&r, StackOrigin::Army(slot), "ab:peasant", (1, 0)))
            .collect();
        expected.push(record(
            &r,
            StackOrigin::WarMachine,
            "base:first_aid_tent",
            (1, 1),
        ));
        expected.push(record(
            &r,
            StackOrigin::WarMachine,
            "base:ammo_cart",
            (1, 1),
        ));
        expected.extend([record(&r, StackOrigin::Created, "base:air_elemental", (2, 2)); 2]);
        assert_eq!(outcome.stacks[0], expected);
        assert_eq!(
            outcome.stacks[1],
            [record(&r, StackOrigin::Army(0), "base:skeleton", (10, 10))]
        );
    }
}
