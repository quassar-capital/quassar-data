use quassar_algebraic_coding::{
    GF256,
    linear_system::{LinearSolveError, solve_unique},
};
fn f(value: u8) -> GF256 {
    GF256(value)
}

fn verify_solution(matrix: &[Vec<GF256>], rhs: &[GF256], solution: &[GF256]) {
    for (row, &expected) in matrix.iter().zip(rhs) {
        let actual = row
            .iter()
            .zip(solution)
            .fold(GF256::ZERO, |sum, (&a, &x)| sum.add(a.mul(x)));

        assert_eq!(actual, expected);
    }
}

#[test]
fn solves_identity_system() {
    let matrix = vec![
        vec![f(1), f(0), f(0)],
        vec![f(0), f(1), f(0)],
        vec![f(0), f(0), f(1)],
    ];
    let rhs = vec![f(0x53), f(0xA7), f(0xFF)];

    assert_eq!(solve_unique(&matrix, &rhs).unwrap(), rhs);
}

#[test]
fn handles_row_pivoting() {
    let matrix = vec![vec![f(0), f(1)], vec![f(1), f(1)]];
    let rhs = vec![f(7), f(2)];

    let solution = solve_unique(&matrix, &rhs).unwrap();

    assert_eq!(solution, vec![f(5), f(7)]);
    verify_solution(&matrix, &rhs, &solution);
}

#[test]
fn solves_overdetermined_consistent_system() {
    let matrix = vec![vec![f(1), f(0)], vec![f(0), f(1)], vec![f(1), f(1)]];
    let rhs = vec![f(3), f(5), f(6)];

    let solution = solve_unique(&matrix, &rhs).unwrap();

    assert_eq!(solution, vec![f(3), f(5)]);
    verify_solution(&matrix, &rhs, &solution);
}

#[test]
fn rejects_inconsistent_system() {
    let matrix = vec![vec![f(1), f(1)], vec![f(1), f(1)]];
    let rhs = vec![f(2), f(3)];

    assert_eq!(
        solve_unique(&matrix, &rhs),
        Err(LinearSolveError::Inconsistent)
    );
}

#[test]
fn detects_underdetermined_system() {
    let matrix = vec![vec![f(1), f(1)], vec![f(2), f(2)]];
    let rhs = vec![f(3), f(6)];

    assert_eq!(
        solve_unique(&matrix, &rhs),
        Err(LinearSolveError::Underdetermined)
    );
}

#[test]
fn rejects_invalid_dimensions() {
    assert_eq!(
        solve_unique(&[], &[]),
        Err(LinearSolveError::InvalidDimensions)
    );

    assert_eq!(
        solve_unique(&[vec![f(1), f(2)]], &[]),
        Err(LinearSolveError::InvalidDimensions)
    );

    assert_eq!(
        solve_unique(&[vec![f(1)], vec![f(1), f(2)]], &[f(1), f(2)]),
        Err(LinearSolveError::InvalidDimensions)
    );
}

#[test]
fn recovers_known_solution_over_gf256() {
    let expected = vec![f(0x53), f(0xA7), f(0xFF)];

    let matrix = vec![
        vec![f(1), f(2), f(3)],
        vec![f(0), f(1), f(4)],
        vec![f(0), f(0), f(1)],
        vec![f(1), f(3), f(7)],
    ];

    let rhs: Vec<_> = matrix
        .iter()
        .map(|row| {
            row.iter()
                .zip(&expected)
                .fold(GF256::ZERO, |sum, (&a, &x)| sum.add(a.mul(x)))
        })
        .collect();

    let recovered = solve_unique(&matrix, &rhs).unwrap();

    assert_eq!(recovered, expected);
    verify_solution(&matrix, &rhs, &recovered);
}
