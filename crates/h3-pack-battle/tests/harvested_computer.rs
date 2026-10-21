//! The original's battle AI on the harvested packs (E23 T2a; ADR-0027 point 3, the epic's exit
//! criterion 1): in random battles of the SoD creatures, on the terrains' ordinary fields, on
//! every special battlefield and at a town of every faction's siege record at every fort level,
//! in both quirk profiles, at every difficulty and with random heroes (Tactics, Artillery, First
//! Aid, Ballistics, spells), the computer's choice for the side to act is a command the engine
//! accepts at every step (no fallback fires), and the same state and stream give the same
//! command. Most steps play it; the others play a random legal command or a hero's cast, so the
//! games reach effects and stand-ins (`h3_battle_ai`'s stubs) the policy alone would not. The
//! Cyclops's wall shot (E23 T2b, BAT-AI-004) at a town's walls, below.

use std::collections::BTreeSet;

use h3_battle::{
    BattleState, CastTargets, Command, LegalCommands, NoopSink, apply, cast_targets,
    castable_spells, computer, legal_commands, second_targets, validate,
};
use h3_content::{CreatureFlag, CreatureTypeId, Formation, HeroTypeId, SpellCategory, SpellId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide, FortLevel, Moat, Siege};
use h3_heroes::{ArtifactInstance, HeroDetails, derive_battle_hero, war_machines};
use h3_kernel::Rng;

/// A uniform index below `n` (at least 1).
fn pick(rng: &mut Rng, n: usize) -> usize {
    rng.below(u32::try_from(n).unwrap()) as usize
}

/// Valid details of a random hero: level 1 to 30, primary skills 0 to 20, Tactics, Artillery,
/// First Aid and Ballistics each at a random level (none to Expert), each war machine's artifact
/// half the time, a spell book with half the spells and some mana; the level's experience
/// (HER-EXP-002, E08 T27).
fn details(r: &BattleRuleset, rng: &mut Rng) -> HeroDetails {
    let hero_type = HeroTypeId::from_index(pick(rng, r.hero_types().len())).unwrap();
    let level = u8::try_from(1 + rng.below(30)).unwrap();
    let mut d = HeroDetails {
        hero_type,
        level,
        experience: h3_heroes::experience_for(r.hero_params().unwrap(), level),
        offered: Default::default(),
        primary: [0; 4].map(|_| u8::try_from(rng.below(21)).unwrap()),
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: [0, 20, 100][pick(rng, 3)],
    };
    for skill in [
        "base:tactics",
        "base:artillery",
        "base:first_aid",
        "base:ballistics",
    ] {
        let level = u8::try_from(rng.below(4)).unwrap();
        if level > 0 {
            d.skills.push((r.skill_by_key(skill).unwrap(), level));
        }
    }
    for artifact in ["base:spell_book", "base:ballista", "base:first_aid_tent"] {
        if rng.below(2) == 0 {
            d.worn
                .push(ArtifactInstance::from(r.artifact_by_key(artifact).unwrap()));
        }
    }
    d.spells = (0..r.spells().len())
        .filter_map(SpellId::from_index)
        .filter(|&s| r.spell(s).unwrap().category != SpellCategory::Ability)
        .filter(|_| rng.below(2) == 0)
        .collect();
    h3_heroes::validate(&d, r).unwrap();
    d
}

/// One to seven stacks of `pool`, counts from 1 to 200.
fn army(rng: &mut Rng, pool: &[CreatureTypeId]) -> Army {
    Army {
        stacks: (0..=rng.below(7))
            .map(|slot| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: pool[pick(rng, pool.len())],
                count: 1 + rng.below(200),
            })
            .collect(),
    }
}

/// How the games went: the policy's commands by kind, and the battles that ended.
#[derive(Debug, Default)]
struct Reach {
    shots: u32,
    wall_shots: u32,
    tactics_ends: u32,
    defends: u32,
    over: u32,
}

