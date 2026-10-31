//! Stacks created mid-battle and the dissolves (E17 M5 T5a; `docs/epics/E17-design.md` §7.25).
//! `create_battle_stack` 0x47A100 sets a record up for a new stack and shows it in a dissolve
//! (Summon Elemental, a Pit Lord's Demons); Clone slides its clone from the original's place
//! instead (0x5A6FDE). A stack that leaves no corpse dissolves away (`battle_dissolve_removed`
//! 0x466E00: Sacrifice's victim, a created stack's death, a clone gone with its original or its
//! rounds), and Teleport dissolves the battlefield (0x5A1B4D). Each dissolve saves a rectangle of
//! the screen (`screen_save_rect` 0x602CC0), redraws the battle unseen and blends the new picture
//! into the saved one (`screen_dissolve` 0x602DC0, `h3_render::Screen::dissolve_rect`).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{DamageRoll, Event, Side, Stack};
use h3_render::Bugs;

use super::{CountKey, GONE, Key, Live, LookKey, Member, Player, Pose, Sound, TintKey};
use crate::battle_log::Logged;
use crate::scene::Moment;

/// A dissolve's steps (`screen_dissolve` 0x602DC0): step k shows k eighths of the new picture,
/// then the new picture shows whole.
pub const DISSOLVE_STEPS: u32 = 8;
/// A dissolve's step when stacks leave without a corpse (`battle_dissolve_removed` 0x4670EF:
/// trunc(150 F) ms; E17 M3 T9d).
pub(super) const REMOVAL_STEP: u32 = 150;
/// A created stack's dissolve step (`create_battle_stack` 0x47A2A8: 75 ms, whatever the speed).
pub const CREATION_STEP: u32 = 75;
/// Teleport's dissolve step (`screen_dissolve`'s default for −1, 0x602E74: 33 ms, whatever the
/// speed; 0x5A1BEA).
pub const TELEPORT_STEP: u32 = 33;
/// The clone's slide from its original's place: 16 redraws (0x5A6FE0), each of trunc(50 F)
/// (0x5A7017).
const SLIDES: i32 = 16;
const SLIDE_TIME: u32 = 50;
/// Teleport (the executable's spell number).
pub const TELEPORT: u16 = 63;
/// The rectangle Teleport dissolves (0x5A1B6C: the battlefield, 800 × 556 from the corner), left,
/// top, right and bottom, exclusive.
pub const BATTLEFIELD: [i32; 4] = [0, 0, 800, 556];

/// Where a dissolve blends (E17 M5 T5a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DissolveArea {
    /// The stacks' rectangle: `battle_refresh_creatures` 0x495BF0 measures each in either picture,
    /// its sprite's content box and its count box (measured whatever hides the box, 0x43E576),
    /// each cut to the battlefield, into the dirty rectangle (`crate::compose`).
    Stacks(Vec<u8>),
    /// The battlefield ([`BATTLEFIELD`]: Teleport).
    Battlefield,
}

/// A dissolve (E17 M5 T5a, design §7.25): the screen at `start` saved, the battle redrawn without
/// showing it, and the new picture blended in over the area in [`DISSOLVE_STEPS`] steps of `step`
/// ms outside the deadline, each held until its start plus the step (`wait_until` 0x4F82F0 at
/// 0x603014); then the new picture shows whole. What changes with it is drawn at `start` and
/// shows from `start + 1` ([`Player::key_now`]): the moment at `start` is the saved picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DissolveKey {
    /// When its first step shows (weight 0: the saved picture).
    pub start: u32,
    /// Its step in ms.
    pub step: u32,
    /// Its area.
    pub area: DissolveArea,
}

impl DissolveKey {
    /// When the new picture shows whole.
    #[must_use]
    pub fn end(&self) -> u32 {
        self.start
            .saturating_add(DISSOLVE_STEPS.saturating_mul(self.step))
    }

    /// The step showing at `t`, if the dissolve blends then (from `start + 1` to its end; at
    /// `start` its step 0 is the saved picture itself).
    #[must_use]
    pub fn step_at(&self, t: u32) -> Option<u8> {
        (self.start < t && t < self.end())
            .then(|| u8::try_from((t - self.start) / self.step.max(1)).unwrap_or(0))
    }
}

/// A dissolve showing (E17 M5 T5a): the battle the saved rectangle shows, the step and the area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dissolve {
    /// The battle as the screen showed it when the dissolve started.
    pub before: Box<Moment>,
    /// The step (0 to 7).
    pub step: u8,
    /// Where it blends.
    pub area: DissolveArea,
}

