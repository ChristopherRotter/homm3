//! The battle's message boxes (E17 M5 T4a; `docs/epics/E17-design.md` §7.19): what each asks
//! or tells and what its OK does, shared by the window's battle (`crate::battle`) and the capture
//! set's stills (`crate::spellbook::still`), which lay them out with `crate::log::Writer`.

use h3_content_battle::BattleRuleset;
use h3_protocol::wire::{Command, Manifest, Observation, OwnHero, Sex, Side};
use h3_render::Bugs;
use h3_ui::text::{Args, Catalogue};
use h3_view::book;
use h3_view::message::{Picture, YES};
use h3_view::tables::Tables;

/// What a message box asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Nothing: its OK alone closes it (the book's own refusal or description over the book, the
    /// surrender refused).
    Told,
    /// A refusal of the spell book button or of the aim's entry over the battle (E17 M6 T5a,
    /// design §12.17: 685, 684 before the book; 748, 537 and the entry's boxes after it): its OK
    /// alone closes it, then the button's path posts the mouse where it rests (0x474F72), as
    /// after a book closed without a spell.
    Refused,
    /// Retreat (2002, `battle_retreat_button` 0x474FA1): its OK retreats.
    Retreat,
    /// Surrender (2001, `battle_surrender_button` 0x474FE9) at its price: its OK surrenders, or
    /// tells the side it cannot pay (GENRLTXT 30) when its funds are below the price.
    Surrender {
        /// The price.
        price: u32,
        /// The side's gold.
        funds: u32,
    },
    /// The battle's result window (E17 M5 T4b1, `crate::results`): its OK (Enter or Escape)
    /// closes the battle's screen.
    Result,
    /// A siege turret's info by a left press (E17 M6 T2b2, `turret_info` 0x47A380 from
    /// 0x476E2F): its OK closes it, then the battle takes the mouse again as after a stack's
    /// stats dialog (0x476E34, `Input::Looked`).
    Turret,
    /// A turret's info by the right button (E17 M6 T2b2, 0x476BBA), or a panel control's help
    /// (E17 M6 T3a, `battle_control_help` 0x472A50), or a hero's info window (E17 M6 T3b1,
    /// `army_info_show` 0x46CDF0 with `popup_run_held` and no popup): a popup without buttons
    /// held while the button is (`popup_run_held` 0x602A40): its ends close it, the input
    /// meanwhile dropped, the cursor hidden.
    Held,
}

/// How the side to act may leave the battle by the panel's buttons: whether it may retreat (the
/// engine lists it), and its surrender's price, its gold and the other side's hero's name when it
/// may surrender at all (its own view prices it).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Leave {
    /// It may retreat.
    pub retreat: bool,
    /// Its surrender: the price, its gold, the other side's hero's name.
    pub surrender: Option<(u32, u32, String)>,
}

impl Leave {
    /// The side's ways to leave in its own view `own` with its legal commands `legal`, the
    /// heroes' names from the manifest's `hero_types` through `catalogue`.
    pub fn of(
        own: &Observation,
        legal: &[Command],
        hero_types: &[String],
        catalogue: &Catalogue,
    ) -> Self {
        let surrender = own
            .own_hero
            .as_ref()
            .and_then(|h| Some((h.surrender_price?, h.funds)))
            .map(|(price, funds)| {
                let other = 1 - usize::from(own.viewer == Side::Defender);
                let name = own.sides[other]
                    .hero
                    .as_ref()
                    .and_then(|h| hero_types.get(usize::from(h.hero_type)))
                    .map(|k| catalogue.text(&format!("hero.{k}"), &Args::new()))
                    .unwrap_or_default();
                (price, funds, name)
            });
        Self {
            retreat: legal.contains(&Command::Retreat {}),
            surrender,
        }
    }

    /// The question the panel's button `id` asks (GENRLTXT 29 to retreat; 33 with the other
    /// side's hero's name and the price to surrender) and what its OK does; none when the side
    /// may not leave so.
    pub fn ask(&self, catalogue: &Catalogue, id: u16) -> Option<(String, Asked)> {
        match id {
            h3_view::panel::RETREAT if self.retreat => Some((
                catalogue.text("battle.retreat.ask", &Args::new()),
                Asked::Retreat,
            )),
            h3_view::panel::SURRENDER => {
                let (price, funds, name) = self.surrender.as_ref()?;
                let args = Args::new()
                    .text("hero", name)
                    .number("price", i64::from(*price));
                Some((
                    catalogue.text("battle.surrender.ask", &args),
                    Asked::Surrender {
                        price: *price,
                        funds: *funds,
                    },
                ))
            }
            _ => None,
        }
    }
}

/// What a box's closing by its button `id` leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answered {
    /// Nothing more.
    Closed,
    /// The command its OK gives.
    Give(Command),
    /// Another box at once, telling this (the surrender refused, GENRLTXT 30: 0x475119).
    Tell(String),
}