/// A hero's random cast for `side`, if it may cast.
fn cast(
    state: &BattleState,
    r: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Option<Command> {
    let spells: Vec<SpellId> = castable_spells(state, r, side).iter().collect();
    if spells.is_empty() {
        return None;
    }
    let spell = spells[pick(rng, spells.len())];
    let (mut targets, mut seconds) = (CastTargets::new(), CastTargets::new());
    cast_targets(state, r, side, spell, &mut targets);
    let target = targets.as_slice()[pick(rng, targets.len())];
    second_targets(state, r, side, (spell, target), &mut seconds);
    let second = seconds.as_slice()[pick(rng, seconds.len())];
    Some(Command::Cast {
        spell,
        target,
        second,
    })
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn the_computers_choice_is_legal_in_random_battles_of_the_harvested_creatures() {
    let mut reach = Reach::default();
    for profile in ["base:faithful", "base:strict-fixed"] {
        let sod = h3_pack_battle::compile(
            &[
                h3_pack_battle::SOD_PACK,
                h3_pack_battle::AB_PACK,
                h3_pack_battle::BASE_PACK,
            ],
            profile,
        )
        .unwrap();
        let pool: Vec<CreatureTypeId> = (0..sod.creature_count())
            .filter_map(CreatureTypeId::from_index)
            .filter(|&c| {
                sod.creature(c)
                    .is_some_and(|t| !t.flags.contains(CreatureFlag::WarMachine))
            })
            .collect();
        let factions: Vec<_> = sod.sieges().iter().map(|s| s.faction).collect();
        let battlefields: Vec<_> = sod.battlefields().iter().map(|b| b.key.clone()).collect();
        let mut rng = Rng::from_seed(23);
        let mut seat = Rng::from_seed(2023);
        let mut legal = LegalCommands::new();
        for game in 0..48u64 {
            let details = [details(&sod, &mut rng), details(&sod, &mut rng)];
            let heroes = details
                .clone()
                .map(|d| (rng.below(3) != 0).then(|| derive_battle_hero(&d, &sod).unwrap()));
            // Half the games at a town, a quarter on a special battlefield.
            let (siege, battlefield) = match game % 4 {
                0 | 1 => {
                    let fort = [
                        FortLevel::None,
                        FortLevel::Fort,
                        FortLevel::Citadel,
                        FortLevel::Castle,
                    ][pick(&mut rng, 4)];
                    let moat = if fort >= FortLevel::Citadel {
                        [Moat::Absent, Moat::Harmless, Moat::Damaging][pick(&mut rng, 3)]
                    } else {
                        Moat::Absent
                    };
                    let town = Siege {
                        faction: factions[usize::try_from(game).unwrap() % factions.len()],
                        fort,
                        moat,
                        structures: u8::try_from(rng.below(256)).unwrap(),
                        escape_tunnel: rng.below(2) == 0,
                        morale: 0,
                        luck: 0,
                    };
                    (Some(town), None)
                }
                2 => (
                    None,
                    Some(battlefields[pick(&mut rng, battlefields.len())].clone()),
                ),
                _ => (None, None),
            };
            let machines = [0, 1].map(|i| {
                if heroes[i].is_some() {
                    war_machines(&details[i], &sod)
                } else {
                    vec![]
                }
            });
            let setup = BattleSetup {
                attacker: army(&mut rng, &pool),
                defender: army(&mut rng, &pool),
                battlefield,
                siege,
                war_machines: machines,
                heroes,
                formations: [Formation::Loose, Formation::Loose],
                funds: [rng.next_u32(), rng.next_u32()],
                difficulty: u8::try_from(game % 5).unwrap(),
                human: [rng.below(2) == 0, rng.below(2) == 0],
                ..BattleSetup::default()
            };
            let mut state = BattleState::new(&sod, &setup, game)
                .unwrap_or_else(|e| panic!("{profile} game {game}: the setup is refused: {e:?}"));
            for _ in 0..300u32 {
                let Some(side) = state.side_to_act() else {
                    reach.over += 1;
                    break;
                };
                let proposed = h3_battle_ai::unchecked(&state, &sod, side, &mut seat.clone());
                validate(&state, &sod, side, proposed).unwrap_or_else(|e| {
                    panic!("{profile} game {game}: the computer's {proposed:?} is refused: {e:?}")
                });
                let mut again = seat.clone();
                let command = h3_battle_ai::choose(&state, &sod, side, &mut seat);
                assert_eq!(command, proposed, "{profile} game {game}");
                assert_eq!(
                    command,
                    h3_battle_ai::choose(&state, &sod, side, &mut again),
                    "{profile} game {game}"
                );
                assert_eq!(seat, again, "{profile} game {game}");
                let roll = rng.below(10);
                let played = if roll < 7 {
                    match command {
                        Command::Shoot { .. } => reach.shots += 1,
                        Command::ShootWall { .. } => reach.wall_shots += 1,
                        Command::EndTactics => reach.tactics_ends += 1,
                        _ => reach.defends += 1,
                    }
                    command
                } else if let Some(cast) = cast(&state, &sod, side, &mut rng).filter(|_| roll < 9) {
                    cast
                } else {
                    legal_commands(&state, &sod, side, &mut legal);
                    let staying: Vec<Command> = legal
                        .iter()
                        .copied()
                        .filter(|c| !c.leaves_battle())
                        .collect();
                    staying[pick(&mut rng, staying.len())]
                };
                apply(&mut state, &sod, side, played, &mut NoopSink).unwrap_or_else(|e| {
                    panic!("{profile} game {game}: {played:?} is rejected: {e:?}")
                });
            }
        }
    }
    assert!(
        reach.shots > 300
            && reach.wall_shots > 20
            && reach.tactics_ends > 20
            && reach.defends > 1000
            && reach.over > 30,
        "{reach:?}"
    );
}

/// The SoD packs' ruleset in `profile`.
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

/// An army of the harvested creatures `stacks` (keys and counts).
fn harvested(r: &BattleRuleset, stacks: &[(&str, u32)]) -> Army {
    Army {
        stacks: (0u8..)
            .zip(stacks)
            .map(|(slot, &(key, count))| ArmyStack {
                slot,
                creature: r.creature_by_key(key).unwrap(),
                count,
            })
            .collect(),
    }
}

/// A battle without heroes at the Castle town's Fort (its walls and gate, no moat or turrets),
/// or on the open field without `fort`, on Normal.
fn walled(
    r: &BattleRuleset,
    attacker: &[(&str, u32)],
    defender: &[(&str, u32)],
    fort: bool,
) -> BattleState {
    let setup = BattleSetup {
        attacker: harvested(r, attacker),
        defender: harvested(r, defender),
        siege: fort.then(|| Siege {
            faction: r.sieges()[0].faction,
            fort: FortLevel::Fort,
            moat: Moat::Absent,
            structures: 0,
            escape_tunnel: false,
            morale: 0,
            luck: 0,
        }),
        formations: [Formation::Loose, Formation::Loose],
        difficulty: 1,
        ..BattleSetup::default()
    };
    BattleState::new(r, &setup, 3).unwrap()
}

/// Whether the active stack is of the creature `key`.
fn acting(r: &BattleRuleset, state: &BattleState, key: &str) -> bool {
    let id = state.active().unwrap();
    Some(state.stack(id).unwrap().creature) == r.creature_by_key(key)
}

// covers: BAT-AI-004
#[test]
fn the_cyclopes_batter_the_weakest_wall_while_their_army_is_locked_out() {
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        // Behind the closed gate the pikemen outside reach no defender: most of the attacker's
        // strength is locked out, against the little the cyclopes' best shot is worth.
        let state = walled(
            &r,
            &[("base:cyclops", 5), ("base:pikeman", 200)],
            &[("base:pikeman", 300)],
            true,
        );
        assert!(acting(&r, &state, "base:cyclops"));
        // Every wall stands with the same hit points: each is drawn (random(1, 4)).
        let walls: BTreeSet<u8> = computer::walls(&state).map(|(part, _)| part).collect();
        assert_eq!(walls, BTreeSet::from([1, 2, 4, 5]));
        let drawn: BTreeSet<u8> = (0..40)
            .map(|seed| {
                let command = h3_battle_ai::choose(
                    &state,
                    &r,
                    BattleSide::Attacker,
                    &mut Rng::from_seed(seed),
                );
                let Command::ShootWall { part } = command else {
                    panic!("{profile}: {command:?}");
                };
                part
            })
            .collect();
        assert_eq!(drawn, walls, "{profile}");
        // Played on: the cyclopes batter the weakest standing wall, the only one once a wall
        // has lost hit points, until a breach lets the pikemen in and they shoot instead (or,
        // blocked by a defender, defend).
        let mut state = state;
        let mut seat = Rng::from_seed(11);
        let (mut alone, mut shots) = (0, 0);
        for _ in 0..80 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            let command = if acting(&r, &state, "base:cyclops") {
                let walls: Vec<(u8, u8)> = computer::walls(&state).collect();
                let least = walls.iter().map(|&(_, hp)| hp).min();
                let command = h3_battle_ai::choose(&state, &r, side, &mut seat);
                match command {
                    Command::ShootWall { part } => {
                        let weakest: Vec<u8> = walls
                            .iter()
                            .filter(|w| Some(w.1) == least)
                            .map(|w| w.0)
                            .collect();
                        assert!(weakest.contains(&part), "{profile}: {part} of {walls:?}");
                        alone += u32::from(weakest.len() == 1);
                    }
                    Command::Shoot { .. } => shots += 1,
                    // Blocked by a defender that came out, they are walkers (BAT-AI-005).
                    Command::Attack { .. } | Command::Move { .. } | Command::Defend => {
                        assert!(!computer::shoots(&state, &r, state.active().unwrap()));
                    }
                    other => panic!("{profile}: {other:?}"),
                }
                command
            } else {
                Command::Defend
            };
            apply(&mut state, &r, side, command, &mut NoopSink).unwrap();
        }
        assert!(alone > 0 && shots > 0, "{profile}: {alone} {shots}");
    }
}

