//! The battle log's lines (E17 M3 T4b-2a, `docs/epics/E17-design.md` §9.4): the texts the
//! executable writes into the log for a strike, a wait, a defend, morale and luck, composed from the
//! string catalogue's keys as its code composes them from its text rows, the log's entries a
//! line becomes (`battle_log_add` 0x472E90) and the text its item shows (0x472C30, M3 T4b-2b-2).
//!
//! A stack is named by its creature's key in the catalogue (`creature.base:pikeman`) and a number,
//! which picks the name's form: the original's singular for exactly 1, else its plural (the
//! creature table's two names, 0x440100).
//!
//! The log over a battle (E17 M3 T6b, design §9.8): the timeline says what the battle writes and
//! when ([`Logged`], `crate::anim`), [`word`] words it, and [`texts`] keeps the log as the
//! executable does ([`Log`]: the combat dialog's entries, its new entries' count, the first shown,
//! the time it showed them) to give the text its item shows from each time on.

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::text::{Args, Catalogue};

/// What the battle writes into its log, or does to it, at a time (E17 M3 T6b, design §9.8): the
/// timeline's record of it (`crate::anim::Timeline::log`), each stack by its id and its number
/// then.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Logged {
    /// An action starts (`action_dispatcher` 0x478D80's head): entries new for 3 000 ms or more
    /// are new no longer (0x472BF0).
    Action,
    /// The stack's turn starts (`battle_controls_state` 0x4782D0): for a human player's, entries
    /// new for 3 000 ms or more are new no longer (0x47884C), and the mouse is posted again
    /// (0x4ECC00), which sets the status line to its hint where it rests.
    Turn(u8),
    /// A strike's or a shot's line (`battle_log_damage` 0x469A90): the striker and its number,
    /// how the hits add up, each hit's stack, damage and kills.
    Strike {
        /// The striker.
        attacker: u8,
        /// Its number then.
        count: u32,
        /// How its hits add up.
        spread: Spread,
        /// Each hit: the stack, its damage and its kills.
        hits: Vec<(u8, u64, u32)>,
    },
    /// A stack waits (0x479988; none in the tactics phase).
    Wait {
        /// The stack.
        stack: u8,
        /// Its number.
        count: u32,
    },
    /// A stack defends (0x4797E3; none for a war machine).
    Defend {
        /// The stack.
        stack: u8,
        /// Its number.
        count: u32,
        /// Its defense skill then (the gain's base, 0x479758; E17 M3 T7b-2a: lowered by
        /// Disrupting Ray).
        defense: i32,
    },
    /// The First Aid Tent heals a stack (0x478D0A).
    Heal {
        /// The tent.
        healer: u8,
        /// The patient.
        target: u8,
        /// The tent's number and the patient's.
        counts: [u32; 2],
        /// The hit points healed.
        hit_points: u32,
    },
    /// Good or bad morale (0x464AD5, 0x464CDF).
    Morale {
        /// The stack.
        stack: u8,
        /// Its number.
        count: u32,
        /// Good morale.
        good: bool,
    },
    /// Good luck on a strike or a shot (0x44187A, 0x43F998), before its animation.
    Luck {
        /// The striker.
        stack: u8,
        /// Its number.
        count: u32,
    },
    /// A stack frozen by Fear (`fear_check` 0x464EFF, E17 M3 T7b-2b-2b), before its animation.
    Fear {
        /// The frightened stack.
        stack: u8,
        /// Its number.
        count: u32,
        /// A frightening stack: its creature's name.
        by: u8,
        /// The frightening creatures' number on that side, which picks the name's form.
        number: u32,
    },
    /// A round starts outside the tactics phase (`round_start` 0x4761E9).
    Round,
    /// A hero's spell on a stack (`battle_log_cast` 0x5A8950 for the caster kind 0 with a target;
    /// E17 M3 T7b-1): written after the spell's effect, before its animation (0x5A109F).
    Cast {
        /// The caster's side (0 the attacker's).
        side: u8,
        /// The spell's handle.
        spell: u16,
        /// The stack it took effect on.
        target: u8,
        /// Its number, which picks its name.
        count: u32,
    },
    /// A spell resisted by the stack it was aimed at (`cast_spell` 0x5A1FB0 to 0x5A2035; E17 M3
    /// T7b-1): written after the resistance's sound, before its animation.
    Resisted {
        /// The stack.
        stack: u8,
        /// Its number, which picks its name and the verb.
        count: u32,
    },
    /// Disrupting Ray's line (0x5A1B0B to 0x5A1B43; E17 M3 T7b-2a): the defense it took from the
    /// stack, after its animation and effect; no cast line.
    Disrupted {
        /// The stack.
        stack: u8,
        /// The defense lowered (at most the stack's, 0x5A1ACB).
        lowered: i32,
    },
    /// Life Drain (`striker_effects` 0x440D1F; E17 M3 T7b-2b-3): before its animation.
    Drain {
        /// The drinkers.
        stack: u8,
        /// Their number before the creatures it raised.
        count: u32,
        /// The hit points drained.
        hit_points: u64,
        /// The creatures it raised (a row of their own after the line when any).
        raised: u32,
        /// The stack struck.
        target: u8,
        /// Its number before the strike's kills.
        target_count: u32,
    },
    /// Death Stare (`striker_effects` 0x440FD3; E17 M3 T7b-2b-3): before its animation.
    DeathStare {
        /// The Gorgons.
        stack: u8,
        /// Their number.
        count: u32,
        /// The stack stared at.
        target: u8,
        /// The creatures it killed.
        kills: u32,
    },
    /// The Thunderbird's bolt strikes a stack (`striker_effects` 0x441276; E17 M3 T7b-2b-3):
    /// before the bolt from the sky.
    Thunderbolt {
        /// The stack.
        stack: u8,
        /// Its number.
        count: u32,
    },
    /// Creatures raised in a stack (`raise_stack` 0x5A772F, GENRLTXT 117 and 118 by their
    /// number; E17 M3 T7c-3): before the raise's animation.
    Raised {
        /// The stack.
        stack: u8,
        /// The creatures raised, which pick its name.
        count: u32,
    },
    /// Poison takes hold of a stack after a strike (a creature's cast, `battle_log_cast`
    /// 0x5A8EFC; E17 M3 T7b-2b-3): before its animation.
    Poisoned {
        /// The stack.
        stack: u8,
        /// Its number.
        count: u32,
    },
    /// Acid Breath lowers a stack's defense (`cast_spell` 0x5A1F27; E17 M3 T7b-2b-3): after its
    /// animation.
    AcidBreath {
        /// The stack.
        stack: u8,
        /// Its number.
        count: u32,
        /// The defense lowered (at most 3, to 0 at least: 0x5A1E70).
        lowered: i32,
    },
    /// A spell's damage on one stack (`battle_log_damage` 0x469A90 named by the spell, its number
    /// 1: Magic Arrow 0x5A0A1F, Ice Bolt 0x5A0AEB; E17 M3 T7b-1), after its animation.
    SpellDamage {
        /// The spell's handle.
        spell: u16,
        /// The stack, its damage and its kills.
        hit: (u8, u64, u32),
    },
    /// A hero's spell without a target (`battle_log_cast` 0x5A8950 for the caster kind 0 without
    /// one, GENRLTXT 197; E17 M3 T7c-1): a mass timed spell after its effects, before its
    /// animation (0x5A10D0); Death Ripple and Destroy Undead before their damage (0x5A0C89,
    /// 0x5A0E86).
    CastAll {
        /// The caster's side (0 the attacker's).
        side: u8,
        /// The spell's handle.
        spell: u16,
    },
    /// Death Ripple's or Destroy Undead's damage (GENRLTXT 188, 187; E17 M3 T7c-1): one stack's
    /// damage when it struck one, else the caster's (0x5A0DC6 to 0x5A0E0A); Death Ripple's before
    /// its animation, Destroy Undead's after it.
    MassDamage {
        /// The spell's handle.
        spell: u16,
        /// The damage.
        damage: u64,
        /// Destroy Undead's row (the spell named), else Death Ripple's.
        undead: bool,
    },
    /// Armageddon's damage (GENRLTXT 89, `mass_damage_handler` 0x5A5244; E17 M3 T7c-2): after
    /// its animation and the deaths, the spell's value without the caster's bonuses (the
    /// original's, CLT-Q16).
    Armageddon {
        /// The damage.
        damage: u64,
    },
    /// An area spell's damage (`area_spell_handler` 0x5A4B83: `battle_log_damage` named by the
    /// spell, its number 1; E17 M3 T7c-1), before the reactions: one stack's damage and kills
    /// naming it, or with several the caster's damage and every kill, named by the word for
    /// creatures (0x5A4B1D).
    AreaDamage {
        /// The spell's handle.
        spell: u16,
        /// The damage.
        damage: u64,
        /// The creatures killed.
        kills: u32,
        /// The one stack struck (none: several).
        target: Option<u8>,
    },
    /// The status line written (0x472E90 with its second argument 0; E17 M3 T6c-2): the mouse's
    /// hint, forced over new entries or not (`crate::mouse::track`).
    Status {
        /// The text.
        text: String,
        /// Whether it replaces the item's text over new entries.
        forced: bool,
    },
    /// The mouse's stack unmarked as a cast starts (`battle_clear_highlight` 0x477E10 from
    /// `cast_spell` 0x5A0055; E17 M3 T7c-1: a creature's cast, whose action leaves it marked until
    /// its own frames are over): no line.
    Unmark,
    /// The log scrolled by its buttons or the arrow keys (0x472DF0, 0x472E40, 0x472DB0; E17 M3
    /// T6c-2: the window's), by entries (up: negative).
    Scroll(i64),
}

