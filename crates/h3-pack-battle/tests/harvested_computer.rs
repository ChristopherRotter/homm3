//! The original's battle AI on the harvested packs (E23 T2a; ADR-0027 point 3, the epic's exit
//! criterion 1): in random battles of the SoD creatures, on the terrains' ordinary fields, on
//! every special battlefield and at a town of every faction's siege record at every fort level,
//! in both quirk profiles, at every difficulty and with random heroes (Tactics, Artillery, First
//! Aid, Ballistics, spells), the computer's choice for the side to act is a command the engine
//! accepts at every step (no fallback fires), and the same state and stream give the same
//! command. Most steps play it; the others play a random legal command or a hero's cast, so the
//! games reach effects and stand-ins (`h3_battle_ai`'s stubs) the policy alone would not. The
//! fair system actor's property on the same battles (E26 T3, BAT-AI-025: no fallback but
//! BAT-STUB-18, twins that differ in what the side may not know, the other side's hidden pits
//! among it). The Cyclops's wall shot (E23 T2b, BAT-AI-004) at a town's walls, below.

use std::cell::OnceCell;
use std::collections::BTreeSet;

use h3_battle::{
    BattleState, CastTargets, Command, LegalCommands, NoopSink, SpellObstacle, Status, apply,
    cast_targets, castable_spells, computer, legal_commands, observe, second_targets, validate,
};
use h3_content::{
    ContentKey, CreatureFlag, CreatureTypeId, FactionId, Formation, HeroTypeId, SpellCategory,
    SpellId,
};
use h3_content_battle::{BattleRuleset, WallKind};
use h3_contracts::{
    Army, ArmyStack, BattleHero, BattleSetup, BattleSide, CatapultControl, DamageBonuses,
    FortLevel, Moat, Rate, RetreatFacts, Siege,
};
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
        worn: h3_heroes::wearing(r, &[]),
        backpack: vec![],
        spells: vec![],
        mana: [0, 20, 100][pick(rng, 3)],
        bonuses: Default::default(),
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
            let id = r.artifact_by_key(artifact).unwrap();
            let at = h3_heroes::first_position(&d, r, id).unwrap();
            d.worn[at.0] = Some(ArtifactInstance::from(id));
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
    Some(cast_of(state, r, (side, spell), rng))
}

/// A cast of `spell`, which the hero of `side` may cast, at random targets.
fn cast_of(
    state: &BattleState,
    r: &BattleRuleset,
    (side, spell): (BattleSide, SpellId),
    rng: &mut Rng,
) -> Command {
    let (mut targets, mut seconds) = (CastTargets::new(), CastTargets::new());
    cast_targets(state, r, side, spell, &mut targets);
    let target = targets.as_slice()[pick(rng, targets.len())];
    second_targets(state, r, side, (spell, target), &mut seconds);
    let second = seconds.as_slice()[pick(rng, seconds.len())];
    Command::Cast {
        spell,
        target,
        second,
    }
}

/// What random battles draw from: the creatures but the war machines, the factions of the siege
/// records, the special battlefields.
struct Pools {
    creatures: Vec<CreatureTypeId>,
    factions: Vec<FactionId>,
    battlefields: Vec<ContentKey>,
}

impl Pools {
    /// The pools of the ruleset `sod`.
    fn of(sod: &BattleRuleset) -> Self {
        Self {
            creatures: (0..sod.creature_count())
                .filter_map(CreatureTypeId::from_index)
                .filter(|&c| {
                    sod.creature(c)
                        .is_some_and(|t| !t.flags.contains(CreatureFlag::WarMachine))
                })
                .collect(),
            factions: sod.sieges().iter().map(|s| s.faction).collect(),
            battlefields: sod.battlefields().iter().map(|b| b.key.clone()).collect(),
        }
    }
}

