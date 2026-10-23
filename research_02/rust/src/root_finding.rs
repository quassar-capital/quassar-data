//! Exact, budget-bounded extraction of polynomial roots Q(x, f(x)) = 0.
//!
//! Uses coefficient lifting over GF(256), with explicit degree and
//! search budgets. Exhaustiveness is guaranteed only when the
//! algorithm completes without reaching its budget.

use crate::{GF256, bivariate::BivariatePolynomial};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootFindingError {
    InvalidInput,
    SearchLimitExceeded,
}

fn substitute(
    q: &BivariatePolynomial,
    coefficients: &[GF256],
    max_x_degree: usize,
    max_y_degree: usize,
) -> Vec<GF256> {
    let mut powers = vec![vec![GF256::ONE]];

    for _ in 0..max_y_degree {
        let previous = powers.last().unwrap();
        let mut next = vec![GF256::ZERO; previous.len() + coefficients.len() - 1];

        for (i, &a) in previous.iter().enumerate() {
            for (j, &b) in coefficients.iter().enumerate() {
                next[i + j] = next[i + j].add(a.mul(b));
            }
        }

        powers.push(next);
    }

    let mut result = vec![GF256::ZERO; max_x_degree + max_y_degree * (coefficients.len() - 1) + 1];

    for i in 0..=max_x_degree {
        for (j, power) in powers.iter().enumerate() {
            let coefficient = q.coefficient(i, j);

            if coefficient == GF256::ZERO {
                continue;
            }

            for (k, &value) in power.iter().enumerate() {
                result[i + k] = result[i + k].add(coefficient.mul(value));
            }
        }
    }

    result
}

fn has_zero_low_coefficients(
    q: &BivariatePolynomial,
    coefficients: &[GF256],
    max_x_degree: usize,
    max_y_degree: usize,
) -> bool {
    let expansion = substitute(q, coefficients, max_x_degree, max_y_degree);

    expansion
        .iter()
        .take(coefficients.len())
        .all(|&value| value == GF256::ZERO)
}

fn is_exact_root(
    q: &BivariatePolynomial,
    coefficients: &[GF256],
    max_x_degree: usize,
    max_y_degree: usize,
) -> bool {
    substitute(q, coefficients, max_x_degree, max_y_degree)
        .iter()
        .all(|&value| value == GF256::ZERO)
}

struct SearchContext<'a> {
    q: &'a BivariatePolynomial,
    degree_limit: usize,
    max_x_degree: usize,
    max_y_degree: usize,
    budget: usize,
    visited: usize,
    solutions: Vec<Vec<GF256>>,
}

fn search(
    context: &mut SearchContext<'_>,
    coefficients: &mut Vec<GF256>,
) -> Result<(), RootFindingError> {
    if coefficients.len() == context.degree_limit + 1 {
        if is_exact_root(
            context.q,
            coefficients,
            context.max_x_degree,
            context.max_y_degree,
        ) {
            let mut root = coefficients.clone();

            while root.last() == Some(&GF256::ZERO) && root.len() > 1 {
                root.pop();
            }

            if !context.solutions.contains(&root) {
                context.solutions.push(root);
            }
        }

        return Ok(());
    }

    for value in 0u16..=255 {
        if context.visited >= context.budget {
            return Err(RootFindingError::SearchLimitExceeded);
        }

        context.visited += 1;
        coefficients.push(GF256(value as u8));

        if has_zero_low_coefficients(
            context.q,
            coefficients,
            context.max_x_degree,
            context.max_y_degree,
        ) {
            search(context, coefficients)?;
        }

        coefficients.pop();
    }

    Ok(())
}

pub fn polynomial_roots(
    q: &BivariatePolynomial,
    degree_limit: usize,
    max_x_degree: usize,
    max_y_degree: usize,
    budget: usize,
) -> Result<Vec<Vec<GF256>>, RootFindingError> {
    if q.is_zero()
        || degree_limit > 254
        || max_x_degree > 254
        || max_y_degree == 0
        || max_y_degree > 254
        || budget == 0
    {
        return Err(RootFindingError::InvalidInput);
    }

    let mut context = SearchContext {
        q,
        degree_limit,
        max_x_degree,
        max_y_degree,
        budget,
        visited: 0,
        solutions: Vec::new(),
    };

    search(&mut context, &mut Vec::new())?;

    Ok(context.solutions)
}
