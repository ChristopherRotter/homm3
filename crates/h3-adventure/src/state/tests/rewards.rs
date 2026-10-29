//! Tests of the visits of increment 7's pickups, teachers and finds (E08 T62a; design §11, §13):
//! what each kind gives, asks, refuses or turns into, the marks, the knowledge and the finds
//! emptied, the choices' answers, a monster's treasure artifact, `ADV-QUIRK-22` in both profiles,
//! and what the observation shows of them.

use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ArtifactClass, ArtifactTypeId, ContentKey, HeroEffectDef, HeroParamsDef, SpellId,
};
use h3_heroes::{ArtifactInstance, HeroChange, HeroDetails, PrimarySkill, ScholarGift};

use super::draws::{object, rules_with};
use super::*;
use crate::command::{Choice, ChoiceKind, HeroGains, Pick, UniversityVisit};
use crate::observe::{Known, KnownObject};
use crate::ruleset::{
    FindContentDef, FindDef, GiftDef, Gives, ObjectKindDef, ObjectTypeDef, OnceBonus, TurnDef,
    Visit,
};
use crate::setup::Holds;
use crate::state::Resolved;
use crate::testing::{
    LOGISTICS, PATHFINDING, PEASANT, PIKEMAN, hero_at, hero_details, key, started, world,
};

pub(super) type Rules = (Ruleset, AdventureRuleset);

pub(super) const RED: PlayerId = PlayerId::new(0);
pub(super) const BLUE: PlayerId = PlayerId::new(1);
pub(super) const HERO: HeroId = HeroId::new(1);
/// The tile every object of these tests stands on: beside the hero's (1, 1).
pub(super) const AT: Pos = Pos::new(2, 0, 0);

pub(super) const RING: ArtifactTypeId = ArtifactTypeId::from_raw(0);
pub(super) const CHARM: ArtifactTypeId = ArtifactTypeId::from_raw(1);
pub(super) const BOOK: ArtifactTypeId = ArtifactTypeId::from_raw(6);
const SCROLL: ArtifactTypeId = ArtifactTypeId::from_raw(7);
pub(super) const GUST: SpellId = SpellId::from_raw(0);
pub(super) const DOOM: SpellId = SpellId::from_raw(6);
pub(super) const WOOD: ResourceId = ResourceId::from_raw(0);
pub(super) const GOLD: ResourceId = ResourceId::from_raw(1);

