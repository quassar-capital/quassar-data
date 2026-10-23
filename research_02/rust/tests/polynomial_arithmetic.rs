use quassar_algebraic_coding::{GF256, polynomial::Polynomial};
fn poly(values: &[u8]) -> Polynomial {
    Polynomial::new(values.iter().copied().map(GF256).collect())
}

#[test]
fn canonical_zero_and_degree() {
    assert_eq!(poly(&[0, 0, 0]), Polynomial::zero());
    assert_eq!(Polynomial::zero().degree(), None);
    assert_eq!(poly(&[1, 2, 0, 0]).degree(), Some(1));
}

#[test]
fn polynomial_addition_and_multiplication() {
    let a = poly(&[1, 1]);
    let b = poly(&[1, 1]);

    assert_eq!(a.add(&b), Polynomial::zero());
    assert_eq!(a.mul(&b), poly(&[1, 0, 1]));
}

#[test]
fn horner_evaluation_matches_direct_computation() {
    let p = poly(&[0x53, 0xA7, 0x02, 0x11]);

    for value in 0..=255u8 {
        let x = GF256(value);
        let direct = p
            .coefficients()
            .iter()
            .enumerate()
            .fold(GF256::ZERO, |sum, (degree, &coefficient)| {
                sum.add(coefficient.mul(x.pow(degree as u32)))
            });

        assert_eq!(p.evaluate(x), direct);
    }
}

#[test]
fn formal_derivative_respects_characteristic_two() {
    let p = poly(&[1, 2, 3, 4, 5, 6]);
    assert_eq!(p.derivative(), poly(&[2, 0, 4, 0, 6]));
}

#[test]
fn polynomial_division_identity() {
    let dividends = [
        poly(&[]),
        poly(&[1]),
        poly(&[1, 2, 3]),
        poly(&[0x53, 0xA7, 0x02, 0x11, 0xFF]),
    ];

    let divisors = [
        poly(&[1]),
        poly(&[1, 1]),
        poly(&[2, 0, 1]),
        poly(&[0x53, 0xA7, 0x02]),
    ];

    for dividend in &dividends {
        for divisor in &divisors {
            let (quotient, remainder) = dividend.div_rem(divisor).unwrap();

            assert_eq!(quotient.mul(divisor).add(&remainder), *dividend);

            assert!(remainder.is_zero() || remainder.degree().unwrap() < divisor.degree().unwrap());
        }
    }
}

#[test]
fn division_by_zero_is_rejected() {
    assert!(poly(&[1, 2, 3]).div_rem(&Polynomial::zero()).is_err());
}

#[test]
fn distributivity_over_polynomial_addition() {
    let a = poly(&[1, 2, 3]);
    let b = poly(&[4, 5]);
    let c = poly(&[6, 7, 8, 9]);

    assert_eq!(a.mul(&b.add(&c)), a.mul(&b).add(&a.mul(&c)));
}

#[test]
fn scaling_and_evaluation_are_consistent() {
    let p = poly(&[1, 2, 3, 4]);
    let scalar = GF256(0xA7);

    for value in 0..=255u8 {
        let x = GF256(value);
        assert_eq!(p.scale(scalar).evaluate(x), p.evaluate(x).mul(scalar));
    }
}
