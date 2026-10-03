//! Property tests: no reader panics or over-allocates on arbitrary input (E02 exit criterion 4).

use h3_formats::{exe, lod, text};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn lod_parse_and_read_never_panic(mut bytes in proptest::collection::vec(any::<u8>(), 0..2048), magic in any::<bool>()) {
        if magic && bytes.len() >= 4 {
            bytes[..4].copy_from_slice(b"LOD\0");
        }
        if let Ok(archive) = lod::LodArchive::parse(&bytes) {
            for entry in archive.entries() {
                let _ = archive.read(entry);
            }
        }
    }

    #[test]
    fn lod_mutations_of_a_valid_archive_never_panic(flips in proptest::collection::vec((any::<usize>(), any::<u8>()), 1..8)) {
        let data = b"Name\tValue\r\n\"multi\nline\"\t1\r\n".repeat(8);
        let mut bytes = lod::build(500, &[("a.txt", &data, true), ("b.txt", b"stored", false)]);
        for (at, value) in flips {
            let len = bytes.len();
            bytes[at % len] = value;
        }
        if let Ok(archive) = lod::LodArchive::parse(&bytes) {
            for entry in archive.entries() {
                if let Ok(out) = archive.read(entry) {
                    prop_assert_eq!(out.len(), entry.size as usize);
                }
            }
        }
    }

    #[test]
    fn text_parse_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let _ = text::TextTable::parse(&bytes);
    }

    #[test]
    fn text_fields_round_trip(rows in proptest::collection::vec(proptest::collection::vec("[a-z0-9 ]{0,6}", 1..5), 0..6)) {
        let mut src = String::new();
        for row in &rows {
            src.push_str(&row.join("\t"));
            src.push_str("\r\n");
        }
        let parsed = text::TextTable::parse(src.as_bytes()).unwrap();
        prop_assert_eq!(parsed.records, rows);
    }

    #[test]
    fn exe_parse_never_panics(mut bytes in proptest::collection::vec(any::<u8>(), 0..1024), va in any::<u32>(), count in 0usize..300) {
        if bytes.len() >= 2 {
            bytes[..2].copy_from_slice(b"MZ");
        }
        if let Ok(image) = exe::PeImage::parse(&bytes) {
            let _ = image.c_string(va);
            let _ = exe::creature_table_at(&image, va, count);
            let tables = exe::HeroTables {
                hero_va: va,
                hero_count: count,
                specialty_va: va.wrapping_add(8),
                spell_va: va.wrapping_add(16),
                spell_count: count,
                skill_values: &[exe::SkillValueTable { skill: 0, va: 0x0040_1000, float: true }],
                specialty_growth_va: va,
            };
            let _ = exe::hero_data_at(&image, &tables);
        }
        let _ = exe::creature_table(&bytes);
        let _ = exe::hero_data(&bytes);
    }

    #[test]
    fn exe_mutations_never_panic(flips in proptest::collection::vec((any::<usize>(), any::<u8>()), 1..8), count in 0usize..8) {
        let mut payload = b"name\0".to_vec();
        payload.resize(8 + 116 * 4, 0);
        let mut bytes = exe::build_test_image(7, &payload);
        for (at, value) in flips {
            let len = bytes.len();
            bytes[at % len] = value;
        }
        if let Ok(image) = exe::PeImage::parse(&bytes) {
            let _ = exe::creature_table_at(&image, 0x0040_1008, count);
            let tables = exe::HeroTables {
                hero_va: 0x0040_1008,
                hero_count: count,
                specialty_va: 0x0040_1010,
                spell_va: 0x0040_1020,
                spell_count: count,
                skill_values: &[exe::SkillValueTable { skill: 0, va: 0x0040_1000, float: false }],
                specialty_growth_va: 0x0040_1004,
            };
            let _ = exe::hero_data_at(&image, &tables);
        }
    }
}
