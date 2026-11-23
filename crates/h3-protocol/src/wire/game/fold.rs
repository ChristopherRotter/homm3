//! The fold of the whole game's events (E16 M5 T3c1; SRV-Q26, SRV-Q27): a viewer's
//! [`GameObservation`] kept current by its projected events ([`GameObservation::fold`]), as a
//! client and a record's reader keep it between snapshots (`docs/epics/E16-design.md` §14: events
//! only).
//!
//! A fold applies what an event carries and nothing the rules derive (SRV-Q27 (2)). What no event
//! carries yet stays as it was, each a gap for the adventure and town lanes (SRV-Q26, SRV-Q27):
//! a reveal's tiles, objects, heroes and towns; a hero, object or boat coming into sight without a
//! reveal; two changes told to nobody who sees them; the views of an
//! offer, a choice, a University, a hire dialog, a market, an Altar and a Hill Fort; a town's or a map garrison's view moved by a
//! Visions holder's reach; and the values the rules derive (where an added artifact or joined creatures land, a
//! capped gain, what a new owner's team sees of an object). The
//! battle is its snapshots' (SRV-Q27 (1)): its events are the battle client's, and the fold only
//! ends it.
//!
//! Each gap is a stub (`SRV-STUB-3` to `SRV-STUB-11` and the town lane's `TWN-STUB-13`,
//! `docs/stubs.toml`, marked where its event is folded), lifted when its lane's events carry it
//! (`SRV-STUB-2`, a hero's movement points, by `MovementChanged`). A reveal's content comes after
//! the events that a command told the viewer, when the command explored tiles for it
//! ([`GameObservation::reveal`], E16 M5 T5c; `SRV-STUB-1` lifted): the tiles whose sight changed,
//! the heroes, objects, empty boats, towns and (E08 T114b) map garrisons it newly sees. The gate is `tests/game_fold.rs`:
//! over the golden replays the fold equals `observe` after every command for every viewer, but
//! for the parts a stub names.
//!
//! Provisional (E16 M5 part 1's review): the fold's place, what it derives from an event, a
//! reveal folded after the events.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::{
    Adventure, AdventureEvent, AdventureReveal, Amount, ArmyRef, ArmySlot, ArmyStack, Artifact,
    ArtifactPlace, Boat, Bonuses, Decision, GameEvent, GameObservation, GameReveal, Garrison,
    GarrisonHero, GarrisonStack, Guards, HeroChange, HeroDetails, HeroInfo, HiredPart, LevelUp,
    MapGarrison, MapHero, Monster, Object, ObjectRef, OfferLevels, OwnMapHero, Pos, PrimarySkill,
    RevealedArea, RevealedView, SkillLevel, TavernHero, TavernPart, Tile, Town, TownOwnerFacts,
};
use crate::wire::WireU64;

/// A creature month's monster's masks: its one tile, blocked and visited from its neighbours
/// (`h3_adventure::AdventureEventKind::MonsterAdded`, ADV-MON-014).
const ONE_TILE: u64 = 1;

/// A built boat's facing, the map's boats' (`h3_adventure::AdventureEventKind::BoatBuilt`,
/// ADV-SHY-003).
const BUILT_FACING: u8 = 2;

/// The columns and rows of an object's masks ([`Object::blocked`]).
const MASK_COLUMNS: u16 = 8;
/// See [`MASK_COLUMNS`].
const MASK_ROWS: u16 = 6;

impl GameObservation {
    /// Folds one of the viewer's events (`crate::map::game::events`, in their order) into this
    /// view: what the event carries, nothing the rules derive; the gaps the module names stay as
    /// they were. Since protocol v80 the viewer's list of heroes follows its heroes on the map
    /// ([`Self::keep_list`]).
    pub fn fold(&mut self, event: &GameEvent) {
        self.fold_event(event);
        self.keep_list();
    }

    /// Keeps the viewer's list of heroes ([`GameObservation::hero_list`], GAM-HER-003; E08 T141a)
    /// in step with its heroes on the map: one that left the map leaves the list, the rest closing
    /// up, and one that came onto it joins its end (in id order when several come in one event,
    /// which only the start's snapshot holds).
    fn keep_list(&mut self) {
        let viewer = self.viewer;
        let on_map = |id: u16, heroes: &[MapHero]| {
            heroes.iter().any(|hero| hero.id == id && hero.owner == viewer)
        };
        let heroes = &self.adventure.heroes;
        self.hero_list.retain(|&id| on_map(id, heroes));
        for hero in heroes.iter().filter(|hero| hero.owner == viewer) {
            if !self.hero_list.contains(&hero.id) {
                self.hero_list.push(hero.id);
            }
        }
    }

