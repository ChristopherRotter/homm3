//! Glicko-2 (E16 M4 T1; ADR-0019 point 6; `E16-design.md` §11): a ladder's ratings, as Mark
//! Glickman's "Example of the Glicko-2 system" (2013) computes them, with its published defaults.
//! A ladder rates each game as one rating period of one game for both players, in the order games
//! end; a draw scores a half.
//!
//! Floats are allowed here: the server is an adapter, not game logic (ADR-0019 point 6). The
//! platform's `exp` and `ln` may differ in the last bit between targets, so a ladder's results log
//! keeps every rating computed and a start reads them back: no rating is computed twice.
//!
//! Provisional (E16 M4 T1): no deviation growth for inactivity between games (the server has no
//! wall clock); the volatility's iteration bounded at [`STEPS`] steps.

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

/// A new account's rating.
pub const RATING: f64 = 1500.0;
/// A new account's rating deviation, and every account's at the start of a season (the user,
/// SRV-Q19).
pub const DEVIATION: f64 = 350.0;
/// A new account's volatility.
pub const VOLATILITY: f64 = 0.06;
/// The system constant τ: how far the volatility may move in a period.
pub const TAU: f64 = 0.5;
/// The volatility iteration's tolerance.
const EPSILON: f64 = 0.000_001;
/// The Glicko-2 scale: 400 / ln 10.
const SCALE: f64 = 173.7178;
/// The most steps of each loop of the volatility's iteration: it converges in a few, so the bound
/// only keeps a broken input from hanging the server.
const STEPS: u32 = 100;

/// A player's rating.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rating {
    /// The rating.
    pub rating: f64,
    /// Its deviation: how uncertain it is.
    pub deviation: f64,
    /// Its volatility: how erratic the player's results are.
    pub volatility: f64,
}

impl Default for Rating {
    fn default() -> Self {
        Self {
            rating: RATING,
            deviation: DEVIATION,
            volatility: VOLATILITY,
        }
    }
}

impl Rating {
    /// The rating a new season starts from: this one with the default deviation.
    #[must_use]
    pub const fn new_season(self) -> Self {
        Self {
            deviation: DEVIATION,
            ..self
        }
    }

    /// Whether Glicko-2 can go on from it: every number finite, the deviation and the volatility
    /// above 0.
    pub fn is_valid(self) -> bool {
        self.rating.is_finite()
            && self.deviation.is_finite()
            && self.deviation > 0.0
            && self.volatility.is_finite()
            && self.volatility > 0.0
    }
}

/// An account's row on a ladder in a season (E16 M4 T1): its rating as it stands and its games.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Standing {
    /// Its rating after its last game, or as the season began.
    pub rating: Rating,
    /// The games rated.
    pub games: u32,
    /// Of them, won.
    pub wins: u32,
    /// Lost.
    pub losses: u32,
    /// Drawn.
    pub draws: u32,
    /// The games void because both players stayed away (E16 M4 T3a).
    pub no_shows: u32,
}

impl Standing {
    /// A row with no games yet, at `rating`.
    pub const fn new(rating: Rating) -> Self {
        Self {
            rating,
            games: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            no_shows: 0,
        }
    }
}

/// A game's result for one of its players.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    /// It won.
    Win,
    /// A draw.
    Draw,
    /// It lost.
    Loss,
}

impl Score {
    /// The score Glicko-2 counts.
    const fn value(self) -> f64 {
        match self {
            Self::Win => 1.0,
            Self::Draw => 0.5,
            Self::Loss => 0.0,
        }
    }

    /// The same game's result for the other player.
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Win => Self::Loss,
            Self::Draw => Self::Draw,
            Self::Loss => Self::Win,
        }
    }
}

/// The two players' ratings after a game between them, `score` the first's: each updated against
/// the other's rating before the game, one rating period of one game.
pub fn game(first: Rating, second: Rating, score: Score) -> (Rating, Rating) {
    (
        period(first, &[(second, score)]),
        period(second, &[(first, score.reversed())]),
    )
}

/// A player's rating after a rating period in which it played `games`: each opponent's rating
/// before the period and the player's score. Without games only the deviation grows.
pub fn period(player: Rating, games: &[(Rating, Score)]) -> Rating {
    let mu = (player.rating - RATING) / SCALE;
    let phi = player.deviation / SCALE;
    if games.is_empty() {
        return Rating {
            deviation: SCALE * norm(phi, player.volatility),
            ..player
        };
    }
    // Steps 3 and 4: the estimated variance and the improvement.
    let (mut variance, mut sum) = (0.0, 0.0);
    for &(opponent, score) in games {
        let mu_j = (opponent.rating - RATING) / SCALE;
        let g = g(opponent.deviation / SCALE);
        let expected = 1.0 / (1.0 + exp(-g * (mu - mu_j)));
        variance += g * g * expected * (1.0 - expected);
        sum += g * (score.value() - expected);
    }
    let v = 1.0 / variance;
    let volatility = volatility(phi, player.volatility, v, v * sum);
    // Steps 6 to 8.
    let phi_star = norm(phi, volatility);
    let phi_new = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / v).sqrt();
    Rating {
        rating: SCALE * (mu + phi_new * phi_new * sum) + RATING,
        deviation: SCALE * phi_new,
        volatility,
    }
}

