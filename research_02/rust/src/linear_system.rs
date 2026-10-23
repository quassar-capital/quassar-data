//! Exact linear-system solving over GF(256).
//!
//! Gauss-Jordan elimination with column-wise pivot selection.

use crate::GF256;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinearSolveError {
    InvalidDimensions,
    Inconsistent,
    Underdetermined,
}

pub fn solve_unique(
    coefficients: &[Vec<GF256>],
    rhs: &[GF256],
) -> Result<Vec<GF256>, LinearSolveError> {
    let rows = coefficients.len();

    if rows == 0 || rows != rhs.len() {
        return Err(LinearSolveError::InvalidDimensions);
    }

    let columns = coefficients[0].len();

    if columns == 0 || coefficients.iter().any(|row| row.len() != columns) {
        return Err(LinearSolveError::InvalidDimensions);
    }

    let mut augmented: Vec<Vec<GF256>> = coefficients
        .iter()
        .zip(rhs.iter())
        .map(|(row, &value)| {
            let mut extended = row.clone();
            extended.push(value);
            extended
        })
        .collect();

    let mut pivot_row = 0usize;
    let mut pivot_columns = Vec::new();

    for column in 0..columns {
        if pivot_row == rows {
            break;
        }

        let Some(found) = (pivot_row..rows).find(|&r| augmented[r][column] != GF256::ZERO) else {
            continue;
        };

        augmented.swap(pivot_row, found);

        let inverse = augmented[pivot_row][column]
            .inverse()
            .expect("nonzero pivot must be invertible");

        for value in augmented[pivot_row].iter_mut().skip(column) {
            *value = value.mul(inverse);
        }

        // Snapshot eliminates aliasing between the pivot and target rows.
        let pivot_values = augmented[pivot_row].clone();

        for (row_index, row_values) in augmented.iter_mut().enumerate() {
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

    for row in &augmented {
        if row[..columns].iter().all(|&x| x == GF256::ZERO) && row[columns] != GF256::ZERO {
            return Err(LinearSolveError::Inconsistent);
        }
    }

    if pivot_columns.len() != columns {
        return Err(LinearSolveError::Underdetermined);
    }

    let mut solution = vec![GF256::ZERO; columns];

    for (row_index, &column) in pivot_columns.iter().enumerate() {
        solution[column] = augmented[row_index][columns];
    }

    Ok(solution)
}
