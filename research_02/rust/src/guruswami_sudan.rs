//! Bounded Guruswami-Sudan list decoding over GF(256).
//!
//! The interpolation stage uses Hasse multiplicities.
//! Polynomial-root extraction is budget-bounded and returns
//! an explicit error if the search cannot be completed.

use crate::{
    GF256,
    interpolation::{InterpolationError, interpolate},
    reed_solomon::ReedSolomon,
    root_finding::{RootFindingError, polynomial_roots},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuruswamiSudanError {
    InvalidWordLength,
    InvalidParameters,
    Interpolation(InterpolationError),
    RootFinding(RootFindingError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedCandidate {
    pub message: Vec<GF256>,
    pub distance: usize,
}

impl ReedSolomon {
    pub fn decode_list_guruswami_sudan(
        &self,
        received: &[GF256],
        radius: usize,
        multiplicity: usize,
        root_budget: usize,
    ) -> Result<Vec<DecodedCandidate>, GuruswamiSudanError> {
        if received.len() != self.length() {
            return Err(GuruswamiSudanError::InvalidWordLength);
        }

        if radius > self.length() || multiplicity == 0 || root_budget == 0 || self.dimension() < 2 {
            return Err(GuruswamiSudanError::InvalidParameters);
        }

        // A message agreeing in at least n-radius positions
        // yields multiplicity * (n-radius) intersection weight.
        //
        // A strict weighted-degree bound below this value
        // ensures Q(x, f(x)) vanishes identically.
        let agreement = self.length() - radius;

        let threshold = multiplicity
            .checked_mul(agreement)
            .ok_or(GuruswamiSudanError::InvalidParameters)?;

        if threshold == 0 {
            return Err(GuruswamiSudanError::InvalidParameters);
        }

        let degree_bound = threshold - 1;
        let y_weight = self.dimension() - 1;

        let points: Vec<_> = self
            .evaluation_points()
            .iter()
            .copied()
            .zip(received.iter().copied())
            .collect();

        let q = interpolate(&points, multiplicity, y_weight, degree_bound)
            .map_err(GuruswamiSudanError::Interpolation)?;

        // Weighted degree bounds both individual x and y degrees.
        let max_x_degree = degree_bound;
        let max_y_degree = degree_bound / y_weight;

        let roots = polynomial_roots(
            &q,
            self.dimension() - 1,
            max_x_degree,
            max_y_degree,
            root_budget,
        )
        .map_err(GuruswamiSudanError::RootFinding)?;

        let mut candidates = Vec::new();

        for mut message in roots {
            if message.len() > self.dimension() {
                continue;
            }

            message.resize(self.dimension(), GF256::ZERO);

            let codeword = self
                .encode(&message)
                .expect("candidate dimension must be valid");

            let distance = codeword
                .iter()
                .zip(received)
                .filter(|(a, b)| a != b)
                .count();

            if distance <= radius {
                candidates.push(DecodedCandidate { message, distance });
            }
        }

        candidates.sort_by(|a, b| {
            a.distance.cmp(&b.distance).then_with(|| {
                a.message
                    .iter()
                    .map(|value| value.0)
                    .cmp(b.message.iter().map(|value| value.0))
            })
        });

        candidates.dedup_by(|a, b| a.message == b.message);

        Ok(candidates)
    }
}
