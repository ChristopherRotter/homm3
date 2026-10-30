//! The scripted presses' test (E17 M5 T2a, `docs/epics/E17-design.md` §9.11): over every turn of
//! every scenario the bot fights to its end, a press at every hex (in each slot of a hex the
//! acting stack may strike), on the siege's rectangles and on the panel's Wait and Defend (in the
//! tactics phase its Next Creature and Start Combat, E17 M5 T4c2b) gives exactly the engine's
//! legal walks, strikes, shots, wall shots, heals, Wait, Defend and `EndTactics`
//! (`h3_view::mouse::click`, `button`), each stack facing its side's way as the engine has it;
//! but the original's one gap, a two-hex stack's step back by one hex where two are open too
//! (CLT-Q25, copied: an option later). An acting two-hex stack turned the other way (its poses
//! mirrored then, `pointer::facing`) takes its other hexes: its presses give other commands, each
//! one legal still (E17 M5 T8a, R3-6).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use h3_battle::{BattleState, LegalCommands, NoopSink, Viewer};
use h3_content_battle::BattleRuleset;
use h3_protocol::map;
use h3_protocol::session::Bot as _;
use h3_protocol::wire::{Command, Observation, Side};
use h3_render::Bugs;
use h3_view::layout::{BattleLayout, parse_battle};
use h3_view::mouse::{self, DEFEND, Facing, Field, NEXT_CREATURE, START_COMBAT, WAIT};

/// The client workspace.
fn clients() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The scenarios: the situations' and the sequences' (`clients/situations`, but its lists), then
/// the packs' synthetic ones.
fn scenarios() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in [
        clients().join("situations"),
        clients().join("../packs/synthetic/scenarios"),
    ] {
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .filter(|p| {
                let stem = p.file_stem().unwrap_or_default();
                !["counts", "sequences", "set"].iter().any(|s| stem == *s)
            })
            .collect();
        files.sort();
        out.extend(files);
    }
    out
}

/// The commands the presses give in `field` ([`points`], the rectangles, the panels' buttons).
fn pressed(
    layout: &BattleLayout,
    field: &Field<'_>,
    hexes: &[([i32; 2], Vec<[i32; 2]>)],
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut take = |c: Option<Command>| {
        if let Some(c) = c {
            out.insert(format!("{c:?}"));
        }
    };
    for (centre, slots) in hexes {
        let c = mouse::click(field, layout, *centre);
        if matches!(c, Some(Command::Attack { .. })) {
            for p in slots {
                take(mouse::click(field, layout, *p));
            }
        }
        take(c);
    }
    for rect in [layout.mouse.keep, layout.mouse.tower] {
        take(mouse::click(field, layout, crate::pointer::middle(rect)));
    }
    for id in [WAIT, DEFEND, NEXT_CREATURE, START_COMBAT] {
        take(mouse::button(field.legal, id));
    }
    out
}

/// The legal walks no press gives in the original (CLT-Q25): a two-hex stack's one hex back, its
/// own position being that position's front hex, while it may walk two back too (a press there
/// walks to the position behind, 0x476C5F; on its own hex it shows its info).
fn unpressed(observation: &Observation, commands: &[Command]) -> BTreeSet<String> {
    let acting = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id))
        .filter(|s| s.two_hex);
    let Some(acting) = acting else {
        return BTreeSet::new();
    };
    let back = |hex: u8| {
        if acting.side == Side::Attacker {
            hex.checked_sub(1)
        } else {
            hex.checked_add(1)
        }
    };
    let walks = |to: Option<u8>| to.is_some_and(|to| commands.contains(&Command::Move { to }));
    let one = back(acting.hex);
    if walks(one) && walks(one.and_then(back)) {
        one.map(|to| format!("{:?}", Command::Move { to }))
            .into_iter()
            .collect()
    } else {
        BTreeSet::new()
    }
}

/// What a scenario's turns check: the turns, the commands compared, the turns that differ; the
/// turns of a two-hex stack checked turned too, and those whose presses then changed.
#[derive(Default)]
struct Checked {
    turns: usize,
    commands: usize,
    gaps: usize,
    failures: Vec<String>,
    turned: usize,
    changed: usize,
}

/// The stacks facing left when `observation`'s acting stack, two hexes wide, turned the other way
/// (none for another stack).
fn turned(observation: &Observation) -> Option<Facing> {
    let acting = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id))
        .filter(|s| s.two_hex)?;
    let left = (observation.stacks.iter())
        .filter(|s| (s.side == Side::Defender) != (s.id == acting.id))
        .map(|s| s.id)
        .collect();
    Some(Facing::Left(left))
}