/// `sqrt(a² + b²)`, without `hypot` (banned in the workspace).
fn norm(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

/// Glickman's g(φ).
fn g(phi: f64) -> f64 {
    1.0 / (1.0 + 3.0 * phi * phi / (PI * PI)).sqrt()
}

/// The new volatility (step 5): the root of Glickman's f by the Illinois algorithm, `delta` the
/// improvement Δ.
fn volatility(phi: f64, sigma: f64, v: f64, delta: f64) -> f64 {
    let a = ln(sigma * sigma);
    let rest = delta * delta - phi * phi - v;
    let f = |x: f64| {
        let ex = exp(x);
        let d = phi * phi + v + ex;
        ex * (rest - ex) / (2.0 * d * d) - (x - a) / (TAU * TAU)
    };
    let mut low = a;
    let mut high = if rest > 0.0 {
        ln(rest)
    } else {
        let mut k = 1;
        while f(a - f64::from(k) * TAU) < 0.0 && k < STEPS {
            k += 1;
        }
        a - f64::from(k) * TAU
    };
    let (mut f_low, mut f_high) = (f(low), f(high));
    let mut steps = 0;
    while (high - low).abs() > EPSILON && steps < STEPS {
        let c = low + (low - high) * f_low / (f_high - f_low);
        let f_c = f(c);
        if f_c * f_high <= 0.0 {
            (low, f_low) = (high, f_high);
        } else {
            f_low /= 2.0;
        }
        (high, f_high) = (c, f_c);
        steps += 1;
    }
    exp(low / 2.0)
}

// The workspace bans the platform's transcendentals for game logic (ADR-0003 §3); a rating is not
// game logic, and each one computed is written down and read back, never computed again (above).
#[allow(clippy::disallowed_methods)]
fn exp(x: f64) -> f64 {
    x.exp()
}

#[allow(clippy::disallowed_methods)]
fn ln(x: f64) -> f64 {
    x.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rating(rating: f64, deviation: f64) -> Rating {
        Rating {
            rating,
            deviation,
            volatility: VOLATILITY,
        }
    }

    /// Within `tolerance` of `expected`, each number.
    fn close(actual: Rating, expected: (f64, f64, f64), tolerance: (f64, f64, f64)) {
        let (r, d, v) = expected;
        assert!(
            (actual.rating - r).abs() <= tolerance.0
                && (actual.deviation - d).abs() <= tolerance.1
                && (actual.volatility - v).abs() <= tolerance.2,
            "{actual:?} against {expected:?}"
        );
    }

    /// The numbers of an independent implementation (Python, `E16-log.md` M4 T1), the same steps.
    const EXACT: (f64, f64, f64) = (1e-9, 1e-9, 1e-12);

    #[test]
    fn glickmans_example() {
        // Glickman's worked example: 1500/200/0.06 against 1400/30 (won), 1550/100 and 1700/300
        // (lost), τ = 0.5: the paper's 1464.06, 151.52, 0.05999 (it rounds μ' first).
        let games = [
            (rating(1400.0, 30.0), Score::Win),
            (rating(1550.0, 100.0), Score::Loss),
            (rating(1700.0, 300.0), Score::Loss),
        ];
        let after = period(rating(1500.0, 200.0), &games);
        close(after, (1464.06, 151.52, 0.059_99), (0.01, 0.005, 0.000_01));
        close(
            after,
            (
                1_464.050_670_539_301_3,
                151.516_524_123_857_27,
                0.059_995_984_286_488_495,
            ),
            EXACT,
        );
    }

    #[test]
    fn one_game_each_way() {
        // Two new players: the winner up, the loser down by as much, both less uncertain.
        let (winner, loser) = game(Rating::default(), Rating::default(), Score::Win);
        close(
            winner,
            (
                1_662.310_893_906_297_7,
                290.318_963_717_980_47,
                0.059_999_675_372_338_14,
            ),
            EXACT,
        );
        assert!((winner.rating - RATING - (RATING - loser.rating)).abs() < 1e-9);
        assert!((winner.deviation - loser.deviation).abs() < 1e-9);
        // A draw between unequal players moves them towards each other.
        let (high, low) = game(rating(1600.0, 80.0), rating(1400.0, 120.0), Score::Draw);
        close(
            high,
            (
                1_591.693_231_743_093_7,
                79.272_856_033_789_25,
                0.059_998_505_134_868_55,
            ),
            EXACT,
        );
        close(
            low,
            (
                1_418.910_949_584_518_2,
                115.693_290_929_987_47,
                0.059_998_517_422_438_245,
            ),
            EXACT,
        );
        assert_eq!(Score::Draw.reversed(), Score::Draw);
        assert_eq!(Score::Loss.reversed(), Score::Win);
    }

    #[test]
    fn a_period_without_games_and_a_new_season() {
        let quiet = period(rating(1500.0, 200.0), &[]);
        close(quiet, (1500.0, 200.271_416_698_770_65, VOLATILITY), EXACT);
        let carried = Rating {
            rating: 1712.5,
            deviation: 61.0,
            volatility: 0.059,
        }
        .new_season();
        assert_eq!(
            (carried.rating, carried.deviation, carried.volatility),
            (1712.5, DEVIATION, 0.059)
        );
        assert!(carried.is_valid() && Rating::default().is_valid());
        for bad in [
            Rating {
                rating: f64::NAN,
                ..carried
            },
            Rating {
                deviation: 0.0,
                ..carried
            },
            Rating {
                volatility: f64::INFINITY,
                ..carried
            },
        ] {
            assert!(!bad.is_valid(), "{bad:?}");
        }
    }

    #[test]
    fn a_rating_reads_back_bit_for_bit() {
        let (a, _) = game(rating(1543.21, 97.3), rating(1488.8, 210.0), Score::Loss);
        let text = serde_json::to_string(&a).unwrap();
        let back: Rating = serde_json::from_str(&text).unwrap();
        assert_eq!(
            [back.rating, back.deviation, back.volatility].map(f64::to_bits),
            [a.rating, a.deviation, a.volatility].map(f64::to_bits),
            "{text}"
        );
    }
}
