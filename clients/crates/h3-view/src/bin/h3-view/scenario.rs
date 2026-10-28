//! A scenario file to a record (E17 M1 T7; ADR-0020 point 8): the battle a scenario file sets up,
//! as `h3 run` reads it (its packs compiled with its profile, obstacles generated from the seed
//! when it asks), hosted by `h3-protocol`'s session in-process, and the record of a spectator who
//! joined it: the manifest and the snapshot at the battle's start. An adapter: it reads files.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use h3_battle::{BattleSetup, BattleSide, BattleState, NoopSink, Viewer};
use h3_content::{CompiledHeroEffect, HeroEffect};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, FortLevel, MachineControl};
use h3_kernel::{Rng, StreamId};
use h3_pack_battle::ScenarioDef;
use h3_protocol::session::{ConnId, GameConfig, SeatKind, Session};
use h3_protocol::wire::{ClientMessage, Command, Manifest, Observation, Seat, ServerMessage};
use h3_protocol::{codec, map};
use h3_view::mouse::aim::Aim;

/// The game's id in the in-process session.
const GAME: u32 = 1;

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The texts of the packs `ids`: `<packs>/<id>/core.toml` and, where it exists, `battle.toml`,
/// as `h3_cli::load_ruleset` reads them.
fn pack_texts(packs: &Path, ids: &[String]) -> Result<Vec<(String, String)>, String> {
    let mut texts = Vec::new();
    for id in ids {
        if id.is_empty() || id.contains(['/', '\\', '.', ':']) {
            return Err(format!("invalid pack id `{id}`"));
        }
        let dir = packs.join(id);
        let battle = dir.join("battle.toml");
        let battle = if battle.is_file() {
            read(&battle)?
        } else {
            String::new()
        };
        texts.push((read(&dir.join("core.toml"))?, battle));
    }
    Ok(texts)
}

/// The ruleset a record's `manifest` names, its packs read from `packs` (E17 M3 T7b-2a: what a
/// spell's effect is).
pub fn ruleset(packs: &Path, manifest: &Manifest) -> Result<BattleRuleset, String> {
    let ids: Vec<String> = manifest.packs.iter().map(|p| p.id.clone()).collect();
    let texts = pack_texts(packs, &ids)?;
    let refs: Vec<(&str, &str)> = texts
        .iter()
        .map(|(c, b)| (c.as_str(), b.as_str()))
        .collect();
    h3_pack_battle::compile(&refs, &manifest.profile)
        .map_err(|e| format!("{}: {e}", packs.display()))
}

/// The battle `scenario` sets up with `seed`, its packs read from `packs`: the ruleset its packs
/// compile to with its profile, and the setup (obstacles generated from the seed when it asks).
pub fn battle(
    scenario: &Path,
    packs: &Path,
    seed: u64,
) -> Result<(BattleRuleset, BattleSetup), String> {
    let def = definition(scenario)?;
    let ruleset = compiled(&def, packs)?;
    let setup = set_up(&def, &ruleset, seed).map_err(|e| format!("{}: {e}", scenario.display()))?;
    Ok((ruleset, setup))
}

/// The scenario file's definition.
pub fn definition(scenario: &Path) -> Result<ScenarioDef, String> {
    h3_pack_battle::parse_scenario(&read(scenario)?)
        .map_err(|e| format!("{}: {e}", scenario.display()))
}

/// The ruleset `def`'s packs (read from `packs`) compile to with its profile.
pub fn compiled(def: &ScenarioDef, packs: &Path) -> Result<BattleRuleset, String> {
    let texts = pack_texts(packs, &def.packs)?;
    let refs: Vec<(&str, &str)> = texts
        .iter()
        .map(|(c, b)| (c.as_str(), b.as_str()))
        .collect();
    h3_pack_battle::compile(&refs, &def.profile).map_err(|e| format!("{}: {e}", packs.display()))
}

/// The setup `def` resolves to in `ruleset` with `seed` (its obstacles generated from the seed
/// when it asks).
pub fn set_up(
    def: &ScenarioDef,
    ruleset: &BattleRuleset,
    seed: u64,
) -> Result<BattleSetup, String> {
    let mut setup = def.resolve(ruleset).map_err(|e| e.to_string())?;
    if def.random_obstacles {
        // For the terrain and the special battlefield, as `h3 run` (E17 M3 T2).
        let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
        setup.obstacles = h3_battle::generate_setup_obstacles(ruleset, &setup, &mut rng);
    }
    Ok(setup)
}

