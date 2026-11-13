//! The bonus visitables' visits (E08 T115b; increment 12's design note §8, §9; survey
//! `E08-garrisons-bonuses.md` §10; ADV-BON-002 to -009): the ten `bonus` kinds' battle bonus and
//! movement, the Fountain of Fortune's luck, the Magic Well's and the Magic Spring's spell points,
//! the Stables' movement and upgrade, the Sirens' losses; the kind marks the team learns; the
//! Fountains' and the Springs' blocks in the hash. Each visit reads the hero's marks first (the
//! hero on the map's, T115a's [`super::bonuses`]) and, when its mark is held, gives nothing and
//! tells so ([`AdventureEventKind::NothingGained`]); what it gives is told by the command's
//! diff of the hero's movement and marks ([`crate::OwnHeroes::tell`]) and by the game's change
//! of its details (the spell points, the experience).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, Ruleset, SIRENS_AGAIN_AFTER_BATTLE};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroChange, HeroDetails, VisitBonus, max_mana};
use h3_kernel::num::Real;
use h3_kernel::{CanonicalHash, StateHasher};

use super::AdventureState;
use super::economy::allied;
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, Drowned, HeroGains, NothingWhy, Sink,
    Viewers,
};
use crate::movement::details_of;
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::Pos;

type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

/// The object a visit concerns: its index, its type and its anchor.
pub(super) type Visited = (usize, ObjectTypeId, Pos);