    /// [`Self::fold`]'s event.
    fn fold_event(&mut self, event: &GameEvent) {
        match *event {
            GameEvent::TurnEnded { player } => self.set_acting(player, false),
            GameEvent::DayStarted { day } => {
                // The spell points a new day restores are not carried (stub: SRV-STUB-6).
                self.day = day;
                // Today's timed events fire at the viewer's turn start (GAM-CAL-004).
                self.events_today.clear();
            }
            GameEvent::WeekStarted { week: _, kind } => self.week = kind,
            GameEvent::MonthStarted { month: _, kind } => self.month = kind,
            GameEvent::TurnStarted { player } => self.start_turn(player),
            GameEvent::TimedEventFired { player: _, event } => self.events_today.push(event),
            GameEvent::Adventure { event } => self.fold_adventure(event),
            GameEvent::ResourceGained {
                player: _,
                resource,
                amount,
            } => {
                if let Some(held) = self.treasury.get_mut(usize::from(resource)) {
                    *held = held.saturating_add(amount);
                }
            }
            GameEvent::ResourceLost {
                player: _,
                resource,
                amount,
            }
            | GameEvent::ResourcePaid {
                player: _,
                resource,
                amount,
            } => {
                if let Some(held) = self.treasury.get_mut(usize::from(resource)) {
                    *held = held.saturating_sub(amount);
                }
            }
            // Their views are not carried (SRV-Q27 (2)): an offer's kind and price, a choice's
            // options, a University's skills and slots, a hire dialog's creatures and costs, since
            // E08 T133b (v75) a market's window, since E08 T134a (v76) an Altar's, since E08
            // T134b1 (v77) a Hill Fort's (stub: SRV-STUB-5).
            GameEvent::OfferOpened { .. }
            | GameEvent::ChoiceOpened { .. }
            | GameEvent::UniversityOpened { .. }
            | GameEvent::HireOpened { .. }
            | GameEvent::MarketOpened { .. }
            | GameEvent::AltarOpened { .. }
            | GameEvent::HillFortOpened { .. } => {}
            // A message: the treasury could not pay.
            GameEvent::OfferRefused { .. } => {}
            // A Cartographer's type is not carried (stub: SRV-STUB-11).
            GameEvent::OfferAnswered { .. } => self.decision = None,
            // The battle's snapshot follows its start; its events are the battle client's
            // (SRV-Q27 (1)).
            GameEvent::BattleStarted { .. } | GameEvent::Battle { .. } => {}
            // Its losses in the heroes' armies are not carried (stub: SRV-STUB-9).
            GameEvent::BattleEnded { .. } => self.battle = None,
            GameEvent::HeroChanged {
                player: _,
                hero,
                change,
            } => {
                if let Some(details) = self.details_mut(hero) {
                    details.fold(change);
                }
            }
            // Where they join is the rules' (SRV-Q27 (2); stub: SRV-STUB-9).
            GameEvent::CreaturesRaised { .. } => {}
            GameEvent::LevelUpOpened {
                player: _,
                hero,
                level,
                primary,
                offers,
            } => {
                self.decision = Some(Decision::LevelUp {
                    level_up: LevelUp {
                        hero,
                        level,
                        primary,
                        offers,
                    },
                });
            }
            GameEvent::LevelGained {
                player: _,
                hero,
                level,
                primary,
                skill,
            } => {
                // The levels at which Wisdom and a school were last offered are not carried
                // (stub: SRV-STUB-8).
                if matches!(&self.decision, Some(Decision::LevelUp { level_up }) if level_up.hero == hero)
                {
                    self.decision = None;
                }
                if let Some(details) = self.details_mut(hero) {
                    details.gain_level(level, primary, skill);
                }
            }
            GameEvent::StackChanged {
                player: _,
                at,
                stack,
            } => self.set_stack(at, stack),
            GameEvent::FormationSet {
                player: _,
                hero,
                formation,
            } => {
                if let Some(own) = self.own_mut(hero) {
                    own.formation = formation;
                }
            }
            GameEvent::TacticsSet {
                player: _,
                hero,
                on,
            } => {
                if let Some(own) = self.own_mut(hero) {
                    own.tactics = on;
                }
            }
            GameEvent::MeetingOpened { player: _, heroes } => {
                self.decision = Some(Decision::Meeting { heroes });
            }
            GameEvent::ArmyDialogOpened {
                player: _,
                hero,
                stack,
            } => self.open_group(hero, stack),
            GameEvent::LeaveGuardsOpened {
                player: _,
                hero,
                object_type,
                anchor,
            } => {
                self.decision = Some(Decision::ArmyDialog {
                    hero,
                    group: Vec::new(),
                    mine: Some(ObjectRef {
                        object_type,
                        anchor,
                    }),
                });
            }
            GameEvent::ExchangeOpened {
                player: _,
                hero,
                garrison,
                object_type,
                anchor,
                removable,
            } => {
                self.decision = Some(Decision::Exchange {
                    hero,
                    garrison,
                    object: ObjectRef {
                        object_type,
                        anchor,
                    },
                    removable,
                    army: Vec::new(),
                });
            }
            // E09 T39a: the Tavern's view whole (its offers are the tavern heroes').
            GameEvent::TavernOpened {
                player: _,
                hero,
                object_type,
                anchor,
            } => {
                self.decision = Some(Decision::Tavern {
                    hero,
                    object: ObjectRef {
                        object_type,
                        anchor,
                    },
                });
            }
            GameEvent::DecisionClosed { player: _ } => self.close_decision(),
            // Told after its dialog closed, with its group.
            GameEvent::GroupLost { .. } => {}
            // The viewer's whole view of the town but its anchor, faction and name; its stacks
            // and its income lines follow (E09 T18b2a). A change of a Visions holder's reach is
            // not told (stub: TWN-STUB-13).
            GameEvent::TownChanged {
                player: _,
                town,
                owner,
                fort,
                capitol,
                built_today,
                level,
                slots,
                hall,
                garrison_hero,
                ref buildings,
            } => {
                if let Some(view) = self.town_mut(town) {
                    view.owner = owner;
                    view.fort = fort;
                    view.capitol = capitol;
                    view.built_today = built_today;
                    view.garrison = Garrison {
                        level,
                        stacks: Vec::new(),
                        army: slots.then(Vec::new),
                    };
                    view.owner_facts = hall.map(|hall| TownOwnerFacts {
                        hall,
                        income: Vec::new(),
                        garrison_hero,
                    });
                    view.buildings.clone_from(buildings);
                }
            }
            GameEvent::TownStackSeen {
                player: _,
                town,
                stack,
                slot,
            } => {
                if let Some(view) = self.town_mut(town) {
                    view.garrison.see(stack, slot);
                }
            }
            GameEvent::TownIncomeSeen {
                player: _,
                town,
                resource,
                amount,
            } => {
                let view = self.town_mut(town);
                if let Some(facts) = view.and_then(|view| view.owner_facts.as_mut()) {
                    facts.income.push(Amount { resource, amount });
                }
            }
            // The viewer's tavern hero in a slot, its parts following (E09 T37a, TWN-TAV-002):
            // the slot's view replaced, its lists empty until its parts come.
            GameEvent::TavernOffered {
                player: _,
                slot,
                hero,
            } => {
                let view = self.tavern.get_mut(usize::from(slot));
                view.into_iter().for_each(|view| {
                    *view = hero.map(|hero| TavernHero {
                        hero,
                        army: Vec::new(),
                        skills: Vec::new(),
                        artifacts: Vec::new(),
                    });
                });
            }
            GameEvent::TavernPartSeen {
                player: _,
                slot,
                part,
            } => {
                let view = self.tavern.get_mut(usize::from(slot));
                if let Some(view) = view.and_then(Option::as_mut) {
                    view.see(part);
                }
            }
            // A hire (E09 T38a1, TWN-TAV-003): the hero on the map as the viewer sees it; its
            // owner's view of it the slot's, its details completed by the parts following, its
            // points by the command's `MovementChanged`.
            GameEvent::HeroHired {
                player,
                hero,
                hero_type,
                slot,
                pos,
            } => {
                let held = slot.and_then(|slot| self.tavern.get_mut(usize::from(slot)));
                let view = held.and_then(Option::take);
                self.hire(
                    MapHero {
                        id: hero,
                        owner: player,
                        pos,
                        boat: None,
                        own: None,
                    },
                    hero_type,
                    view,
                );
            }
            GameEvent::HiredPartSeen {
                player: _,
                hero,
                part,
            } => {
                let details = self.details_mut(hero);
                details.into_iter().for_each(|details| details.see(part));
            }
            // A freed prisoner (E08 T141a, ADV-PRI-002): as a hire, its owner's view the event's
            // head with its lists following as `PrisonerPartSeen`, its other parts as
            // `HiredPartSeen`, its points as the command's `MovementChanged`.
            GameEvent::PrisonerFreed {
                player,
                hero,
                hero_type,
                pos,
                view,
            } => {
                let view = view.map(|hero| TavernHero {
                    hero,
                    army: Vec::new(),
                    skills: Vec::new(),
                    artifacts: Vec::new(),
                });
                self.hire(
                    MapHero {
                        id: hero,
                        owner: player,
                        pos,
                        boat: None,
                        own: None,
                    },
                    hero_type,
                    view,
                );
            }
            GameEvent::PrisonerPartSeen {
                player: _,
                hero,
                part,
            } => self.see_freed(hero, part),
            // What a full or an emptied Prison's visit tells is the client's message; an emptied
            // Prison's removal came as its `ObjectRemoved`.
            GameEvent::PrisonFull { .. } | GameEvent::PrisonEmptied { .. } => {}
            // A Den of Thieves' Thieves' Guild (E09 T39b2): a snapshot, not in the observation (a
            // client makes it with `ThievesGuild::see`).
            GameEvent::ThievesGuildSeen { .. } | GameEvent::GuildPartSeen { .. } => {}
            // The viewer's whole view of a map garrison, its stacks following (E08 T114b,
            // ADV-GAR-006): put in its place, a garrison the viewer did not see yet too. A change
            // of a Visions holder's reach is not told (stub: TWN-STUB-13).
            GameEvent::GarrisonChanged {
                player: _,
                object_type,
                anchor,
                owner,
                level,
                slots,
                removable,
            } => {
                let view = MapGarrison {
                    object_type,
                    anchor,
                    owner,
                    garrison: Garrison {
                        level,
                        stacks: Vec::new(),
                        army: slots.then(Vec::new),
                    },
                    removable,
                };
                put_garrison(&mut self.garrisons, view);
            }
            GameEvent::GarrisonStackSeen {
                player: _,
                object_type,
                anchor,
                stack,
                slot,
            } => {
                let seen = self
                    .garrisons
                    .iter_mut()
                    .find(|view| (view.object_type, view.anchor) == (object_type, anchor));
                if let Some(view) = seen {
                    view.garrison.see(stack, slot);
                }
            }
            // What a building changes in its town's view comes as the owner's `TownChanged`
            // after it; its price came as `ResourcePaid`.
            GameEvent::TownBuilt { .. } => {}
            // A town event's message is the client's; what it gave came as `ResourceGained` and
            // `ResourceLost` before it, what it built as `TownBuilt` and `TownChanged` after it.
            GameEvent::TownEventFired { .. } => {}
        }
    }