/// A stack as [`word`] names it: its creature's key in the catalogue and whether it is a war
/// machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// Its creature's key (`creature.base:pikeman`).
    pub creature: String,
    /// A war machine.
    pub war_machine: bool,
}

/// What else than a stack a line names (E17 M3 T7b-1): a spell by its handle, a side's hero
/// (0 the attacker's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Name {
    /// A spell.
    Spell(u16),
    /// A side's hero.
    Hero(u8),
}

/// The line `logged` writes (none for what writes none, or a stack `stack` or a spell or hero
/// `key` does not name; `key` gives a [`Name`]'s key in the catalogue).
#[must_use]
pub fn word(
    catalogue: &Catalogue,
    logged: &Logged,
    stack: &dyn Fn(u8) -> Option<Named>,
    key: &dyn Fn(Name) -> Option<String>,
) -> Option<String> {
    let named = |id: u8, count: u32| stack(id).map(|n| (n, count));
    match logged {
        Logged::Action
        | Logged::Turn(_)
        | Logged::Status { .. }
        | Logged::Scroll(_)
        | Logged::Unmark => None,
        Logged::Strike {
            attacker,
            count,
            spread,
            hits,
        } => {
            let (striker, count) = named(*attacker, *count)?;
            let struck: Vec<(Named, u64, u32)> = hits
                .iter()
                .map(|&(id, damage, kills)| stack(id).map(|n| (n, damage, kills)))
                .collect::<Option<_>>()?;
            let hits: Vec<Hit<'_>> = struck
                .iter()
                .map(|(n, damage, kills)| Hit {
                    creature: &n.creature,
                    war_machine: n.war_machine,
                    damage: *damage,
                    kills: *kills,
                })
                .collect();
            let attacker = Stack {
                creature: &striker.creature,
                count,
            };
            Some(strike(catalogue, attacker, *spread, &hits))
        }
        Logged::Wait { stack, count } => {
            let (n, count) = named(*stack, *count)?;
            Some(wait(catalogue, n.stack(count)))
        }
        Logged::Defend {
            stack,
            count,
            defense,
        } => {
            let (n, count) = named(*stack, *count)?;
            // 0x479758: max(1, its defense × 20 / 100).
            let gain = (i64::from(*defense) * 20 / 100).max(1);
            Some(defend(catalogue, n.stack(count), gain))
        }
        Logged::Disrupted { lowered, .. } => Some(catalogue.text(
            "battle.log.disrupted",
            &Args::new().number("defense", i64::from(*lowered)),
        )),
        Logged::Heal {
            healer,
            target,
            counts,
            hit_points,
        } => {
            let (tent, _) = named(*healer, counts[0])?;
            let (patient, _) = named(*target, counts[1])?;
            Some(heal(
                catalogue,
                tent.stack(counts[0]),
                patient.stack(counts[1]),
                *hit_points,
            ))
        }
        Logged::Morale { stack, count, good } => {
            let (n, count) = named(*stack, *count)?;
            let compose = if *good { morale_good } else { morale_bad };
            Some(compose(catalogue, n.stack(count)))
        }
        Logged::Luck { stack, count } => {
            let (n, count) = named(*stack, *count)?;
            Some(luck_good(catalogue, n.stack(count)))
        }
        Logged::Fear {
            stack,
            count,
            by,
            number,
        } => {
            let (frightener, _) = named(*by, *number)?;
            let (n, count) = named(*stack, *count)?;
            Some(fear(catalogue, frightener.stack(*number), n.stack(count)))
        }
        Logged::Round => Some(catalogue.text("battle.log.round", &Args::new())),
        Logged::Cast {
            side,
            spell,
            target,
            count,
        } => {
            let (n, count) = named(*target, *count)?;
            let hero = catalogue.text(&key(Name::Hero(*side))?, &Args::new());
            let spell = catalogue.text(&key(Name::Spell(*spell))?, &Args::new());
            Some(cast_on(catalogue, &hero, &spell, n.stack(count)))
        }
        Logged::Resisted { stack, count } => {
            let (n, count) = named(*stack, *count)?;
            Some(resisted(catalogue, n.stack(count)))
        }
        Logged::Drain {
            stack,
            count,
            hit_points,
            raised,
            target,
            target_count,
        } => {
            let (drinkers, count) = named(*stack, *count)?;
            let (struck, target_count) = named(*target, *target_count)?;
            let hit_points = i64::try_from(*hit_points).unwrap_or(i64::MAX);
            Some(drain(
                catalogue,
                drinkers.stack(count),
                (hit_points, *raised),
                struck.stack(target_count),
            ))
        }
        Logged::DeathStare {
            stack,
            count,
            target,
            kills,
        } => {
            let (gorgons, count) = named(*stack, *count)?;
            let (struck, kills) = named(*target, *kills)?;
            Some(death_stare(
                catalogue,
                gorgons.stack(count),
                struck.stack(kills),
            ))
        }
        Logged::Thunderbolt { stack, count } => {
            let (n, count) = named(*stack, *count)?;
            Some(named_line(
                catalogue,
                "battle.log.thunderbolt",
                n.stack(count),
            ))
        }
        Logged::Raised { stack, count } => {
            let (n, count) = named(*stack, *count)?;
            Some(by_count(
                catalogue,
                "battle.log.raised",
                n.stack(count),
                Args::new(),
            ))
        }
        Logged::Poisoned { stack, count } => {
            let (n, count) = named(*stack, *count)?;
            Some(by_count(
                catalogue,
                "battle.log.poisoned",
                n.stack(count),
                Args::new(),
            ))
        }
        Logged::AcidBreath {
            stack,
            count,
            lowered,
        } => {
            let (n, count) = named(*stack, *count)?;
            Some(
                catalogue.text(
                    "battle.log.acid-breath",
                    &Args::new()
                        .text("creature", &name(catalogue, n.stack(count)))
                        .number("defense", i64::from(*lowered)),
                ),
            )
        }
        Logged::SpellDamage {
            spell,
            hit: (target, damage, kills),
        } => {
            let spell = key(Name::Spell(*spell))?;
            let struck = stack(*target)?;
            let hit = Hit {
                creature: &struck.creature,
                war_machine: struck.war_machine,
                damage: *damage,
                kills: *kills,
            };
            let attacker = Stack {
                creature: &spell,
                count: 1,
            };
            Some(strike(catalogue, attacker, Spread::Target, &[hit]))
        }
        Logged::CastAll { side, spell } => {
            let hero = catalogue.text(&key(Name::Hero(*side))?, &Args::new());
            let spell = catalogue.text(&key(Name::Spell(*spell))?, &Args::new());
            Some(cast_all(catalogue, &hero, &spell))
        }
        Logged::MassDamage {
            spell,
            damage,
            undead,
        } => {
            let damage = i64::try_from(*damage).unwrap_or(i64::MAX);
            let args = Args::new().number("damage", damage);
            Some(if *undead {
                let spell = catalogue.text(&key(Name::Spell(*spell))?, &Args::new());
                catalogue.text("battle.log.destroy-undead", &args.text("spell", &spell))
            } else {
                catalogue.text("battle.log.death-ripple", &args)
            })
        }
        Logged::Armageddon { damage } => {
            let damage = i64::try_from(*damage).unwrap_or(i64::MAX);
            let args = Args::new().number("damage", damage);
            Some(catalogue.text("battle.log.armageddon", &args))
        }
        Logged::AreaDamage {
            spell,
            damage,
            kills,
            target,
        } => {
            let spell = key(Name::Spell(*spell))?;
            // Several stacks: the line names the word for creatures by the kills.
            let struck = match target {
                Some(t) => stack(*t)?,
                None => Named {
                    creature: String::from("word.creature"),
                    war_machine: false,
                },
            };
            let hit = Hit {
                creature: &struck.creature,
                war_machine: struck.war_machine,
                damage: *damage,
                kills: *kills,
            };
            let attacker = Stack {
                creature: &spell,
                count: 1,
            };
            Some(strike(catalogue, attacker, Spread::Target, &[hit]))
        }
    }
}

