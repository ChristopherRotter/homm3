use super::*;
use crate::build;
use crate::palette::Rgb;
use alloc::vec;
use proptest::prelude::*;

fn frame(encoding: Encoding, width: u32, height: u32, pixels: Vec<u8>) -> Frame {
    Frame {
        encoding,
        full_width: 64,
        full_height: 40,
        left: 3,
        top: -2,
        width,
        height,
        pixels,
        short_header: false,
    }
}

/// A row with every kind of run: fills, literals, the literal escape's own index.
fn sample(width: usize, height: usize) -> Vec<u8> {
    (0..width * height)
        .map(|i| match (i / 5) % 4 {
            0 => 0,
            1 => u8::try_from(i % 256).unwrap(),
            2 => 0xFF,
            _ => 5,
        })
        .collect()
}

fn palette() -> Palette {
    let mut p = Palette::default();
    p.0[9] = Rgb { r: 9, g: 8, b: 7 };
    p
}

#[test]
fn the_group_table_is_parsed() {
    let frames = [
        frame(Encoding::Raw, 2, 1, vec![1, 2]),
        frame(Encoding::Raw, 1, 1, vec![3]),
    ];
    let groups: [(u32, &[(&str, usize)]); 2] =
        [(0, &[("one", 0), ("two", 1)]), (7, &[("again", 0)])];
    let bytes = build::def(0x42, (64, 40), &palette(), &groups, &frames);
    let def = parse_def(&bytes).unwrap();
    assert_eq!((def.kind, def.width, def.height), (0x42, 64, 40));
    assert_eq!(def.palette, palette());
    assert_eq!(def.groups.len(), 2);
    assert_eq!(def.groups[1].id, 7);
    let names: Vec<&str> = def.groups[0]
        .frames
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(names, ["one", "two"]);
    assert_eq!(
        def.groups[1].frames[0].offset,
        def.groups[0].frames[0].offset
    );
    let offsets: Vec<u32> = def.offsets().into_iter().collect();
    assert_eq!(offsets.len(), 2);
    for (offset, expected) in offsets.iter().zip(&frames) {
        assert_eq!(&decode_frame(&bytes, *offset).unwrap(), expected);
    }
}

#[test]
fn names_end_at_the_first_nul_or_after_13_bytes() {
    let bytes = build::def(0x40, (1, 1), &palette(), &[(0, &[("x", 0)])], &[]);
    let names_at = 16 + PALETTE_BYTES + 16;
    let mut garbage = bytes.clone();
    garbage[names_at..names_at + NAME_BYTES].copy_from_slice(b"ab\0cdefghijkl");
    assert_eq!(parse_def(&garbage).unwrap().groups[0].frames[0].name, "ab");
    garbage[names_at..names_at + NAME_BYTES].copy_from_slice(b"ABCDEFGHIJKL\xE9");
    assert_eq!(
        parse_def(&garbage).unwrap().groups[0].frames[0].name,
        "ABCDEFGHIJKL\u{e9}"
    );
}

#[test]
fn a_cut_group_table_is_truncated() {
    let bytes = build::def(0x40, (1, 1), &palette(), &[(0, &[("x", 0), ("y", 0)])], &[]);
    let table = 16 + PALETTE_BYTES;
    // The header, the palette, a group's header, its names, its offsets.
    for cut in [15, table - 1, table + 15, table + 16 + 25, bytes.len() - 1] {
        assert_eq!(
            parse_def(&bytes[..cut]),
            Err(Error::Truncated),
            "cut at {cut}"
        );
    }
    assert!(parse_def(&bytes).is_ok());
}

#[test]
fn every_encoding_round_trips() {
    for (encoding, width) in [
        (Encoding::Raw, 7),
        (Encoding::Rows, 300),
        (Encoding::PackedRows, 45),
        (Encoding::PackedBlocks, 64),
    ] {
        let f = frame(encoding, width, 3, sample(index(width), 3));
        let bytes = build::frame(&f);
        assert_eq!(decode_frame(&bytes, 0), Ok(f), "{encoding:?}");
    }
}

#[test]
fn long_runs_are_split_at_the_encodings_limits() {
    // Encoding 1: a run of 256 and one of 300 (split); encodings 2 and 3: runs of 32 at most.
    let mut pixels = vec![9; 256];
    pixels.extend(vec![0xFF; 300]);
    let f = frame(Encoding::Rows, 556, 1, pixels);
    assert_eq!(decode_frame(&build::frame(&f), 0), Ok(f));
    let f = frame(Encoding::PackedRows, 70, 1, vec![6; 70]);
    assert_eq!(decode_frame(&build::frame(&f), 0), Ok(f));
}