    /// Folds an adventure event.
    fn fold_adventure(&mut self, event: AdventureEvent) {
        match event {
            // A hero coming into sight is not carried (SRV-Q26 (c)): its owner, type and boat
            // (stub: SRV-STUB-3); the mover's movement points come as `MovementChanged`.
            AdventureEvent::HeroStepped { hero, from: _, to }
            | AdventureEvent::HeroTeleported { hero, from: _, to } => match to {
                Some(to) => {
                    if let Some(seen) = self.adventure.hero_mut(hero) {
                        seen.pos = to;
                    }
                }
                // It went out of sight.
                None => self.remove_hero(hero),
            },
            AdventureEvent::ObjectRemoved {
                object_type,
                anchor,
            } => self.adventure.remove_object(ObjectRef {
                object_type,
                anchor,
            }),
            // Its tiles' content, as a step's reveal, comes after the events
            // (`GameObservation::reveal`, SRV-Q26 (a)).
            AdventureEvent::TilesRevealed { .. } => {}
            AdventureEvent::KeyGained { key } => {
                self.adventure.keys |= 1_u8.checked_shl(u32::from(key)).unwrap_or(0);
            }
            AdventureEvent::HeroRemoved { hero, pos: _ } => self.remove_hero(hero),
            // E09 T28a (v64): the owner moves its own view into its garrison heroes, every other
            // viewer sees the hero leave the map.
            AdventureEvent::GarrisonEntered { hero, town, pos: _ } => {
                let at = self
                    .adventure
                    .heroes
                    .iter()
                    .position(|seen| seen.id == hero);
                let own = at.and_then(|at| self.adventure.heroes[at].own.clone().map(|o| (at, o)));
                match own {
                    Some((at, own)) => {
                        self.adventure.heroes.remove(at);
                        let held = &mut self.adventure.garrison_heroes;
                        let place = held.partition_point(|g| g.town < town);
                        held.insert(
                            place,
                            GarrisonHero {
                                town,
                                id: hero,
                                own,
                            },
                        );
                    }
                    None => self.remove_hero(hero),
                }
            }
            // The owner moves its own view back onto the map; a hero coming into sight is not
            // carried for another viewer (SRV-Q26 (c); stub: SRV-STUB-3).
            AdventureEvent::GarrisonLeft { hero, town: _, pos } => {
                let held = &mut self.adventure.garrison_heroes;
                if let Some(at) = held.iter().position(|g| g.id == hero) {
                    let GarrisonHero { own, .. } = held.remove(at);
                    let seen = MapHero {
                        id: hero,
                        owner: self.adventure.viewer,
                        pos,
                        boat: None,
                        own: Some(own),
                    };
                    let heroes = &mut self.adventure.heroes;
                    let place = heroes.partition_point(|h| h.id < hero);
                    heroes.insert(place, seen);
                }
            }
            // Where they join is the rules' (SRV-Q27 (2); stub: SRV-STUB-9).
            AdventureEvent::CreaturesJoined { .. } => {}
            // Messages: nothing the view holds changes. What a visit tells the viewer (the objects
            // it knows, a once-per-hero mark) has no event (SRV-Q26 (d); stub: SRV-STUB-4).
            AdventureEvent::EventFired { .. }
            | AdventureEvent::MessageRead { .. }
            | AdventureEvent::ObjectEmpty { .. }
            | AdventureEvent::ShipyardBlocked { .. }
            | AdventureEvent::NoneToRecruit { .. }
            | AdventureEvent::JoinRefused { .. }
            | AdventureEvent::LeadsNowhere { .. }
            | AdventureEvent::NothingGained { .. } => {}
            // E08 T115b: a kind mark the viewer's team learned (ADV-BON-009).
            AdventureEvent::KindKnown { object_type } => {
                let kinds = &mut self.adventure.kinds;
                if let Err(at) = kinds.binary_search(&object_type) {
                    kinds.insert(at, object_type);
                }
            }
            // E08 T115b: the Stables' upgrade of a stack, its count kept (ADV-BON-005).
            AdventureEvent::CreaturesUpgraded { hero, slot, to, .. } => {
                if let Some(own) = self.own_mut(hero) {
                    own.army
                        .iter_mut()
                        .filter(|stack| stack.slot == slot)
                        .for_each(|stack| stack.creature = to);
                }
            }
            AdventureEvent::MonsterChanged {
                object_type,
                anchor,
                count_word,
            } => {
                let at = ObjectRef {
                    object_type,
                    anchor,
                };
                if let Some(monster) = self
                    .adventure
                    .object_mut(at)
                    .and_then(|object| object.monster.as_mut())
                {
                    monster.count_word = count_word;
                }
            }
            AdventureEvent::Boarded { hero, boat, at } => {
                self.adventure.boats.retain(|empty| empty.tile != at);
                if let Some(seen) = self.adventure.hero_mut(hero) {
                    seen.boat = Some(boat);
                }
            }
            AdventureEvent::Landed {
                hero,
                boat,
                at,
                facing,
            } => {
                self.adventure.add_boat(Boat {
                    tile: at,
                    kind: boat,
                    facing,
                });
                if let Some(seen) = self.adventure.hero_mut(hero) {
                    seen.boat = None;
                }
            }
            // It left with its hero (`HeroRemoved` before).
            AdventureEvent::BoatRemoved { .. } => {}
            // What its new owner's team sees (a mine's production, a Shipyard's boat tile) is not
            // carried (stub: SRV-STUB-10).
            AdventureEvent::ObjectFlagged {
                object_type,
                anchor,
                owner,
            } => {
                let at = ObjectRef {
                    object_type,
                    anchor,
                };
                if let Some(object) = self.adventure.object_mut(at) {
                    object.owner = Some(owner);
                }
            }
            // Its view after the capture comes as each viewer's `TownChanged` (E09 T18b2a).
            AdventureEvent::TownCaptured {
                town,
                anchor: _,
                owner,
            } => {
                if let Some(view) = self.town_mut(town) {
                    view.owner = Some(owner);
                }
                let objects = &mut self.adventure.objects;
                if let Some(object) = objects.iter_mut().find(|o| o.town == Some(town)) {
                    object.owner = Some(owner);
                }
            }
            AdventureEvent::GuardsChanged {
                object_type,
                anchor,
                creature,
                count_word,
            } => {
                let at = ObjectRef {
                    object_type,
                    anchor,
                };
                if let Some(object) = self.adventure.object_mut(at) {
                    object.guards = match (count_word, object.guards.take()) {
                        (None, _) => None,
                        (Some(count_word), held) => Some(Guards {
                            creature,
                            count_word,
                            army: held.and_then(|guards| guards.army),
                        }),
                    };
                }
            }
            AdventureEvent::ObjectMarked {
                object_type,
                anchor,
            } => {
                let at = ObjectRef {
                    object_type,
                    anchor,
                };
                if let Some(object) = self.adventure.object_mut(at) {
                    object.visited = true;
                }
            }
            AdventureEvent::BoatBuilt { tile, boat_type } => self.adventure.add_boat(Boat {
                tile,
                kind: boat_type,
                facing: BUILT_FACING,
            }),
            AdventureEvent::MonsterAdded {
                object_type,
                anchor,
                creature,
                count_word,
            } => self.adventure.add_monster(
                ObjectRef {
                    object_type,
                    anchor,
                },
                Monster {
                    creature,
                    count_word,
                },
            ),
            // The command's last events (SRV-Q26 (b)): the owner's hero's points and maximum.
            AdventureEvent::MovementChanged {
                hero,
                movement,
                max_movement,
            } => {
                if let Some(own) = self.own_mut(hero) {
                    own.movement = movement;
                    own.max_movement = max_movement;
                }
            }
            // Then its battle bonuses and timed marks (v57, E08 T115a): the marks dropped, the
            // sums, the marks gained, each kept sorted.
            AdventureEvent::BonusChanged { hero, morale, luck } => {
                if let Some(own) = self.own_mut(hero) {
                    own.bonuses.morale = morale;
                    own.bonuses.luck = luck;
                }
            }
            AdventureEvent::OnceMarkChanged { hero, bonus, held } => {
                if let Some(own) = self.own_mut(hero) {
                    let once = &mut own.bonuses.once;
                    match (once.binary_search_by_key(&bonus.mark, |b| b.mark), held) {
                        (Err(at), true) => once.insert(at, bonus),
                        (Ok(at), false) => {
                            once.remove(at);
                        }
                        _ => {}
                    }
                }
            }
            AdventureEvent::TimedMarkChanged {
                hero,
                object_type,
                held,
            } => {
                if let Some(own) = self.own_mut(hero) {
                    let timed = &mut own.timed;
                    match (timed.binary_search(&object_type), held) {
                        (Err(at), true) => timed.insert(at, object_type),
                        (Ok(at), false) => {
                            timed.remove(at);
                        }
                        _ => {}
                    }
                }
            }
            // E08 T115b: the Sirens' loss as the Whirlpool's (ADV-BON-008; a stack never empties).
            AdventureEvent::WhirlpoolLoss {
                hero,
                slot,
                creature: _,
                count,
            }
            | AdventureEvent::SirensLoss {
                hero,
                slot,
                creature: _,
                count,
            } => {
                if let Some(own) = self.own_mut(hero) {
                    let left = own
                        .army
                        .iter()
                        .find(|stack| stack.slot == slot)
                        .map(|stack| stack.count.saturating_sub(count));
                    match left {
                        Some(0) => own.army.retain(|stack| stack.slot != slot),
                        Some(left) => own
                            .army
                            .iter_mut()
                            .filter(|stack| stack.slot == slot)
                            .for_each(|stack| stack.count = left),
                        None => {}
                    }
                }
            }
        }
    }

