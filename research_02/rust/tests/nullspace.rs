use quassar_algebraic_coding::{
    GF256,
    nullspace::{NullspaceError, nullspace_basis},
};
fn f(value: u8) -> GF256 {
    GF256(value)
}

fn verify_kernel(matrix: &[Vec<GF256>], basis: &[Vec<GF256>], columns: usize) {
    for vector in basis {
        assert_eq!(vector.len(), columns);
        assert!(vector.iter().any(|&x| x != GF256::ZERO));

        for row in matrix {
            let result = row
                .iter()
                .zip(vector)
                .fold(GF256::ZERO, |sum, (&a, &b)| sum.add(a.mul(b)));

            assert_eq!(result, GF256::ZERO);
        }
    }
}

#[test]
fn identity_matrix_has_trivial_kernel() {
    let matrix = vec![
        vec![f(1), f(0), f(0)],
        vec![f(0), f(1), f(0)],
        vec![f(0), f(0), f(1)],
    ];

    let basis = nullspace_basis(&matrix, 3).unwrap();

    assert!(basis.is_empty());
}

#[test]
fn zero_matrix_has_full_dimensional_kernel() {
    let matrix = vec![vec![f(0), f(0), f(0)], vec![f(0), f(0), f(0)]];

    let basis = nullspace_basis(&matrix, 3).unwrap();

    assert_eq!(basis.len(), 3);
    verify_kernel(&matrix, &basis, 3);
}

#[test]
fn dependent_rows_produce_nontrivial_kernel() {
    let matrix = vec![vec![f(1), f(1), f(0)], vec![f(1), f(1), f(0)]];

    let basis = nullspace_basis(&matrix, 3).unwrap();

    assert_eq!(basis.len(), 2);
    verify_kernel(&matrix, &basis, 3);
}

#[test]
fn rectangular_matrix_respects_rank_nullity() {
    let matrix = vec![vec![f(1), f(0), f(1), f(0)], vec![f(0), f(1), f(0), f(1)]];

    let basis = nullspace_basis(&matrix, 4).unwrap();

    assert_eq!(basis.len(), 2);
    assert_eq!(2 + basis.len(), 4);
    verify_kernel(&matrix, &basis, 4);
}

#[test]
fn pivoting_over_gf256_is_exact() {
    let matrix = vec![vec![f(0), f(0x53), f(1)], vec![f(0xA7), f(1), f(0)]];

    let basis = nullspace_basis(&matrix, 3).unwrap();

    assert_eq!(basis.len(), 1);
    verify_kernel(&matrix, &basis, 3);
}

#[test]
fn empty_constraint_system_has_full_kernel() {
    let matrix: Vec<Vec<GF256>> = Vec::new();

    let basis = nullspace_basis(&matrix, 4).unwrap();

    assert_eq!(basis.len(), 4);
    verify_kernel(&matrix, &basis, 4);
}

#[test]
fn rejects_invalid_dimensions() {
    assert_eq!(
        nullspace_basis(&[vec![f(1), f(2)]], 3),
        Err(NullspaceError::InvalidDimensions)
    );

    assert_eq!(
        nullspace_basis(&[], 0),
        Err(NullspaceError::InvalidDimensions)
    );
}

#[test]
fn kernel_basis_vectors_are_linearly_independent() {
    let matrix = vec![vec![f(1), f(1), f(1), f(1)]];

    let basis = nullspace_basis(&matrix, 4).unwrap();

    assert_eq!(basis.len(), 3);
    verify_kernel(&matrix, &basis, 4);

    // Every constructed basis vector has a distinct free-variable
    // coordinate equal to one and zero in the other free coordinates.
    for i in 0..basis.len() {
        assert_eq!(basis[i][i + 1], GF256::ONE);

        for j in 0..basis.len() {
            if i != j {
                assert_eq!(basis[i][j + 1], GF256::ZERO);
            }
        }
    }
}
