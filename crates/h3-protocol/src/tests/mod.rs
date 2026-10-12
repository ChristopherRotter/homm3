//! Tests of the wire types, the codec and the mapping (E16 M1 T1), the connection and the session
//! core (M1 T2), the lobby (M2 T1), the decoder under mutated frames (M2 T5d), the ladders' queues
//! (M4 T2).

mod codec;
mod connection;
mod ladder;
mod lobby;
mod mapping;
mod mutation;
mod session;
mod wire;

use h3_battle::{BattleSetup, BattleState};
use h3_content_battle::BattleRuleset;

/// The synthetic `full` scenario: seven stacks a side, both led by casting heroes.
fn full() -> (BattleRuleset, BattleSetup) {
    h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap()
}

/// A battle of the `full` scenario.
fn battle() -> (BattleRuleset, BattleState) {
    let (ruleset, setup) = full();
    let state = BattleState::new(&ruleset, &setup, 7).unwrap();
    (ruleset, state)
}
