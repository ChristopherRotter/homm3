//! Experiment specs (E04a T2, OPEN-QUESTIONS Q28; `docs/experiments/README.md`).
//!
//! A spec lists *encounters*: a hero of the user's (or a second human player's) next to the
//! opponent it fights, with a procedure and the values to record. [`build`] compiles a spec into a
//! SoD map (written with [`h3_formats::h3m`]) and a results template the user fills in. Names
//! resolve against the original tables of the user's installation ([`Tables`]), so a spec reads
//! like the game ("Archangel", "Basic Armorer") and a typo fails at build time instead of in the
//! game.

pub mod tables;

use std::collections::BTreeMap;
use std::fmt::Write as _;

use h3_formats::h3m::{
    ARMY_SLOTS, ARTIFACT_SLOTS, Hero, HeroArtifacts, Map, MapObject, Monster, ObjectTemplate,
    PLAYERS, Player, Terrain,
};
use serde::Deserialize;

pub use tables::Tables;

/// Heroes a player may have on the map.
pub const MAX_HEROES_PER_PLAYER: usize = 8;
/// Horizontal distance between encounters on the map.
const COLUMN_STEP: u8 = 6;
/// Vertical distance between encounters on the map.
const ROW_STEP: u8 = 5;
/// First encounter column and row.
const ORIGIN: u8 = 3;
/// Slot names of a hero's artifacts, in map slot order.
pub const SLOT_NAMES: [&str; ARTIFACT_SLOTS] = [
    "head",
    "shoulders",
    "neck",
    "right hand",
    "left hand",
    "torso",
    "right ring",
    "left ring",
    "feet",
    "misc 1",
    "misc 2",
    "misc 3",
    "misc 4",
    "war machine 1",
    "war machine 2",
    "war machine 3",
    "war machine 4",
    "spell book",
    "misc 5",
];
/// Skill level names, 1-based.
const LEVELS: [&str; 3] = ["Basic", "Advanced", "Expert"];

/// A whole spec file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// The map.
    pub map: MapSpec,
    /// The encounters, in the order they are laid out and listed.
    #[serde(default)]
    pub encounter: Vec<Encounter>,
}

/// Map-wide settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapSpec {
    /// Name in the scenario list.
    pub name: String,
    /// Description in the scenario list.
    pub description: String,
    /// Width and height (36, 72, 108, 144).
    #[serde(default = "default_size")]
    pub size: u8,
    /// Terrain of the whole map (also the battlefield): dirt, sand, grass, snow, swamp, rough,
    /// subterranean, lava.
    pub terrain: String,
    /// General instructions at the top of the template (how to start the map, saving, …).
    #[serde(default)]
    pub instructions: Vec<String>,
}

fn default_size() -> u8 {
    36
}

/// One staged battle (or an inspection without a battle).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    /// Short id, also the hero's name in the game unless the hero has its own.
    pub id: String,
    /// What it settles.
    pub title: String,
    /// Question and dispute ids (OPEN-QUESTIONS, surveys).
    #[serde(default)]
    pub questions: Vec<String>,
    /// The hero the user plays.
    pub hero: HeroSpec,
    /// What the hero fights (none for an inspection).
    #[serde(default)]
    pub opponent: Option<Opponent>,
    /// Steps for the user, in order.
    #[serde(default)]
    pub procedure: Vec<String>,
    /// What the user writes down.
    #[serde(default)]
    pub record: Vec<Record>,
    /// Expected values per hypothesis; for the agent who evaluates the results, never shown in
    /// the template.
    #[serde(default)]
    pub predictions: Option<String>,
}

/// A hero on the map.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSpec {
    /// Hero type by name (or id, for names that occur twice).
    #[serde(rename = "type")]
    pub hero_type: NameOrId,
    /// The specialty the spec expects, as [`Tables::specialty_text`] words it; checked so that a
    /// hero with an interfering specialty is not picked by mistake.
    pub specialty: String,
    /// "red" or "blue".
    #[serde(default = "red")]
    pub owner: String,
    /// Name in the game (default: the encounter id).
    #[serde(default)]
    pub name: Option<String>,
    /// Experience (0 = level 1).
    #[serde(default)]
    pub experience: u32,
    /// Attack, defense, power, knowledge.
    pub primary: [u8; 4],
    /// Secondary skills as (name, level 1–3); replaces the hero type's.
    #[serde(default)]
    pub skills: Vec<(String, u8)>,
    /// Army as (creature, count), at most seven stacks.
    pub army: Vec<(String, u16)>,
    /// Worn artifacts by slot name ([`SLOT_NAMES`]); replaces the hero type's (so no spell book
    /// unless listed).
    #[serde(default)]
    pub artifacts: BTreeMap<String, String>,
    /// Tight formation.
    #[serde(default)]
    pub tight: bool,
}

