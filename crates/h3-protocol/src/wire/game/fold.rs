//! The fold of the whole game's events (E16 M5 T3c1; SRV-Q26, SRV-Q27): a viewer's
//! [`GameObservation`] kept current by its projected events ([`GameObservation::fold`]), as a
//! client and a record's reader keep it between snapshots (`docs/epics/E16-design.md` §14: events
//! only).
//!
//! A fold applies what an event carries and nothing the rules derive (SRV-Q27 (2)). What no event
//! carries yet stays as it was, each a gap for the adventure and town lanes (SRV-Q26, SRV-Q27):
//! a reveal's tiles, objects, heroes and towns; a hero's movement points; a hero, object or boat
//! coming into sight without a reveal; two changes told to nobody who sees them; the views of an
//! offer, a choice, a University and a hire dialog; and the values the rules derive (where an
//! added artifact or joined creatures land, a capped gain, what a new owner's team sees). The
//! battle is its snapshots' (SRV-Q27 (1)): its events are the battle client's, and the fold only
//! ends it.
//!
//! Provisional (E16 M5 part 1's review): the fold's place, what it derives from an event.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::{
    Adventure, AdventureEvent, ArmyRef, ArmySlot, ArmyStack, Artifact, ArtifactPlace, Boat,
    Bonuses, Decision, GameEvent, GameObservation, Guards, HeroChange, HeroDetails, LevelUp,
    MapHero, Monster, Object, ObjectRef, OwnMapHero, Pos, PrimarySkill, SkillLevel, Tile, Town,
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
    /// they were.
    pub fn fold(&mut self, event: &GameEvent) {
        match *event {
            GameEvent::TurnEnded { player } => self.set_acting(player, false),
            GameEvent::DayStarted { day } => {
                self.day = day;
                // Today's timed events fire at the viewer's turn start (GAM-CAL-004).
                self.events_today.clear();
            }
            // No part of the view holds the week.
            GameEvent::WeekStarted { week: _ } => {}
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
            // options, a University's skills and slots, a hire dialog's creatures and costs.
            GameEvent::OfferOpened { .. }
            | GameEvent::ChoiceOpened { .. }
            | GameEvent::UniversityOpened { .. }
            | GameEvent::HireOpened { .. } => {}
            // A message: the treasury could not pay.
            GameEvent::OfferRefused { .. } => {}
            GameEvent::OfferAnswered { .. } => self.decision = None,
            // The battle's snapshot follows its start; its events are the battle client's
            // (SRV-Q27 (1)).
            GameEvent::BattleStarted { .. } | GameEvent::Battle { .. } => {}
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
            // Where they join is the rules' (SRV-Q27 (2)).
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
            GameEvent::DecisionClosed { player: _ } => self.close_decision(),
            // Told after its dialog closed, with its group.
            GameEvent::GroupLost { .. } => {}
            GameEvent::TownChanged { town, capitol } => {
                if let Some(town) = self.town_mut(town) {
                    town.capitol = capitol;
                }
            }
            // What a building changes in its town's view (its fort, its hall and income for the
            // owner's team) is the rules' (SRV-Q27 (2)); its price came as `ResourcePaid`.
            GameEvent::TownBuilt { .. } => {}
        }
    }

    /// Folds an adventure event.
    fn fold_adventure(&mut self, event: AdventureEvent) {
        match event {
            // A hero coming into sight is not carried (SRV-Q26 (c)): its owner, type and boat.
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
            // Its tiles' content is not carried (SRV-Q26 (a)).
            AdventureEvent::TilesRevealed { .. } => {}
            AdventureEvent::KeyGained { key } => {
                self.adventure.keys |= 1_u8.checked_shl(u32::from(key)).unwrap_or(0);
            }
            AdventureEvent::HeroRemoved { hero, pos: _ } => self.remove_hero(hero),
            // Where they join is the rules' (SRV-Q27 (2)).
            AdventureEvent::CreaturesJoined { .. } => {}
            // Messages: nothing the view holds changes.
            AdventureEvent::EventFired { .. }
            | AdventureEvent::MessageRead { .. }
            | AdventureEvent::ObjectEmpty { .. }
            | AdventureEvent::ShipyardBlocked { .. }
            | AdventureEvent::NoneToRecruit { .. }
            | AdventureEvent::JoinRefused { .. }
            | AdventureEvent::LeadsNowhere { .. } => {}
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
            AdventureEvent::WhirlpoolLoss {
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
        self.heroes.retain(|info| info.id != hero);
    }

    /// The details of the viewer's hero `hero`.
    fn details_mut(&mut self, hero: u16) -> Option<&mut HeroDetails> {
        self.heroes
            .iter_mut()
            .find(|info| info.id == hero)
            .and_then(|info| info.details.as_mut())
    }

    /// What the viewer sees of its hero `hero` on the map beyond the others' view.
    fn own_mut(&mut self, hero: u16) -> Option<&mut OwnMapHero> {
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

impl Adventure {
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

    /// A creature month's monster came onto the map (ADV-MON-014): an object added after every
    /// other, its tile blocked and visited from.
    fn add_monster(&mut self, at: ObjectRef, monster: Monster) {
        self.objects.push(Object {
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

    /// The index of the tile `pos` in tile order; none off the map.
    fn index(&self, pos: Pos) -> Option<usize> {
        (pos.x < self.width && pos.y < self.height && pos.level < self.levels).then(|| {
            (usize::from(pos.level) * usize::from(self.height) + usize::from(pos.y))
                * usize::from(self.width)
                + usize::from(pos.x)
        })
    }
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
            HeroChange::Bonus { morale, luck, mark } => {
                let bit = mark.map_or(0, |mark| 1_u8.checked_shl(u32::from(mark)).unwrap_or(0));
                // A once-mark set already gates the bonus (HER-ADV-008).
                if self.bonuses.marks & bit == 0 {
                    let bonuses = &mut self.bonuses;
                    bonuses.morale = bonuses.morale.saturating_add(morale);
                    bonuses.luck = bonuses.luck.saturating_add(luck);
                    bonuses.marks |= bit;
                }
            }
            HeroChange::ClearBonuses {} => {
                self.bonuses = Bonuses {
                    morale: 0,
                    luck: 0,
                    marks: 0,
                };
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