/// The draws' rules (`t:faithful` with `ADV-QUIRK-21` and `ADV-QUIRK-22` on, or `t:strict`) with
/// the book a spell book, hero parameters (a backpack of two, spells to level 2 without Wisdom,
/// level 2 at 1 000 experience) and the gifts, finds and the scroll the draws do not use.
pub(super) fn rules(faithful: bool) -> Rules {
    rules_with(
        faithful,
        |core| {
            core.profiles[0]
                .quirks
                .push(("quirk.adventure.full_backpack_refuses".into(), true));
            core.artifacts[6].effects = vec![HeroEffectDef::SpellBook];
            let mut scroll = core.artifacts[6].clone();
            scroll.key = key("t:spell_scroll");
            scroll.effects = vec![HeroEffectDef::Scroll];
            core.artifacts.push(scroll);
            core.hero_params = Some(HeroParamsDef {
                primary_min: [0, 0, 1, 1],
                primary_max: 99,
                backpack: 2,
                learn_level: 2,
                experience: vec![0, 1000, 2000],
                max_level: 74,
                wisdom: key("t:logistics"),
                schools: vec![],
                mana_ceiling: 999,
            });
        },
        |types| {
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind,
            };
            let gift = |gives, prices: Vec<Vec<(ContentKey, u32)>>, admits, leave| {
                ObjectKindDef::Gift(GiftDef {
                    gives,
                    prices,
                    admits,
                    leave,
                })
            };
            let skills = |a, b| [a, b];
            types.extend([
                ty(
                    "t:marletto",
                    gift(Gives::Primary([0, 1, 0, 0]), vec![], None, false),
                ),
                ty(
                    "t:learning",
                    gift(Gives::Experience(1000), vec![], None, false),
                ),
                ty(
                    "t:library",
                    gift(Gives::Primary([2, 2, 2, 2]), vec![], Some(10), false),
                ),
                ty(
                    "t:arena",
                    gift(
                        Gives::Choice {
                            skills: skills(PrimarySkill::Attack, PrimarySkill::Defense),
                            amount: 2,
                        },
                        vec![],
                        None,
                        false,
                    ),
                ),
                ty(
                    "t:school",
                    gift(
                        Gives::Choice {
                            skills: skills(PrimarySkill::Power, PrimarySkill::Knowledge),
                            amount: 1,
                        },
                        vec![vec![(key("t:gold"), 1000)]],
                        None,
                        true,
                    ),
                ),
                ty(
                    "t:tomb",
                    ObjectKindDef::Find(FindDef {
                        draws: vec![(100, FindContentDef::Artifact(vec![ArtifactClass::Treasure]))],
                        full_backpack: Some(TurnDef::Stays),
                        question: true,
                        bonus: Some(OnceBonus {
                            morale: -3,
                            luck: 0,
                            mark: 0,
                        }),
                        leaves: false,
                    }),
                ),
                ty(
                    "t:lean_to",
                    ObjectKindDef::Find(FindDef {
                        draws: vec![(
                            100,
                            FindContentDef::Resources {
                                amount: [1, 5],
                                resources: vec![key("t:wood")],
                            },
                        )],
                        full_backpack: None,
                        question: false,
                        bonus: None,
                        leaves: false,
                    }),
                ),
                ty("t:scroll", ObjectKindDef::Scroll),
                // E08 T62b1 (`records.rs`).
                ty("t:box", ObjectKindDef::PandoraBox),
                // E08 T62b2 (`events.rs`).
                ty("t:event", ObjectKindDef::Event),
                // E08 T72: a find of fixed resources with an artifact beside them and one whose
                // artifact a full backpack loses, both staying on the map (the sea's leave and are
                // visited from a boat, T74).
                ty(
                    "t:hoard",
                    ObjectKindDef::Find(FindDef {
                        draws: vec![(
                            100,
                            FindContentDef::Fixed {
                                resources: vec![(key("t:gold"), 1000)],
                                artifact: Some(vec![ArtifactClass::Treasure]),
                            },
                        )],
                        full_backpack: Some(TurnDef::Give(vec![(key("t:gold"), 1500)])),
                        question: false,
                        bonus: None,
                        leaves: false,
                    }),
                ),
                ty(
                    "t:castaway",
                    ObjectKindDef::Find(FindDef {
                        draws: vec![(100, FindContentDef::Artifact(vec![ArtifactClass::Treasure]))],
                        full_backpack: Some(TurnDef::Lost),
                        question: false,
                        bonus: None,
                        leaves: false,
                    }),
                ),
            ]);
            // E08 T74a (`sea_pickups.rs`): Flotsam and the Survivor, which leave, as the base
            // pack's (one entry each, the test's draw set by hand; the Sea Chest is `draws.rs`'s),
            // and two messages.
            let sea = |draws, full_backpack| {
                ObjectKindDef::Find(FindDef {
                    draws: vec![(100, draws)],
                    full_backpack,
                    question: false,
                    bonus: None,
                    leaves: true,
                })
            };
            let fixed = FindContentDef::Fixed {
                resources: vec![(key("t:wood"), 5), (key("t:gold"), 200)],
                artifact: None,
            };
            types.extend([
                ty("t:flotsam", sea(fixed, None)),
                ty(
                    "t:survivor",
                    sea(
                        FindContentDef::Artifact(vec![ArtifactClass::Treasure]),
                        Some(TurnDef::Lost),
                    ),
                ),
                ty("t:bottle", ObjectKindDef::Message { leaves: true }),
                ty("t:sign", ObjectKindDef::Message { leaves: false }),
            ]);
        },
    )
}

/// The object type of key `k`.
pub(super) fn type_of(rules: &Rules, k: &str) -> ObjectTypeId {
    rules.1.object_type_by_key(k).unwrap()
}

/// A started 16 × 2 grass map with these objects (each [`object`] on the top row), red's hero 1
/// at (1, 1) and blue's hero 2 at (14, 1).
pub(super) fn map_of(rules: &Rules, objects: Vec<ObjectSetup>) -> AdventureState {
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![
            hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(14, 1, 0), &[PEASANT]),
        ],
    );
    setup.map.objects.extend(objects);
    started(rules, &setup).unwrap()
}

/// [`map_of`] with one object of type `k` holding `holds` at [`AT`].
pub(super) fn one(rules: &Rules, k: &str, holds: Holds) -> AdventureState {
    map_of(rules, vec![object(rules, AT.x, k, holds)])
}

/// Sets what the build drew for the object at `index`.
pub(super) fn draw(
    state: &mut AdventureState,
    index: usize,
    resolved: Resolved,
    contents: &[(ResourceId, u32)],
) {
    let object = &mut Arc::make_mut(&mut state.map).objects[index];
    object.resolved = resolved;
    object.contents = contents.to_vec();
}

/// Hero 1's details: one free finger, nothing else.
pub(super) fn hero() -> HeroDetails {
    HeroDetails {
        worn: vec![None],
        ..hero_details(&[])
    }
}

/// `details` wearing the spell book.
pub(super) fn with_book(details: HeroDetails) -> HeroDetails {
    HeroDetails {
        worn: vec![Some(BOOK.into())],
        ..details
    }
}

/// `details` with a full backpack (two charms) and its finger free.
pub(super) fn full(details: HeroDetails) -> HeroDetails {
    HeroDetails {
        backpack: vec![CHARM.into(), CHARM.into()],
        ..details
    }
}

/// The store of heroes with hero 1's details `details`.
pub(super) fn store(details: HeroDetails) -> Vec<Arc<HeroDetails>> {
    vec![
        Arc::new(hero_details(&[])),
        Arc::new(details),
        Arc::new(hero_details(&[])),
    ]
}

