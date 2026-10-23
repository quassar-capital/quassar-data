use quassar_algebraic_coding::{
    GF256,
    bivariate::BivariatePolynomial,
    root_finding::{RootFindingError, polynomial_roots},
};

fn f(value: u8) -> GF256 {
    GF256(value)
}

#[test]
fn recovers_constant_polynomial_root() {
    // Q(x,y) = y + 7.
    let q = BivariatePolynomial::new(&[(0, 1, f(1)), (0, 0, f(7))]);

    let roots = polynomial_roots(&q, 0, 0, 1, 256).unwrap();

    assert_eq!(roots, vec![vec![f(7)]]);
}

#[test]
fn recovers_linear_polynomial_root() {
    // Q(x,y) = y + x + 3.
    // The exact root is f(x) = x + 3.
    let q = BivariatePolynomial::new(&[(0, 1, f(1)), (1, 0, f(1)), (0, 0, f(3))]);

    let roots = polynomial_roots(&q, 1, 1, 1, 512).unwrap();

    assert_eq!(roots, vec![vec![f(3), f(1)]]);
}

#[test]
fn rejects_nonroot_polynomials() {
    // Q(x,y) = y^2 + y + 1 has no roots in GF(256):
    // GF(256) contains the two roots of this polynomial, so instead
    // use Q(x,y) = x, which cannot vanish identically after y=f(x).
    let q = BivariatePolynomial::new(&[(1, 0, f(1))]);

    let roots = polynomial_roots(&q, 0, 1, 1, 256).unwrap();

    assert!(roots.is_empty());
}

#[test]
fn reports_exhausted_search_budget() {
    let q = BivariatePolynomial::new(&[(0, 1, f(1)), (0, 0, f(7))]);

    assert_eq!(
        polynomial_roots(&q, 0, 0, 1, 1),
        Err(RootFindingError::SearchLimitExceeded)
    );
}