impl AdventureState {
    /// Tells `player` that its hero `hero`'s visit of `object` gave nothing, and `why` (the
    /// original's "visited", "full" and "emptied" messages): the owner alone.
    fn nothing<S: Sink>(
        (player, hero): (PlayerId, HeroId),
        (_, object_type, anchor): Visited,
        why: NothingWhy,
        sink: &mut S,
    ) {
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::NothingGained {
                hero,
                object_type,
                anchor,
                why,
            },
            seen_by: Viewers::NONE.with(player),
        });
    }

    /// Whether the hero on the map `hero` holds the timed mark of `object_type` (the Magic Well's
    /// day mark, the Stables' week mark).
    fn holds_timed(&self, hero: HeroId, object_type: ObjectTypeId) -> bool {
        self.visit_marks(hero)
            .timed()
            .binary_search(&object_type)
            .is_ok()
    }

    /// The kind mark of `object_type` for `player`'s team (ADV-BON-009; increment 12's design note
    /// §3, decision 302: the original's `kinds_visited`, game +0x4E344 + k, a bit for every player
    /// of the visitor's team, `player_team`), for a type the part marks
    /// ([`crate::ObjectType::kind_mark`]): each player of the team that did not hold it gains it,
    /// told to each of them ([`AdventureEventKind::KindKnown`]). The team is the visitor alone
    /// until E10's teams ([`allied`]). Nothing for a type without the flag, nor when the team
    /// holds it.
    // rule: ADV-BON-009
    pub(super) fn mark_kind<S: Sink>(
        &mut self,
        part: &AdventureRuleset,
        player: PlayerId,
        object_type: ObjectTypeId,
        sink: &mut S,
    ) {
        if !part.object_type(object_type).is_some_and(|ty| ty.kind_mark) {
            return;
        }
        let team = (0..self.players)
            .map(PlayerId::new)
            .filter(|&p| allied(p, player) && !self.knows_kind(p, object_type))
            .fold(Viewers::NONE, Viewers::with);
        if team == Viewers::NONE {
            return;
        }
        let flags = Arc::make_mut(self.flags.get_or_insert_with(Arc::default));
        flags.mark_kind(team.bits(), object_type);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::KindKnown { object_type },
            seen_by: team,
        });
    }

    /// Whether `player`'s team holds the kind mark of `object_type` (ADV-BON-009; the map's hint
    /// reads it).
    pub fn knows_kind(&self, player: PlayerId, object_type: ObjectTypeId) -> bool {
        self.flags
            .as_deref()
            .is_some_and(|flags| flags.knows_kind(player, object_type))
    }

    /// The object types whose kind mark `player`'s team holds, in type order (ADV-BON-009, the
    /// observation's list).
    pub fn kinds_known(&self, player: PlayerId) -> Vec<ObjectTypeId> {
        self.flags
            .as_deref()
            .map_or_else(Vec::new, |flags| flags.kinds(player).collect())
    }

    /// A `bonus` kind's visit by `player`'s hero `hero` (ADV-BON-002, ADV-BON-004; increment 12's
    /// design note §8.1; survey §10.2): while the hero does not hold `mark`, the day's pair
    /// `(morale, luck)` under it (the hero on the map's once-marks with their amounts, T115a), then
    /// `points` added to the points left and the day's maximum, saturating (the original's 32 bits
    /// unbounded), or with `drain` the points left set to 0 and the maximum kept (the Swan Pond),
    /// then the kind mark. Holding the mark: nothing, told. The movement is told by the command's
    /// [`AdventureEventKind::MovementChanged`], the mark by its `OnceMarkChanged`.
    // rule: ADV-BON-002, ADV-BON-004
    pub(super) fn visit_bonus<S: Sink>(
        &mut self,
        part: &AdventureRuleset,
        (player, hero): (PlayerId, HeroId),
        object: Visited,
        (mark, (morale, luck)): (u8, (i8, i8)),
        (points, drain): (u32, bool),
        sink: &mut S,
    ) {
        if !self.give_once(hero, VisitBonus { mark, morale, luck }) {
            return Self::nothing((player, hero), object, NothingWhy::Visited, sink);
        }
        if let Some(on_map) = self.hero_mut(hero) {
            if drain {
                on_map.movement = 0;
            } else {
                on_map.movement = on_map.movement.saturating_add(points);
                on_map.max_movement = on_map.max_movement.saturating_add(points);
            }
        }
        self.mark_kind(part, player, object.1, sink);
    }

    /// A Fountain of Fortune's visit by `player`'s hero `hero` (ADV-BON-003; increment 12's design
    /// note §8.2; survey §10.2, §10.6): first, on every visit, the fountain's knowledge for the
    /// visitor's team (ADV-VIS-006: the original's tile team bits, which the hint reads; the
    /// visitor alone until E10), then, while the hero does not hold `mark`, the fountain's value
    /// as luck under it (drawn when the game was built and at each week's start). Holding the
    /// mark: nothing, told. The value is never shown; its holder learns it as its hero's bonus.
    // rule: ADV-BON-003, ADV-VIS-006
    pub(super) fn visit_fortune<S: Sink>(
        &mut self,
        (player, hero): (PlayerId, HeroId),
        object: Visited,
        mark: u8,
        sink: &mut S,
    ) {
        self.learn(player, object.0);
        let luck = self
            .fortunes
            .as_deref()
            .zip(self.map.fountain_index(object.0))
            .and_then(|(values, place)| values.get(place).copied())
            .unwrap_or(0);
        let bonus = VisitBonus {
            mark,
            morale: 0,
            luck,
        };
        if !self.give_once(hero, bonus) {
            Self::nothing((player, hero), object, NothingWhy::Visited, sink);
        }
    }

    /// A Magic Well's visit by `player`'s hero `hero` (ADV-BON-006; increment 12's design note
    /// §8.3; survey §10.2, §10.4): holding the Well's day mark (its type: any Well of the type),
    /// nothing, told. Else the kind mark, then with spell points below the hero's maximum
    /// (`h3_heroes::max_mana`, HER-MAN-001) the points set to it (the outcome's change, the game's)
    /// and the day mark; at or above it nothing, told, **and no mark**, so the hero may drink
    /// again that day after casting (the original's message 79).
    // rule: ADV-BON-006
    pub(super) fn visit_well<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        object: Visited,
        sink: &mut S,
    ) -> AdventureOutcome {
        if self.holds_timed(hero, object.1) {
            Self::nothing((player, hero), object, NothingWhy::Visited, sink);
            return AdventureOutcome::default();
        }
        self.mark_kind(rules.1, player, object.1, sink);
        let (mana, most) = spell_points(rules, hero);
        if mana >= most {
            Self::nothing((player, hero), object, NothingWhy::Full, sink);
            return AdventureOutcome::default();
        }
        self.change_marks(hero, |marks| marks.add_timed(object.1));
        set_spell_points(hero, most)
    }

    /// A Magic Spring's visit by `player`'s hero `hero` from the entrance `tile` (ADV-BON-007;
    /// increment 12's design note §8.4; survey §10.2, §10.6, D-62): on every visit first the
    /// Spring's knowledge for the visitor's team (the original's player record's +0xBC bit, the
    /// hint's; ADV-VIS-006) and the kind mark; then the entrance's state: emptied this week,
    /// nothing, told; full, with spell points below twice the hero's maximum, the points set to
    /// twice it (saturating at the 16 bits) and the entrance emptied; at or above, nothing, told,
    /// the entrance staying full. The other entrance keeps its own state.
    // rule: ADV-BON-007, ADV-VIS-006
    pub(super) fn visit_spring<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (object, tile): (Visited, Pos),
        sink: &mut S,
    ) -> AdventureOutcome {
        self.learn(player, object.0);
        self.mark_kind(rules.1, player, object.1, sink);
        let emptied = self.springs.as_deref();
        if emptied.is_some_and(|emptied| emptied.binary_search(&tile).is_ok()) {
            Self::nothing((player, hero), object, NothingWhy::Emptied, sink);
            return AdventureOutcome::default();
        }
        let (mana, most) = spell_points(rules, hero);
        let twice = most.saturating_mul(2);
        if mana >= twice {
            Self::nothing((player, hero), object, NothingWhy::Full, sink);
            return AdventureOutcome::default();
        }
        let emptied = Arc::make_mut(self.springs.get_or_insert_with(Arc::default));
        let at = emptied.partition_point(|&t| t < tile);
        emptied.insert(at, tile);
        set_spell_points(hero, twice)
    }

    /// The Stables' visit by `player`'s hero `hero` (ADV-BON-005; increment 12's design note
    /// §8.5; survey §10.2, §10.4, D-63): without the Stables' week mark (its type: any Stables of
    /// the type) `points` added to the points left and the day's maximum and the week mark, which
    /// adds them to every day's land points until the week ends (T115a's `week_land_points`);
    /// with it, no movement. Then **at every visit** every slot of `from` becomes `to`, its count
    /// kept, no price, told per slot ([`AdventureEventKind::CreaturesUpgraded`]). Neither:
    /// nothing, told (the original's message 136).
    // rule: ADV-BON-005
    pub(super) fn visit_stables<S: Sink>(
        &mut self,
        (player, hero): (PlayerId, HeroId),
        object: Visited,
        (points, from, to): (u32, CreatureTypeId, CreatureTypeId),
        sink: &mut S,
    ) {
        let moves = !self.holds_timed(hero, object.1);
        if moves {
            self.change_marks(hero, |marks| marks.add_timed(object.1));
        }
        let Some(on_map) = self.hero_mut(hero) else {
            return;
        };
        if moves {
            on_map.movement = on_map.movement.saturating_add(points);
            on_map.max_movement = on_map.max_movement.saturating_add(points);
        }
        let mut upgraded = false;
        for stack in on_map
            .army
            .iter_mut()
            .filter(|stack| stack.creature == from)
        {
            stack.creature = to;
            upgraded = true;
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::CreaturesUpgraded {
                    hero,
                    slot: stack.slot,
                    from,
                    to,
                },
                seen_by: Viewers::NONE.with(player),
            });
        }
        if !moves && !upgraded {
            Self::nothing((player, hero), object, NothingWhy::Visited, sink);
        }
    }

    /// The Sirens' visit by `player`'s hero `hero` (ADV-BON-008; increment 12's design note §9;
    /// survey §10.5, D-64, A-45): holding `mark`, nothing, told. Else, slot by slot in slot order,
    /// a stack of more than one creature keeps **trunc(count × keep)**, the original's double
    /// product ([`sirens_keep`]), in full width (A-45 fixed, `ADV-QUIRK-47`), told per slot that
    /// lost ([`AdventureEventKind::SirensLoss`]); a lone creature is kept. The losses go to the
    /// outcome ([`AdventureOutcome::drowned`]): the game sums their hit points into the hero's
    /// experience with its levels. The mark is set either way (no battle bonus); a battle's end
    /// clears it only under `ADV-QUIRK-46` ([`Self::clear_battle_bonuses`]).
    // rule: ADV-BON-008
    pub(super) fn visit_sirens<S: Sink>(
        &mut self,
        (player, hero): (PlayerId, HeroId),
        object: Visited,
        (mark, keep): (u8, (u32, u32)),
        sink: &mut S,
    ) -> AdventureOutcome {
        let bonus = VisitBonus {
            mark,
            morale: 0,
            luck: 0,
        };
        if !self.give_once(hero, bonus) {
            Self::nothing((player, hero), object, NothingWhy::Visited, sink);
            return AdventureOutcome::default();
        }
        let mut lost = Vec::new();
        if let Some(on_map) = self.hero_mut(hero) {
            for stack in on_map.army.iter_mut().filter(|stack| stack.count > 1) {
                let count = stack.count - sirens_keep(stack.count, keep);
                if count > 0 {
                    stack.count -= count;
                    lost.push((stack.creature, count));
                    sink.emit(AdventureEvent {
                        kind: AdventureEventKind::SirensLoss {
                            hero,
                            slot: stack.slot,
                            creature: stack.creature,
                            count,
                        },
                        seen_by: Viewers::NONE.with(player),
                    });
                }
            }
        }
        AdventureOutcome {
            drowned: (!lost.is_empty()).then_some(Drowned { hero, lost }),
            ..AdventureOutcome::default()
        }
    }

    /// The end of a battle for its heroes on the map (GAM-BAT-014 amended, E08 T115a; increment
    /// 12's design note §12.3, §14): each of `heroes` still on the map loses its battle bonuses and
    /// their once-marks but those the part names as outliving a battle under the profile, the
    /// game calling it after the battle's capture and the heroes' part, as the original's
    /// `hero_battle_bonuses_clear` from `battle_end` (0x477792). Since E08 T115b the Sirens' mark
    /// outlives it with `ADV-QUIRK-46` off (`quirk.adventure.sirens_again_after_battle`; A-44,
    /// decision 308: the one mark of every Sirens kind kept, so a hero visits Sirens once); with
    /// it on (`faithful`, the original) it is cleared with the rest, so the hero visits Sirens
    /// again after each battle. A hero that left the map left with them. Told as the command's
    /// [`AdventureEventKind::OnceMarkChanged`] and [`AdventureEventKind::BonusChanged`].
    // rule: GAM-BAT-014, ADV-BON-008
    pub fn clear_battle_bonuses(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[HeroId],
    ) {
        let kept: Vec<u8> = if core.quirks().is_enabled(SIRENS_AGAIN_AFTER_BATTLE) {
            Vec::new()
        } else {
            (part.object_types().iter())
                .filter_map(|ty| match ty.kind {
                    ObjectKind::Sirens { mark, .. } => Some(mark),
                    _ => None,
                })
                .collect()
        };
        for &id in heroes {
            if self.visit_marks(id).bonuses().clears(&kept) {
                self.change_marks(id, |marks| marks.clear_bonuses(&kept));
            }
        }
    }

    /// The Fountains' values, then the Springs' emptied entrances, each as its length and entries
    /// (the adventure state hash v24; none and an empty block alike).
    pub(super) fn hash_visitables(&self, h: &mut StateHasher) {
        let fortunes = self.fortunes.as_deref().unwrap_or_default();
        h.write_len(fortunes.len());
        for &value in fortunes {
            h.write_i32(i32::from(value));
        }
        let springs = self.springs.as_deref().map_or(&[][..], Vec::as_slice);
        h.write_len(springs.len());
        for tile in springs {
            tile.hash_into(h);
        }
    }
}

