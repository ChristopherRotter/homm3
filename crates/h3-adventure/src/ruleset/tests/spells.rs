//! Tests of the adventure spells' data (E08 T142a; increment 15's design note §3, ADV-SPL-001):
//! the bindings of core spells to the five behaviours, their checks, `[params.spells]` and the
//! part's hash.

use h3_content::{SpellCategory, SpellDef, SpellSchoolDef};

use super::*;

/// [`super::core`]'s packs with an air school and three spells in `t`: two of the adventure
/// category, `t:wings` and `t:wade`, and a combat one, `t:bolt`; `u` adds `u:mist`, an adventure
/// spell.
fn spell_core() -> Ruleset {
    let spell = |k: &str, category| SpellDef {
        key: key(k),
        name: "Spell".into(),
        category,
        level: 1,
        schools: vec![key("t:air")],
        cost: [5, 4, 4, 4],
        power: 0,
        effect: [0; 4],
        granted_by_name_only: false,
    };
    let t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        resources: vec![resource("t:wood"), resource("t:gold")],
        creatures: vec![
            creature("t:imp", 1, false),
            creature("t:gog", 2, false),
            creature("t:cart", 1, true),
        ],
        spell_schools: vec![SpellSchoolDef {
            key: key("t:air"),
            name: "Air".into(),
            protection_rank: 0,
        }],
        spells: vec![
            spell("t:wings", SpellCategory::Adventure),
            spell("t:wade", SpellCategory::Adventure),
            spell("t:bolt", SpellCategory::Combat),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    let u = ContentPack {
        terrains: vec![terrain("u:swamp")],
        resources: vec![resource("u:gem")],
        creatures: vec![creature("u:wisp", 1, false)],
        spells: vec![spell("u:mist", SpellCategory::Adventure)],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let v = ContentPack::new(manifest("v", &[]));
    Ruleset::compile(&[t, u, v], "t:faithful").unwrap()
}

fn bind(spell: &str, effect: AdventureSpell) -> AdventureSpellDef {
    AdventureSpellDef {
        spell: key(spell),
        effect,
    }
}

const SPELLS: SpellParams = SpellParams {
    door_points: [300, 300, 300, 200],
    target_range: [9, 8],
    boats_most: NonZeroU16::new(64).unwrap(),
};

/// [`packs`] with `bindings` in `pack` and `params` as the spells' parameters.
fn with(
    id: &str,
    bindings: Vec<AdventureSpellDef>,
    params: Option<SpellParams>,
) -> Result<AdventureRuleset, AdventureCompileError> {
    let packs = edit(|p| {
        pack(p, id).spells = bindings;
        pack(p, "t").params.as_mut().unwrap().spells = params;
    });
    AdventureRuleset::compile(&spell_core(), &packs)
}

// covers: ADV-SPL-001
#[test]
fn compiles_the_spells_bindings_and_their_parameters() {
    let core = spell_core();
    let id = |k: &str| core.spell_by_key(k).unwrap();
    let part = with(
        "u",
        vec![
            bind("u:mist", AdventureSpell::DimensionDoor),
            bind("t:wings", AdventureSpell::Fly),
        ],
        Some(SPELLS),
    )
    .unwrap();
    // In spell order, whatever the packs' order.
    assert_eq!(
        part.spells(),
        [
            (id("t:wings"), AdventureSpell::Fly),
            (id("u:mist"), AdventureSpell::DimensionDoor),
        ]
    );
    assert_eq!(
        part.adventure_spell(id("t:wings")),
        Some(AdventureSpell::Fly)
    );
    assert_eq!(part.adventure_spell(id("t:wade")), None);
    assert_eq!(part.adventure_spell(id("t:bolt")), None);
    assert_eq!(part.params().spells(), Some(&SPELLS));
    // Fly and Water Walk need no parameters.
    let plain = with("t", vec![bind("t:wings", AdventureSpell::Fly)], None).unwrap();
    assert_eq!(plain.params().spells(), None);
    // The part's hash sees the bindings and the parameters.
    let hashes = [
        with("t", vec![], None),
        with("t", vec![], Some(SPELLS)),
        with("t", vec![bind("t:wings", AdventureSpell::Fly)], None),
        with("t", vec![bind("t:wings", AdventureSpell::WaterWalk)], None),
        with("t", vec![bind("t:wade", AdventureSpell::Fly)], None),
        with(
            "t",
            vec![],
            Some(SpellParams {
                door_points: [300, 300, 300, 100],
                ..SPELLS
            }),
        ),
        with(
            "t",
            vec![],
            Some(SpellParams {
                target_range: [9, 9],
                ..SPELLS
            }),
        ),
        with(
            "t",
            vec![],
            Some(SpellParams {
                boats_most: NonZeroU16::new(63).unwrap(),
                ..SPELLS
            }),
        ),
    ]
    .map(|part| part.unwrap().content_hash());
    let distinct: BTreeSet<u64> = hashes.iter().copied().collect();
    assert_eq!(distinct.len(), hashes.len());
}

// covers: ADV-SPL-001
#[test]
fn each_binding_names_an_adventure_spell_of_the_closure_once() {
    let invalid = |key: &str, reason| AdventureCompileError::InvalidSpell {
        key: key.into(),
        reason,
    };
    let fly = || bind("t:wings", AdventureSpell::Fly);
    assert_eq!(
        with("t", vec![bind("t:ghost", AdventureSpell::Fly)], None),
        Err(AdventureCompileError::UnknownSpell("t:ghost".into()))
    );
    assert_eq!(
        with("t", vec![bind("u:mist", AdventureSpell::Fly)], None),
        Err(AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:mist".into(),
            pack: "t".into(),
        })
    );
    assert_eq!(
        with("t", vec![bind("t:bolt", AdventureSpell::Fly)], None),
        Err(invalid("t:bolt", "not of the adventure category"))
    );
    assert_eq!(
        with(
            "t",
            vec![fly(), bind("t:wings", AdventureSpell::WaterWalk)],
            None
        ),
        Err(invalid("t:wings", "bound twice"))
    );
    assert_eq!(
        with("t", vec![fly(), bind("t:wade", AdventureSpell::Fly)], None),
        Err(invalid("t:wade", "its effect is bound to another spell"))
    );
    // A behaviour that reads the parameters needs them; a window of 0 is refused.
    for effect in [
        AdventureSpell::SummonBoat,
        AdventureSpell::ScuttleBoat,
        AdventureSpell::DimensionDoor,
    ] {
        assert_eq!(
            with("t", vec![bind("t:wade", effect)], None),
            Err(AdventureCompileError::InvalidParams(
                "a bound Summon Boat, Scuttle Boat or Dimension Door needs [params.spells]"
            ))
        );
    }
    for target_range in [[0, 8], [9, 0]] {
        assert_eq!(
            with(
                "t",
                vec![],
                Some(SpellParams {
                    target_range,
                    ..SPELLS
                })
            ),
            Err(AdventureCompileError::InvalidParams(
                "a spells target_range entry is 0"
            ))
        );
    }
    assert_eq!(
        AdventureCompileError::UnknownSpell("t:ghost".into()).to_string(),
        "unknown spell `t:ghost`"
    );
    assert_eq!(
        invalid("t:bolt", "bound twice").to_string(),
        "adventure spell `t:bolt`: bound twice"
    );
}

// covers: ADV-SPL-001
#[test]
fn the_five_behaviours_have_their_keys_and_targets() {
    let keys = AdventureSpell::ALL.map(AdventureSpell::key);
    assert_eq!(
        keys,
        [
            "summon_boat",
            "scuttle_boat",
            "fly",
            "water_walk",
            "dimension_door"
        ]
    );
    for spell in AdventureSpell::ALL {
        assert_eq!(AdventureSpell::from_key(spell.key()), Some(spell));
    }
    assert_eq!(AdventureSpell::from_key("visions"), None);
    let targets = AdventureSpell::ALL.map(AdventureSpell::targets);
    assert_eq!(targets, [false, true, false, false, true]);
}
