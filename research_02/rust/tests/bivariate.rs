use quassar_algebraic_coding::{GF256, bivariate::BivariatePolynomial};
fn f(value: u8) -> GF256 {
    GF256(value)
}

#[test]
fn combines_duplicate_monomials_in_characteristic_two() {
    let polynomial = BivariatePolynomial::new(&[
        (2, 1, f(0x53)),
        (2, 1, f(0xA7)),
        (2, 1, f(0xF4)),
        (0, 0, f(7)),
    ]);

    assert_eq!(polynomial.term_count(), 1);
    assert_eq!(polynomial.coefficient(2, 1), GF256::ZERO);
    assert_eq!(polynomial.coefficient(0, 0), f(7));
}

#[test]
fn zero_polynomial_has_no_weighted_degree() {
    let polynomial = BivariatePolynomial::zero();

    assert!(polynomial.is_zero());
    assert_eq!(polynomial.term_count(), 0);
    assert_eq!(polynomial.weighted_degree(1, 3), None);
    assert_eq!(polynomial.evaluate(f(5), f(9)), GF256::ZERO);
}

#[test]
fn evaluates_bivariate_polynomial_exactly() {
    // Q(x,y) = x^2 + xy + y^2 + 1.
    let polynomial = BivariatePolynomial::new(&[
        (2, 0, GF256::ONE),
        (1, 1, GF256::ONE),
        (0, 2, GF256::ONE),
        (0, 0, GF256::ONE),
    ]);

    let x = f(3);
    let y = f(5);

    let expected = x.mul(x).add(x.mul(y)).add(y.mul(y)).add(GF256::ONE);

    assert_eq!(polynomial.evaluate(x, y), expected);
}

#[test]
fn computes_weighted_degree() {
    let polynomial =
        BivariatePolynomial::new(&[(5, 0, GF256::ONE), (2, 2, GF256::ONE), (0, 3, GF256::ONE)]);

    assert_eq!(polynomial.weighted_degree(1, 4), Some(12));
    assert_eq!(polynomial.weighted_degree(1, 1), Some(5));
}

#[test]
fn hasse_derivative_detects_odd_binomial_coefficients() {
    // In characteristic two:
    // D_x^(1)(x^3) = x^2,
    // D_x^(2)(x^3) = x,
    // D_x^(1)(x^2) = 0.
    let cubic = BivariatePolynomial::new(&[(3, 0, GF256::ONE)]);

    let square = BivariatePolynomial::new(&[(2, 0, GF256::ONE)]);

    let x = f(7);

    assert_eq!(cubic.hasse_derivative(1, 0, x, GF256::ZERO), x.mul(x));

    assert_eq!(cubic.hasse_derivative(2, 0, x, GF256::ZERO), x);

    assert_eq!(square.hasse_derivative(1, 0, x, GF256::ZERO), GF256::ZERO);
}

#[test]
fn mixed_hasse_derivative_is_exact() {
    // Q(x,y) = x^3 y^2.
    // D^(1,2) Q = x^2.
    let polynomial = BivariatePolynomial::new(&[(3, 2, GF256::ONE)]);

    let x = f(11);
    let y = f(13);

    assert_eq!(polynomial.hasse_derivative(1, 2, x, y), x.mul(x));

    assert_eq!(polynomial.hasse_derivative(2, 1, x, y), GF256::ZERO);
}

#[test]
fn verifies_multiplicity_two_at_origin() {
    // Q(x,y) = x^2 + xy + y^2.
    let polynomial =
        BivariatePolynomial::new(&[(2, 0, GF256::ONE), (1, 1, GF256::ONE), (0, 2, GF256::ONE)]);

    assert!(polynomial.vanishes_with_multiplicity(GF256::ZERO, GF256::ZERO, 2));

    assert!(!polynomial.vanishes_with_multiplicity(GF256::ZERO, GF256::ZERO, 3));
}

#[test]
fn verifies_shifted_multiplicity_three() {
    // Q(x,y) = (x+a)^3 over characteristic two.
    let a = f(0x53);
    let a_squared = a.mul(a);
    let a_cubed = a_squared.mul(a);

    let polynomial = BivariatePolynomial::new(&[
        (3, 0, GF256::ONE),
        (2, 0, a),
        (1, 0, a_squared),
        (0, 0, a_cubed),
    ]);

    assert!(polynomial.vanishes_with_multiplicity(a, f(0xA7), 3));

    assert!(!polynomial.vanishes_with_multiplicity(a, f(0xA7), 4));
}

#[test]
fn multiplicity_zero_imposes_no_constraints() {
    let polynomial = BivariatePolynomial::new(&[(0, 0, GF256::ONE)]);

    assert!(polynomial.vanishes_with_multiplicity(f(17), f(29), 0));

    assert!(!polynomial.vanishes_with_multiplicity(f(17), f(29), 1));
}
