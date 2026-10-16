//! What observations show (E04b T9a): what the battle keeps per stack, the natural obstacles a
//! removal cleared, each hero's hidden magic to its own side only (BAT-Q26), a spectator's view
//! (SRV-Q4), and a town's private facts (its mines, its Escape Tunnel; E04c T10b).

use std::vec;
use std::vec::Vec;

use h3_content_battle::{BattleRuleset, ObstacleKind};
use h3_contracts::{BattleEffect, BattleSide, EagleEye, FortLevel, Moat, Rate, Siege};

use super::spells::{arena, arena_on, mage, spell};
use super::support::*;
use crate::{
    BattleState, Defences, Event, EventKind, Hex, NO_CORPSE, Opening, SpellSet, StackId, Viewer,
    Visibility, observe, observe_as, project, project_as,
};

/// Two stacks a side, the attacker's first active.
fn duel(r: &BattleRuleset, heroes: [Option<h3_contracts::BattleHero>; 2]) -> BattleState {
    arena(
        r,
        heroes,
        &[("fixed", 10, 2, 5), ("fixed", 4, 2, 7)],
        &[("fixed", 10, 14, 5), ("fixed", 4, 14, 7)],
        1,
    )
}

/// Places a hidden Quicksand of `side` on `hex`.
fn pit(s: &mut BattleState, r: &BattleRuleset, side: BattleSide, hex: Hex) {
    let id = spell(r, "quicksand");
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let placed = (id, ObstacleKind::Pit, &shape);
    crate::obstacles::place(s, placed, side, (&[hex], 0), &mut Vec::new());
}

// covers: BAT-MAG-035, BAT-MAG-036, BAT-MAG-038, BAT-MAG-039, BAT-MAG-044, BAT-MAG-045,
// BAT-MAG-054
#[test]
fn observations_show_what_the_battle_keeps_per_stack() {
    let r = ruleset();
    let mut s = duel(&r, [None, None]);
    // Bind, Poison and Age, the raising spells' counts, the wait phase and the summons' creature
    // as the battle keeps them.
    s.binds.by[2] = 0b1001;
    s.vitality.poison[2] = 3;
    s.vitality.base[2] = 30;
    s.raised.temporary[0] = 4;
    s.raised.corpse[1] = 2;
    s.raised.corpse[3] = NO_CORPSE;
    s.wait_phase = true;
    for viewer in [
        Viewer::Side(BattleSide::Attacker),
        Viewer::Side(BattleSide::Defender),
    ]
    .into_iter()
    .chain([Viewer::Spectator])
    {
        let o = observe_as(&s, &r, viewer);
        assert_eq!(o.binders[..4], [0, 0, 0b1001, 0]);
        assert_eq!((o.poison[2], o.start_hp[2], o.start_hp[1]), (3, 30, 0));
        assert_eq!(
            (o.temporary[0], &o.corpses[..4]),
            (4, &[0, 2, 0, NO_CORPSE][..])
        );
        assert!(o.wait_phase);
        // No clone, no summons, no hero to change the costs.
        assert!(o.clone_links.iter().all(Option::is_none));
        assert_eq!((o.summoned, o.cost_change), ([None, None], [0, 0]));
    }
}

// covers: BAT-MAG-061, BAT-MAG-062, BAT-MAG-063, BAT-MAG-064, BAT-MAG-065, BAT-MAG-066,
// BAT-MAG-068, BAT-MAG-069, HER-BAT-014
#[test]
fn a_heros_hidden_magic_shows_only_to_its_side() {
    let r = ruleset();
    let bolt = spell(&r, "bolt");
    let mut h = mage(&r, &["bolt"]);
    h.effects = vec![
        BattleEffect::MagicResistance {
            rate: Rate::bp(2000),
        },
        BattleEffect::SpellDuration { rounds: 2 },
        BattleEffect::NoCasting,
        BattleEffect::SpellLevelLimit { max_level: 2 },
        BattleEffect::SpellImmunity { spell: bolt },
        BattleEffect::SpellLevelImmunity { max_level: 3 },
        BattleEffect::MindImmunity,
        BattleEffect::NegateImmunities,
        BattleEffect::OpeningSpell {
            spell: spell(&r, "prayer"),
            mastery: 3,
            power: 10,
        },
    ];
    h.magic.eagle_eye = Some(EagleEye {
        chance: Rate::bp(4000),
        spells: vec![bolt],
    });
    let s = duel(&r, [Some(h), Some(mage(&r, &["bolt"]))]);
    let own = observe(&s, &r, BattleSide::Attacker).own_hero.unwrap();
    let mut immune = SpellSet::default();
    immune.insert(bolt);
    assert_eq!(
        (own.magic_resistance_bp, own.duration_bonus, own.no_casting),
        (2000, 2, true)
    );
    assert_eq!(own.level_limit, Some(2));
    assert_eq!(
        own.defences,
        Defences {
            spells: immune,
            up_to_level: Some(3),
            mind: true
        }
    );
    assert!(own.negates);
    let opening = Opening {
        spell: spell(&r, "prayer"),
        mastery: 3,
        power: 10,
    };
    // Cast at the side's first activation, which the duel ran (BAT-MAG-068).
    assert_eq!(
        (own.opening.as_slice(), own.opening_due),
        (&[opening][..], false)
    );
    assert_eq!(
        (own.eagle_chance, own.eagle_spells, own.learned),
        (40, immune, SpellSet::default())
    );
    // The other side's own part is its own hero's, which has none of it; a spectator has none.
    let other = observe(&s, &r, BattleSide::Defender).own_hero.unwrap();
    assert_eq!((other.magic_resistance_bp, other.level_limit), (0, None));
    assert_eq!(
        (other.no_casting, other.negates, other.opening_due),
        (false, false, false)
    );
    assert_eq!(observe_as(&s, &r, Viewer::Spectator).own_hero, None);
}

