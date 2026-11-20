//! Tests of the Altars of Sacrifice's visits and the adventure's checks of their sacrifices (E08
//! T134a; increment 14's design note §10, §11, §14): the kind mark and the window by the hero's
//! class's faction at every visit (ADV-ALT-001, ADV-BON-009), a sacrifice of artifacts by their
//! places (ADV-ALT-002; A-47 accepted as `ADV-QUIRK-49`) and of creatures by slot (ADV-ALT-003),
//! each item's Learning truncated on its own (D-73), what the window offers, the places' set
//! (decision 386) and the errors' texts.

use std::string::ToString;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ArtifactClass, ArtifactTypeId, CreatureTypeId, HeroClassDef, HeroEffectDef, HeroParamsDef,
    HeroTypeDef, HeroTypeId, SecondarySkillDef, SkillId,
};
use h3_contracts::ArmyStack;
use h3_heroes::{ArtifactPlace, HeroDetails, Position};

use super::draws::{object, rules_with};
use super::rewards::{AT, BOOK, CHARM, HERO, RED, RING, Rules, map_of, type_of};
use super::*;
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{
    ALTAR_BACKPACK_PLACES, ALTAR_WORN_PLACES, AltarError, AltarPlaces, AltarVisit, ServiceVisit,
};
use crate::ruleset::{AltarDef, AltarScreens, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::Holds;
use crate::state::{AltarArtifact, AltarOffers, AltarStack, Sacrificed};
use crate::testing::{GNOLL, PEASANT, PIKEMAN, WOLF, hero_details, key, stream};

const AMULET: ArtifactTypeId = ArtifactTypeId::from_raw(2);
const CLOAK: ArtifactTypeId = ArtifactTypeId::from_raw(3);
const HELM: ArtifactTypeId = ArtifactTypeId::from_raw(4);
const CROWN: ArtifactTypeId = ArtifactTypeId::from_raw(5);
/// Learning, after the five skills of `hero_skills` (5 %, 10 %, 15 %).
const LEARNING: SkillId = SkillId::from_raw(5);
/// The hero types by their class's faction: the castle's (artifacts), the fortress's (both), the
/// wild's (creatures).
const KNIGHT: HeroTypeId = HeroTypeId::from_raw(0);
const WARDEN: HeroTypeId = HeroTypeId::from_raw(1);
const SHAMAN: HeroTypeId = HeroTypeId::from_raw(2);

/// The draws' rules with Learning, a hero class and type for each of the three factions, hero
/// parameters (a backpack of eight, levels at 0, 1 000 and 2 000), the amulet a special artifact
/// the Altar takes (a Spell Scroll's case) beside the book it keeps, AI values (the pikeman 80, the
/// gnoll 39, the peasant 160, the wolf `u32::MAX`), and an entered `t:altar` (the base pack's
/// experience by class in hundredths, so Learning's shares are fractions: 10, 15, 30, 60, 0;
/// three places; the base pack's 5 × ⌊AI value / 40⌋). The factions' screens are the draws'
/// rules' (castle artifacts, fortress both, wild creatures).
fn rules() -> Rules {
    rules_with(
        true,
        |core| {
            core.skills.push(SecondarySkillDef {
                key: key("t:learning"),
                name: "Learning".into(),
                levels: [1_u16, 2, 3]
                    .map(|level| vec![HeroEffectDef::ExperienceGain { bp: 500 * level }]),
            });
            core.artifacts[AMULET.index()].class = ArtifactClass::Special;
            for (creature, ai) in [(PIKEMAN, 80), (GNOLL, 39), (PEASANT, 160), (WOLF, u32::MAX)] {
                core.creatures[creature.index()].info.ai_value = ai;
            }
            let class = |k: &str, faction: &str| HeroClassDef {
                key: key(k),
                name: "Class".into(),
                faction: key(faction),
                primary: [1, 1, 1, 1],
                primary_gain: [[25, 25, 25, 25], [25, 25, 25, 25]],
                skill_weights: vec![],
                offer_intervals: [4, 4],
            };
            core.hero_classes = vec![
                class("t:knight", "t:castle"),
                class("t:warden", "t:fortress"),
                class("t:shaman", "t:wild"),
            ];
            let hero_type = |k: &str, class: &str| HeroTypeDef {
                key: key(k),
                name: "Hero".into(),
                class: key(class),
                skills: vec![],
                spell_book: false,
                spells: vec![],
                army: vec![],
                war_machines: vec![],
                specialty: vec![],
            };
            core.heroes = vec![
                hero_type("t:sir", "t:knight"),
                hero_type("t:ward", "t:warden"),
                hero_type("t:sham", "t:shaman"),
            ];
            core.hero_params = Some(HeroParamsDef {
                primary_min: [0, 0, 1, 1],
                primary_max: 99,
                backpack: 8,
                learn_level: 2,
                experience: vec![0, 1000, 2000],
                max_level: 74,
                wisdom: key("t:logistics"),
                schools: vec![],
                mana_ceiling: 999,
                base_mana_regain: 1,
                regain_specialist_bonus: 1,
            });
        },
        |types| {
            types.push(ObjectTypeDef {
                key: key("t:altar"),
                name: "Altar".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: true,
                kind: ObjectKindDef::Altar(AltarDef {
                    experience: [10, 15, 30, 60, 0],
                    keeps: vec![key("t:book")],
                    places: 3,
                    creature_factor: 5,
                    creature_divisor: 40,
                }),
            });
        },
    )
}

/// The Altar's window at object index 0 and [`AT`] for hero 1.
fn window(rules: &Rules) -> AltarVisit {
    AltarVisit {
        hero: HERO,
        index: 0,
        object_type: type_of(rules, "t:altar"),
        anchor: AT,
    }
}

/// A stack.
fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// Hero 1's details: of `hero_type`, with Basic Learning, wearing `worn` on its one position and
/// carrying `backpack`.
fn details(
    hero_type: HeroTypeId,
    worn: Option<ArtifactTypeId>,
    backpack: &[ArtifactTypeId],
) -> HeroDetails {
    HeroDetails {
        hero_type,
        worn: vec![worn.map(Into::into)],
        backpack: backpack.iter().map(|&a| a.into()).collect(),
        ..hero_details(&[(LEARNING, 1)])
    }
}

/// The store of heroes with hero 1's `details`.
fn store(details: HeroDetails) -> Vec<Arc<HeroDetails>> {
    vec![
        Arc::new(hero_details(&[])),
        Arc::new(details),
        Arc::new(hero_details(&[])),
    ]
}

/// [`map_of`] with the Altar at [`AT`] and hero 1's army `army`.
fn at_altar(rules: &Rules, army: Vec<ArmyStack>) -> AdventureState {
    let mut state = map_of(rules, vec![object(rules, AT.x, "t:altar", Holds::Nothing)]);
    state.hero_mut(HERO).unwrap().army = army;
    state
}

/// Red's `command` with `heroes`: its outcome and events.
fn apply(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    command: &AdventureCommand,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let turn = (RED, Weekday::FIRST, &mut stream());
    let outcome = state.apply(&rules.0, &rules.1, heroes, turn, command, &mut events);
    (outcome.unwrap(), events)
}

/// The places `places` as an Altar's set.
fn set(places: &[ArtifactPlace]) -> AltarPlaces {
    places
        .iter()
        .fold(AltarPlaces::NONE, |set, &p| set.with(p).unwrap())
}

const fn worn(p: usize) -> ArtifactPlace {
    ArtifactPlace::Worn(Position(p))
}

const fn pack(i: usize) -> ArtifactPlace {
    ArtifactPlace::Backpack(i)
}

// covers: ADV-ALT-001, ADV-BON-009, ADV-OBJ-003, ADV-TEL-008
#[test]
fn an_altars_step_marks_its_kind_and_opens_the_window_its_heros_faction_allows() {
    let rules = rules();
    let altar = type_of(&rules, "t:altar");
    let step = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![AT],
    };
    for hero_type in [KNIGHT, WARDEN, SHAMAN] {
        let mut state = at_altar(&rules, vec![stack(0, PEASANT, 1)]);
        let heroes = store(details(hero_type, None, &[]));
        let (outcome, events) = apply(&rules, &mut state, &heroes, &step);
        let opened = AdventureOutcome {
            service: Some(ServiceVisit::Altar(window(&rules))),
            ..AdventureOutcome::default()
        };
        assert_eq!(outcome, opened);
        // Entered: the move ends on the entrance; the team learns the kind once.
        assert_eq!(state.hero(HERO).unwrap().pos, AT);
        let known = AdventureEventKind::KindKnown { object_type: altar };
        assert!(events.iter().any(|e| e.kind == known));
        assert!(state.knows_kind(RED, altar));
        // Every visit opens it again, the kind already known.
        let revisit = AdventureCommand::Revisit { hero: HERO };
        let (again, events) = apply(&rules, &mut state, &heroes, &revisit);
        assert_eq!(again, opened);
        assert!(events.iter().all(|e| e.kind != known));
    }
    // A hero the store lacks (no class, no faction) opens nothing, the kind marked all the same.
    let mut state = at_altar(&rules, vec![stack(0, PEASANT, 1)]);
    let heroes = store(hero_details(&[]));
    let (outcome, _) = apply(&rules, &mut state, &heroes, &step);
    assert_eq!(outcome, AdventureOutcome::default());
    assert!(state.knows_kind(RED, altar));
}

