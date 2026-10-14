//! The adapter's tests (moved from `h3-pack` with the content split, R01b3). They patch the packs
//! by text, as before the split: a [`Texts`] holds a pack's two files and a patch applies to both.

use std::num::NonZeroU32;
use std::sync::LazyLock;

use super::*;
use h3_content::{
    Affects, ContentPack, CreatureSelector, CreatureStat, CreatureTypeId, Formation, HeroEffect,
    SchoolId, SpellGrant, SpellId, SpellScope,
};
use h3_contracts::{
    ArmyStack, BallistaControl, BattleEffect, BattleHero, BattleSide, DamageBonuses, EagleEye,
    FactionGroup, FortLevel, Moat, Obstacle, Rate, StatOp, TentControl,
};

/// A pack's `core.toml` and `battle.toml`, patched together (R01b3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Texts {
    core: String,
    battle: String,
}

impl Texts {
    fn new(core: &str, battle: &str) -> Self {
        Self {
            core: core.into(),
            battle: battle.into(),
        }
    }

    /// Replaces `from` in both files; the patch must match (a patch that matches nothing would
    /// test nothing, silently).
    fn replace(&self, from: &str, to: &str) -> Self {
        assert!(self.contains(from), "the patch `{from}` matches nothing");
        Self {
            core: self.core.replace(from, to),
            battle: self.battle.replace(from, to),
        }
    }

    /// Replaces the first `n` matches of `from` in the one file that has it.
    fn replacen(&self, from: &str, to: &str, n: usize) -> Self {
        assert!(self.contains(from), "the patch `{from}` matches nothing");
        let mut out = self.clone();
        let both = self.core.contains(from) && self.battle.contains(from);
        assert!(!both, "the patch `{from}` matches in both files: say which");
        if self.core.contains(from) {
            out.core = self.core.replacen(from, to, n);
        } else {
            out.battle = self.battle.replacen(from, to, n);
        }
        out
    }

    /// Replaces `from` in the battle file only; the patch must match.
    fn battle_replace(&self, from: &str, to: &str) -> Self {
        assert!(
            self.battle.contains(from),
            "the patch `{from}` matches nothing"
        );
        Self {
            core: self.core.clone(),
            battle: self.battle.replace(from, to),
        }
    }

    fn contains(&self, text: &str) -> bool {
        self.core.contains(text) || self.battle.contains(text)
    }

    /// The pack with `extra` appended to its core file.
    fn core_with(&self, extra: &str) -> Self {
        Self {
            core: format!("{}\n{extra}", self.core),
            battle: self.battle.clone(),
        }
    }

    /// The pack with `extra` appended to its battle file.
    fn battle_with(&self, extra: &str) -> Self {
        Self {
            core: self.core.clone(),
            battle: format!("{}\n{extra}", self.battle),
        }
    }
}

static SYNTHETIC_PACK: LazyLock<Texts> =
    LazyLock::new(|| Texts::new(h3_pack::SYNTHETIC_CORE, SYNTHETIC_BATTLE));
static SYNTHETIC_EXT_PACK: LazyLock<Texts> =
    LazyLock::new(|| Texts::new(h3_pack::SYNTHETIC_EXT_CORE, SYNTHETIC_EXT_BATTLE));
static BASE_PACK: LazyLock<Texts> = LazyLock::new(|| Texts::new(h3_pack::BASE_CORE, BASE_BATTLE));
static AB_PACK: LazyLock<Texts> = LazyLock::new(|| Texts::new(h3_pack::AB_CORE, AB_BATTLE));
static SOD_PACK: LazyLock<Texts> = LazyLock::new(|| Texts::new(h3_pack::SOD_CORE, SOD_BATTLE));

/// [`super::compile`] over patched packs.
fn compile(packs: &[&Texts], profile: &str) -> Result<BattleRuleset, BattlePackError> {
    let texts: Vec<(&str, &str)> = packs
        .iter()
        .map(|t| (t.core.as_str(), t.battle.as_str()))
        .collect();
    super::compile(&texts, profile)
}

/// Both halves of a patched pack.
fn parse_pack_parts(texts: &Texts) -> Result<(ContentPack, BattlePack), BattlePackError> {
    let core = h3_pack::parse_pack(&texts.core)?;
    let battle = parse_battle(&core.manifest, &texts.battle)?;
    Ok((core, battle))
}

/// The core half of a patched pack (its battle file must parse too).
fn parse_pack(texts: &Texts) -> Result<ContentPack, BattlePackError> {
    parse_pack_parts(texts).map(|(core, _)| core)
}
#[test]
fn the_siege_scenario_names_its_town() {
    let (ruleset, setup) = synthetic(SYNTHETIC_SIEGE).unwrap();
    let town = setup.siege.unwrap();
    let vale = ruleset.faction_by_key("synthetic:vale").unwrap();
    assert_eq!(
        (town.faction, town.fort, town.moat, town.structures),
        (vale, FortLevel::Citadel, Moat::Damaging, 3)
    );
    // The entry and back (a replay's `[setup.siege]`).
    let entry = SiegeEntry::of(&town, &ruleset).unwrap();
    assert_eq!(
        (entry.fort.as_str(), entry.moat.as_str()),
        ("citadel", "damaging")
    );
    assert_eq!(entry.to_siege(&ruleset).unwrap(), town);
    let refused = |edit: &dyn Fn(&mut SiegeEntry)| {
        let mut e = entry.clone();
        edit(&mut e);
        e.to_siege(&ruleset).unwrap_err().to_string()
    };
    assert!(refused(&|e| e.faction = "synthetic:nowhere".into()).contains("unknown faction"));
    assert!(refused(&|e| e.fort = "keep".into()).contains("unknown fort level `keep`"));
    assert!(refused(&|e| e.moat = "lava".into()).contains("unknown moat `lava`"));
    let stray = h3_contracts::Siege {
        faction: h3_content::FactionId::from_index(9).unwrap(),
        ..town
    };
    assert!(SiegeEntry::of(&stray, &ruleset).is_err());
}

#[test]
fn synthetic_pack_and_scenarios_load() {
    for text in [
        SYNTHETIC_SKIRMISH,
        SYNTHETIC_DUEL,
        SYNTHETIC_FULL,
        SYNTHETIC_SIEGE,
    ] {
        let (ruleset, setup) = synthetic(text).unwrap();
        assert_eq!(ruleset.creature_count(), 15);
        assert!(!setup.attacker.stacks.is_empty());
        assert!(!setup.defender.stacks.is_empty());
    }
    let (ruleset, full) = synthetic(SYNTHETIC_FULL).unwrap();
    let key = |c| ruleset.creature_key(c).unwrap().as_str();
    assert_eq!(
        full.war_machines
            .map(|m| m.into_iter().map(key).collect::<Vec<_>>()),
        [
            ["synthetic:bolt_thrower", "synthetic:aid_tent"],
            ["synthetic:bolt_thrower", "synthetic:supply_wagon"]
        ]
    );
    let (ruleset, setup) = synthetic(SYNTHETIC_SKIRMISH).unwrap();
    assert_eq!(ruleset.profile().as_str(), "synthetic:faithful");
    assert_eq!(ruleset.battle().army_slots, 7);
    let brute = ruleset.creature_by_key("synthetic:brute").unwrap();
    assert_eq!(
        setup.attacker.stacks[1],
        ArmyStack {
            slot: 1,
            creature: brute,
            count: 10
        }
    );
    let spearman = ruleset
        .creature(ruleset.creature_by_key("synthetic:spearman").unwrap())
        .unwrap();
    assert_eq!(
        (
            spearman.damage_min,
            spearman.damage_max,
            spearman.hit_points
        ),
        (1, 3, 10)
    );
    assert!(compile(&[&SYNTHETIC_PACK], "synthetic:strict-fixed").is_ok());
}

#[test]
fn strict_parsing() {
    let unknown_field = SYNTHETIC_PACK.replace("speed = 4", "speed = 4\nflying = true");
    assert!(matches!(
        parse_pack(&unknown_field),
        Err(BattlePackError::Syntax(_))
    ));
    let out_of_range = SYNTHETIC_PACK.replace("speed = 4", "speed = 400");
    assert!(matches!(
        parse_pack(&out_of_range),
        Err(BattlePackError::Syntax(_))
    ));
    let bad_name = SYNTHETIC_PACK.replace("id = \"brute\"", "id = \"Brute\"");
    assert!(matches!(
        parse_pack(&bad_name),
        Err(BattlePackError::Key(_))
    ));
    let bad_profile = SYNTHETIC_PACK.replace("id = \"faithful\"", "id = \"Faithful\"");
    assert!(matches!(
        parse_pack(&bad_profile),
        Err(BattlePackError::Key(_))
    ));
    let quirky = SYNTHETIC_PACK.replace(
        "quirks = { \"quirk.battle.no_luck_without_hero\" = true, \
         \"quirk.battle.single_precision_rates\" = true, \
         \"quirk.battle.unlimited_ballista_shots\" = true, \
         \"quirk.battle.first_protection\" = true, \
         \"quirk.battle.counterstrike_extra_round\" = true, \
         \"quirk.battle.frenzy_reduced_defense\" = true, \
         \"quirk.battle.fire_shield_first_stage\" = true, \
         \"quirk.battle.early_mass_version\" = true, \
         \"quirk.battle.mass_cure_skips_hypnotized\" = true, \
         \"quirk.battle.obstacle_anchors\" = true, \
         \"quirk.battle.draws_avoid_hidden\" = true, \
         \"quirk.battle.sacrifice_tests_as_resurrection\" = true, \
         \"quirk.battle.clone_on_its_original\" = true, \
         \"quirk.battle.periodic_first_stack\" = true }",
        "quirks = { \"quirk.x\" = true }",
    );
    let pack = parse_pack(&quirky).unwrap();
    assert_eq!(pack.profiles[0].quirks, vec![("quirk.x".to_owned(), true)]);
    assert!(matches!(
        compile(&[&quirky], "synthetic:faithful"),
        Err(BattlePackError::Compile(_))
    ));
    // A retired flag is no flag any more (ADR-0007): a pack that still sets it, as the packs
    // did until its retirement, does not compile. BAT-QUIRK-2 and BAT-QUIRK-9, retired on
    // 2026-10-12.
    for retired in [
        "quirk.battle.attack_from_seen_pit",
        "quirk.battle.creature_mirror_untested",
    ] {
        let old = SYNTHETIC_PACK.replace(
            "\"quirk.battle.periodic_first_stack\" = true }",
            &format!("\"quirk.battle.periodic_first_stack\" = true, \"{retired}\" = true }}"),
        );
        let named = (retired.to_owned(), true);
        assert!(
            parse_pack(&old).unwrap().profiles[0]
                .quirks
                .contains(&named)
        );
        assert!(matches!(
            compile(&[&old], "synthetic:faithful"),
            Err(BattlePackError::Compile(CompileError::UnknownQuirk { flag, .. })) if flag == retired
        ));
    }
    let useless =
        SYNTHETIC_PACK.battle_replace("id = \"spark\"", "id = \"spark\"\nuseful = \"nope\"");
    assert!(matches!(
        parse_pack(&useless),
        Err(BattlePackError::Syntax(_))
    ));
    let (no_core, no_battle) =
        parse_pack_parts(&Texts::new("[pack]\nid = \"x\"\nversion = \"1\"\n", "")).unwrap();
    assert!(no_battle.battle.is_none() && no_core.creatures.is_empty());
    assert_eq!(no_battle.manifest, no_core.manifest);
}

#[test]
fn scenario_errors() {
    assert!(matches!(
        parse_scenario("nonsense = ["),
        Err(BattlePackError::Syntax(_))
    ));
    let bad_profile = SYNTHETIC_DUEL.replace("synthetic:faithful", "faithful");
    assert!(matches!(
        parse_scenario(&bad_profile),
        Err(BattlePackError::Key(_))
    ));
    let unknown = SYNTHETIC_DUEL.replace("synthetic:spearman", "synthetic:dragon");
    assert_eq!(
        synthetic(&unknown),
        Err(BattlePackError::UnknownCreature(
            "synthetic:dragon".to_owned()
        ))
    );
    let unknown_attacker = SYNTHETIC_DUEL.replace("synthetic:brute", "synthetic:dragon");
    assert!(synthetic(&unknown_attacker).is_err());
    let empty = parse_scenario("[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n").unwrap();
    assert_eq!(empty.id, "e");
    assert!(empty.packs.is_empty());
    assert_eq!(parse_scenario(SYNTHETIC_DUEL).unwrap().packs, ["synthetic"]);
    // Obstacles as pieces (E04b T5b): the hexes, an optional anchor and an optional type
    // (E17 M1, CLT-Q3).
    let pieces = parse_scenario(
        "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n\
         obstacles = [{ hexes = [58, 75], anchor = 57, kind = \"a:rock\" }, { hexes = [93] }]\n",
    )
    .unwrap();
    assert_eq!(
        pieces.obstacles,
        [
            Obstacle {
                hexes: vec![58, 75],
                anchor: Some(57),
                kind: ContentKey::parse("a:rock").ok(),
            },
            Obstacle {
                hexes: vec![93],
                anchor: None,
                kind: None,
            }
        ]
    );
    for piece in &pieces.obstacles {
        let entry = ObstacleEntry::of(piece);
        assert_eq!(entry.to_obstacle().as_ref(), Ok(piece));
    }
    assert_eq!(
        parse_scenario(
            "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n\
             obstacles = [{ hexes = [1], kind = \"rock\" }]\n"
        )
        .unwrap_err(),
        BattlePackError::Syntax("bad obstacle type `rock`".to_owned())
    );
    assert!(
        parse_scenario(
            "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n\
             obstacles = [{ hexes = [1], ancor = 2 }]\n"
        )
        .is_err()
    );
    let header = "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n";
    for (extra, error) in [
        (
            "random_obstacles = true\n",
            "random_obstacles needs a terrain",
        ),
        (
            "random_obstacles = true\nterrain = \"a:t\"\nobstacles = [{ hexes = [40] }]\n",
            "random_obstacles excludes an obstacles list",
        ),
    ] {
        assert_eq!(
            parse_scenario(&format!("{header}{extra}")),
            Err(BattlePackError::Syntax(error.to_owned()))
        );
    }
    let duel = parse_scenario(SYNTHETIC_DUEL).unwrap();
    assert!(duel.random_obstacles);
    let (ruleset, setup) = synthetic(SYNTHETIC_DUEL).unwrap();
    assert_eq!(setup.terrain, ruleset.terrain_by_key("synthetic:meadow"));
    assert_eq!(
        (setup.battlefield.as_ref(), setup.anti_magic),
        (None, false)
    );
    // A special battlefield and an anti-magic garrison (E04c T2b): the key is checked by the
    // battle, only its syntax here.
    let field = SYNTHETIC_DUEL.replace(
        "random_obstacles = true",
        "random_obstacles = true\nbattlefield = \"synthetic:hallow\"\nanti_magic = true",
    );
    let (_, setup) = synthetic(&field).unwrap();
    assert_eq!(
        setup.battlefield.map(|k| k.as_str().to_owned()),
        Some("synthetic:hallow".to_owned())
    );
    assert!(setup.anti_magic);
    assert!(parse_scenario(&format!("{header}battlefield = \"no key\"\n")).is_err());
    let elsewhere = SYNTHETIC_DUEL.replace("synthetic:meadow", "synthetic:moon");
    assert_eq!(
        synthetic(&elsewhere),
        Err(BattlePackError::UnknownTerrain("synthetic:moon".to_owned()))
    );
    for (sides, err) in [
        ("[\"attacker\", \"defender\"]", None),
        ("[\"left\"]", Some("unknown side `left`")),
        (
            "[\"defender\", \"defender\"]",
            Some("side `defender` listed twice"),
        ),
    ] {
        let text = format!("{header}tight_formation = {sides}\n");
        match err {
            None => assert_eq!(
                parse_scenario(&text).unwrap().formations,
                [Formation::Tight; 2]
            ),
            Some(e) => assert_eq!(
                parse_scenario(&text),
                Err(BattlePackError::Syntax(e.to_owned()))
            ),
        }
    }
    assert_eq!(
        side_names([false, true]),
        ["defender"],
        "inverse of parse_sides"
    );
    assert_eq!(side_names([true, false]), ["attacker"]);
    let tight = parse_scenario(&format!("{header}tight_formation = [\"defender\"]\n")).unwrap();
    assert_eq!(tight.formations, [Formation::Loose, Formation::Tight]);
    assert_eq!(tight_sides(tight.formations), ["defender"]);
    assert_eq!(
        formations(&["x".to_owned()]),
        Err(BattlePackError::Syntax("unknown side `x`".to_owned()))
    );
}