/// What closing a box that asked `asked` by its button `id` leads to (0x474FD0, 0x4750E4).
pub fn answered(asked: Asked, id: u16, catalogue: &Catalogue) -> Answered {
    match asked {
        Asked::Retreat if id == YES => Answered::Give(Command::Retreat {}),
        Asked::Surrender { price, funds } if id == YES => {
            if funds >= price {
                Answered::Give(Command::Surrender {})
            } else {
                Answered::Tell(catalogue.text("battle.surrender.poor", &Args::new()))
            }
        }
        _ => Answered::Closed,
    }
}

/// The box the book's `outcome` shows over it, if any: a spell above the hero's spell points
/// (GENRLTXT 207 with the cost and the points, 0x59D462) or an adventure spell's description with
/// its picture (`spells`, its frame the spell's number, of `size`) and name (0x59D4DC).
pub fn book_box(
    outcome: book::Outcome,
    hero: &book::Hero,
    catalogue: &Catalogue,
    (spells, size): (&str, [i32; 2]),
) -> Option<(String, Option<Picture>)> {
    match outcome {
        book::Outcome::Refused { cost, mana } => {
            let args = Args::new()
                .number("cost", i64::from(cost))
                .number("mana", i64::from(mana));
            Some((catalogue.text("book.refused", &args), None))
        }
        book::Outcome::Described(handle) => {
            let spell = hero.spells.iter().find(|s| s.handle == handle)?;
            let picture = Picture {
                def: spells.to_owned(),
                frame: spell.original,
                size,
                caption: spell.name.clone(),
            };
            Some((spell.description.clone(), Some(picture)))
        }
        _ => None,
    }
}

/// The original's stack spells (15 to 19, 24 to 39, 41 to 58, 60 to 62) and Teleport (63): the
/// aim's entry tells that the spell affects no one (GENRLTXT 186) when no hex takes them
/// (`enter_player_cast` 0x59F085).
pub fn affects_someone_only(original: u32) -> bool {
    matches!(original, 15..=19 | 24..=39 | 41..=58 | 60..=63)
}

/// The Orb of Inhibition's and Recanter's Cloak's original numbers (ARTRAITS; the catalogue's
/// family `artifact`).
const ORB: u32 = 126;
const CLOAK: u32 = 83;
/// The original's numbers of the spells the aim's entry refuses on its own (E17 M6 T5a):
/// Land Mine, Earthquake, Sacrifice, Remove Obstacle, Clone.
const LAND_MINE: u32 = 11;
const EARTHQUAKE: u32 = 14;
const SACRIFICE: u32 = 40;
const REMOVE_OBSTACLE: u32 = 64;
const CLONE: u32 = 65;
/// The Summons (Fire, Earth, Water, Air Elemental) and the creatures they summon, by their
/// original numbers (`enter_player_cast` 0x59EC50's case of 66 to 69).
const SUMMONS: [(u32, u32); 4] = [(66, 114), (67, 113), (68, 115), (69, 112)];
/// The original's limit of a side's stack records (+0x54BC against 20): where the ruleset is not
/// known (a server's game of other content).
const STACK_LIMIT: u32 = 20;

/// A side's hero as the book's and the aim's refusals read it (E17 M6 T5a, design §12.17).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroFacts {
    /// Its name.
    pub name: String,
    /// It is female: its sex on the map, else its hero type's (the Summons' pronoun).
    pub female: bool,
    /// It forbids every cast (the Orb of Inhibition: its side's `OwnHero::no_casting`), as far as
    /// the seat knows it.
    pub forbids: bool,
    /// The highest spell level it allows (Recanter's Cloak: `OwnHero::level_limit`), as far as
    /// the seat knows it.
    pub level_limit: Option<u8>,
}

/// What the spell book button's and the aim's entry's refusals read (E17 M6 T5a, design §12.17):
/// the battle's facts as the side to act may know them. Local play knows every one (both sides'
/// own views); a server's seat its own hero's magic alone ([`seat_magic`]; gap G1, CLT-STUB-1:
/// the other hero's Orb and Cloak show no box until the wire carries them).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// The battle is at an anti-magic garrison (`Observation::anti_magic`).
    pub garrison: bool,
    /// The attacker's and the defender's hero.
    pub heroes: [Option<HeroFacts>; 2],
    /// The side to act (+0x132C0, the acting stack's controller): 0 the attacker, 1 the
    /// defender.
    pub side: usize,
    /// The highest spell level the battlefield allows (Cursed Ground's 1), none without a limit.
    pub field_level: Option<u8>,
    /// The other side had a stack on its native terrain at the battle's start (Land Mine's).
    pub natives: bool,
    /// The battle is a siege of a town with a fort (Earthquake's).
    pub fort: bool,
    /// The side's stack records, the dead included (as the engine's room, the original's
    /// +0x54BC).
    pub records: u32,
    /// The side's limit of stack records.
    pub limit: u32,
    /// The elemental the side's Summons are bound to: its original number and plural name.
    pub summoned: Option<(u32, String)>,
}

/// What names the facts: the record's manifest, the asset tables, the catalogue.
pub type Names<'a> = (&'a Manifest, &'a Tables, &'a Catalogue);