#[test]
fn the_short_header_is_recognised() {
    // A 4 × 4 frame of encoding 1 behind a 16-byte header: its row offsets (16, 18, …), read
    // as width and height, exceed the canvas.
    let mut f = frame(Encoding::Rows, 4, 4, vec![2; 16]);
    (f.full_width, f.full_height, f.left, f.top, f.short_header) = (4, 4, 0, 0, true);
    let bytes = build::frame(&f);
    assert_eq!(u32_at(&bytes, 16), Some(16));
    assert_eq!(decode_frame(&bytes, 0), Ok(f.clone()));
    // The same frame behind a full header is not short.
    f.short_header = false;
    assert_eq!(decode_frame(&build::frame(&f), 0), Ok(f.clone()));
    // Only encoding 1 has a short header.
    let mut raw = build::frame(&frame(Encoding::Raw, 1, 1, vec![0]));
    raw[16..24].copy_from_slice(&[9, 0, 0, 0, 9, 0, 0, 0]);
    raw[8..16].copy_from_slice(&[1, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(decode_frame(&raw, 0), Err(Error::Truncated));
}

#[test]
fn frame_headers_are_checked() {
    let good = build::frame(&frame(Encoding::Raw, 2, 2, vec![1; 4]));
    assert_eq!(decode_frame(&good, 5), Err(Error::Truncated));
    assert_eq!(decode_frame(&good, u32::MAX), Err(Error::Truncated));
    let mut bad = good.clone();
    bad[4] = 4;
    assert_eq!(decode_frame(&bad, 0), Err(Error::Field("encoding")));
    let mut bad = good.clone();
    bad[16..20].copy_from_slice(&(MAX_SIDE + 1).to_le_bytes());
    assert_eq!(decode_frame(&bad, 0), Err(Error::TooLarge));
    let mut bad = good.clone();
    bad[20..24].copy_from_slice(&(MAX_SIDE + 1).to_le_bytes());
    assert_eq!(decode_frame(&bad, 0), Err(Error::TooLarge));
    // `size` beyond the file; `size` shorter than the rows.
    assert_eq!(
        decode_frame(&good[..good.len() - 1], 0),
        Err(Error::Truncated)
    );
    let mut bad = good;
    bad[0] = 3;
    assert_eq!(decode_frame(&bad, 0), Err(Error::Truncated));
}

/// A frame of `encoding`, `width` × 1, with `data` after its header.
fn handmade(encoding: Encoding, width: u32, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let size = u32::try_from(data.len()).unwrap();
    for v in [size, encoding.number(), width, 1, width, 1, 0, 0] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(data);
    out
}

#[test]
fn encoding_1_is_bounded() {
    let cases: [(&[u8], Error); 5] = [
        (&[4, 0, 0], Error::Truncated),                   // the row table
        (&[4, 0, 0, 0, 5], Error::Truncated),             // a run's header
        (&[4, 0, 0, 0, 0xFF, 2, 1, 2], Error::Truncated), // a literal run
        (&[4, 0, 0, 0, 5, 4], Error::Overrun),            // 5 pixels in a row of 4
        (&[4, 0, 0, 0, 0xFF, 1, 1, 2, 5, 2], Error::Overrun),
    ];
    for (data, error) in cases {
        assert_eq!(
            decode_frame(&handmade(Encoding::Rows, 4, data), 0),
            Err(error),
            "{data:?}"
        );
    }
    let ok = handmade(Encoding::Rows, 4, &[4, 0, 0, 0, 0xFF, 1, 1, 2, 5, 1, 0xEE]);
    assert_eq!(decode_frame(&ok, 0).unwrap().pixels, [1, 2, 5, 5]);
}

#[test]
fn encodings_2_and_3_are_bounded() {
    let cases: [(Encoding, u32, &[u8], Error); 7] = [
        (Encoding::PackedRows, 4, &[2], Error::Truncated),
        (Encoding::PackedRows, 4, &[2, 0], Error::Truncated),
        (Encoding::PackedRows, 4, &[2, 0, 0xE2, 1], Error::Truncated),
        (Encoding::PackedRows, 4, &[2, 0, 0x24], Error::Overrun),
        (Encoding::PackedBlocks, 33, &[], Error::Field("width")),
        (Encoding::PackedBlocks, 32, &[2], Error::Truncated),
        (
            Encoding::PackedBlocks,
            32,
            &[2, 0, 0x1E, 0x21],
            Error::Overrun,
        ),
    ];
    for (encoding, width, data, error) in cases {
        let bytes = handmade(encoding, width, data);
        assert_eq!(decode_frame(&bytes, 0), Err(error), "{encoding:?} {data:?}");
    }
    let ok = handmade(Encoding::PackedRows, 4, &[2, 0, 0xE1, 9, 8, 0x41]);
    assert_eq!(decode_frame(&ok, 0).unwrap().pixels, [9, 8, 2, 2]);
}

fn any_encoding() -> impl Strategy<Value = Encoding> {
    prop_oneof![
        Just(Encoding::Raw),
        Just(Encoding::Rows),
        Just(Encoding::PackedRows),
        Just(Encoding::PackedBlocks),
    ]
}

proptest! {
    #[test]
    fn frames_round_trip(
        encoding in any_encoding(),
        width in 0u32..70,
        height in 0u32..6,
        seed: u64,
        left: i32,
        top: i32,
    ) {
        let width = if encoding == Encoding::PackedBlocks { width / 32 * 32 } else { width };
        let n = index(width * height);
        // Runs of small indices (fills) mixed with any index (literals).
        let pixels: Vec<u8> = (0..n)
            .map(|i| {
                let x = seed.rotate_left(u32::try_from(i % 64).unwrap());
                if x & 3 == 0 { u8::try_from(x >> 56).unwrap() } else { u8::try_from(x & 1).unwrap() }
            })
            .collect();
        let f = Frame { left, top, ..frame(encoding, width, height, pixels) };
        prop_assert_eq!(decode_frame(&build::frame(&f), 0), Ok(f));
    }

    #[test]
    fn parsing_and_decoding_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..1200), offset: u16) {
        let _ = parse_def(&bytes);
        let _ = decode_frame(&bytes, u32::from(offset) % 64);
    }

    #[test]
    fn a_mutated_def_never_panics(at in 0usize..1000, value: u8, encoding in any_encoding()) {
        let f = frame(encoding, 32, 3, sample(32, 3));
        let mut bytes = build::def(0x42, (64, 40), &palette(), &[(0, &[("a", 0)])], &[f]);
        if let Some(b) = bytes.get_mut(at) {
            *b = value;
        }
        if let Ok(def) = parse_def(&bytes) {
            for offset in def.offsets() {
                let _ = decode_frame(&bytes, offset);
            }
        }
    }
}