// covers: BAT-HER-001
#[test]
fn scenario_heroes() {
    let (ruleset, setup) = synthetic(SYNTHETIC_FULL).unwrap();
    let brea = ruleset.hero_type_by_key("synthetic:brea").unwrap();
    let mut expected = BattleHero::new(brea);
    [
        expected.attack,
        expected.defense,
        expected.power,
        expected.knowledge,
    ] = [2, 2, 2, 1];
    expected.tactics = 2;
    expected.magic.mana = 40;
    expected.magic.max_mana = 40;
    // The four damage spells, Ward, five timed spells, Mend, Blaze, Glass, Haze, Charm,
    // Scorch, Blink, Snare, Revive and Offer with Basic Storm magic (E04b T2c, T3a, T3b1,
    // T3b1b, T3b2, T4a, T4b, T4c, T5a, T5c2, T6a).
    expected.magic.spell_book = Some(
        [
            "spark", "cinders", "arc", "blight", "ward", "hasten", "drag", "fury", "smiting",
            "riposte", "mend", "blaze", "glass", "haze", "charm", "scorch", "blink", "snare",
            "revive", "offer", "copy",
        ]
        .map(|k| ruleset.spell_by_key(&format!("synthetic:{k}")).unwrap())
        .to_vec(),
    );
    // Basic Storm magic: every book spell but the stone ones (blight, revive) at Basic, six
    // at Advanced (E04b T9b).
    let advanced =
        ["spark", "cinders", "haze", "charm", "scorch", "snare"].map(|k| format!("synthetic:{k}"));
    expected.magic.masteries = expected
        .magic
        .spell_book
        .iter()
        .flatten()
        .filter(|&&s| {
            let key = ruleset.spell(s).unwrap().key.as_str();
            key != "synthetic:blight" && key != "synthetic:revive"
        })
        .map(|&s| {
            let key = ruleset.spell(s).unwrap().key.as_str();
            (
                s,
                if advanced.iter().any(|a| a == key) {
                    2
                } else {
                    1
                },
            )
        })
        .collect();
    expected.magic.masteries.sort_unstable_by_key(|&(s, _)| s);
    expected.damage = DamageBonuses {
        ranged: Rate::bp(1000),
        melee: Rate::bp(1000),
        reduction: Rate::bp(500),
    };
    expected.war_machines.ballista = Some(BallistaControl {
        shots: 1,
        double_damage_chance_bp: 5000,
    });
    expected.war_machines.tent = Some(TentControl {
        heal_bonus: Rate::bp(10_000),
    });
    expected.effects = vec![
        BattleEffect::Morale {
            amount: 1,
            affects: Affects::Own,
        },
        BattleEffect::Luck {
            amount: 1,
            affects: Affects::Own,
        },
        BattleEffect::SpellSpecialty {
            spell: ruleset.spell_by_key("synthetic:hasten").unwrap(),
            bp_per_level: 300,
            level: 6,
        },
        // The casting workload (E04b T9b): a Pendant, an opening spell.
        BattleEffect::SpellImmunity {
            spell: ruleset.spell_by_key("synthetic:spark").unwrap(),
        },
        BattleEffect::OpeningSpell {
            spell: ruleset.spell_by_key("synthetic:hasten").unwrap(),
            mastery: 3,
            power: 3,
        },
    ];
    expected.magic.eagle_eye = Some(EagleEye {
        chance: Rate::bp(4000),
        spells: [
            "boon", "bane", "hallow", "purge", "gloom", "corrode", "aegis", "rage", "dazzle",
            "uproot",
        ]
        .map(|k| ruleset.spell_by_key(&format!("synthetic:{k}")).unwrap())
        .to_vec(),
    });
    assert_eq!(setup.heroes[0], Some(expected.clone()));
    // The defender's Corwin (E04a T11c): Gunnery, a 10 % discount, nine stack effects, a
    // spell duration bonus (E04b T3a) and a spell specialty (T3c), written and read back
    // unchanged; both owners can pay for a surrender.
    let corwin = setup.heroes[1].as_ref().unwrap();
    assert_eq!(
        corwin.hero_type,
        ruleset.hero_type_by_key("synthetic:corwin").unwrap()
    );
    assert_eq!(
        (corwin.surrender_discount_bp, corwin.effects.len()),
        (1000, 15)
    );
    assert_eq!(corwin.effects[9], BattleEffect::SpellDuration { rounds: 2 });
    // The casting workload (E04b T9b): resistance, a Pendant, two opening spells, Eagle Eye.
    let spell = |k: &str| ruleset.spell_by_key(&format!("synthetic:{k}")).unwrap();
    let opening = |k| BattleEffect::OpeningSpell {
        spell: spell(k),
        mastery: 3,
        power: 2,
    };
    assert_eq!(
        corwin.effects[11..],
        [
            BattleEffect::MagicResistance {
                rate: Rate::bp(1000),
            },
            BattleEffect::SpellImmunity {
                spell: spell("cinders"),
            },
            opening("drag"),
            opening("bane"),
        ]
    );
    assert_eq!(
        corwin.magic.eagle_eye,
        Some(EagleEye {
            chance: Rate::bp(5000),
            spells: [
                "hasten", "drag", "mend", "ward", "fury", "smiting", "riposte", "blaze", "scorch",
                "cinders", "haze", "charm", "blink",
            ]
            .map(spell)
            .to_vec(),
        })
    );
    assert_eq!(corwin.war_machines.ballista, expected.war_machines.ballista);
    assert_eq!(
        &HeroEntry::of(corwin, &ruleset)
            .unwrap()
            .resolve(&ruleset)
            .unwrap(),
        corwin
    );
    assert_eq!(setup.funds, [10_000, 10_000]);
    // Fields default to 0; the entry is the inverse of the resolution.
    let header = "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n";
    let text = format!("{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\n");
    let entry = parse_scenario(&text).unwrap().heroes[1].clone().unwrap();
    let ansel = entry.resolve(&ruleset).unwrap();
    assert_eq!(
        ansel,
        BattleHero::new(ruleset.hero_type_by_key("synthetic:ansel").unwrap())
    );
    assert_eq!(HeroEntry::of(&ansel, &ruleset), Ok(entry));
    assert_eq!(
        HeroEntry::of(&expected, &ruleset)
            .unwrap()
            .resolve(&ruleset),
        Ok(expected.clone())
    );
    // Unknown hero types and fields.
    let unknown = SYNTHETIC_FULL.replace("synthetic:brea", "synthetic:nobody");
    assert_eq!(
        synthetic(&unknown),
        Err(BattlePackError::UnknownHeroType(
            "synthetic:nobody".to_owned()
        ))
    );
    let extra = SYNTHETIC_FULL.replace("max_mana = 40", "max_mana = 40\nluck = 1");
    assert!(matches!(
        parse_scenario(&extra),
        Err(BattlePackError::Syntax(_))
    ));
    for bad in [
        "kind = \"no_retreat\", sides = 2",
        "kind = \"luck\", amount = 1, value = 2",
    ] {
        let text = format!(
            "{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\neffects = [{{ {bad} }}]\n"
        );
        assert!(
            matches!(parse_scenario(&text), Err(BattlePackError::Syntax(_))),
            "{bad}"
        );
    }
    // Every morale and luck kind for every side survives the round trip, written as a list of
    // tables after the values.
    let spark = ruleset.spell_by_key("synthetic:spark").unwrap();
    let mut every = expected.clone();
    every.effects = Affects::ALL
        .into_iter()
        .flat_map(|affects| {
            [
                BattleEffect::Morale {
                    amount: -2,
                    affects,
                },
                BattleEffect::Luck { amount: 3, affects },
                BattleEffect::MoraleAtMost { value: 0, affects },
                BattleEffect::LuckAtMost { value: 1, affects },
            ]
        })
        .chain([BattleEffect::NoRetreat])
        .chain(
            [
                CreatureSelector::All,
                CreatureSelector::Creature(ruleset.creature_by_key("synthetic:brute").unwrap()),
                CreatureSelector::Flag(CreatureFlag::Dragon),
            ]
            .into_iter()
            .zip(CreatureStat::ALL.into_iter().zip([
                StatOp::Add,
                StatOp::PercentBp,
                StatOp::PercentBpDown,
            ]))
            .chain(
                CreatureStat::ALL
                    .into_iter()
                    .map(|stat| (CreatureSelector::All, (stat, StatOp::Add))),
            )
            .map(|(creatures, (stat, op))| BattleEffect::StackStat {
                creatures,
                stat,
                op,
                amount: -3,
            }),
        )
        .chain([
            BattleEffect::Regeneration {
                creatures: CreatureSelector::Flag(CreatureFlag::Alive),
                hit_points: 50,
            },
            BattleEffect::NoRangePenalty {
                creatures: CreatureSelector::All,
            },
            BattleEffect::ShootWhenAdjacent {
                creatures: CreatureSelector::Creature(
                    ruleset.creature_by_key("synthetic:brute").unwrap(),
                ),
            },
            BattleEffect::AlignmentGroup {
                factions: FactionGroup::new(&[ruleset.faction_by_key("synthetic:vale").unwrap()])
                    .unwrap(),
            },
        ])
        .chain([
            BattleEffect::SpellDamage {
                scope: SpellScope::All,
                rate: Rate {
                    bp: 2250,
                    single: NonZeroU32::new(0x3e66_6667),
                },
            },
            BattleEffect::SpellDamage {
                scope: SpellScope::School(SchoolId::from_raw(1)),
                rate: Rate::bp(5000),
            },
            BattleEffect::SpellDamage {
                scope: SpellScope::Spell(spark),
                rate: Rate::bp(2500),
            },
            BattleEffect::SpellSpecialty {
                spell: spark,
                bp_per_level: 300,
                level: 12,
            },
            BattleEffect::MagicResistance {
                rate: Rate {
                    bp: 3000,
                    single: NonZeroU32::new(0x3e99_999a),
                },
            },
            // E04b T8b.
            BattleEffect::NoCasting,
            BattleEffect::SpellLevelLimit { max_level: 2 },
            BattleEffect::SpellImmunity { spell: spark },
            BattleEffect::SpellLevelImmunity { max_level: 4 },
            BattleEffect::MindImmunity,
            BattleEffect::NegateImmunities,
            // E04b T8c.
            BattleEffect::OpeningSpell {
                spell: spark,
                mastery: 3,
                power: 50,
            },
        ])
        .collect();
    // Single-precision values of their own (E04b T3c, OPEN-QUESTIONS Q45).
    every.damage.ranged.single = NonZeroU32::new(0x3eb5_c290);
    every.damage.melee.single = NonZeroU32::new(1);
    every.damage.reduction.single = NonZeroU32::new(2);
    if let Some(tent) = &mut every.war_machines.tent {
        tent.heal_bonus.single = NonZeroU32::new(0x40ce_6667);
    }
    every.surrender_discount_bp = 4000;
    every.magic.spell_book = Some(vec![spark]);
    every.magic.masteries = vec![(spark, 2)];
    // E04b T8d: Eagle Eye's chance with a single-precision value of its own, its spells.
    every.magic.eagle_eye = Some(EagleEye {
        chance: Rate {
            bp: 4400,
            single: NonZeroU32::new(0x3ee1_47af),
        },
        spells: vec![spark],
    });
    let entry = HeroEntry::of(&every, &ruleset).unwrap();
    assert_eq!(entry.resolve(&ruleset), Ok(every));
    let written = toml::to_string(&entry).unwrap();
    assert!(
        written.contains("[[effects]]\nkind = \"morale_at_most\""),
        "{written}"
    );
    assert!(
        written.contains(
            "[[effects]]\nkind = \"stack_stat\"\nstat = \"defense\"\nop = \"percent_bp\"\namount = -3\n\n[effects.creatures]\ncreature = \"synthetic:brute\""
        ),
        "{written}"
    );
    assert_eq!(toml::from_str::<HeroEntry>(&written), Ok(entry));
    // Damage bonuses and the Ballista's control are written only when set.
    assert!(
        written.contains(
            "[[effects]]\nkind = \"regeneration\"\nhit_points = 50\n\n[effects.creatures]\nflag = \"alive\"\n\n[[effects]]\nkind = \"no_range_penalty\"\n\n[[effects]]"
        ),
        "{written}"
    );
    assert!(
        written.contains(
            "ranged_bp = 1000\nranged_single = 1052099216\nmelee_bp = 1000\nmelee_single = 1\n\
             reduction_bp = 500\nreduction_single = 2\n"
        ),
        "{written}"
    );
    assert!(written.contains("heal_bonus_bp = 10000\nsingle = 1087268455\n"));
    assert!(
        written.contains(
            "[eagle_eye]\nchance_bp = 4400\nsingle = 1054951343\nspells = [\"synthetic:spark\"]\n"
        ),
        "{written}"
    );
    assert!(
        written.contains(
            "[[effects]]\nkind = \"opening_spell\"\nspell = \"synthetic:spark\"\nmastery = 3\npower = 50\n"
        ),
        "{written}"
    );
    assert!(
        written.contains("[ballista]\nshots = 1\ndouble_damage_chance_bp = 5000\n"),
        "{written}"
    );
    assert!(
        written.contains("[tent]\nheal_bonus_bp = 10000\n"),
        "{written}"
    );
    // The magic: the spell book among the values, the masteries as a table, the bonuses.
    assert!(
        written.contains("spell_book = [\"synthetic:spark\"]\n"),
        "{written}"
    );
    assert!(
        written.contains("[masteries]\n\"synthetic:spark\" = 2\n"),
        "{written}"
    );
    assert!(
        written.contains(
            "[[effects]]\nkind = \"spell_damage\"\nbp = 5000\nschool = \"synthetic:stone\"\n"
        ),
        "{written}"
    );
    assert!(
        written.contains(
            "[[effects]]\nkind = \"spell_specialty\"\nspell = \"synthetic:spark\"\nbp_per_level = 300\nlevel = 12\n"
        ),
        "{written}"
    );
    let plain = toml::to_string(&HeroEntry::of(&ansel, &ruleset).unwrap()).unwrap();
    assert!(
        !plain.contains("_bp") && !plain.contains("ballista") && !plain.contains("tent"),
        "{plain}"
    );
    let mut stranger = expected.clone();
    stranger.hero_type = h3_content::HeroTypeId::from_raw(99);
    assert_eq!(
        HeroEntry::of(&stranger, &ruleset),
        Err(BattlePackError::UnknownHeroType("#99".to_owned()))
    );
    // Stack statistics name known creatures and flags.
    for (creatures, error) in [
        (
            "{ creature = \"synthetic:nobody\" }",
            BattlePackError::UnknownCreature("synthetic:nobody".to_owned()),
        ),
        (
            "{ flag = \"winged\" }",
            BattlePackError::from(CompileError::UnknownFlag("winged".to_owned())),
        ),
    ] {
        let text = format!(
            "{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\neffects = [{{ kind = \"stack_stat\", creatures = {creatures}, stat = \"speed\", op = \"add\", amount = 1 }}]\n"
        );
        let entry = parse_scenario(&text).unwrap().heroes[1].clone().unwrap();
        assert_eq!(entry.resolve(&ruleset), Err(error));
    }
    // Alignment groups name known factions, at most 16.
    let vale = "\"synthetic:vale\"";
    for (factions, error) in [
        (
            "\"synthetic:nowhere\"".to_owned(),
            BattlePackError::UnknownFaction("synthetic:nowhere".to_owned()),
        ),
        (
            [vale; 17].join(", "),
            BattlePackError::Syntax("alignment_group: more than 16 factions".to_owned()),
        ),
    ] {
        let text = format!(
            "{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\neffects = [{{ kind = \"alignment_group\", factions = [{factions}] }}]\n"
        );
        let entry = parse_scenario(&text).unwrap().heroes[1].clone().unwrap();
        assert_eq!(entry.resolve(&ruleset), Err(error));
    }
    let mut stranger_faction = expected.clone();
    stranger_faction.effects = vec![BattleEffect::AlignmentGroup {
        factions: FactionGroup::new(&[h3_content::FactionId::from_raw(99)]).unwrap(),
    }];
    assert_eq!(
        HeroEntry::of(&stranger_faction, &ruleset),
        Err(BattlePackError::UnknownFaction("#99".to_owned()))
    );
    let mut stranger_creature = expected.clone();
    stranger_creature.effects = vec![BattleEffect::StackStat {
        creatures: CreatureSelector::Creature(CreatureTypeId::from_raw(999)),
        stat: CreatureStat::Speed,
        op: StatOp::Add,
        amount: 1,
    }];
    assert_eq!(
        HeroEntry::of(&stranger_creature, &ruleset),
        Err(BattlePackError::UnknownCreature("#999".to_owned()))
    );
    // Spells and schools outside the ruleset.
    type Change = (fn(&mut BattleHero), BattlePackError);
    let unknown_spell = || BattlePackError::UnknownSpell("#99".to_owned());
    let changes: [Change; 6] = [
        (
            |h| h.magic.spell_book = Some(vec![SpellId::from_raw(99)]),
            unknown_spell(),
        ),
        (
            |h| {
                h.magic.eagle_eye = Some(EagleEye {
                    chance: Rate::bp(4000),
                    spells: vec![SpellId::from_raw(99)],
                });
            },
            unknown_spell(),
        ),
        (
            |h| {
                h.effects = vec![BattleEffect::OpeningSpell {
                    spell: SpellId::from_raw(99),
                    mastery: 3,
                    power: 1,
                }];
            },
            unknown_spell(),
        ),
        (
            |h| h.magic.masteries = vec![(SpellId::from_raw(99), 1)],
            unknown_spell(),
        ),
        (
            |h| {
                h.effects = vec![BattleEffect::SpellDamage {
                    scope: SpellScope::School(SchoolId::from_raw(9)),
                    rate: Rate::bp(1),
                }];
            },
            BattlePackError::UnknownSchool("#9".to_owned()),
        ),
        (
            |h| {
                h.effects = vec![BattleEffect::SpellSpecialty {
                    spell: SpellId::from_raw(99),
                    bp_per_level: 1,
                    level: 1,
                }];
            },
            unknown_spell(),
        ),
    ];
    for (change, error) in changes {
        let mut hero = expected.clone();
        change(&mut hero);
        assert_eq!(HeroEntry::of(&hero, &ruleset), Err(error));
    }
    let entry = |extra: &str| {
        let text =
            format!("{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\n{extra}\n");
        parse_scenario(&text).unwrap().heroes[1].clone().unwrap()
    };
    for (extra, error) in [
        (
            "spell_book = [\"synthetic:nova\"]",
            BattlePackError::UnknownSpell("synthetic:nova".to_owned()),
        ),
        (
            "masteries = { \"synthetic:nova\" = 1 }",
            BattlePackError::UnknownSpell("synthetic:nova".to_owned()),
        ),
        (
            "[scenario.defender_hero.eagle_eye]\nchance_bp = 4000\nspells = [\"synthetic:nova\"]",
            BattlePackError::UnknownSpell("synthetic:nova".to_owned()),
        ),
        (
            "effects = [{ kind = \"opening_spell\", spell = \"synthetic:nova\", mastery = 3, power = 1 }]",
            BattlePackError::UnknownSpell("synthetic:nova".to_owned()),
        ),
        (
            "effects = [{ kind = \"spell_damage\", bp = 1, school = \"synthetic:storm\", spell = \"synthetic:spark\" }]",
            BattlePackError::Syntax("spell_damage takes `school` or `spell`, not both".to_owned()),
        ),
        (
            "effects = [{ kind = \"spell_specialty\", spell = \"synthetic:nova\", bp_per_level = 1, level = 1 }]",
            BattlePackError::UnknownSpell("synthetic:nova".to_owned()),
        ),
        (
            "effects = [{ kind = \"spell_damage\", bp = 1, school = \"synthetic:sky\" }]",
            BattlePackError::UnknownSchool("synthetic:sky".to_owned()),
        ),
    ] {
        assert_eq!(entry(extra).resolve(&ruleset), Err(error), "{extra}");
    }
}

