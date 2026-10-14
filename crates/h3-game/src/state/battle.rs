//! Battles through the game (ADR-0017 points 6 and 11; E08 T19a): a battle a visit starts is a
//! pending decision of the players of its sides ([`OpenBattle`]). The game builds its setup from
//! the visit and the store of heroes (GAM-BAT-001), routes the commands of the owner of the side
//! to act to the battle, maps its events to the owners of the sides they are visible to, and
//! applies its outcome when it ends (GAM-BAT-002 to GAM-BAT-005): the armies and the map's part,
//! the heroes' through `h3_heroes`' changes (E08 T19a2a), a surrender's gold.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_adventure::{BattleStart, Combatant, Pos, Viewers};
use h3_battle::{BattleState, Command, Event, Visibility, generate_setup_obstacles};
use h3_content::{OBSTACLES_FROM_TILE, Ruleset};
use h3_contracts::{
    Army, ArmyStack, BattleHero, BattleOutcome, BattleResult, BattleSetup, BattleSide, HeroId,
    PlayerId, StackOrigin, StackRecord,
};
use h3_heroes::{
    HeroChange, apply_change, check_change, derive_battle_hero, gained_experience,
    lost_war_machine, max_mana, next_capture, war_machines,
};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{AdventureSink, GameCommandError, GameEvent, Sink};

/// A battle in progress on the adventure map (ADR-0017 points 6 and 11; GAM-BAT-001, E08 T19a):
/// the battle's state, the players owning its sides (the attacker's the visitor's; against
/// another player's hero, the defender's that player's), the heroes leading them and the tile it
/// is fought at. A pending decision of its owners, shared between forks and copied on its first
/// write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenBattle {
    state: BattleState,
    owners: [Option<PlayerId>; 2],
    heroes: [Option<HeroId>; 2],
    tile: Pos,
}

impl OpenBattle {
    /// The battle's state: server-only as a whole (its stream, the other side's private part);
    /// each owner sees its side's observation ([`crate::GameObservation::battle`]).
    pub const fn state(&self) -> &BattleState {
        &self.state
    }

    /// The players owning the sides, by [`BattleSide::index`].
    pub const fn owners(&self) -> [Option<PlayerId>; 2] {
        self.owners
    }

    /// The heroes leading the sides, by [`BattleSide::index`].
    pub const fn heroes(&self) -> [Option<HeroId>; 2] {
        self.heroes
    }

    /// The tile the battle is fought at: the defender's.
    pub const fn tile(&self) -> Pos {
        self.tile
    }

    /// The side `player` owns, if any.
    pub fn side_of(&self, player: PlayerId) -> Option<BattleSide> {
        BattleSide::BOTH
            .into_iter()
            .find(|side| self.owners[side.index()] == Some(player))
    }

    /// The player owning the side to act, while the battle goes on.
    pub fn owner_to_act(&self) -> Option<PlayerId> {
        self.state
            .side_to_act()
            .and_then(|side| self.owners[side.index()])
    }
}

impl CanonicalHash for OpenBattle {
    /// The game state's v6 encoding of a battle: the owners and the heroes by side (each 0 for
    /// none, else 1 and the id), the tile, the battle state's hash.
    fn hash_into(&self, h: &mut StateHasher) {
        for owner in self.owners {
            owner.map(PlayerId::raw).hash_into(h);
        }
        for hero in self.heroes {
            hero.map(HeroId::raw).hash_into(h);
        }
        self.tile.hash_into(h);
        h.write_u64(self.state.state_hash());
    }
}

/// The owners as a set of players: who sees a battle's start, its public events and its end.
fn owners_seeing(owners: [Option<PlayerId>; 2]) -> Viewers {
    owners
        .into_iter()
        .flatten()
        .fold(Viewers::NONE, Viewers::with)
}

/// Who of a battle's `owners` sees an event of `visibility` (ADR-0017 point 11): both a public
/// one, a side's that side's owner.
pub(crate) fn seen_by(owners: [Option<PlayerId>; 2], visibility: Visibility) -> Viewers {
    match visibility {
        Visibility::Public => owners_seeing(owners),
        Visibility::Side(side) => owners_seeing([owners[side.index()], None]),
    }
}

