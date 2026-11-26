//! A cast on the adventure map (GAM-SPL-001, ADV-SPL-001; E08 T142a, increment 15's design note
//! §14, ADR-0017 point 4): the game's checks in the original's order, the adventure's half
//! ([`h3_adventure::AdventureState::apply_cast`], its draws from the caster's player's stream
//! since E08 T143a), then the spell points. The game holds what the
//! adventure lacks: the hero's details (its spells, masteries and points) and the battle part's
//! battlefields, whose special ground under the hero raises a mastery or limits a spell's level
//! (ADR-0015 point 2, ADR-0013 point 17).

use alloc::vec::Vec;

use h3_adventure::{Cast, Pos};
use h3_content::SpellId;
use h3_content_battle::Battlefield;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroChange, HeroDetails, MapSpell, SchoolRaise, can_cast_on_map, map_spells};

use super::GameState;
use crate::GameRuleset;
use crate::command::{AdventureSink, GameCommand, GameCommandError, GameEvent, MapCastError, Sink};

/// A cast the game accepted (GAM-SPL-001): the adventure's cast (the spell's behaviour, the
/// mastery on the hero's ground, the target) and the spell points it costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MapCast {
    pub(crate) cast: Cast,
    pub(crate) cost: u16,
}

impl GameState {
    /// Checks `player`'s cast of `spell` by its hero `hero` at `target` as [`Self::cast`] would
    /// make it (GAM-SPL-001; increment 15's design note §14, E08 decision 414), in order:
    ///
    /// 1. the hero one of the player's heroes on the map ([`MapCastError::NotYourHero`]);
    /// 2. the spell castable by it (`h3_heroes::can_cast_on_map`: among its available spells,
    ///    worn scrolls included, no book test, D-79);
    /// 3. a behaviour the adventure part binds to the spell
    ///    ([`h3_adventure::AdventureRuleset::adventure_spell`]);
    /// 4. a target exactly for Scuttle Boat and Dimension Door;
    /// 5. the cost (`h3_heroes::map_spell_cost` at the mastery on the hero's tile's ground: the
    ///    special ground's battlefield raises its `expert_schools` to Expert, ADR-0013 point 17)
    ///    against the spell points (the original's book tests them first, survey `E08-spells.md`
    ///    §4.1);
    /// 6. the ground: the special ground's highest spell level (Cursed Ground's 1, BAT-TER-004's
    ///    data) not below the spell's level, then no anti-magic garrison under the hero
    ///    (ADV-GAR-005: in or on one);
    /// 7. the adventure's own test ([`h3_adventure::AdventureState::check_cast`]).
    ///
    /// Batch E15 saw the ground's and the garrison's refusals free (E15-4) and Magic Plains'
    /// raise (E15-5). Nothing changes.
    // rule: GAM-SPL-001, ADV-SPL-001
    pub(crate) fn check_cast(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        (hero, spell, target): (HeroId, SpellId, Option<Pos>),
    ) -> Result<MapCast, MapCastError> {
        let (core, part) = (rules.core(), rules.adventure());
        let (pos, details) = self.caster(player, hero)?;
        let ground = self.ground_of(rules, pos);
        if can_cast_on_map(details, core, spell) != Ok(true) {
            return Err(MapCastError::Unknown(spell));
        }
        // E08 T144a1: the mastery and the cost as the listing derives them
        // (`h3_heroes::map_spells`: `spell_masteries_on`'s and `map_spell_cost`'s answers).
        let known = map_spells(details, core, &[spell], raise_of(ground));
        let (mastery, cost) = known
            .first()
            .map_or((0, u16::MAX), |known| (known.mastery, known.cost));
        // stub: ADV-STUB-19
        // stub: ADV-STUB-20
        let effect = part
            .adventure_spell(spell)
            .ok_or(MapCastError::NotOnMap(spell))?;
        if effect.targets() != target.is_some() {
            return Err(MapCastError::Target);
        }
        self.cast_frame(rules, (pos, details.mana, ground), (spell, cost))?;
        let cast = Cast {
            spell,
            effect,
            mastery,
            target,
        };
        self.adventure
            .check_cast((core, part), &self.heroes, (player, hero), cast)
            .map_err(MapCastError::Adventure)?;
        Ok(MapCast { cast, cost })
    }

    /// The first test of [`Self::check_cast`]: `hero` one of `player`'s heroes on the map, with
    /// its tile and details.
    fn caster(&self, player: PlayerId, hero: HeroId) -> Result<(Pos, &HeroDetails), MapCastError> {
        // Every hero on the map is in the store (GAM-HER-001).
        self.adventure
            .hero(hero)
            .filter(|on| on.owner == player)
            .and_then(|on| {
                let details = self.heroes.get(usize::from(hero.raw()));
                details.map(|details| (on.pos, &**details))
            })
            .ok_or(MapCastError::NotYourHero(hero))
    }