#[test]
fn format_v2_fields_and_expansion_packs() {
    let base = compile(&[&SYNTHETIC_PACK], "synthetic:faithful").unwrap();
    assert_eq!(base.resources().len(), 2);
    assert_eq!(base.factions()[0].name, "Vale");
    // Terrain content (E03): terrains, native terrain, obstacles.
    let meadow = base.terrain_by_key("synthetic:meadow").unwrap();
    assert_eq!(base.terrains().len(), 2);
    assert_eq!(base.factions()[0].native_terrain, Some(meadow));
    let obstacles: Vec<(&str, bool, usize)> = base
        .obstacles()
        .iter()
        .map(|o| (o.key.as_str(), o.absolute, o.blocked.len()))
        .collect();
    assert_eq!(
        obstacles,
        [
            ("synthetic:boulder", false, 2),
            ("synthetic:crag", false, 3),
            ("synthetic:cairn", false, 2),
            ("synthetic:ridge", true, 4)
        ]
    );
    // Special battlefields (pack format v36, E04c T2b): the hallow's own obstacle, its rules.
    let hallow = base.battlefield_by_key("synthetic:hallow").unwrap();
    assert_eq!(base.obstacles()[2].battlefields, [hallow]);
    assert!(base.obstacles()[2].terrains.is_empty());
    let h = base.battlefield(hallow).unwrap();
    assert!(h.own_obstacles && h.morale[0].amount == 1 && h.expert_schools.len() == 1);
    let barrens = base
        .battlefield(base.battlefield_by_key("synthetic:barrens").unwrap())
        .unwrap();
    assert!(barrens.no_morale && barrens.no_luck && barrens.no_native);
    assert!(barrens.no_creature_casts && barrens.max_spell_level == 1);
    assert!(barrens.luck.is_empty() && barrens.blocked.is_empty() && !barrens.expert_other_casts);
    assert_eq!(base.obstacles()[1].blocked[2], (1, -1));
    let bad_ref =
        SYNTHETIC_PACK.replace("native_terrain = \"meadow\"", "native_terrain = \"Meadow\"");
    assert!(matches!(parse_pack(&bad_ref), Err(BattlePackError::Key(_))));
    let bad_obstacle = SYNTHETIC_PACK.replace("id = \"boulder\"", "id = \"Boulder\"");
    assert!(matches!(
        parse_pack(&bad_obstacle),
        Err(BattlePackError::Key(_))
    ));
    let bad_terrain = SYNTHETIC_PACK.replace("terrains = [\"meadow\"]", "terrains = [\"Meadow\"]");
    assert!(matches!(
        parse_pack(&bad_terrain),
        Err(BattlePackError::Key(_))
    ));
    let bad_faction = SYNTHETIC_PACK.replace("id = \"vale\"", "id = \"Vale\"");
    assert!(matches!(
        parse_pack(&bad_faction),
        Err(BattlePackError::Key(_))
    ));
    let brute = base.creature_by_key("synthetic:brute").unwrap();
    let info = base.creature_info(brute).unwrap();
    assert_eq!(
        (info.name.as_str(), info.plural.as_str()),
        ("Brute", "Brutes")
    );
    assert_eq!(info.level, 2);
    // Cost entries come in key order (`coin` before `shard`).
    let costs: Vec<(&str, u32)> = info
        .cost
        .iter()
        .map(|(r, n)| (base.resources()[r.index()].key.as_str(), *n))
        .collect();
    assert_eq!(costs, [("synthetic:coin", 120), ("synthetic:shard", 1)]);
    assert!(
        base.creature(brute)
            .unwrap()
            .flags
            .contains(CreatureFlag::TwoHex)
    );
    let wisp = base.creature_by_key("synthetic:wisp").unwrap();
    assert_eq!(base.creature_info(wisp).unwrap().faction, None);

    // The expansion may be listed first; the compiler loads its dependency first.
    let both = compile(
        &[&SYNTHETIC_EXT_PACK, &SYNTHETIC_PACK],
        "synthetic:faithful",
    )
    .unwrap();
    let ids: Vec<&str> = both.packs().iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["synthetic", "synthetic-ext"]);
    assert_eq!(both.creature_count(), 16);
    let wisp_info = both.creature_info(wisp).unwrap();
    let marsh = wisp_info.faction.unwrap();
    assert_eq!(
        both.factions()[marsh.index()].key.as_str(),
        "synthetic-ext:marsh"
    );
    assert_eq!(wisp_info.level, 2);
    let brute_info = both.creature_info(brute).unwrap();
    assert_eq!((brute_info.growth, brute_info.horde_growth), (8, 3));
    assert_eq!(brute_info.cost[0].1, 110);
    // Battle stats are untouched by these overrides.
    assert_eq!(both.creature(brute), base.creature(brute));

    // The expansion alone is rejected: its dependency is not enabled.
    assert_eq!(
        compile(&[&SYNTHETIC_EXT_PACK], "synthetic:faithful"),
        Err(BattlePackError::Compile(CompileError::MissingDependency {
            pack: "synthetic-ext".into(),
            depends: "synthetic".into()
        }))
    );
}