    /// Folds what the viewer newly sees (E16 M5 T5c; SRV-Q26 (a)), after the events of the
    /// command that explored tiles for it: the tiles whose sight changed with their views, the
    /// heroes with their types, the objects and empty boats it newly sees and its views of the
    /// towns it newly explored, each put in its place in the view or replacing the one there.
    pub fn reveal(&mut self, reveal: &GameReveal) {
        let GameReveal {
            adventure,
            heroes,
            towns,
            garrisons,
        } = reveal;
        self.adventure.reveal(adventure);
        for info in heroes {
            put(&mut self.heroes, info.clone(), |info| info.id);
        }
        for view in towns {
            put(&mut self.towns, view.clone(), |view| view.town);
        }
        for view in garrisons {
            put_garrison(&mut self.garrisons, view.clone());
        }
    }

    /// Whether `player`'s turn is running.
    fn acting(&self, player: u8) -> bool {
        self.players
            .get(usize::from(player))
            .is_some_and(|view| view.acting)
    }

    /// Sets whether `player`'s turn is running.
    fn set_acting(&mut self, player: u8, acting: bool) {
        if let Some(view) = self.players.get_mut(usize::from(player)) {
            view.acting = acting;
        }
    }

    /// A player's turn started: the first of its group's ends the group before (each of whose
    /// players had ended its turn).
    fn start_turn(&mut self, player: u8) {
        if !self.turn_group.iter().any(|&p| self.acting(p)) {
            self.turn_group.clear();
        }
        if !self.turn_group.contains(&player) {
            self.turn_group.push(player);
        }
        self.set_acting(player, true);
    }