/// A stack the battle created (E17 M5 T5a): from `drawn` its record holds it (the harness's
/// timeline lists it from then), from `shown` the screen draws it as `stack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedKey {
    /// When its record is set up.
    pub drawn: u32,
    /// When it shows.
    pub shown: u32,
    /// Its record and first pose.
    pub member: Member,
    /// Its numbers as the viewer draws them ([`created_stack`]).
    pub stack: Stack,
}

/// A created creature's body (the ruleset's; E17 M5 T5a): what the battle's drawing needs of a
/// stack the record's first snapshot lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Body {
    /// It takes two hexes.
    pub two_hex: bool,
    /// It flies.
    pub flies: bool,
    /// It shoots.
    pub shooter: bool,
}

/// A created stack's numbers as the viewer draws it (E17 M5 T5a): `id`, `side`, `creature`,
/// `count` on `hex` with `body`; created in the battle (army slot 255), a clone or raised as said.
/// A record holds only the first snapshot, so the numbers no drawing reads are left at 0
/// (*provisional*: the attack, defense, damage and the like).
#[must_use]
pub fn created_stack(
    (id, side): (u8, Side),
    (creature, count, hex): (u16, u32, u8),
    body: Body,
    (clone, raised): (bool, bool),
) -> Stack {
    Stack {
        id,
        side,
        controller: side,
        slot: u8::MAX,
        creature,
        hex,
        alive: count > 0,
        count,
        base_count: count,
        top_hp: 0,
        max_hp: 0,
        attack: 0,
        defense: 0,
        damage_min: 0,
        damage_max: 0,
        speed: 0,
        morale: 0,
        luck: 0,
        shots: 0,
        retaliations: 0,
        retaliations_left: 0,
        regeneration: 0,
        acted: false,
        waited: false,
        retaliated: false,
        defending: false,
        had_morale: false,
        started: false,
        two_hex: body.two_hex,
        flies: body.flies,
        shooter: body.shooter,
        war_machine: false,
        no_distance_penalty: false,
        shoots_when_adjacent: false,
        native: false,
        created: true,
        clone,
        raised,
        clone_rounds: 0,
        clone_link: None,
        temporary: 0,
        corpse: 0,
        bound_by: Vec::new(),
        poison: 0,
        start_hp: 0,
        casts_left: 0,
        cast_candidates: 0,
        melee_attack_bonus: 0,
        ranged_attack_bonus: 0,
        damage_roll: DamageRoll::Random {},
        speed_percent: 100,
        effects: Vec::new(),
    }
}

