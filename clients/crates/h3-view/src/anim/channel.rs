//! The Familiars' channel after a hero's cast (E17 M6 T5d2b; `docs/epics/E17-design.md` §12.24):
//! `cast_spell`'s tail, after the hero stands and the stack whose turn it is is outlined again
//! (0x5A216B), gives the other hero a share of the cost (0x5A2170 to 0x5A2228, BAT-MAG-055) and,
//! unless the battle is hidden, writes its line, then plays the drain's sound and animation at the
//! caster's corner and the fill's at the other, and waits for both sounds (0x5A2740, 0x5A2750).

use h3_protocol::wire::Side;

use super::{Player, Sound, TICK};
use crate::battle_log::Logged;

/// The channel's drain at the caster's corner (`play_magic_animation_at_hex` from 0x5A2709:
/// `sp07_B`).
pub const CHANNEL_DRAIN: u8 = 76;
/// Its fill at the gaining side's corner (0x5A2736: `sp07_A`).
pub const CHANNEL_FILL: u8 = 75;
/// The attacker's corner (hex 0) and the defender's (16), by the side whose turn it is, the
/// caster's (0x5A26F1 to 0x5A2702, 0x5A271B to 0x5A272B).
const CORNERS: (u8, u8) = (0, 16);

impl Player<'_> {
    /// The hero of `side` gained `mana` from its side's channellers after the other hero's cast
    /// (`cast_spell` 0x5A222E to 0x5A2750; E17 M6 T5d2b, design §12.24): the stack whose turn it
    /// is outlined from the hero's standing redraw on (`outline_turn_stack` 0x5A216B ran before
    /// it: the recorded `channel`, outlined through both animations), the log's line, GENRLTXT
    /// 112 with the Familiars' singular name when the side controls exactly one of them, else 113
    /// with their plural, and the mana (the creatures of every channelling stack the side
    /// controls counted, a hypnotized one by its controller as 0x5A21D5 to 0x5A21EC; named by the
    /// first's creature, *provisional*: the original's are creature 43's); `MagChDrn.wav` (category 2)
    /// and animation [`CHANNEL_DRAIN`] at the caster's corner, a frame a redraw at trunc(100 F)
    /// and a last redraw erasing it; `MagChFil.wav` and [`CHANNEL_FILL`] at the other corner the
    /// same; then each sound waited for in turn.
    pub(super) fn channeled(&mut self, side: Side, mana: u16) {
        self.outlined.push((self.clock.now, self.acting));
        let mut channellers = self
            .stacks
            .iter()
            .filter(|&(&id, l)| l.channels && l.count > 0 && self.controller(id) == side);
        let first = channellers.next().map(|(&stack, l)| (stack, l.count));
        let line = first.map(|(stack, count)| Logged::Channeled {
            stack,
            count: count + channellers.map(|(_, l)| l.count).sum::<u32>(),
            mana,
        });
        if let Some(line) = line {
            self.log.push((self.clock.now, line));
        }
        let (attacker, defender) = CORNERS;
        let (drained, filled) = if side == Side::Attacker {
            (defender, attacker)
        } else {
            (attacker, defender)
        };
        let time = self.speed.scale(TICK);
        let drain = self.cue(0, Sound::ChannelDrain, false);
        self.at_hex(CHANNEL_DRAIN, drained, time, false);
        let fill = self.cue(0, Sound::ChannelFill, false);
        self.at_hex(CHANNEL_FILL, filled, time, false);
        self.wait_for(drain);
        self.wait_for(fill);
    }
}
