//! Batch 2c's control and battlefield spells on the harvested SoD packs (E04b T11b review, r3
//! F-10): F1, F2 and H1 of `experiments/results/E04b-batch-2c.md` (SoD 4.0 Complete, plain HD,
//! 2026-10-07) replayed through real commands with the heroes the batch used, derived in the
//! Heroes context: Fire Wall and Force Field (BAT-MAG-027, BAT-MAG-028), a walker's path
//! (BAT-MOV-003) and Hypnotize (BAT-MAG-026). Also, without a batch, no attack from a seen
//! Quicksand pit (BAT-MAG-032) on these packs.

use h3_battle::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Direction, Event, EventKind,
    Field, Hex, LegalCommands, StackId, apply, cast_targets, legal_commands,
};
use h3_content::{Formation, SkillId, SpellId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};
use h3_heroes::{HeroDetails, derive_battle_hero};

fn sod(profile: &str) -> BattleRuleset {
    h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
        profile,
    )
    .unwrap()
}

fn spell(r: &BattleRuleset, key: &str) -> SpellId {
    r.spell_by_key(key).unwrap()
}

/// `hero` at level 1 with attack and defense 0, `power` and `knowledge`, the skills `skills`, a
/// worn Spell Book, knowing `spells`, with full mana.
fn details(
    r: &BattleRuleset,
    hero: &str,
    (power, knowledge): (u8, u8),
    skills: Vec<(SkillId, u8)>,
    spells: &[&str],
) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [0, 0, power, knowledge],
        skills,
        worn: h3_heroes::wearing(r, &[r.artifact_by_key("base:spell_book").unwrap().into()]),
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

/// The battle of Red's `heroes[0]` with `attacker` against Blue's `heroes[1]` with `defender`,
/// without obstacles.
fn battle(
    r: &BattleRuleset,
    heroes: [&HeroDetails; 2],
    attacker: &[(&str, u32)],
    defender: &[(&str, u32)],
) -> BattleState {
    battle_seeded(r, heroes, attacker, defender, 1)
}

