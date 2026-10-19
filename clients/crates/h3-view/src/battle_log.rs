//! The battle log's lines (E17 M3 T4b-2a, `docs/epics/E17-design.md` §9.4): the texts the
//! executable writes into the log for a strike, a wait, a defend, morale and luck, composed from the
//! string catalogue's keys as its code composes them from its text rows, and the log's entries a
//! line becomes (`battle_log_add` 0x472E90).
//!
//! A stack is named by its creature's key in the catalogue (`creature.base:pikeman`) and a number,
//! which picks the name's form: the original's singular for exactly 1, else its plural (the
//! creature table's two names, 0x440100). Which event writes which line, and when none (a hidden
//! battle, a wait in the tactics phase, a war machine's defend), is the caller's: the event
//! mapping comes with the battle panel (M3 T6).

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::text::{Args, Catalogue};

/// A stack as a line names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack<'a> {
    /// Its creature's key in the catalogue (`creature.base:pikeman`).
    pub creature: &'a str,
    /// Its number, which picks the name's form.
    pub count: u32,
}

/// A stack a strike hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit<'a> {
    /// Its creature's key in the catalogue.
    pub creature: &'a str,
    /// A war machine: its death is the "destroyed" row, not the perish row.
    pub war_machine: bool,
    /// The damage it took.
    pub damage: u64,
    /// The creatures it lost.
    pub kills: u32,
}

/// How a strike's hits add up to its one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spread {
    /// A strike or shot at one target, a breath's stack behind it as the second hit
    /// (`melee_strike` 0x441B9B, the shot 0x43FDB4): the line is the target's; a second hit of
    /// another creature is added in and the line names the word for creatures; one of the same
    /// creature is left out (the original's: such a breath reports only its target).
    Target,
    /// Every stack around: a multi-headed striker's strike (0x440480), an area shot (0x43FC4F,
    /// 0x44003F): all hits added up, the line named by their creature when they share one (the
    /// last hit's), else by the word for creatures.
    Around,
}

/// The line of a strike (`ReportDamageDone` 0x469A90): the damage row by the attacker's number,
/// then, when it killed any, the perish row by the kills (the destroyed row for a war machine).
#[must_use]
pub fn strike(
    catalogue: &Catalogue,
    attacker: Stack<'_>,
    spread: Spread,
    hits: &[Hit<'_>],
) -> String {
    let (damage, kills, target) = sum(spread, hits);
    let damage = i64::try_from(damage).unwrap_or(i64::MAX);
    let mut line = catalogue.text(
        "battle.log.damage",
        &Args::new()
            .text("attacker", &name(catalogue, attacker))
            .number("damage", damage)
            .count(i64::from(attacker.count)),
    );
    if kills > 0 {
        let fallen = |creature| Stack {
            creature,
            count: kills,
        };
        let perish = match target {
            Some(hit) if hit.war_machine => catalogue.text(
                "battle.log.destroyed",
                &Args::new().text("creature", &name(catalogue, fallen(hit.creature))),
            ),
            Some(hit) => perish(catalogue, &name(catalogue, fallen(hit.creature)), kills),
            None => perish(catalogue, &name(catalogue, fallen("word.creature")), kills),
        };
        line.push_str(&perish);
    }
    line
}

/// A stack waits (0x479988).
#[must_use]
pub fn wait(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    by_count(catalogue, "battle.log.wait", stack, Args::new())
}

/// A stack defends and gains `defense` (0x4797E3; the executable's gain is
/// max(1, its defense × 20 / 100), 0x479758).
#[must_use]
pub fn defend(catalogue: &Catalogue, stack: Stack<'_>, defense: i64) -> String {
    by_count(
        catalogue,
        "battle.log.defend",
        stack,
        Args::new().number("defense", defense),
    )
}

/// Good morale (`good_morale_roll` 0x464AD5).
#[must_use]
pub fn morale_good(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    named(catalogue, "battle.log.morale-good", stack)
}

/// Bad morale (`bad_morale_check` 0x464CDF).
#[must_use]
pub fn morale_bad(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    named(catalogue, "battle.log.morale-bad", stack)
}

/// Good luck on a strike or a shot (0x44187A, 0x43F998).
#[must_use]
pub fn luck_good(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    named(catalogue, "battle.log.luck-good", stack)
}

/// The log's entries a line becomes (`battle_log_add` 0x472E90): a line without a line break is
/// one entry; with one, its first break becomes a space when the line then takes fewer than two
/// lines (`lines`: how many the log's font takes at the log's width, `text_line_count` 0x4B5820),
/// else the line is two entries, split there (any later break stays in the second).
#[must_use]
pub fn entries(line: &str, lines: impl Fn(&str) -> i64) -> Vec<String> {
    let Some((first, rest)) = line.split_once('\n') else {
        return vec![line.into()];
    };
    let joined = format!("{first} {rest}");
    if lines(&joined) < 2 {
        vec![joined]
    } else {
        vec![first.into(), rest.into()]
    }
}

/// A stack's name by its number.
fn name(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    catalogue.text(stack.creature, &Args::new().count(i64::from(stack.count)))
}

/// A key with the stack's name as `creature` and its number as `count`.
fn by_count(catalogue: &Catalogue, key: &str, stack: Stack<'_>, args: Args) -> String {
    let args = args
        .text("creature", &name(catalogue, stack))
        .count(i64::from(stack.count));
    catalogue.text(key, &args)
}

/// A key with the stack's name (by its number) as `creature`, and no form of its own.
fn named(catalogue: &Catalogue, key: &str, stack: Stack<'_>) -> String {
    catalogue.text(key, &Args::new().text("creature", &name(catalogue, stack)))
}

/// The perish row with the fallen's name.
fn perish(catalogue: &Catalogue, creature: &str, kills: u32) -> String {
    catalogue.text(
        "battle.log.perish",
        &Args::new()
            .text("creature", creature)
            .count(i64::from(kills)),
    )
}

/// A strike's damage, kills and the hit that names its line (none: the word for creatures).
fn sum<'a>(spread: Spread, hits: &[Hit<'a>]) -> (u64, u32, Option<Hit<'a>>) {
    let counted = match (spread, hits) {
        (Spread::Target, [first, second, ..]) if first.creature == second.creature => {
            core::slice::from_ref(first)
        }
        (Spread::Target, _) => hits.get(..2).unwrap_or(hits),
        (Spread::Around, _) => hits,
    };
    let damage = counted
        .iter()
        .fold(0_u64, |s, h| s.saturating_add(h.damage));
    let kills = counted.iter().fold(0_u32, |s, h| s.saturating_add(h.kills));
    let last = counted.last().copied();
    let alike = counted
        .iter()
        .all(|h| Some(h.creature) == last.map(|l| l.creature));
    (damage, kills, last.filter(|_| alike))
}

#[cfg(test)]
mod tests;
