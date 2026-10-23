use quassar_algebraic_coding::{
    GF256,
    interpolation::{InterpolationError, interpolate},
};

fn f(value: u8) -> GF256 {
    GF256(value)
}

fn verify(points: &[(GF256, GF256)], multiplicity: usize, y_weight: usize, bound: usize) {
    let q = interpolate(points, multiplicity, y_weight, bound).unwrap();

    assert!(!q.is_zero());
    assert!(q.weighted_degree(1, y_weight).unwrap() <= bound);

    for &(x, y) in points {
        assert!(q.vanishes_with_multiplicity(x, y, multiplicity));

        for a in 0..multiplicity {
            for b in 0..multiplicity - a {
                assert_eq!(q.hasse_derivative(a, b, x, y), GF256::ZERO);
            }
        }
    }
}

#[test]
fn interpolates_one_point_with_multiplicity_one() {
    verify(&[(f(3), f(7))], 1, 1, 1);
}

#[test]
fn interpolates_multiple_distinct_points() {
    verify(&[(f(1), f(2)), (f(3), f(5)), (f(7), f(11))], 1, 2, 3);
}

#[test]
fn satisfies_second_order_hasse_constraints() {
    verify(&[(f(0x53), f(0xA7)), (f(0x21), f(0x42))], 2, 2, 4);
}

#[test]
fn satisfies_third_order_hasse_constraints() {
    verify(&[(f(0x53), f(0xA7))], 3, 1, 4);
}

#[test]
fn handles_repeated_interpolation_points() {
    verify(&[(f(3), f(7)), (f(3), f(7)), (f(5), f(9))], 1, 1, 3);
}

#[test]
fn rejects_empty_points() {
    assert_eq!(
        interpolate(&[], 1, 1, 2),
        Err(InterpolationError::InvalidInput)
    );
}

#[test]
fn rejects_zero_multiplicity_and_weight() {
    let points = [(f(1), f(2))];

    assert_eq!(
        interpolate(&points, 0, 1, 2),
        Err(InterpolationError::InvalidInput)
    );

    assert_eq!(
        interpolate(&points, 1, 0, 2),
        Err(InterpolationError::InvalidInput)
    );
}

#[test]
fn rejects_insufficient_monomial_space() {
    let points = [(f(1), f(2)), (f(3), f(4))];

    assert_eq!(
        interpolate(&points, 1, 2, 1),
        Err(InterpolationError::NoNonzeroSolution)
    );
}

#[test]
fn rejects_degree_bound_above_supported_limit() {
    let points = [(f(1), f(2))];

    assert_eq!(
        interpolate(&points, 1, 1, 255),
        Err(InterpolationError::InvalidInput)
    );
}