    /// The tests 5 and 6 of [`Self::check_cast`] of a cast of `spell` for `cost` by a hero on
    /// `pos` with `mana` spell points and `ground` under it: the cost against the points, then
    /// the ground's highest spell level and the anti-magic garrison.
    fn cast_frame(
        &self,
        rules: &GameRuleset,
        (pos, mana, ground): (Pos, u16, Option<&Battlefield>),
        (spell, cost): (SpellId, u16),
    ) -> Result<(), MapCastError> {
        if cost > mana {
            return Err(MapCastError::SpellPoints);
        }
        let level = rules.core().spell(spell).map_or(u8::MAX, |s| s.level);
        if ground.is_some_and(|field| level > field.max_spell_level) {
            return Err(MapCastError::Ground);
        }
        if self.adventure.anti_magic(rules.adventure(), pos) {
            return Err(MapCastError::AntiMagic);
        }
        Ok(())
    }

    /// The casts without a target `player` may make now, as [`Self::check_cast`] accepts them
    /// (GAM-SPL-001; E08 T144a1, increment 15's design note §17, E08 decision 422): for each of
    /// its heroes on the map in its list's order (GAM-HER-003), each spell the adventure part
    /// binds without a target that the hero may cast (Summon Boat, Fly, Water Walk), in the
    /// part's order (the core's spell order). Scuttle Boat and Dimension Door are given per hero
    /// by their targets ([`Self::cast_targets`], `crate::legal_cast_targets`), as the moves are.
    /// Each call runs on every step of a random game, so a hero is first tested by the ruleset's
    /// sources of the bound spells, without validating its details
    /// (`h3_heroes::SpellSources::may_have`), and only a possible caster's spells are derived,
    /// once (`h3_heroes::map_spells`).
    // rule: GAM-SPL-001
    pub(crate) fn legal_casts(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        listed: &mut Vec<GameCommand>,
    ) {
        let (core, part) = (rules.core(), rules.adventure());
        let sources = part.cast_sources();
        // The heroes of the store are those on the map (GAM-HER-001); the key of the player's
        // list (`hero_list`'s, unique by the id).
        let mut casters: Vec<(u32, HeroId)> = self
            .adventure
            .heroes()
            .iter()
            .filter(|on| on.owner == player)
            .filter(|on| {
                let details = self.heroes.get(usize::from(on.id.raw()));
                details.is_some_and(|details| sources.may_have(details))
            })
            .map(|on| (on.listed, on.id))
            .collect();
        if casters.is_empty() {
            return;
        }
        let _ = h3_kernel::order::sort_by_unique_key(&mut casters, |&(place, id)| (place, id));
        // Each a hero of the player's on the map (above).
        let casters = casters
            .into_iter()
            .filter_map(|(_, hero)| self.caster(player, hero).ok().map(|c| (hero, c)));
        for (hero, (pos, details)) in casters {
            let ground = self.ground_of(rules, pos);
            let known = map_spells(details, core, sources.spells(), raise_of(ground));
            // The sources' spells are the part's without a target.
            let untargeted = known
                .into_iter()
                .filter_map(|known| part.adventure_spell(known.spell).map(|e| (known, e)));
            for (
                MapSpell {
                    spell,
                    mastery,
                    cost,
                },
                effect,
            ) in untargeted
            {
                let frame = (pos, details.mana, ground);
                let cast = Cast {
                    spell,
                    effect,
                    mastery,
                    target: None,
                };
                let parts = (core, part);
                if self.cast_frame(rules, frame, (spell, cost)).is_ok()
                    && self
                        .adventure
                        .check_cast(parts, &self.heroes, (player, hero), cast)
                        .is_ok()
                {
                    listed.push(GameCommand::Cast {
                        hero,
                        spell,
                        target: None,
                    });
                }
            }
        }
    }