/// Red's hero steps toward [`AT`], visiting the object there.
pub(super) fn visit(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
) -> AdventureOutcome {
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![AT],
    };
    state
        .apply(&rules.0, &rules.1, heroes, RED, &command, &mut Vec::new())
        .unwrap()
}

/// The outcome giving hero 1 these changes.
pub(super) fn gives(changes: &[HeroChange]) -> AdventureOutcome {
    AdventureOutcome {
        hero: Some(HeroGains {
            hero: HERO,
            changes: changes.to_vec(),
            experience: 0,
        }),
        ..AdventureOutcome::default()
    }
}

/// The outcome giving hero 1 this experience.
fn experience(amount: u32) -> AdventureOutcome {
    AdventureOutcome {
        hero: Some(HeroGains {
            hero: HERO,
            changes: vec![],
            experience: amount,
        }),
        ..AdventureOutcome::default()
    }
}

/// The outcome of these gains.
fn gains(gains: &[(ResourceId, u32)]) -> AdventureOutcome {
    AdventureOutcome {
        gains: gains.to_vec(),
        ..AdventureOutcome::default()
    }
}

pub(super) fn add(artifact: ArtifactTypeId) -> HeroChange {
    HeroChange::AddArtifact(artifact.into())
}

// covers: ADV-PIK-005
#[test]
fn a_chest_gives_its_artifact_or_the_choice_between_its_gold_and_the_experience() {
    for faithful in [true, false] {
        let rules = rules(faithful);
        let chest = type_of(&rules, "t:chest");
        // An artifact the hero receives: given, the chest gone.
        let mut s = one(&rules, "t:chest", Holds::Nothing);
        draw(&mut s, 0, Resolved::Artifact(RING), &[]);
        assert_eq!(visit(&rules, &mut s, &store(hero())), gives(&[add(RING)]));
        assert!(s.is_removed(0));
        // Gold: the choice, the experience the gold less 500 (no Learning); gone either way.
        let choice = |gold, experience| AdventureOutcome {
            choice: Some(Choice {
                hero: HERO,
                object_type: chest,
                anchor: AT,
                kind: ChoiceKind::Chest {
                    gold: (GOLD, gold),
                    experience,
                },
            }),
            ..AdventureOutcome::default()
        };
        let mut s = one(&rules, "t:chest", Holds::Nothing);
        draw(&mut s, 0, Resolved::Nothing, &[(GOLD, 1500)]);
        assert_eq!(visit(&rules, &mut s, &store(hero())), choice(1500, 1000));
        assert!(s.is_removed(0));
        // A full backpack (ADV-QUIRK-22): under the flag the artifact turns into the 1 000 gold's
        // choice even with the finger free; without it the finger takes it.
        let mut s = one(&rules, "t:chest", Holds::Nothing);
        draw(&mut s, 0, Resolved::Artifact(RING), &[]);
        let outcome = visit(&rules, &mut s, &store(full(hero())));
        if faithful {
            assert_eq!(outcome, choice(1000, 500));
        } else {
            assert_eq!(outcome, gives(&[add(RING)]));
        }
        // No room at all, or an artifact draw that found none: the 1 000 gold's choice; a hero
        // without details gets no experience.
        let mut s = one(&rules, "t:chest", Holds::Nothing);
        draw(&mut s, 0, Resolved::Artifact(RING), &[]);
        let worn = HeroDetails {
            worn: vec![Some(BOOK.into())],
            ..full(hero())
        };
        assert_eq!(visit(&rules, &mut s, &store(worn)), choice(1000, 500));
        let mut s = one(&rules, "t:chest", Holds::Nothing);
        assert_eq!(visit(&rules, &mut s, &[]), choice(1000, 0));
    }
}

// covers: ADV-PIK-005, ADV-TCH-001, ADV-TCH-002
#[test]
fn a_choice_gives_what_its_pick_names() {
    let rules = rules(true);
    let chest = Choice {
        hero: HERO,
        object_type: type_of(&rules, "t:chest"),
        anchor: AT,
        kind: ChoiceKind::Chest {
            gold: (GOLD, 2000),
            experience: 1500,
        },
    };
    // The chest's gold or experience; it cannot be left.
    let mut s = one(&rules, "t:arena", Holds::Nothing);
    assert_eq!(s.choose(&chest, Pick::First), gains(&[(GOLD, 2000)]));
    assert_eq!(s.choose(&chest, Pick::Second), experience(1500));
    assert_eq!(s.choose(&chest, Pick::Leave), AdventureOutcome::default());
    assert!(chest.allows(Pick::First) && chest.allows(Pick::Second) && !chest.allows(Pick::Leave));
    assert_eq!(
        (chest.default_pick(), chest.price()),
        (Pick::First, &[][..])
    );
    // The Arena's: its two skills, each marking the hero.
    let arena = match visit(&rules, &mut s, &store(hero())).choice {
        Some(choice) => choice,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        arena.kind,
        ChoiceKind::Primary {
            skills: [PrimarySkill::Attack, PrimarySkill::Defense],
            amount: 2,
            price: vec![],
            leave: false,
        }
    );
    assert!(!s.is_marked(HERO, 0), "no mark before the answer");
    assert_eq!(
        (arena.default_pick(), arena.allows(Pick::Leave)),
        (Pick::First, false)
    );
    let mut other = s.clone();
    assert_eq!(
        s.choose(&arena, Pick::Second),
        gives(&[HeroChange::GainPrimary([0, 2, 0, 0])])
    );
    assert!(s.is_marked(HERO, 0));
    assert_eq!(
        other.choose(&arena, Pick::First),
        gives(&[HeroChange::GainPrimary([2, 0, 0, 0])])
    );
    // Marked: the next visit is refused.
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    // A School's: for its price, or left without a mark.
    let mut s = one(&rules, "t:school", Holds::Nothing);
    let school = visit(&rules, &mut s, &store(hero())).choice.unwrap();
    assert_eq!(school.price(), &[(GOLD, 1000)][..]);
    assert_eq!(
        (school.default_pick(), school.allows(Pick::Leave)),
        (Pick::Leave, true)
    );
    assert_eq!(s.choose(&school, Pick::Leave), AdventureOutcome::default());
    assert!(!s.is_marked(HERO, 0));
    assert_eq!(
        s.clone().choose(&school, Pick::Second),
        gives(&[HeroChange::GainPrimary([0, 0, 0, 1])])
    );
    // A choice whose object is gone gives nothing.
    let gone = Choice {
        anchor: Pos::new(9, 0, 0),
        ..school
    };
    assert_eq!(s.choose(&gone, Pick::First), AdventureOutcome::default());
}

