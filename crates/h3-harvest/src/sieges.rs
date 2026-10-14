//! Sieges (E04c T5): the siege's field (`siege` in `[battle]`) and one `[[siege]]` record per
//! town type, from the executable's tables and instructions (`docs/formats/installation.md`
//! §7.27, §7.28). A record goes to the pack that defines its town. The wall shots (E04c T6,
//! §7.29): the targets, lists and the Catapult's hex from the executable, the Ballistics table
//! from `BALLIST.TXT`, each town's targets' hit points from `WALLS.TXT`.

use std::fmt::Write as _;

use h3_formats::exe::{SiegeData, WallShotData};
use h3_formats::text::TextTable;

use crate::heroes::reference;

/// Hit points a Castle adds to each wall (the four `inc` at 0x466348; `installation.md` §7.28).
pub const CASTLE_WALL_BONUS: u8 = 1;

/// Wall parts per town in `WALLS.TXT` (and the executable's 0x66D868).
const WALL_PARTS: usize = 18;

/// The wall shots' part of the siege's field and each town's targets' hit points (§7.29).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WallShots {
    /// The keys added to the `siege = { … }` line, from `catapult` on.
    pub field: String,
    /// Each town's targets' hit points, by town type.
    pub hit_points: Vec<Vec<u8>>,
}

/// The wall shots of the executable's tables `exe`, `WALLS.TXT` (`walls`) and `BALLIST.TXT`
/// (`ballistics`) for `towns` town types.
///
/// # Errors
/// A target's wall index outside the wall hexes `wall_hexes`, a table that is not the expected
/// shape: `WALLS.TXT` one block of 18 parts per town, `BALLIST.TXT` four levels of 4 chances,
/// shots and 3 damage chances.
pub fn wall_shots(
    exe: &WallShotData,
    wall_hexes: &[u8],
    (walls, ballistics): (&TextTable, &TextTable),
    towns: usize,
) -> Result<WallShots, String> {
    const KINDS: [&str; 4] = ["keep", "tower", "gate", "wall"];
    let list = |l: &[u8]| l.iter().map(u8::to_string).collect::<Vec<_>>().join(", ");
    let mut targets = Vec::new();
    for t in &exe.targets {
        let kind = KINDS[usize::from(t.kind)];
        match (t.wall, t.turret) {
            (None, None) => targets.push(format!("{{ kind = \"{kind}\" }}")),
            // A turret's target (E04c T7): the position of the turret it holds.
            (None, Some(at)) => targets.push(format!("{{ kind = \"{kind}\", turret = {at} }}")),
            (Some(_), Some(_)) => {
                return Err("Heroes3.exe: a wall-shot target with a wall and a turret".to_owned());
            }
            (Some(i), None) => {
                let hex = wall_hexes.get(usize::from(i)).ok_or_else(|| {
                    format!("Heroes3.exe: a wall-shot target's wall index {i} is no wall hex")
                })?;
                targets.push(format!("{{ kind = \"{kind}\", wall = {hex} }}"));
            }
        }
    }
    // BALLIST.TXT: the rows naming a level, its cells after the name.
    let percent = |cell: &str| cell.trim().trim_end_matches('%').trim().parse::<u8>().ok();
    let levels: Vec<String> = ballistics
        .records
        .iter()
        .filter(|r| {
            r.get(1)
                .is_some_and(|n| ["None", "Basic", "Advanced", "Expert"].contains(&n.trim()))
        })
        .map(|r| {
            let v: Vec<u8> = r
                .iter()
                .skip(2)
                .take(8)
                .filter_map(|c| percent(c))
                .collect();
            (v.len() == 8)
                .then(|| {
                    format!(
                        "{{ hit = [{}], shots = {}, damage = [{}] }}",
                        list(&v[..4]),
                        v[4],
                        list(&v[5..])
                    )
                })
                .ok_or_else(|| "BALLIST.TXT: a level row without 8 values".to_owned())
        })
        .collect::<Result<_, _>>()?;
    if levels.len() != 4 {
        return Err(format!("BALLIST.TXT: {} levels, not 4", levels.len()));
    }
    // WALLS.TXT: blocks of a town's name row and its parts' rows (name, hit points).
    let mut blocks: Vec<Vec<u8>> = Vec::new();
    for r in walls.records.iter().skip(2) {
        let name = r.first().map_or("", |c| c.trim());
        match r.get(1).and_then(|c| c.trim().parse::<u8>().ok()) {
            Some(v) => blocks
                .last_mut()
                .ok_or_else(|| "WALLS.TXT: a part before the first town".to_owned())?
                .push(v),
            None if !name.is_empty() => blocks.push(Vec::new()),
            None => {}
        }
    }
    if blocks.len() != towns || blocks.iter().any(|b| b.len() != WALL_PARTS) {
        return Err(format!(
            "WALLS.TXT: {} towns of {:?} parts, not {towns} of {WALL_PARTS}",
            blocks.len(),
            blocks.iter().map(Vec::len).collect::<Vec<_>>()
        ));
    }
    let hit_points = blocks
        .iter()
        .map(|b| exe.targets.iter().map(|t| b[usize::from(t.part)]).collect())
        .collect();
    Ok(WallShots {
        field: format!(
            "catapult = {}, turret_count = {}, targets = [{}], missed = [{}], automatic = [{}], \
             fallback = [{}], castle_bonus = {CASTLE_WALL_BONUS}, ballistics = [{}]",
            exe.catapult,
            exe.turret_count,
            targets.join(", "),
            list(&exe.missed),
            list(&exe.auto_walls),
            list(&exe.auto_others),
            levels.join(", ")
        ),
        hit_points,
    })
}