/// What the original's map needs of a scenario's battle (E17 M2 T1, `situation`): its terrain's
/// key, its special battlefield's (M3 T2), each side's stacks (slot, the creature's singular
/// name, count) and formation, its hero (M3 T3a) and its war machines (M3 T3b).
#[derive(Debug, PartialEq, Eq)]
pub struct Sides {
    pub terrain: Option<String>,
    /// The special battlefield's key (E17 M3 T2).
    pub battlefield: Option<String>,
    pub stacks: [Vec<(u8, String, u32)>; 2],
    pub tight: [bool; 2],
    pub heroes: [Option<SideHero>; 2],
    /// Whether each side's player is human (the scenario's `human`; E17 M3 T6a).
    pub human: [bool; 2],
    /// Each side's war machines as its hero's artifacts (E17 M3 T3b): the worn slot (13 to 16,
    /// `ArtSlots.txt`'s order) and the artifact's number, which is the original's when the packs
    /// come in their order (base first: the Ballista 4, the Ammo Cart 5, the First Aid Tent 6).
    pub machines: [Vec<(usize, u16)>; 2],
    /// The town the battle is fought at (E17 M4 T1b): its town type (the faction's index in the
    /// ruleset, the original's when the packs come in their order) and fort level (1 Fort, 2
    /// Citadel, 3 Castle; 0 none).
    pub siege: Option<(usize, u8)>,
}

/// A side's hero for the original's map (E17 M3 T3a): its hero type's and class's numbers in
/// the ruleset, which are the original's when its packs come in their order (base, ab, sod, as
/// `harvest` keys the heroes; `situation` checks the name against `hotraits.txt`), its name, its
/// attack, defense, power and knowledge, its spell book (M3 T7a-2a) and the secondary skills
/// behind its battle effects (M3 T7b-2b-2a: [`effect_skills`], each the skill's number and
/// level).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideHero {
    pub hero_type: usize,
    pub class: usize,
    pub name: String,
    pub primary: [u8; 4],
    pub book: Option<Book>,
    pub skills: Vec<(u8, u8)>,
}

/// A hero's spell book for the original's map (E17 M3 T7a-2a): the book's artifact number
/// (the original's when the packs come in their order, base first: 0), worn in the map's slot
/// 17, and its spells' names (the core ruleset's; `situation` numbers them by `sptraits.txt`:
/// the SoD pack adds 57, Titan's Lightning Bolt, after the base pack's others, so a handle is
/// the original's number only below 57). The original's hero
/// starts the map with 10 spell points per knowledge (no Intelligence), so `sides` refuses a
/// scenario whose hero's `mana` differs. `skills` (E17 M3 T7a-2b): the school skills that give
/// the hero the scenario's masteries in the original ([`school_skills`]), each the skill's
/// number and level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    pub artifact: u16,
    pub spells: Vec<String>,
    pub skills: Vec<(u8, u8)>,
}

/// The school skills behind a hero's masteries on the original's map (E17 M3 T7a-2b,
/// *provisional*): the original takes a spell's mastery from the hero's best skill among the
/// spell's schools (HER-SPL-003), so each school gets the level the scenario's `masteries` ask of
/// it, the spells of one school first (a spell of several raises its first school only when none
/// of them gives it yet). Checked against every mastery named and every spell of the book (none
/// named: none given). The skill of school `<pack>:<name>` is `<pack>:<name>_magic`, its number
/// the ruleset's, which is the original's (`sstraits.txt`'s order). `masteries` and `book` by the
/// spells' indices; the skills' numbers and levels, by number.
pub fn school_skills(
    ruleset: &BattleRuleset,
    masteries: &[(usize, u8)],
    book: &[usize],
) -> Result<Vec<(u8, u8)>, String> {
    let core = &**ruleset;
    let schools = |spell: usize| -> Vec<usize> {
        core.spells()
            .get(spell)
            .map_or_else(Vec::new, |s| s.schools.iter().map(|l| l.index()).collect())
    };
    let best = |levels: &BTreeMap<usize, u8>, spell: usize| {
        schools(spell)
            .iter()
            .map(|l| levels.get(l).copied().unwrap_or(0))
            .max()
            .unwrap_or(0)
    };
    let mut asked = masteries.to_vec();
    asked.sort_by_key(|&(spell, _)| (schools(spell).len(), spell));
    let mut levels = BTreeMap::new();
    for &(spell, level) in &asked {
        if let Some(&first) = schools(spell)
            .first()
            .filter(|_| best(&levels, spell) < level)
        {
            levels.insert(first, level);
        }
    }
    for spell in masteries
        .iter()
        .map(|&(s, _)| s)
        .chain(book.iter().copied())
    {
        let wanted = masteries
            .iter()
            .find(|&&(s, _)| s == spell)
            .map_or(0, |&(_, l)| l);
        if best(&levels, spell) != wanted {
            let name = core.spells().get(spell).map_or("?", |s| s.name.as_str());
            return Err(format!(
                "{name}: mastery {wanted}, but the school skills give {}",
                best(&levels, spell)
            ));
        }
    }
    levels
        .into_iter()
        .map(|(school, level)| {
            let key = core
                .spell_schools()
                .get(school)
                .map_or("", |s| s.key.as_str());
            let skill = format!("{key}_magic");
            let id = core
                .skill_by_key(&skill)
                .ok_or_else(|| format!("no skill `{skill}` for school `{key}`"))?;
            let number = u8::try_from(id.index()).map_err(|_| format!("{skill}: number"))?;
            Ok((number, level))
        })
        .collect()
}