/// What the presses are checked with: the layout, the asset tables, the hexes' points, the
/// rulesets compiled so far (by the scenario's packs and profile).
struct With {
    layout: BattleLayout,
    tables: h3_view::tables::Tables,
    hexes: Vec<([i32; 2], Vec<[i32; 2]>)>,
    rulesets: BTreeMap<(Vec<String>, String), BattleRuleset>,
}

/// Fights `path`'s battle to its end with the bot for both sides, checking the presses before each
/// command.
fn check(path: &Path, with: &mut With, out: &mut Checked) {
    let name = path.file_stem().unwrap().to_string_lossy().into_owned();
    let def = crate::scenario::definition(path).unwrap();
    let ruleset = with
        .rulesets
        .entry((def.packs.clone(), def.profile.clone()))
        .or_insert_with(|| crate::scenario::compiled(&def, &clients().join("../packs")).unwrap());
    let manifest = map::manifest(ruleset);
    let setup = crate::scenario::set_up(&def, ruleset, 1).unwrap();
    let mut state = BattleState::new(ruleset, &setup, 1).unwrap();
    let (mut bot, mut legal_now) = (crate::bot::Greedy, LegalCommands::new());
    let mut i = 0;
    while let Some(side) = state.side_to_act() {
        let observation = map::observe(&state, ruleset, Viewer::Spectator);
        let commands = map::legal_commands(&state, ruleset, side);
        let legal = crate::pointer::legal(&manifest, &with.tables, &observation, (&commands, &[]));
        let field = Field {
            observation: &observation,
            legal: &legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        };
        let got = pressed(&with.layout, &field, &with.hexes);
        let gap = unpressed(&observation, &commands);
        let want: BTreeSet<String> = commands
            .iter()
            .filter(|c| {
                matches!(
                    c,
                    Command::Move { .. }
                        | Command::Attack { .. }
                        | Command::Shoot { .. }
                        | Command::ShootWall { .. }
                        | Command::Heal { .. }
                        | Command::Wait {}
                        | Command::Defend {}
                        | Command::EndTactics {}
                )
            })
            .map(|c| format!("{c:?}"))
            .filter(|c| !gap.contains(c))
            .collect();
        if let Some(facing) = turned(&observation) {
            let field = Field {
                facing: &facing,
                ..field
            };
            let presses = pressed(&with.layout, &field, &with.hexes);
            let all: BTreeSet<String> = commands.iter().map(|c| format!("{c:?}")).collect();
            let illegal: Vec<_> = presses.difference(&all).collect();
            if !illegal.is_empty() {
                out.failures
                    .push(format!("{name}, turn {i}, turned: illegal {illegal:?}"));
            }
            out.turned += 1;
            out.changed += usize::from(presses != got);
        }
        out.turns += 1;
        out.commands += want.len();
        out.gaps += gap.len();
        if got != want {
            let missing: Vec<_> = want.difference(&got).collect();
            let extra: Vec<_> = got.difference(&want).collect();
            out.failures.push(format!(
                "{name}, turn {i} (stack {:?}): missing {missing:?}, extra {extra:?}",
                observation.active
            ));
        }
        assert!(i < crate::scenario::FOUGHT_LIMIT, "{name}: no end");
        h3_battle::legal_commands(&state, ruleset, side, &mut legal_now);
        let choice = bot.command(&state, ruleset, side, &legal_now);
        h3_battle::apply(&mut state, ruleset, side, choice, &mut NoopSink).unwrap();
        i += 1;
    }
}

#[test]
fn presses_give_exactly_the_engine_s_legal_commands() {
    let layout =
        parse_battle(&fs::read_to_string(clients().join("layouts/battle.toml")).unwrap()).unwrap();
    let mut with = With {
        tables: crate::assets::load(&clients().join("assets"), crate::assets::ALL).unwrap(),
        hexes: crate::pointer::press_points(&layout),
        layout,
        rulesets: BTreeMap::new(),
    };
    let mut checked = Checked::default();
    for path in scenarios() {
        check(&path, &mut with, &mut checked);
    }
    let Checked {
        turns,
        commands,
        gaps,
        failures,
        turned,
        changed,
    } = checked;
    assert!(
        turns > 1_000 && commands > 10_000 && gaps > 0,
        "{turns} turns, {commands} commands, {gaps} gaps"
    );
    assert!(
        turned > 100 && changed > 100,
        "{turned} turns turned, {changed} changed"
    );
    assert!(
        failures.is_empty(),
        "{} of {turns} turns:\n{}",
        failures.len(),
        failures
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
