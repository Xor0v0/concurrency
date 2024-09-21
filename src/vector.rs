use std::ops::{Add, AddAssign, Deref, Mul};

use anyhow::{anyhow, Result};

/// Vector: a list of numbers that represent a row or column in a matrix
pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// dot_product: calculate the dot product of two vectors
pub fn dot_product<T>(row: Vector<T>, col: Vector<T>) -> Result<T>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T> + AddAssign,
{
    if row.len() != col.len() {
        return Err(anyhow!("Vector dot product: dimension mismatch"));
    }

    let mut result = T::default();
    for i in 0..row.len() {
        result += row[i] * col[i];
    }
    Ok(result)
}