/// The secondary skills behind a hero's battle effects on the original's map (E17 M3 T7b-2b-2a,
/// *provisional*; the scenario's `effects`, a derived hero's morale and luck): for each effect the
/// skill and level whose effects are that effect alone (the packs' Leadership for a morale change
/// of 1 to 3 to the own stacks, Luck for a luck change), the skill's number the ruleset's, which
/// is the original's (`sstraits.txt`'s order). An effect no skill level gives alone, or two of one
/// skill, the map cannot give the hero: an error. By number.
pub fn effect_skills(
    ruleset: &BattleRuleset,
    effects: &[BattleEffect],
) -> Result<Vec<(u8, u8)>, String> {
    let core = &**ruleset;
    let mut skills: Vec<(u8, u8)> = Vec::new();
    for effect in effects {
        let wanted: CompiledHeroEffect = match *effect {
            BattleEffect::Morale { amount, affects } => HeroEffect::Morale { amount, affects },
            BattleEffect::Luck { amount, affects } => HeroEffect::Luck { amount, affects },
            ref other => {
                return Err(format!(
                    "{other:?}: no skill gives it on the original's map"
                ));
            }
        };
        let (number, level) = core
            .skills()
            .iter()
            .enumerate()
            .find_map(|(i, skill)| {
                (1..=3)
                    .find(|&l| skill.effects(l) == [wanted.clone()])
                    .map(|l| (i, l))
            })
            .ok_or_else(|| format!("{effect:?}: no skill level gives it alone"))?;
        let number = u8::try_from(number).map_err(|_| format!("skill number {number}"))?;
        if skills.iter().any(|&(n, _)| n == number) {
            return Err(format!("{effect:?}: a second effect of skill {number}"));
        }
        skills.push((number, level));
    }
    skills.sort_unstable();
    Ok(skills)
}