impl Named {
    /// The stack as a line names it with `count` creatures.
    fn stack(&self, count: u32) -> Stack<'_> {
        Stack {
            creature: &self.creature,
            count,
        }
    }
}

/// How long new entries keep the status line out (0x472F09, 0x472C06): 3 000 ms.
pub const NEW_FOR: u32 = 3000;

/// The battle's log as the executable keeps it (the combat dialog's +0x54 to +0x6C; design §9.4,
/// §9.8): its entries, how many of the last are new (+0x64, at most 2), the first shown (+0x68),
/// when it last showed them (+0x6C) and the text its item shows, the status line's too.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Log {
    /// The entries, oldest first.
    pub entries: Vec<String>,
    /// How many of the last entries are new.
    pub fresh: u32,
    /// The first entry shown.
    pub first: usize,
    /// When it last showed its entries (ms).
    pub at: u32,
    /// The text its item shows.
    pub text: String,
}

impl Log {
    /// A line enters the log at `now` (0x472E90 with its second argument 1): its entries
    /// ([`entries`], by `lines`) added and new, at most two new in all, the log shown from the
    /// first new one.
    pub fn add(&mut self, line: &str, now: u32, lines: &dyn Fn(&str) -> i64) {
        let new = entries(line, lines);
        let added = u32::try_from(new.len()).unwrap_or(u32::MAX);
        self.at = now;
        self.fresh = self.fresh.saturating_add(added).min(2);
        self.entries.extend(new);
        let fresh = usize::try_from(self.fresh).unwrap_or(usize::MAX);
        self.show_from(self.entries.len().saturating_sub(fresh), now);
    }

