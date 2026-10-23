//! Berlekamp-Welch unique decoding for evaluation Reed-Solomon codes.

use crate::{
    GF256, linear_system::solve_unique, polynomial::Polynomial, reed_solomon::ReedSolomon,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidWordLength,
    Uncorrectable,
}

impl ReedSolomon {
    /// Recover the message polynomial when the received word is
    /// within the unique-decoding radius of a codeword.
    pub fn decode_unique(&self, received: &[GF256]) -> Result<Vec<GF256>, DecodeError> {
        if received.len() != self.length() {
            return Err(DecodeError::InvalidWordLength);
        }

        let k = self.dimension();
        let t = self.unique_decoding_radius();

        for errors in 0..=t {
            let q_len = k + errors;
            let unknowns = q_len + errors;

            let mut matrix = Vec::with_capacity(self.length());
            let mut rhs = Vec::with_capacity(self.length());

            for (&x, &y) in self.evaluation_points().iter().zip(received) {
                let mut row = Vec::with_capacity(unknowns);
                let mut power = GF256::ONE;

                for _ in 0..q_len {
                    row.push(power);
                    power = power.mul(x);
                }

                power = GF256::ONE;

                for _ in 0..errors {
                    row.push(y.mul(power));
                    power = power.mul(x);
                }

                matrix.push(row);
                rhs.push(y.mul(x.pow(errors as u32)));
            }

            let Ok(solution) = solve_unique(&matrix, &rhs) else {
                continue;
            };

            let q = Polynomial::new(solution[..q_len].to_vec());

            let mut e_coefficients = solution[q_len..].to_vec();
            e_coefficients.push(GF256::ONE);
            let e = Polynomial::new(e_coefficients);

            let Ok((message, remainder)) = q.div_rem(&e) else {
                continue;
            };

            if !remainder.is_zero() || message.degree().is_some_and(|d| d >= k) {
                continue;
            }

            let mut coefficients = message.coefficients().to_vec();
            coefficients.resize(k, GF256::ZERO);

            let Ok(codeword) = self.encode(&coefficients) else {
                continue;
            };

            let distance = codeword
                .iter()
                .zip(received)
                .filter(|(a, b)| a != b)
                .count();

            if distance <= t {
                return Ok(coefficients);
            }
        }

        Err(DecodeError::Uncorrectable)
    }
}
