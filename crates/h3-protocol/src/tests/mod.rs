//! Tests of the wire types, the codec and the mapping (E16 M1 T1), the connection and the session
//! core (M1 T2), the lobby (M2 T1), the decoder under mutated frames (M2 T5d), the ladders' queues
//! (M4 T2), the server's card (M6 T2).

mod card;
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

/// A stand-in for the server's key (E16 M6 T1): its signature folds the message into 64 bytes, so
/// a test sees which bytes were signed.
#[derive(Debug)]
struct Fold;

impl crate::connection::Signer for Fold {
    fn key(&self) -> [u8; 32] {
        [0x5a; 32]
    }

    fn sign(&self, message: &[u8]) -> [u8; 64] {
        let mut out = [0u8; 64];
        for (i, b) in message.iter().enumerate() {
            out[i % 64] = out[i % 64].rotate_left(3) ^ b;
        }
        out
    }
}

/// The synthetic `full` scenario: seven stacks a side, both led by casting heroes.
fn full() -> (BattleRuleset, BattleSetup) {
    h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap()
}

/// A battle of the `full` scenario.
fn battle() -> (BattleRuleset, BattleState) {
    let (ruleset, setup) = full();
    let state = BattleState::new(&ruleset, &setup, 7).unwrap();
    (ruleset, state)
}
