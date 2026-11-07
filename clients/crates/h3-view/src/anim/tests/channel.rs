//! E17 M6 T5d2b: Ice Bolt's explosion (`cast_spell` 0x5A0AAB to 0x5A0AFB, design §12.24), its sound
//! as its animation strikes and its wait after the line; the Familiars' channel at a hero's cast's
//! tail (0x5A222E to 0x5A2750) with its line, its two sounds and its two animations at the
//! battlefield's corners.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, Target};
use h3_render::Bugs;

use super::{
    GOBLIN, PIKEMAN, SWORDSMAN, applied, at, casts, cues, layout, looks, snapshot, spell_damage,
    spell_magic, tables, timeline,
};
use crate::anim::{
    CHANNEL_DRAIN, CHANNEL_FILL, Caster, Casts, Command, HYPNOTIZE, ICE_BOLT, Logged, MagicLook,
    Shell, Sound, Speed, SpellLook, Stage, Timeline,
};
use crate::sound::Lengths;

/// Ice Bolt's animation (`C08spW5`, 4 frames here, centred, translucent in the tables).
const ICE_BOLT_LOOK: u8 = 46;
/// Hypnotize's handle here.
const HYPNOTIZE_SPELL: u16 = 59;

/// The casts' spells with Ice Bolt (its projectile `c08spw0`–`4`) and both heroes casting, the
/// Goblins channelling.
fn channel_casts() -> Casts {
    let mut casts = casts();
    let shell = |k: usize, angle| Shell {
        def: alloc::format!("c08spw{k}.def"),
        size: [100, 60],
        frames: 8,
        angle,
    };
    let bolt = SpellLook {
        original: ICE_BOLT,
        sign: -1,
        animation: Some(ICE_BOLT_LOOK),
        projectile: [0, -16, -34, -56, -83]
            .into_iter()
            .enumerate()
            .map(|(k, angle)| shell(k, angle))
            .collect(),
        ..SpellLook::default()
    };
    casts.spells.insert(ICE_BOLT, bolt);
    casts.casters[1] = Some(Caster {
        point: [751, 48],
        before: 5,
        frames: 8,
    });
    casts.channellers.insert(GOBLIN);
    casts
}

/// The casts' animations with Ice Bolt's and the channel's two (3 frames each here, one size).
fn channel_magic() -> BTreeMap<u8, MagicLook> {
    let entry = |def: &str, frames, size| MagicLook {
        def: def.into(),
        frames,
        size,
        place: 1,
        translucent: false,
    };
    let mut all = spell_magic();
    all.insert(ICE_BOLT_LOOK, entry("C08spW5.def", 4, [146, 154]));
    all.insert(CHANNEL_FILL, entry("sp07_A.def", 3, [110, 130]));
    all.insert(CHANNEL_DRAIN, entry("sp07_B.def", 3, [110, 130]));
    all
}

/// The timeline of `snapshot` and `commands` with the channel's stage and the samples' `lengths`
/// (none: no sound).
fn play(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    lengths: Option<&Lengths>,
    casts: &Casts,
) -> Timeline {
    let layout = layout();
    let magic = channel_magic();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts,
        bugs: Bugs::Original,
        siege: None,
        lengths,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &looks(), stage, Speed::default()).unwrap()
}

fn lengths(sounds: &[(Sound, u32)]) -> Lengths {
    Lengths {
        named: sounds.iter().copied().collect(),
        ..Lengths::default()
    }
}

fn cast_by(side: Side, spell: u16, target: u8) -> Event {
    Event::SpellCast {
        side,
        spell,
        target: Target::Stack { stack: target },
        second: Target::None {},
    }
}

/// The Pikemen (0, active) and a Goblin stack (3) on the left; the Swordsmen (1) and two Goblin
/// stacks (2, 4) on the right.
fn armies() -> Observation {
    let mut two = at(4, Side::Defender, GOBLIN, 118, false);
    two.count = 5;
    snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 49, false),
        at(2, Side::Defender, GOBLIN, 151, false),
        at(3, Side::Attacker, GOBLIN, 137, false),
        two,
    ])
}

/// Animation `number`'s keys at a hex in `t`: (shown, frame, where).
fn spots(t: &Timeline, number: u8) -> Vec<(u32, Option<u32>, [i32; 2])> {
    t.spots
        .iter()
        .filter(|k| k.animation == number)
        .map(|k| (k.shown, k.frame, k.at))
        .collect()
}

/// When the hero of `side` shows its casting frame `frame`.
fn hero_shown(t: &Timeline, side: u8, frame: u32) -> Option<u32> {
    t.hero_keys
        .iter()
        .find(|k| k.side == side && k.group == 4 && k.frame == frame)
        .map(|k| k.shown)
}