/// The text of the siege's field in `[battle]` and the records of `base` and `ab`.
#[derive(Debug, Default)]
pub struct Texts {
    /// The `siege = { … }` line of `[battle]`.
    pub field: String,
    /// `base`'s records.
    pub base: String,
    /// `ab`'s records.
    pub ab: String,
}

/// Writes the siege's field and records of `data`. `towns` are the `TownType.txt` keys with the
/// pack that defines each, `spells` each spell's key and pack by spell id.
///
/// # Errors
/// A damage table that does not list every town, a town type of the instructions without a town,
/// a mines' spell without a key or in a pack after its town's.
pub fn write(
    (data, shots): (&SiegeData, &WallShots),
    towns: &[(String, &'static str)],
    spells: &[(String, &'static str)],
) -> Result<Texts, String> {
    if data.moat_damage.len() != towns.len() {
        return Err(format!(
            "Heroes3.exe: {} moat damages for {} towns",
            data.moat_damage.len(),
            towns.len()
        ));
    }
    if shots.hit_points.len() != towns.len() {
        return Err(format!(
            "WALLS.TXT: {} towns' hit points for {} towns",
            shots.hit_points.len(),
            towns.len()
        ));
    }
    for (what, town) in [
        ("second moat", data.second_moat_town),
        ("mines", data.mines_town),
    ] {
        if usize::from(town) >= towns.len() {
            return Err(format!(
                "Heroes3.exe: the {what}' town type {town} is no town"
            ));
        }
    }
    let (mines, mines_pack) = spells.get(usize::from(data.mines_spell)).ok_or_else(|| {
        format!(
            "Heroes3.exe: the mines' spell {} is no spell",
            data.mines_spell
        )
    })?;
    let list = |hexes: &[u8]| {
        hexes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = Texts {
        field: format!(
            "siege = {{ walls = [{}], gate = {}, moat = [{}], second_moat = [{}], \
             moat_defense_penalty = {}, mine_power = {}, {} }}\n",
            list(&data.walls),
            data.gate,
            list(&data.moat),
            list(&data.second_moat),
            data.moat_defense_penalty,
            data.mine_power,
            shots.field
        ),
        ..Texts::default()
    };
    for (t, ((key, pack), damage)) in towns.iter().zip(&data.moat_damage).enumerate() {
        let text = if *pack == "ab" {
            &mut out.ab
        } else {
            &mut out.base
        };
        let _ = write!(
            text,
            "\n[[siege]]\nfaction = \"{key}\"\nmoat_damage = {damage}\n"
        );
        if t == usize::from(data.second_moat_town) {
            text.push_str("second_moat = true\n");
        }
        if t == usize::from(data.mines_town) {
            let spell = reference(mines, mines_pack, pack, "siege mines")?;
            let _ = writeln!(text, "mines = \"{spell}\"");
        }
        let _ = writeln!(text, "hit_points = [{}]", list(&shots.hit_points[t]));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_formats::exe::WallTarget;

    fn data() -> SiegeData {
        SiegeData {
            walls: vec![12, 29],
            moat: vec![11, 28],
            second_moat: vec![10, 27],
            moat_damage: vec![70, 150, 90],
            gate: 29,
            moat_defense_penalty: 3,
            mine_power: 3,
            second_moat_town: 2,
            mines_town: 1,
            mines_spell: 1,
        }
    }

    fn shots() -> WallShots {
        WallShots {
            field: "catapult = 119".into(),
            hit_points: vec![vec![2, 2], vec![3, 2], vec![2, 1]],
        }
    }

    fn towns() -> Vec<(String, &'static str)> {
        vec![
            ("alpha".into(), "base"),
            ("beta".into(), "base"),
            ("gamma".into(), "ab"),
        ]
    }

    fn spells() -> Vec<(String, &'static str)> {
        vec![("bolt".into(), "base"), ("mine".into(), "base")]
    }

    #[test]
    fn writes_the_field_and_a_record_per_town_in_its_pack() {
        let t = write((&data(), &shots()), &towns(), &spells()).unwrap();
        assert_eq!(
            t.field,
            "siege = { walls = [12, 29], gate = 29, moat = [11, 28], second_moat = [10, 27], \
             moat_defense_penalty = 3, mine_power = 3, catapult = 119 }\n"
        );
        assert_eq!(
            t.base,
            "\n[[siege]]\nfaction = \"alpha\"\nmoat_damage = 70\nhit_points = [2, 2]\n\n\
             [[siege]]\nfaction = \"beta\"\nmoat_damage = 150\nmines = \"mine\"\n\
             hit_points = [3, 2]\n"
        );
        assert_eq!(
            t.ab,
            "\n[[siege]]\nfaction = \"gamma\"\nmoat_damage = 90\nsecond_moat = true\n\
             hit_points = [2, 1]\n"
        );
        // An `ab` town's mines refer to the `base` spell by its pack.
        let d = SiegeData {
            mines_town: 2,
            ..data()
        };
        assert!(
            write((&d, &shots()), &towns(), &spells())
                .unwrap()
                .ab
                .contains("mines = \"base:mine\"")
        );
    }

    #[test]
    fn refuses_tables_that_do_not_fit() {
        let err = |d: SiegeData, spells: &[(String, &'static str)]| {
            write((&d, &shots()), &towns(), spells).unwrap_err()
        };
        let mut d = data();
        d.moat_damage.pop();
        assert_eq!(err(d, &spells()), "Heroes3.exe: 2 moat damages for 3 towns");
        let d = SiegeData {
            mines_town: 3,
            ..data()
        };
        assert_eq!(
            err(d, &spells()),
            "Heroes3.exe: the mines' town type 3 is no town"
        );
        assert_eq!(
            err(data(), &spells()[..1]),
            "Heroes3.exe: the mines' spell 1 is no spell"
        );
        // A spell of a later pack than its town.
        assert!(err(data(), &[("bolt".into(), "base"), ("mine".into(), "sod")]).contains("sod"));
        let mut few = shots();
        few.hit_points.pop();
        assert_eq!(
            write((&data(), &few), &towns(), &spells()).unwrap_err(),
            "WALLS.TXT: 2 towns' hit points for 3 towns"
        );
    }

    fn table(text: &str) -> TextTable {
        TextTable::parse(text.as_bytes()).unwrap()
    }

    /// `WALLS.TXT` of `towns` towns, each part's hit points its index + 1 plus the town's.
    fn walls(towns: u8) -> TextTable {
        let mut text = String::from("\t\nWall Section\tHit Points\n");
        for t in 0..towns {
            text.push_str(&format!("Town {t}\t\t\n"));
            for p in 0..18u8 {
                text.push_str(&format!("Part {p}\t{}\t\n", p + 1 + t));
            }
            text.push('\n');
        }
        table(&text)
    }

    fn ballistics(rows: &[&str]) -> TextTable {
        let mut text = String::from("Ballistics\t\tChance\nx\t\tKeep\n");
        for r in rows {
            text.push_str(r);
            text.push('\n');
        }
        table(&text)
    }

    const LEVELS: [&str; 4] = [
        "Skill Level\tNone\t5%\t10%\t25%\t50%\t1\t10%\t60%\t30%\t100%",
        " \tBasic\t7%\t15%\t30%\t60%\t1\t0%\t50%\t50%\t100%",
        " \tAdvanced\t7%\t15%\t30%\t60%\t2\t0%\t50%\t50%\t100%",
        " \tExpert\t10%\t20%\t40%\t75%\t2\t0%\t0%\t100%\t100%",
    ];

    fn exe() -> WallShotData {
        WallShotData {
            targets: vec![
                WallTarget {
                    aim: 255,
                    wall: None,
                    part: 5,
                    kind: 1,
                    turret: Some(255),
                },
                WallTarget {
                    aim: 29,
                    wall: Some(1),
                    part: 6,
                    kind: 3,
                    turret: None,
                },
                WallTarget {
                    aim: 96,
                    wall: Some(0),
                    part: 9,
                    kind: 2,
                    turret: None,
                },
            ],
            missed: vec![1],
            auto_walls: vec![1],
            auto_others: vec![2, 0],
            catapult: 119,
            turret_count: 4,
        }
    }

    #[test]
    fn reads_the_wall_shots() {
        let w = wall_shots(&exe(), &[96, 29], (&walls(2), &ballistics(&LEVELS)), 2).unwrap();
        assert_eq!(
            w.field,
            "catapult = 119, turret_count = 4, targets = [{ kind = \"tower\", turret = 255 }, \
             { kind = \"wall\", wall = 29 }, { kind = \"gate\", wall = 96 }], missed = [1], automatic = [1], fallback = [2, 0], \
             castle_bonus = 1, ballistics = [{ hit = [5, 10, 25, 50], shots = 1, damage = [10, \
             60, 30] }, { hit = [7, 15, 30, 60], shots = 1, damage = [0, 50, 50] }, { hit = [7, \
             15, 30, 60], shots = 2, damage = [0, 50, 50] }, { hit = [10, 20, 40, 75], shots = 2, \
             damage = [0, 0, 100] }]"
        );
        assert_eq!(w.hit_points, [[6, 7, 10], [7, 8, 11]]);
    }

    #[test]
    fn refuses_wall_shot_tables_that_do_not_fit() {
        let levels = ballistics(&LEVELS);
        let err = |e: &WallShotData, w: &TextTable, b: &TextTable| {
            wall_shots(e, &[96, 29], (w, b), 2).unwrap_err()
        };
        let mut far = exe();
        far.targets[1].wall = Some(2);
        assert!(err(&far, &walls(2), &levels).contains("wall index 2"));
        let mut both = exe();
        both.targets[1].turret = Some(254);
        assert!(err(&both, &walls(2), &levels).contains("a wall and a turret"));
        assert!(err(&exe(), &walls(3), &levels).starts_with("WALLS.TXT: 3 towns"));
        assert!(err(&exe(), &walls(2), &ballistics(&LEVELS[..3])).contains("3 levels"));
        let short = ballistics(&[LEVELS[0], LEVELS[1], LEVELS[2], " \tExpert\t10%"]);
        assert!(err(&exe(), &walls(2), &short).contains("without 8 values"));
        let orphan = table("\t\nx\n\t7\t\n");
        assert!(err(&exe(), &orphan, &levels).contains("before the first town"));
    }
}