// covers: ADV-ALT-002, HER-EXP-001
#[test]
fn a_sacrifice_of_artifacts_names_its_places_and_takes_learning_per_artifact() {
    let rules = rules();
    let mut state = at_altar(&rules, vec![stack(0, PEASANT, 1)]);
    let altar = window(&rules);
    let backpack = [CHARM, CLOAK, HELM, CROWN, BOOK, AMULET];
    let heroes = store(details(WARDEN, Some(RING), &backpack));
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    let check =
        |places: &[ArtifactPlace]| state.check_artifact_sacrifice(rules3, &altar, set(places));
    // Two treasures of 10 with Basic Learning: 10.5 each, truncated each (D-73): 20, not 21.
    let two = check(&[worn(0), pack(0)]).unwrap();
    assert_eq!((two.hero(), two.experience()), (HERO, 20));
    // The minor, the major and the relic: 15.75, 31.5 and 63, each truncated.
    let three = check(&[pack(1), pack(2), pack(3)]).unwrap();
    assert_eq!(three.experience(), 15 + 31 + 63);
    // A special artifact gives 0 and goes with the others (A-47, ADV-QUIRK-49); alone it gives
    // nothing, as no place does.
    assert_eq!(check(&[pack(5), worn(0)]).unwrap().experience(), 10);
    assert_eq!(check(&[pack(5)]), Err(AltarError::Nothing));
    assert_eq!(check(&[]), Err(AltarError::Nothing));
    // The kept book, more places than the Altar's three, a place without an artifact.
    assert_eq!(check(&[pack(4)]), Err(AltarError::Kept(BOOK)));
    let four = [worn(0), pack(0), pack(1), pack(2)];
    assert_eq!(check(&four), Err(AltarError::Places));
    assert_eq!(check(&[pack(6)]), Err(AltarError::Places));
    assert_eq!(check(&[worn(1)]), Err(AltarError::Places));
    // Made: the places in the change's order, the worn first, the backpack's from the last down.
    let made = state.sacrifice(check(&[pack(0), worn(0), pack(3)]).unwrap());
    let removed = Sacrificed {
        hero: HERO,
        removed: vec![worn(0), pack(3), pack(0)],
        lowered: vec![],
        experience: 10 + 10 + 63,
    };
    assert_eq!(made, removed);
    // The army is the game's to tell, never touched by an artifacts' sacrifice.
    assert_eq!(state.hero(HERO).unwrap().army, [stack(0, PEASANT, 1)]);
    // A creatures' faction does not sacrifice artifacts; a visit that is no Altar's is refused.
    let shaman = store(details(SHAMAN, Some(RING), &[]));
    let refused =
        state.check_artifact_sacrifice((&rules.0, &rules.1, &shaman), &altar, set(&[worn(0)]));
    assert_eq!(refused, Err(AltarError::Screen));
    let elsewhere = AltarVisit { index: 1, ..altar };
    let refused = state.check_artifact_sacrifice(rules3, &elsewhere, set(&[worn(0)]));
    assert_eq!(refused, Err(AltarError::Visit));
}