fn red() -> String {
    "red".into()
}

/// A hero type by name or by id.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum NameOrId {
    /// `HOTRAITS.TXT` name.
    Name(String),
    /// Hero type id.
    Id(u8),
}

/// The opponent: a wandering monster or a hero of the second player.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opponent {
    /// Creature of a wandering monster.
    #[serde(default)]
    pub monster: Option<String>,
    /// Number of creatures of the monster.
    #[serde(default)]
    pub count: Option<u16>,
    /// A hero (owner "blue" by default for opponents).
    #[serde(default)]
    pub hero: Option<HeroSpec>,
}

/// Something to write down: a single value or a table.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// A single value ("Ballista shots in round 1").
    #[serde(default)]
    pub field: Option<String>,
    /// Column headings of a table.
    #[serde(default)]
    pub table: Option<Vec<String>>,
    /// Empty rows of the table.
    #[serde(default)]
    pub rows: Option<u8>,
}

/// Parses a spec.
///
/// # Errors
/// TOML syntax or unknown fields.
pub fn parse(text: &str) -> Result<Spec, String> {
    toml::from_str(text).map_err(|e| e.to_string())
}

/// Compiles a spec into a map and its results template. `map_file` is the map's file name as the
/// template mentions it.
///
/// # Errors
/// Unknown names, values outside the game's ranges, too many encounters or heroes.
pub fn build(spec: &Spec, tables: &Tables, map_file: &str) -> Result<(Map, String), String> {
    let size = spec.map.size;
    if ![36, 72, 108, 144].contains(&size) {
        return Err(format!("map size {size} (36, 72, 108 or 144)"));
    }
    let terrain = terrain(&spec.map.terrain)?;
    let columns = (size - 2 * ORIGIN) / COLUMN_STEP + 1;
    let capacity = usize::from(columns) * usize::from((size - 2 * ORIGIN) / ROW_STEP + 1);
    if spec.encounter.len() > capacity {
        return Err(format!(
            "{} encounters, a {size} map holds {capacity}",
            spec.encounter.len()
        ));
    }
    let mut objects = Vec::new();
    let mut used_types = Vec::new();
    let mut heroes_per_player = [0usize; PLAYERS];
    let mut text = String::new();
    let _ = writeln!(text, "# {} — results\n", spec.map.name);
    let _ = writeln!(
        text,
        "Map file: `{map_file}`. Fill in the blanks below and keep this file; an agent records the \
         results (`docs/experiments/README.md`).\n"
    );
    for (i, step) in spec.map.instructions.iter().enumerate() {
        let _ = writeln!(text, "{}. {step}", i + 1);
    }
    for (index, enc) in spec.encounter.iter().enumerate() {
        let index = u8::try_from(index).map_err(|_| "too many encounters".to_owned())?;
        let x = ORIGIN + (index % columns) * COLUMN_STEP;
        let y = ORIGIN + (index / columns) * ROW_STEP;
        let ctx = |e: String| format!("encounter {}: {e}", enc.id);
        let mut add_hero = |h: &HeroSpec, tile: (u8, u8), default_owner: &str| {
            let hero = hero(h, tables, tile, &enc.id, default_owner).map_err(ctx)?;
            if used_types.contains(&hero.hero_type) {
                return Err(ctx(format!(
                    "hero type {} is used twice",
                    tables.heroes[usize::from(hero.hero_type)]
                )));
            }
            used_types.push(hero.hero_type);
            let n = &mut heroes_per_player[usize::from(hero.owner)];
            *n += 1;
            if *n > MAX_HEROES_PER_PLAYER {
                return Err(ctx(format!(
                    "more than {MAX_HEROES_PER_PLAYER} heroes for one player"
                )));
            }
            let summary = hero_summary(h, &hero, tables);
            objects.push(MapObject::Hero(hero));
            Ok(summary)
        };
        let own = add_hero(&enc.hero, (x, y), "red")?;
        let against = match &enc.opponent {
            None => None,
            Some(Opponent {
                monster: Some(creature),
                count: Some(count),
                hero: None,
            }) => {
                let id = lookup(&tables.creatures, creature, "creature").map_err(ctx)?;
                let template = tables
                    .monsters
                    .iter()
                    .find(|(c, _)| *c == id as u32)
                    .map(|(_, t)| t.clone())
                    .ok_or_else(|| ctx(format!("no wandering-monster template for {creature}")))?;
                if *count == 0 {
                    return Err(ctx("a monster needs at least one creature".into()));
                }
                objects.push(monster(template, (x + 1, y), *count));
                Some(format!(
                    "{count} {creature} (wandering monster, never flees)"
                ))
            }
            Some(Opponent {
                monster: None,
                count: None,
                hero: Some(h),
            }) => Some(add_hero(h, (x + 1, y), "blue")?),
            Some(_) => {
                return Err(ctx(
                    "an opponent is either `monster` + `count` or `hero`".into()
                ));
            }
        };
        encounter_text(&mut text, enc, &own, against.as_deref());
    }
    let mut players: [Option<Player>; PLAYERS] = Default::default();
    // Both players may be human, so the map can be played in hot seat; the allowed towns are the
    // smoke map's (proven in the game).
    players[0] = Some(Player {
        human: true,
        computer: true,
        towns: 1,
    });
    if heroes_per_player[1] > 0 {
        players[1] = Some(Player {
            human: true,
            computer: true,
            towns: 1 << 6,
        });
    }
    if heroes_per_player.iter().skip(2).any(|&n| n > 0) {
        return Err("owners are red or blue".into());
    }
    let map = Map {
        name: spec.map.name.clone(),
        description: spec.map.description.clone(),
        size,
        terrain,
        players,
        objects,
    };
    Ok((map, text))
}