impl Player<'_> {
    /// `stack` takes `pose` now, in memory from now and on the screen from the next millisecond (a
    /// dissolve's change: the moment at its start is the saved picture, [`DissolveKey`]).
    pub(super) fn key_now(&mut self, stack: u8, pose: Pose) {
        let now = self.clock.now;
        self.keys.push(Key {
            stack,
            drawn: now,
            shown: now + 1,
            pose,
        });
    }

    /// A dissolve over `area` from now, of `step` ms a step: its key, and the clock past its
    /// steps, outside the deadline (§7.25).
    pub(super) fn dissolve(&mut self, area: DissolveArea, step: u32) {
        let start = self.clock.now;
        self.dissolves.push(DissolveKey { start, step, area });
        self.clock.wait(DISSOLVE_STEPS * step);
    }

    /// The stacks of `dead` (each with the pose it dies in) that leave no corpse, and the clones
    /// gone with their originals since the last death ([`Player::vanished`]), leave the
    /// battlefield in a dissolve before the corpses lie (`stack_death` 0x444120 marks
    /// Sacrifice's victim and a summoned stack, +0x84 0x10000000 or 0x400000, 0x4442C5;
    /// `battle_dissolve_removed` 0x466E00 from `strike_animation` 0x469316, `mass_damage_handler`
    /// 0x5A51D3, `mass_spell_animation` 0x5A6C59; §7.25): off the field from its start (their
    /// poses [`GONE`]), the battle without them blended in over their rectangle in eight steps of
    /// trunc(150 F). Returns the stacks gone; the others lie as corpses.
    pub(super) fn remove_gone(&mut self, dead: &[(u8, Pose)]) -> BTreeSet<u8> {
        let mut gone: BTreeMap<u8, Pose> = dead
            .iter()
            .filter(|(s, _)| self.vanishing.remove(s) || self.summoned.contains(s))
            .copied()
            .collect();
        for s in core::mem::take(&mut self.vanished) {
            if let Some(live) = self.stacks.get(&s) {
                let pose = self.standing(live, live.hex);
                gone.entry(s).or_insert(pose);
            }
        }
        let gone: Vec<(u8, Pose)> = gone.into_iter().collect();
        self.remove(&gone, self.speed.scale(REMOVAL_STEP))
    }

    /// `gone` (each in its last pose) leave the battlefield in a dissolve of `step` ms a step;
    /// returns them.
    fn remove(&mut self, gone: &[(u8, Pose)], step: u32) -> BTreeSet<u8> {
        if gone.is_empty() {
            return BTreeSet::new();
        }
        for &(s, pose) in gone {
            self.stacks.remove(&s);
            self.corpses.remove(&s);
            self.lying.remove(&s);
            self.key_now(
                s,
                Pose {
                    layer: GONE,
                    ..pose
                },
            );
        }
        let ids: Vec<u8> = gone.iter().map(|(s, _)| *s).collect();
        self.dissolve(DissolveArea::Stacks(ids.clone()), step);
        ids.into_iter().collect()
    }

    /// The clones that vanished at a round's start (`round_start` 0x47614D, after every stack's
    /// round start: their rounds ran out): one dissolve over them all (§7.25).
    pub(super) fn vanish_at_round(&mut self, events: &[Event]) {
        let gone: Vec<(u8, Pose)> = events
            .iter()
            .filter_map(|e| match *e {
                Event::Vanished { stack, .. } => self.stacks.get(&stack).map(|l| (stack, l)),
                _ => None,
            })
            .map(|(s, live)| (s, self.standing(live, live.hex)))
            .collect();
        self.remove(&gone, self.speed.scale(REMOVAL_STEP));
    }

    /// The created stack `id` (`creature`, `count` on `hex`, made as `kind`; a clone copies its
    /// original's body): its record, the first that is free (a killed created stack's, which the
    /// engine's id reuses on the same side, else the side's next: 0x47A12B), set up now and drawn
    /// from the next millisecond, its first pose standing `offset` from its hex (E17 M5 T5a).
    /// Returns its numbers as the timeline follows it (none: a creature without a look, or no
    /// record left).
    fn set_up(
        &mut self,
        (id, side): (u8, Side),
        (creature, count, hex): (u16, u32, u8),
        kind: Made,
        offset: [i32; 2],
    ) -> Option<Live> {
        let s = usize::from(side == Side::Defender);
        let original = match kind {
            Made::Clone(o) => Some(o),
            Made::Summoned | Made::Raised => None,
        };
        let body = match original.and_then(|o| self.stacks.get(&o)) {
            Some(o) => Body {
                two_hex: o.two_hex,
                flies: o.flies,
                shooter: false,
            },
            None => self
                .stage
                .casts
                .bodies
                .get(&creature)
                .copied()
                .unwrap_or_default(),
        };
        let look = self.looks.get(&creature)?.clone();
        let live = Live {
            side,
            hex,
            count,
            flies: body.flies,
            two_hex: body.two_hex,
            defending: false,
            war_machine: false,
            right: side == Side::Attacker,
            boxless: false,
            look,
            spells: BTreeSet::new(),
            defense: 0,
            frightens: self.stage.casts.frighteners.contains(&creature),
            shielded: self.stage.casts.shielded.contains(&creature),
            turret: None,
        };
        // A reused id keeps its record on its own side alone; past the side's last record (a
        // hostile record's 255th creation) the stack is not drawn (E17 M5 T8b2, R6-3).
        let record = match self.records.get(&id) {
            Some(&(owner, record)) if owner == side => record,
            _ => {
                let record = self.next_record[s];
                self.next_record[s] = record.checked_add(1)?;
                record
            }
        };
        self.records.insert(id, (side, record));
        // Its cues play its creature's sounds, not those of a stack that held the id before.
        self.voices.insert(id, creature);
        let pose = Pose {
            offset,
            ..self.standing(&live, hex)
        };
        let now = self.clock.now;
        let stack = created_stack(
            (id, side),
            (creature, count, hex),
            Body {
                shooter: self
                    .stage
                    .casts
                    .bodies
                    .get(&creature)
                    .is_some_and(|b| b.shooter),
                ..body
            },
            (original.is_some(), kind == Made::Raised),
        );
        self.created.push(CreatedKey {
            drawn: now,
            shown: now + 1,
            member: Member {
                stack: id,
                side,
                record,
                start: pose,
            },
            stack,
        });
        // A reused id's box, hue and look start afresh.
        self.counts.push(CountKey {
            stack: id,
            shown: now + 1,
            count,
        });
        self.tints.push(TintKey {
            stack: id,
            shown: now + 1,
            hue: None,
        });
        self.look_keys.push(LookKey {
            stack: id,
            shown: now + 1,
            look: None,
        });
        self.log.push((
            now,
            Logged::Created {
                stack: id,
                creature,
                side,
            },
        ));
        Some(live)
    }

    /// The stacks a hero's cast creates among its `effects` (E17 M5 T5a, §7.25): Summon
    /// Elemental's line (GENRLTXT 676, `summon_elemental` 0x5A72A1, written whether a stack is
    /// made or not: CLT-Q33, in the client's profile only when one is) and its stack shown in a dissolve of 75 ms steps; a clone set up
    /// without one (0x5A6E4F, its animation argument 0) and slid from its original's place in 16
    /// redraws of trunc(50 F), its offset the anchors' difference times (16 − i) / 16, truncated
    /// (0x5A6FDE to 0x5A7040), then standing in a redraw at once (0x5A7067); an original turned
    /// into its own clone (quirk BAT-QUIRK-7) the 16 redraws with no offset; a clone without a
    /// place nothing but the message ([`Player::notices`]). The spell's sound starts before the
    /// handler (0x5A0331). Returns whether the cast created (or failed to create) a stack.
    pub(super) fn creating(&mut self, side: u8, spell: u16, effects: &[Event]) -> bool {
        for e in effects {
            match *e {
                Event::StackCreated {
                    stack,
                    creature,
                    count,
                    hex,
                    original: None,
                } => {
                    self.cue(stack, Sound::Spell(spell), false);
                    let line = Logged::Summoned {
                        side,
                        creature,
                        count,
                    };
                    self.log.push((self.clock.now, line));
                    let owner = if side == 0 {
                        Side::Attacker
                    } else {
                        Side::Defender
                    };
                    let live = self.set_up(
                        (stack, owner),
                        (creature, count, hex),
                        Made::Summoned,
                        [0, 0],
                    );
                    if let Some(live) = live {
                        let pose = self.standing(&live, hex);
                        self.key_now(stack, pose);
                        self.stacks.insert(stack, live);
                        self.summoned.insert(stack);
                        self.dissolve(DissolveArea::Stacks(vec![stack]), CREATION_STEP);
                    }
                    return true;
                }
                Event::StackCreated {
                    stack,
                    creature,
                    count,
                    hex,
                    original: Some(original),
                } => {
                    self.cue(original, Sound::Spell(spell), false);
                    let Some(from) = self.stacks.get(&original).map(|o| o.hex) else {
                        return true;
                    };
                    let side = self
                        .stacks
                        .get(&original)
                        .map_or(Side::Attacker, |o| o.side);
                    let [fx, fy] = self.stage.anchor(from);
                    let [tx, ty] = self.stage.anchor(hex);
                    let d = [fx - tx, fy - ty];
                    let made = Made::Clone(original);
                    let Some(live) = self.set_up((stack, side), (creature, count, hex), made, d)
                    else {
                        return true;
                    };
                    self.stacks.insert(stack, live.clone());
                    self.summoned.insert(stack);
                    self.slide(stack, &live, d);
                    return true;
                }
                Event::TurnedSummoned { stack, .. } => {
                    self.cue(stack, Sound::Spell(spell), false);
                    self.summoned.insert(stack);
                    let time = self.speed.scale(SLIDE_TIME);
                    for _ in 0..SLIDES {
                        self.frame(&[], time);
                    }
                    self.redraw();
                    return true;
                }
                Event::NotCreated {
                    side: owner,
                    creature,
                    count,
                    original,
                } => {
                    match original {
                        // The clone's message (0x5A6E22).
                        Some(o) => {
                            self.cue(o, Sound::Spell(spell), false);
                            self.notices.push((self.clock.now, Notice::NoPlace));
                        }
                        // Summon Elemental's line, written before the stack is made
                        // (0x5A72A1): the client writes none when nothing was made (CLT-Q33,
                        // answered (b) at E17 M5 T8c), the original's for the comparisons.
                        None if self.stage.bugs == Bugs::Original => {
                            let line = Logged::Summoned {
                                side: u8::from(owner == Side::Defender),
                                creature,
                                count,
                            };
                            self.log.push((self.clock.now, line));
                        }
                        None => {}
                    }
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    /// The clone `stack` slides from its original's place, `d` away (0x5A6FDE to 0x5A7067).
    fn slide(&mut self, stack: u8, live: &Live, d: [i32; 2]) {
        let time = self.speed.scale(SLIDE_TIME);
        let base = self.standing(live, live.hex);
        for i in 0..SLIDES {
            // 0x5A6FF6: the difference times 16 less i times it, over 16, truncated (cdq, and,
            // sar).
            let offset = d.map(|v| v * (SLIDES - i) / SLIDES);
            self.frame(&[(stack, Pose { offset, ..base })], time);
        }
        self.pending.push((stack, base));
        self.redraw();
    }

    /// Teleport's move among a cast's `effects` (0x5A1B4D to 0x5A1BFA; E17 M5 T5a, §7.25): a
    /// redraw shown at once, the battlefield saved, the stack's Bind links and its own Bind ended
    /// (0x5A1B88, 0x5A1B91: the engine's `EffectEnded`s, their hues with the new picture; E17 M5
    /// T8b2, R6-2), the stack moved and the battle redrawn unseen, then a dissolve over the
    /// battlefield of 33 ms steps. Returns whether it moved one.
    pub(super) fn teleporting(&mut self, spell: u16, effects: &[Event]) -> bool {
        let Some((stack, to)) = effects.iter().find_map(|e| match *e {
            Event::Teleported { stack, to, .. } => Some((stack, to)),
            _ => None,
        }) else {
            return false;
        };
        self.cue(stack, Sound::Spell(spell), false);
        self.redraw();
        self.effects(effects);
        let Some(live) = self.stacks.get_mut(&stack) else {
            return true;
        };
        live.hex = to;
        let live = live.clone();
        let pose = self.standing(&live, to);
        self.key_now(stack, pose);
        self.show_tints(self.clock.now + 1);
        self.dissolve(DissolveArea::Battlefield, TELEPORT_STEP);
        true
    }

    /// A Pit Lord raises Demons `(new, creature, count)` from `corpse` (`creature_cast_action`
    /// 0x448627, `pit_lord_raise_demons` 0x5A7390; E17 M5 T5a, §7.25): its casting frames toward
    /// the corpse, `Resurect.wav` (spell 38's sound, started as a file), the corpse gone and the
    /// Demons set up (0x47A100 with its animation: a dissolve of 75 ms steps over their
    /// rectangle), a redraw at once (0x5A749C), the raise's line (GENRLTXT 117 or 118); the sound
    /// waited for (0x5A754E, E17 M5 T5b2), then the caster's own (0x4486E2) and it turns back.
    pub(super) fn demons(
        &mut self,
        stack: u8,
        corpse: u8,
        (new, creature, count, hex): (u8, u16, u32, u8),
    ) {
        let Some(dead) = self.corpses.get(&corpse).cloned() else {
            return;
        };
        let turned = self.creature_casting(stack, dead.position(dead.hex));
        let cue = self
            .spell_handle(super::raise::RESURRECTION)
            .map(|spell| self.cue(corpse, Sound::SpellFile(spell), false));
        // The corpse goes (0x5A73DD), in the pose it lay in (memory keeps it), with the redraw
        // the dissolve blends in. (The original blends from its back buffer, which the casting
        // frames' clipped redraws left as the background outside the caster's rectangle: the
        // corpse and the obstacles there pop away at the first step. A slip, not copied: the
        // user's, 2026-10-29; design §7.25.)
        self.corpses.remove(&corpse);
        let lying = self.lying.remove(&corpse);
        if let Some(lying) = lying.filter(|_| corpse != new) {
            self.key_now(
                corpse,
                Pose {
                    layer: GONE,
                    ..lying
                },
            );
        }
        let at = (creature, count, hex);
        if let Some(live) = self.set_up((new, dead.side), at, Made::Raised, [0, 0]) {
            let pose = self.standing(&live, live.hex);
            self.key_now(new, pose);
            self.stacks.insert(new, live);
            self.summoned.remove(&new);
            self.dissolve(DissolveArea::Stacks(vec![new]), CREATION_STEP);
        }
        self.redraw();
        let line = Logged::Raised { stack: new, count };
        self.log.push((self.clock.now, line));
        if let Some(cue) = cue {
            self.wait_for(cue);
        }
        self.creature_cast_over(stack, turned);
    }
}

/// What the battle tells its player in a message box (E17 M5 T5a): the viewers open it at its
/// time, the battle waiting behind it as for any message box (design §7.19).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// Clone found no place (0x5A6E22: GENRLTXT 189 in an OK box).
    NoPlace,
}

/// How a stack was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Made {
    /// Summoned (Summon Elemental).
    Summoned,
    /// A clone of the stack.
    Clone(u8),
    /// Raised from a corpse (a Pit Lord's Demons).
    Raised,
}
