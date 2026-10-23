//! Exhaustive bounded list decoding for small Reed-Solomon codes.
//!
//! This reference implementation enumerates every message in GF(256)^k.
//! It is intended for independently checking advanced list decoders,
//! not for production-scale decoding.

use crate::{GF256, reed_solomon::ReedSolomon};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListDecodeError {
    InvalidWordLength,
    InvalidRadius,
    SearchSpaceTooLarge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListCandidate {
    pub message: Vec<GF256>,
    pub distance: usize,
}

impl ReedSolomon {
    pub fn decode_list_exhaustive(
        &self,
        received: &[GF256],
        radius: usize,
        max_messages: u64,
    ) -> Result<Vec<ListCandidate>, ListDecodeError> {
        if received.len() != self.length() {
            return Err(ListDecodeError::InvalidWordLength);
        }

        if radius > self.length() {
            return Err(ListDecodeError::InvalidRadius);
        }

        let search_space = 256u64
            .checked_pow(self.dimension() as u32)
            .ok_or(ListDecodeError::SearchSpaceTooLarge)?;

        if search_space > max_messages {
            return Err(ListDecodeError::SearchSpaceTooLarge);
        }

        let mut candidates = Vec::new();

        for index in 0..search_space {
            let mut value = index;
            let mut message = vec![GF256::ZERO; self.dimension()];

            for coefficient in &mut message {
                *coefficient = GF256((value & 0xFF) as u8);
                value >>= 8;
            }

            let codeword = self
                .encode(&message)
                .expect("enumerated message has valid dimension");

            let distance = codeword
                .iter()
                .zip(received)
                .filter(|(a, b)| a != b)
                .count();

            if distance <= radius {
                candidates.push(ListCandidate { message, distance });
            }
        }

        candidates.sort_by(|a, b| {
            a.distance.cmp(&b.distance).then_with(|| {
                a.message
                    .iter()
                    .map(|x| x.0)
                    .cmp(b.message.iter().map(|x| x.0))
            })
        });

        Ok(candidates)
    }
}
