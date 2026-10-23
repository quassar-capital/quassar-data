//! Algebraic coding research: exact arithmetic over GF(2^8).
//!
//! Field representation: GF(2)[x] / (x^8 + x^4 + x^3 + x^2 + 1).
//! Elements are represented by polynomial coefficients packed into u8.

/// An element of the finite field GF(256).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GF256(pub u8);

impl GF256 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    /// Field addition, equivalent to subtraction in characteristic two.
    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    /// Multiplication using polynomial reduction modulo 0x11D.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: Self) -> Self {
        let mut a = self.0;
        let mut b = rhs.0;
        let mut result = 0u8;

        for _ in 0..8 {
            if b & 1 != 0 {
                result ^= a;
            }

            let overflow = a & 0x80 != 0;
            a <<= 1;

            if overflow {
                a ^= 0x1D;
            }

            b >>= 1;
        }

        Self(result)
    }

    /// Exponentiation by repeated squaring.
    pub fn pow(self, mut exponent: u32) -> Self {
        let mut base = self;
        let mut result = Self::ONE;

        while exponent != 0 {
            if exponent & 1 != 0 {
                result = result.mul(base);
            }
            base = base.mul(base);
            exponent >>= 1;
        }

        result
    }

    /// Multiplicative inverse, defined for every nonzero field element.
    pub fn inverse(self) -> Option<Self> {
        if self == Self::ZERO {
            None
        } else {
            Some(self.pow(254))
        }
    }

    /// Division, undefined when the divisor is zero.
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        rhs.inverse().map(|inverse| self.mul(inverse))
    }
}

#[cfg(test)]
mod tests {
    use super::GF256;

    #[test]
    fn additive_group_laws() {
        for a in 0..=255u8 {
            let x = GF256(a);
            assert_eq!(x.add(GF256::ZERO), x);
            assert_eq!(x.add(x), GF256::ZERO);
        }
    }

    #[test]
    fn multiplicative_inverses_are_exact() {
        assert_eq!(GF256::ZERO.inverse(), None);

        for a in 1..=255u8 {
            let x = GF256(a);
            let inverse = x.inverse().expect("nonzero element must be invertible");
            assert_eq!(x.mul(inverse), GF256::ONE);
        }
    }

    #[test]
    fn distributive_law_exhaustive_over_pairs() {
        for a in 0..=255u8 {
            for b in 0..=255u8 {
                let x = GF256(a);
                let y = GF256(b);
                let z = GF256(0xA7);

                assert_eq!(x.mul(y.add(z)), x.mul(y).add(x.mul(z)));
            }
        }
    }

    #[test]
    fn division_round_trip() {
        for a in 0..=255u8 {
            let x = GF256(a);

            for b in 1..=255u8 {
                let y = GF256(b);
                let quotient = x.checked_div(y).unwrap();

                assert_eq!(quotient.mul(y), x);
            }
        }
    }
}

pub mod polynomial;

pub mod reed_solomon;

pub mod linear_system;

pub mod berlekamp_welch;

pub mod list_decoding;

pub mod bivariate;

pub mod nullspace;

pub mod interpolation;

pub mod root_finding;

pub mod guruswami_sudan;