/// The [`Sides`] of the battle `scenario` sets up, its packs read from `packs`.
pub fn sides(scenario: &Path, packs: &Path) -> Result<Sides, String> {
    let (ruleset, setup) = battle(scenario, packs, 1)?;
    let terrain = setup
        .terrain
        .and_then(|t| ruleset.terrains().get(t.index()))
        .map(|t| t.key.as_str().to_owned());
    let stacks = [&setup.attacker, &setup.defender].map(|army| {
        army.stacks
            .iter()
            .map(|s| {
                // The core ruleset's creature (its name), not the battle's (its numbers).
                let name = (*ruleset).creature(s.creature).map(|c| c.name.clone());
                (s.slot, name.unwrap_or_default(), s.count)
            })
            .collect()
    });
    let tight = h3_pack_battle::tight_sides(setup.formations);
    // The spell book's artifact (E17 M3 T7a-2a): the one worn in the `spell_book` slot.
    let book_artifact = || -> Result<u16, String> {
        let id = ruleset
            .artifacts()
            .iter()
            .position(|a| {
                a.slot
                    .and_then(|s| ruleset.artifact_slot(s))
                    .is_some_and(|s| s.key.as_str().rsplit(':').next() == Some("spell_book"))
            })
            .ok_or("no artifact is worn in the `spell_book` slot")?;
        u16::try_from(id).map_err(|_| format!("the spell book: number {id}"))
    };
    let hero = |side: usize| -> Result<Option<SideHero>, String> {
        let Some(h) = &setup.heroes[side] else {
            return Ok(None);
        };
        let Some(t) = (*ruleset).hero_type(h.hero_type) else {
            return Ok(None);
        };
        let book = match &h.magic.spell_book {
            None => None,
            Some(spells) => {
                let mana = 10 * u16::from(h.knowledge);
                if h.magic.mana != mana {
                    return Err(format!(
                        "{}: mana {}, but the original's hero starts with {mana} (10 per knowledge)",
                        t.name, h.magic.mana
                    ));
                }
                let indices: Vec<usize> = spells.iter().map(|s| s.index()).collect();
                // The core ruleset's spell (its name), as the stacks' creatures.
                let spells = spells
                    .iter()
                    .map(|&s| {
                        (*ruleset)
                            .spell(s)
                            .map_or(String::new(), |s| s.name.clone())
                    })
                    .collect();
                let masteries: Vec<(usize, u8)> = h
                    .magic
                    .masteries
                    .iter()
                    .map(|&(s, l)| (s.index(), l))
                    .collect();
                Some(Book {
                    artifact: book_artifact()?,
                    spells,
                    skills: school_skills(&ruleset, &masteries, &indices)?,
                })
            }
        };
        let mut skills =
            effect_skills(&ruleset, &h.effects).map_err(|e| format!("{}: {e}", t.name))?;
        skills.extend(machine_skills(&ruleset, &h.war_machines)?);
        skills.sort_unstable();
        Ok(Some(SideHero {
            hero_type: h.hero_type.index(),
            class: t.class.index(),
            name: t.name.clone(),
            primary: [h.attack, h.defense, h.power, h.knowledge],
            book,
            skills,
        }))
    };
    let heroes = [hero(0)?, hero(1)?];
    // Each war machine's artifact (E17 M3 T3b): the one that places the machine, worn in its
    // `war_machine_<n>` slot, the map's slot 12 + n.
    let artifact = |machine| -> Result<(usize, u16), String> {
        let (id, artifact) = ruleset
            .artifacts()
            .iter()
            .enumerate()
            .find(|(_, a)| a.war_machine == Some(machine))
            .ok_or("a war machine no artifact places")?;
        let slot = artifact
            .slot
            .and_then(|s| ruleset.artifact_slot(s))
            .map_or("", |s| s.key.as_str());
        let n: usize = slot
            .rsplit("war_machine_")
            .next()
            .and_then(|n| n.parse().ok())
            .filter(|n| (1..=4).contains(n))
            .ok_or_else(|| format!("{}: slot `{slot}` is no war machine's", artifact.key))?;
        let id = u16::try_from(id).map_err(|_| format!("{}: number {id}", artifact.key))?;
        Ok((12 + n, id))
    };
    let machines = |side: usize| -> Result<Vec<(usize, u16)>, String> {
        setup.war_machines[side]
            .iter()
            .map(|&m| artifact(m))
            .collect()
    };
    Ok(Sides {
        terrain,
        battlefield: setup.battlefield.as_ref().map(|k| k.as_str().to_owned()),
        stacks,
        tight: ["attacker", "defender"].map(|side| tight.iter().any(|t| t == side)),
        heroes,
        human: setup.human,
        machines: [machines(0)?, machines(1)?],
        siege: setup.siege.map(|s| {
            let fort = match s.fort {
                FortLevel::None => 0,
                FortLevel::Fort => 1,
                FortLevel::Citadel => 2,
                FortLevel::Castle => 3,
            };
            (s.faction.index(), fort)
        }),
    })
}

/// The skills behind a hero's command of its war machines on the original's map (E17 M4 T1b):
/// Artillery for its Ballista and a town's turrets (BAT-SIE-016), Ballistics for a siege's
/// Catapult (BAT-SIE-011), each at the control's level, its number the ruleset's (the original's,
/// `sstraits.txt`'s order). A siege's situation commands both, so that no war machine acts on its
/// own before the first command: the engine's record opens after such acts (`BattleState::new`
/// drops their events), the original's first frame before them. First Aid has none here.
fn machine_skills(
    ruleset: &BattleRuleset,
    control: &MachineControl,
) -> Result<Vec<(u8, u8)>, String> {
    let wanted = [
        ("artillery", control.ballista.map(|b| b.level)),
        ("ballistics", control.catapult.map(|c| c.level)),
    ];
    if control.tent.is_some() {
        return Err("a commanded First Aid Tent: no skill on the original's map here".into());
    }
    let mut skills = Vec::new();
    for (name, level) in wanted {
        let Some(level) = level else {
            continue;
        };
        let number = (**ruleset)
            .skills()
            .iter()
            .position(|s| s.key.as_str().rsplit(':').next() == Some(name))
            .and_then(|n| u8::try_from(n).ok())
            .ok_or_else(|| format!("no skill `{name}` in the ruleset"))?;
        skills.push((number, level));
    }
    Ok(skills)
}

/// The record of `seat` (a spectator's, or a side's: what that side sees, its own hidden objects
/// included; E17 M4 T1b, a Tower's mines on the defender's turn) of the battle `scenario` sets
/// up with `seed`, its packs read from `packs`.
pub fn record_of(scenario: &Path, packs: &Path, seed: u64, seat: Seat) -> Result<String, String> {
    played_as(scenario, packs, seed, &[], seat)
}