    /// A hero left the viewer's sight or the map.
    fn remove_hero(&mut self, hero: u16) {
        self.adventure.heroes.retain(|seen| seen.id != hero);
        // E09 T28a, T28b2: a garrison hero leaves the game at its town's capture (`TWN-QUIRK-11`)
        // or by its siege.
        self.adventure
            .garrison_heroes
            .retain(|held| held.id != hero);
        self.heroes.retain(|info| info.id != hero);
    }

    /// A hired hero came onto the map (E09 T38a1, TWN-TAV-003): `seen` with its type among the
    /// heroes, in id order; with `view`, the owner's tavern slot's view it was hired from, the
    /// owner's own view of it (its army, formation and tactics flag, its details but what its
    /// [`HiredPart`]s tell, its points none until told).
    fn hire(&mut self, mut seen: MapHero, hero_type: u16, view: Option<TavernHero>) {
        let id = seen.id;
        let details = view.map(|view| {
            let TavernHero {
                hero,
                army,
                skills,
                artifacts,
            } = view;
            seen.own = Some(OwnMapHero {
                army,
                movement: 0,
                max_movement: 0,
                formation: hero.formation,
                tactics: hero.tactics,
                marks: Vec::new(),
                bonuses: Bonuses {
                    morale: 0,
                    luck: 0,
                    once: Vec::new(),
                },
                timed: Vec::new(),
            });
            let mut details = HeroDetails {
                hero_type,
                level: hero.level,
                experience: hero.experience,
                offered: OfferLevels {
                    wisdom: 0,
                    school: 0,
                },
                primary: hero.primary,
                skills,
                worn: Vec::new(),
                backpack: Vec::new(),
                spells: Vec::new(),
                mana: hero.mana,
            };
            // The worn ones by position (the positions after the last told by its parts), then
            // the backpack's first places in order.
            for (place, artifact) in artifacts {
                put_artifact(&mut details, place, artifact);
            }
            details
        });
        let heroes = &mut self.adventure.heroes;
        let at = heroes.partition_point(|h| h.id < id);
        heroes.insert(at, seen);
        let at = self.heroes.partition_point(|h| h.id < id);
        self.heroes.insert(
            at,
            HeroInfo {
                id,
                hero_type,
                details,
            },
        );
    }

    /// The details of the viewer's hero `hero`.
    /// Adds a part of a freed prisoner's view after the ones before it (`PrisonerPartSeen`, E08
    /// T141a): a stack to its owner's army, a skill to its details' skills, an artifact at its
    /// place as a hire's view's ([`Self::hire`]).
    fn see_freed(&mut self, hero: u16, part: TavernPart) {
        match part {
            TavernPart::Stack { stack } => {
                if let Some(own) = self.own_mut(hero) {
                    own.army.push(stack);
                }
            }
            TavernPart::Skill { skill } => {
                if let Some(details) = self.details_mut(hero) {
                    details.skills.push(skill);
                }
            }
            TavernPart::Artifact { place, artifact } => {
                if let Some(details) = self.details_mut(hero) {
                    put_artifact(details, place, artifact);
                }
            }
        }
    }

    fn details_mut(&mut self, hero: u16) -> Option<&mut HeroDetails> {
        self.heroes
            .iter_mut()
            .find(|info| info.id == hero)
            .and_then(|info| info.details.as_mut())
    }

