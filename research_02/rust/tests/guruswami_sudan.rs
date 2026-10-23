use quassar_algebraic_coding::{GF256, reed_solomon::ReedSolomon};

#[test]
fn recovers_message_after_one_symbol_error() {
    let rs = ReedSolomon::new(7, 2).unwrap();
    let message = vec![GF256(3), GF256(5)];

    let mut received = rs.encode(&message).unwrap();
    received[0] = received[0].add(GF256(1));

    let candidates = rs
        .decode_list_guruswami_sudan(&received, 1, 1, 100_000)
        .unwrap();

    assert!(
        candidates
            .iter()
            .any(|candidate| { candidate.message == message && candidate.distance == 1 }),
        "The original message must be recovered with Hamming distance 1"
    );

    assert!(
        candidates.iter().all(|candidate| candidate.distance <= 1),
        "Every returned candidate must satisfy the decoding radius"
    );
}

#[test]
fn recovers_message_without_errors() {
    let rs = ReedSolomon::new(7, 2).unwrap();
    let message = vec![GF256(11), GF256(9)];

    let received = rs.encode(&message).unwrap();

    let candidates = rs
        .decode_list_guruswami_sudan(&received, 0, 1, 100_000)
        .unwrap();

    assert!(
        candidates
            .iter()
            .any(|candidate| { candidate.message == message && candidate.distance == 0 }),
        "The original message must be recovered at distance zero"
    );
}