// covers: BAT-AI-004
#[test]
fn the_cyclopes_shoot_a_stack_without_walls_to_batter_or_with_their_army_free() {
    let r = sod("base:faithful");
    // Alone outside the walls, the cyclopes record their enemies (they can shoot): nothing is
    // locked out, so they shoot.
    let shoots = |state: &BattleState, side| {
        matches!(
            h3_battle_ai::choose(state, &r, side, &mut Rng::from_seed(1)),
            Command::Shoot { .. }
        )
    };
    let state = walled(&r, &[("base:cyclops", 5)], &[("base:pikeman", 300)], true);
    assert!(shoots(&state, BattleSide::Attacker));
    // On the open field there is no wall.
    let state = walled(
        &r,
        &[("base:cyclops", 5), ("base:pikeman", 200)],
        &[("base:pikeman", 300)],
        false,
    );
    assert!(shoots(&state, BattleSide::Attacker));
    // The defender's cyclopes never batter their own walls.
    let mut state = walled(
        &r,
        &[("base:pikeman", 200)],
        &[("base:cyclops", 5), ("base:pikeman", 300)],
        true,
    );
    while !acting(&r, &state, "base:cyclops") {
        let side = state.side_to_act().unwrap();
        apply(&mut state, &r, side, Command::Defend, &mut NoopSink).unwrap();
    }
    assert!(shoots(&state, BattleSide::Defender));
}

