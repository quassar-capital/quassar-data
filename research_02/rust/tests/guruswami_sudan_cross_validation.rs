use std::collections::BTreeSet;

use quassar_algebraic_coding::{GF256, reed_solomon::ReedSolomon};

#[test]
fn algebraic_decoder_matches_exhaustive_reference() {
    let rs = ReedSolomon::new(7, 2).unwrap();
    let original = vec![GF256(13), GF256(21)];

    let mut received = rs.encode(&original).unwrap();

    for index in [0, 2, 4] {
        received[index] = received[index].add(GF256(1));
    }

    let algebraic = rs
        .decode_list_guruswami_sudan(&received, 3, 2, 1_000_000)
        .unwrap();

    let algebraic_set: BTreeSet<(u8, u8)> = algebraic
        .iter()
        .map(|candidate| (candidate.message[0].0, candidate.message[1].0))
        .collect();

    let mut exhaustive_set = BTreeSet::new();

    for a in 0u16..256 {
        for b in 0u16..256 {
            let message = vec![GF256(a as u8), GF256(b as u8)];
            let codeword = rs.encode(&message).unwrap();

            let distance = codeword
                .iter()
                .zip(&received)
                .filter(|(x, y)| x != y)
                .count();

            if distance <= 3 {
                exhaustive_set.insert((a as u8, b as u8));
            }
        }
    }

    assert_eq!(
        algebraic_set, exhaustive_set,
        "Algebraic and exhaustive decoding must return identical candidate sets"
    );

    assert!(algebraic_set.contains(&(13, 21)));
}
