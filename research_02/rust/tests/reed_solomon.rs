use quassar_algebraic_coding::{GF256, reed_solomon::ReedSolomon};

fn field(values: &[u8]) -> Vec<GF256> {
    values.iter().copied().map(GF256).collect()
}

fn hamming_distance(a: &[GF256], b: &[GF256]) -> usize {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

#[test]
fn rejects_invalid_parameters() {
    assert!(ReedSolomon::new(0, 0).is_err());
    assert!(ReedSolomon::new(10, 0).is_err());
    assert!(ReedSolomon::new(4, 5).is_err());
    assert!(ReedSolomon::new(256, 2).is_err());
    assert!(ReedSolomon::new(255, 255).is_ok());
}

#[test]
fn code_parameters_are_consistent() {
    let code = ReedSolomon::new(15, 9).unwrap();

    assert_eq!(code.length(), 15);
    assert_eq!(code.dimension(), 9);
    assert_eq!(code.minimum_distance(), 7);
    assert_eq!(code.unique_decoding_radius(), 3);
}

#[test]
fn evaluation_points_are_distinct() {
    let code = ReedSolomon::new(255, 10).unwrap();
    let mut observed = [false; 256];

    for point in code.evaluation_points() {
        assert_ne!(*point, GF256::ZERO);
        assert!(!observed[point.0 as usize]);
        observed[point.0 as usize] = true;
    }

    assert!(observed[1..].iter().all(|&present| present));
}

#[test]
fn encoded_messages_are_valid_codewords() {
    let code = ReedSolomon::new(15, 7).unwrap();

    let messages = [
        field(&[0, 0, 0, 0, 0, 0, 0]),
        field(&[1, 0, 0, 0, 0, 0, 0]),
        field(&[1, 2, 3, 4, 5, 6, 7]),
        field(&[0x53, 0xA7, 0xFF, 0, 1, 2, 3]),
    ];

    for message in messages {
        let encoded = code.encode(&message).unwrap();
        assert_eq!(encoded.len(), code.length());
        assert!(code.is_codeword(&encoded).unwrap());
    }
}

#[test]
fn encoding_is_linear() {
    let code = ReedSolomon::new(12, 5).unwrap();
    let a = field(&[1, 2, 3, 4, 5]);
    let b = field(&[9, 8, 7, 6, 5]);

    let sum: Vec<_> = a.iter().zip(&b).map(|(&x, &y)| x.add(y)).collect();

    let encoded_sum = code.encode(&sum).unwrap();
    let encoded_a = code.encode(&a).unwrap();
    let encoded_b = code.encode(&b).unwrap();

    for i in 0..code.length() {
        assert_eq!(encoded_sum[i], encoded_a[i].add(encoded_b[i]));
    }
}

#[test]
fn single_symbol_corruption_is_detected() {
    let code = ReedSolomon::new(15, 7).unwrap();
    let mut word = code.encode(&field(&[1, 2, 3, 4, 5, 6, 7])).unwrap();

    word[10] = word[10].add(GF256(0x53));

    assert!(!code.is_codeword(&word).unwrap());
}

#[test]
fn minimum_distance_is_exhaustively_verified_for_small_code() {
    let code = ReedSolomon::new(5, 2).unwrap();
    let zero = code.encode(&field(&[0, 0])).unwrap();
    let mut minimum = usize::MAX;

    for a in 0..=255u8 {
        for b in 0..=255u8 {
            if a == 0 && b == 0 {
                continue;
            }

            let word = code.encode(&[GF256(a), GF256(b)]).unwrap();
            minimum = minimum.min(hamming_distance(&word, &zero));
        }
    }

    assert_eq!(minimum, code.minimum_distance());
    assert_eq!(minimum, 4);
}

#[test]
fn rejects_invalid_word_and_message_lengths() {
    let code = ReedSolomon::new(10, 4).unwrap();

    assert!(code.encode(&field(&[1, 2, 3])).is_err());
    assert!(code.is_codeword(&field(&[0; 9])).is_err());
}