impl Facts {
    /// The facts of the battle the side to act sees in its own observation `own`, its opening
    /// `first` (the natives), each side's hero's private part `magic` as far as the seat knows it
    /// (local play: both sides' own views; a server's seat: its own, [`seat_magic`]) and the
    /// battle's `ruleset` when the game's content is the client's own (the field's level limit,
    /// the stack limit; else Cursed Ground by its key and the original's 20), named by `names`.
    /// The side to act is the active stack's controller (+0x132C0, as the engine's
    /// `side_to_act`: a hypnotized stack's turn is its controller's, whose hero casts), else the
    /// viewer.
    pub fn of(
        own: &Observation,
        first: &Observation,
        magic: [Option<&OwnHero>; 2],
        ruleset: Option<&BattleRuleset>,
        (manifest, tables, catalogue): Names<'_>,
    ) -> Self {
        let acting = own
            .active
            .and_then(|id| own.stacks.iter().find(|s| s.id == id))
            .map_or(own.viewer, |s| s.controller);
        let side = usize::from(acting == Side::Defender);
        let heroes = [0, 1].map(|i| {
            let hero = own.sides[i].hero.as_ref()?;
            let key = manifest.keys.hero_types.get(usize::from(hero.hero_type));
            let name = key.map_or_else(String::new, |k| {
                catalogue.text(&format!("hero.{k}"), &Args::new())
            });
            let female = match hero.sex {
                Some(sex) => sex == Sex::Female,
                None => key
                    .and_then(|k| tables.heroes.get(k))
                    .is_some_and(|h| h.female),
            };
            Some(HeroFacts {
                name,
                female,
                forbids: magic[i].is_some_and(|m| m.no_casting),
                level_limit: magic[i].and_then(|m| m.level_limit),
            })
        });
        let field_level = match ruleset {
            Some(ruleset) => own
                .battlefield
                .and_then(|b| ruleset.battlefields().get(usize::from(b)))
                .map(|b| b.max_spell_level)
                .filter(|&level| level != u8::MAX),
            None => {
                (crate::stats::field(manifest, own) == h3_view::stats::Field::Cursed).then_some(1)
            }
        };
        let limit = ruleset.map_or(STACK_LIMIT, |r| u32::from(r.battle().side_stack_limit));
        let records = own.stacks.iter().filter(|s| s.side == acting).count();
        let summoned = own.sides[side].summoned.and_then(|creature| {
            let key = manifest.keys.creatures.get(usize::from(creature))?;
            let original = u32::from(tables.creatures.get(key)?.original);
            let plural = catalogue.text(&format!("creature.{key}"), &Args::new().count(2));
            Some((original, plural))
        });
        Self {
            garrison: own.anti_magic,
            heroes,
            side,
            field_level,
            natives: (first.stacks.iter()).any(|s| s.side != acting && s.native),
            fort: own.siege.as_ref().is_some_and(|s| s.fort > 0),
            records: u32::try_from(records).unwrap_or(u32::MAX),
            limit,
            summoned,
        }
    }

    /// The acting side's hero's name (none without a hero).
    fn hero(&self) -> &str {
        self.heroes[self.side].as_ref().map_or("", |h| &h.name)
    }
}

/// Each side's hero's magic limits as a seat on a server knows them from its own view `own` (E17
/// M6 T5a, design §12.17, gap G1): its own hero's (`OwnHero`, private) alone; the other side's
/// none, so its Orb of Inhibition (684) and Recanter's Cloak (537) show no box there, where local
/// play shows them (the entry's box instead, e.g. 186 or 713, or a wall's aim with every hex
/// refused). The user (2026-11-05) requires a server's game to behave as a local one: the battle
/// and server lanes are to put each side's two facts (forbids casting, spell level limit) on the
/// wire, read here once they are.
pub fn seat_magic(own: &Observation) -> [Option<OwnHero>; 2] {
    let mut magic = [None, None];
    // stub: CLT-STUB-1 (the other side's forbids-casting and level-limit facts, once on the wire)
    magic[usize::from(own.viewer == Side::Defender)] = own.own_hero.clone();
    magic
}

/// The box the spell book button shows instead of the book (`hero_book_button` 0x59E900's tests
/// 3 and 4; E17 M6 T5a, design §12.17): an anti-magic garrison (GENRLTXT 685), else the Orb of
/// Inhibition on a hero the seat knows wears it, the attacker tested first (684 with the wearer's
/// name and the Orb's); none: the book opens.
pub fn before_book(facts: &Facts, catalogue: &Catalogue) -> Option<String> {
    if facts.garrison {
        return Some(catalogue.text("book.garrison", &Args::new()));
    }
    let wearer = facts.heroes.iter().flatten().find(|h| h.forbids)?;
    let args = Args::new()
        .text("hero", &wearer.name)
        .text("artifact", &artifact(catalogue, ORB));
    Some(catalogue.text("book.orb", &args))
}