/// The hero `hero`'s spell points and maximum (`h3_heroes::max_mana`, HER-MAN-001), from the
/// game's store; none known reads as full.
fn spell_points((core, _, heroes): Rules<'_>, hero: HeroId) -> (u16, u16) {
    details_of(heroes, hero).map_or((0, 0), |details| (details.mana, max_mana(details, core)))
}

/// The outcome that sets `hero`'s spell points to `points` (the game's change, checked against
/// the details the visit read).
fn set_spell_points(hero: HeroId, points: u16) -> AdventureOutcome {
    AdventureOutcome {
        hero: Some(HeroGains::changes(
            hero,
            alloc::vec![HeroChange::SetSpellPoints(points)],
        )),
        ..AdventureOutcome::default()
    }
}

/// What a stack of `count` keeps at the Sirens (ADV-BON-008; increment 12's design note §9; survey
/// §10.5, D-64): **trunc(count × keep)**, the product of the count and the keep's quotient in
/// `kernel::num`'s `Real` (ADR-0003's audited surface): the correctly rounded quotient 7 ÷ 10 is
/// the double nearest 0.7, the original's constant 0x63E688, and the original's x87 product at
/// 53 bits is the double product, so 90 keeps 62 and 170 keeps 118 (not the exact tenths). In
/// full width (A-45 fixed: the original cuts the count to 16 bits signed, `ADV-QUIRK-47`); at
/// least one (the original's 0.7 always keeps one of two; a pack's smaller part would empty a
/// slot: *agent's call*), at most the count. The count fits a `Real` exactly (a stack's most is
/// 1 000 000, below 2^31).
// rule: ADV-BON-008
pub(crate) fn sirens_keep(count: u32, (numerator, denominator): (u32, u32)) -> u32 {
    let real = |value: u32| Real::from_i32(i32::try_from(value).unwrap_or(i32::MAX));
    real(numerator)
        .checked_div(real(denominator))
        .and_then(|part| real(count).checked_mul(part))
        .and_then(Real::trunc_to_i64)
        .ok()
        .and_then(|kept| u32::try_from(kept).ok())
        .unwrap_or(count)
        .clamp(1, count)
}