    /// The status line (0x472E90 with its second argument 0): `text` replaces the item's, unless
    /// new entries showed less than [`NEW_FOR`] ago and `force` is false.
    pub fn status(&mut self, text: &str, now: u32, force: bool) {
        if self.fresh > 0 && now.saturating_sub(self.at) < NEW_FOR && !force {
            return;
        }
        self.text = String::from(text);
    }

    /// Entries new for [`NEW_FOR`] or more are new no longer, and the status line is emptied
    /// (0x472BF0).
    pub fn settle(&mut self, now: u32) {
        if self.fresh != 0 && now.saturating_sub(self.at) >= NEW_FOR {
            self.fresh = 0;
            self.status("", now, false);
        }
    }

    /// The scroll buttons and the arrow keys (0x472DF0, 0x472E40, 0x472DB0): the log shown from
    /// `by` entries later (earlier when negative), from the next-to-last at most and the first
    /// at least.
    pub fn scroll(&mut self, by: i64, now: u32) {
        let count = i64::try_from(self.entries.len()).unwrap_or(i64::MAX);
        let first = i64::try_from(self.first).unwrap_or(0);
        let to = first.saturating_add(by).min(count - 2).max(0);
        self.show_from(usize::try_from(to).unwrap_or(0), now);
    }

    /// The log shown from entry `first` (`battle_log_show_from` 0x472C30): nothing past the last.
    fn show_from(&mut self, first: usize, now: u32) {
        if first >= self.entries.len() {
            return;
        }
        self.first = first;
        self.text = shown(&self.entries, first);
        self.at = now;
    }
}