#[test]
fn harvested_packs_compile_in_every_allowed_combination() {
    let profile = "base:faithful";
    let roe = compile(&[&BASE_PACK], profile).unwrap();
    let ab = compile(&[&BASE_PACK, &AB_PACK], profile).unwrap();
    let sod = compile(&[&SOD_PACK, &AB_PACK, &BASE_PACK], profile).unwrap();
    // RoE: 8 towns x 14 + 6 neutrals + 5 war machines (the Catapult since E04c T6, the Arrow
    // Tower since T7); AB adds Conflux (10 new) and 13 neutrals.
    assert_eq!(roe.creature_count(), 123);
    assert_eq!(ab.creature_count(), 146);
    assert_eq!(sod.creature_count(), 146);
    assert_eq!((roe.factions().len(), ab.factions().len()), (8, 9));
    // SoD requires AB; AB requires base.
    assert!(matches!(
        compile(&[&BASE_PACK, &SOD_PACK], profile),
        Err(BattlePackError::Compile(
            CompileError::MissingDependency { .. }
        ))
    ));
    assert!(compile(&[&AB_PACK], profile).is_err());
    // Spot checks of the version differences (sources: crates/h3-harvest/data/gapfill.toml).
    let gems_of = |r: &BattleRuleset, key: &str| {
        let info = r.creature_info(r.creature_by_key(key).unwrap()).unwrap();
        info.cost
            .iter()
            .find(|(res, _)| r.resources()[res.index()].key.as_str() == "base:gems")
            .map_or(0, |(_, n)| *n)
    };
    assert_eq!(gems_of(&ab, "base:angel"), 0);
    assert_eq!(gems_of(&sod, "base:angel"), 1);
    let air = roe.creature_by_key("base:air_elemental").unwrap();
    assert_eq!(roe.creature_info(air).unwrap().faction, None);
    assert_eq!(roe.creature_info(air).unwrap().level, 4);
    assert_eq!(ab.creature_info(air).unwrap().level, 2);
    let lizard = roe.creature_by_key("base:lizardman").unwrap();
    assert_eq!(roe.creature_info(lizard).unwrap().growth, 9);
    let cerberus = roe.creature_by_key("base:cerberus").unwrap();
    assert_eq!(roe.creature(cerberus).unwrap().damage_max, 5);
    assert_eq!(sod.creature(cerberus).unwrap().damage_max, 7);
    assert_eq!(roe.battle().attack_bonus_cap_percent, 300);
    assert_eq!(roe.battle().formation_row(Formation::Loose, 7, 3), Some(5));
    assert_eq!(roe.battle().formation_row(Formation::Tight, 3, 0), Some(4));
    // Abilities (E03 T2): RoE elementals hate their opposites; AB adds the Conflux ones.
    // Four magic abilities follow (E04b T2d: one immunity, three vulnerabilities), and
    // Blind's immunity (E04b T4b).
    assert_eq!(roe.abilities(air).len(), 6);
    assert_eq!(ab.abilities(air).len(), 7);
    let cavalier = roe.creature_by_key("base:cavalier").unwrap();
    assert_eq!(
        roe.abilities(cavalier),
        [h3_content_battle::Ability::Jousting { percent_per_hex: 5 }]
    );
    // Hero content (E04a T3): RoE has 16 classes and 128 heroes, AB adds Conflux and 28
    // heroes, SoD Titan's Lightning Bolt.
    let counts = |r: &BattleRuleset| {
        (
            r.skills().len(),
            r.spells().len(),
            r.hero_classes().len(),
            r.hero_types().len(),
        )
    };
    assert_eq!(counts(&roe), (28, 80, 16, 128));
    assert_eq!(counts(&ab), (28, 80, 18, 156));
    assert_eq!(counts(&sod), (28, 81, 18, 156));
    assert_eq!(sod.spell_schools().len(), 4);
    // Hero parameters and the slots whose artifacts never go to the backpack (E04a T4a).
    let params = roe.hero_params().unwrap();
    assert_eq!(
        (params.primary_min, params.primary_max, params.backpack),
        ([0, 0, 1, 1], 99, 64)
    );
    let fixed: Vec<&str> = sod
        .artifact_slots()
        .iter()
        .filter(|s| !s.backpack)
        .map(|s| s.key.as_str())
        .collect();
    assert_eq!(
        fixed,
        [
            "base:war_machine_1",
            "base:war_machine_2",
            "base:war_machine_3",
            "base:war_machine_4",
            "base:spell_book"
        ]
    );
    let archery = sod
        .skill(sod.skill_by_key("base:archery").unwrap())
        .unwrap();
    assert_eq!(archery.effects(3), [HeroEffect::RangedDamage { bp: 5000 }]);
    let pikeman = roe.creature_by_key("base:pikeman").unwrap();
    let halberdier = roe.creature_by_key("base:halberdier").unwrap();
    assert_eq!(roe.upgrades(pikeman), [halberdier]);
    assert!(roe.upgrades(air).is_empty());
    let storm = ab.creature_by_key("ab:storm_elemental").unwrap();
    assert_eq!(ab.upgrades(air), [storm]);
    let orrin = roe
        .hero_type(roe.hero_type_by_key("base:orrin").unwrap())
        .unwrap();
    assert_eq!(orrin.army.len(), 3);
    assert!(sod.hero_type_by_key("ab:lord_haart_death_knight").is_some());
    // Artifacts (E04a T3b): RoE 127 with four misc positions, AB adds two, SoD the twelve
    // combinations and a fifth misc position.
    let misc = |r: &BattleRuleset| {
        r.artifact_slot(r.artifact_slot_by_key("base:misc").unwrap())
            .unwrap()
            .count
    };
    assert_eq!((roe.artifacts().len(), misc(&roe)), (127, 4));
    assert_eq!((ab.artifacts().len(), misc(&ab)), (129, 4));
    assert_eq!((sod.artifacts().len(), misc(&sod)), (141, 5));
    assert_eq!(sod.artifact_slots().len(), 14);
    let slot = |k: &str| sod.artifact_slot_by_key(k).unwrap();
    let alliance = sod
        .artifact(sod.artifact_by_key("sod:angelic_alliance").unwrap())
        .unwrap();
    assert_eq!(alliance.slot, Some(slot("base:right_hand")));
    assert_eq!(
        alliance.locks,
        [
            "base:torso",
            "base:feet",
            "base:neck",
            "base:left_hand",
            "base:head"
        ]
        .map(slot)
    );
    let sword = sod.artifact_by_key("base:sword_of_judgement").unwrap();
    assert_eq!(
        sod.combination_of(sword),
        sod.artifact_by_key("sod:angelic_alliance")
    );
    let gladius = sod
        .artifact(sod.artifact_by_key("base:titans_gladius").unwrap())
        .unwrap();
    assert_eq!(
        gladius.effects,
        [HeroEffect::PrimarySkills {
            attack: 12,
            defense: -3,
            power: 0,
            knowledge: 0
        }]
    );
    let ballista = sod
        .artifact(sod.artifact_by_key("base:ballista").unwrap())
        .unwrap();
    assert_eq!(ballista.war_machine, sod.creature_by_key("base:ballista"));
    let bow = sod
        .artifact(sod.artifact_by_key("base:bow_of_elven_cherrywood").unwrap())
        .unwrap();
    assert_eq!(bow.needs_skill, sod.skill_by_key("base:archery"));
}

#[test]
fn override_syntax() {
    let with = |extra: &str| SYNTHETIC_EXT_PACK.core_with(extra);
    // An override's two halves, one in each file.
    let full = with(
        "[[creature_override]]\ntarget = \"synthetic:spearman\"\nspeed = 5\nname = \"Pike\"\n\
         plural = \"Pikes\"\nfaction = \"\"\nhorde_growth = 1\nfight_value = 1\nai_value = 2\n\
         adventure_count = [1, 2]\nflags = [\"undead\"]\n",
    )
    .battle_with(
        "[[creature_override]]\ntarget = \"synthetic:spearman\"\nattack = 9\ndefense = 9\n\
         damage = [2, 6]\nhit_points = 12\nshots = 1\ncasts = 2\n",
    );
    let r = compile(&[&SYNTHETIC_PACK, &full], "synthetic:faithful").unwrap();
    let id = r.creature_by_key("synthetic:spearman").unwrap();
    let stats = r.creature(id).unwrap();
    assert_eq!(
        (
            stats.attack,
            stats.damage_min,
            stats.damage_max,
            stats.speed
        ),
        (9, 2, 6, 5)
    );
    let info = r.creature_info(id).unwrap();
    assert_eq!(info.faction, None); // `faction = ""` makes it neutral
    assert_eq!(
        (info.name.as_str(), stats.shots, stats.casts),
        ("Pike", 1, 2)
    );
    assert!(stats.flags.contains(CreatureFlag::Undead));

    let bad_target = with("[[creature_override]]\ntarget = \"spearman\"\n");
    assert!(matches!(
        parse_pack(&bad_target),
        Err(BattlePackError::Key(_))
    ));
    let bad_ref =
        with("[[creature_override]]\ntarget = \"synthetic:spearman\"\nfaction = \"A:b\"\n");
    assert!(matches!(parse_pack(&bad_ref), Err(BattlePackError::Key(_))));
    let bad_cost =
        with("[[creature_override]]\ntarget = \"synthetic:spearman\"\ncost = { \"X\" = 1 }\n");
    assert!(matches!(
        parse_pack(&bad_cost),
        Err(BattlePackError::Key(_))
    ));
    let bad_resource = SYNTHETIC_PACK.replace("id = \"coin\"", "id = \"Coin\"");
    assert!(matches!(
        parse_pack(&bad_resource),
        Err(BattlePackError::Key(_))
    ));
    let bad_flag = SYNTHETIC_PACK.replace("flags = [\"alive\"]", "flags = [\"swims\"]");
    assert!(matches!(
        compile(&[&bad_flag], "synthetic:faithful"),
        Err(BattlePackError::Compile(CompileError::UnknownFlag(_)))
    ));
}

#[test]
fn gated_override_syntax() {
    // Pack format v34 (E04b T8e, ADR-0007): `fixes` gates a creature or hero override by a
    // data quirk flag; it applies only in profiles that turn the flag off.
    let keeps = SYNTHETIC_PACK.replace(
        "\"quirk.battle.periodic_first_stack\" = true }",
        "\"quirk.battle.periodic_first_stack\" = true, \
         \"quirk.content.elemental_levels\" = true, \
         \"quirk.content.roland_crusader_specialty\" = true }",
    );
    assert_ne!(keeps, *SYNTHETIC_PACK);
    let ext = SYNTHETIC_EXT_PACK.core_with(
        "[[creature_override]]\ntarget = \"synthetic:spearman\"\n\
         fixes = \"quirk.content.elemental_levels\"\nlevel = 7\n\n[[hero_override]]\n\
         target = \"synthetic:brea\"\nfixes = \"quirk.content.roland_crusader_specialty\"\n\
         specialty = [{ kind = \"creature_specialty\", creature = \"synthetic:wisp\", \
         bp_per_step = 500, speed = 1 }]\n",
    );
    let faithful = compile(&[&keeps, &ext], "synthetic:faithful").unwrap();
    let fixed = compile(&[&keeps, &ext], "synthetic:strict-fixed").unwrap();
    let spearman = fixed.creature_by_key("synthetic:spearman").unwrap();
    let level = |r: &BattleRuleset| r.creature_info(spearman).unwrap().level;
    assert_eq!((level(&faithful), level(&fixed)), (1, 7));
    let brea = fixed.hero_type_by_key("synthetic:brea").unwrap();
    let specialty = |r: &BattleRuleset, c: &str| {
        (
            r.hero_type(brea).unwrap().specialty.clone(),
            std::vec![HeroEffect::CreatureSpecialty {
                creature: r.creature_by_key(c).unwrap(),
                bp_per_step: 500,
                speed: 1,
            }],
        )
    };
    // `faithful` keeps the pack's specialty, `strict-fixed` takes the override's.
    let (got, want) = specialty(&faithful, "synthetic:brute");
    assert_eq!(got, want);
    let (got, want) = specialty(&fixed, "synthetic:wisp");
    assert_eq!(got, want);
    // The gate names a data quirk flag; the target is a full key.
    let behaviour = ext.replace(
        "fixes = \"quirk.content.elemental_levels\"",
        "fixes = \"quirk.battle.obstacle_anchors\"",
    );
    assert!(matches!(
        compile(&[&keeps, &behaviour], "synthetic:faithful"),
        Err(BattlePackError::Compile(CompileError::NotADataQuirk(_)))
    ));
    let bad_target = ext.replace("target = \"synthetic:brea\"", "target = \"brea\"");
    assert_ne!(bad_target, ext);
    assert!(matches!(
        parse_pack(&bad_target),
        Err(BattlePackError::Key(_))
    ));
    let bad_effect = ext.replace("creature = \"synthetic:wisp\"", "creature = \"A:b\"");
    assert_ne!(bad_effect, ext);
    assert!(matches!(
        parse_pack(&bad_effect),
        Err(BattlePackError::Key(_))
    ));
}