// covers: ADV-PIK-006, ADV-PIK-008, ADV-OBJ-005
#[test]
fn an_artifact_or_a_scroll_is_picked_up_unless_the_hero_cannot_receive_it() {
    for faithful in [true, false] {
        let rules = rules(faithful);
        // A placed artifact and a scroll, as the map holds them.
        let scroll = ArtifactInstance::scroll(SCROLL, GUST);
        for (k, item) in [("t:artifact", RING.into()), ("t:scroll", scroll)] {
            let mut s = one(&rules, k, Holds::Artifact(item));
            assert_eq!(
                visit(&rules, &mut s, &store(hero())),
                gives(&[HeroChange::AddArtifact(item)])
            );
            assert!(s.is_removed(0));
            // A full backpack refuses under the flag (the object stays), not without it.
            let mut s = one(&rules, k, Holds::Artifact(item));
            let outcome = visit(&rules, &mut s, &store(full(hero())));
            assert_eq!(outcome == AdventureOutcome::default(), faithful, "{k}");
            assert_eq!(s.is_removed(0), !faithful);
        }
        // A random artifact: its draw; one whose draw found none leaves the map, giving nothing.
        let mut s = one(&rules, "t:random_treasure", Holds::Nothing);
        draw(&mut s, 0, Resolved::Artifact(CHARM), &[]);
        assert_eq!(visit(&rules, &mut s, &store(hero())), gives(&[add(CHARM)]));
        let mut s = one(&rules, "t:random_treasure", Holds::Nothing);
        draw(&mut s, 0, Resolved::Nothing, &[]);
        assert_eq!(
            visit(&rules, &mut s, &store(hero())),
            AdventureOutcome::default()
        );
        assert!(s.is_removed(0));
        // No room at all, in either profile; a hero without details receives nothing.
        let mut s = one(&rules, "t:artifact", Holds::Artifact(RING.into()));
        let worn = HeroDetails {
            worn: vec![Some(BOOK.into())],
            ..full(hero())
        };
        assert_eq!(
            visit(&rules, &mut s, &store(worn)),
            AdventureOutcome::default()
        );
        assert_eq!(visit(&rules, &mut s, &[]), AdventureOutcome::default());
        assert!(!s.is_removed(0));
    }
}

/// [`one`] with the object guarded by three pikemen, asking first or not.
fn guarded(rules: &Rules, k: &str, item: ArtifactInstance, question: bool) -> AdventureState {
    map_of(
        rules,
        vec![ObjectSetup {
            guards: Some(Guards {
                army: vec![ArmyStack {
                    slot: 0,
                    creature: PIKEMAN,
                    count: 3,
                }],
                question,
            }),
            ..object(rules, AT.x, k, Holds::Artifact(item))
        }],
    )
}

