//! A cast on the adventure map (GAM-SPL-001, ADV-SPL-001; E08 T142a, increment 15's design note
//! §14, ADR-0017 point 4): the game's checks in the original's order, the adventure's half
//! ([`h3_adventure::AdventureState::cast`]), then the spell points. The game holds what the
//! adventure lacks: the hero's details (its spells, masteries and points) and the battle part's
//! battlefields, whose special ground under the hero raises a mastery or limits a spell's level
//! (ADR-0015 point 2, ADR-0013 point 17).

use h3_adventure::{AdventureSpell, Pos};
use h3_content::SpellId;
use h3_content_battle::Battlefield;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroChange, SchoolRaise, can_cast_on_map, map_spell_cost, spell_masteries_on};

use super::GameState;
use crate::GameRuleset;
use crate::command::{GameCommandError, GameEvent, MapCastError, Sink};

/// A cast the game accepted (GAM-SPL-001): the spell's behaviour, the mastery on the hero's
/// ground and the spell points it costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MapCast {
    pub(crate) effect: AdventureSpell,
    pub(crate) mastery: u8,
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
        // Every hero on the map is in the store (GAM-HER-001).
        let (pos, details) = self
            .adventure
            .hero(hero)
            .filter(|on| on.owner == player)
            .and_then(|on| Some((on.pos, self.heroes.get(usize::from(hero.raw()))?)))
            .ok_or(MapCastError::NotYourHero(hero))?;
        if can_cast_on_map(details, core, spell) != Ok(true) {
            return Err(MapCastError::Unknown(spell));
        }
        // stub: ADV-STUB-19
        // stub: ADV-STUB-20
        let effect = part
            .adventure_spell(spell)
            .ok_or(MapCastError::NotOnMap(spell))?;
        if effect.targets() != target.is_some() {
            return Err(MapCastError::Target);
        }
        let ground = self.ground_of(rules, pos);
        let raise = ground.map_or(SchoolRaise::None, |field| {
            SchoolRaise::Schools(field.expert_schools.iter().copied().collect())
        });
        // The spell is known and the details valid (`can_cast_on_map`).
        let cost = map_spell_cost(details, core, spell, raise).unwrap_or(u16::MAX);
        if cost > details.mana {
            return Err(MapCastError::SpellPoints);
        }
        let level = core.spell(spell).map_or(u8::MAX, |s| s.level);
        if ground.is_some_and(|field| level > field.max_spell_level) {
            return Err(MapCastError::Ground);
        }
        if self.adventure.anti_magic(part, pos) {
            return Err(MapCastError::AntiMagic);
        }
        let mastery = spell_masteries_on(details, core, raise)
            .ok()
            .and_then(|masteries| masteries.into_iter().find(|&(s, _)| s == spell))
            .map_or(0, |(_, mastery)| mastery);
        self.adventure
            .check_cast(core, &self.heroes, (player, hero), effect, mastery)
            .map_err(MapCastError::Adventure)?;
        Ok(MapCast {
            effect,
            mastery,
            cost,
        })
    }

    /// `player`'s cast of `spell` by its hero `hero` at `target` (GAM-SPL-001; increment 15's
    /// design note §14), checked whole first ([`Self::check_cast`]): the adventure makes its
    /// effect ([`h3_adventure::AdventureState::apply_cast`]), the game tells
    /// [`GameEvent::SpellCast`] to the owner and sets the hero's spell points to the points less
    /// the cost, at least 0 (`HeroChange::SetSpellPoints`, its `HeroChanged`; the original's
    /// `hero_spend_mana`). The changed day spells are told after the command
    /// (`DaySpellsChanged`). No draw.
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
        let core = rules.core();
        self.adventure.apply_cast(hero, cast.effect, cast.mastery);
        sink.emit(GameEvent::SpellCast {
            player,
            hero,
            spell,
            mastery: cast.mastery,
            cost: cast.cost,
        });
        let points = self
            .heroes
            .get(usize::from(hero.raw()))
            .map_or(0, |details| details.mana.saturating_sub(cast.cost));
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