// covers: ADV-ALT-003, HER-EXP-001
#[test]
fn a_sacrifice_of_creatures_keeps_the_army_a_creature_and_takes_learning_per_stack() {
    let rules = rules();
    let army = vec![
        stack(0, PIKEMAN, 5),
        stack(2, GNOLL, 3),
        stack(4, PEASANT, 2),
    ];
    let mut state = at_altar(&rules, army);
    let altar = window(&rules);
    let heroes = store(details(SHAMAN, None, &[]));
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    let check = |amounts| state.check_creature_sacrifice(rules3, &altar, amounts);
    // A pikeman of AI value 80: 5 × 2 = 10, 10.5 with Basic Learning; a peasant's 20, 21: per
    // stack truncated (D-73), 31.
    assert_eq!(check([1, 0, 0, 0, 1, 0, 0]).unwrap().experience(), 10 + 21);
    // Five pikemen 52.5, a gnoll of AI value 39 nothing, one peasant 21.
    assert_eq!(check([5, 0, 3, 0, 1, 0, 0]).unwrap().experience(), 52 + 21);
    // Every creature: the army keeps one.
    assert_eq!(check([5, 0, 3, 0, 2, 0, 0]), Err(AltarError::LastCreature));
    // An amount above its stack, on an empty slot; the gnolls alone or nothing give nothing.
    assert_eq!(check([6, 0, 0, 0, 0, 0, 0]), Err(AltarError::Amount(0)));
    assert_eq!(check([1, 1, 0, 0, 0, 0, 0]), Err(AltarError::Amount(1)));
    assert_eq!(check([0, 0, 3, 0, 0, 0, 0]), Err(AltarError::Nothing));
    assert_eq!(check([0; 7]), Err(AltarError::Nothing));
    // Made: each stack lowered, a stack at 0 emptied; the slots told in order.
    let made = state.sacrifice(check([5, 0, 3, 0, 1, 0, 0]).unwrap());
    let slot = |slot| ArmySlot {
        army: ArmyRef::Hero(HERO),
        slot,
    };
    let lowered = Sacrificed {
        hero: HERO,
        removed: vec![],
        lowered: vec![slot(0), slot(2), slot(4)],
        experience: 73,
    };
    assert_eq!(made, lowered);
    assert_eq!(state.hero(HERO).unwrap().army, [stack(4, PEASANT, 1)]);
    // A stack's experience saturates in 32 bits, the original's product (a hero without
    // Learning: nine wolves of AI value `u32::MAX`, 536 870 910 each, pass 2^32 together).
    state.hero_mut(HERO).unwrap().army = vec![stack(0, WOLF, 10)];
    let plain = store(HeroDetails {
        hero_type: SHAMAN,
        ..hero_details(&[])
    });
    let amounts = [9, 0, 0, 0, 0, 0, 0];
    let rich = state.check_creature_sacrifice((&rules.0, &rules.1, &plain), &altar, amounts);
    assert_eq!(rich.unwrap().experience(), u32::MAX);
    // An artifacts' faction does not sacrifice creatures.
    let knight = store(details(KNIGHT, None, &[]));
    let refused = state.check_creature_sacrifice(
        (&rules.0, &rules.1, &knight),
        &altar,
        [1, 0, 0, 0, 0, 0, 0],
    );
    assert_eq!(refused, Err(AltarError::Screen));
    let elsewhere = AltarVisit { index: 1, ..altar };
    let refused = state.check_creature_sacrifice(rules3, &elsewhere, [1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(refused, Err(AltarError::Visit));
}

// covers: ADV-ALT-001, ADV-ALT-002, ADV-ALT-003
#[test]
fn the_window_offers_each_artifact_and_stack_with_its_experience() {
    let rules = rules();
    let state = at_altar(&rules, vec![stack(1, PIKEMAN, 4)]);
    let altar = window(&rules);
    let offers = |details| {
        let heroes = store(details);
        state.altar_offers((&rules.0, &rules.1, &heroes), &altar)
    };
    let both = offers(HeroDetails {
        experience: 400,
        ..details(WARDEN, Some(RING), &[BOOK, AMULET, CROWN])
    });
    let artifact = |place, artifact, experience| AltarArtifact {
        place,
        artifact,
        experience,
    };
    let expected = AltarOffers {
        screens: AltarScreens {
            artifacts: true,
            creatures: true,
        },
        places: 3,
        artifacts: vec![
            artifact(worn(0), RING, Some(10)),
            artifact(pack(0), BOOK, None),
            artifact(pack(1), AMULET, Some(0)),
            artifact(pack(2), CROWN, Some(63)),
        ],
        // The lone stack: its count less one; three pikemen of 10, 31.5 with Learning.
        stacks: vec![AltarStack {
            slot: 1,
            each: 10,
            most: 3,
            experience: 31,
        }],
        to_next_level: 600,
    };
    assert_eq!(both, Ok(expected));
    // Each screen shows only what its faction sacrifices; an empty position shows nothing.
    let castle = offers(details(KNIGHT, None, &[CROWN])).unwrap();
    assert_eq!((castle.artifacts.len(), castle.stacks.len()), (1, 0));
    let wild = offers(details(SHAMAN, Some(RING), &[])).unwrap();
    assert_eq!((wild.artifacts.len(), wild.stacks.len()), (0, 1));
    let elsewhere = AltarVisit { index: 1, ..altar };
    let heroes = store(details(SHAMAN, None, &[]));
    assert_eq!(
        state.altar_offers((&rules.0, &rules.1, &heroes), &elsewhere),
        Err(AltarError::Visit)
    );
}

// covers: ADV-ALT-002
#[test]
fn the_places_are_a_set_given_in_the_changes_order() {
    let places = set(&[pack(9), worn(31), pack(0), worn(2), pack(63), worn(2)]);
    assert_eq!(places.len(), 5);
    assert!(places.contains(worn(2)) && !places.contains(worn(3)));
    let order: Vec<_> = places.places().collect();
    assert_eq!(order, [worn(2), worn(31), pack(63), pack(9), pack(0)]);
    // The bits are the wire's form.
    let bits = (places.worn_bits(), places.backpack_bits());
    assert_eq!(bits, (1 << 2 | 1 << 31, 1 << 63 | 1 << 9 | 1));
    assert_eq!(AltarPlaces::from_bits(bits.0, bits.1), places);
    // Past the bits: no place.
    assert_eq!(places.with(worn(ALTAR_WORN_PLACES)), None);
    assert_eq!(places.with(pack(ALTAR_BACKPACK_PLACES)), None);
    assert_eq!(places.with(pack(usize::MAX)), None);
    assert!(AltarPlaces::NONE.is_empty() && !places.is_empty());
}

#[test]
fn the_altars_errors_read_as_sentences() {
    let texts = [
        (
            AltarError::Visit,
            "the altar's hero is not on the map or its object no altar",
        ),
        (
            AltarError::Screen,
            "the hero's faction does not sacrifice that",
        ),
        (
            AltarError::Places,
            "the places are not the hero's artifacts the altar takes",
        ),
        (AltarError::Kept(BOOK), "artifact 6 is never sacrificed"),
        (AltarError::Amount(3), "slot 3's amount is above its stack"),
        (
            AltarError::LastCreature,
            "the hero keeps its army's last creature",
        ),
        (AltarError::Nothing, "the sacrifice gives no experience"),
    ];
    for (error, text) in texts {
        assert_eq!(error.to_string(), text);
    }
}
