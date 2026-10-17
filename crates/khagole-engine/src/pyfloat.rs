//! Python float semantics the reference implementation relies on. Porting `x % 360.0`
//! or `round(x, 2)` naively changes results at boundaries (a planet at 29°59'59.99"
//! must land in the same sign, an orb must round to the same hundredth).

/// CPython `divmod(a, b)` for floats: floor division and a remainder with the sign of `b`.
pub fn divmod(a: f64, b: f64) -> (f64, f64) {
    let mut m = a % b;
    let mut div = (a - m) / b;
    if m != 0.0 {
        if (b < 0.0) != (m < 0.0) {
            m += b;
            div -= 1.0;
        }
    } else {
        m = 0.0_f64.copysign(b);
    }
    let floordiv = if div != 0.0 {
        let mut f = div.floor();
        if div - f > 0.5 {
            f += 1.0;
        }
        f
    } else {
        0.0_f64.copysign(a / b)
    };
    (floordiv, m)
}

/// CPython `a % b`.
pub fn rem(a: f64, b: f64) -> f64 {
    divmod(a, b).1
}

/// CPython `a // b`.
pub fn floordiv(a: f64, b: f64) -> f64 {
    divmod(a, b).0
}

/// Angular distance between two longitudes in degrees, 0-180: `abs(a - b) % 360`,
/// folded.
pub fn separation(a: f64, b: f64) -> f64 {
    let d = rem((a - b).abs(), 360.0);
    if d > 180.0 {
        360.0 - d
    } else {
        d
    }
}

/// An angle in degrees brought into [-180, 180): `(x + 180) % 360 - 180`.
pub fn wrap180(x: f64) -> f64 {
    rem(x + 180.0, 360.0) - 180.0
}

/// CPython `round(x, ndigits)`: correctly rounded, ties to even on the exact binary value.
/// Rust's float formatting rounds the same way.
pub fn round(x: f64, ndigits: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{x:.ndigits$}").parse().unwrap()
}

/// CPython `round(x)`: nearest integer, ties to even.
pub fn round_int(x: f64) -> i64 {
    x.round_ties_even() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulo_follows_python() {
        assert_eq!(rem(-30.0, 360.0), 330.0);
        assert_eq!(rem(370.0, 360.0), 10.0);
        assert_eq!(rem(-0.0, 360.0), 0.0);
        // Python: (-1e-18) % 360.0 == 360.0
        assert_eq!(rem(-1e-18, 360.0), 360.0);
    }

    #[test]
    fn floordiv_follows_python() {
        // Python: 1.0 // 0.1 == 9.0, although floor(1.0 / 0.1) == 10.0
        assert_eq!((1.0_f64 / 0.1).floor(), 10.0);
        assert_eq!(floordiv(1.0, 0.1), 9.0);
        assert_eq!(floordiv(89.99999999999999, 30.0), 2.0);
        assert_eq!(floordiv(-1.0, 30.0), -1.0);
    }

    #[test]
    fn round_follows_python() {
        assert_eq!(round(0.125, 2), 0.12);
        assert_eq!(round(0.375, 2), 0.38);
        assert_eq!(round(2.675, 2), 2.67);
        assert_eq!(round(1.0000005, 6), 1.000001);
        assert_eq!(round_int(2.5), 2);
        assert_eq!(round_int(3.5), 4);
    }
}