fn terrain(name: &str) -> Result<Terrain, String> {
    Ok(match name {
        "dirt" => Terrain::Dirt,
        "sand" => Terrain::Sand,
        "grass" => Terrain::Grass,
        "snow" => Terrain::Snow,
        "swamp" => Terrain::Swamp,
        "rough" => Terrain::Rough,
        "subterranean" => Terrain::Subterranean,
        "lava" => Terrain::Lava,
        other => return Err(format!("unknown terrain `{other}`")),
    })
}

/// Index of a unique name in a list (ASCII case ignored).
fn lookup(list: &[String], name: &str, what: &str) -> Result<usize, String> {
    let mut hits = list
        .iter()
        .enumerate()
        .filter(|(_, n)| n.eq_ignore_ascii_case(name.trim()));
    match (hits.next(), hits.next()) {
        (Some((i, _)), None) => Ok(i),
        (Some(_), Some(_)) => Err(format!("{what} `{name}` is ambiguous; use its id")),
        (None, _) => Err(format!("unknown {what} `{name}`")),
    }
}

fn hero(
    h: &HeroSpec,
    tables: &Tables,
    tile: (u8, u8),
    encounter: &str,
    default_owner: &str,
) -> Result<Hero, String> {
    let hero_type = match &h.hero_type {
        NameOrId::Name(n) => lookup(&tables.heroes, n, "hero")?,
        NameOrId::Id(id) => usize::from(*id),
    };
    let class = *tables
        .hero_classes
        .get(hero_type)
        .ok_or_else(|| format!("unknown hero type {hero_type}"))?;
    let specialty = tables.specialty_text(hero_type);
    if specialty != h.specialty {
        return Err(format!(
            "{} has the specialty `{specialty}`, the spec expects `{}`",
            tables.heroes[hero_type], h.specialty
        ));
    }
    let owner = match (h.owner.as_str(), default_owner) {
        ("red", "blue") => 1, // an opponent hero always belongs to the second player
        ("red", _) => 0,
        ("blue", _) => 1,
        (other, _) => return Err(format!("owner `{other}` (red or blue)")),
    };
    if h.skills.len() > 8 {
        return Err("at most 8 secondary skills".into());
    }
    let skills = h
        .skills
        .iter()
        .map(|(name, level)| {
            if !(1..=3).contains(level) {
                return Err(format!("skill level {level} (1–3)"));
            }
            Ok((lookup(&tables.skills, name, "skill")? as u8, *level))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if h.army.is_empty() || h.army.len() > ARMY_SLOTS {
        return Err(format!("{} army stacks (1–{ARMY_SLOTS})", h.army.len()));
    }
    let mut army = [None; ARMY_SLOTS];
    for (slot, (name, count)) in h.army.iter().enumerate() {
        if *count == 0 {
            return Err(format!("empty stack of {name}"));
        }
        army[slot] = Some((lookup(&tables.creatures, name, "creature")? as u16, *count));
    }
    let mut worn = [None; ARTIFACT_SLOTS];
    for (slot_name, artifact) in &h.artifacts {
        let slot = SLOT_NAMES
            .iter()
            .position(|s| s == slot_name)
            .ok_or_else(|| format!("unknown slot `{slot_name}`"))?;
        let names: Vec<String> = tables.artifacts.iter().map(|a| a.name.clone()).collect();
        let id = lookup(&names, artifact, "artifact")?;
        if !tables.artifacts[id].slots[slot] {
            return Err(format!("{artifact} cannot be worn in `{slot_name}`"));
        }
        worn[slot] = Some(id as u16);
    }
    Ok(Hero {
        tile,
        owner,
        hero_type: hero_type as u8,
        class,
        name: Some(h.name.clone().unwrap_or_else(|| encounter.to_owned())),
        experience: Some(h.experience),
        skills: Some(skills),
        army: Some(army),
        tight_formation: h.tight,
        artifacts: Some(HeroArtifacts {
            worn,
            backpack: Vec::new(),
        }),
        spells: Some([0; 9]),
        primary: Some(h.primary),
    })
}

fn monster(template: ObjectTemplate, tile: (u8, u8), count: u16) -> MapObject {
    MapObject::Monster(Monster {
        tile,
        template,
        count,
        disposition: 4, // savage: never joins
        never_flees: true,
        fixed_count: true,
    })
}

/// One line describing a hero as the user should see it in the game.
fn hero_summary(h: &HeroSpec, hero: &Hero, tables: &Tables) -> String {
    let owner = if hero.owner == 0 { "Red" } else { "Blue" };
    let [a, d, p, k] = h.primary;
    let skills = if h.skills.is_empty() {
        "no skills".to_owned()
    } else {
        h.skills
            .iter()
            .map(|(s, l)| format!("{} {s}", LEVELS[usize::from(*l) - 1]))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let army = h
        .army
        .iter()
        .map(|(c, n)| format!("{n} {c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut artifacts = h
        .artifacts
        .iter()
        .map(|(slot, a)| format!("{a} ({slot})"))
        .collect::<Vec<_>>()
        .join(", ");
    if artifacts.is_empty() {
        artifacts = "no artifacts".into();
    }
    let level = if h.experience == 0 {
        String::new()
    } else {
        format!(", {} experience", h.experience)
    };
    format!(
        "{owner} hero **{}** ({}, specialty {}{level}): attack {a}, defense {d}, power {p}, \
         knowledge {k}; {skills}; {artifacts}; army {army}",
        hero.name.as_deref().unwrap_or_default(),
        tables.heroes[usize::from(hero.hero_type)],
        h.specialty,
    )
}

fn encounter_text(out: &mut String, enc: &Encounter, own: &str, against: Option<&str>) {
    let questions = if enc.questions.is_empty() {
        String::new()
    } else {
        format!(" ({})", enc.questions.join(", "))
    };
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    let _ = writeln!(out, "## {} — {}{questions}\n", enc.id, enc.title);
    let _ = writeln!(out, "- {own}");
    if let Some(against) = against {
        let _ = writeln!(out, "- Opponent: {against}");
    }
    let _ = writeln!(out);
    for (i, step) in enc.procedure.iter().enumerate() {
        let _ = writeln!(out, "{}. {step}", i + 1);
    }
    if enc.record.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n**Record**\n");
    for r in &enc.record {
        if let Some(field) = &r.field {
            let _ = writeln!(out, "- {field}: ");
        }
        if let Some(columns) = &r.table {
            if !out.ends_with("\n\n") {
                out.push('\n');
            }
            let _ = writeln!(out, "| {} |", columns.join(" | "));
            let _ = writeln!(out, "|{}", "---|".repeat(columns.len()));
            for _ in 0..r.rows.unwrap_or(1) {
                let _ = writeln!(out, "|{}", "   |".repeat(columns.len()));
            }
            let _ = writeln!(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::{Artifact, Specialty};

    fn tables() -> Tables {
        let mut slots = [false; ARTIFACT_SLOTS];
        slots[13] = true;
        Tables {
            creatures: vec!["Pikeman".into(), "Archangel".into()],
            heroes: vec![
                "Orrin".into(),
                "Sylvia".into(),
                "Twin".into(),
                "Twin".into(),
            ],
            hero_classes: vec![0, 0, 1, 1],
            specialties: vec![
                Specialty { kind: 0, id: 0 },
                Specialty { kind: 2, id: 6 },
                Specialty { kind: 1, id: 1 },
                Specialty { kind: -1, id: 0 },
            ],
            skills: vec!["Archery".into(), "Armorer".into()],
            artifacts: vec![Artifact {
                name: "Ballista".into(),
                slots,
            }],
            monsters: vec![(
                0,
                ObjectTemplate::from_objects_txt(
                    "AvWPike.def 011111111111111111111111111111111111111111111111 \
                     100000000000000000000000000000000000000000000000 111111111 000000100 54 0 2 0",
                )
                .expect("template"),
            )],
        }
    }

    const SPEC: &str = r#"
        [map]
        name = "Test"
        description = "d"
        terrain = "dirt"
        instructions = ["Start it."]

        [[encounter]]
        id = "R1"
        title = "A test"
        questions = ["Q23"]
        procedure = ["Attack."]
        predictions = "secret"
        [encounter.hero]
        type = "Sylvia"
        specialty = "resource gold"
        primary = [4, 0, 1, 1]
        skills = [["Armorer", 1]]
        army = [["Archangel", 2]]
        artifacts = { "war machine 1" = "Ballista" }
        [encounter.opponent]
        monster = "Pikeman"
        count = 10
        [[encounter.record]]
        table = ["stack", "damage"]
        rows = 2
        [[encounter.record]]
        field = "anything odd"

        [[encounter]]
        id = "B1"
        title = "Second"
        [encounter.hero]
        type = "Orrin"
        specialty = "skill Archery"
        primary = [0, 0, 0, 0]
        army = [["Pikeman", 1]]
        [encounter.opponent.hero]
        type = 2
        specialty = "creature Archangel"
        primary = [1, 2, 3, 4]
        army = [["Pikeman", 5]]
    "#;

    #[test]
    fn builds_map_and_template() {
        let spec = parse(SPEC).expect("spec");
        let (map, text) = build(&spec, &tables(), "t.h3m").expect("build");
        assert_eq!(map.objects.len(), 4);
        let MapObject::Hero(h) = &map.objects[0] else {
            panic!("hero first")
        };
        assert_eq!((h.hero_type, h.owner, h.tile), (1, 0, (3, 3)));
        assert_eq!(h.skills, Some(vec![(1, 1)]));
        assert_eq!(h.army.expect("army")[0], Some((1, 2)));
        assert_eq!(h.artifacts.as_ref().expect("artifacts").worn[13], Some(0));
        assert_eq!(h.spells, Some([0; 9]));
        let MapObject::Monster(m) = &map.objects[1] else {
            panic!("monster second")
        };
        assert_eq!((m.tile, m.count), ((4, 3), 10));
        let MapObject::Hero(blue) = &map.objects[3] else {
            panic!("blue hero")
        };
        assert_eq!((blue.owner, blue.hero_type, blue.tile), (1, 2, (10, 3)));
        assert!(map.players[1].is_some());
        assert!(text.contains("## R1 — A test (Q23)"));
        assert!(text.contains("| stack | damage |"));
        assert!(text.contains("- anything odd: "));
        assert!(text.contains("; Basic Armorer;"));
        assert!(!text.contains("secret"));
        h3_formats::h3m::write(&map).expect("writable");
    }

    #[test]
    fn rejects_mistakes() {
        let t = tables();
        let fails = |from: &str, to: &str| {
            let spec = parse(&SPEC.replacen(from, to, 1)).expect("spec");
            build(&spec, &t, "t.h3m").expect_err(to)
        };
        assert!(fails("resource gold", "resource gems").contains("specialty"));
        assert!(fails("\"Archangel\", 2", "\"Angel\", 2").contains("unknown creature"));
        assert!(fails("[\"Armorer\", 1]", "[\"Armorer\", 4]").contains("skill level"));
        assert!(fails("\"war machine 1\"", "\"head\"").contains("cannot be worn"));
        assert!(fails("type = 2", "type = \"Twin\"").contains("ambiguous"));
        assert!(fails("type = 2", "type = \"Sylvia\"").contains("specialty"));
        assert!(fails("terrain = \"dirt\"", "terrain = \"water\"").contains("terrain"));
        assert!(fails("count = 10", "count = 0").contains("at least one"));
    }
}
