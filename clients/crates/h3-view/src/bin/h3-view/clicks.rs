//! The scripted presses' test (E17 M5 T2a, `docs/epics/E17-design.md` §9.11): over every turn of
//! every scenario the bot fights to its end, a press at every hex (in each slot of a hex the
//! acting stack may strike), on the siege's rectangles and on the panel's Wait and Defend gives
//! exactly the engine's legal walks, strikes, shots, wall shots, heals, Wait and Defend
//! (`h3_view::mouse::click`, `button`), each stack facing its side's way as the engine has it;
//! but the original's one gap, a two-hex stack's step back by one hex where two are open too
//! (CLT-Q25, copied: an option later).

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
use h3_view::mouse::{self, DEFEND, Facing, Field, Spot, WAIT};

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

/// A point of `hex` in each of its twelve slots, outside the siege's rectangles where the slot
/// has one there.
fn slot_points(layout: &BattleLayout, hex: u8) -> Vec<[i32; 2]> {
    let [left, top] = layout.hex_box(hex / 17, hex % 17);
    let [w, h] = layout.hex_step;
    let within =
        |[l, t, r, b]: [i32; 4], [x, y]: [i32; 2]| (l..=r).contains(&x) && (t..=b).contains(&y);
    let free = |p: [i32; 2]| !within(layout.mouse.keep, p) && !within(layout.mouse.tower, p);
    let inside: Vec<[i32; 2]> = (top - h..top + 2 * h)
        .flat_map(|y| (left..left + w).map(move |x| [x, y]))
        .filter(|&p| mouse::spot(layout, p, false) == Spot::Hex(hex))
        .collect();
    (0..12)
        .filter_map(|s| {
            let in_slot = |p: &&[i32; 2]| mouse::slot(layout, **p, hex) == s;
            let mut all = inside.iter().filter(in_slot);
            let first = all.clone().next().copied();
            all.find(|p| free(**p)).copied().or(first)
        })
        .collect()
}

/// Each hex's centre (the harness's point) and its slots' points.
fn points(layout: &BattleLayout) -> Vec<([i32; 2], Vec<[i32; 2]>)> {
    (0..187u8)
        .map(|hex| {
            (
                crate::pointer::hex_point(layout, hex),
                slot_points(layout, hex),
            )
        })
        .collect()
}

/// A rectangle's centre.
fn middle([l, t, r, b]: [i32; 4]) -> [i32; 2] {
    [(l + r) / 2, (t + b) / 2]
}

/// The commands the presses give in `field` ([`points`], the rectangles, the two buttons).
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
        take(mouse::click(field, layout, middle(rect)));
    }
    for id in [WAIT, DEFEND] {
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

/// What a scenario's turns check: the turns, the commands compared, the turns that differ.
#[derive(Default)]
struct Checked {
    turns: usize,
    commands: usize,
    gaps: usize,
    failures: Vec<String>,
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
        let legal = crate::pointer::legal(&manifest, &with.tables, &observation, &commands);
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
                )
            })
            .map(|c| format!("{c:?}"))
            .filter(|c| !gap.contains(c))
            .collect();
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
        hexes: points(&layout),
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
    } = checked;
    assert!(
        turns > 1_000 && commands > 10_000 && gaps > 0,
        "{turns} turns, {commands} commands, {gaps} gaps"
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