/// The legal commands of the side to act at the opening of the battle `scenario` sets up with
/// `seed`, its packs read from `packs` (E17 M3 T6c-1: what the mouse's cursor asks in a still);
/// none when no side acts.
pub fn opening_legal(scenario: &Path, packs: &Path, seed: u64) -> Result<Vec<Command>, String> {
    let (ruleset, state) = opening(scenario, packs, seed)?;
    Ok(state
        .side_to_act()
        .map(|side| map::legal_commands(&state, &ruleset, side))
        .unwrap_or_default())
}

/// The spells of the side to act's hero that the aim's entry tells affect no one at the opening
/// of `scenario` (E17 M5 T4a; `enter_player_cast` 0x59F085): the stack spells and Teleport no hex
/// takes (the engine gives no aim), by handle, among `spells` (their original numbers).
pub fn opening_no_one(
    scenario: &Path,
    packs: &Path,
    seed: u64,
    spells: &[(u16, u32)],
) -> Result<Vec<u16>, String> {
    let (ruleset, state) = opening(scenario, packs, seed)?;
    let Some(side) = state.side_to_act() else {
        return Ok(Vec::new());
    };
    Ok(spells
        .iter()
        .filter(|&&(handle, original)| {
            crate::boxes::affects_someone_only(original)
                && crate::aims::hero(&state, &ruleset, side, handle).is_none()
        })
        .map(|&(handle, _)| handle)
        .collect())
}

/// The ruleset and the opening state of `scenario`'s battle.
fn opening(
    scenario: &Path,
    packs: &Path,
    seed: u64,
) -> Result<(BattleRuleset, BattleState), String> {
    let (ruleset, setup) = battle(scenario, packs, seed)?;
    let state =
        BattleState::new(&ruleset, &setup, seed).map_err(|e| format!("the setup: {e:?}"))?;
    Ok((ruleset, state))
}

/// A turn of a battle as the mouse asks it (E17 M3 T6c-2; M5 T3b): a spectator's observation,
/// the side to act's legal commands and the spells it may aim there (their engine parts,
/// [`crate::aims`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Turned {
    /// The spectator's observation.
    pub observation: Observation,
    /// The side to act's legal commands.
    pub commands: Vec<Command>,
    /// The spells it may aim: its hero's of the ones asked, its Faerie Dragon's drawn one.
    pub aims: Vec<Aim>,
}

impl Turned {
    /// The turn of the side `side` to act in `state`, its hero's aims of `spells`.
    pub fn of(
        state: &BattleState,
        ruleset: &BattleRuleset,
        side: BattleSide,
        spells: &[u16],
    ) -> Self {
        let observation = map::observe(state, ruleset, Viewer::Spectator);
        let commands = map::legal_commands(state, ruleset, side);
        let own = map::observe(state, ruleset, Viewer::Side(side));
        let mut aims: Vec<Aim> = spells
            .iter()
            .filter_map(|&spell| crate::aims::hero(state, ruleset, side, spell))
            .collect();
        aims.extend(crate::aims::dragon(state, ruleset, side, &own));
        Self {
            observation,
            commands,
            aims,
        }
    }
}

/// The battle `scenario` sets up with `seed` before each of `commands` and after the last, while
/// a side is to act (E17 M3 T6c-2: what the mouse's cursor asks at each turn): a spectator's
/// observation and the legal commands of the side to act, with the aims of `spells` (E17 M5 T3b).
/// A rejected command is an error.
pub fn turned(
    scenario: &Path,
    packs: &Path,
    seed: u64,
    commands: &[(Seat, Command)],
    spells: &[u16],
) -> Result<Vec<Turned>, String> {
    let (ruleset, setup) = battle(scenario, packs, seed)?;
    let mut state =
        BattleState::new(&ruleset, &setup, seed).map_err(|e| format!("the setup: {e:?}"))?;
    let mut out = Vec::new();
    let mut commands = commands.iter();
    while let Some(side) = state.side_to_act() {
        out.push(Turned::of(&state, &ruleset, side, spells));
        let Some(&(_, command)) = commands.next() else {
            break;
        };
        h3_battle::apply(
            &mut state,
            &ruleset,
            side,
            map::command_to_engine(command),
            &mut NoopSink,
        )
        .map_err(|e| format!("command {} ({command:?}): {e:?}", out.len()))?;
    }
    Ok(out)
}

/// How the battle `scenario` sets up with `seed` ends after `commands`, each for the side to act
/// (E17 M5 T4b1: the result window of a sequence or a still): the engine's outcome, with its
/// ruleset; none while a side is still to act. A rejected command is an error.
pub fn outcome(
    scenario: &Path,
    packs: &Path,
    seed: u64,
    commands: &[Command],
) -> Result<(BattleRuleset, Option<h3_contracts::BattleOutcome>), String> {
    let (ruleset, mut state) = opening(scenario, packs, seed)?;
    for (n, &command) in commands.iter().enumerate() {
        let Some(side) = state.side_to_act() else {
            break;
        };
        h3_battle::apply(
            &mut state,
            &ruleset,
            side,
            map::command_to_engine(command),
            &mut NoopSink,
        )
        .map_err(|e| format!("command {} ({command:?}): {e:?}", n + 1))?;
    }
    let outcome = state.outcome();
    Ok((ruleset, outcome))
}