/// The game's sink of a battle's events: each event with the owners that see it ([`seen_by`]).
struct BattleSink<'a, S> {
    sink: &'a mut S,
    owners: [Option<PlayerId>; 2],
}

impl<S: Sink> h3_battle::Sink for BattleSink<'_, S> {
    fn emit(&mut self, event: Event) {
        let seen_by = seen_by(self.owners, event.visibility);
        self.sink.emit(GameEvent::Battle { event, seen_by });
    }
}

/// The seed of a battle's obstacles from its tile under `ADV-QUIRK-5` (GAM-BAT-001): the
/// original's 110 291 x + 167 801 y + 81 569 in 32-bit arithmetic, the level not included (code
/// 0x463A08; survey `E08-battles.md` §4.2).
fn tile_seed(tile: Pos) -> u64 {
    let (x, y) = (u32::from(tile.x), u32::from(tile.y));
    u64::from(
        110_291u32
            .wrapping_mul(x)
            .wrapping_add(167_801u32.wrapping_mul(y))
            .wrapping_add(81_569),
    )
}

impl GameState {
    /// The battle of `player`'s first open decision, if it is a battle (GAM-BAT-001, E08 T19a).
    pub fn battle(&self, player: PlayerId) -> Option<&OpenBattle> {
        self.decisions.as_deref().unwrap_or_default()[self.first(player)?]
            .battle()
            .map(|battle| &**battle)
    }

    /// A hero's battle hero (`BattleSetup.heroes`): the hero model's translation of its details
    /// (ADR-0013 point 2) with its maximum spell points (HER-MAN-001).
    fn battle_hero(&self, core: &Ruleset, hero: HeroId) -> Option<BattleHero> {
        self.hero(hero).and_then(|details| {
            derive_battle_hero(details, core).ok().map(|mut battle| {
                battle.magic.max_mana = max_mana(details, core);
                battle
            })
        })
    }

