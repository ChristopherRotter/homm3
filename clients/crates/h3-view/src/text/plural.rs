//! Plural forms (E17 M3 T4b-1, `docs/epics/E17-design.md` §9.3): which of a message's forms a
//! count takes. A language pack's forms are CLDR's plural categories; the original's own text has
//! two rows (one, other) and the executable picks one for a count of exactly 1.
//!
//! The rules are CLDR's cardinal rules (`common/supplemental/plurals.xml`, the `main` branch read
//! on 2026-10-19) for integers only (the operands v = 0, e = 0, i = n): the battle's counts are
//! whole numbers. A language not listed has no rule, and a pack in it is refused.

/// A CLDR plural category, as a pack's suffix names it (`_one`, `_few`, …): those of the
/// languages here (CLDR's `zero` and `two` belong to languages without a rule here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    /// `_one`.
    One,
    /// `_few`.
    Few,
    /// `_many`.
    Many,
    /// `_other`: every language has it, and a lookup falls back to it.
    Other,
}

impl Category {
    /// Every category, in CLDR's order.
    pub const ALL: [Self; 4] = [Self::One, Self::Few, Self::Many, Self::Other];

    /// The suffix's word (`one`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    /// The category a suffix's word names.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// A language's rule for whole numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// The executable's: `one` for exactly 1, else `other` (the original's own text).
    Original,
    /// `one` for 1, else `other` (en, de, nl, sv, da, nb, hu, …).
    OneOther,
    /// `one` for 1, `many` for a non-zero multiple of a million, else `other` (it, es, ca).
    OneManyOther,
    /// French and Portuguese: `one` for 0 and 1, `many` for a non-zero multiple of a million,
    /// else `other`.
    French,
    /// Polish: `one` for 1, `few` for 2–4 in the last digit outside 12–14, else `many`.
    Polish,
    /// Russian and Ukrainian: `one` for 1 in the last digit outside 11, `few` for 2–4 in the
    /// last digit outside 12–14, else `many`.
    EastSlavic,
    /// Czech and Slovak: `one` for 1, `few` for 2–4, else `other`.
    CzechSlovak,
    /// Only `other` (zh, ja, ko).
    OtherOnly,
}

impl Rule {
    /// The rule of a language by its primary subtag (`de`, `pt`; a region such as `pt-BR` is
    /// cut off, but Portugal's Portuguese has a rule of its own, CLDR's `pt_PT`: `one` for 1
    /// only, as Italian's), or `None` for a language without one here.
    #[must_use]
    pub fn of(language: &str) -> Option<Self> {
        let mut tags = language.split(['-', '_']);
        let primary = tags.next().unwrap_or_default();
        if primary == "pt" && tags.next().is_some_and(|r| r.eq_ignore_ascii_case("PT")) {
            return Some(Self::OneManyOther);
        }
        Some(match primary {
            "en" | "de" | "nl" | "sv" | "da" | "nb" | "nn" | "no" | "fi" | "et" | "hu" | "tr"
            | "el" | "bg" => Self::OneOther,
            "it" | "es" | "ca" => Self::OneManyOther,
            "fr" | "pt" => Self::French,
            "pl" => Self::Polish,
            "ru" | "uk" => Self::EastSlavic,
            "cs" | "sk" => Self::CzechSlovak,
            "zh" | "ja" | "ko" => Self::OtherOnly,
            _ => return None,
        })
    }

    /// The category of a count (its absolute value, as CLDR's operand n).
    #[must_use]
    pub fn category(self, count: i64) -> Category {
        let n = count.unsigned_abs();
        let (d, h) = (n % 10, n % 100);
        let million = n != 0 && n.is_multiple_of(1_000_000);
        match self {
            Self::Original | Self::OneOther => one_if(n == 1, Category::Other),
            Self::OneManyOther => one_if(n == 1, many_if(million)),
            Self::French => one_if(n <= 1, many_if(million)),
            Self::Polish => one_if(
                n == 1,
                few_if((2..=4).contains(&d) && !(12..=14).contains(&h)),
            ),
            Self::EastSlavic => one_if(
                d == 1 && h != 11,
                few_if((2..=4).contains(&d) && !(12..=14).contains(&h)),
            ),
            Self::CzechSlovak => match n {
                1 => Category::One,
                2..=4 => Category::Few,
                _ => Category::Other,
            },
            Self::OtherOnly => Category::Other,
        }
    }
}

const fn one_if(one: bool, otherwise: Category) -> Category {
    if one { Category::One } else { otherwise }
}

const fn many_if(many: bool) -> Category {
    if many {
        Category::Many
    } else {
        Category::Other
    }
}

/// Polish and East Slavic: `few`, else `many` (every other whole number).
const fn few_if(few: bool) -> Category {
    if few { Category::Few } else { Category::Many }
}