#[test]
fn ability_syntax() {
    use h3_content_battle::{Ability, AreaShot, AuraTarget};
    // Every kind with its parameters; references follow the usual rules.
    let all = "abilities = [\
        { kind = \"hate\", creature = \"wisp\", percent = 50 }, \
        { kind = \"hate\", creature = \"synthetic:brute\", percent = 100 }, \
        { kind = \"jousting\", percent_per_hex = 5 }, { kind = \"charge_immunity\" }, \
        { kind = \"defense_reduction\", percent = 40 }, \
        { kind = \"death_blow\", chance_percent = 20 }, \
        { kind = \"life_drain\", percent = 100 }, { kind = \"regeneration\" }, \
        { kind = \"death_stare\", chance_percent = 10 }, \
        { kind = \"extra_retaliations\", count = 1 }, { kind = \"strike_and_return\" }, \
        { kind = \"teleports\" }, \
        { kind = \"morale_aura\", amount = 1, affects = \"allies\" }, \
        { kind = \"luck_aura\", amount = -1, affects = \"enemies\" }, \
        { kind = \"minimum_morale\", value = 1 }, { kind = \"minimum_luck\", value = 1 }, \
        { kind = \"no_distance_penalty\" }, { kind = \"first_aid\", min_hp = 1, max_hp = 25 }, \
        { kind = \"ammo_supply\" }]\n";
    // The war-machine abilities need flag war_machine.
    // The Spearmen's battle row (its `hit_points = 10` is the only one).
    let text =
        SYNTHETIC_PACK.battle_replace("hit_points = 10\n", &std::format!("hit_points = 10\n{all}"));
    assert!(matches!(
        compile(&[&text], "synthetic:faithful"),
        Err(BattlePackError::Compile(CompileError::InvalidCreature { reason, .. }))
            if reason == "ammo_supply and first_aid require flag war_machine"
    ));
    // (War machines have speed 0, BAT-WAR-001.)
    let text = &text
        .replace(
            "flags = [\"alive\"]\n",
            "flags = [\"alive\", \"war_machine\"]\n",
        )
        .replace("speed = 4\n", "speed = 0\n");
    let r = compile(&[text], "synthetic:faithful").unwrap();
    let spearman = r.creature_by_key("synthetic:spearman").unwrap();
    let wisp = r.creature_by_key("synthetic:wisp").unwrap();
    let list = r.abilities(spearman);
    assert_eq!(list.len(), 19);
    assert_eq!(
        list[17],
        Ability::FirstAid {
            min_hp: 1,
            max_hp: 25
        }
    );
    assert_eq!(list[18], Ability::AmmoSupply);
    assert_eq!(
        list[0],
        Ability::Hate {
            target: wisp,
            percent: 50
        }
    );
    assert_eq!(
        list[13],
        Ability::LuckAura {
            amount: -1,
            affects: AuraTarget::Enemies
        }
    );
    // Patterns and area shots need their flags; overrides replace the whole list.
    let ext = SYNTHETIC_EXT_PACK
        .core_with(
            "[[creature_override]]\ntarget = \"synthetic:wisp\"\n\
             flags = [\"multi_headed\", \"area_attack\"]\n\
             [[creature_override]]\ntarget = \"synthetic:brute\"\n\
             flags = [\"multi_headed\", \"area_attack\"]\n",
        )
        .battle_with(
            "[[creature_override]]\ntarget = \"synthetic:wisp\"\nabilities = [\
             { kind = \"attacks_all_adjacent\" }, { kind = \"area_shot\", effect = \"death_cloud\" }]\n\
             [[creature_override]]\ntarget = \"synthetic:brute\"\n\
             abilities = [{ kind = \"three_headed_attack\" }, { kind = \"unlimited_retaliations\" }, \
             { kind = \"area_shot\", effect = \"fireball\" }]\n",
        );
    let r = compile(&[&SYNTHETIC_PACK, &ext], "synthetic:faithful").unwrap();
    assert_eq!(
        r.abilities(wisp),
        [
            Ability::AttacksAllAdjacent,
            Ability::AreaShot {
                effect: AreaShot::DeathCloud
            }
        ]
    );
    let brute = r.creature_by_key("synthetic:brute").unwrap();
    assert_eq!(r.abilities(brute).len(), 3);
    // Strict parsing: unknown kinds, unknown or missing parameters, bad references.
    for bad in [
        "abilities = [{ kind = \"swims\" }]",
        "abilities = [{ kind = \"teleports\", range = 3 }]",
        "abilities = [{ kind = \"jousting\" }]",
        "abilities = [{ kind = \"morale_aura\", amount = 1, affects = \"everyone\" }]",
        "abilities = [{ kind = \"area_shot\", effect = \"lightning\" }]",
    ] {
        let text = SYNTHETIC_PACK.battle_replace(
            "hit_points = 10\n",
            &std::format!("hit_points = 10\n{bad}\n"),
        );
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Syntax(_))),
            "{bad}"
        );
    }
    let bad_ref = SYNTHETIC_PACK.battle_replace(
        "hit_points = 10\n",
        "hit_points = 10\nabilities = [{ kind = \"hate\", creature = \"A:b\", percent = 1 }]\n",
    );
    assert!(matches!(parse_pack(&bad_ref), Err(BattlePackError::Key(_))));
    let bad_override = SYNTHETIC_EXT_PACK.battle_with(
        "[[creature_override]]\ntarget = \"synthetic:wisp\"\n\
         abilities = [{ kind = \"hate\", creature = \"A:b\", percent = 1 }]\n",
    );
    assert!(matches!(
        parse_pack(&bad_override),
        Err(BattlePackError::Key(_))
    ));
    // A damage penalty names a known flag (BAT-DMG-010); an innate spell's reference follows
    // the rules (BAT-MAG-057, BAT-MAG-058).
    let with = |abilities: &str| {
        let text = SYNTHETIC_PACK.battle_replace(
            "hit_points = 10\n",
            &std::format!("hit_points = 10\nabilities = [{abilities}]\n"),
        );
        assert_ne!(text, *SYNTHETIC_PACK);
        parse_pack(&text)
    };
    assert!(matches!(
        with("{ kind = \"damage_penalty_by_flag\", flag = \"slimy\", percent = 50 }"),
        Err(BattlePackError::Compile(CompileError::UnknownFlag(f))) if f == "slimy"
    ));
    assert!(matches!(
        with("{ kind = \"innate_spell\", spell = \"A:b\", value = 20 }"),
        Err(BattlePackError::Key(_))
    ));
    assert!(matches!(
        with("{ kind = \"damage_penalty\", creature = \"A:b\", percent = 50 }"),
        Err(BattlePackError::Key(_))
    ));
}

// covers: HER-BAT-011, HER-BAT-012
#[test]
fn harvested_magic_derives() {
    use h3_heroes::{HeroDetails, derive_battle_hero};
    let sod = compile(&[&SOD_PACK, &AB_PACK, &BASE_PACK], "base:faithful").unwrap();
    let spell = |k: &str| sod.spell_by_key(k).unwrap();
    let details = |hero: &str, skills: &[(&str, u8)], worn: &[&str]| HeroDetails {
        hero_type: sod.hero_type_by_key(hero).unwrap(),
        level: 12,
        experience: 0,
        primary: [1, 1, 5, 5],
        skills: skills
            .iter()
            .map(|&(k, l)| (sod.skill_by_key(k).unwrap(), l))
            .collect(),
        worn: worn
            .iter()
            .map(|k| sod.artifact_by_key(k).unwrap().into())
            .collect(),
        backpack: vec![],
        spells: vec![spell("base:magic_arrow"), spell("base:chain_lightning")],
        mana: 50,
    };
    let school = |k: &str| {
        let i = sod.spell_schools().iter().position(|s| s.key.as_str() == k);
        SchoolId::from_raw(u16::try_from(i.unwrap()).unwrap())
    };
    // Solmyr with Expert Air Magic and Sorcery, the book and the Orb of the Firmament:
    // Expert air mastery, +15 % for every spell, +50 % for air spells, and his Chain
    // Lightning specialty at 3 % per level (wiki Hero specialty, SoD).
    let solmyr = details(
        "base:solmyr",
        &[("base:air_magic", 3), ("base:sorcery", 3)],
        &["base:spell_book", "base:orb_of_the_firmament"],
    );
    let hero = derive_battle_hero(&solmyr, &sod).unwrap();
    assert_eq!(hero.magic.spell_book, Some(solmyr.spells.clone()));
    // Expert Air: every air spell at Expert, nothing else (HER-SPL-003).
    assert!(
        hero.magic
            .masteries
            .contains(&(spell("base:chain_lightning"), 3))
    );
    assert!(
        hero.magic.masteries.iter().all(|&(s, m)| {
            m == 3 && sod.spell(s).unwrap().schools.contains(&school("base:air"))
        })
    );
    assert_eq!(
        hero.effects,
        [
            BattleEffect::SpellDamage {
                scope: SpellScope::All,
                rate: Rate::bp(1500)
            },
            BattleEffect::SpellDamage {
                scope: SpellScope::School(school("base:air")),
                rate: Rate::bp(5000)
            },
            BattleEffect::SpellSpecialty {
                spell: spell("base:chain_lightning"),
                bp_per_level: 300,
                level: 12
            },
        ]
    );
    // Batch 2b (SoD, plain HD): Mephala at level 10 with Expert Armorer reduces by 15 %
    // grown 5 % per level in single precision, 0.22500000894 (R6: 619 and 154, rules.rs);
    // Octavia's Basic Armorer is the skill's 5 % (R4: 189).
    let armorer = |hero: &str, level: u8, mastery: u8| {
        let mut d = details(hero, &[("base:armorer", mastery)], &[]);
        d.level = level;
        derive_battle_hero(&d, &sod).unwrap().damage.reduction
    };
    assert_eq!(
        armorer("base:mephala", 10, 3),
        Rate {
            bp: 2250,
            single: NonZeroU32::new(0x3e66_6667)
        }
    );
    assert_eq!(armorer("base:octavia", 10, 1), Rate::bp(500));
    // Ciele: +50 % Magic Arrow damage whatever her level (wiki Ciele); no book, no casting.
    let ciele = derive_battle_hero(&details("ab:ciele", &[], &[]), &sod).unwrap();
    assert_eq!(ciele.magic.spell_book, None);
    assert_eq!(
        ciele.effects,
        [BattleEffect::SpellDamage {
            scope: SpellScope::Spell(spell("base:magic_arrow")),
            rate: Rate::bp(5000)
        }]
    );
}

// covers: HER-ART-001, HER-ART-002, HER-WAR-001, HER-BAT-001, HER-BAT-002, HER-BAT-003
// covers: HER-BAT-004, HER-BAT-005, HER-BAT-010
#[test]
fn hero_details_fit_the_harvested_slots() {
    use h3_heroes::{DetailsError, HeroDetails, derive_battle_hero, validate, war_machines};
    let profile = "base:faithful";
    let ab = compile(&[&BASE_PACK, &AB_PACK], profile).unwrap();
    let sod = compile(&[&SOD_PACK, &AB_PACK, &BASE_PACK], profile).unwrap();
    let details = |r: &BattleRuleset, worn: &[&str], backpack: &[&str]| {
        let art = |k: &&str| r.artifact_by_key(k).unwrap();
        HeroDetails {
            hero_type: r.hero_type_by_key("base:orrin").unwrap(),
            level: 10,
            experience: 20_000,
            primary: [5, 4, 2, 2],
            skills: vec![(r.skill_by_key("base:archery").unwrap(), 3)],
            worn: worn.iter().map(art).map(Into::into).collect(),
            backpack: backpack.iter().map(art).map(Into::into).collect(),
            spells: vec![r.spell_by_key("base:magic_arrow").unwrap()],
            mana: 20,
        }
    };
    let machines = [
        "base:ballista",
        "base:ammo_cart",
        "base:first_aid_tent",
        "base:catapult",
        "base:spell_book",
    ];
    let misc = ["base:clover_of_fortune"; 5];
    let rings = ["base:quiet_eye_of_the_dragon"; 2];
    // SoD: Angelic Alliance (right hand) with five misc artifacts, both rings, every
    // machine and the book; the Grail in the backpack.
    let full: Vec<&str> = ["sod:angelic_alliance"]
        .into_iter()
        .chain(misc)
        .chain(rings)
        .chain(machines)
        .collect();
    let d = details(&sod, &full, &["base:the_grail"]);
    assert_eq!(validate(&d, &sod), Ok(()));
    let creature = |k: &str| sod.creature_by_key(k).unwrap();
    assert_eq!(
        war_machines(&d, &sod),
        [
            creature("base:ballista"),
            creature("base:ammo_cart"),
            creature("base:first_aid_tent")
        ]
    );
    // In battle the Alliance's six parts give +21 to every primary skill (wiki Angelic
    // Alliance), the two Quiet Eyes +1 attack and +1 defense each; Tactics from the skill.
    let hero = derive_battle_hero(&d, &sod).unwrap();
    assert_eq!(
        [hero.attack, hero.defense, hero.power, hero.knowledge],
        [28, 27, 23, 23]
    );
    assert_eq!(hero.tactics, 0);
    // Orrin's specialty grows Expert Archery's 50 % by 5 % of itself per level: 75 % at
    // level 10 (wiki Hero specialty); the Bow of the Sharpshooter's three parts add 30 %
    // (HER-BAT-004).
    assert_eq!(
        hero.damage,
        DamageBonuses {
            ranged: Rate::bp(7500),
            ..DamageBonuses::default()
        }
    );
    let bow = details(&sod, &["sod:bow_of_the_sharpshooter"], &[]);
    assert_eq!(
        derive_battle_hero(&bow, &sod).unwrap().damage.ranged.bp,
        10_500
    );
    // Artillery: Basic one shot at 50 % double damage, Expert two at 100 % (batch 1a R6, R7;
    // HER-BAT-005); without it Orrin does not command his Ballista.
    assert_eq!(hero.war_machines.ballista, None);
    let mut gunner = details(&sod, &["base:ballista"], &[]);
    let artillery = sod.skill_by_key("base:artillery").unwrap();
    for (level, shots, chance) in [(1, 1, 5000), (3, 2, 10_000)] {
        gunner.skills = vec![(artillery, level)];
        assert_eq!(
            derive_battle_hero(&gunner, &sod)
                .unwrap()
                .war_machines
                .ballista,
            Some(BallistaControl {
                shots,
                double_damage_chance_bp: chance
            })
        );
    }
    // First Aid: the tent's largest heal +100/200/300 %; Rion, a First Aid specialist, at
    // level 10 with Expert First Aid: 300 % × 1.5 (the executable: 25 × (1 + 3 × 1.5), 137).
    assert_eq!(hero.war_machines.tent, None);
    let first_aid = sod.skill_by_key("base:first_aid").unwrap();
    gunner.skills = vec![(first_aid, 1)];
    let tent = |d: &HeroDetails| derive_battle_hero(d, &sod).unwrap().war_machines.tent;
    assert_eq!(
        tent(&gunner),
        Some(TentControl {
            heal_bonus: Rate::bp(10_000)
        })
    );
    let rion = HeroDetails {
        hero_type: sod.hero_type_by_key("base:rion").unwrap(),
        skills: vec![(first_aid, 3)],
        ..gunner.clone()
    };
    assert_eq!(
        tent(&rion),
        Some(TentControl {
            heal_bonus: Rate::bp(45_000)
        })
    );
    // Five Clovers of Fortune give luck +1: identical artifacts do not stack (wiki Artifact).
    // Angelic Alliance lets the five SoD towns mix (HER-BAT-010) and casts Expert Prayer
    // with power 10 at the side's first activation (HER-BAT-017).
    let luck = |amount| BattleEffect::Luck {
        amount,
        affects: Affects::Own,
    };
    let towns = ["castle", "rampart", "tower", "fortress", "stronghold"]
        .map(|t| sod.faction_by_key(&format!("base:{t}")).unwrap());
    assert_eq!(
        hero.effects,
        [
            luck(1),
            BattleEffect::AlignmentGroup {
                factions: FactionGroup::new(&towns).unwrap()
            },
            BattleEffect::OpeningSpell {
                spell: sod.spell_by_key("base:prayer").unwrap(),
                mastery: 3,
                power: 10,
            }
        ]
    );
    // Expert Leadership (+3), Pendant of Courage (+3 each), Badge of Courage and two Clovers
    // (+1 each, once); Spirit of Oppression caps both sides' morale at 0 (the battle clamps
    // the sums to +3); the Badge's mind immunity (E04b T8b2).
    let mut leader = details(
        &sod,
        &[
            "base:spirit_of_oppression",
            "base:pendant_of_courage",
            "base:badge_of_courage",
            "base:clover_of_fortune",
            "base:clover_of_fortune",
        ],
        &[],
    );
    leader
        .skills
        .push((sod.skill_by_key("base:leadership").unwrap(), 3));
    assert_eq!(
        derive_battle_hero(&leader, &sod).unwrap().effects,
        [
            BattleEffect::Morale {
                amount: 7,
                affects: Affects::Own
            },
            luck(4),
            BattleEffect::MoraleAtMost {
                value: 0,
                affects: Affects::Both
            },
            BattleEffect::MindImmunity,
        ]
    );
    let mut tactician = d.clone();
    tactician
        .skills
        .push((sod.skill_by_key("base:tactics").unwrap(), 2));
    assert_eq!(derive_battle_hero(&tactician, &sod).unwrap().tactics, 2);
    // The Alliance locks the head; machines and the book stay out of the backpack.
    let head = details(
        &sod,
        &[&full[..], &["base:helm_of_heavenly_enlightenment"]].concat(),
        &[],
    );
    assert_eq!(
        validate(&head, &sod),
        Err(DetailsError::SlotFull(
            sod.artifact_slot_by_key("base:head").unwrap()
        ))
    );
    for k in machines {
        let d = details(&sod, &[], &[k]);
        assert_eq!(
            validate(&d, &sod),
            Err(DetailsError::NotForBackpack(
                sod.artifact_by_key(k).unwrap()
            ))
        );
    }
    let crowded = details(&sod, &[], &["base:clover_of_fortune"; 65]);
    assert_eq!(validate(&crowded, &sod), Err(DetailsError::BackpackFull));
    // Before SoD a hero has four misc positions.
    let five = details(&ab, &misc, &[]);
    assert_eq!(
        validate(&five, &ab),
        Err(DetailsError::SlotFull(
            ab.artifact_slot_by_key("base:misc").unwrap()
        ))
    );
    assert_eq!(validate(&details(&ab, &misc[1..], &[]), &ab), Ok(()));
}