/// A hero of the first hero type with Fire Wall in its spell book, 100 mana and spell power 5.
fn fire_mage(r: &BattleRuleset) -> h3_contracts::BattleHero {
    let d = HeroDetails {
        hero_type: HeroTypeId::from_index(0).unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [0, 0, 5, 10],
        skills: vec![],
        worn: vec![ArtifactInstance::from(
            r.artifact_by_key("base:spell_book").unwrap(),
        )],
        backpack: vec![],
        spells: vec![r.spell_by_key("base:fire_wall").unwrap()],
        mana: 100,
    };
    h3_heroes::validate(&d, r).unwrap();
    derive_battle_hero(&d, r).unwrap()
}

/// Where the defender's lone `walker` strikes the attacker's lone angel from, on Normal, after the
/// angel flew to six hexes before it, its hero having cast Fire Wall (two hexes: the hex between
/// them on their row and its upper right neighbour) first if `wall`: `(the hex it strikes from,
/// the wall's target hex)`.
fn strike_hex(r: &BattleRuleset, walker: &str, wall: bool) -> (h3_battle::Hex, h3_battle::Hex) {
    let setup = BattleSetup {
        attacker: harvested(r, &[("base:angel", 1)]),
        defender: harvested(r, &[(walker, 5)]),
        heroes: [Some(fire_mage(r)), None],
        formations: [Formation::Loose, Formation::Loose],
        difficulty: 1,
        ..BattleSetup::default()
    };
    let mut state = BattleState::new(r, &setup, 7).unwrap();
    assert!(acting(r, &state, "base:angel"));
    let field = h3_battle::Field::new(r.battle().field);
    let (x, y) = field.coords(state.stacks()[1].hex);
    let (target, to) = (field.hex(x - 5, y).unwrap(), field.hex(x - 6, y).unwrap());
    if wall {
        let cast = Command::Cast {
            spell: r.spell_by_key("base:fire_wall").unwrap(),
            target: h3_battle::CastTarget::Hex(target),
            second: h3_battle::CastTarget::None,
        };
        apply(&mut state, r, BattleSide::Attacker, cast, &mut NoopSink).unwrap();
        assert!(
            state
                .spell_obstacles()
                .iter()
                .any(|o| o.hexes.contains(target))
        );
    }
    let fly = Command::Move { to };
    apply(&mut state, r, BattleSide::Attacker, fly, &mut NoopSink).unwrap();
    assert!(acting(r, &state, walker));
    let command = h3_battle_ai::choose(&state, r, BattleSide::Defender, &mut Rng::from_seed(3));
    let Command::Attack { from, .. } = command else {
        panic!("{walker}: {command:?}");
    };
    (from, target)
}

// covers: BAT-AI-007, BAT-QUIRK-25
#[test]
fn a_walker_strikes_from_a_hex_off_the_fire_wall() {
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        // Without the wall the minotaurs (speed 6) strike from the nearest hex next to the
        // angel, the one on their row; on it the wall's value is taken off that hex's score
        // (BAT-AI-007), so they strike from another.
        let (from, target) = strike_hex(&r, "base:minotaur", false);
        assert_eq!(from, target, "{profile}");
        let (from, target) = strike_hex(&r, "base:minotaur", true);
        assert_ne!(from, target, "{profile}");
        // The efreet are immune to the wall: the original values its damage all the same (slip
        // 18) and avoids it; fixed, only the damage it does counts, none.
        let (from, target) = strike_hex(&r, "base:efreeti", true);
        assert_eq!(from == target, profile == "base:strict-fixed", "{profile}");
    }
}