    /// What the viewer sees of its hero `hero` on the map beyond the others' view, since E09 T28a
    /// or in a town's garrison.
    fn own_mut(&mut self, hero: u16) -> Option<&mut OwnMapHero> {
        let held = self.adventure.garrison_heroes.iter().any(|g| g.id == hero);
        if held {
            return self
                .adventure
                .garrison_heroes
                .iter_mut()
                .find(|g| g.id == hero)
                .map(|g| &mut g.own);
        }
        self.adventure
            .hero_mut(hero)
            .and_then(|seen| seen.own.as_mut())
    }

    /// The view of the town `town`.
    fn town_mut(&mut self, town: u16) -> Option<&mut Town> {
        self.towns.iter_mut().find(|view| view.town == town)
    }

    /// A stack of an army dialog's group at its opening: the first opens it, for its hero
    /// (after a Leave Guards dialog's opening, its guards).
    fn open_group(&mut self, hero: u16, stack: ArmyStack) {
        match &mut self.decision {
            Some(Decision::ArmyDialog {
                hero: open,
                group,
                mine: _,
            }) if *open == hero => set_slot(group, stack.slot, Some(stack)),
            _ => {
                self.decision = Some(Decision::ArmyDialog {
                    hero,
                    group: alloc::vec![stack],
                    mine: None,
                });
            }
        }
    }

    /// The viewer's decision closed: a Leave Guards dialog's group stays on its mine as its
    /// guards (GAM-DEC-004 amended, ADV-MIN-002), which the owner's team sees by slot; their hint
    /// is a `GuardsChanged`'s, before or after.
    fn close_decision(&mut self) {
        let Some(Decision::ArmyDialog {
            hero: _,
            group,
            mine: Some(mine),
        }) = self.decision.take()
        else {
            return;
        };
        if let Some(object) = self.adventure.object_mut(mine) {
            match &mut object.guards {
                Some(guards) => guards.army = Some(group),
                None if !group.is_empty() => {
                    object.guards = Some(Guards {
                        creature: None,
                        count_word: 0,
                        army: Some(group),
                    });
                }
                None => {}
            }
        }
    }

    /// A slot of an army changed.
    fn set_stack(&mut self, at: ArmySlot, stack: Option<ArmyStack>) {
        let ArmySlot { army, slot } = at;
        let held = match army {
            ArmyRef::Hero { hero } => self.own_mut(hero).map(|own| &mut own.army),
            ArmyRef::Group {} => match &mut self.decision {
                Some(Decision::ArmyDialog { group, .. }) => Some(group),
                _ => None,
            },
            ArmyRef::Garrison { town } => self
                .town_mut(town)
                .and_then(|view| view.garrison.army.as_mut()),
            // E08 T114a2: the open exchange's garrison (its stacks at the opening, then its
            // commands' changes); every viewer's view of it (`garrisons`) changes by its
            // `GarrisonChanged` (E08 T114b).
            ArmyRef::MapGarrison { garrison } => match &mut self.decision {
                Some(Decision::Exchange {
                    garrison: open,
                    army,
                    ..
                }) if *open == garrison => Some(army),
                _ => None,
            },
        };
        if let Some(held) = held {
            set_slot(held, slot, stack);
        }
    }
}

/// Sets an army's slot: its stacks stay in slot order.
fn set_slot(army: &mut Vec<ArmyStack>, slot: u8, stack: Option<ArmyStack>) {
    army.retain(|held| held.slot != slot);
    if let Some(stack) = stack {
        let at = army.partition_point(|held| held.slot < slot);
        army.insert(at, ArmyStack { slot, ..stack });
    }
}

/// Puts `item` into `list`, ordered by `key`: in its place, or in place of the one with its key.
fn put<T>(list: &mut Vec<T>, item: T, key: impl Fn(&T) -> u16) {
    match list.binary_search_by_key(&key(&item), &key) {
        Ok(at) => list[at] = item,
        Err(at) => list.insert(at, item),
    }
}

/// Puts the map garrison `view` in its place among `list`, in their places' order
/// ([`MapGarrison::place`]), replacing the one there.
fn put_garrison(list: &mut Vec<MapGarrison>, view: MapGarrison) {
    match list.binary_search_by_key(&view.place(), MapGarrison::place) {
        Ok(at) => list[at] = view,
        Err(at) => list.insert(at, view),
    }
}

/// Puts a view's artifact at its place in `details` (a hire's, E09 T38a1; a freed prisoner's, E08
/// T141a): a worn one at its position (the worn list grown to it), a backpack's after the ones
/// before it.
fn put_artifact(details: &mut HeroDetails, place: ArtifactPlace, artifact: Artifact) {
    match place {
        ArtifactPlace::Worn { position } => {
            let at = usize::from(position);
            let len = details.worn.len().max(at + 1);
            details.worn.resize(len, None);
            details.worn[at] = Some(artifact);
        }
        ArtifactPlace::Backpack { index: _ } => details.backpack.push(artifact),
    }
}

impl TavernHero {
    /// Adds a part of its view after the ones before it (`TavernPartSeen`, E09 T37a): to the list
    /// of its kind.
    pub(crate) fn see(&mut self, part: TavernPart) {
        match part {
            TavernPart::Stack { stack } => self.army.push(stack),
            TavernPart::Skill { skill } => self.skills.push(skill),
            TavernPart::Artifact { place, artifact } => self.artifacts.push((place, artifact)),
        }
    }
}

impl Garrison {
    /// Adds a stack as the viewer's level shows it, after the ones before it (a town's
    /// `TownStackSeen`, a map garrison's `GarrisonStackSeen`): to the garrison by slot too, with
    /// its slot and count, for a viewer that sees it so.
    fn see(&mut self, stack: GarrisonStack, slot: Option<u8>) {
        self.stacks.push(stack);
        if let (Some(army), Some(slot)) = (self.army.as_mut(), slot) {
            army.push(ArmyStack {
                slot,
                creature: stack.creature,
                count: stack.count.unwrap_or_default(),
            });
        }
    }
}

