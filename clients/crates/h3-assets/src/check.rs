//! One archive entry decoded by its extension, with what the real-asset gate reports about it
//! (E17 M1 T2, T3): the decoders' verdict and the oddities that do not stop decoding.

use alloc::vec::Vec;

use crate::Error;
use crate::archive::extension;
use crate::def::{self, Def};
use crate::msk;
use crate::palette;
use crate::pcx::{self, Pcx};

/// What an entry decoded to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoded {
    /// A PCX image.
    Pcx {
        /// Width.
        width: u32,
        /// Height.
        height: u32,
        /// 24-bit (else 8-bit).
        rgb: bool,
    },
    /// A palette file and its number of extra chunks.
    Pal {
        /// Chunks besides `data`.
        chunks: usize,
    },
    /// A DEF and its frames.
    Def {
        /// The DEF's type.
        kind: u32,
        /// Its groups.
        groups: usize,
        /// Its distinct frames.
        frames: usize,
        /// How many frames use each encoding (by number).
        encodings: [u32; 4],
    },
    /// An MSK.
    Msk {
        /// Width in tiles.
        width: u8,
        /// Height in tiles.
        height: u8,
    },
    /// An extension the client does not decode (yet).
    Skipped,
}

/// Something the gate reports about an entry that decoded (or about one of its frames).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finding {
    /// The archive's type tag disagrees with the content: a PCX's is 16 for 8-bit and 17 for
    /// 24-bit, a DEF's its type.
    TypeTag {
        /// The archive's tag.
        tag: u32,
        /// What the content implies.
        content: u32,
    },
    /// A frame's canvas is not the DEF's.
    Canvas {
        /// The frame's offset.
        offset: u32,
    },
    /// A frame reaches outside its canvas.
    OutsideCanvas {
        /// The frame's offset.
        offset: u32,
    },
    /// A frame was read with the short header (`def` module documentation).
    ShortHeader {
        /// The frame's offset.
        offset: u32,
    },
    /// A frame does not decode.
    Frame {
        /// The frame's offset.
        offset: u32,
        /// Why.
        error: Error,
    },
}

/// The gate's verdict on one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryCheck {
    /// What it decoded to, or the first error (a DEF fails if any of its frames does).
    pub decoded: Result<Decoded, Error>,
    /// The findings, in the order found.
    pub findings: Vec<Finding>,
}

/// Decodes an entry by its name's extension (PCX, PAL, DEF, MSK; any other is
/// [`Decoded::Skipped`]) and collects the findings. `tag` is the archive's type tag.
#[must_use]
pub fn check_entry(name: &str, tag: u32, bytes: &[u8]) -> EntryCheck {
    let mut findings = Vec::new();
    let decoded = match extension(name).as_str() {
        "PCX" => pcx::decode_pcx(bytes).map(|image| {
            let rgb = matches!(image, Pcx::Rgb(_));
            type_tag(&mut findings, tag, if rgb { 17 } else { 16 });
            Decoded::Pcx {
                width: image.width(),
                height: image.height(),
                rgb,
            }
        }),
        "PAL" => palette::decode_pal(bytes).map(|pal| Decoded::Pal {
            chunks: pal.chunks.len(),
        }),
        "DEF" => def::parse_def(bytes).and_then(|d| {
            type_tag(&mut findings, tag, d.kind);
            check_frames(&d, bytes, &mut findings)
        }),
        "MSK" => msk::decode_msk(bytes).map(|m| Decoded::Msk {
            width: m.width,
            height: m.height,
        }),
        _ => Ok(Decoded::Skipped),
    };
    EntryCheck { decoded, findings }
}

fn type_tag(findings: &mut Vec<Finding>, tag: u32, content: u32) {
    if tag != content {
        findings.push(Finding::TypeTag { tag, content });
    }
}

/// Decodes every distinct frame of `d`.
fn check_frames(d: &Def, bytes: &[u8], findings: &mut Vec<Finding>) -> Result<Decoded, Error> {
    let offsets = d.offsets();
    let mut encodings = [0u32; 4];
    let mut first_error = None;
    for &offset in &offsets {
        match def::decode_frame(bytes, offset) {
            Ok(frame) => {
                for (count, number) in encodings.iter_mut().zip(0..) {
                    *count += u32::from(number == frame.encoding.number());
                }
                if (frame.full_width, frame.full_height) != (d.width, d.height) {
                    findings.push(Finding::Canvas { offset });
                }
                if !inside(frame.left, frame.width, frame.full_width)
                    || !inside(frame.top, frame.height, frame.full_height)
                {
                    findings.push(Finding::OutsideCanvas { offset });
                }
                if frame.short_header {
                    findings.push(Finding::ShortHeader { offset });
                }
            }
            Err(error) => {
                findings.push(Finding::Frame { offset, error });
                first_error = first_error.or(Some(error));
            }
        }
    }
    match first_error {
        Some(error) => Err(error),
        None => Ok(Decoded::Def {
            kind: d.kind,
            groups: d.groups.len(),
            frames: offsets.len(),
            encodings,
        }),
    }
}