/// The text the log's item shows from each time on (E17 M3 T6b), as the executable's log keeps it
/// through `logged` (in time order): each line worded by `word`, its entries by `lines`; at the
/// turn of a stack `human` names the new entries settle, and the status line takes what the
/// mouse's hints write (`Logged::Status`, E17 M3 T6c-2: `crate::mouse::track::follow` puts them
/// in). The first change is the opening's empty log at 0.
#[must_use]
pub fn texts(
    logged: &[(u32, Logged)],
    word: &dyn Fn(&Logged) -> Option<String>,
    lines: &dyn Fn(&str) -> i64,
    human: &dyn Fn(u8) -> bool,
) -> Vec<(u32, String)> {
    let mut log = Log::default();
    let mut out = vec![(0, String::new())];
    for (now, happened) in logged {
        let now = *now;
        match happened {
            Logged::Action => log.settle(now),
            Logged::Turn(stack) if human(*stack) => log.settle(now),
            Logged::Turn(_) => {}
            Logged::Status { text, forced } => log.status(text, now, *forced),
            Logged::Scroll(by) => log.scroll(*by, now),
            other => {
                if let Some(line) = word(other) {
                    log.add(&line, now, lines);
                }
            }
        }
        if out.last().is_some_and(|(_, text)| *text != log.text) {
            out.push((now, log.text.clone()));
        }
    }
    out
}

