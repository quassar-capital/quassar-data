//! Exact Guruswami-Sudan interpolation over GF(256).
//!
//! Constructs a nonzero bivariate polynomial satisfying
//! prescribed Hasse multiplicity constraints.

use crate::{GF256, bivariate::BivariatePolynomial, nullspace::nullspace_basis};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpolationError {
    InvalidInput,
    DimensionOverflow,
    NoNonzeroSolution,
}

fn binomial_odd(n: usize, k: usize) -> bool {
    k <= n && (k & n) == k
}

fn monomials(degree_bound: usize, y_weight: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();

    for j in 0..=degree_bound / y_weight {
        for i in 0..=degree_bound - j * y_weight {
            result.push((i, j));
        }
    }

    result
}

pub fn interpolate(
    points: &[(GF256, GF256)],
    multiplicity: usize,
    y_weight: usize,
    degree_bound: usize,
) -> Result<BivariatePolynomial, InterpolationError> {
    if points.is_empty() || multiplicity == 0 || y_weight == 0 || degree_bound > 254 {
        return Err(InterpolationError::InvalidInput);
    }

    let count_per_point = multiplicity
        .checked_mul(multiplicity + 1)
        .and_then(|v| v.checked_div(2))
        .ok_or(InterpolationError::DimensionOverflow)?;

    let total_constraints = points
        .len()
        .checked_mul(count_per_point)
        .ok_or(InterpolationError::DimensionOverflow)?;

    let basis_monomials = monomials(degree_bound, y_weight);

    if basis_monomials.len() <= total_constraints {
        return Err(InterpolationError::NoNonzeroSolution);
    }

    let mut matrix = Vec::with_capacity(total_constraints);

    for &(x, y) in points {
        for a in 0..multiplicity {
            for b in 0..(multiplicity - a) {
                let row = basis_monomials
                    .iter()
                    .map(|&(i, j)| {
                        if i < a || j < b || !binomial_odd(i, a) || !binomial_odd(j, b) {
                            GF256::ZERO
                        } else {
                            x.pow((i - a) as u32).mul(y.pow((j - b) as u32))
                        }
                    })
                    .collect();

                matrix.push(row);
            }
        }
    }

    let kernel = nullspace_basis(&matrix, basis_monomials.len())
        .map_err(|_| InterpolationError::InvalidInput)?;

    let Some(solution) = kernel.first() else {
        return Err(InterpolationError::NoNonzeroSolution);
    };

    let terms: Vec<_> = basis_monomials
        .iter()
        .zip(solution)
        .filter_map(|(&(i, j), &coefficient)| {
            (coefficient != GF256::ZERO).then_some((i, j, coefficient))
        })
        .collect();

    let polynomial = BivariatePolynomial::new(&terms);

    if polynomial.is_zero()
        || polynomial.weighted_degree(1, y_weight) > Some(degree_bound)
        || points
            .iter()
            .any(|&(x, y)| !polynomial.vanishes_with_multiplicity(x, y, multiplicity))
    {
        return Err(InterpolationError::NoNonzeroSolution);
    }

    Ok(polynomial)
}
