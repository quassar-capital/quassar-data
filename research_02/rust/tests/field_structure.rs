use quassar_algebraic_coding::GF256;
#[test]
fn primitive_element_has_order_255() {
    let alpha = GF256(0x02);
    let mut power = GF256::ONE;

    for exponent in 1..255 {
        power = power.mul(alpha);
        assert_ne!(
            power,
            GF256::ONE,
            "Unexpected multiplicative order at exponent {exponent}"
        );
    }

    power = power.mul(alpha);
    assert_eq!(power, GF256::ONE);
}

#[test]
fn powers_of_primitive_element_cover_all_nonzero_elements() {
    let alpha = GF256(0x02);
    let mut observed = [false; 256];
    let mut power = GF256::ONE;

    for _ in 0..255 {
        assert_ne!(power, GF256::ZERO);
        assert!(!observed[power.0 as usize]);
        observed[power.0 as usize] = true;
        power = power.mul(alpha);
    }

    assert_eq!(power, GF256::ONE);
    assert!(observed[1..].iter().all(|&present| present));
}

#[test]
fn multiplication_is_commutative_and_associative() {
    let samples = [
        GF256(0x00),
        GF256(0x01),
        GF256(0x02),
        GF256(0x53),
        GF256(0xA7),
        GF256(0xFF),
    ];

    for &a in &samples {
        for &b in &samples {
            assert_eq!(a.mul(b), b.mul(a));

            for &c in &samples {
                assert_eq!(a.mul(b).mul(c), a.mul(b.mul(c)));
            }
        }
    }
}