/// The text of `texts` ([`texts`]) at `t`: the last change by then.
#[must_use]
pub fn text_at(texts: &[(u32, String)], t: u32) -> &str {
    texts
        .iter()
        .rev()
        .find(|(at, _)| *at <= t)
        .map_or("", |(_, text)| text.as_str())
}

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
    named_line(catalogue, "battle.log.morale-good", stack)
}

/// Bad morale (`bad_morale_check` 0x464CDF).
#[must_use]
pub fn morale_bad(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    named_line(catalogue, "battle.log.morale-bad", stack)
}

/// Good luck on a strike or a shot (0x44187A, 0x43F998).
#[must_use]
pub fn luck_good(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    named_line(catalogue, "battle.log.luck-good", stack)
}

/// Fear freezes a stack (`fear_check` 0x464EFF, E17 M3 T7b-2b-2b): the frightening creature's
/// name by the number of them on the other side, then the frozen stack's by its own.
#[must_use]
pub fn fear(catalogue: &Catalogue, frightener: Stack<'_>, stack: Stack<'_>) -> String {
    catalogue.text(
        "battle.log.fear",
        &Args::new()
            .text("frightener", &name(catalogue, frightener))
            .text("creature", &name(catalogue, stack)),
    )
}

/// Life Drain (`striker_effects` 0x440D1F, E17 M3 T7b-2b-3): the drinkers' name and verb by their
/// number before the creatures it raised, the hit points drained and the struck stack's name; then
/// the raised creatures' row when any (0x440DD1: its line break starts it on a line of its own,
/// as the log's entries take it).
#[must_use]
pub fn drain(
    catalogue: &Catalogue,
    drinkers: Stack<'_>,
    (hit_points, raised): (i64, u32),
    struck: Stack<'_>,
) -> String {
    let mut line = by_count(
        catalogue,
        "battle.log.drain",
        drinkers,
        Args::new()
            .number("hit_points", hit_points)
            .text("target", &name(catalogue, struck)),
    );
    if raised > 0 {
        let rise = catalogue.text(
            "battle.log.drain-raised",
            &Args::new().count(i64::from(raised)),
        );
        line.push_str(&rise);
    }
    line
}