/// The most commands a fought battle may take.
pub(crate) const FOUGHT_LIMIT: usize = 5_000;

/// The record of a spectator of the battle `scenario` sets up with `seed`, fought to its end by
/// [`Greedy`] for both sides (E17 M2 T4b-2a, `h3-view play`): the engine plays the battle, then
/// the session replays its commands from the seats, as [`played`] does.
/// With it the commands each seat sent (E17 M3 T6c-2: the window asks the battle at each turn,
/// [`turned`]). Since E17 M5 T1b `play` and `window` fight it as they play it (`crate::live`):
/// this is the reference its test compares with.
#[cfg(test)]
pub fn fought(
    scenario: &Path,
    packs: &Path,
    seed: u64,
) -> Result<(String, Vec<(Seat, Command)>), String> {
    use crate::bot::Greedy;
    use h3_battle::{BattleSide, LegalCommands};
    use h3_protocol::session::Bot as _;

    let (ruleset, setup) = battle(scenario, packs, seed)?;
    let mut state =
        BattleState::new(&ruleset, &setup, seed).map_err(|e| format!("the setup: {e:?}"))?;
    let (mut bot, mut legal, mut commands) = (Greedy, LegalCommands::new(), Vec::new());
    while let Some(side) = state.side_to_act() {
        if commands.len() >= FOUGHT_LIMIT {
            return Err(format!("no end after {FOUGHT_LIMIT} commands"));
        }
        h3_battle::legal_commands(&state, &ruleset, side, &mut legal);
        let choice = bot.command(&state, &ruleset, side, &legal);
        h3_battle::apply(&mut state, &ruleset, side, choice, &mut NoopSink)
            .map_err(|e| format!("command {} ({choice:?}): {e:?}", commands.len() + 1))?;
        let seat = match side {
            BattleSide::Attacker => Seat::Attacker,
            BattleSide::Defender => Seat::Defender,
        };
        commands.push((seat, map::command(choice)));
    }
    let text = played(scenario, packs, seed, &commands)?;
    Ok((text, commands))
}

/// The host's configuration of a game of `setup` with `seed`: one spectator, every message
/// recorded.
fn config(setup: BattleSetup, seed: u64) -> GameConfig {
    GameConfig {
        game: GAME,
        setup,
        seed,
        checkpoint_every: 0,
        shadow: false,
        spectators: 1,
        watchable: true,
        record_bytes: usize::MAX,
        ranked: None,
    }
}

/// The record of the spectator on `viewer`.
pub(crate) fn fetch_record(session: &mut Session<'_>, viewer: ConnId) -> Result<String, String> {
    let mut out = Vec::new();
    session.handle(viewer, ClientMessage::FetchRecord { seq: 1 }, &mut out);
    for frame in out {
        if let Ok(ServerMessage::Record { record, .. }) =
            codec::decode_server(frame.frame.as_bytes(), usize::MAX)
        {
            return Ok(record);
        }
    }
    Err("the session sent no record".to_owned())
}

/// The record of a spectator of the battle `scenario` sets up with `seed` when each seat sends
/// its `commands` in order (E17 M2 T4a: the recorded sequences); a rejected command is an error.
pub fn played(
    scenario: &Path,
    packs: &Path,
    seed: u64,
    commands: &[(Seat, Command)],
) -> Result<String, String> {
    played_as(scenario, packs, seed, commands, Seat::Spectator)
}

/// [`played`], the record `seat`'s.
fn played_as(
    scenario: &Path,
    packs: &Path,
    seed: u64,
    commands: &[(Seat, Command)],
    seat: Seat,
) -> Result<String, String> {
    let (ruleset, setup) = battle(scenario, packs, seed)?;
    let seated = !commands.is_empty() || seat != Seat::Spectator;
    let mut session = hosted(&ruleset, setup, seed, seated)?;
    let viewer = VIEWER;
    for (seq, &(seat, command)) in (1..).zip(commands) {
        let conn = seat_conn(seat);
        let mut replies = Vec::new();
        session.handle(conn, ClientMessage::Command { seq, command }, &mut replies);
        for frame in &replies {
            if let Ok(ServerMessage::Rejected { code, .. }) =
                codec::decode_server(frame.frame.as_bytes(), usize::MAX)
            {
                return Err(format!("command {seq} ({command:?}): rejected, {code:?}"));
            }
        }
    }
    let reader = match seat {
        Seat::Spectator => viewer,
        seat => seat_conn(seat),
    };
    fetch_record(&mut session, reader)
}