// covers: ADV-PIK-007, ADV-PIK-008, ADV-PIK-004
#[test]
fn guards_ask_or_fight_and_a_win_gives_the_artifact() {
    let rules = rules(true);
    let parts = (&rules.0, &rules.1, &store(hero())[..]);
    // An artifact's guards ask (D-29), a scroll's without a text fight at once.
    let mut s = guarded(&rules, "t:artifact", RING.into(), true);
    let asked = visit(&rules, &mut s, parts.2);
    let offer = asked.offer.clone().unwrap();
    assert_eq!(offer.kind, OfferKind::Fight);
    let mut events = Vec::new();
    let fought = s.accept_offer(parts, RED, &offer, &mut events);
    assert!(fought.battle.is_some());
    let mut scroll = guarded(
        &rules,
        "t:scroll",
        ArtifactInstance::scroll(SCROLL, GUST),
        false,
    );
    assert!(visit(&rules, &mut scroll, parts.2).battle.is_some());
    // A full backpack refuses before the question.
    let mut s2 = guarded(&rules, "t:artifact", RING.into(), true);
    assert_eq!(
        visit(&rules, &mut s2, &store(full(hero()))),
        AdventureOutcome::default()
    );
    // Lost: the guards' survivors stay; won: the hero gets it and it leaves the map.
    let left = [ArmyStack {
        slot: 0,
        creature: PIKEMAN,
        count: 1,
    }];
    let mut lost = s.clone();
    let ended = lost.end_unowned_battle(parts, (HERO, AT), (&left, false), &mut events);
    assert_eq!(ended, AdventureOutcome::default());
    assert_eq!(lost.guards_left(0), &left[..]);
    // Won: the artifact after the winner's levels (E08 T63b, GAM-DEC-007: `give_won`).
    let ended = s.end_unowned_battle(parts, (HERO, AT), (&[], true), &mut events);
    assert_eq!(
        (ended.reward_after, s.is_removed(0)),
        (Some((HERO, AT)), false)
    );
    let won = s.give_won(parts, (HERO, AT), &mut events);
    assert_eq!(won, gives(&[add(RING)]));
    assert!(s.is_removed(0));
}

/// A monster of five pikemen at [`AT`] whose treasure is 300 gold and the ring.
fn treasured(rules: &Rules) -> AdventureState {
    map_of(
        rules,
        vec![ObjectSetup {
            object_type: Some(crate::testing::MONSTER),
            monster: Some(MonsterSetup {
                creature: Some(PIKEMAN),
                count: 5,
                disposition: 2,
                never_flees: false,
                no_growth: false,
                treasure: vec![(GOLD, 300)],
                artifact: Some(RING),
            }),
            ..object(rules, AT.x, "t:artifact", Holds::Nothing)
        }],
    )
}

// covers: ADV-MON-011
#[test]
fn a_monsters_treasure_artifact_goes_to_the_hero_with_room_else_it_is_lost() {
    for faithful in [true, false] {
        let rules = rules(faithful);
        let monster = crate::testing::MONSTER;
        let join = Offer {
            hero: HERO,
            object_type: monster,
            anchor: AT,
            kind: OfferKind::Join,
        };
        let treasure = |changes: &[HeroChange]| AdventureOutcome {
            gains: vec![(GOLD, 300)],
            hero: (!changes.is_empty()).then(|| HeroGains {
                hero: HERO,
                changes: changes.to_vec(),
                experience: 0,
            }),
            ..AdventureOutcome::default()
        };
        // Joined: the artifact, then the gold.
        let heroes = store(hero());
        let parts = (&rules.0, &rules.1, &heroes[..]);
        let mut s = treasured(&rules);
        assert_eq!(
            s.accept_offer(parts, RED, &join, &mut Vec::new()),
            treasure(&[add(RING)])
        );
        // Beaten: the same; a full backpack loses the artifact under the flag.
        let mut s = treasured(&rules);
        let won = s.end_unowned_battle(parts, (HERO, AT), (&[], true), &mut Vec::new());
        assert_eq!(won, treasure(&[add(RING)]));
        let heroes = store(full(hero()));
        let parts = (&rules.0, &rules.1, &heroes[..]);
        let mut s = treasured(&rules);
        let won = s.end_monster_battle(parts, (HERO, AT), (&[], true), &mut Vec::new());
        let kept = if faithful {
            treasure(&[])
        } else {
            treasure(&[add(RING)])
        };
        assert_eq!(won, kept);
        // A draw gives none of it.
        let mut s = treasured(&rules);
        let drawn = s.end_monster_battle(parts, (HERO, AT), (&[], false), &mut Vec::new());
        assert_eq!(drawn, AdventureOutcome::default());
        assert!(s.is_removed(0));
    }
}

// covers: ADV-TCH-001, ADV-TCH-003, ADV-VIS-006
#[test]
fn a_once_per_hero_gift_gives_once_and_marks_the_hero() {
    let rules = rules(true);
    // Marletto Tower: +1 Defense, then refused for that hero, not for another.
    let mut s = one(&rules, "t:marletto", Holds::Nothing);
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        gives(&[HeroChange::GainPrimary([0, 1, 0, 0])])
    );
    assert!(s.is_marked(HERO, 0) && !s.is_marked(HeroId::new(2), 0));
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    // The Learning Stone: 1 000 experience times the hero's Learning (none: 1 000); a hero
    // without details gets the mark and nothing.
    let mut s = one(&rules, "t:learning", Holds::Nothing);
    assert_eq!(visit(&rules, &mut s, &store(hero())), experience(1000));
    let mut s = one(&rules, "t:learning", Holds::Nothing);
    assert_eq!(visit(&rules, &mut s, &[]), experience(0));
    assert!(s.is_marked(HERO, 0));
    // The Library: refused below level 10 (no mark), given at 10.
    let mut s = one(&rules, "t:library", Holds::Nothing);
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    assert!(!s.is_marked(HERO, 0));
    let tenth = HeroDetails {
        level: 10,
        ..hero()
    };
    assert_eq!(
        visit(&rules, &mut s, &store(tenth)),
        gives(&[HeroChange::GainPrimary([2, 2, 2, 2])])
    );
    // A free Tree: its level at once, without Learning (level 1 to 2: 1 000); nothing known to
    // tell of a free gift's one price, but a drawn price is known.
    let tree = type_of(&rules, "t:tree");
    let mut s = one(&rules, "t:tree", Holds::Nothing);
    draw(&mut s, 0, Resolved::Price(0), &[]);
    assert_eq!(visit(&rules, &mut s, &store(hero())), experience(1000));
    assert!(s.knows(RED, 0) && !s.knows(BLUE, 0));
    assert_eq!(
        s.observe(&rules.0, &rules.1, RED).known,
        [KnownObject {
            object_type: tree,
            anchor: AT,
            known: Known::Price(vec![]),
        }]
    );
    assert!(s.observe(&rules.0, &rules.1, BLUE).known.is_empty());
    let mut s = one(&rules, "t:marletto", Holds::Nothing);
    visit(&rules, &mut s, &store(hero()));
    assert!(!s.knows(RED, 0));
}

