//! Sparse bivariate polynomials over GF(256).
//!
//! Terms are represented as (x_degree, y_degree, coefficient).
//! Canonical construction combines duplicate monomials and removes zeros.

use crate::GF256;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BivariatePolynomial {
    terms: BTreeMap<(usize, usize), GF256>,
}

impl BivariatePolynomial {
    pub fn new(terms: &[(usize, usize, GF256)]) -> Self {
        let mut combined = BTreeMap::new();

        for &(x_degree, y_degree, coefficient) in terms {
            let key = (x_degree, y_degree);
            let current = combined.get(&key).copied().unwrap_or(GF256::ZERO);
            let updated = current.add(coefficient);

            if updated == GF256::ZERO {
                combined.remove(&key);
            } else {
                combined.insert(key, updated);
            }
        }

        Self { terms: combined }
    }

    pub fn zero() -> Self {
        Self {
            terms: BTreeMap::new(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    pub fn coefficient(&self, x_degree: usize, y_degree: usize) -> GF256 {
        self.terms
            .get(&(x_degree, y_degree))
            .copied()
            .unwrap_or(GF256::ZERO)
    }

    pub fn evaluate(&self, x: GF256, y: GF256) -> GF256 {
        self.terms
            .iter()
            .fold(GF256::ZERO, |sum, (&(i, j), &coefficient)| {
                sum.add(coefficient.mul(x.pow(i as u32)).mul(y.pow(j as u32)))
            })
    }

    pub fn weighted_degree(&self, x_weight: usize, y_weight: usize) -> Option<usize> {
        self.terms
            .keys()
            .map(|&(i, j)| {
                i.saturating_mul(x_weight)
                    .saturating_add(j.saturating_mul(y_weight))
            })
            .max()
    }

    /// Hasse derivative evaluated at a point.
    ///
    /// D^(a,b) Q(x,y) =
    /// sum q_ij * binom(i,a) * binom(j,b) * x^(i-a) * y^(j-b).
    ///
    /// In characteristic two, each binomial coefficient reduces to
    /// its parity, determined by the binary subset relation.
    pub fn hasse_derivative(&self, a: usize, b: usize, x: GF256, y: GF256) -> GF256 {
        self.terms
            .iter()
            .fold(GF256::ZERO, |sum, (&(i, j), &coefficient)| {
                if i < a || j < b {
                    return sum;
                }

                let x_binomial_odd = (a & i) == a;
                let y_binomial_odd = (b & j) == b;

                if !x_binomial_odd || !y_binomial_odd {
                    return sum;
                }

                sum.add(
                    coefficient
                        .mul(x.pow((i - a) as u32))
                        .mul(y.pow((j - b) as u32)),
                )
            })
    }

    pub fn vanishes_with_multiplicity(&self, x: GF256, y: GF256, multiplicity: usize) -> bool {
        for a in 0..multiplicity {
            for b in 0..(multiplicity - a) {
                if self.hasse_derivative(a, b, x, y) != GF256::ZERO {
                    return false;
                }
            }
        }

        true
    }
}
