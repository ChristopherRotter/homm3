//! A siege's battle as the game builds it (E09 T8b1; the town epic's design note §15, ADR-0023
//! points 2, 3 and 5): the town's facts in the battle's setup, the defending hero raised by the
//! town's buildings, the battle's terrain and battlefield, and after the battle the defending
//! hero's spell points and the attacking winner's experience. The battle is opened and ended as every battle on the map
//! (`GameState::start_battle`, `end_battle`); its capture after the attacker's win is the
//! adventure's (`AdventureState::give_won`, TWN-CAP-003) composed by [`super::capture`].

use h3_adventure::{AdventureState, Pos};
use h3_content::HeroParams;
use h3_contracts::{BattleHero, BattleSetup, TownId};
use h3_towns::{DefenderBonus, Towns, defender_bonus, siege};

use crate::GameRuleset;

/// Puts the siege of `town`, fought at `tile`, its entrance, into `setup`, built as every battle
/// on the map (GAM-BAT-001): `siege` the town's facts ([`h3_towns::siege`], TWN-SIE-003); the
/// defending hero, if one defends, raised by the town's buildings ([`raise`], TWN-SIE-004); the
/// terrain its faction's native one, the battlefield the special ground on `tile` if any, never
/// the coast's shore nor a boat's deck (TWN-SIE-005: the original's `battle_setup_battlefield`
/// 0x4643B0 takes a siege's terrain first, then the tile's special ground in every case;
/// `installation.md` §7.26). A faction without a native terrain keeps the setup's. The obstacles
/// are the caller's, drawn after this (`generate_setup_obstacles`: none at a town with a fort).
/// A town the store lacks, or a `tile` that is not its entrance, changes nothing: no siege (the
/// store holds every town of the map; the battle's end finds the town of a siege by its tile).
// rule: TWN-SIE-003, TWN-SIE-004, TWN-SIE-005
pub(crate) fn siege_setup(
    (towns, adventure): (&Towns, &AdventureState),
    rules: &GameRuleset,
    (town, tile): (TownId, Pos),
    setup: &mut BattleSetup,
) {
    // The battle's town is its tile's at its end (`GameState::end_battle`): a tile that is not
    // the town's entrance fights no siege.
    let at_entrance = adventure.map().town_at(tile) == Some(town);
    let Some(record) = towns.get(town).filter(|_| at_entrance) else {
        return;
    };
    let part = rules.towns();
    // rule: TWN-SIE-003
    setup.siege = Some(siege(record, part));
    // rule: TWN-SIE-004
    if let Some(hero) = setup.heroes[1].as_mut() {
        raise(
            hero,
            defender_bonus(record, part),
            rules.core().hero_params(),
        );
    }
    // rule: TWN-SIE-005
    let native = rules
        .core()
        .factions()
        .get(record.faction().index())
        .and_then(|faction| faction.native_terrain);
    setup.terrain = native.or(setup.terrain);
    setup.battlefield = adventure.special_ground(rules.adventure(), tile);
    // stub: TWN-STUB-8 (a walled town whose record draws natural obstacles,
    // `h3_towns::siege_obstacles`, gets none: `generate_setup_obstacles` places none at a fort)
}

/// Raises a siege's defending hero by `bonus`, its town's (TWN-SIE-004; ADR-0023 point 3): its
/// attack, defense and power by the bonus's, at most the core's `primary_max` (the original
/// raises the hero's own skills, 0x463C60, which its effective skills then hold within the
/// highest, HER-DET-001), and its spell points by the bonus's mana, saturating (above the
/// maximum, as a Mana Vortex's; the battle's `BattleMagic::mana`).
// rule: TWN-SIE-004
pub(crate) fn raise(hero: &mut BattleHero, bonus: DefenderBonus, params: Option<&HeroParams>) {
    let max = params.map_or(u8::MAX, |params| params.primary_max);
    let up = |value: u8, by: u8| value.saturating_add(by).min(max.max(value));
    hero.attack = up(hero.attack, bonus.attack);
    hero.defense = up(hero.defense, bonus.defense);
    hero.power = up(hero.power, bonus.power);
    hero.magic.mana = hero.magic.mana.saturating_add(bonus.mana);
}

/// The spell points a siege's defending hero keeps after the battle (TWN-SIE-004; ADR-0023 point
/// 3): the smaller of those it has left and those it had before (the original's battle end
/// 0x4776E1 to 0x477719, which also gives back its own attack, defense and power: the battle
/// never wrote them to the hero).
// rule: TWN-SIE-004
pub(crate) const fn mana_after(left: u16, before: u16) -> u16 {
    if left < before { left } else { before }
}

/// The experience a siege's attacking hero gains as its winner, before Learning, beside the
/// battle's (TWN-SIE-006; ADR-0023 point 1): `base` (GAM-BAT-002's) and [`SIEGE_EXPERIENCE`] when
/// the attacker `won` a battle at a town, whatever the fort and the result (a retreat through the
/// Escape Tunnel included); `base` otherwise. Saturating.
// rule: TWN-SIE-006
pub(crate) const fn siege_experience(base: u32, won: bool) -> u32 {
    if won {
        base.saturating_add(SIEGE_EXPERIENCE)
    } else {
        base
    }
}

/// The attacking side's experience for a battle won at a town (TWN-SIE-006; the original's
/// `battle_experience` 0x46A350, +500 at 0x46A416 when the battle's town +0x53C8 is set and the
/// winning side is the attacker, after the defeated hero's +500 and the −500 of a retreat or a
/// surrender; `installation.md` §7.27).
pub(crate) const SIEGE_EXPERIENCE: u32 = 500;