/// Game `game`'s random battle, drawn from `rng`: each side's hero (two in three, [`details`])
/// with its war machines; half the games at a town of the factions in turn (a random fort level,
/// moat, structures and Escape Tunnel), a quarter on a random special battlefield; random armies,
/// funds and `human`, the difficulty `game` mod 5.
fn random_setup(sod: &BattleRuleset, pools: &Pools, game: u64, rng: &mut Rng) -> BattleSetup {
    let details = [details(sod, rng), details(sod, rng)];
    let heroes = details
        .clone()
        .map(|d| (rng.below(3) != 0).then(|| derive_battle_hero(&d, sod).unwrap()));
    // Half the games at a town, a quarter on a special battlefield.
    let (siege, battlefield) = match game % 4 {
        0 | 1 => {
            let fort = [
                FortLevel::None,
                FortLevel::Fort,
                FortLevel::Citadel,
                FortLevel::Castle,
            ][pick(rng, 4)];
            let moat = if fort >= FortLevel::Citadel {
                [Moat::Absent, Moat::Harmless, Moat::Damaging][pick(rng, 3)]
            } else {
                Moat::Absent
            };
            let factions = &pools.factions;
            let town = Siege {
                faction: factions[usize::try_from(game).unwrap() % factions.len()],
                fort,
                moat,
                structures: u8::try_from(rng.below(256)).unwrap(),
                escape_tunnel: rng.below(2) == 0,
                bonuses: h3_contracts::TownBonuses::NONE,
            };
            (Some(town), None)
        }
        2 => (
            None,
            Some(pools.battlefields[pick(rng, pools.battlefields.len())].clone()),
        ),
        _ => (None, None),
    };
    let machines = [0, 1].map(|i| {
        if heroes[i].is_some() {
            war_machines(&details[i], sod)
        } else {
            vec![]
        }
    });
    BattleSetup {
        attacker: army(rng, &pools.creatures),
        defender: army(rng, &pools.creatures),
        battlefield,
        siege,
        war_machines: machines,
        heroes,
        formations: [Formation::Loose, Formation::Loose],
        funds: [rng.next_u32(), rng.next_u32()],
        difficulty: u8::try_from(game % 5).unwrap(),
        human: [rng.below(2) == 0, rng.below(2) == 0],
        ..BattleSetup::default()
    }
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn the_computers_choice_is_legal_in_random_battles_of_the_harvested_creatures() {
    let mut reach = Reach::default();
    for profile in ["base:faithful", "base:strict-fixed"] {
        let sod = sod(profile);
        let pools = Pools::of(&sod);
        let mut rng = Rng::from_seed(23);
        let mut seat = Rng::from_seed(2023);
        let mut legal = LegalCommands::new();
        for game in 0..48u64 {
            let setup = random_setup(&sod, &pools, game, &mut rng);
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

/// What [`the_fair_choice_reads_nothing_hidden_on_the_harvested_packs`] compared: the decisions
/// (each checked for a fallback and made twice) and the fallbacks known (BAT-STUB-18), those where
/// the side may know everything (the original's); the worlds of the battle with the other hero as
/// it is (another stream, the other side's hidden pits placed again), `pits` of them with such a
/// pit, the all-seeing command differing there `pit_differed` times; the twins of another setup
/// compared while the side's observations agreed, `with_hero` of them with another hero, the
/// all-seeing command differing there `differed` times, `pit_twins` with the other side's hidden
/// pits on other hexes; and the twins dropped when they parted.
#[derive(Debug, Default)]
struct Fair {
    decisions: u32,
    fallbacks: u32,
    original: u32,
    worlds: u32,
    pits: u32,
    pit_differed: u32,
    twins: u32,
    with_hero: u32,
    differed: u32,
    pit_twins: u32,
    parted: u32,
}

/// The spell obstacles of `side` its opponent does not see (BAT-MAG-031): its hidden pits.
fn hidden(state: &BattleState, side: BattleSide) -> Vec<SpellObstacle> {
    state
        .spell_obstacles()
        .iter()
        .filter(|o| o.side == side && !o.revealed)
        .copied()
        .collect()
}

/// Random retreat facts.
fn retreat(rng: &mut Rng) -> RetreatFacts {
    RetreatFacts {
        experience: rng.below(4000),
        artifacts: rng.below(2000),
        tavern: rng.below(4) != 0,
        wanted: rng.below(8) == 0,
    }
}

/// `hero` with another private part, what its opponent may not know: every spell but the
/// abilities in its book if it carries one (a superset, so that it accepts the hero's casts),
/// each at a random mastery but the `pits`' (the number and value of their hidden obstacles
/// follow it, which a side's world keeps: AI-STUB-1), half the time other damage bonuses, another
/// surrender discount. Its public part stays.
fn other_private(
    r: &BattleRuleset,
    hero: &BattleHero,
    pits: [SpellId; 2],
    rng: &mut Rng,
) -> BattleHero {
    let mut twin = hero.clone();
    if twin.magic.spell_book.is_some() {
        let book: Vec<SpellId> = (0..r.spells().len())
            .filter_map(SpellId::from_index)
            .filter(|&s| r.spell(s).unwrap().category != SpellCategory::Ability)
            .collect();
        let own = |s: SpellId| hero.magic.masteries.iter().find(|m| m.0 == s).map(|m| m.1);
        twin.magic.masteries = book
            .iter()
            .filter_map(|&s| {
                let drawn = u8::try_from(rng.below(4)).unwrap();
                let mastery = if pits.contains(&s) {
                    own(s)
                } else {
                    Some(drawn)
                };
                mastery.filter(|&m| m > 0).map(|m| (s, m))
            })
            .collect();
        twin.magic.spell_book = Some(book);
    }
    if rng.below(2) == 0 {
        twin.damage = DamageBonuses {
            ranged: Rate::bp(rng.below(5001)),
            melee: Rate::bp(rng.below(5001)),
            reduction: Rate::bp(rng.below(3001)),
        };
    }
    twin.surrender_discount_bp = u16::try_from(rng.below(5001)).unwrap();
    twin
}

/// A hero's cast for `side`: one of the `pits` half the time it may cast one, else a random cast
/// ([`cast`]).
fn pit_or_cast(
    state: &BattleState,
    r: &BattleRuleset,
    (side, pits): (BattleSide, [SpellId; 2]),
    rng: &mut Rng,
) -> Option<Command> {
    let castable = castable_spells(state, r, side);
    let pit = pits[pick(rng, 2)];
    if castable.contains(pit) && rng.below(2) == 0 {
        return Some(cast_of(state, r, (side, pit), rng));
    }
    cast(state, r, side, rng)
}

/// Whether `command` is the active stack's shot at itself, a hypnotized shooter's: the computer
/// takes every stack of the other side as a target, whoever controls it, which the engine refuses
/// (BAT-STUB-18).
fn shoots_itself(state: &BattleState, command: Command) -> bool {
    let active = state.active();
    matches!(command, Command::Shoot { target, .. } if Some(target) == active)
        && active
            .and_then(|id| state.stack(id))
            .is_some_and(|s| s.spells.status.contains(Status::HYPNOTIZED))
}

/// Asserts that the fair command for `side` on `twin`, which `side` observes as the battle, from
/// `seat` is `fair` and leaves the stream at `after`, as on the battle.
fn fair_on_twin(
    r: &BattleRuleset,
    twin: &BattleState,
    (side, seat): (BattleSide, &Rng),
    (fair, after): (Command, &Rng),
    game: &str,
) {
    let mut on = seat.clone();
    assert_eq!(
        h3_battle_ai::choose_fair(twin, r, side, &mut on),
        fair,
        "{game}"
    );
    assert_eq!(&on, after, "{game}");
}

/// E26 T3, exit criteria 1 and 2 on the harvested packs (beside the property above): random
/// battles of the SoD creatures ([`random_setup`]: the field, the special battlefields, sieges of
/// every faction), both profiles, every difficulty, with random heroes and retreat facts. At every
/// step the original's choice on the side to act's world is a command the engine accepts on the
/// battle (no fallback; the one known, BAT-STUB-18, gives Defend) and is the fair command, made
/// twice from the same stream; where the side may know everything (no other hero, no hidden pit
/// of the other side) both are the original's. Twins that differ only in what the side may not
/// know give the same fair command and leave the stream the same: at every step the battle's
/// world for the side with the other hero as it is (another stream, the other side's hidden
/// Quicksand and Land Mines placed again, their observations asserted equal first), and the twin
/// battle of another setup (the other side's hero's private part, funds and retreat facts, the
/// defender's Escape Tunnel), played on with the same commands while the side's observations of
/// the two agree; a cast of Quicksand or Land Mine draws its hexes from another stream in the
/// twin of the caster's side. Six steps in ten play the fair command, the others a hero's cast
/// (Quicksand or Land Mine half the time it may) or a random legal command.
// covers: BAT-AI-025
#[test]
fn the_fair_choice_reads_nothing_hidden_on_the_harvested_packs() {
    let mut fair = Fair::default();
    for profile in ["base:faithful", "base:strict-fixed"] {
        let sod = sod(profile);
        let pools = Pools::of(&sod);
        let pits = ["base:quicksand", "base:land_mine"].map(|k| sod.spell_by_key(k).unwrap());
        let mut rng = Rng::from_seed(26);
        let mut seat = Rng::from_seed(2026);
        let mut legal = LegalCommands::new();
        for game in 0..20u64 {
            let at = |game| format!("{profile} game {game}");
            let mut setup = random_setup(&sod, &pools, game, &mut rng);
            setup.retreat = [retreat(&mut rng), retreat(&mut rng)];
            let twin_setups = [0, 1].map(|i| {
                let mut twin = setup.clone();
                twin.heroes[i] = setup.heroes[i]
                    .as_ref()
                    .map(|h| other_private(&sod, h, pits, &mut rng));
                twin.funds[i] = rng.next_u32();
                twin.retreat[i] = retreat(&mut rng);
                if let Some(town) = twin.siege.as_mut().filter(|_| i == 1) {
                    town.escape_tunnel = !town.escape_tunnel;
                }
                twin
            });
            let mut state = BattleState::new(&sod, &setup, game).unwrap();
            let mut twins = twin_setups.map(|s| Some(BattleState::new(&sod, &s, game).unwrap()));
            for _ in 0..200u32 {
                let Some(side) = state.side_to_act() else {
                    break;
                };
                let other = side.opponent();
                let world = state
                    .determinized(&sod, side, None, seat.clone().next_u64())
                    .unwrap();
                let mut again = seat.clone();
                let proposed = h3_battle_ai::unchecked(&world, &sod, side, &mut again);
                let refused = validate(&state, &sod, side, proposed).is_err();
                // stub: BAT-STUB-18
                assert!(
                    !refused || shoots_itself(&state, proposed),
                    "{}: the fair {proposed:?} is refused",
                    at(game)
                );
                fair.fallbacks += u32::from(refused);
                let mut after = seat.clone();
                let command = h3_battle_ai::choose_fair(&state, &sod, side, &mut after);
                let expected = if refused { Command::Defend } else { proposed };
                assert_eq!(command, expected, "{}", at(game));
                assert_eq!(after, again, "{}", at(game));
                fair.decisions += 1;
                let unseen = hidden(&state, other);
                let real = setup.heroes[other.index()].as_ref();
                // The all-seeing command, once.
                let original = OnceCell::new();
                let all_seeing = || {
                    *original
                        .get_or_init(|| h3_battle_ai::choose(&state, &sod, side, &mut seat.clone()))
                };
                if real.is_none() && unseen.is_empty() {
                    let mut stream = seat.clone();
                    let _ = original.set(h3_battle_ai::choose(&state, &sod, side, &mut stream));
                    assert_eq!(command, all_seeing(), "{}", at(game));
                    assert_eq!(after, stream, "{}", at(game));
                    fair.original += 1;
                }
                let seen = observe(&state, &sod, side);
                let decision = (command, &after);
                let twin = state
                    .determinized(&sod, side, real, rng.next_u64())
                    .unwrap();
                assert_eq!(observe(&twin, &sod, side), seen, "{}", at(game));
                fair_on_twin(&sod, &twin, (side, &seat), decision, &at(game));
                fair.worlds += 1;
                if !unseen.is_empty() {
                    fair.pits += 1;
                    fair.pit_differed += u32::from(
                        h3_battle_ai::choose(&twin, &sod, side, &mut seat.clone()) != all_seeing(),
                    );
                }
                match twins[other.index()]
                    .as_ref()
                    .map(|t| (observe(t, &sod, side) == seen, t))
                {
                    Some((true, twin)) => {
                        fair_on_twin(&sod, twin, (side, &seat), decision, &at(game));
                        fair.twins += 1;
                        fair.pit_twins += u32::from(hidden(twin, other) != unseen);
                        if real.is_some() {
                            fair.with_hero += 1;
                            fair.differed += u32::from(
                                h3_battle_ai::choose(twin, &sod, side, &mut seat.clone())
                                    != all_seeing(),
                            );
                        }
                    }
                    Some((false, _)) => {
                        twins[other.index()] = None;
                        fair.parted += 1;
                    }
                    None => {}
                }
                seat = after;
                let roll = rng.below(10);
                let played = if roll < 6 {
                    command
                } else if let Some(cast) =
                    pit_or_cast(&state, &sod, (side, pits), &mut rng).filter(|_| roll < 9)
                {
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
                // The caster's twin draws a pit's hexes from another stream, then all go on alike.
                let pit = matches!(played, Command::Cast { spell, .. } if pits.contains(&spell));
                if pit {
                    let k = rng.next_u64();
                    state.reseed(k);
                    for (i, twin) in twins.iter_mut().enumerate() {
                        if let Some(t) = twin {
                            t.reseed(if i == side.index() { k ^ 1 } else { k });
                        }
                    }
                }
                apply(&mut state, &sod, side, played, &mut NoopSink)
                    .unwrap_or_else(|e| panic!("{}: {played:?} is rejected: {e:?}", at(game)));
                for twin in &mut twins {
                    if twin
                        .as_mut()
                        .is_some_and(|t| apply(t, &sod, side, played, &mut NoopSink).is_err())
                    {
                        *twin = None;
                        fair.parted += 1;
                    }
                }
                if pit {
                    let k = rng.next_u64();
                    state.reseed(k);
                    twins.iter_mut().flatten().for_each(|t| t.reseed(k));
                }
            }
        }
    }
    // 1728 decisions, the all-seeing command differing on 8 twins of 833 with another hero
    // (E26-log.md T3).
    assert!(
        fair.decisions > 1500
            && fair.original > 500
            && fair.pits > 80
            && fair.with_hero > 600
            && fair.pit_twins > 20
            && fair.differed > 4,
        "{fair:?}"
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
            bonuses: h3_contracts::TownBonuses::NONE,
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

// covers: BAT-AI-021
#[test]
fn a_commanded_catapult_aims_at_the_gate_then_the_weakest_wall_while_all_stand() {
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod(profile);
        let field = r.siege_field().unwrap();
        // An attacking hero with Basic Ballistics commands its Catapult at a Fort (walls and
        // gate, no turrets): the Catapult acts first in every round.
        let mut hero = BattleHero::new(HeroTypeId::from_index(0).unwrap());
        hero.war_machines.catapult = Some(CatapultControl { level: 1 });
        let setup = BattleSetup {
            attacker: harvested(&r, &[("base:pikeman", 50)]),
            defender: harvested(&r, &[("base:pikeman", 50)]),
            heroes: [Some(hero), None],
            siege: Some(Siege {
                faction: r.sieges()[0].faction,
                fort: FortLevel::Fort,
                moat: Moat::Absent,
                structures: 0,
                escape_tunnel: false,
                bonuses: h3_contracts::TownBonuses::NONE,
            }),
            formations: [Formation::Loose, Formation::Loose],
            difficulty: 1,
            ..BattleSetup::default()
        };
        let (mut gate, mut weakest, mut rest) = (0, 0, 0);
        // A shot that misses the gate lands on a wall: over a few battles the gate falls with
        // every wall standing too.
        for seed in 0..8 {
            let mut state = BattleState::new(&r, &setup, seed).unwrap();
            let mut seat = Rng::from_seed(5);
            for _ in 0..300 {
                let Some(side) = state.side_to_act() else {
                    break;
                };
                let id = state.active().unwrap();
                if !state.stack(id).unwrap().is_catapult() {
                    apply(&mut state, &r, side, Command::Defend, &mut NoopSink).unwrap();
                    continue;
                }
                let siege = *computer::siege(&state).unwrap();
                let gate_target = (0..siege.targets)
                    .find(|&t| siege.kind(t) == Some(WallKind::Gate))
                    .unwrap();
                let walls: Vec<(u8, u8)> = computer::walls(&state).collect();
                let command = h3_battle_ai::choose(&state, &r, side, &mut seat);
                let Command::ShootWall { part } = command else {
                    panic!("{profile}: {command:?}");
                };
                if siege.stands(gate_target) {
                    assert_eq!(part, gate_target, "{profile}");
                    gate += 1;
                } else if walls.len() == field.automatic.len() {
                    let least = walls.iter().map(|w| w.1).min();
                    assert!(
                        walls.iter().any(|w| w.0 == part && Some(w.1) == least),
                        "{profile}: {part} of {walls:?}"
                    );
                    weakest += 1;
                } else {
                    let first = field
                        .fallback
                        .iter()
                        .copied()
                        .chain(0..siege.targets)
                        .find(|&t| siege.stands(t));
                    assert_eq!(Some(part), first, "{profile}: {walls:?}");
                    rest += 1;
                }
                apply(&mut state, &r, side, command, &mut NoopSink).unwrap();
            }
        }
        assert!(
            gate > 0 && weakest > 0 && rest > 0,
            "{profile}: {gate} {weakest} {rest}"
        );
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
fn fire_mage(r: &BattleRuleset) -> BattleHero {
    let d = HeroDetails {
        hero_type: HeroTypeId::from_index(0).unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [0, 0, 5, 10],
        skills: vec![],
        worn: h3_heroes::wearing(
            r,
            &[ArtifactInstance::from(
                r.artifact_by_key("base:spell_book").unwrap(),
            )],
        ),
        backpack: vec![],
        spells: vec![r.spell_by_key("base:fire_wall").unwrap()],
        mana: 100,
        bonuses: Default::default(),
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

// covers: BAT-AI-007
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

// A bound walker (BAT-MAG-045) values only the enemies next to it (0x421680's marks at 0; the
// walker's choice, BAT-AI-005): the Dendroid Guards bind the Pikemen that attacked them, and the
// Pikemen's next choice is legal and strikes a Dendroid stack beside them or does not move. A fixed
// game for the line the random battles reach only by chance.
// covers: BAT-AI-005
#[test]
fn a_bound_walker_values_only_the_enemies_beside_it() {
    let r = sod("base:faithful");
    let mut state = walled(
        &r,
        &[("base:pikeman", 20)],
        &[("base:dendroid_guard", 10)],
        false,
    );
    let pikemen = h3_battle::StackId(0);
    let mut legal = LegalCommands::new();
    for _ in 0..40 {
        let side = state.side_to_act().unwrap();
        if computer::bound(&state, pikemen) && state.active() == Some(pikemen) {
            let command = h3_battle_ai::choose(&state, &r, side, &mut Rng::from_seed(1));
            assert!(validate(&state, &r, side, command).is_ok(), "{command:?}");
            assert!(
                !matches!(command, Command::Move { .. }),
                "a bound walker does not move: {command:?}"
            );
            return;
        }
        legal_commands(&state, &r, side, &mut legal);
        let attack = legal
            .iter()
            .copied()
            .find(|c| matches!(c, Command::Attack { .. }));
        // Otherwise the attacker walks the furthest it may towards the defender, which waits.
        let command = attack.unwrap_or_else(|| {
            if side == BattleSide::Attacker {
                legal
                    .iter()
                    .copied()
                    .filter_map(|c| match c {
                        Command::Move { to } => Some((to.0 % 17, c)),
                        _ => None,
                    })
                    .max_by_key(|&(x, _)| x)
                    .map_or(Command::Defend, |(_, c)| c)
            } else {
                Command::Defend
            }
        });
        apply(&mut state, &r, side, command, &mut NoopSink).unwrap();
    }
    panic!("the pikemen were never bound on their turn");
}