/// Death Stare (`striker_effects` 0x440FD3, 0x441086; E17 M3 T7b-2b-3): the creatures killed (the
/// row by their number, which picks the struck creature's name) and the Gorgons' name by theirs.
#[must_use]
pub fn death_stare(catalogue: &Catalogue, gorgons: Stack<'_>, killed: Stack<'_>) -> String {
    catalogue.text(
        "battle.log.death-stare",
        &Args::new()
            .text("creature", &name(catalogue, killed))
            .text("starer", &name(catalogue, gorgons))
            .count(i64::from(killed.count)),
    )
}

/// The First Aid Tent heals a stack (`first_aid_tent_action` 0x478CE4, E17 M3 T6b): the tent's
/// name by its number, the patient's by its, and the hit points healed.
#[must_use]
pub fn heal(
    catalogue: &Catalogue,
    healer: Stack<'_>,
    patient: Stack<'_>,
    hit_points: u32,
) -> String {
    catalogue.text(
        "battle.log.heal",
        &Args::new()
            .text("healer", &name(catalogue, healer))
            .text("creature", &name(catalogue, patient))
            .number("hit_points", i64::from(hit_points)),
    )
}

/// A hero's spell on a stack (`battle_log_cast` 0x5A8950, E17 M3 T7b-1): the hero's name, the
/// spell's and the stack's by its number.
#[must_use]
pub fn cast_on(catalogue: &Catalogue, hero: &str, spell: &str, stack: Stack<'_>) -> String {
    catalogue.text(
        "battle.log.cast-on",
        &Args::new()
            .text("hero", hero)
            .text("spell", spell)
            .text("creature", &name(catalogue, stack)),
    )
}

/// A hero's spell without a target (`battle_log_cast` 0x5A8950, E17 M3 T7c-1): the hero's name and
/// the spell's.
#[must_use]
pub fn cast_all(catalogue: &Catalogue, hero: &str, spell: &str) -> String {
    catalogue.text(
        "battle.log.cast",
        &Args::new().text("hero", hero).text("spell", spell),
    )
}

/// A spell resisted by the stack it was aimed at (0x5A1FB0, E17 M3 T7b-1): its name and the verb
/// by its number.
#[must_use]
pub fn resisted(catalogue: &Catalogue, stack: Stack<'_>) -> String {
    let verb = catalogue.text("word.resist", &Args::new().count(i64::from(stack.count)));
    catalogue.text(
        "battle.log.resisted",
        &Args::new()
            .text("creature", &name(catalogue, stack))
            .text("verb", &verb),
    )
}

/// The log's entries a line becomes (`battle_log_add` 0x472E90): a line without a line break is
/// one entry; with one, its first break becomes a space when the line then takes fewer than two
/// lines (`lines`: how many the log's font takes at the log's width, `text_line_count` 0x4B5820),
/// else the line is two entries, split there (any later break stays in the second). (A concrete
/// type for `lines`: a generic one shows false misses in the coverage summary.)
#[must_use]
pub fn entries(line: &str, lines: &dyn Fn(&str) -> i64) -> Vec<String> {
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

/// The log item's text (`battle_log_show_from` 0x472C30, design §9.5): the entry `first` on the
/// box's first line and, after a line break, the next one on its second, if there is one; nothing
/// from past the last entry. (Which entry shows first, set by the entries a line added and by the
/// scroll buttons, is the panel's: M3 T6.)
#[must_use]
pub fn shown(entries: &[String], first: usize) -> String {
    let lines: Vec<&str> = entries
        .iter()
        .skip(first)
        .take(2)
        .map(String::as_str)
        .collect();
    lines.join("\n")
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
fn named_line(catalogue: &Catalogue, key: &str, stack: Stack<'_>) -> String {
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
