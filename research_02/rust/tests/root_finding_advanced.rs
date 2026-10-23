use quassar_algebraic_coding::{
    GF256, bivariate::BivariatePolynomial, root_finding::polynomial_roots,
};

fn f(value: u8) -> GF256 {
    GF256(value)
}

#[test]
fn recovers_two_distinct_polynomial_roots() {
    // Q(x,y) = (y+x)(y+x+1)
    let q = BivariatePolynomial::new(&[(0, 2, f(1)), (0, 1, f(1)), (2, 0, f(1)), (1, 0, f(1))]);

    let roots = polynomial_roots(&q, 1, 2, 2, 100_000).unwrap();

    assert_eq!(roots, vec![vec![f(0), f(1)], vec![f(1), f(1)]]);
}

#[test]
fn recovers_repeated_polynomial_root() {
    // Q(x,y) = (y+x)^2 = y^2+x^2
    let q = BivariatePolynomial::new(&[(0, 2, f(1)), (2, 0, f(1))]);

    let roots = polynomial_roots(&q, 1, 2, 2, 100_000).unwrap();

    assert_eq!(roots, vec![vec![f(0), f(1)]]);
}
