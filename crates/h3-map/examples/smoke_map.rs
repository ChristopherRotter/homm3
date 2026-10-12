//! Writes the E04a T2 smoke map: two heroes next to each other and a stack of Pikemen, to check
//! that the game accepts our `.h3m` files (protocol in `docs/experiments/README.md`).
//!
//! Usage: `cargo run -p h3-map --example smoke_map -- <Objects.txt> <output.h3m>`, where
//! `Objects.txt` is the SoD copy extracted from `H3bitmap.lod` (`original-data/extracted/`).

use std::process::ExitCode;

use h3_map::writer::{
    ARTIFACT_SLOTS, Hero, HeroArtifacts, Map, MapObject, Monster, ObjectTemplate, PLAYERS, Player,
    Terrain, gzip, write,
};

/// The Pikeman template line of `Objects.txt` (class 54, subclass 0).
fn pikeman_template(objects_txt: &str) -> Option<ObjectTemplate> {
    objects_txt
        .lines()
        .filter_map(|line| ObjectTemplate::from_objects_txt(line).ok())
        .find(|t| t.class == 54 && t.subclass == 0)
}

fn smoke_map(pikeman: ObjectTemplate) -> Map {
    let mut players: [Option<Player>; PLAYERS] = Default::default();
    players[0] = Some(Player {
        human: true,
        computer: true,
        towns: 1,
    }); // red, Castle
    players[1] = Some(Player {
        human: false,
        computer: true,
        towns: 1 << 6,
    }); // blue, Stronghold
    let mut red_artifacts = [None; ARTIFACT_SLOTS];
    red_artifacts[3] = Some(7); // right hand: Centaur's Axe (+2 attack)
    red_artifacts[13] = Some(4); // war machine 1: Ballista
    Map {
        name: "E04a smoke test".into(),
        description:
            "Engine-generated test map (E04a T2). Red: Smoke Red (Orrin) with 20 Archers, \
                      10 Pikemen, Advanced Archery, Basic Armorer, Centaur's Axe and a Ballista; \
                      attack 5+2, defense 4. Blue: Smoke Blue (Crag Hack) with 30 Goblins and 10 \
                      Wolf Riders, Basic Offense, attack 3, defense 3. Optional: 10 Pikemen south \
                      of Red."
                .into(),
        size: 36,
        terrain: Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects: vec![
            MapObject::Hero(Hero {
                tile: (10, 10),
                owner: 0,
                hero_type: 0, // Orrin
                class: 0,     // Knight
                name: Some("Smoke Red".into()),
                experience: Some(0),
                skills: Some(vec![(1, 2), (23, 1)]), // Advanced Archery, Basic Armorer
                army: Some([Some((2, 20)), Some((0, 10)), None, None, None, None, None]),
                tight_formation: false,
                artifacts: Some(HeroArtifacts {
                    worn: red_artifacts,
                    backpack: vec![],
                }),
                spells: None,
                primary: Some([5, 4, 1, 1]),
            }),
            MapObject::Hero(Hero {
                tile: (11, 10),
                owner: 1,
                hero_type: 102, // Crag Hack
                class: 12,      // Barbarian
                name: Some("Smoke Blue".into()),
                experience: Some(0),
                skills: Some(vec![(22, 1)]), // Basic Offense
                army: Some([Some((84, 30)), Some((86, 10)), None, None, None, None, None]),
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: Some([3, 3, 1, 1]),
            }),
            MapObject::Monster(Monster {
                tile: (10, 13),
                template: pikeman,
                count: 10,
                disposition: 4, // savage: never joins
                never_flees: true,
                fixed_count: true,
            }),
        ],
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [objects_txt, output] = &args[..] else {
        eprintln!("usage: smoke_map <Objects.txt> <output.h3m>");
        return ExitCode::FAILURE;
    };
    let text = match std::fs::read(objects_txt) {
        Ok(bytes) => bytes.iter().map(|&b| char::from(b)).collect::<String>(),
        Err(err) => {
            eprintln!("cannot read {objects_txt}: {err}");
            return ExitCode::FAILURE;
        }
    };
    let Some(pikeman) = pikeman_template(&text) else {
        eprintln!("{objects_txt} has no Pikeman template (class 54, subclass 0)");
        return ExitCode::FAILURE;
    };
    let bytes = match write(&smoke_map(pikeman)) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("cannot write the map: {err}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(dir) = std::path::Path::new(output).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(err) = std::fs::write(output, gzip(&bytes)) {
        eprintln!("cannot write {output}: {err}");
        return ExitCode::FAILURE;
    }
    println!("wrote {output} ({} bytes uncompressed)", bytes.len());
    ExitCode::SUCCESS
}