// covers: BAT-MAG-031, BAT-MAG-049, BAT-MAG-050
#[test]
fn a_spectator_sees_what_both_sides_see() {
    let r = ruleset();
    let mut s = duel(&r, [None, None]);
    let hex = at(&r, 8, 3);
    pit(&mut s, &r, BattleSide::Attacker, hex);
    let shown = |s: &BattleState, viewer: Viewer| -> Vec<(Hex, bool)> {
        observe_as(s, &r, viewer)
            .spell_obstacles
            .iter()
            .flat_map(|o| o.hexes.iter().map(move |h| (h, o.revealed)))
            .collect()
    };
    // The hidden pit: its side sees it, unseen by the other side; the others do not.
    assert_eq!(
        shown(&s, Viewer::Side(BattleSide::Attacker)),
        [(hex, false)]
    );
    assert!(shown(&s, Viewer::Side(BattleSide::Defender)).is_empty());
    assert!(shown(&s, Viewer::Spectator).is_empty());
    // Once revealed, everyone sees it, seen by both sides.
    s.spell_obstacles.reveal_for_tests(hex);
    for viewer in [Viewer::Side(BattleSide::Defender), Viewer::Spectator] {
        assert_eq!(shown(&s, viewer), [(hex, true)]);
    }
    // A drawn spell shows to the caster's controller only.
    s.casts.drawn = Some((StackId(0), spell(&r, "bolt")));
    assert_eq!(
        observe(&s, &r, BattleSide::Attacker).drawn,
        Some(spell(&r, "bolt"))
    );
    let spectator = observe_as(&s, &r, Viewer::Spectator);
    assert_eq!(
        (spectator.drawn, spectator.viewer),
        (None, BattleSide::Attacker)
    );
    assert!(spectator.spectator && spectator.cast_candidates.iter().all(|&n| n == 0));
    // Events: a spectator is told the public ones, a side its own too.
    let event = |visibility| Event {
        visibility,
        kind: EventKind::RoundStarted { round: 2 },
    };
    let public = event(Visibility::Public);
    let private = event(Visibility::Side(BattleSide::Defender));
    assert_eq!(project_as(&public, Viewer::Spectator), Some(public));
    assert_eq!(project_as(&private, Viewer::Spectator), None);
    assert_eq!(project(&private, BattleSide::Defender), Some(private));
    assert_eq!(project_as(&private, BattleSide::Attacker.into()), None);
}

// covers: BAT-SIE-007, BAT-SIE-024, BAT-MAG-031
#[test]
fn a_towns_mines_and_escape_tunnel_show_to_its_defender_only() {
    let r = ruleset();
    // A Citadel of the Tower's test record with a damaging moat (its mines) and a defending
    // hero; the attacker has none (no Catapult).
    let start = |escape_tunnel| {
        let town = Siege {
            faction: r.faction_by_key("t:necropolis").unwrap(),
            fort: FortLevel::Citadel,
            moat: Moat::Damaging,
            structures: 0,
            escape_tunnel,
            morale: 0,
            luck: 0,
        };
        arena_on(
            &r,
            [None, Some(hero(&r))],
            &[("fixed", 10, 2, 5)],
            &[("fixed", 10, 14, 9)],
            1,
            |s| s.siege = Some(town),
        )
    };
    let (plain, tunnel) = (start(false), start(true));
    // The mines: the defender sees its ten, none seen by the attacker; the others see none.
    for s in [&plain, &tunnel] {
        let own = observe(s, &r, BattleSide::Defender).spell_obstacles;
        assert_eq!(own.len(), 10);
        assert!(
            own.iter()
                .all(|o| o.kind == ObstacleKind::Mine && !o.revealed)
        );
        for viewer in [Viewer::Side(BattleSide::Attacker), Viewer::Spectator] {
            assert!(observe_as(s, &r, viewer).spell_obstacles.is_empty());
        }
    }
    // The Escape Tunnel (BAT-Q50 (4)): the defender sees its own option to retreat; the attacker
    // and a spectator see it as `false` (unknown) and the same town as without the tunnel.
    assert_eq!(
        observe(&plain, &r, BattleSide::Defender).may_retreat,
        [false, false]
    );
    assert_eq!(
        observe(&tunnel, &r, BattleSide::Defender).may_retreat,
        [false, true]
    );
    for viewer in [Viewer::Side(BattleSide::Attacker), Viewer::Spectator] {
        let seen = observe_as(&tunnel, &r, viewer);
        assert_eq!(seen.may_retreat, [false, false]);
        assert_eq!(seen, observe_as(&plain, &r, viewer));
    }
    // The defender's own view differs in its option and its town only.
    let mut own = observe(&tunnel, &r, BattleSide::Defender);
    let without = observe(&plain, &r, BattleSide::Defender);
    assert_ne!(own.siege, without.siege);
    own.may_retreat = without.may_retreat;
    own.siege = without.siege;
    assert_eq!(own, without);
}