impl Adventure {
    /// Folds the adventure map's part of a reveal ([`GameObservation::reveal`]).
    fn reveal(&mut self, reveal: &AdventureReveal) {
        let AdventureReveal {
            // The view's own viewer.
            viewer: _,
            palette,
            areas,
            heroes,
            objects,
            boats,
            // Their views are the game's part's.
            towns: _,
        } = reveal;
        if !areas.is_empty() {
            let mut views: Vec<Tile> = (0..self.tiles.len())
                .map(|i| self.tile(i).copied().unwrap_or(Tile::Hidden {}))
                .collect();
            for (pos, &RevealedView { explored, view }) in revealed(palette, areas) {
                // A tile off the map is no tile of the view.
                let Some(held) = self.index(pos).and_then(|i| views.get_mut(i)) else {
                    continue;
                };
                *held = view;
                let fringe = !explored && !matches!(view, Tile::Hidden {});
                // In 64 bits: a hostile view's sides overflow a 32-bit `usize` (a wasm client).
                let row = u64::from(pos.level) * u64::from(self.height) + u64::from(pos.y);
                let word = row * u64::from(self.stride) + u64::from(pos.x / 64);
                let word = usize::try_from(word).ok();
                let bit = 1_u64 << (pos.x % 64);
                for (plane, on) in [(&mut self.explored, explored), (&mut self.fringe, fringe)] {
                    let held = word.and_then(|word| plane.get_mut(word));
                    held.into_iter().for_each(|WireU64(w)| {
                        *w = if on { *w | bit } else { *w & !bit };
                    });
                }
            }
            (self.palette, self.tiles) = pack(&views);
        }
        for hero in heroes {
            put(&mut self.heroes, hero.clone(), |hero| hero.id);
        }
        // An object the events told already is replaced by its view now.
        self.objects
            .retain(|held| !objects.iter().any(|o| o.place() == held.place()));
        for object in objects {
            self.add_object(object.clone());
        }
        for &boat in boats {
            self.add_boat(boat);
        }
    }

    /// An object came into view, in its place ([`Object::place`]).
    fn add_object(&mut self, object: Object) {
        let at = self
            .objects
            .partition_point(|held| held.place() <= object.place());
        self.objects.insert(at, object);
    }

    /// The hero `hero` as the viewer sees it.
    fn hero_mut(&mut self, hero: u16) -> Option<&mut MapHero> {
        self.heroes.iter_mut().find(|seen| seen.id == hero)
    }

    /// The object `at` as the viewer sees it.
    fn object_mut(&mut self, at: ObjectRef) -> Option<&mut Object> {
        self.objects.iter_mut().find(|object| is(object, at))
    }

    /// An empty boat came into view, in tile order.
    fn add_boat(&mut self, boat: Boat) {
        let key = |b: &Boat| (b.tile.level, b.tile.y, b.tile.x);
        self.boats.retain(|empty| empty.tile != boat.tile);
        let at = self.boats.partition_point(|empty| key(empty) < key(&boat));
        self.boats.insert(at, boat);
    }

    /// An object left the map (ADV-OBJ-005): what the viewer knew of it goes, and the seen tiles
    /// it covered that no other object it sees covers become free (an object covering a seen tile
    /// is seen, ADV-FOG-004).
    fn remove_object(&mut self, at: ObjectRef) {
        let Some(index) = self.objects.iter().position(|object| is(object, at)) else {
            return;
        };
        let removed = self.objects.remove(index);
        self.known
            .retain(|known| (known.object_type, known.anchor) != (at.object_type, at.anchor));
        self.retile(
            removed.anchor,
            removed.blocked.0 | removed.visitable.0,
            |objects, pos, blocked, visitable| {
                *blocked &= objects.iter().any(|o| covers(o.anchor, o.blocked.0, pos));
                *visitable &= objects.iter().any(|o| covers(o.anchor, o.visitable.0, pos));
            },
        );
    }

    /// A creature month's monster came onto the map (ADV-MON-014): an object in its place, its
    /// tile blocked and visited from.
    fn add_monster(&mut self, at: ObjectRef, monster: Monster) {
        self.add_object(Object {
            object_type: at.object_type,
            anchor: at.anchor,
            blocked: WireU64(ONE_TILE),
            visitable: WireU64(ONE_TILE),
            drawn: WireU64(0),
            monster: Some(monster),
            owner: None,
            production: None,
            guards: None,
            visited: false,
            boat_tile: None,
            town: None,
        });
        self.retile(at.anchor, ONE_TILE, |_, _, blocked, visitable| {
            *blocked = true;
            *visitable = true;
        });
    }

    /// Changes the objects' flags of the seen tiles of `mask` anchored at `anchor` by `change`,
    /// then packs the palette anew, in the order the tiles first use its views.
    fn retile(
        &mut self,
        anchor: Pos,
        mask: u64,
        change: impl Fn(&[Object], Pos, &mut bool, &mut bool),
    ) {
        let mut tiles: Vec<Tile> = (0..self.tiles.len())
            .map(|i| self.tile(i).copied().unwrap_or(Tile::Hidden {}))
            .collect();
        for bit in (0..MASK_COLUMNS * MASK_ROWS).filter(|bit| mask >> bit & 1 == 1) {
            // A tile left of the map's first column or above its first row wraps beyond its
            // last: off the map.
            let pos = Pos {
                x: anchor.x.wrapping_sub(bit % MASK_COLUMNS),
                y: anchor.y.wrapping_sub(bit / MASK_COLUMNS),
                level: anchor.level,
            };
            if let Some(Tile::Seen {
                blocked, visitable, ..
            }) = self.index(pos).and_then(|i| tiles.get_mut(i))
            {
                change(&self.objects, pos, blocked, visitable);
            }
        }
        (self.palette, self.tiles) = pack(&tiles);
    }

    /// The index of the tile `pos` in tile order; none off the map, or beyond a `usize`.
    fn index(&self, pos: Pos) -> Option<usize> {
        (pos.x < self.width && pos.y < self.height && pos.level < self.levels)
            .then(|| {
                // In 64 bits, as the planes' words (`reveal`).
                (u64::from(pos.level) * u64::from(self.height) + u64::from(pos.y))
                    * u64::from(self.width)
                    + u64::from(pos.x)
            })
            .and_then(|i| usize::try_from(i).ok())
    }
}

