//! The battle as a seat on a server sees it (E17 M5 T6b1; design §11.2): no engine runs where the
//! window plays a seat on a server, so the mouse, the panel and the book of the turn the server
//! asks the seat to play take the battle from the seat's record and the timeline drawn from it.

use alloc::collections::BTreeMap;

use h3_protocol::wire::{Event, Observation, Side};

use crate::anim::Timeline;

/// The battle as the seat sees it at `t` of its `timeline`: the record's opening `snapshot` (the
/// seat's own view) with the `events` since applied as far as the mouse, the panel and the book
/// read them. The turn's active stack, the round, its wait phase, the heroes' casts this round and
/// the tactics phase as the panel's turns fold them ([`crate::panel::turns`]); the stacks the
/// battle created, as the timeline shows them by `t`; each stack's hex by its moves and teleports
/// and where it was created, in order; its count and its life as the timeline shows them at `t`;
/// the heroes' mana by their casts (`cost` of a side's spell), and the mana channelled and drained;
/// the gate open or closed. *Provisional* (the M5 review list): a shooter's shots, the stacks'
/// spell effects, the walls' hexes and the wall parts stay the opening's.
#[must_use]
pub fn seen(
    snapshot: &Observation,
    events: &[Event],
    timeline: &Timeline,
    t: u32,
    cost: impl Fn(Side, u16) -> u16,
) -> Observation {
    let mut now = crate::panel::turns(snapshot, events)
        .pop()
        .unwrap_or_else(|| snapshot.clone());
    let mut hexes = BTreeMap::new();
    let mut mana = [0i32; 2];
    let side = |side: Side| usize::from(side == Side::Defender);
    for event in events {
        match *event {
            Event::Moved { stack, to, .. } | Event::Teleported { stack, to, .. } => {
                hexes.insert(stack, to);
            }
            Event::StackCreated { stack, hex, .. } | Event::RoseFromCorpse { stack, hex, .. } => {
                hexes.insert(stack, hex);
            }
            Event::SpellCast {
                side: by, spell, ..
            } => {
                mana[side(by)] -= i32::from(cost(by, spell));
            }
            Event::ManaChanneled { side: to, mana: m } => mana[side(to)] += i32::from(m),
            Event::ManaDrained {
                side: from,
                mana: m,
                ..
            } => mana[side(from)] -= i32::from(m),
            Event::GateOpened {} | Event::GateClosed {} => {
                if let Some(siege) = now.siege.as_mut() {
                    siege.gate_open = matches!(event, Event::GateOpened {});
                }
            }
            _ => {}
        }
    }
    for (view, change) in now.sides.iter_mut().zip(mana) {
        if let Some(hero) = view.hero.as_mut() {
            let left = (i32::from(hero.mana) + change).clamp(0, i32::from(u16::MAX));
            hero.mana = u16::try_from(left).unwrap_or(0);
        }
    }
    for (id, stack) in timeline.created_at(t) {
        match now.stacks.iter_mut().find(|s| s.id == id) {
            Some(s) => *s = stack,
            None => now.stacks.push(stack),
        }
    }
    let (counts, poses) = (timeline.counts(t), timeline.at(t));
    for stack in &mut now.stacks {
        if let Some(&hex) = hexes.get(&stack.id) {
            stack.hex = hex;
        }
        if let Some(&count) = counts.get(&stack.id) {
            stack.count = count;
        }
        if let Some(pose) = poses.get(&stack.id) {
            stack.alive = !pose.dead();
        }
    }
    now
}