    /// The targets at which `player`'s hero `hero` may cast `spell` now, Scuttle Boat or
    /// Dimension Door, in the window's row order: exactly those [`Self::check_cast`] accepts
    /// (GAM-SPL-001; E08 T144a1, decision 422; a charged failure too: under `strict-fixed` the
    /// door's and the scuttle's hidden tiles, under `faithful` the door's wrong kind, increment
    /// 15's design note §11), the hero's and the cast's own tests made once
    /// ([`h3_adventure::AdventureState::cast_targets`]). None for a spell without a target, one
    /// the hero may not cast there, or a hero not `player`'s on the map.
    // rule: GAM-SPL-001
    pub(crate) fn cast_targets(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        (hero, spell): (HeroId, SpellId),
    ) -> Vec<Pos> {
        let (core, part) = (rules.core(), rules.adventure());
        let mut targets = Vec::new();
        let Ok((pos, details)) = self.caster(player, hero) else {
            return targets;
        };
        let Some(effect) = part.adventure_spell(spell).filter(|e| e.targets()) else {
            return targets;
        };
        // E08 T144a2: the env asks every hero on every step, so whether the hero may cast the
        // spell comes first, by the ruleset's sources (as the listing's pre-test), then its
        // spells, and only then its ground (a search of its tile's objects): a ground raises a
        // mastery, never makes a spell castable (HER-SPL-003).
        if !part.target_sources().may_have(details) {
            return targets;
        }
        let plain = map_spells(details, core, &[spell], SchoolRaise::None);
        let Some(&first) = plain.first() else {
            return targets;
        };
        let ground = self.ground_of(rules, pos);
        let MapSpell { mastery, cost, .. } = match ground {
            None => first,
            // The same spell, raised.
            Some(_) => map_spells(details, core, &[spell], raise_of(ground))
                .first()
                .copied()
                .unwrap_or(first),
        };
        if self
            .cast_frame(rules, (pos, details.mana, ground), (spell, cost))
            .is_err()
        {
            return targets;
        }
        let cast = Cast {
            spell,
            effect,
            mastery,
            target: None,
        };
        let parts = (core, part);
        self.adventure
            .cast_targets(parts, (player, hero), cast, |t| targets.push(t));
        targets
    }

    /// `player`'s cast of `spell` by its hero `hero` at `target` (GAM-SPL-001; increment 15's
    /// design note §14), checked whole first ([`Self::check_cast`]): the game tells
    /// [`GameEvent::SpellCast`] to the owner, the adventure makes its effect
    /// ([`h3_adventure::AdventureState::apply_cast`]: since E08 T143a Summon Boat's and Scuttle
    /// Boat's chance drawn from `player`'s stream, ADR-0017 point 4, and their boats' events;
    /// since E08 T143b Dimension Door's arrival with its movement points), a charged failure is
    /// told as [`GameEvent::CastFailed`], and the hero's spell points are set
    /// to the points less the cost, at least 0, in every accepted case (`HeroChange::
    /// SetSpellPoints`, its `HeroChanged`; the original's `hero_spend_mana`). The changed day
    /// spells are told after the command (`DaySpellsChanged`).
    ///
    /// # Errors
    ///
    /// [`GameCommandError::Cast`] with the first test the cast fails; nothing changed.
    // rule: GAM-SPL-001
    pub(crate) fn cast<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        (hero, spell, target): (HeroId, SpellId, Option<Pos>),
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        let cast = self
            .check_cast(rules, player, (hero, spell, target))
            .map_err(GameCommandError::Cast)?;
        let (core, part) = (rules.core(), rules.adventure());
        let MapCast { cast, cost } = cast;
        sink.emit(GameEvent::SpellCast {
            player,
            hero,
            spell,
            mastery: cast.mastery,
            cost,
        });
        let (adventure, heroes, draws) = self.adventure_for(player);
        let made = adventure.apply_cast(
            (core, part, heroes),
            (player, hero),
            cast,
            draws,
            &mut AdventureSink(&mut *sink),
        );
        if let Err(reason) = made {
            sink.emit(GameEvent::CastFailed {
                player,
                hero,
                spell,
                reason,
            });
        }
        let points = self
            .heroes
            .get(usize::from(hero.raw()))
            .map_or(0, |details| details.mana.saturating_sub(cost));
        self.change_hero(
            core,
            hero,
            Some(player),
            HeroChange::SetSpellPoints(points),
            sink,
        );
        Ok(())
    }

    /// The battlefield of the special ground on `pos`, if one lies there and the battle part has
    /// it (ADV-GRD-001: [`h3_adventure::AdventureState::special_ground`]).
    fn ground_of<'r>(&self, rules: &'r GameRuleset, pos: Pos) -> Option<&'r Battlefield> {
        let battle = rules.battle();
        self.adventure
            .special_ground(rules.adventure(), pos)
            .and_then(|key| battle.battlefield_by_key(key.as_str()))
            .and_then(|id| battle.battlefield(id))
    }
}

/// The raise of the special ground `ground`, if any: its battlefield's `expert_schools` at Expert
/// (ADR-0013 point 17).
fn raise_of(ground: Option<&Battlefield>) -> SchoolRaise {
    ground.map_or(SchoolRaise::None, |field| {
        SchoolRaise::Schools(field.expert_schools.iter().copied().collect())
    })
}