    /// Starts the battle a visit of `player`'s hero started (GAM-BAT-001, GAM-BAT-004; E08 T19a):
    /// its setup from the visit and the store of heroes — the sides' armies and formations as on
    /// the map, each hero's battle hero and war machines (the hero model's), the terrain and the
    /// battlefield of the tile, each side owner's gold as its funds (a surrender's price, the
    /// battle part's resource) — and its obstacles; the battle's own seed is the attacking player's
    /// stream's next draw, and the obstacles' seed the tile's under `ADV-QUIRK-5` (`faithful`),
    /// else the stream's draw after it (ADR-0017 point 4). The battle opens as a decision of both
    /// players, told to them with its start's events, unless it is already over (then its outcome
    /// applies at once). A setup the battle refuses cannot come from a valid game (the armies,
    /// the heroes and the parts were checked when they were built); it would start no battle.
    // rule: GAM-BAT-001, GAM-BAT-004
    pub(crate) fn start_battle<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        start: BattleStart,
        sink: &mut S,
    ) {
        let core = rules.core();
        let sides: [&Combatant; 2] = [&start.attacker, &start.defender];
        let resource = rules.battle().battle().surrender.map(|s| s.resource);
        let funds = |owner: PlayerId| {
            resource
                .zip(self.treasury(owner))
                .and_then(|(resource, held)| held.get(usize::from(resource.raw())).copied())
                .unwrap_or(0)
        };
        let mut setup = BattleSetup {
            attacker: Army {
                stacks: start.attacker.army.clone(),
            },
            defender: Army {
                stacks: start.defender.army.clone(),
            },
            heroes: sides.map(|side| self.battle_hero(core, side.hero)),
            terrain: Some(start.terrain),
            battlefield: start.battlefield.clone(),
            war_machines: sides.map(|side| {
                self.hero(side.hero)
                    .map(|details| war_machines(details, core))
                    .unwrap_or_default()
            }),
            formations: sides.map(|side| side.formation),
            funds: sides.map(|side| funds(side.owner)),
            ..BattleSetup::default()
        };
        // ADV-QUIRK-5 (`faithful`): the tile's seed; else the stream's second draw.
        let stream = &mut self.players[player.index()].rng;
        let seed = stream.next_u64();
        let obstacles = if core.quirks().is_enabled(OBSTACLES_FROM_TILE) {
            tile_seed(start.tile)
        } else {
            stream.next_u64()
        };
        setup.obstacles =
            generate_setup_obstacles(rules.battle(), &setup, &mut Rng::from_seed(obstacles));
        let owners = sides.map(|side| Some(side.owner));
        let heroes = sides.map(|side| Some(side.hero));
        let mut events = Vec::new();
        let Ok(state) = BattleState::start(rules.battle(), &setup, seed, &mut events) else {
            return;
        };
        sink.emit(GameEvent::BattleStarted {
            heroes,
            tile: start.tile,
            seen_by: owners_seeing(owners),
        });
        let mut battle_sink = BattleSink { sink, owners };
        for event in events {
            h3_battle::Sink::emit(&mut battle_sink, event);
        }
        self.push_decision(Decision::Battle(Arc::new(OpenBattle {
            state,
            owners,
            heroes,
            tile: start.tile,
        })));
        // The battle may be over before its first command (BattleState::start).
        self.settle(
            rules,
            self.decisions.as_deref().unwrap_or_default().len() - 1,
            sink,
        );
    }

    /// `Battle`: a command of `player` to the battle of its first open decision, for the side it
    /// owns while that side is to act, in its turn or not (ADR-0017 point 6; GAM-BAT-001, E08
    /// T19a); the battle's events go to the owners that see them, and the command that ends the
    /// battle applies its outcome ([`Self::end_battle`]) and closes the decision.
    ///
    /// # Errors
    ///
    /// `UnknownPlayer`, then `NoBattle` (no decision, or the first is an offer), then
    /// `NotYourSide`, then the battle's own refusal; the state is then unchanged.
    // rule: GAM-BAT-001
    pub(crate) fn battle_command<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        command: Command,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        self.players
            .get(player.index())
            .ok_or(GameCommandError::UnknownPlayer(player))?;
        let at = self.first(player).ok_or(GameCommandError::NoBattle)?;
        // The decision list is copied on its first write when a fork shares it (its entries are
        // references), the battle only once the command is valid.
        let list = self
            .decisions
            .as_mut()
            .map(Arc::make_mut)
            .unwrap_or_default();
        let battle = list
            .get_mut(at)
            .and_then(Decision::battle_mut)
            .ok_or(GameCommandError::NoBattle)?;
        let side = battle
            .state
            .side_to_act()
            .filter(|side| battle.owners[side.index()] == Some(player))
            .ok_or(GameCommandError::NotYourSide)?;
        h3_battle::validate(&battle.state, rules.battle(), side, command)
            .map_err(GameCommandError::Battle)?;
        let open = Arc::make_mut(battle);
        let mut events = BattleSink {
            sink: &mut *sink,
            owners: open.owners,
        };
        // Validated: the command applies.
        let _ = h3_battle::apply(&mut open.state, rules.battle(), side, command, &mut events);
        self.settle(rules, at, sink);
        Ok(())
    }

    /// Ends the battle of the decision at `at` if it is over: closes the decision and applies the
    /// outcome ([`Self::end_battle`]).
    fn settle<S: Sink>(&mut self, rules: &GameRuleset, at: usize, sink: &mut S) {
        let ended = self
            .decisions
            .as_deref()
            .unwrap_or_default()
            .get(at)
            .and_then(Decision::battle)
            .and_then(|battle| {
                let sides = (battle.owners, battle.heroes);
                battle.state.outcome().map(|outcome| (sides, outcome))
            });
        if let Some((sides, outcome)) = ended {
            self.close(at);
            self.end_battle(rules, sides, &outcome, sink);
        }
    }

    /// Applies an ended battle's outcome (ADR-0017 point 11; E08 T19a), told first to its owners
    /// as its end, one context after the other: in the adventure (GAM-BAT-002, GAM-BAT-003) a hero
    /// whose side won and has army survivors keeps them as its army (a damaged top creature
    /// forgotten), every other hero — lost, retreated, surrendered, in a draw, or left with stacks
    /// created in the battle only (BAT-END-001) — leaves the map; then the heroes' part
    /// ([`Self::heroes_after_battle`], E08 T19a2a); then a surrender's price leaves the
    /// surrendering side's owner's treasury and enters the other side's owner's (GAM-BAT-005,
    /// D-13), each told to its owner.
    // rule: GAM-BAT-002, GAM-BAT-003, GAM-BAT-005
    pub(crate) fn end_battle<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (owners, heroes): ([Option<PlayerId>; 2], [Option<HeroId>; 2]),
        outcome: &BattleOutcome,
        sink: &mut S,
    ) {
        let result = outcome.result;
        sink.emit(GameEvent::BattleEnded {
            heroes,
            result,
            seen_by: owners_seeing(owners),
        });
        let army = |side: BattleSide| -> Option<&[ArmyStack]> {
            let survivors = &outcome.survivors[side.index()].stacks;
            (result.winner() == Some(side) && !survivors.is_empty()).then_some(&survivors[..])
        };
        let changes: Vec<(HeroId, Option<&[ArmyStack]>)> = BattleSide::BOTH
            .into_iter()
            .filter_map(|side| heroes[side.index()].map(|hero| (hero, army(side))))
            .collect();
        self.adventure
            .end_battle(rules.core(), &changes, &mut AdventureSink(&mut *sink));
        let winner = result.winner().filter(|&side| army(side).is_some());
        self.heroes_after_battle(rules, (owners, heroes), outcome, winner, sink);
        let resource = rules.battle().battle().surrender.map(|s| s.resource);
        let surrendered = match result {
            BattleResult::Surrender(side) => Some(side),
            _ => None,
        };
        let payment =
            surrendered.map(|side| (owners[side.index()], owners[side.opponent().index()]));
        if let (Some((Some(payer), Some(payee))), Some(price), Some(resource)) =
            (payment, outcome.surrender_cost, resource)
        {
            let amount = self.treasuries.pay(payer, resource, price);
            sink.emit(GameEvent::ResourcePaid {
                player: payer,
                resource,
                amount,
            });
            let amount = self.treasuries.gain(payee, resource, amount);
            sink.emit(GameEvent::ResourceGained {
                player: payee,
                resource,
                amount,
            });
        }
    }

    /// The heroes' part of an ended battle's outcome (ADR-0017 point 11; E08 T19a2a), after the
    /// adventure's, each change checked by `h3_heroes` and told to the hero's owner
    /// ([`Self::change_hero`]). Each side's hero keeps the spell points it has left
    /// (`HeroOutcome::mana`) and loses the artifact of each war machine destroyed in the battle
    /// (HER-WAR-002), whatever the result (GAM-BAT-002). The `winner`'s hero — that of the side
    /// that won and keeps its army, so it stays on the map — gains the battle's experience
    /// (GAM-BAT-002: [`battle_experience`], grown by Learning, HER-EXP-001) and, after a victory
    /// over a hero, its artifacts (GAM-BAT-004, HER-ART-004): one by one while the winner has room
    /// (HER-CHG-002), each moved from the loser's details to the winner's. The original's order
    /// where read (`battle_end` 0x477470: the experience 0x477747, Necromancy 0x47776E, Eagle
    /// Eye 0x477776, the capture 0x477782); the spell points are spent during the original's battle,
    /// and the war machines' artifacts go first, as in VCMI.
    fn heroes_after_battle<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (owners, heroes): ([Option<PlayerId>; 2], [Option<HeroId>; 2]),
        outcome: &BattleOutcome,
        winner: Option<BattleSide>,
        sink: &mut S,
    ) {
        let core = rules.core();
        let sides = BattleSide::BOTH.map(|side| {
            let i = side.index();
            heroes[i].map(|hero| (i, hero, owners[i]))
        });
        for (i, hero, owner) in sides.into_iter().flatten() {
            // rule: GAM-BAT-002
            let mana = outcome.heroes[i].as_ref().map(|h| h.mana);
            let changed = mana.filter(|&m| self.hero(hero).is_some_and(|d| d.mana != m));
            if let Some(points) = changed {
                self.change_hero(core, hero, owner, HeroChange::SetSpellPoints(points), sink);
            }
            let destroyed = outcome.stacks[i]
                .iter()
                .filter(|s| s.origin == StackOrigin::WarMachine && s.end == 0);
            for record in destroyed {
                let lost = self
                    .hero(hero)
                    .and_then(|d| lost_war_machine(d, core, record.creature));
                if let Some(place) = lost {
                    self.change_hero(core, hero, owner, HeroChange::RemoveArtifact(place), sink);
                }
            }
        }
        let won = winner.and_then(|w| heroes[w.index()].map(|hero| (w, hero)));
        let Some((w, hero)) = won else {
            return;
        };
        let (l, owner) = (w.opponent().index(), owners[w.index()]);
        let defeated = matches!(outcome.result, BattleResult::Victory(_)) && heroes[l].is_some();
        let base = battle_experience(rules, &outcome.stacks[l], defeated);
        let gained = self
            .hero(hero)
            .map_or(0, |d| gained_experience(d, core, base));
        if gained > 0 {
            self.change_hero(core, hero, owner, HeroChange::GainExperience(gained), sink);
        }
        // stub: ADV-STUB-3
        // rule: GAM-BAT-004
        if let Some(lost) = heroes[l].filter(|_| defeated) {
            while let Some((place, item)) = self.hero(lost).and_then(|d| next_capture(d, core)) {
                let add = HeroChange::AddArtifact(item);
                if self
                    .hero(hero)
                    .is_none_or(|d| check_change(d, core, add).is_err())
                {
                    break;
                }
                self.change_hero(core, hero, owner, add, sink);
                let remove = HeroChange::RemoveArtifact(place);
                self.change_hero(core, lost, owners[l], remove, sink);
            }
        }
    }

    /// Applies `change`, checked, to the details of `hero` in the store (copied on its first
    /// write when a fork shares it) and tells it to the hero's owner (`HeroChanged`; E08 T19a2a).
    fn change_hero<S: Sink>(
        &mut self,
        core: &Ruleset,
        hero: HeroId,
        owner: Option<PlayerId>,
        change: HeroChange,
        sink: &mut S,
    ) {
        let store = Arc::make_mut(&mut self.heroes);
        // Every hero of a battle is in the store (GAM-HER-001).
        store
            .get_mut(usize::from(hero.raw()))
            .into_iter()
            .for_each(|details| apply_change(Arc::make_mut(details), core, change));
        owner.into_iter().for_each(|player| {
            sink.emit(GameEvent::HeroChanged {
                player,
                hero,
                change,
            });
        });
    }
}

