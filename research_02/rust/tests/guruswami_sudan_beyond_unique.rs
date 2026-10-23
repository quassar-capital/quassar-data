use quassar_algebraic_coding::{GF256, reed_solomon::ReedSolomon};

#[test]
fn recovers_message_beyond_unique_decoding_radius() {
    let rs = ReedSolomon::new(7, 2).unwrap();
    let message = vec![GF256(13), GF256(21)];

    let mut received = rs.encode(&message).unwrap();

    for index in [0, 2, 4] {
        received[index] = received[index].add(GF256(1));
    }

    let candidates = rs
        .decode_list_guruswami_sudan(&received, 3, 2, 1_000_000)
        .unwrap();

    assert!(
        candidates
            .iter()
            .any(|candidate| { candidate.message == message && candidate.distance == 3 }),
        "The original message must be recovered beyond the unique-decoding radius"
    );

    assert!(
        candidates.iter().all(|candidate| candidate.distance <= 3),
        "All candidates must satisfy the Hamming-distance bound"
    );
}