/// Whether `[start, start + len)` lies inside `[0, full)`.
fn inside(start: i32, len: u32, full: u32) -> bool {
    start >= 0 && i64::from(start) + i64::from(len) <= i64::from(full)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use crate::def::{Encoding, Frame};
    use crate::msk::Mask;
    use crate::palette::{Palette, Rgb};
    use alloc::vec;

    fn frame(left: i32, width: u32, full_width: u32) -> Frame {
        Frame {
            encoding: Encoding::Raw,
            full_width,
            full_height: 1,
            left,
            top: 0,
            width,
            height: 1,
            pixels: vec![1; usize::try_from(width).unwrap()],
            runs: vec![false; usize::try_from(width).unwrap()],
            short_header: false,
        }
    }

    #[test]
    fn pcx_images_and_their_tags() {
        let indexed = build::pcx_indexed(2, 1, &[1, 2], &Palette::default());
        let c = check_entry("a.pcx", 16, &indexed);
        assert_eq!(
            c.decoded,
            Ok(Decoded::Pcx {
                width: 2,
                height: 1,
                rgb: false
            })
        );
        assert!(c.findings.is_empty());
        let rgb = build::pcx_rgb(1, 1, &[Rgb::default()]);
        let c = check_entry("B.PCX", 16, &rgb);
        assert_eq!(
            c.findings,
            [Finding::TypeTag {
                tag: 16,
                content: 17
            }]
        );
        assert_eq!(
            check_entry("c.pcx", 16, &[1]).decoded,
            Err(Error::Truncated)
        );
    }

    #[test]
    fn palettes_masks_and_others() {
        let pal = build::pal(&Palette::default(), &[0; 256], 0, &[]);
        assert_eq!(
            check_entry("x.pal", 96, &pal).decoded,
            Ok(Decoded::Pal { chunks: 0 })
        );
        let mask = Mask {
            width: 2,
            height: 1,
            masks: [[0; 6]; 2],
        };
        assert_eq!(
            check_entry("x.msk", 79, &build::msk(&mask)).decoded,
            Ok(Decoded::Msk {
                width: 2,
                height: 1
            })
        );
        assert_eq!(check_entry("x.txt", 2, &[]).decoded, Ok(Decoded::Skipped));
    }

    #[test]
    fn a_def_is_decoded_frame_by_frame() {
        let mut short = frame(0, 4, 4);
        (short.encoding, short.height, short.full_height) = (Encoding::Rows, 4, 4);
        (short.pixels, short.short_header) = (vec![3; 16], true);
        let frames = [
            frame(0, 2, 2),
            frame(1, 2, 2),
            frame(-1, 1, 2),
            frame(0, 2, 3),
            short,
        ];
        let refs: [(&str, usize); 5] = [("a", 0), ("b", 1), ("c", 2), ("d", 3), ("e", 4)];
        let bytes = build::def(0x42, (2, 1), &Palette::default(), &[(0, &refs)], &frames);
        let c = check_entry("x.def", 0x43, &bytes);
        assert_eq!(
            c.decoded,
            Ok(Decoded::Def {
                kind: 0x42,
                groups: 1,
                frames: 5,
                encodings: [4, 1, 0, 0]
            })
        );
        let offsets: Vec<u32> = parse(&bytes).offsets().into_iter().collect();
        assert_eq!(
            c.findings,
            [
                Finding::TypeTag {
                    tag: 0x43,
                    content: 0x42
                },
                Finding::OutsideCanvas { offset: offsets[1] },
                Finding::OutsideCanvas { offset: offsets[2] },
                Finding::Canvas { offset: offsets[3] },
                Finding::Canvas { offset: offsets[4] },
                Finding::ShortHeader { offset: offsets[4] },
            ]
        );
    }

    fn parse(bytes: &[u8]) -> Def {
        def::parse_def(bytes).unwrap()
    }

    #[test]
    fn a_def_with_a_broken_frame_fails() {
        let frames = [frame(0, 2, 2), frame(0, 1, 2)];
        let refs: [(&str, usize); 3] = [("a", 0), ("b", 1), ("gone", 9)];
        let bytes = build::def(0x40, (2, 1), &Palette::default(), &[(0, &refs)], &frames);
        let c = check_entry("x.def", 0x40, &bytes);
        assert_eq!(c.decoded, Err(Error::Truncated));
        assert_eq!(
            c.findings,
            [Finding::Frame {
                offset: u32::MAX,
                error: Error::Truncated
            }]
        );
        assert_eq!(check_entry("y.def", 0, &[]).decoded, Err(Error::Truncated));
    }

    #[test]
    fn inside_checks_both_ends() {
        assert!(inside(0, 2, 2));
        assert!(!inside(-1, 1, 2));
        assert!(!inside(1, 2, 2));
        assert!(inside(i32::MAX, 0, u32::MAX));
    }
}