/// The experience a battle gives the winner's hero before Learning (GAM-BAT-002, GAM-BAT-004;
/// E08 T19a2a): the hit points of the creatures the loser's army stacks and raised stacks lost
/// (`StackRecord`: `start − end`, each at its type's hit points, without artifacts), not those of
/// its war machines or of stacks the battle created (summoned stacks, clones), plus
/// [`DEFEATED_HERO_EXPERIENCE`] for a `defeated` hero (a victory over a side with a hero; not after
/// a retreat or a surrender). Sums saturate. The original's `battle_experience` (0x46A350: the
/// loser's 20 records without the summoned and war machine flags, `start − count` times the
/// type's hit points; +500 for the loser's hero, −500 again when a side retreated or
/// surrendered).
// rule: GAM-BAT-002, GAM-BAT-004
fn battle_experience(rules: &GameRuleset, stacks: &[StackRecord], defeated: bool) -> u32 {
    let killed = stacks
        .iter()
        .filter(|s| matches!(s.origin, StackOrigin::Army(_) | StackOrigin::Raised(_)))
        .map(|s| {
            let hit_points = rules
                .battle()
                .creature(s.creature)
                .map_or(0, |c| c.hit_points);
            s.start.saturating_sub(s.end).saturating_mul(hit_points)
        })
        .fold(0_u32, u32::saturating_add);
    let bonus = if defeated {
        DEFEATED_HERO_EXPERIENCE
    } else {
        0
    };
    killed.saturating_add(bonus)
}

/// The experience for defeating a hero in battle (GAM-BAT-004; the original's 500 at 0x46A3DF,
/// the wiki's Experience page).
pub(crate) const DEFEATED_HERO_EXPERIENCE: u32 = 500;