#[test]
fn ice_bolt_s_explosion_sounds_as_its_animation_strikes_and_is_waited_for() {
    // Orrin's Ice Bolt at the Swordsmen: the projectile, then `IceRayEx.wav` as 46 starts over
    // them with their hit, the line after it; with the sample's length the hero's last frames
    // wait for its end (0x5A0AFB).
    let events = vec![cast_by(Side::Attacker, ICE_BOLT, 1), spell_damage(1, 12, 1)];
    let casts = channel_casts();
    let t = play(&armies(), &[(Some(0), events.clone())], None, &casts);
    let explosion: Vec<_> = cues(&t)
        .into_iter()
        .filter(|c| c.2 == Sound::IceBolt)
        .collect();
    let [(cued, 1, Sound::IceBolt, false)] = explosion[..] else {
        panic!("{explosion:?}");
    };
    let bolt: Vec<_> = t
        .magics
        .iter()
        .filter(|k| k.animation == ICE_BOLT_LOOK)
        .map(|k| k.shown)
        .collect();
    // `strike_animation` shows its first frame at once (as Magic Arrow's 64).
    assert_eq!(bolt.first().copied(), Some(cued), "{bolt:?}");
    let line = Logged::SpellDamage {
        spell: ICE_BOLT,
        hit: (1, 12, 1),
    };
    let written = t.log.iter().find(|(_, l)| *l == line).map(|(at, _)| *at);
    assert!(written.is_some_and(|w| w > cued), "{:?}", t.log);
    let rest = hero_shown(&t, 0, 5).unwrap();
    assert!(rest < cued + 3000);
    // With sound the rest of the hero's frames wait for the sample's end.
    let long = lengths(&[(Sound::IceBolt, 3000)]);
    let heard = play(&armies(), &[(Some(0), events)], Some(&long), &casts);
    assert_eq!(hero_shown(&heard, 0, 5), Some(cued + 3000));
    // The line before the wait (0x5A0AEB, then 0x5A0AFB).
    let heard_line = heard
        .log
        .iter()
        .find(|(_, l)| *l == line)
        .map(|(at, _)| *at);
    assert_eq!(heard_line, written);
    // Magic Arrow plays no explosion.
    let arrow = vec![cast_by(Side::Attacker, 15, 1), spell_damage(1, 12, 1)];
    let t = play(&armies(), &[(Some(0), arrow)], Some(&long), &casts);
    assert!(cues(&t).iter().all(|c| c.2 != Sound::IceBolt));
}