/// The spectator's connection in the in-process session.
pub(crate) const VIEWER: ConnId = ConnId(1);

/// The connection that holds `seat` (the attacker's or the defender's) in the in-process session.
pub(crate) fn seat_conn(seat: Seat) -> ConnId {
    ConnId(if seat == Seat::Attacker { 2 } else { 3 })
}

/// The battle `setup` with `seed` hosted by the session in-process (`ruleset` compiled from its
/// packs): a spectator joined on [`VIEWER`] (its record every message), both seats taken by
/// their tokens on their connections ([`seat_conn`]) when `seated` (the game then starts).
pub(crate) fn hosted<'r>(
    ruleset: &'r BattleRuleset,
    setup: BattleSetup,
    seed: u64,
    seated: bool,
) -> Result<Session<'r>, String> {
    let config = config(setup, seed);
    let seats = ["attacker", "defender"].map(|token| SeatKind::Player {
        token: token.to_owned(),
    });
    let mut session =
        Session::new(ruleset, config, seats).map_err(|e| format!("the setup: {e:?}"))?;
    let mut out = Vec::new();
    let join = ClientMessage::Join {
        game: GAME,
        seat: Seat::Spectator,
        token: None,
    };
    session.handle(VIEWER, join, &mut out);
    if seated {
        for (seat, token) in [(Seat::Attacker, "attacker"), (Seat::Defender, "defender")] {
            let join = ClientMessage::Join {
                game: GAME,
                seat,
                token: Some(token.to_owned()),
            };
            session.handle(seat_conn(seat), join, &mut out);
        }
    }
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn a_scenario_becomes_a_spectators_record() {
        let scenario = repo().join("packs/synthetic/scenarios/duel.toml");
        let text = record_of(&scenario, &repo().join("packs"), 1, Seat::Spectator).unwrap();
        let record = h3_view::record::parse(&text).unwrap();
        assert_eq!(record.header.seat, Seat::Spectator);
        let snapshot = record.first_snapshot().unwrap();
        assert!(snapshot.spectator);
        // A side's record (E17 M4 T1b): what it sees.
        let text = record_of(&scenario, &repo().join("packs"), 1, Seat::Defender).unwrap();
        let record = h3_view::record::parse(&text).unwrap();
        assert_eq!(record.header.seat, Seat::Defender);
        assert!(!record.first_snapshot().unwrap().spectator);
        let text = record_of(&scenario, &repo().join("packs"), 1, Seat::Attacker).unwrap();
        assert_eq!(
            h3_view::record::parse(&text).unwrap().header.seat,
            Seat::Attacker
        );
        assert!(!snapshot.stacks.is_empty());
        // The duel asks for random obstacles: each placed one has its type.
        let typed = snapshot
            .removable_obstacles
            .iter()
            .all(|o| o.kind.is_some());
        assert!(typed);
    }

    #[test]
    fn war_machines_become_their_artifacts() {
        // E17 M3 T3b: the Ballista, the Ammo Cart and the First Aid Tent as artifacts 4, 5, 6 in
        // the war machines' slots 13, 14, 15.
        let scenario = repo().join("clients/situations/war-machines.toml");
        let sides = sides(&scenario, &repo().join("packs")).unwrap();
        let all = vec![(13, 4), (14, 5), (15, 6)];
        assert_eq!(sides.machines, [all.clone(), all]);
        let grass = repo().join("clients/situations/grass.toml");
        let sides = super::sides(&grass, &repo().join("packs")).unwrap();
        assert_eq!(sides.machines, [Vec::new(), Vec::new()]);
    }

    #[test]
    fn a_spell_book_becomes_its_artifact_and_spells() {
        // E17 M3 T7a-2a: Orrin's book of Slow (54), the Spell Book artifact 0; Crag Hack without.
        let packs = repo().join("packs");
        let cast = repo().join("clients/situations/seq-cast.toml");
        let sides = sides(&cast, &packs).unwrap();
        let books = sides.heroes.map(|h| h.and_then(|h| h.book));
        let slow = Book {
            artifact: 0,
            spells: vec!["Slow".to_owned()],
            skills: Vec::new(),
        };
        assert_eq!(books, [Some(slow), None]);
        // The original's hero starts with 10 spell points per knowledge: other mana is refused.
        let dir = std::env::temp_dir().join(format!("h3-view-book-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let wrong = dir.join("wrong.toml");
        let text = fs::read_to_string(&cast).unwrap();
        fs::write(&wrong, text.replace("mana = 10\nmax", "mana = 9\nmax")).unwrap();
        assert!(super::sides(&wrong, &packs).unwrap_err().contains("mana 9"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn battle_effects_become_leadership_and_luck() {
        // E17 M3 T7b-2b-2a: Leadership (6) and Luck (9) at the level of the morale and luck asked.
        use h3_content::Affects;
        let packs = repo().join("packs");
        let (ruleset, _) =
            battle(&repo().join("clients/situations/seq-walk.toml"), &packs, 1).unwrap();
        let luck = |amount, affects| BattleEffect::Luck { amount, affects };
        let morale = BattleEffect::Morale {
            amount: 2,
            affects: Affects::Own,
        };
        assert_eq!(
            effect_skills(&ruleset, &[luck(3, Affects::Own), morale]),
            Ok(vec![(6, 2), (9, 3)])
        );
        assert_eq!(effect_skills(&ruleset, &[]), Ok(Vec::new()));
        // No skill level gives these alone, or one skill twice.
        let capped = BattleEffect::LuckAtMost {
            value: 0,
            affects: Affects::Enemy,
        };
        for bad in [
            vec![luck(4, Affects::Own)],
            vec![luck(1, Affects::Enemy)],
            vec![luck(1, Affects::Own), luck(2, Affects::Own)],
            vec![capped],
        ] {
            assert!(effect_skills(&ruleset, &bad).is_err(), "{bad:?}");
        }
        // A scenario's hero effects reach its hero.
        let dir = std::env::temp_dir().join(format!("h3-view-effects-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let lucky = dir.join("lucky.toml");
        let text = fs::read_to_string(repo().join("clients/situations/seq-walk.toml")).unwrap();
        let text = text.replacen(
            "knowledge = 1\n",
            "knowledge = 1\neffects = [{ kind = \"luck\", amount = 3 }, { kind = \"morale\", amount = 1 }]\n",
            1,
        );
        fs::write(&lucky, text).unwrap();
        let heroes = sides(&lucky, &packs).unwrap().heroes;
        let skills = heroes.map(|h| h.map(|h| h.skills));
        assert_eq!(skills, [Some(vec![(6, 1), (9, 3)]), Some(Vec::new())]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn masteries_become_school_skills() {
        // E17 M3 T7a-2b: the original's mastery is the best of the spell's school skills.
        let packs = repo().join("packs");
        let (ruleset, _) =
            battle(&repo().join("clients/situations/seq-cast.toml"), &packs, 1).unwrap();
        let spell = |key: &str| (*ruleset).spell_by_key(key).unwrap().index();
        let (slow, arrow, ripple) = (
            spell("base:slow"),
            spell("base:magic_arrow"),
            spell("base:death_ripple"),
        );
        let fire = spell("base:fireball");
        // Expert Earth Magic (17) for Slow; Death Ripple, an earth spell, follows it.
        assert_eq!(
            school_skills(&ruleset, &[(slow, 3), (ripple, 3)], &[slow, ripple]),
            Ok(vec![(17, 3)])
        );
        // Magic Arrow (all four schools, earth first in the pack) alone raises Earth Magic to
        // Basic; with Slow at Expert Earth it already is Expert.
        assert_eq!(
            school_skills(&ruleset, &[(arrow, 1)], &[arrow]),
            Ok(vec![(17, 1)])
        );
        assert_eq!(
            school_skills(&ruleset, &[(arrow, 3), (slow, 3)], &[arrow, slow]),
            Ok(vec![(17, 3)])
        );
        // None named, none given; masteries no school skills give are refused: Death Ripple
        // unnamed under Expert Earth, Magic Arrow below Slow's school.
        assert_eq!(school_skills(&ruleset, &[], &[slow, fire]), Ok(Vec::new()));
        for (masteries, book, name) in [
            (vec![(slow, 3)], vec![slow, ripple], "Death Ripple"),
            (vec![(slow, 3), (arrow, 1)], vec![arrow], "Magic Arrow"),
        ] {
            let error = school_skills(&ruleset, &masteries, &book).unwrap_err();
            assert!(error.contains(name), "{error}");
        }
    }

    #[test]
    fn a_bad_scenario_is_an_error() {
        let packs = repo().join("packs");
        assert!(record_of(Path::new("/no/such.toml"), &packs, 1, Seat::Spectator).is_err());
        let dir = std::env::temp_dir().join(format!("h3-view-scenario-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.toml");
        fs::write(
            &bad,
            "[scenario]\nid = \"x\"\npacks = [\"../x\"]\nprofile = \"a:b\"\n",
        )
        .unwrap();
        assert!(
            record_of(&bad, &packs, 1, Seat::Spectator)
                .unwrap_err()
                .contains("invalid pack id")
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
