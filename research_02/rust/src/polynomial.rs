//! Exact polynomial arithmetic over GF(256).
//!
//! Coefficients are stored in ascending degree order.

use crate::GF256;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial {
    coefficients: Vec<GF256>,
}

impl Polynomial {
    pub fn new(mut coefficients: Vec<GF256>) -> Self {
        while coefficients.last() == Some(&GF256::ZERO) {
            coefficients.pop();
        }
        Self { coefficients }
    }

    pub fn zero() -> Self {
        Self::new(Vec::new())
    }

    pub fn one() -> Self {
        Self::new(vec![GF256::ONE])
    }

    pub fn coefficients(&self) -> &[GF256] {
        &self.coefficients
    }

    pub fn degree(&self) -> Option<usize> {
        self.coefficients.len().checked_sub(1)
    }

    pub fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    pub fn leading_coefficient(&self) -> GF256 {
        self.coefficients.last().copied().unwrap_or(GF256::ZERO)
    }

    pub fn add(&self, rhs: &Self) -> Self {
        let length = self.coefficients.len().max(rhs.coefficients.len());
        let mut result = vec![GF256::ZERO; length];

        for (i, value) in result.iter_mut().enumerate() {
            let a = self.coefficients.get(i).copied().unwrap_or(GF256::ZERO);
            let b = rhs.coefficients.get(i).copied().unwrap_or(GF256::ZERO);
            *value = a.add(b);
        }

        Self::new(result)
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }

        let mut result = vec![GF256::ZERO; self.coefficients.len() + rhs.coefficients.len() - 1];

        for (i, &a) in self.coefficients.iter().enumerate() {
            for (j, &b) in rhs.coefficients.iter().enumerate() {
                result[i + j] = result[i + j].add(a.mul(b));
            }
        }

        Self::new(result)
    }

    pub fn scale(&self, scalar: GF256) -> Self {
        Self::new(
            self.coefficients
                .iter()
                .map(|&coefficient| coefficient.mul(scalar))
                .collect(),
        )
    }

    pub fn evaluate(&self, x: GF256) -> GF256 {
        self.coefficients
            .iter()
            .rev()
            .fold(GF256::ZERO, |acc, &coefficient| acc.mul(x).add(coefficient))
    }

    pub fn derivative(&self) -> Self {
        let mut result = Vec::with_capacity(self.coefficients.len().saturating_sub(1));

        for i in 1..self.coefficients.len() {
            result.push(if i % 2 == 1 {
                self.coefficients[i]
            } else {
                GF256::ZERO
            });
        }

        Self::new(result)
    }

    pub fn div_rem(&self, divisor: &Self) -> Result<(Self, Self), &'static str> {
        if divisor.is_zero() {
            return Err("polynomial division by zero");
        }

        let mut remainder = self.clone();
        let divisor_degree = divisor.degree().unwrap();

        if remainder
            .degree()
            .is_none_or(|degree| degree < divisor_degree)
        {
            return Ok((Self::zero(), remainder));
        }

        let quotient_length = remainder.degree().unwrap() - divisor_degree + 1;
        let mut quotient = vec![GF256::ZERO; quotient_length];

        while let Some(remainder_degree) = remainder.degree() {
            if remainder_degree < divisor_degree {
                break;
            }

            let shift = remainder_degree - divisor_degree;
            let factor = remainder
                .leading_coefficient()
                .checked_div(divisor.leading_coefficient())
                .ok_or("noninvertible leading coefficient")?;

            quotient[shift] = quotient[shift].add(factor);

            let mut subtraction = vec![GF256::ZERO; shift];
            subtraction.extend(
                divisor
                    .coefficients()
                    .iter()
                    .map(|&coefficient| coefficient.mul(factor)),
            );

            remainder = remainder.add(&Self::new(subtraction));
        }

        Ok((Self::new(quotient), remainder))
    }
}
