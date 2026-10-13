//! Sieges (E04c T5): the siege's field (`siege` in `[battle]`) and one `[[siege]]` record per
//! town type, from the executable's tables and instructions (`docs/formats/installation.md`
//! §7.27, §7.28). A record goes to the pack that defines its town.

use std::fmt::Write as _;

use h3_formats::exe::SiegeData;

use crate::heroes::reference;

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
    data: &SiegeData,
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
    for (what, town) in [
        ("second moat", data.second_moat_town),
        ("mines", data.mines_town),
    ] {
        if usize::from(town) >= towns.len() {
            return Err(format!("Heroes3.exe: the {what}' town type {town} is no town"));
        }
    }
    let (mines, mines_pack) = spells
        .get(usize::from(data.mines_spell))
        .ok_or_else(|| format!("Heroes3.exe: the mines' spell {} is no spell", data.mines_spell))?;
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
             moat_defense_penalty = {}, mine_power = {} }}\n",
            list(&data.walls),
            data.gate,
            list(&data.moat),
            list(&data.second_moat),
            data.moat_defense_penalty,
            data.mine_power
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
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let t = write(&data(), &towns(), &spells()).unwrap();
        assert_eq!(
            t.field,
            "siege = { walls = [12, 29], gate = 29, moat = [11, 28], second_moat = [10, 27], \
             moat_defense_penalty = 3, mine_power = 3 }\n"
        );
        assert_eq!(
            t.base,
            "\n[[siege]]\nfaction = \"alpha\"\nmoat_damage = 70\n\n[[siege]]\nfaction = \"beta\"\n\
             moat_damage = 150\nmines = \"mine\"\n"
        );
        assert_eq!(
            t.ab,
            "\n[[siege]]\nfaction = \"gamma\"\nmoat_damage = 90\nsecond_moat = true\n"
        );
        // An `ab` town's mines refer to the `base` spell by its pack.
        let d = SiegeData {
            mines_town: 2,
            ..data()
        };
        assert!(write(&d, &towns(), &spells()).unwrap().ab.contains("mines = \"base:mine\""));
    }

    #[test]
    fn refuses_tables_that_do_not_fit() {
        let err = |d: SiegeData, spells: &[(String, &'static str)]| {
            write(&d, &towns(), spells).unwrap_err()
        };
        let mut d = data();
        d.moat_damage.pop();
        assert_eq!(
            err(d, &spells()),
            "Heroes3.exe: 2 moat damages for 3 towns"
        );
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
    }
}