// covers: ADV-TCH-003
#[test]
fn a_gift_with_a_price_asks_to_pay_and_gives_once_accepted() {
    let rules = rules(true);
    let heroes = store(hero());
    let parts = (&rules.0, &rules.1, &heroes[..]);
    // The Tree's drawn 2 000 gold: the offer, known before it; accepted, the level and the mark.
    let tree = type_of(&rules, "t:tree");
    let mut s = one(&rules, "t:tree", Holds::Nothing);
    draw(&mut s, 0, Resolved::Price(1), &[]);
    let offer = Offer {
        hero: HERO,
        object_type: tree,
        anchor: AT,
        kind: OfferKind::Gift {
            price: vec![(GOLD, 2000)],
        },
    };
    let asked = visit(&rules, &mut s, &heroes);
    assert_eq!(asked.offer.as_ref(), Some(&offer));
    assert_eq!(offer.price(), &[(GOLD, 2000)][..]);
    let join = Offer {
        kind: OfferKind::JoinForGold {
            price: vec![(GOLD, 5)],
        },
        ..offer.clone()
    };
    assert_eq!(join.price(), &[(GOLD, 5)][..]);
    assert!(s.knows(RED, 0) && !s.is_marked(HERO, 0));
    let mut declined = s.clone();
    assert_eq!(
        declined.decline_offer(parts, &offer, &mut Vec::new()),
        AdventureOutcome::default()
    );
    assert!(!declined.is_marked(HERO, 0));
    assert_eq!(
        s.accept_offer(parts, RED, &offer, &mut Vec::new()),
        experience(1000)
    );
    assert!(s.is_marked(HERO, 0));
    // A gift of one price (the stone's 10 gold) asks too.
    let mut s = one(&rules, "t:stone", Holds::Nothing);
    let asked = visit(&rules, &mut s, &heroes).offer.unwrap();
    assert_eq!(asked.price(), &[(GOLD, 10)][..]);
    assert!(
        !s.knows(RED, 0),
        "one price is the type's, nothing drawn to know"
    );
    // An offer naming no gift, or a gone object, gives nothing.
    let chart = Offer {
        object_type: crate::testing::CHART,
        ..offer.clone()
    };
    let gone = Offer {
        anchor: Pos::new(9, 0, 0),
        ..offer
    };
    for stray in [chart, gone] {
        assert_eq!(
            s.accept_offer(parts, RED, &stray, &mut Vec::new()),
            AdventureOutcome::default()
        );
    }
}

// covers: ADV-TCH-004, ADV-VIS-006
#[test]
fn a_shrine_teaches_its_spell_to_a_hero_who_can_learn_it() {
    let rules = rules(true);
    let shrine = type_of(&rules, "t:shrine");
    let mut s = one(&rules, "t:shrine", Holds::Spell(None));
    draw(&mut s, 0, Resolved::Spell(GUST), &[]);
    // No spell book: refused, the spell known all the same; every visit.
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    assert!(s.knows(RED, 0));
    let book = with_book(hero());
    assert_eq!(
        visit(&rules, &mut s, &store(book.clone())),
        gives(&[HeroChange::LearnSpell(GUST)])
    );
    let known = HeroDetails {
        spells: vec![GUST],
        ..book.clone()
    };
    assert_eq!(
        visit(&rules, &mut s, &store(known)),
        AdventureOutcome::default()
    );
    assert_eq!(
        s.observe(&rules.0, &rules.1, RED).known,
        [KnownObject {
            object_type: shrine,
            anchor: AT,
            known: Known::Spell(Some(GUST)),
        }]
    );
    // Above the hero's level (5 against 2), or no spell to give: refused.
    draw(&mut s, 0, Resolved::Spell(DOOM), &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(book.clone())),
        AdventureOutcome::default()
    );
    draw(&mut s, 0, Resolved::Nothing, &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(book)),
        AdventureOutcome::default()
    );
    assert_eq!(
        s.observe(&rules.0, &rules.1, RED).known[0].known,
        Known::Spell(None)
    );
}