// covers: HER-BAT-008, HER-BAT-009, HER-BAT-010, BAT-HER-005, BAT-HER-006, BAT-HER-007
/// E04a exit criterion 3: with the harvested packs `faithful` turns on every quirk flag that
/// exists and `strict-fixed` none, so the two profiles differ exactly by the quirks of the
/// ledger that have a flag, the data quirks' included (E04b T8e). The position of a retired
/// flag is no flag, and no profile sets its bit: 10 and 14 (BAT-QUIRK-2 and BAT-QUIRK-9,
/// retired on 2026-10-12) and 9 (BAT-QUIRK-1, retired on 2026-10-13).
#[test]
fn the_profiles_differ_exactly_by_the_quirk_flags() {
    let registry = h3_content::REGISTRY;
    let all = registry
        .flags()
        .fold(0u64, |bits, flag| bits | 1 << flag.index());
    assert_eq!(
        all,
        ((1 << registry.len()) - 1) & !(1 << 9 | 1 << 10 | 1 << 14)
    );
    for (profile, bits) in [("base:faithful", all), ("base:strict-fixed", 0)] {
        let r = compile(&[&SOD_PACK, &AB_PACK, &BASE_PACK], profile).unwrap();
        assert_eq!(r.quirks().bits(), bits, "{profile}");
    }
}

// covers: HER-BAT-008
/// E04b T8e (ADR-0007, amendment E04b T8e): the data quirks keep the executable's values in
/// `faithful`; their gated overrides fix them in `strict-fixed`. Roland's specialty names the
/// Swordsman there, as Sorsha's (Q-0006), and the Ice, Magma and Energy Elementals have their
/// base creatures' levels (BAT-QUIRK-19), with and without SoD.
#[test]
fn the_data_quirks_are_fixed_in_strict_fixed() {
    use h3_heroes::{HeroDetails, derive_battle_hero};
    let elementals = [
        "ab:ice_elemental",
        "ab:magma_elemental",
        "ab:energy_elemental",
    ];
    let effects = |r: &BattleRuleset, hero: &str| {
        let details = HeroDetails {
            hero_type: r.hero_type_by_key(hero).unwrap(),
            level: 4,
            experience: 0,
            primary: [0, 0, 1, 1],
            skills: vec![],
            worn: vec![],
            backpack: vec![],
            spells: vec![],
            mana: 0,
        };
        derive_battle_hero(&details, r).unwrap().effects
    };
    for packs in [
        &[&*SOD_PACK, &*AB_PACK, &*BASE_PACK][..],
        &[&*AB_PACK, &*BASE_PACK],
    ] {
        for (profile, levels, crusader, like_sorsha) in [
            ("base:faithful", [4, 3, 5], "base:crusader", false),
            ("base:strict-fixed", [3, 5, 4], "base:swordsman", true),
        ] {
            let r = compile(packs, profile).unwrap();
            let level = |k| {
                r.creature_info(r.creature_by_key(k).unwrap())
                    .unwrap()
                    .level
            };
            assert_eq!(elementals.map(level), levels, "{profile}");
            let roland = r.hero_type_by_key("ab:roland").unwrap();
            assert_eq!(
                r.hero_type(roland).unwrap().specialty,
                [HeroEffect::CreatureSpecialty {
                    creature: r.creature_by_key(crusader).unwrap(),
                    bp_per_step: 500,
                    speed: 1,
                }],
                "{profile}"
            );
            let same = effects(&r, "ab:roland") == effects(&r, "base:sorsha");
            assert_eq!(same, like_sorsha, "{profile}");
        }
    }
}

#[test]
fn harvested_hero_effects_reproduce_the_experiments() {
    use h3_battle::{BattleState, Stack, StackTraits};
    use h3_contracts::{Army, ArmyStack};
    use h3_heroes::{HeroDetails, derive_battle_hero, war_machines};
    let sod = compile(&[&SOD_PACK, &AB_PACK, &BASE_PACK], "base:faithful").unwrap();
    let creature = |k: &str| sod.creature_by_key(k).unwrap();
    let to_army = |stacks: &[(&str, u32)]| Army {
        stacks: stacks
            .iter()
            .enumerate()
            .map(|(slot, &(k, count))| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: creature(k),
                count,
            })
            .collect(),
    };
    // The stacks of the hero's army and war machines in a battle it leads as attacker with
    // `army` against one Peasant.
    let battle = |hero: &str, level, primary, worn: &[&str], army: &[(&str, u32)]| {
        let details = HeroDetails {
            hero_type: sod.hero_type_by_key(hero).unwrap(),
            level,
            experience: 0,
            primary,
            skills: vec![],
            worn: worn
                .iter()
                .map(|k| sod.artifact_by_key(k).unwrap().into())
                .collect(),
            backpack: vec![],
            spells: vec![],
            mana: 0,
        };
        let setup = BattleSetup {
            attacker: to_army(army),
            defender: to_army(&[("ab:peasant", 1)]),
            obstacles: vec![],
            heroes: [Some(derive_battle_hero(&details, &sod).unwrap()), None],
            terrain: None,
            battlefield: None,
            anti_magic: false,
            siege: None,
            war_machines: [war_machines(&details, &sod), vec![]],
            formations: [Formation::Loose; 2],
            funds: [0; 2],
        };
        let state = BattleState::new(&sod, &setup, 1).unwrap();
        let stacks: Vec<Stack> = state.stacks().to_vec();
        stacks
            .into_iter()
            .filter(|s| s.side == BattleSide::Attacker)
            .collect::<Vec<_>>()
    };
    // (attack, defense, hit points, speed) of each.
    let stacks = |hero: &str, level: u8, primary: [u8; 4], worn: &[&str], army: &[(&str, u32)]| {
        battle(hero, level, primary, worn, army)
            .iter()
            .map(|s| (s.attack, s.defense, s.max_hp, s.speed))
            .collect::<Vec<_>>()
    };
    // (no range penalty, shooting while adjacent, regeneration) of each.
    let traits = |worn: &[&str], army: &[(&str, u32)]| {
        battle("base:saurug", 1, [0, 0, 1, 1], worn, army)
            .iter()
            .map(|s| {
                (
                    s.has(StackTraits::NO_DISTANCE_PENALTY),
                    s.has(StackTraits::SHOOTS_WHEN_ADJACENT),
                    s.regeneration,
                )
            })
            .collect::<Vec<_>>()
    };
    // Batch 1a R8: Trolls with the Elixir of Life have 54 hit points, 40 + 25 % + the parts'
    // flat 4; its regeneration restores at most 50 a turn to living creatures (Trolls
    // regenerate fully on their own, BAT-TRN-005), not to Skeletons.
    let elixir = ["sod:elixir_of_life"];
    let troll = stacks("base:saurug", 1, [0, 0, 1, 1], &elixir, &[("ab:troll", 20)]);
    assert_eq!(troll[0].2, 54);
    assert_eq!(
        traits(&elixir, &[("ab:troll", 20), ("base:skeleton", 5)]),
        [(false, false, 50), (false, false, 0)]
    );
    // Batch 1b R8: Pikemen 14 hit points and speed 7 (Ring of Vitality, Ring of Life, Vial of
    // Lifeblood +4; Necklace of Swiftness +1, Cape of Velocity +2); the war machines get the
    // hit points but not the speed: Ballista 254, Ammo Cart 104, First Aid Tent 79, speed 0.
    let worn = [
        "base:ring_of_life",
        "base:vial_of_lifeblood",
        "base:golden_bow",
        "base:necklace_of_swiftness",
        "base:ring_of_vitality",
        "base:cape_of_velocity",
        "base:ballista",
        "base:ammo_cart",
        "base:first_aid_tent",
    ];
    let r8 = stacks(
        "base:saurug",
        1,
        [4, 0, 1, 1],
        &worn,
        &[("base:pikeman", 60)],
    );
    assert_eq!(
        r8.iter().map(|s| (s.2, s.3)).collect::<Vec<_>>(),
        [(14, 7), (254, 0), (104, 0), (79, 0)]
    );
    // The Golden Bow takes the range penalty from the shooters, the Ballista included
    // (batch 1b R8: 23, 17 and 20 at long range); the Bow of the Sharpshooter also lets them
    // shoot next to an enemy.
    assert_eq!(
        traits(&worn, &[("base:archer", 10)])
            .iter()
            .map(|t| t.0)
            .collect::<Vec<_>>(),
        [true, true, false, false]
    );
    assert_eq!(
        traits(
            &["sod:bow_of_the_sharpshooter"],
            &[("base:archer", 10), ("base:pikeman", 1)]
        ),
        [(true, true, 0), (false, false, 0)]
    );
    // Batch 1b R1: Angelic Alliance lets the five SoD towns' creatures mix: Pikemen
    // (Castle), Centaurs (Rampart), Pixies (Conflux) and Peasants (neutral) are three
    // alignments (−1), without the Pixies or the Peasants two (0); without the Alliance four
    // (−2).
    let morale = |worn: &[&str], army: &[(&str, u32)]| {
        battle("base:caitlin", 1, [0, 0, 1, 1], worn, army)
            .iter()
            .map(|s| s.morale)
            .collect::<Vec<_>>()
    };
    let alliance = ["sod:angelic_alliance"];
    let all = [
        ("base:pikeman", 5),
        ("base:centaur", 5),
        ("ab:pixie", 5),
        ("ab:peasant", 5),
    ];
    assert_eq!(morale(&alliance, &all), [-1; 4]);
    assert_eq!(morale(&alliance, &all[..3]), [0; 3]);
    assert_eq!(morale(&alliance, &[all[0], all[1], all[3]]), [0; 3]);
    assert_eq!(morale(&[], &all), [-2; 4]);
    // Batch 1a B4-B6 at level 4 (one step of the creature level 4): Catherine's and Sorsha's
    // Swordsman specialty raises Swordsmen and Crusaders by +1 attack, +1 defense (5 % of
    // 10 or 12, rounded up) and +1 speed; Roland's names the Crusader only (quirk Q-0006,
    // the executable's value).
    let swordsmen = [("base:swordsman", 10), ("base:crusader", 10)];
    assert_eq!(
        stacks("ab:catherine", 4, [1, 1, 1, 1], &[], &swordsmen),
        [(12, 14, 35, 6), (14, 14, 35, 7)]
    );
    assert_eq!(
        stacks("ab:roland", 4, [2, 1, 1, 1], &[], &swordsmen),
        [(12, 13, 35, 5), (15, 14, 35, 7)]
    );
    assert_eq!(
        stacks("base:sorsha", 4, [2, 1, 1, 1], &[], &swordsmen),
        [(13, 14, 35, 6), (15, 14, 35, 7)]
    );
    // Batch 1b B7: Xeron's Devils 23 / 23 / 12 (+4 attack, +2 defense, +1 speed).
    assert_eq!(
        stacks("ab:xeron", 1, [0, 0, 1, 1], &[], &[("base:devil", 5)])[0],
        (23, 23, 160, 12)
    );
}

