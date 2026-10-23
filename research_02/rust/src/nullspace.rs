//! Exact nullspace computation over GF(256).
//!
//! Constructs a kernel basis using Gauss-Jordan elimination.

use crate::GF256;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NullspaceError {
    InvalidDimensions,
}

pub fn nullspace_basis(
    matrix: &[Vec<GF256>],
    columns: usize,
) -> Result<Vec<Vec<GF256>>, NullspaceError> {
    if columns == 0 || matrix.iter().any(|row| row.len() != columns) {
        return Err(NullspaceError::InvalidDimensions);
    }

    let mut reduced = matrix.to_vec();
    let rows = reduced.len();
    let mut pivot_columns = Vec::new();
    let mut pivot_row = 0usize;

    for column in 0..columns {
        if pivot_row == rows {
            break;
        }

        let Some(found) = (pivot_row..rows).find(|&r| reduced[r][column] != GF256::ZERO) else {
            continue;
        };

        reduced.swap(pivot_row, found);

        let inverse = reduced[pivot_row][column]
            .inverse()
            .expect("nonzero pivot must be invertible");

        for value in reduced[pivot_row].iter_mut().skip(column) {
            *value = value.mul(inverse);
        }

        // Read the normalized pivot from an immutable snapshot.
        let pivot_values = reduced[pivot_row].clone();

        for (row_index, row_values) in reduced.iter_mut().enumerate() {
            if row_index == pivot_row {
                continue;
            }

            let factor = row_values[column];

            if factor == GF256::ZERO {
                continue;
            }

            for (value, &pivot_value) in row_values.iter_mut().zip(pivot_values.iter()).skip(column)
            {
                *value = value.add(factor.mul(pivot_value));
            }
        }

        pivot_columns.push(column);
        pivot_row += 1;
    }

    let mut basis = Vec::new();

    for free_column in 0..columns {
        if pivot_columns.contains(&free_column) {
            continue;
        }

        let mut vector = vec![GF256::ZERO; columns];
        vector[free_column] = GF256::ONE;

        for (row_index, &pivot_column) in pivot_columns.iter().enumerate() {
            vector[pivot_column] = reduced[row_index][free_column];
        }

        basis.push(vector);
    }

    Ok(basis)
}