// covers: ADV-TCH-005, ADV-VIS-006
#[test]
fn a_witch_hut_teaches_its_skill_at_basic() {
    let rules = rules(true);
    let mut s = one(&rules, "t:hut", Holds::Skills(None));
    draw(&mut s, 0, Resolved::Skill(LOGISTICS), &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        gives(&[HeroChange::RaiseSkill {
            skill: LOGISTICS,
            level: 1,
        }])
    );
    // Known: refused, every visit; the skill known to the visitor's owner.
    let known = HeroDetails {
        skills: vec![(LOGISTICS, 1)],
        ..hero()
    };
    assert_eq!(
        visit(&rules, &mut s, &store(known)),
        AdventureOutcome::default()
    );
    assert_eq!(
        s.observe(&rules.0, &rules.1, RED).known[0].known,
        Known::Skill(Some(LOGISTICS))
    );
    draw(&mut s, 0, Resolved::Nothing, &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    assert_eq!(
        s.observe(&rules.0, &rules.1, RED).known[0].known,
        Known::Skill(None)
    );
}

// covers: ADV-TCH-006
#[test]
fn a_scholar_gives_its_gift_or_its_fallback_and_leaves() {
    let rules = rules(true);
    let mut s = one(&rules, "t:scholar", Holds::Gift(None));
    draw(
        &mut s,
        0,
        Resolved::Scholar(ScholarGift::Skill(PATHFINDING), PrimarySkill::Power),
        &[],
    );
    let mut other = s.clone();
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        gives(&[HeroChange::RaiseSkill {
            skill: PATHFINDING,
            level: 1,
        }])
    );
    assert!(s.is_removed(0));
    // A hero without details gets nothing; the Scholar leaves all the same.
    assert_eq!(visit(&rules, &mut other, &[]), AdventureOutcome::default());
    assert!(other.is_removed(0));
}

// covers: ADV-TCH-007, ADV-VIS-006
#[test]
fn a_university_opens_over_its_skills_at_its_price() {
    let rules = rules(true);
    let university = type_of(&rules, "t:university");
    let mut s = one(&rules, "t:university", Holds::Nothing);
    draw(
        &mut s,
        0,
        Resolved::Skills(vec![PATHFINDING, LOGISTICS]),
        &[],
    );
    let opened = visit(&rules, &mut s, &store(hero()));
    assert_eq!(
        opened.university,
        Some(UniversityVisit {
            hero: HERO,
            object_type: university,
            anchor: AT,
            skills: vec![PATHFINDING, LOGISTICS],
            price: vec![(GOLD, 2000)],
        })
    );
    assert_eq!(
        s.observe(&rules.0, &rules.1, RED).known[0].known,
        Known::Skills(vec![PATHFINDING, LOGISTICS])
    );
    assert_eq!(Resolved::Nothing.skills(), &[][..]);
    assert_eq!(Resolved::Skills(vec![LOGISTICS]).artifact(), None);
}

// covers: ADV-PIK-009, ADV-PIK-010, ADV-PIK-012, ADV-VIS-006
#[test]
fn a_find_gives_its_contents_once_and_turns_an_artifact_the_hero_cannot_receive() {
    let rules = rules(true);
    let corpse = type_of(&rules, "t:corpse");
    // The Corpse: its artifact, then emptied; known to its owner as emptied.
    let mut s = one(&rules, "t:corpse", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[]);
    assert_eq!(visit(&rules, &mut s, &store(hero())), gives(&[add(RING)]));
    assert!(s.is_emptied(0));
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    let known = KnownObject {
        object_type: corpse,
        anchor: AT,
        known: Known::Emptied(true),
    };
    assert_eq!(s.observe(&rules.0, &rules.1, RED).known, [known]);
    // A full backpack: its 1 000 gold instead; the Wagon's drawn resources; an empty one gives
    // nothing and is emptied.
    let mut s = one(&rules, "t:corpse", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(full(hero()))),
        gains(&[(GOLD, 1000)])
    );
    let mut s = one(&rules, "t:wagon", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[(WOOD, 4)]);
    assert_eq!(
        visit(&rules, &mut s, &store(full(hero()))),
        gains(&[(WOOD, 4)])
    );
    let mut s = one(&rules, "t:corpse", Holds::Nothing);
    draw(&mut s, 0, Resolved::Nothing, &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
    assert!(s.is_emptied(0));
    // The Lean To: its resources to the owner, once.
    let mut s = one(&rules, "t:lean_to", Holds::Nothing);
    draw(&mut s, 0, Resolved::Nothing, &[(WOOD, 3)]);
    assert_eq!(visit(&rules, &mut s, &store(hero())), gains(&[(WOOD, 3)]));
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome::default()
    );
}

