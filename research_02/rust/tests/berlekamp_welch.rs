use quassar_algebraic_coding::{GF256, berlekamp_welch::DecodeError, reed_solomon::ReedSolomon};

fn message(values: &[u8]) -> Vec<GF256> {
    values.iter().copied().map(GF256).collect()
}

fn corrupt(word: &mut [GF256], positions: &[usize]) {
    for (i, &position) in positions.iter().enumerate() {
        let error = GF256((i as u8) + 1);
        word[position] = word[position].add(error);
    }
}

#[test]
fn recovers_message_without_errors() {
    let code = ReedSolomon::new(15, 9).unwrap();
    let original = message(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let received = code.encode(&original).unwrap();

    assert_eq!(code.decode_unique(&received).unwrap(), original);
}

#[test]
fn corrects_single_symbol_error() {
    let code = ReedSolomon::new(15, 9).unwrap();
    let original = message(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let mut received = code.encode(&original).unwrap();

    corrupt(&mut received, &[4]);

    assert_eq!(code.decode_unique(&received).unwrap(), original);
}

#[test]
fn corrects_two_symbol_errors() {
    let code = ReedSolomon::new(15, 9).unwrap();
    let original = message(&[0x53, 0xA7, 3, 4, 5, 6, 7, 8, 9]);
    let mut received = code.encode(&original).unwrap();

    corrupt(&mut received, &[2, 11]);

    assert_eq!(code.decode_unique(&received).unwrap(), original);
}

#[test]
fn corrects_errors_at_unique_decoding_radius() {
    let code = ReedSolomon::new(15, 9).unwrap();
    assert_eq!(code.unique_decoding_radius(), 3);

    let original = message(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let mut received = code.encode(&original).unwrap();

    corrupt(&mut received, &[0, 7, 14]);

    assert_eq!(code.decode_unique(&received).unwrap(), original);
}

#[test]
fn corrects_errors_with_zero_leading_message_coefficients() {
    let code = ReedSolomon::new(15, 9).unwrap();
    let original = message(&[0x53, 0xA7, 3, 4, 0, 0, 0, 0, 0]);
    let mut received = code.encode(&original).unwrap();

    corrupt(&mut received, &[1, 6, 12]);

    assert_eq!(code.decode_unique(&received).unwrap(), original);
}

#[test]
fn rejects_invalid_received_length() {
    let code = ReedSolomon::new(15, 9).unwrap();

    assert_eq!(
        code.decode_unique(&message(&[1, 2, 3])),
        Err(DecodeError::InvalidWordLength)
    );
}

#[test]
fn rejects_a_constructed_word_outside_all_decoding_balls() {
    // For [5, 2], the unique-decoding radius is one.
    // Exhaustively enumerate all 65,536 codewords and construct
    // a received word at distance >= 2 from every codeword.
    let code = ReedSolomon::new(5, 2).unwrap();
    let mut received = code.encode(&message(&[1, 2])).unwrap();
    corrupt(&mut received, &[0, 1]);

    let minimum_distance = (0..=255u8)
        .flat_map(|a| (0..=255u8).map(move |b| (a, b)))
        .map(|(a, b)| {
            let word = code.encode(&[GF256(a), GF256(b)]).unwrap();
            word.iter().zip(&received).filter(|(x, y)| x != y).count()
        })
        .min()
        .unwrap();

    assert!(minimum_distance > code.unique_decoding_radius());
    assert_eq!(
        code.decode_unique(&received),
        Err(DecodeError::Uncorrectable)
    );
}