/// The box shown once the book closed on the spell numbered `original` of `level` that the engine
/// refuses (no target, no cast without one; E17 M6 T5a, design §12.17): the button's tests after
/// the book (Cursed Ground's 748 above its level, Recanter's Cloak's 537 above a hero's limit with
/// the Cloak's name and the acting side's hero's), then the aim's entry's by the spell
/// (`enter_player_cast` 0x59EC50): Land Mine against the other side's natives 186 (oddity 1,
/// CLT-Q58: copied, *provisional*: HD or HotA may have a fix), Earthquake without a fort 183, the
/// stack spells and Teleport 186, Sacrifice 186, Remove Obstacle 713, Clone at the side's limit
/// 184 with its records (oddity 2, CLT-Q59: copied, *provisional* as Q58) else 186, the Summons
/// bound to another elemental 539 (the hero, the elemental's plural name, the pronoun 540 or 541)
/// and at the limit, in the profile `bugs`: the client's 184 with the records (CLT-Q57, fixed),
/// the original's 539 with the side's elemental, none an empty name (oddity 3). None for any
/// other spell or reason (the walls' aim opens with no free hex).
pub fn after_book(
    facts: &Facts,
    (original, level): (u32, u8),
    bugs: Bugs,
    catalogue: &Catalogue,
) -> Option<String> {
    let text = |key: &str| catalogue.text(key, &Args::new());
    if facts.field_level.is_some_and(|limit| level > limit) {
        return Some(text("aim.cursed"));
    }
    let limited =
        (facts.heroes.iter().flatten()).any(|h| h.level_limit.is_some_and(|limit| level > limit));
    if limited {
        let args = Args::new()
            .text("artifact", &artifact(catalogue, CLOAK))
            .text("hero", facts.hero());
        return Some(catalogue.text("aim.cloak", &args));
    }
    let full = facts.records >= facts.limit;
    let records = || {
        let args = Args::new().number("stacks", i64::from(facts.records));
        catalogue.text("aim.stack-limit", &args)
    };
    match original {
        LAND_MINE => facts.natives.then(|| text("aim.no-one")),
        EARTHQUAKE => (!facts.fort).then(|| text("aim.no-fort")),
        SACRIFICE => Some(text("aim.no-one")),
        REMOVE_OBSTACLE => Some(text("aim.no-obstacle")),
        CLONE if full => Some(records()),
        CLONE => Some(text("aim.no-one")),
        o if affects_someone_only(o) => Some(text("aim.no-one")),
        o => {
            let &(_, own) = SUMMONS.iter().find(|(spell, _)| *spell == o)?;
            let other = facts.summoned.as_ref().is_some_and(|(c, _)| *c != own);
            if other || (full && bugs == Bugs::Original) {
                let female = facts.heroes[facts.side].as_ref().is_some_and(|h| h.female);
                let context = if female { "female" } else { "male" };
                let pronoun = catalogue.text("word.possessive", &Args::new().context(context));
                let creature = facts.summoned.as_ref().map_or("", |(_, name)| name);
                let args = Args::new()
                    .text("hero", facts.hero())
                    .text("creature", creature)
                    .text("pronoun", &pronoun);
                Some(catalogue.text("aim.summoned", &args))
            } else {
                full.then(records)
            }
        }
    }
}