#[test]
fn the_familiars_channel_after_a_hero_s_cast() {
    // Orrin casts Slow at the Swordsmen; the defender's Goblins channel 2 mana: the line (both
    // their stacks' 45, the left Goblins not), `MagChDrn.wav` with 76 at the caster's corner (hex
    // 0), a frame each 100 ms and a redraw erasing it, then `MagChFil.wav` with 75 at the
    // defender's (hex 16).
    let channel = Event::ManaChanneled {
        side: Side::Defender,
        mana: 2,
    };
    let events = vec![cast_by(Side::Attacker, 54, 1), applied(1, 54), channel];
    let casts = channel_casts();
    let t = play(&armies(), &[(Some(0), events.clone())], None, &casts);
    let line = Logged::Channeled {
        stack: 2,
        count: 45,
        mana: 2,
    };
    let at = t.log.iter().find(|(_, l)| *l == line).map(|(at, _)| *at);
    let at = at.unwrap_or_else(|| panic!("{:?}", t.log));
    // After the hero stands (0x5A216B), the Pikemen whose turn it is outlined through both
    // animations (the recorded `channel`).
    assert!(t.hero_keys.iter().all(|k| k.drawn <= at));
    assert_eq!(t.outlined(at - 1), None);
    assert_eq!(t.outlined(at), Some(0));
    assert_eq!(t.outlined(at + 500), Some(0));
    let heard: Vec<_> = cues(&t)
        .into_iter()
        .filter(|c| matches!(c.2, Sound::ChannelDrain | Sound::ChannelFill))
        .collect();
    assert_eq!(
        heard,
        [
            (at, 0, Sound::ChannelDrain, false),
            (at + 400, 0, Sound::ChannelFill, false)
        ]
    );
    let drain = spots(&t, CHANNEL_DRAIN);
    let fill = spots(&t, CHANNEL_FILL);
    let shown = |s: &[(u32, Option<u32>, [i32; 2])]| -> Vec<(u32, Option<u32>)> {
        s.iter().map(|k| (k.0, k.1)).collect()
    };
    assert_eq!(
        shown(&drain),
        [
            (at + 100, Some(0)),
            (at + 200, Some(1)),
            (at + 300, Some(2)),
            (at + 400, None)
        ]
    );
    assert_eq!(
        shown(&fill),
        [
            (at + 500, Some(0)),
            (at + 600, Some(1)),
            (at + 700, Some(2)),
            (at + 800, None)
        ]
    );
    let left = drain[0].2;
    let right = fill[0].2;
    assert!(left[0] < right[0], "{left:?} {right:?}");
    // The defender's hero cast: the attacker's Goblins channel, the corners swapped.
    let theirs = vec![
        cast_by(Side::Defender, 54, 0),
        applied(0, 54),
        Event::ManaChanneled {
            side: Side::Attacker,
            mana: 1,
        },
    ];
    let t = play(&armies(), &[(Some(0), theirs)], None, &casts);
    let line = Logged::Channeled {
        stack: 3,
        count: 40,
        mana: 1,
    };
    assert!(t.log.iter().any(|(_, l)| *l == line), "{:?}", t.log);
    assert_eq!(spots(&t, CHANNEL_DRAIN)[0].2, right);
    assert_eq!(spots(&t, CHANNEL_FILL)[0].2, left);
    // With the samples' lengths each is waited for in turn: the drain's from its start, the
    // fill's from its own; the next turn starts after both.
    let long = lengths(&[(Sound::ChannelDrain, 1500), (Sound::ChannelFill, 2000)]);
    let next = vec![Event::TurnStarted { stack: 1 }];
    let commands = [(Some(0), events), (None, next)];
    let quiet = play(&armies(), &commands, None, &casts);
    let heard = play(&armies(), &commands, Some(&long), &casts);
    let turn = |t: &Timeline| t.turns.last().map(|t| t.0);
    assert_eq!(turn(&quiet).map(|q| q - at), Some(800));
    assert_eq!(turn(&heard).map(|h| h - at), Some(400 + 2000));
    // A drain outlasting the fill's end holds the turn by itself.
    let long = lengths(&[(Sound::ChannelDrain, 3000), (Sound::ChannelFill, 500)]);
    let heard = play(&armies(), &commands, Some(&long), &casts);
    assert_eq!(turn(&heard).map(|h| h - at), Some(3000));
}

#[test]
fn a_hypnotized_familiar_channels_for_its_controller() {
    // The left Goblins (3) hypnotized: the defender's channel counts them with its own (0x5A21D5
    // to 0x5A21EC: the controller), the attacker's has none to name.
    let mut casts = channel_casts();
    casts.spells.insert(
        HYPNOTIZE_SPELL,
        SpellLook {
            original: HYPNOTIZE,
            ..SpellLook::default()
        },
    );
    let hypnotized = Event::EffectApplied {
        target: 3,
        spell: HYPNOTIZE_SPELL,
        rounds: Some(2),
    };
    let channel = |side, mana| Event::ManaChanneled { side, mana };
    let commands = [
        (
            Some(0),
            vec![
                hypnotized,
                cast_by(Side::Attacker, 54, 1),
                applied(1, 54),
                channel(Side::Defender, 2),
            ],
        ),
        (
            Some(20_000),
            vec![
                cast_by(Side::Defender, 54, 0),
                applied(0, 54),
                channel(Side::Attacker, 1),
            ],
        ),
    ];
    let t = play(&armies(), &commands, None, &casts);
    let lines: Vec<_> = t
        .log
        .iter()
        .filter_map(|(_, l)| match l {
            Logged::Channeled { stack, count, mana } => Some((*stack, *count, *mana)),
            _ => None,
        })
        .collect();
    assert_eq!(lines, [(2, 85, 2)]);
}

#[test]
fn a_channel_without_a_channeller_in_sight_writes_no_line() {
    // No stack the gaining side controls channels (the record's creatures not known as such):
    // the animations and sounds only.
    let mut casts = channel_casts();
    casts.channellers.clear();
    let events = vec![
        cast_by(Side::Attacker, 54, 1),
        applied(1, 54),
        Event::ManaChanneled {
            side: Side::Defender,
            mana: 2,
        },
    ];
    let t = play(&armies(), &[(Some(0), events)], None, &casts);
    assert!(
        t.log
            .iter()
            .all(|(_, l)| !matches!(l, Logged::Channeled { .. }))
    );
    assert_eq!(spots(&t, CHANNEL_DRAIN).len(), 4);
    assert_eq!(spots(&t, CHANNEL_FILL).len(), 4);
    let heard: Vec<_> = cues(&t).into_iter().map(|c| c.2).collect();
    assert!(heard.contains(&Sound::ChannelDrain) && heard.contains(&Sound::ChannelFill));
}