/// The tiles of a reveal's areas whose sight changed, each with its view: an index beyond the
/// palette names none, a tile beyond the sides a position can hold is none.
fn revealed<'a>(
    palette: &'a [RevealedView],
    areas: &'a [RevealedArea],
) -> impl Iterator<Item = (Pos, &'a RevealedView)> + 'a {
    areas
        .iter()
        .filter(|area| area.width > 0)
        .flat_map(move |area| {
            let width = usize::from(area.width);
            area.tiles.iter().enumerate().filter_map(move |(i, &n)| {
                let at = usize::try_from(n).ok().and_then(|n| n.checked_sub(1));
                let view = at.and_then(|at| palette.get(at))?;
                let x = u16::try_from(usize::from(area.corner.x) + i % width).ok()?;
                let y = u16::try_from(usize::from(area.corner.y) + i / width).ok()?;
                let level = area.corner.level;
                Some((Pos { x, y, level }, view))
            })
        })
}

/// Whether `object` is the one named `at`.
fn is(object: &Object, at: ObjectRef) -> bool {
    object.object_type == at.object_type && object.anchor == at.anchor
}

/// Whether the mask `mask` anchored at `anchor` covers the tile `pos` ([`Object::blocked`]'s
/// layout: bit `row × 8 + column` the tile `column` columns left of and `row` rows above the
/// anchor).
fn covers(anchor: Pos, mask: u64, pos: Pos) -> bool {
    let (column, row) = (anchor.x.wrapping_sub(pos.x), anchor.y.wrapping_sub(pos.y));
    pos.level == anchor.level
        && column < MASK_COLUMNS
        && row < MASK_ROWS
        && mask >> (row * MASK_COLUMNS + column) & 1 == 1
}

/// The tiles' views as a palette in first-use order and an index per tile ([`Adventure::tiles`]).
fn pack(tiles: &[Tile]) -> (Vec<Tile>, Vec<u32>) {
    let mut index = BTreeMap::new();
    let mut palette = Vec::new();
    let packed = tiles
        .iter()
        .map(|&view| {
            *index.entry(view).or_insert_with(|| {
                // Below the tile count, which a map's `u16` sides and `u8` levels bound.
                let next = u32::try_from(palette.len()).unwrap_or(u32::MAX);
                palette.push(view);
                next
            })
        })
        .collect();
    (palette, packed)
}

impl HeroDetails {
    /// Folds a change of the hero's details, as `h3_heroes` applies it but for what the rules
    /// derive (SRV-Q27 (2)): where an added artifact lands, a gain's cap (the experience's level
    /// limit, the primary skills' most), a placed Titan's Thunder's Spell Book.
    fn fold(&mut self, change: HeroChange) {
        match change {
            HeroChange::GainExperience { amount } => {
                self.experience = self.experience.saturating_add(amount);
            }
            HeroChange::SetSpellPoints { points } => self.mana = points,
            // Where it lands is the rules' (stub: SRV-STUB-7).
            HeroChange::AddArtifact { artifact: _ } => {}
            HeroChange::RemoveArtifact { place } => self.remove_artifact(place),
            HeroChange::PlaceArtifact { artifact, at } => self.place_artifact(artifact, at),
            HeroChange::LearnSpell { spell } => self.spells.push(spell),
            HeroChange::GainLevel {
                primary,
                learned,
                offered,
            } => {
                let level = self.level.saturating_add(1);
                self.gain_level(level, primary, learned);
                self.offered = offered;
            }
            HeroChange::GainPrimary { primary } => {
                for (base, gain) in self.primary.iter_mut().zip(primary) {
                    *base = base.saturating_add(gain);
                }
            }
            HeroChange::RaiseSkill { skill, level } => {
                match self.skills.iter_mut().find(|known| known.skill == skill) {
                    Some(known) => known.level = known.level.max(level),
                    None => self.skills.push(SkillLevel { skill, level }),
                }
            }
        }
    }

    /// The hero gained its level `level`: one more of `primary`, the skill learned (a known one a
    /// level up, a new one at Basic after the others). The levels at which Wisdom and a school
    /// were last offered are a [`HeroChange::GainLevel`]'s only.
    fn gain_level(&mut self, level: u8, primary: PrimarySkill, skill: Option<u16>) {
        self.level = level;
        let [attack, defense, power, knowledge] = &mut self.primary;
        let base = match primary {
            PrimarySkill::Attack => attack,
            PrimarySkill::Defense => defense,
            PrimarySkill::Power => power,
            PrimarySkill::Knowledge => knowledge,
        };
        *base = base.saturating_add(1);
        if let Some(skill) = skill {
            match self.skills.iter_mut().find(|known| known.skill == skill) {
                Some(known) => known.level = known.level.saturating_add(1),
                None => self.skills.push(SkillLevel { skill, level: 1 }),
            }
        }
    }

    /// The artifact at `place` left: a position is left empty, the backpack's later artifacts
    /// move up.
    fn remove_artifact(&mut self, place: ArtifactPlace) {
        match place {
            ArtifactPlace::Worn { position } => {
                if let Some(worn) = self.worn.get_mut(usize::from(position)) {
                    *worn = None;
                }
            }
            ArtifactPlace::Backpack { index } => {
                let index = usize::from(index);
                if index < self.backpack.len() {
                    self.backpack.remove(index);
                }
            }
        }
    }

    /// A part of a hired hero's details its tavern slot's view did not show (`HiredPartSeen`, E09
    /// T38a1), after the ones before it: the offered levels and the worn positions' count (the
    /// positions the view named kept), a backpack's artifact after the others, a learned spell.
    fn see(&mut self, part: HiredPart) {
        match part {
            HiredPart::Details { offered, positions } => {
                self.offered = offered;
                self.worn.resize(usize::from(positions), None);
            }
            HiredPart::Carried { artifact } => self.backpack.push(artifact),
            HiredPart::Spell { spell } => self.spells.push(spell),
        }
    }

    /// `artifact` was put at `place`: worn there, or inserted into the backpack there.
    fn place_artifact(&mut self, artifact: Artifact, place: ArtifactPlace) {
        match place {
            ArtifactPlace::Worn { position } => {
                if let Some(worn) = self.worn.get_mut(usize::from(position)) {
                    *worn = Some(artifact);
                }
            }
            ArtifactPlace::Backpack { index } => {
                let index = usize::from(index).min(self.backpack.len());
                self.backpack.insert(index, artifact);
            }
        }
    }
}