#[test]
fn artifact_syntax() {
    use h3_content::ArtifactClass;
    let r = compile(
        &[&SYNTHETIC_PACK, &SYNTHETIC_EXT_PACK],
        "synthetic:faithful",
    )
    .unwrap();
    assert_eq!(r.artifact_slots().len(), 5);
    let charm = r.artifact_slot_by_key("synthetic:charm").unwrap();
    let crown = r.artifact_slot_by_key("synthetic:crown").unwrap();
    assert_eq!(r.artifact_slot(charm).unwrap().count, 3);
    let engine = r.artifact_slot_by_key("synthetic:engine").unwrap();
    assert!(r.artifact_slot(charm).unwrap().backpack);
    assert!(!r.artifact_slot(engine).unwrap().backpack);
    let params = r.hero_params().unwrap();
    assert_eq!(
        (params.primary_min, params.primary_max, params.backpack),
        ([0, 0, 1, 2], 40, 3)
    );
    assert_eq!(r.artifacts().len(), 22);
    let by_key = |k: &str| r.artifact(r.artifact_by_key(k).unwrap()).unwrap();
    let regalia = by_key("synthetic-ext:storm_regalia");
    assert_eq!(regalia.class, ArtifactClass::Relic);
    assert_eq!(regalia.locks, vec![charm, charm, crown]);
    assert_eq!(regalia.parts.len(), 4);
    let lens = by_key("synthetic:sight_lens");
    assert_eq!(lens.needs_skill, r.skill_by_key("synthetic:marksmanship"));
    assert_eq!(lens.cost.len(), 1);
    assert_eq!(
        by_key("synthetic:bolt_thrower_kit").war_machine,
        r.creature_by_key("synthetic:bolt_thrower")
    );
    assert_eq!(by_key("synthetic:relic_stone").slot, None);
    assert_eq!(by_key("synthetic:vigor_stone").cost.len(), 2);
    // Without the expansion the slot keeps its own count.
    let base = compile(&[&SYNTHETIC_PACK], "synthetic:faithful").unwrap();
    assert_eq!(base.artifact_slot(charm).unwrap().count, 2);

    // Strict syntax and malformed references at every artifact site.
    let bad_syntax = [
        (
            "class = \"relic\"\ncost = { coin = 4000 }",
            "class = \"epic\"\ncost = { coin = 4000 }",
        ),
        ("count = 2   ", "count = \"two\""),
        ("id = \"engine\"", "id = \"engine\"\nsize = 1"),
        ("slot = \"engine\"", "slot = \"engine\"\nbroken = true"),
        (
            "{ kind = \"no_retreat\" }",
            "{ kind = \"no_retreat\", sides = 2 }",
        ),
    ];
    for (from, to) in bad_syntax {
        assert!(SYNTHETIC_PACK.contains(from), "{from}");
        let text = SYNTHETIC_PACK.replacen(from, to, 1);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Syntax(_))),
            "{to}"
        );
    }
    let bad_refs = [
        ("id = \"crown\"", "id = \"Crown\""),
        ("id = \"relic_stone\"", "id = \"Relic\""),
        ("cost = { coin = 4000 }", "cost = { \"C:x\" = 4000 }"),
        ("slot = \"engine\"", "slot = \"E:x\""),
        ("war_machine = \"bolt_thrower\"", "war_machine = \"B:x\""),
        ("needs_skill = \"marksmanship\"", "needs_skill = \"M:x\""),
        (
            "{ kind = \"no_range_penalty\", creatures = { flag = \"shooter\" } }",
            "{ kind = \"no_range_penalty\", creatures = { creature = \"S:x\" } }",
        ),
    ];
    for (from, to) in bad_refs {
        assert!(SYNTHETIC_PACK.contains(from), "{from}");
        let text = SYNTHETIC_PACK.replacen(from, to, 1);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Key(_))),
            "{to}"
        );
    }
    let ext_refs = [
        ("target = \"synthetic:charm\"", "target = \"T:x\""),
        (
            "\"synthetic:lucky_knot\", \"synthetic",
            "\"L:x\", \"synthetic",
        ),
        ("\"synthetic:vale\", \"marsh\"", "\"V:x\", \"marsh\""),
    ];
    for (from, to) in ext_refs {
        assert!(SYNTHETIC_EXT_PACK.contains(from), "{from}");
        let text = SYNTHETIC_EXT_PACK.replacen(from, to, 1);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Key(_))),
            "{to}"
        );
    }

    // The spell effects and the named-grant flag (pack format v30, E04b T8a).
    let bell = "{ kind = \"no_retreat\" },";
    let given = &SYNTHETIC_PACK
        .replacen(
            bell,
            "{ kind = \"spells\", school = \"storm\" }, { kind = \"spells\", level = 1 }, \
             { kind = \"spells\", spell = \"spark\" }, \
             { kind = \"spell_mastery\", spell = \"spark\", level = 3 }, { kind = \"scroll\" },",
            1,
        )
        .replacen(
            "name = \"Spark\"\n",
            "name = \"Spark\"\ngranted_by_name_only = true\n",
            1,
        );
    assert!(!given.contains(bell) && given.contains("granted_by_name_only"));
    let g = compile(&[given], "synthetic:faithful").unwrap();
    let spark = g.spell_by_key("synthetic:spark").unwrap();
    let storm = SchoolId::from_raw(0);
    assert!(g.spell(spark).unwrap().granted_by_name_only);
    let bell_effects = &g
        .artifact(g.artifact_by_key("synthetic:gloom_bell").unwrap())
        .unwrap()
        .effects;
    assert_eq!(
        bell_effects[2..],
        [
            HeroEffect::Spells {
                grant: SpellGrant::School(storm)
            },
            HeroEffect::Spells {
                grant: SpellGrant::Level(1)
            },
            HeroEffect::Spells {
                grant: SpellGrant::Spell(spark)
            },
            HeroEffect::SpellMastery {
                spell: spark,
                level: 3
            },
            HeroEffect::Scroll,
        ]
    );
    // The casting limits and the spell defences (pack format v31, E04b T8b).
    let limited = SYNTHETIC_PACK.replacen(
        bell,
        "{ kind = \"no_casting\" }, { kind = \"spell_level_limit\", level = 2 }, \
         { kind = \"spell_immunity\", spell = \"spark\" }, \
         { kind = \"spell_level_immunity\", level = 4 }, { kind = \"mind_immunity\" }, \
         { kind = \"negate_immunities\" },",
        1,
    );
    assert!(!limited.contains(bell));
    let l = compile(&[&limited], "synthetic:faithful").unwrap();
    assert_eq!(
        l.artifact(l.artifact_by_key("synthetic:gloom_bell").unwrap())
            .unwrap()
            .effects[2..],
        [
            HeroEffect::NoCasting,
            HeroEffect::SpellLevelLimit { level: 2 },
            HeroEffect::SpellImmunity {
                spell: l.spell_by_key("synthetic:spark").unwrap()
            },
            HeroEffect::SpellLevelImmunity { level: 4 },
            HeroEffect::MindImmunity,
            HeroEffect::NegateImmunities,
        ]
    );
    for to in [
        "{ kind = \"spells\" },",
        "{ kind = \"spells\", school = \"storm\", level = 1 },",
        "{ kind = \"scroll\", spell = \"spark\" },",
        "{ kind = \"no_casting\", level = 2 },",
        "{ kind = \"spell_level_limit\" },",
        "{ kind = \"mind_immunity\", spell = \"spark\" },",
        "{ kind = \"negate_immunities\", level = 1 },",
    ] {
        let text = SYNTHETIC_PACK.replacen(bell, to, 1);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Syntax(_))),
            "{to}"
        );
    }
    for to in [
        "{ kind = \"spells\", school = \"S:x\" },",
        "{ kind = \"spells\", spell = \"S:x\" },",
        "{ kind = \"spell_mastery\", spell = \"S:x\", level = 3 },",
        "{ kind = \"spell_immunity\", spell = \"S:x\" },",
        "{ kind = \"opening_spell\", spell = \"S:x\", mastery = 3, power = 1 },",
    ] {
        let text = SYNTHETIC_PACK.replacen(bell, to, 1);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Key(_))),
            "{to}"
        );
    }
}

