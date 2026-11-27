use super::*;

use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::game::{
    AltarWindow, Amount, Choice, ChoiceKind, HillFortState, HillFortWindow, Hire, HireSeen,
    HireTarget, LevelUp, MarketSeen, MarketWindow, ObjectRef, Offer, Pos, PrimarySkill,
    UniversityVisit,
};

const ANCHOR: Pos = Pos {
    x: 5,
    y: 4,
    level: 0,
};

const OBJECT: ObjectRef = ObjectRef {
    object_type: 1,
    anchor: ANCHOR,
};

fn offer(kind: OfferKind) -> Decision {
    Decision::Offer {
        offer: Offer {
            hero: 0,
            object_type: 1,
            anchor: ANCHOR,
            kind,
        },
    }
}

/// Every decision kind (each offer's), with its notice and its closing answer.
fn decisions() -> Vec<(Decision, Option<Notice>, Option<Answer>)> {
    let decline = Some(Answer::Offer {
        consent: Consent::Decline,
    });
    let close = Some(Answer::Close {});
    let gold = vec![Amount {
        resource: 6,
        amount: 1000,
    }];
    let asked = [
        OfferKind::Reveal {
            price: gold.clone(),
        },
        OfferKind::Pass {},
        OfferKind::Join {},
        OfferKind::JoinForGold {
            price: gold.clone(),
        },
        OfferKind::Pursue {},
        OfferKind::Gift {
            price: gold.clone(),
        },
        OfferKind::Search {},
        OfferKind::OpenBox {},
        OfferKind::SearchPyramid {},
    ];
    let mut all: Vec<_> = asked.into_iter().map(|k| (offer(k), None, None)).collect();
    all.extend([
        (offer(OfferKind::Fight {}), Some(Notice::Battle), decline),
        (
            offer(OfferKind::Guarded {
                creature: 0,
                count_word: 2,
            }),
            Some(Notice::Battle),
            decline,
        ),
        (
            offer(OfferKind::Boat {
                price: gold.clone(),
            }),
            Some(Notice::Shipyard),
            decline,
        ),
        (
            offer(OfferKind::Hire { creatures: vec![0] }),
            Some(Notice::Recruit),
            decline,
        ),
        (
            Decision::LevelUp {
                level_up: LevelUp {
                    hero: 0,
                    level: 2,
                    primary: PrimarySkill::Attack,
                    offers: [0, 1],
                },
            },
            None,
            None,
        ),
        (
            Decision::Choice {
                choice: Choice {
                    hero: 0,
                    object_type: 1,
                    anchor: ANCHOR,
                    kind: ChoiceKind::Chest {
                        gold: gold[0],
                        experience: 500,
                    },
                },
            },
            None,
            None,
        ),
        (
            Decision::Meeting { heroes: [0, 1] },
            Some(Notice::Meeting),
            close,
        ),
        (
            Decision::ArmyDialog {
                hero: 0,
                group: Vec::new(),
                mine: None,
            },
            Some(Notice::Army),
            close,
        ),
        (
            Decision::University {
                visit: UniversityVisit {
                    hero: 0,
                    object_type: 1,
                    anchor: ANCHOR,
                    skills: Vec::new(),
                    price: gold.clone(),
                },
                slots: Vec::new(),
            },
            Some(Notice::University),
            Some(Answer::University {
                answer: UniversityAnswer::Leave {},
            }),
        ),
        (
            Decision::Hire {
                hire: Hire {
                    target: HireTarget::Hero { hero: 0 },
                    source: HireSeen::Object {
                        object_type: 1,
                        anchor: ANCHOR,
                    },
                    creatures: Vec::new(),
                },
            },
            Some(Notice::Recruit),
            Some(Answer::Hire {
                answer: HireAnswer::Close {},
            }),
        ),
        (
            Decision::Exchange {
                hero: 0,
                garrison: 3,
                object: OBJECT,
                removable: true,
                army: Vec::new(),
            },
            Some(Notice::Garrison),
            close,
        ),
        (
            Decision::Tavern {
                hero: 0,
                object: OBJECT,
            },
            Some(Notice::Tavern),
            Some(Answer::Tavern {
                answer: TavernAnswer::Close {},
            }),
        ),
        (
            Decision::Market {
                market: MarketWindow {
                    hero: Some(0),
                    place: MarketSeen::Object { object: OBJECT },
                    screens: Vec::new(),
                    count: 1,
                    trades: Vec::new(),
                    sales: Vec::new(),
                    stock: Vec::new(),
                    artifact_sales: Vec::new(),
                    recipients: Vec::new(),
                },
            },
            Some(Notice::Market),
            Some(Answer::Market {
                answer: MarketAnswer::Leave {},
            }),
        ),
        (
            Decision::Altar {
                altar: AltarWindow {
                    hero: 0,
                    object: OBJECT,
                    artifacts: true,
                    creatures: false,
                    places: 18,
                    items: Vec::new(),
                    stacks: Vec::new(),
                    to_next_level: 1000,
                },
            },
            Some(Notice::Altar),
            Some(Answer::Altar {
                answer: AltarAnswer::Leave {},
            }),
        ),
        (
            Decision::HillFort {
                fort: HillFortWindow {
                    hero: 0,
                    object: OBJECT,
                    rows: Vec::new(),
                    all: Vec::new(),
                    all_state: HillFortState::NoUpgrade {},
                },
            },
            Some(Notice::HillFort),
            Some(Answer::HillFort {
                answer: HillFortAnswer::Leave {},
            }),
        ),
    ]);
    all
}

#[test]
fn each_decision_s_notice_and_its_closing_answer() {
    for (decision, notice, answer) in decisions() {
        assert_eq!(Notice::of(&decision), notice, "{decision:?}");
        assert_eq!(closing(&decision), answer, "{decision:?}");
    }
}

#[test]
fn each_notice_its_key() {
    for (notice, key) in Notice::ALL.into_iter().zip(KEYS) {
        assert_eq!(notice.key(), key);
    }
    assert_eq!(Notice::Battle.key(), "map.stub.battle");
    // The screens without a decision: none a decision's notice.
    for (decision, notice, _) in decisions() {
        assert!(
            notice.is_none_or(|n| !Notice::SCREENS.contains(&n)),
            "{decision:?}"
        );
    }
}