// covers: ADV-PIK-015, ADV-PIK-016
#[test]
fn a_fixed_entry_gives_its_resources_beside_its_artifact_and_a_lost_one_empties_the_find() {
    // E08 T72: the Sea Chest's shape on land: the artifact and the 1 000 gold beside it, or with
    // a full backpack 1 500 gold instead of both; the Survivor's: nothing, and emptied.
    let rules = rules(true);
    let mut s = one(&rules, "t:hoard", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[(GOLD, 1000)]);
    assert_eq!(
        visit(&rules, &mut s, &store(hero())),
        AdventureOutcome {
            gains: vec![(GOLD, 1000)],
            ..gives(&[add(RING)])
        }
    );
    assert!(s.is_emptied(0));
    let mut s = one(&rules, "t:hoard", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[(GOLD, 1000)]);
    assert_eq!(
        visit(&rules, &mut s, &store(full(hero()))),
        gains(&[(GOLD, 1500)])
    );
    let mut s = one(&rules, "t:castaway", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[]);
    assert_eq!(
        visit(&rules, &mut s, &store(full(hero()))),
        AdventureOutcome::default()
    );
    assert!(s.is_emptied(0));
    let mut s = one(&rules, "t:castaway", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[]);
    assert_eq!(visit(&rules, &mut s, &store(hero())), gives(&[add(RING)]));
}

// covers: ADV-PIK-011, HER-ADV-008
#[test]
fn the_tomb_asks_then_gives_its_artifact_and_costs_morale_whatever_it_held() {
    let rules = rules(true);
    let heroes = store(hero());
    let parts = (&rules.0, &rules.1, &heroes[..]);
    let tomb = type_of(&rules, "t:tomb");
    let mut s = one(&rules, "t:tomb", Holds::Nothing);
    draw(&mut s, 0, Resolved::Artifact(RING), &[]);
    let search = Offer {
        hero: HERO,
        object_type: tomb,
        anchor: AT,
        kind: OfferKind::Search,
    };
    assert_eq!(visit(&rules, &mut s, &heroes).offer, Some(search.clone()));
    assert_eq!(search.price(), &[][..]);
    assert!(!s.knows(RED, 0), "known after the answer");
    // Declined: nothing.
    let mut declined = s.clone();
    assert_eq!(
        declined.decline_offer(parts, &search, &mut Vec::new()),
        AdventureOutcome::default()
    );
    let morale = HeroChange::Bonus {
        morale: -3,
        luck: 0,
        mark: Some(0),
    };
    // A full backpack: the artifact stays (not emptied), the morale is taken.
    let heroes_full = store(full(hero()));
    let mut stays = s.clone();
    let outcome = stays.accept_offer(
        (&rules.0, &rules.1, &heroes_full),
        RED,
        &search,
        &mut Vec::new(),
    );
    assert_eq!(outcome, gives(&[morale]));
    assert!(!stays.is_emptied(0) && stays.knows(RED, 0));
    // With room: the artifact and the morale, then emptied; a searched tomb costs the morale only.
    assert_eq!(
        s.accept_offer(parts, RED, &search, &mut Vec::new()),
        gives(&[add(RING), morale])
    );
    assert!(s.is_emptied(0));
    assert_eq!(
        s.accept_offer(parts, RED, &search, &mut Vec::new()),
        gives(&[morale])
    );
    // An offer naming no find, or a gone object, gives nothing.
    let stray = Offer {
        object_type: crate::testing::CHART,
        ..search.clone()
    };
    let gone = Offer {
        anchor: Pos::new(9, 0, 0),
        ..search
    };
    for stray in [stray, gone] {
        assert_eq!(
            s.accept_offer(parts, RED, &stray, &mut Vec::new()),
            AdventureOutcome::default()
        );
    }
}

// covers: ADV-VIS-006
#[test]
fn a_hero_sees_its_own_marks_and_the_block_is_written_once_needed() {
    let rules = rules(true);
    let marletto = type_of(&rules, "t:marletto");
    let mut s = map_of(
        &rules,
        vec![
            object(&rules, AT.x, "t:marletto", Holds::Nothing),
            object(&rules, 5, "t:learning", Holds::Nothing),
        ],
    );
    // Nothing visited: no block, no marks, nothing known.
    assert!(s.visits().is_none());
    let before = s.state_hash();
    let red = s.observe(&rules.0, &rules.1, RED);
    assert!(
        red.hero(HERO)
            .unwrap()
            .own
            .as_ref()
            .unwrap()
            .marks
            .is_empty()
    );
    assert!(red.known.is_empty());
    visit(&rules, &mut s, &store(hero()));
    assert_ne!(s.state_hash(), before, "the block is hashed");
    let red = s.observe(&rules.0, &rules.1, RED);
    assert_eq!(
        red.hero(HERO).unwrap().own.as_ref().unwrap().marks,
        [(marletto, AT)]
    );
    // Another hero's marks are not this one's; another player sees none.
    let blue = s.observe(&rules.0, &rules.1, BLUE);
    assert!(
        blue.hero(HeroId::new(2))
            .unwrap()
            .own
            .as_ref()
            .unwrap()
            .marks
            .is_empty()
    );
    // A Shrine known twice writes the block once: the fork's block stays shared.
    let mut s = one(&rules, "t:shrine", Holds::Spell(None));
    visit(&rules, &mut s, &store(hero()));
    let fork = s.clone();
    visit(&rules, &mut s, &store(hero()));
    assert!(core::ptr::eq(s.visits().unwrap(), fork.visits().unwrap()));
}
