use quassar_algebraic_coding::{GF256, list_decoding::ListDecodeError, reed_solomon::ReedSolomon};

fn field(values: &[u8]) -> Vec<GF256> {
    values.iter().copied().map(GF256).collect()
}

#[test]
fn exact_codeword_is_found_at_radius_zero() {
    let code = ReedSolomon::new(5, 2).unwrap();
    let original = field(&[0x53, 0xA7]);
    let received = code.encode(&original).unwrap();

    let candidates = code.decode_list_exhaustive(&received, 0, 65_536).unwrap();

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].message, original);
    assert_eq!(candidates[0].distance, 0);
}

#[test]
fn recovers_original_beyond_unique_decoding_radius() {
    let code = ReedSolomon::new(5, 2).unwrap();
    assert_eq!(code.unique_decoding_radius(), 1);

    let original = field(&[0x53, 0xA7]);
    let mut received = code.encode(&original).unwrap();

    received[0] = received[0].add(GF256(1));
    received[3] = received[3].add(GF256(2));

    let candidates = code.decode_list_exhaustive(&received, 2, 65_536).unwrap();

    assert!(
        candidates
            .iter()
            .any(|candidate| { candidate.message == original && candidate.distance == 2 })
    );

    assert!(candidates.iter().all(|candidate| candidate.distance <= 2));
}

#[test]
fn radius_zero_rejects_corrupted_word() {
    let code = ReedSolomon::new(5, 2).unwrap();
    let mut received = code.encode(&field(&[1, 2])).unwrap();

    received[0] = received[0].add(GF256(1));

    let candidates = code.decode_list_exhaustive(&received, 0, 65_536).unwrap();

    assert!(candidates.is_empty());
}

#[test]
fn candidates_are_sorted_by_distance() {
    let code = ReedSolomon::new(5, 2).unwrap();
    let received = code.encode(&field(&[1, 2])).unwrap();

    let candidates = code.decode_list_exhaustive(&received, 3, 65_536).unwrap();

    assert!(!candidates.is_empty());

    assert!(
        candidates
            .windows(2)
            .all(|pair| { pair[0].distance <= pair[1].distance })
    );
}

#[test]
fn rejects_excessive_search_space() {
    let code = ReedSolomon::new(10, 5).unwrap();
    let received = vec![GF256::ZERO; 10];

    assert_eq!(
        code.decode_list_exhaustive(&received, 2, 65_536),
        Err(ListDecodeError::SearchSpaceTooLarge)
    );
}

#[test]
fn rejects_invalid_inputs() {
    let code = ReedSolomon::new(5, 2).unwrap();

    assert_eq!(
        code.decode_list_exhaustive(&field(&[1, 2]), 1, 65_536),
        Err(ListDecodeError::InvalidWordLength)
    );

    assert_eq!(
        code.decode_list_exhaustive(&[GF256::ZERO; 5], 6, 65_536),
        Err(ListDecodeError::InvalidRadius)
    );
}

#[test]
fn every_candidate_is_independently_verified() {
    let code = ReedSolomon::new(5, 2).unwrap();
    let received = code.encode(&field(&[3, 7])).unwrap();

    let candidates = code.decode_list_exhaustive(&received, 3, 65_536).unwrap();

    for candidate in candidates {
        let encoded = code.encode(&candidate.message).unwrap();

        let actual_distance = encoded
            .iter()
            .zip(&received)
            .filter(|(a, b)| a != b)
            .count();

        assert_eq!(actual_distance, candidate.distance);
        assert!(actual_distance <= 3);
    }
}
