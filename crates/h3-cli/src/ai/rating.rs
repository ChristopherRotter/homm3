//! Glicko-2 for `h3 ai eval` (E05 T7): copied from the server lane's ladder (`h3-server`'s
//! `src/rating.rs`, E16 M4 T1; ADR-0019 point 6; the user, 2026-10-19: "use what the server
//! already built"), without the ladder's rows, seasons and serialization. Mark Glickman's
//! "Example of the Glicko-2 system" (2013) with its published defaults; the runner rates its whole
//! run as one rating period (E05 T9b), where the ladder rates each game as one; a draw or a
//! truncation scores a half.
//!
//! Floats are allowed here: a tool's report, not game logic (ADR-0003 §3 bans the platform's
//! transcendentals for game logic). A rating may differ in its last bits between platforms.

use std::f64::consts::PI;

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
#[derive(Debug, Clone, Copy, PartialEq)]
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
/// the other's rating before the game, one rating period of one game (the server ladder's; kept
/// for its worked tests).
#[cfg(test)]
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
}