/// [`battle`] with the battle's seed `seed`.
fn battle_seeded(
    r: &BattleRuleset,
    heroes: [&HeroDetails; 2],
    attacker: &[(&str, u32)],
    defender: &[(&str, u32)],
    seed: u64,
) -> BattleState {
    let setup = BattleSetup {
        attacker: army(r, attacker),
        defender: army(r, defender),
        obstacles: vec![],
        heroes: heroes.map(|d| Some(derive_battle_hero(d, r).unwrap())),
        terrain: None,
        battlefield: None,
        anti_magic: false,
        siege: None,
        war_machines: [vec![], vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
        ..BattleSetup::default()
    };
    BattleState::new(r, &setup, seed).unwrap()
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

/// Every stack defends until one of `stacks` is active; returns that one.
fn defend_until(s: &mut BattleState, r: &BattleRuleset, stacks: &[u8]) -> u8 {
    loop {
        let active = s.active().unwrap().0;
        if stacks.contains(&active) {
            return active;
        }
        turn(s, r, active, Command::Defend);
    }
}

fn cast(spell: SpellId, target: CastTarget) -> Command {
    Command::Cast {
        spell,
        target,
        second: CastTarget::None,
    }
}

/// The spell damage `events` report, as `(target, damage, kills)`.
fn burns(events: &[Event]) -> Vec<(u8, u64, u32)> {
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

/// The hexes of `side`'s spell obstacles placed by `spell`, in hex order.
fn obstacle_hexes(s: &BattleState, side: BattleSide, spell: SpellId) -> Vec<Hex> {
    let mut hexes: Vec<Hex> = s
        .spell_obstacles()
        .iter()
        .filter(|o| o.side == side && o.spell == spell)
        .flat_map(|o| o.hexes.iter())
        .collect();
    hexes.sort_by_key(|h| h.0);
    hexes
}

/// `hexes` in hex order.
fn sorted(mut hexes: Vec<Hex>) -> Vec<Hex> {
    hexes.sort_by_key(|h| h.0);
    hexes
}

// covers: BAT-MAG-028, BAT-MAG-027
/// Batch 2c F1: Aine (power 1) with 50 Pikemen and a Peasant against Gelare (power 1) with ten
/// Wolf Riders and a Peasant. Blue's Force Field covers the clicked hex and its up-right
/// neighbour (not mirrored for the defender), Blue's Fire Wall its hex and the up-left one
/// (mirrored). Red's Fire Wall (10 × 1 + 10 = 20) burns the Pikemen when they walk into its
/// lower hex ("The Fire Wall does 20 damage. 2 Pikemen perish.": 48), when they Wait there (46)
/// and when they Defend there (44); cast in round 1, it is gone in round 3.
#[test]
fn batch_2c_f1_fire_wall_burns_on_entering_on_wait_and_on_defend() {
    let r = sod("base:faithful");
    let field = Field::new(r.battle().field);
    let (wall, fire) = (spell(&r, "base:force_field"), spell(&r, "base:fire_wall"));
    let aine = details(&r, "base:aine", (1, 1), vec![], &["base:fire_wall"]);
    let gelare = details(
        &r,
        "ab:gelare",
        (1, 3),
        vec![],
        &["base:force_field", "base:fire_wall"],
    );
    let mut s = battle(
        &r,
        [&aine, &gelare],
        &[("base:pikeman", 50), ("ab:peasant", 1)],
        &[("base:wolf_rider", 10), ("ab:peasant", 1)],
    );
    // Stacks: 0 the Pikemen, in the third row from the top, 1 Red's Peasant; 2 the Wolf Riders,
    // 3 Blue's Peasant.
    let (pikemen, riders) = (0, 2);
    assert_eq!(field.coords(s.stacks()[0].hex), (1, 2));
    let hex = |x, y| field.hex(x, y).unwrap();
    let beside = |h, d| field.neighbor(h, d).unwrap();
    let count = |s: &BattleState| s.stacks()[0].count;
    // Round 1: the Wolf Riders act first; Blue's Force Field in the right half of the field.
    let clicked = hex(11, 6);
    turn(&mut s, &r, riders, cast(wall, CastTarget::Hex(clicked)));
    assert_eq!(
        obstacle_hexes(&s, BattleSide::Defender, wall),
        sorted(vec![clicked, beside(clicked, Direction::TopRight)])
    );
    turn(&mut s, &r, riders, Command::Defend);
    // The Pikemen's turn: Red's Fire Wall two hexes to their right covers that hex and its
    // up-right neighbour; they walk two hexes straight right, into its lower hex.
    let lower = hex(3, 2);
    turn(&mut s, &r, pikemen, cast(fire, CastTarget::Hex(lower)));
    assert_eq!(
        obstacle_hexes(&s, BattleSide::Attacker, fire),
        sorted(vec![lower, beside(lower, Direction::TopRight)])
    );
    let events = turn(&mut s, &r, pikemen, Command::Move { to: lower });
    assert_eq!((burns(&events), count(&s)), (vec![(0, 20, 2)], 48));
    // Round 2: Blue's Fire Wall: its second hex is the clicked hex's up-left neighbour.
    defend_until(&mut s, &r, &[riders]);
    assert_eq!(s.round(), 2);
    let clicked = hex(12, 8);
    turn(&mut s, &r, riders, cast(fire, CastTarget::Hex(clicked)));
    assert_eq!(
        obstacle_hexes(&s, BattleSide::Defender, fire),
        sorted(vec![clicked, beside(clicked, Direction::TopLeft)])
    );
    turn(&mut s, &r, riders, Command::Defend);
    // The Pikemen Wait in the fire, then Defend in it when they come back: burnt each time.
    let events = turn(&mut s, &r, pikemen, Command::Wait);
    assert_eq!((burns(&events), count(&s)), (vec![(0, 20, 2)], 46));
    defend_until(&mut s, &r, &[pikemen]);
    assert_eq!(s.round(), 2);
    let events = turn(&mut s, &r, pikemen, Command::Defend);
    assert_eq!((burns(&events), count(&s)), (vec![(0, 20, 2)], 44));
    // Round 3: Red's Fire Wall is gone (two rounds); the Pikemen stand there unharmed.
    defend_until(&mut s, &r, &[pikemen]);
    assert_eq!(s.round(), 3);
    assert_eq!(obstacle_hexes(&s, BattleSide::Attacker, fire), []);
    let events = turn(&mut s, &r, pikemen, Command::Defend);
    assert_eq!((burns(&events), count(&s)), (vec![], 44));
}

// covers: BAT-MOV-003, BAT-MAG-028
/// Batch 2c F2: of the two equally short paths to the hex two rows straight up, a walker takes
/// the one through its up-right neighbour, for either side. Octavia's and Grindan's ten Pikemen
/// (each with a Peasant; power 1) first walk two hexes towards the middle. Red's Fire Wall on
/// the Pikemen's up-right neighbour burns them on their way up ("The Fire Wall does 20 damage.
/// 2 Pikemen perish.": 8 left); Blue's on its Pikemen's up-left neighbour does not (10 left).
#[test]
fn batch_2c_f2_a_walker_two_rows_up_passes_its_up_right_neighbour() {
    let r = sod("base:faithful");
    let field = Field::new(r.battle().field);
    let fire = spell(&r, "base:fire_wall");
    let octavia = details(&r, "base:octavia", (1, 1), vec![], &["base:fire_wall"]);
    let grindan = details(&r, "ab:grindan", (1, 1), vec![], &["base:fire_wall"]);
    let armies = [("base:pikeman", 10), ("ab:peasant", 1)];
    let mut s = battle(&r, [&octavia, &grindan], &armies, &armies);
    // Stacks: 0 Red's Pikemen, 1 Red's Peasant; 2 Blue's Pikemen, 3 Blue's Peasant; both
    // Pikemen stacks in the third row from the top.
    let hex = |x, y| field.hex(x, y).unwrap();
    assert_eq!(
        [0, 2].map(|i: usize| field.coords(s.stacks()[i].hex)),
        [(1, 2), (15, 2)]
    );
    // Round 1: two hexes towards the middle; no spells.
    for _ in 0..2 {
        match defend_until(&mut s, &r, &[0, 2]) {
            0 => turn(&mut s, &r, 0, Command::Move { to: hex(3, 2) }),
            _ => turn(&mut s, &r, 2, Command::Move { to: hex(13, 2) }),
        };
    }
    // Round 2: each hero's Fire Wall beside its Pikemen, which then walk two rows straight up:
    // Red's on the up-right neighbour (with that hex's own up-right neighbour), Blue's on the
    // up-left neighbour (with that hex's own up-left neighbour).
    let mut left = [0; 2];
    for _ in 0..2 {
        let (id, from, to, side) = match defend_until(&mut s, &r, &[0, 2]) {
            0 => (0, hex(3, 2), hex(3, 0), Direction::TopRight),
            _ => (2, hex(13, 2), hex(13, 0), Direction::TopLeft),
        };
        assert_eq!((s.round(), s.stacks()[usize::from(id)].hex), (2, from));
        let clicked = field.neighbor(from, side).unwrap();
        turn(&mut s, &r, id, cast(fire, CastTarget::Hex(clicked)));
        let owner = s.stacks()[usize::from(id)].side;
        assert_eq!(
            obstacle_hexes(&s, owner, fire),
            sorted(vec![clicked, field.neighbor(clicked, side).unwrap()])
        );
        let events = turn(&mut s, &r, id, Command::Move { to });
        let burnt = if id == 0 { vec![(0, 20, 2)] } else { vec![] };
        assert_eq!(burns(&events), burnt);
        left[usize::from(id / 2)] = s.stacks()[usize::from(id)].count;
    }
    assert_eq!(left, [8, 10]);
}

// covers: BAT-MAG-026, BAT-MAG-017, BAT-MAG-004
/// Batch 2c H1: Clavius (power 2, no mastery) with five Wolf Riders against Kyrre (power 1,
/// Expert Water Magic, Cure) with two Swordsmen, one Swordsman and ten Pikemen. Hypnotize's
/// limit is 2 × 25 + 10 = 60 hit points: refused at the two Swordsmen (70), allowed at the one
/// (35), and still refused at the two once Magic Arrow (30 damage: two left, the top one at 5)
/// has left them 40 of their 70. Hypnotized, the one Swordsman acts for Red in rounds 2 and 3.
/// Blue's Expert Cure heals the wounded Swordsman to 35 and, with quirk Q-0012 (faithful, as
/// observed), passes over the hypnotized stack; without it (strict-fixed) it frees that stack.
#[test]
fn batch_2c_h1_hypnotize_counts_full_hit_points_and_expert_cure_passes_over_it() {
    for (profile, still) in [("base:faithful", true), ("base:strict-fixed", false)] {
        let r = sod(profile);
        let (arrow, hypnotize, cure) = (
            spell(&r, "base:magic_arrow"),
            spell(&r, "base:hypnotize"),
            spell(&r, "base:cure"),
        );
        let clavius = details(
            &r,
            "base:clavius",
            (2, 3),
            vec![],
            &["base:magic_arrow", "base:hypnotize"],
        );
        let water = vec![(r.skill_by_key("base:water_magic").unwrap(), 3)];
        let kyrre = details(&r, "base:kyrre", (1, 1), water, &["base:cure"]);
        let mut s = battle(
            &r,
            [&clavius, &kyrre],
            &[("base:wolf_rider", 5)],
            &[
                ("base:swordsman", 2),
                ("base:swordsman", 1),
                ("base:pikeman", 10),
            ],
        );
        // Stacks: 0 the Wolf Riders; 1 the two Swordsmen, 2 the one Swordsman, 3 the Pikemen.
        let (riders, two, one) = (0, 1, 2);
        let aims = |s: &BattleState| {
            let mut out = CastTargets::new();
            cast_targets(s, &r, BattleSide::Attacker, hypnotize, &mut out);
            out.iter().copied().collect::<Vec<_>>()
        };
        let stack = |id: u8| CastTarget::Stack(StackId(id));
        let hypnotized = |s: &BattleState| {
            s.effects()
                .iter()
                .any(|e| e.stack == StackId(one) && e.spell == hypnotize)
        };
        // Round 1, the Wolf Riders' turn (they act first).
        assert_eq!((s.round(), s.active()), (1, Some(StackId(riders))));
        assert_eq!(aims(&s), [stack(one)], "{profile}");
        let events = turn(&mut s, &r, riders, cast(arrow, stack(two)));
        assert_eq!(burns(&events), [(two, 30, 0)], "{profile}");
        let wounded = s.stacks()[usize::from(two)];
        assert_eq!((wounded.count, wounded.top_hp), (2, 5));
        turn(&mut s, &r, riders, Command::Defend);
        // Round 2: the wounded two are still refused; Hypnotize on the one Swordsman.
        while s.round() < 2 {
            let active = s.active().unwrap().0;
            turn(&mut s, &r, active, Command::Defend);
        }
        defend_until(&mut s, &r, &[riders]);
        assert_eq!(aims(&s), [stack(one)], "{profile}");
        turn(&mut s, &r, riders, cast(hypnotize, stack(one)));
        assert!(hypnotized(&s));
        turn(&mut s, &r, riders, Command::Defend);
        // Blue's two Swordsmen act next (the army slot decides the tie): Expert Cure.
        defend_until(&mut s, &r, &[two, one]);
        assert_eq!(s.side_to_act(), Some(BattleSide::Defender));
        turn(&mut s, &r, two, cast(cure, CastTarget::None));
        assert_eq!(s.stacks()[usize::from(two)].top_hp, 35, "{profile}");
        assert_eq!(hypnotized(&s), still, "{profile}");
        turn(&mut s, &r, two, Command::Defend);
        // The one Swordsman's turn in round 2, and again in round 3: who commands it.
        let commander = if still {
            BattleSide::Attacker
        } else {
            BattleSide::Defender
        };
        // The batch's last line: while hypnotized, the one Swordsman may attack Blue's stacks
        // and Blue's stacks may not attack it (BAT-MAG-026's relations, `Stack::may_attack`).
        let attacks = |s: &BattleState, side: BattleSide| {
            let mut legal = LegalCommands::new();
            legal_commands(s, &r, side, &mut legal);
            legal
                .iter()
                .filter_map(|c| match *c {
                    Command::Attack { target, .. } => Some(target.0),
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<u8>>()
        };
        for round in [2, 3] {
            defend_until(&mut s, &r, &[one]);
            assert_eq!(
                (s.round(), s.side_to_act()),
                (round, Some(commander)),
                "{profile}"
            );
            if still {
                assert!(attacks(&s, commander).contains(&two), "{profile}");
            }
            turn(&mut s, &r, one, Command::Defend);
            if still && s.active() == Some(StackId(3)) {
                assert!(
                    !attacks(&s, BattleSide::Defender).contains(&one),
                    "{profile}"
                );
            }
        }
    }
}

// covers: BAT-MAG-032
/// No attack starts from a position where the walk stops in a pit the walker's side sees (the
/// user, BAT-Q30; `installation.md` §7.16), on the harvested packs: the synthetic tests place
/// pits by hand, which the public interface cannot (E04b T11b's test gap), so here Quicksand
/// draws them and the battle's seed picks the draws. Blue's Archangel (speed 18) lands three
/// hexes to the right of Red's ten Pikemen; Red's hero casts Expert Quicksand (8 pits). For every
/// pit next to the Archangel that the Pikemen can walk to, the move into it stays legal, while
/// the attack from it is neither listed nor accepted (the state untouched) and an attack from a
/// hex without a pit still is, in either profile.
#[test]
fn quicksand_no_attack_starts_from_a_seen_pit_on_the_harvested_packs() {
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        let field = Field::new(r.battle().field);
        let quicksand = spell(&r, "base:quicksand");
        let earth = vec![(r.skill_by_key("base:earth_magic").unwrap(), 3)];
        let red = details(&r, "base:aine", (1, 1), earth, &["base:quicksand"]);
        let blue = details(&r, "ab:gelare", (1, 1), vec![], &[]);
        let (pikemen, angel) = (0, 1);
        let mut checked = 0;
        for seed in 1..=40 {
            let mut s = battle_seeded(
                &r,
                [&red, &blue],
                &[("base:pikeman", 10)],
                &[("base:archangel", 1)],
                seed,
            );
            let (x, y) = field.coords(s.stacks()[0].hex);
            let landing = field.hex(x + 3, y).unwrap();
            turn(&mut s, &r, angel, Command::Move { to: landing });
            // Good morale may give the Archangel another turn: it defends.
            defend_until(&mut s, &r, &[pikemen]);
            turn(&mut s, &r, pikemen, cast(quicksand, CastTarget::None));
            let pits = obstacle_hexes(&s, BattleSide::Attacker, quicksand);
            assert_eq!(pits.len(), 8, "{profile}, seed {seed}");
            let mut legal = LegalCommands::new();
            legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
            let attack_from = |from| Command::Attack {
                target: StackId(angel),
                from,
            };
            let beside: Vec<Hex> = field.neighbors(landing).into_iter().flatten().collect();
            for &pit in pits.iter().filter(|p| beside.contains(p)) {
                if !legal.iter().any(|c| *c == Command::Move { to: pit }) {
                    continue;
                }
                assert!(
                    !legal.iter().any(|c| *c == attack_from(pit)),
                    "{profile}, seed {seed}"
                );
                let mut t = s.clone();
                let mut events = Vec::new();
                let side = t.side_to_act().unwrap();
                assert_eq!(
                    apply(&mut t, &r, side, attack_from(pit), &mut events),
                    Err(CommandError::Unreachable),
                    "{profile}, seed {seed}"
                );
                assert_eq!(t, s, "{profile}, seed {seed}");
                assert!(
                    beside
                        .iter()
                        .any(|&h| !pits.contains(&h) && legal.iter().any(|c| *c == attack_from(h)))
                );
                checked += 1;
            }
        }
        // 8 pits among about 140 hexes: about one seed in five puts one beside the Archangel.
        assert!(checked >= 3, "{profile}: {checked}");
    }
}