/// The name of the artifact numbered `id` in the original (ARTRAITS's first column).
fn artifact(catalogue: &Catalogue, id: u32) -> String {
    catalogue.text(&format!("artifact.{id}"), &Args::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_question_s_ok_leaves_or_tells() {
        let c = Catalogue::default();
        let leave = Leave {
            retreat: true,
            surrender: Some((300, 200, "Crag Hack".into())),
        };
        assert!(matches!(
            leave.ask(&c, h3_view::panel::RETREAT),
            Some((_, Asked::Retreat))
        ));
        assert!(matches!(
            leave.ask(&c, h3_view::panel::SURRENDER),
            Some((
                _,
                Asked::Surrender {
                    price: 300,
                    funds: 200
                }
            ))
        ));
        assert_eq!(leave.ask(&c, h3_view::panel::AUTO_COMBAT), None);
        assert_eq!(Leave::default().ask(&c, h3_view::panel::RETREAT), None);
        assert_eq!(Leave::default().ask(&c, h3_view::panel::SURRENDER), None);
        let no = h3_view::message::NO;
        assert_eq!(
            answered(Asked::Retreat, YES, &c),
            Answered::Give(Command::Retreat {})
        );
        assert_eq!(answered(Asked::Retreat, no, &c), Answered::Closed);
        let rich = Asked::Surrender {
            price: 300,
            funds: 300,
        };
        let poor = Asked::Surrender {
            price: 300,
            funds: 299,
        };
        assert_eq!(
            answered(rich, YES, &c),
            Answered::Give(Command::Surrender {})
        );
        assert!(matches!(answered(poor, YES, &c), Answered::Tell(_)));
        assert_eq!(answered(poor, no, &c), Answered::Closed);
        for told in [Asked::Told, Asked::Refused] {
            assert_eq!(answered(told, h3_view::message::OK, &c), Answered::Closed);
        }
        assert!(affects_someone_only(39) && affects_someone_only(63) && !affects_someone_only(40));
        assert!(!affects_someone_only(20) && !affects_someone_only(59));
    }

    #[test]
    fn the_terms_name_the_other_side_s_hero() {
        // E17 M5 T8b2 (R5a-5): the attacker's view names the defender's hero, the defender's the
        // attacker's (Orrin's book situation, the hero types renamed); the retreat as the engine
        // lists it.
        use h3_protocol::wire::Hero;
        let pack = r#"{ "battle": { "surrender": { "ask": "{{hero}}: {{price}}" } },
            "hero": { "orrin": "Orrin", "crag_hack": "Crag Hack" } }"#;
        let c = Catalogue {
            packs: vec![h3_ui::text::pack::Pack::from_json("en", pack).unwrap()],
        };
        let (_, mut own, _) = crate::spellbook::tests::opening();
        let types = ["orrin", "crag_hack", "nobody"].map(String::from);
        let hero = own.sides[0].hero.unwrap();
        own.sides[0].hero = Some(Hero {
            hero_type: 0,
            ..hero
        });
        own.sides[1].hero = Some(Hero {
            hero_type: 1,
            ..hero
        });
        let o = own.own_hero.as_mut().unwrap();
        (o.surrender_price, o.funds) = (Some(300), 200);
        let legal = [Command::Defend {}, Command::Retreat {}];
        let named = |name: &str| Some((300, 200, name.to_owned()));
        let attacker = Leave::of(&own, &legal, &types, &c);
        assert_eq!(
            attacker,
            Leave {
                retreat: true,
                surrender: named("Crag Hack"),
            }
        );
        let asked = attacker.ask(&c, h3_view::panel::SURRENDER).unwrap();
        assert_eq!(asked.0, "Crag Hack: 300");
        own.viewer = Side::Defender;
        let defender = Leave::of(&own, &legal[..1], &types, &c);
        assert_eq!(
            defender,
            Leave {
                retreat: false,
                surrender: named("Orrin"),
            }
        );
        assert_eq!(defender.ask(&c, h3_view::panel::RETREAT), None);
        // A hero type no pack names: its key; one the manifest lacks or none facing it: no name.
        for (hero_type, name) in [(2, "hero.nobody"), (3, "")] {
            own.sides[0].hero = Some(Hero { hero_type, ..hero });
            assert_eq!(Leave::of(&own, &[], &types, &c).surrender, named(name));
        }
        own.sides[0].hero = None;
        assert_eq!(Leave::of(&own, &[], &types, &c).surrender, named(""));
        // Not priced (or no own hero): no surrender.
        own.own_hero.as_mut().unwrap().surrender_price = None;
        assert_eq!(Leave::of(&own, &legal, &types, &c).surrender, None);
        own.own_hero = None;
        assert_eq!(
            Leave::of(&own, &legal, &types, &c),
            Leave {
                retreat: true,
                surrender: None
            }
        );
    }

    /// The refusals' texts as a test words them.
    fn refusals_catalogue() -> Catalogue {
        let pack = r#"{
            "book": { "garrison": "Garrison.", "orb": "{{hero}} wears {{artifact}}" },
            "aim": {
                "no-one": "No one.", "cursed": "Cursed.", "no-fort": "No fort.",
                "no-obstacle": "No obstacle.", "stack-limit": "{{stacks}} stacks",
                "cloak": "{{artifact}} stops {{hero}}",
                "summoned": "{{hero}}: {{creature}} ({{pronoun}})"
            },
            "word": { "possessive_male": "his", "possessive_female": "her" },
            "artifact": { "126": "Orb", "83": "Cloak" }
        }"#;
        Catalogue {
            packs: vec![h3_ui::text::pack::Pack::from_json("en", pack).unwrap()],
        }
    }

    fn hero(name: &str) -> HeroFacts {
        HeroFacts {
            name: name.into(),
            ..HeroFacts::default()
        }
    }

    #[test]
    fn the_book_button_refuses_at_a_garrison_and_for_the_orb_s_wearer() {
        // E17 M6 T5a (design §12.17): 685 first; 684 names the wearer, the attacker tested first;
        // a hero the seat does not know to wear it (a server's other side, gap G1) shows nothing.
        let c = refusals_catalogue();
        let mut facts = Facts {
            heroes: [Some(hero("Orrin")), Some(hero("Crag Hack"))],
            ..Facts::default()
        };
        assert_eq!(before_book(&facts, &c), None);
        facts.heroes[1].as_mut().unwrap().forbids = true;
        assert_eq!(before_book(&facts, &c).unwrap(), "Crag Hack wears Orb");
        facts.heroes[0].as_mut().unwrap().forbids = true;
        assert_eq!(before_book(&facts, &c).unwrap(), "Orrin wears Orb");
        facts.garrison = true;
        assert_eq!(before_book(&facts, &c).unwrap(), "Garrison.");
        assert_eq!(before_book(&Facts::default(), &c), None);
    }

    #[test]
    fn the_aim_s_refusals_follow_the_original_s_order() {
        // E17 M6 T5a (design §12.17): Cursed Ground, then the Cloak (the acting side's hero
        // named), then the aim's entry by the spell's original number.
        let c = refusals_catalogue();
        let base = Facts {
            heroes: [Some(hero("Orrin")), Some(hero("Crag Hack"))],
            side: 1,
            limit: 20,
            records: 3,
            ..Facts::default()
        };
        let box_of = |facts: &Facts, spell: (u32, u8)| after_book(facts, spell, Bugs::Fixed, &c);
        let arrow = (15, 1);
        assert_eq!(box_of(&base, arrow).unwrap(), "No one.");
        // A spell the entry does not refuse on its own (a wall): none.
        assert_eq!(box_of(&base, (12, 3)), None);
        let cursed = Facts {
            field_level: Some(1),
            ..base.clone()
        };
        assert_eq!(box_of(&cursed, (17, 2)).unwrap(), "Cursed.");
        // A level at Cursed Ground's limit passes it: spell 13 (a wall) at level 1, none.
        assert_eq!(box_of(&cursed, (13, 1)), None);
        let mut cloaked = base.clone();
        cloaked.heroes[0].as_mut().unwrap().level_limit = Some(2);
        assert_eq!(box_of(&cloaked, (17, 3)).unwrap(), "Cloak stops Crag Hack");
        assert_eq!(box_of(&cloaked, (17, 2)).unwrap(), "No one.");
        // Cursed Ground before the Cloak.
        let both = Facts {
            field_level: Some(1),
            ..cloaked
        };
        assert_eq!(box_of(&both, (17, 3)).unwrap(), "Cursed.");
        // Land Mine against the other side's natives (CLT-Q58), Earthquake without a fort.
        let natives = Facts {
            natives: true,
            ..base.clone()
        };
        assert_eq!(box_of(&natives, (11, 3)).unwrap(), "No one.");
        assert_eq!(box_of(&base, (11, 3)), None);
        assert_eq!(box_of(&base, (14, 3)).unwrap(), "No fort.");
        let fort = Facts {
            fort: true,
            ..base.clone()
        };
        assert_eq!(box_of(&fort, (14, 3)), None);
        // Sacrifice, Remove Obstacle, Teleport.
        assert_eq!(box_of(&base, (40, 5)).unwrap(), "No one.");
        assert_eq!(box_of(&base, (64, 2)).unwrap(), "No obstacle.");
        assert_eq!(box_of(&base, (63, 3)).unwrap(), "No one.");
        // Clone: at the limit its records, the dead among them (CLT-Q59), else no one.
        assert_eq!(box_of(&base, (65, 4)).unwrap(), "No one.");
        let full = Facts {
            records: 21,
            ..base.clone()
        };
        assert_eq!(box_of(&full, (65, 4)).unwrap(), "21 stacks");
    }

    #[test]
    fn the_summons_tell_the_bound_elemental_and_the_limit_by_the_profile() {
        // E17 M6 T5a (design §12.17, oddity 3, CLT-Q57): another elemental summoned: 539 in both
        // profiles, the pronoun by the hero's sex; at the limit the client's says the records
        // (184), the original's 539 with the side's elemental or an empty name.
        let c = refusals_catalogue();
        let fire = (114, "Fire Elementals".to_owned());
        let mut facts = Facts {
            heroes: [Some(hero("Valeska")), None],
            limit: 20,
            records: 5,
            summoned: Some(fire.clone()),
            ..Facts::default()
        };
        let both = |facts: &Facts, spell: u32| {
            [Bugs::Fixed, Bugs::Original].map(|bugs| after_book(facts, (spell, 5), bugs, &c))
        };
        let other = Some("Valeska: Fire Elementals (his)".to_owned());
        assert_eq!(both(&facts, 68), [other.clone(), other]);
        facts.heroes[0].as_mut().unwrap().female = true;
        let other = Some("Valeska: Fire Elementals (her)".to_owned());
        assert_eq!(both(&facts, 68), [other.clone(), other]);
        // Its own elemental below the limit: no box (the engine refuses for a reason of its own).
        assert_eq!(both(&facts, 66), [None, None]);
        facts.records = 20;
        let limit = Some("20 stacks".to_owned());
        let own = Some("Valeska: Fire Elementals (her)".to_owned());
        assert_eq!(both(&facts, 66), [limit.clone(), own]);
        facts.summoned = None;
        let empty = Some("Valeska:  (her)".to_owned());
        assert_eq!(both(&facts, 69), [limit, empty]);
        // Another spell than the Summons: none of theirs.
        assert_eq!(both(&facts, 1), [None, None]);
    }

    /// The side to act's own view at the opening of the capture set's scenario `id`, the
    /// record's manifest and the ruleset.
    fn opening_of(id: &str) -> (BattleRuleset, Observation, Manifest) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let scenario = dir.join(format!("situations/{id}.toml"));
        let (ruleset, setup) =
            crate::scenario::battle(&scenario, &dir.join("../packs"), 1).unwrap();
        let (observation, record) = {
            let mut live = crate::live::Live::new(&ruleset, setup, 1, [true, true]).unwrap();
            (live.own_view().unwrap(), live.record().unwrap())
        };
        let manifest = h3_view::record::parse(&record).unwrap().header.manifest;
        (ruleset, observation, manifest)
    }

    #[test]
    fn the_facts_come_from_the_seat_s_view_the_opening_and_the_heroes_magic() {
        // E17 M6 T5a (design §12.17, "The facts").
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let c = Catalogue::default();
        // Rough: the defender's Stronghold stacks native, the attacker's two stacks and Orrin
        // (male), no field limit, no fort; the ruleset's limit of 20 records.
        let (ruleset, own, manifest) = opening_of("msg-mine");
        let names = (&manifest, &tables, &c);
        let facts = Facts::of(&own, &own, [None, None], Some(&ruleset), names);
        assert!(facts.natives && !facts.fort && !facts.garrison);
        assert_eq!((facts.side, facts.records, facts.limit), (0, 2, 20));
        assert_eq!(facts.field_level, None);
        let orrin = facts.heroes[0].clone().unwrap();
        assert_eq!(orrin.name, "hero.base:orrin");
        assert!(!orrin.female && !orrin.forbids);
        // Cursed Ground by the ruleset (level 1) or, without it, by the field's key.
        let (ruleset, own, manifest) = opening_of("msg-cursed");
        let names = (&manifest, &tables, &c);
        let facts = Facts::of(&own, &own, [None, None], Some(&ruleset), names);
        assert_eq!(facts.field_level, Some(1));
        assert!(!facts.natives);
        let facts = Facts::of(&own, &own, [None, None], None, names);
        assert_eq!((facts.field_level, facts.limit), (Some(1), 20));
        // A siege with a Fort, Valeska female by her type, unless her map says otherwise; the
        // heroes' magic as known; the side's summoned elemental by its original number.
        let (ruleset, mut own, manifest) = opening_of("msg-remove");
        let names = (&manifest, &tables, &c);
        let mut magic = own.own_hero.clone().unwrap();
        (magic.no_casting, magic.level_limit) = (true, Some(2));
        let fire = ruleset
            .creature_by_key("base:fire_elemental")
            .unwrap()
            .raw();
        own.sides[0].summoned = Some(fire);
        let facts = Facts::of(&own, &own, [Some(&magic), None], Some(&ruleset), names);
        assert!(facts.fort);
        let valeska = facts.heroes[0].clone().unwrap();
        assert!(valeska.female && valeska.forbids);
        assert_eq!(valeska.level_limit, Some(2));
        assert_eq!(facts.heroes[1].as_ref().map(|h| h.forbids), Some(false));
        let plural = "creature.base:fire_elemental".to_owned();
        assert_eq!(facts.summoned, Some((114, plural)));
        assert_eq!(facts.hero(), "hero.base:valeska");
        let hero = own.sides[0].hero.as_mut().unwrap();
        hero.sex = Some(Sex::Male);
        let facts = Facts::of(&own, &own, [None, None], Some(&ruleset), names);
        assert!(!facts.heroes[0].as_ref().unwrap().female);
        own.sides[0].hero.as_mut().unwrap().sex = Some(Sex::Female);
        let facts = Facts::of(&own, &own, [None, None], Some(&ruleset), names);
        assert!(facts.heroes[0].as_ref().unwrap().female);
        // No active stack: the viewer's side; no hero: no name; a creature the record does not
        // know: no elemental.
        own.active = None;
        own.viewer = Side::Defender;
        own.sides[1].hero = None;
        own.sides[1].summoned = Some(u16::MAX);
        let facts = Facts::of(&own, &own, [None, None], Some(&ruleset), names);
        assert_eq!((facts.side, facts.hero()), (1, ""));
        assert_eq!(facts.summoned, None);
    }

    #[test]
    fn a_hypnotized_stack_s_turn_names_its_controller_s_hero() {
        // E17 M6 T5a (the review): the side to act is the active stack's controller (+0x132C0, as
        // the engine's `side_to_act`), not its owner: the attacker's Catapult controlled by the
        // defender casts with the defender's hero, whom 537 and 539 name; the records and the
        // elemental are the defender's, the natives the attacker's.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let c = refusals_catalogue();
        let (ruleset, mut own, manifest) = opening_of("msg-remove");
        let names = (&manifest, &tables, &c);
        let active = own.active.unwrap();
        let stack = own.stacks.iter_mut().find(|s| s.id == active).unwrap();
        assert_eq!(
            (stack.side, stack.controller),
            (Side::Attacker, Side::Attacker)
        );
        stack.controller = Side::Defender;
        let raw = |key: &str| ruleset.creature_by_key(key).unwrap().raw();
        own.sides[0].summoned = Some(raw("base:water_elemental"));
        own.sides[1].summoned = Some(raw("base:fire_elemental"));
        let mut facts = Facts::of(&own, &own, [None, None], Some(&ruleset), names);
        assert_eq!(facts.side, 1);
        let defender = facts.heroes[1].clone().unwrap();
        assert_ne!(defender.name, facts.heroes[0].as_ref().unwrap().name);
        assert_eq!(facts.hero(), defender.name);
        let records = own.stacks.iter().filter(|s| s.side == Side::Defender);
        assert_eq!(facts.records, u32::try_from(records.count()).unwrap());
        assert_eq!(facts.summoned.as_ref().map(|s| s.0), Some(114));
        // Summon Water Elemental with the defender's Fire Elementals bound.
        let pronoun = if defender.female { "her" } else { "his" };
        let summoned = after_book(&facts, (68, 5), Bugs::Fixed, &c).unwrap();
        let fire = "creature.base:fire_elemental";
        assert_eq!(summoned, format!("{}: {fire} ({pronoun})", defender.name));
        // The attacker's hero wears the Cloak: the defender's hero is kept from casting.
        facts.heroes[0].as_mut().unwrap().level_limit = Some(2);
        let cloak = after_book(&facts, (17, 3), Bugs::Fixed, &c).unwrap();
        assert_eq!(cloak, format!("Cloak stops {}", defender.name));
    }

    #[test]
    fn a_seat_on_a_server_knows_its_own_hero_s_magic_alone() {
        // E17 M6 T5a (`Battle::facts`' server branch; gap G1, CLT-STUB-1): the seat's own hero's
        // limits filled, the other side's none (its Orb shows no 684 there), the public facts
        // (the garrison, Cursed Ground by its key without the ruleset) present all the same.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let c = refusals_catalogue();
        let (_, mut own, manifest) = opening_of("msg-cursed");
        let names = (&manifest, &tables, &c);
        assert_eq!(own.viewer, Side::Attacker);
        let mine = own.own_hero.as_mut().unwrap();
        (mine.no_casting, mine.level_limit) = (true, Some(3));
        own.anti_magic = true;
        let magic = seat_magic(&own);
        assert_eq!(magic, [own.own_hero.clone(), None]);
        let [attacker, defender] = &magic;
        let facts = Facts::of(
            &own,
            &own,
            [attacker.as_ref(), defender.as_ref()],
            None,
            names,
        );
        assert!(facts.garrison);
        assert_eq!((facts.field_level, facts.limit), (Some(1), 20));
        let hero = facts.heroes[0].clone().unwrap();
        assert!(hero.forbids);
        assert_eq!(hero.level_limit, Some(3));
        assert!(
            facts.heroes[1]
                .iter()
                .all(|h| !h.forbids && h.level_limit.is_none())
        );
        assert_eq!(before_book(&facts, &c).unwrap(), "Garrison.");
        let open = Facts {
            garrison: false,
            ..facts
        };
        assert_eq!(
            before_book(&open, &c).unwrap(),
            format!("{} wears Orb", hero.name)
        );
        // The other side's hero unknown: no box for it.
        let unknown = Facts::of(&own, &own, [None, None], None, names);
        assert_eq!(
            before_book(
                &Facts {
                    garrison: false,
                    ..unknown
                },
                &c
            ),
            None
        );
        // The defender's seat: its own hero's alone.
        own.viewer = Side::Defender;
        assert_eq!(seat_magic(&own), [None, own.own_hero.clone()]);
    }

    #[test]
    fn summon_other_s_second_elemental_is_refused_for_the_first() {
        // E17 M6 T5a (`summon-other`): Fire Elementals summoned in round 1, everyone defends; in
        // round 2 the engine gives Summon Water Elemental no cast, and the box is 539 (the Fire
        // Elementals named), in both profiles.
        use h3_battle::{BattleSide, BattleState, Viewer};
        use h3_protocol::map;
        use h3_protocol::wire::Seat;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let scenario = dir.join("situations/seq-summon-other.toml");
        let (ruleset, setup) =
            crate::scenario::battle(&scenario, &dir.join("../packs"), 1).unwrap();
        let mut state = BattleState::new(&ruleset, &setup, 1).unwrap();
        let first = map::observe(&state, &ruleset, Viewer::Side(BattleSide::Attacker));
        let spell = |key: &str| ruleset.spell_by_key(key).unwrap().raw();
        let cast = Command::Cast {
            spell: spell("base:fire_elemental"),
            target: h3_protocol::wire::Target::None {},
            second: h3_protocol::wire::Target::None {},
        };
        let mut sink = h3_battle::NoopSink;
        let attacker = BattleSide::Attacker;
        h3_battle::apply(
            &mut state,
            &ruleset,
            attacker,
            map::command_to_engine(cast),
            &mut sink,
        )
        .unwrap();
        let defend = map::command_to_engine(Command::Defend {});
        for _ in 0..3 {
            let side = state.side_to_act().unwrap();
            h3_battle::apply(&mut state, &ruleset, side, defend, &mut sink).unwrap();
        }
        assert_eq!(state.side_to_act(), Some(attacker));
        let water = spell("base:water_elemental");
        let mut found = h3_battle::CastTargets::new();
        let id = h3_kernel::Handle::from_raw(water);
        h3_battle::cast_targets(&state, &ruleset, attacker, id, &mut found);
        assert!(found.is_empty());
        let own = map::observe(&state, &ruleset, Viewer::Side(attacker));
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let record =
            crate::scenario::record_of(&scenario, &dir.join("../packs"), 1, Seat::Attacker)
                .unwrap();
        let manifest = h3_view::record::parse(&record).unwrap().header.manifest;
        let c = refusals_catalogue();
        let facts = Facts::of(
            &own,
            &first,
            [None, None],
            Some(&ruleset),
            (&manifest, &tables, &c),
        );
        assert_eq!(facts.summoned.as_ref().map(|s| s.0), Some(114));
        for bugs in [Bugs::Fixed, Bugs::Original] {
            let text = after_book(&facts, (68, 5), bugs, &c).unwrap();
            assert_eq!(text, "hero.base:orrin: creature.base:fire_elemental (his)");
        }
    }
}