#[test]
fn hero_content_syntax() {
    use h3_content::SpellCategory;
    let r = compile(
        &[&SYNTHETIC_PACK, &SYNTHETIC_EXT_PACK],
        "synthetic:faithful",
    )
    .unwrap();
    assert_eq!(r.spell_schools().len(), 2);
    assert_eq!(r.spells().len(), 47);
    // Pack format v6: the storm school's protection rank; stone's defaults to 0.
    let ranks: Vec<u8> = r
        .spell_schools()
        .iter()
        .map(|s| s.protection_rank)
        .collect();
    assert_eq!(ranks, [1, 0]);
    // Battle magic (format v4): every target kind, an area, a chain, target conditions.
    let magic = |k: &str| r.spell(r.spell_by_key(k).unwrap()).unwrap().magic.clone();
    let damage = vec![SpellEffect::Damage];
    assert_eq!(
        magic("synthetic:spark"),
        SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Enemy,
            effects: damage.clone(),
            ..SpellMagic::default()
        }
    );
    assert_eq!(
        magic("synthetic:cinders"),
        SpellMagic {
            target: SpellTarget::Hex,
            area: Some(SpellArea {
                inner: 0,
                radius: [1, 1, 1, 2]
            }),
            conditions: TargetConditions {
                requires: CreatureFlags::default(),
                excludes: CreatureFlags::default().with(CreatureFlag::WarMachine),
                needs_damage: false,
            },
            effects: damage.clone(),
            ..SpellMagic::default()
        }
    );
    assert_eq!(
        magic("synthetic:arc").chain,
        Some(SpellChain {
            targets: [3, 3, 4, 4],
            factor_bp: 5000
        })
    );
    let blight = magic("synthetic:blight");
    assert_eq!(blight.target, SpellTarget::None);
    assert!(blight.conditions.requires.contains(CreatureFlag::Alive));
    // Format v32: a condition on the creature's damage (Boon, as Bless).
    let boon = magic("synthetic:boon").conditions;
    assert!(boon.needs_damage && boon.excludes.contains(CreatureFlag::Undead));
    assert!(!blight.conditions.needs_damage);
    assert_eq!(magic("synthetic:ward"), SpellMagic::default());
    let ward = r.spell(r.spell_by_key("synthetic:ward").unwrap()).unwrap();
    assert_eq!((ward.level, ward.schools.len(), ward.power), (2, 2, 0));
    assert_eq!(ward.polarity, SpellPolarity::Positive);
    let glare = r.spell(r.spell_by_key("synthetic:glare").unwrap()).unwrap();
    assert_eq!(glare.category, SpellCategory::Ability);
    // Timed spells (format v5): changes, mass versions, mind, counters, a permanent and
    // cumulative spell.
    let id = |k: &str| r.spell_by_key(k).unwrap();
    let hasten = r.spell(id("synthetic:hasten")).unwrap();
    assert_eq!(
        hasten.magic,
        SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Own,
            conditions: TargetConditions {
                requires: CreatureFlags::default(),
                excludes: CreatureFlags::default().with(CreatureFlag::WarMachine),
                needs_damage: false,
            },
            effects: vec![SpellEffect::Timed(TimedChange::Speed)],
            mass_from: Some(3),
            specialist: Some(SpecialistBonus::CreatureLevel([3, 3, 2, 2, 1, 1, 0])),
            ..SpellMagic::default()
        }
    );
    assert_eq!(hasten.counters, vec![id("synthetic:drag")]);
    assert!(magic("synthetic:fury").mind);
    assert_eq!(magic("synthetic:zeal").effects.len(), 3);
    let corrode = magic("synthetic:corrode");
    assert_eq!(
        (corrode.duration, corrode.cumulative, corrode.mass_from),
        (EffectDuration::Permanent, true, None)
    );
    for (from, to, error) in [
        (
            "change = \"speed\" }]",
            "change = \"warp\" }]",
            "unknown timed change `warp`",
        ),
        (
            "mass = \"expert\"",
            "mass = \"grand\"",
            "unknown mastery `grand`",
        ),
    ] {
        let text = SYNTHETIC_PACK.replacen(from, to, 1);
        assert_eq!(
            parse_pack(&text).map(|_| ()),
            Err(BattlePackError::Syntax(error.into()))
        );
    }
    for mastery in ["basic", "advanced"] {
        let text =
            SYNTHETIC_PACK.replacen("mass = \"expert\"", &format!("mass = \"{mastery}\""), 1);
        assert!(parse_pack(&text).is_ok());
    }
    // Format v7: dispel and heal effects, the side by mastery, the dispel lists.
    let purge = r.spell(id("synthetic:purge")).unwrap();
    assert_eq!(
        purge.magic,
        SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Own,
            any_side_from: Some(2),
            needs_effect: true,
            ignores_immunity: true,
            mass_from: Some(3),
            effects: vec![SpellEffect::Dispel],
            clears_obstacles_from: Some(3),
            ..SpellMagic::default()
        }
    );
    assert_eq!(
        (purge.spares.clone(), purge.dispels.is_empty()),
        (vec![id("synthetic:drag")], true)
    );
    let mend = r.spell(id("synthetic:mend")).unwrap();
    assert_eq!(mend.magic.effects, [SpellEffect::Dispel, SpellEffect::Heal]);
    assert_eq!(
        mend.dispels,
        ["drag", "bane", "gloom"].map(|k| id(&format!("synthetic:{k}")))
    );
    // Format v8: Fire Shield and Frenzy, a duration by the stack's own turns.
    let rage = magic("synthetic:rage");
    assert_eq!(
        (rage.effects, rage.duration, rage.mind),
        (
            vec![SpellEffect::Timed(TimedChange::Frenzy)],
            EffectDuration::OwnTurn,
            true
        )
    );
    assert_eq!(
        magic("synthetic:blaze").effects,
        [SpellEffect::Timed(TimedChange::FireShield)]
    );
    // Format v10: Anti-Magic and Magic Mirror.
    assert_eq!(
        [
            magic("synthetic:veil").effects,
            magic("synthetic:glass").effects
        ],
        [
            vec![SpellEffect::Timed(TimedChange::SpellImmunity)],
            vec![SpellEffect::Timed(TimedChange::MagicMirror)]
        ]
    );
    // Format v11: Forgetfulness's fixed mass version and Blind's end by damage.
    let (haze, dazzle) = (magic("synthetic:haze"), magic("synthetic:dazzle"));
    assert_eq!(
        (haze.mass_from, haze.fixed_mass_from, haze.effects),
        (
            Some(2),
            Some(3),
            vec![SpellEffect::Timed(TimedChange::Forgetfulness)]
        )
    );
    assert_eq!(
        (dazzle.ends, dazzle.effects),
        (
            EffectEnd::Damaged,
            vec![SpellEffect::Timed(TimedChange::Blind)]
        )
    );
    assert_eq!(magic("synthetic:veil").ends, EffectEnd::Never);
    // Format v12: Berserk's fixed duration and end, Hypnotize's limit and refusal.
    let (rampage, charm) = (magic("synthetic:rampage"), magic("synthetic:charm"));
    assert_eq!(
        (rampage.duration, rampage.ends, rampage.target),
        (
            EffectDuration::Fixed(255),
            EffectEnd::OwnAttack,
            SpellTarget::Hex
        )
    );
    assert_eq!(
        (charm.health_limit, charm.refuses),
        (true, vec![TimedChange::MagicMirror])
    );
    // Format v13: obstacles, Teleport, Expert Dispel's clearing.
    let (bulwark, scorch) = (magic("synthetic:bulwark"), magic("synthetic:scorch"));
    assert_eq!(
        (
            bulwark.effects,
            bulwark.obstacle.map(|o| (o.hexes, o.mirrored))
        ),
        (
            vec![SpellEffect::Obstacle(ObstacleKind::Wall)],
            Some(([2, 2, 3, 3], false))
        )
    );
    assert_eq!(
        scorch.obstacle,
        Some(ObstacleShape {
            steps: vec![HexStep::TopRight, HexStep::TopLeft],
            hexes: [2, 2, 3, 3],
            random: false,
            mirrored: true,
            separate: true,
            rounds: 2,
        })
    );
    assert_eq!(magic("synthetic:blink").effects, [SpellEffect::Teleport]);
    // Format v14: random pits for the whole battle (`rounds` defaults to 0).
    let mire = magic("synthetic:mire");
    assert_eq!(
        (mire.target, mire.effects, mire.obstacle),
        (
            SpellTarget::None,
            vec![SpellEffect::Obstacle(ObstacleKind::Pit)],
            Some(ObstacleShape {
                steps: vec![],
                hexes: [4, 4, 6, 8],
                random: true,
                mirrored: false,
                separate: true,
                rounds: 0,
            })
        )
    );
    // Format v15: random mines (with the spell's value) for the whole battle.
    let snare = magic("synthetic:snare");
    assert_eq!(
        (
            snare.target,
            snare.effects,
            snare.obstacle.map(|o| o.random)
        ),
        (
            SpellTarget::None,
            vec![SpellEffect::Obstacle(ObstacleKind::Mine)],
            Some(true)
        )
    );
    assert_eq!(magic("synthetic:purge").clears_obstacles_from, Some(3));
    // Format v16: a remove effect with the masteries per obstacle kind (BAT-MAG-034).
    let uproot = magic("synthetic:uproot");
    assert_eq!(
        (uproot.target, uproot.effects, uproot.removes_from),
        (
            SpellTarget::Hex,
            vec![SpellEffect::RemoveObstacle],
            [Some(3), Some(2), Some(3), Some(3)]
        )
    );
    // Format v17: a raise effect, temporary below Advanced, and a sacrifice effect tested as
    // the raise spell under quirk BAT-QUIRK-5 (BAT-MAG-035, BAT-MAG-036).
    let revive = magic("synthetic:revive");
    assert_eq!(
        (
            revive.target,
            revive.effects.clone(),
            revive.temporary_below
        ),
        (SpellTarget::Stack, vec![SpellEffect::Raise], Some(2))
    );
    assert!(revive.raises() && revive.temporary_at(1) && !revive.temporary_at(2));
    let offer = r.spell(r.spell_by_key("synthetic:offer").unwrap()).unwrap();
    assert_eq!(
        (offer.magic.effects.clone(), offer.tests_as),
        (
            vec![SpellEffect::Sacrifice],
            r.spell_by_key("synthetic:revive")
        )
    );
    assert!(offer.magic.sacrifices() && !offer.magic.raises());
    // Format v18: a clone effect, a summon effect with the creature it summons, and the
    // side stack limit (BAT-MAG-037 to BAT-MAG-039).
    let copy = magic("synthetic:copy");
    assert_eq!(
        (copy.target, copy.effects.clone()),
        (SpellTarget::Stack, vec![SpellEffect::Clone])
    );
    let conjure = r
        .spell(r.spell_by_key("synthetic:conjure").unwrap())
        .unwrap();
    assert_eq!(
        (
            conjure.magic.target,
            conjure.magic.effects.clone(),
            conjure.summons
        ),
        (
            SpellTarget::None,
            vec![SpellEffect::Summon],
            r.creature_by_key("synthetic:shade")
        )
    );
    assert_eq!(r.battle().side_stack_limit, 20);
    for (from, to) in [
        ("fixed_rounds = 255\n", ""),
        (
            "duration = \"own_turn\"\n",
            "duration = \"own_turn\"\nfixed_rounds = 1\n",
        ),
        ("refuses = [\"magic_mirror\"]", "refuses = [\"mirror\"]"),
        ("obstacle = \"wall\"", "obstacle = \"trap\""),
        ("random = true", "random = 1"),
        (
            "steps = [\"top_right\", \"top_left\"], hexes = [2, 2, 3, 3], rounds",
            "steps = [\"up\"], hexes = [2, 2, 3, 3], rounds",
        ),
        (
            "clears_obstacles = \"expert\"",
            "clears_obstacles = \"master\"",
        ),
        (
            "removes = { fire = \"advanced\"",
            "removes = { trap = \"advanced\"",
        ),
        (
            "removes = { fire = \"advanced\"",
            "removes = { fire = \"master\"",
        ),
        (
            "temporary_below = \"advanced\"",
            "temporary_below = \"master\"",
        ),
        (
            "effects = [{ kind = \"sacrifice\" }]",
            "effects = [{ kind = \"sacrifice\", value = 1 }]",
        ),
        (
            "effects = [{ kind = \"clone\" }]",
            "effects = [{ kind = \"clone\", rounds = 1 }]",
        ),
        ("side_stack_limit = 20", "side_stack_limit = -1"),
    ] {
        assert!(SYNTHETIC_PACK.contains(from), "{from}");
        let text = SYNTHETIC_PACK.replacen(from, to, 1);
        assert!(
            matches!(
                compile(&[&text], "synthetic:faithful"),
                Err(BattlePackError::Syntax(_))
            ),
            "{to}"
        );
    }
    // Format v9: a specialist's rule of the spell's own.
    assert_eq!(
        [
            magic("synthetic:corrode").specialist,
            magic("synthetic:boon").specialist,
        ],
        [Some(SpecialistBonus::Fixed(2)), None]
    );
    let total = SYNTHETIC_PACK.replacen(
        "specialist = { kind = \"fixed\", amount = 2 }",
        "specialist = { kind = \"total\", value = 3 }",
        1,
    );
    let other = compile(&[&total], "synthetic:faithful").unwrap();
    let corrode = other
        .spell(other.spell_by_key("synthetic:corrode").unwrap())
        .unwrap();
    assert_eq!(corrode.magic.specialist, Some(SpecialistBonus::Total(3)));
    let bad = SYNTHETIC_PACK.battle_replace(
        "specialist = { kind = \"fixed\", amount = 2 }",
        "specialist = { kind = \"fixed\", amount = 2, value = 3 }",
    );
    assert!(matches!(parse_pack(&bad), Err(BattlePackError::Syntax(_))));
    let text = SYNTHETIC_PACK.replacen("any_side = \"advanced\"", "any_side = \"all\"", 1);
    assert_eq!(
        parse_pack(&text).map(|_| ()),
        Err(BattlePackError::Syntax("unknown mastery `all`".into()))
    );
    assert_eq!(r.skills().len(), 14);
    let omens = r.skill(r.skill_by_key("synthetic:omens").unwrap()).unwrap();
    assert_eq!(
        omens.effects(3)[1],
        HeroEffect::Luck {
            amount: -1,
            affects: Affects::Enemy
        }
    );
    assert_eq!(r.hero_classes().len(), 2);
    assert_eq!(r.hero_classes()[1].skill_weights.len(), 6);
    assert_eq!(r.hero_types().len(), 5);
    let wisp = r.creature_by_key("synthetic:wisp").unwrap();
    let corwin = r
        .hero_type(r.hero_type_by_key("synthetic:corwin").unwrap())
        .unwrap();
    assert_eq!(
        corwin.specialty[0],
        HeroEffect::CreatureStat {
            creatures: CreatureSelector::Creature(wisp),
            stat: CreatureStat::Attack,
            amount: 2
        }
    );
    assert_eq!(
        corwin.specialty[3],
        HeroEffect::CreatureStat {
            creatures: CreatureSelector::Flag(CreatureFlag::Flying),
            stat: CreatureStat::Defense,
            amount: 1
        }
    );
    let dara = r
        .hero_type(r.hero_type_by_key("synthetic:dara").unwrap())
        .unwrap();
    assert!(dara.spell_book && dara.spells.len() == 1 && dara.war_machines.len() == 1);
    let lurker = r.creature_by_key("synthetic-ext:lurker").unwrap();
    assert_eq!(r.upgrades(wisp), &[lurker]);
    // Every effect kind parses (the synthetic pack uses each at least once).
    let kinds: std::collections::BTreeSet<&str> = r
        .skills()
        .iter()
        .flat_map(|s| s.levels.iter().flatten())
        .chain(r.hero_types().iter().flat_map(|h| &h.specialty))
        .chain(r.artifacts().iter().flat_map(|a| &a.effects))
        .map(HeroEffect::name)
        .collect();
    assert_eq!(kinds.len(), HeroEffect::<u8, u8, u8, u8, u8>::NAMES.len());

    // Strict syntax: unknown kinds, parameters, flags and selectors are errors.
    let bad_syntax = [
        (
            "{ kind = \"ranged_damage\", bp = 1000 }]",
            "{ kind = \"ranged\", bp = 1000 }]",
        ),
        (
            "{ kind = \"ranged_damage\", bp = 1000 }]",
            "{ kind = \"ranged_damage\", percent = 10 }]",
        ),
        (
            "creatures = { flag = \"flying\" }",
            "creatures = { flag = \"flying\", creature = \"wisp\" }",
        ),
        (
            "creatures = { flag = \"flying\" }",
            "creatures = { kind = \"flying\" }",
        ),
        ("category = \"combat\"", "category = \"battle\""),
        ("polarity = \"neutral\"", "polarity = \"good\""),
        ("affects = \"enemy\"", "affects = \"allies\""),
        ("stat = \"speed\"", "stat = \"luck\""),
        (
            "level = 1 }, { skill = \"marksmanship\"",
            "lvl = 1 }, { skill = \"marksmanship\"",
        ),
        ("count = [4, 7] }", "count = [4, 7], slot = 1 }"),
        (
            "faction = \"vale\"\nprimary",
            "faction = \"vale\"\nprimary_skills = [1, 1, 1, 1]\nprimary",
        ),
        ("target = \"hex\"", "target = \"area\""),
        ("side = \"enemy\"", "side = \"foe\""),
        (
            "effects = [{ kind = \"damage\" }]",
            "effects = [{ kind = \"burn\" }]",
        ),
        (
            "effects = [{ kind = \"damage\" }]",
            "effects = [{ kind = \"damage\", value = 1 }]",
        ),
        ("area = { radius", "area = { shape = 1, radius"),
        ("chain = { targets", "chain = { jumps = 1, targets"),
        (
            "{ kind = \"spell_damage\", bp = 2500, spell = \"spark\" }",
            "{ kind = \"spell_damage\", bp = 2500, spell = \"spark\", school = \"storm\" }",
        ),
        (
            "{ kind = \"spell_book\" }",
            "{ kind = \"spell_book\", spells = 1 }",
        ),
    ];
    for (from, to) in bad_syntax {
        assert!(SYNTHETIC_PACK.contains(from), "{from}");
        let text = SYNTHETIC_PACK.replacen(from, to, 1);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Syntax(_))),
            "{to}"
        );
    }
    for (from, to) in [
        ("excludes = [\"war_machine\"]", "excludes = [\"winged\"]"),
        ("requires = [\"alive\"]", "requires = [\"winged\"]"),
    ] {
        assert_eq!(
            parse_pack(&SYNTHETIC_PACK.replace(from, to)),
            Err(BattlePackError::Compile(CompileError::UnknownFlag(
                "winged".into()
            )))
        );
    }
    let unknown_flag = SYNTHETIC_PACK.replace(
        "creatures = { flag = \"flying\" }",
        "creatures = { flag = \"winged\" }",
    );
    assert_eq!(
        parse_pack(&unknown_flag),
        Err(BattlePackError::Compile(CompileError::UnknownFlag(
            "winged".into()
        )))
    );
    // Malformed references at every reference site.
    let bad_refs = [
        ("schools = [\"storm\"]", "schools = [\"S:x\"]"),
        (
            "{ kind = \"ranged_damage\", bp = 1000 }]",
            "{ kind = \"creature_stat\", creatures = { creature = \"W:x\" }, stat = \"speed\", amount = 1 }]",
        ),
        (
            "skill = \"marksmanship\", bp_per_level",
            "skill = \"M:x\", bp_per_level",
        ),
        (
            "creature = \"brute\", bp_per_step",
            "creature = \"B:x\", bp_per_step",
        ),
        ("spell = \"spark\" }", "spell = \"S:x\" }"),
        (
            "spell = \"spark\", bp_per_level",
            "spell = \"S:x\", bp_per_level",
        ),
        (
            "school = \"storm\", level = 1",
            "school = \"S:x\", level = 1",
        ),
        ("school = \"storm\" }", "school = \"S:x\" }"),
        ("faction = \"vale\"\nprimary", "faction = \"V:x\"\nprimary"),
        ("marksmanship = 6", "\"M:x\" = 6"),
        ("class = \"warden\"", "class = \"W:x\""),
        ("{ skill = \"command\"", "{ skill = \"C:x\""),
        ("spells = [\"spark\"]", "spells = [\"S:x\"]"),
        (
            "{ creature = \"brute\", count",
            "{ creature = \"B:x\", count",
        ),
        ("war_machines = [\"aid_tent\"]", "war_machines = [\"A:x\"]"),
        ("id = \"ansel\"", "id = \"Ansel\""),
        ("id = \"warden\"", "id = \"Warden\""),
        ("id = \"drill\"", "id = \"Drill\""),
        ("id = \"spark\"", "id = \"Spark\""),
        ("id = \"storm\"", "id = \"Storm\""),
    ];
    for (from, to) in bad_refs {
        assert!(SYNTHETIC_PACK.contains(from), "{from}");
        let text = SYNTHETIC_PACK.replace(from, to);
        assert!(
            matches!(parse_pack(&text), Err(BattlePackError::Key(_))),
            "{to}"
        );
    }
    let bad_upgrade = SYNTHETIC_EXT_PACK.replace("upgrades = [\"lurker\"]", "upgrades = [\"L:x\"]");
    assert!(matches!(
        parse_pack(&bad_upgrade),
        Err(BattlePackError::Key(_))
    ));
    let bad_upgrade = SYNTHETIC_PACK.replacen(
        "flags = [\"alive\"]\n",
        "flags = [\"alive\"]\nupgrades = [\"U:x\"]\n",
        1,
    );
    assert!(matches!(
        parse_pack(&bad_upgrade),
        Err(BattlePackError::Key(_))
    ));
}

#[test]
fn error_display() {
    assert_eq!(BattlePackError::Syntax("x".into()).to_string(), "syntax: x");
    assert_eq!(
        BattlePackError::UnknownCreature("a:b".into()).to_string(),
        "unknown creature `a:b`"
    );
    let key = ContentKey::parse("bad").unwrap_err();
    assert!(
        BattlePackError::from(key)
            .to_string()
            .contains("not a valid key")
    );
    assert_eq!(
        BattlePackError::from(BattleCompileError::MissingBattleParams).to_string(),
        "no pack provides battle parameters"
    );
    assert!(matches!(
        BattlePackError::from(BattleCompileError::Content(CompileError::UnknownProfile(
            "a:b".into()
        ))),
        BattlePackError::Compile(CompileError::UnknownProfile(_))
    ));
    assert_eq!(
        BattlePackError::UnknownTerrain("a:t".into()).to_string(),
        "unknown terrain `a:t`"
    );
    assert_eq!(
        BattlePackError::UnknownFaction("a:f".into()).to_string(),
        "unknown faction `a:f`"
    );
    assert_eq!(
        BattlePackError::UnknownHeroType("a:h".into()).to_string(),
        "unknown hero type `a:h`"
    );
    assert_eq!(
        BattlePackError::UnknownSpell("a:s".into()).to_string(),
        "unknown spell `a:s`"
    );
    assert_eq!(
        BattlePackError::UnknownSchool("a:l".into()).to_string(),
        "unknown spell school `a:l`"
    );
}
