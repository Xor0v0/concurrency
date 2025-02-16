use anyhow::{anyhow, Result};
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use crate::vector::{dot_product, Vector};

const NUM_PRODUCERS: usize = 4;

pub struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T> Matrix<T> {
    fn new(rows: usize, cols: usize, data: impl Into<Vec<T>>) -> Self {
        Matrix {
            rows,
            cols,
            data: data.into(),
        }
    }
}

/// display the matrix: 3x2 as {1 2 3, 4 5 6}, 2x3 as {1 2, 3 4, 5 6}
impl<T: fmt::Display> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.rows {
            for j in 0..self.cols {
                write!(f, "{}", self.data[i * self.cols + j])?;
                if j < self.cols - 1 {
                    write!(f, " ")?;
                }
            }
            if i < self.rows - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T: fmt::Display> fmt::Debug for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.rows, self.cols, self)
    }
}

/// Info: vector info sent to a worker
struct Info<T> {
    row: Vector<T>,
    col: Vector<T>,
    idx: usize,
}

/// MsgInput: input message to the worker
struct MsgInput<T> {
    info: Info<T>,
    tx: oneshot::Sender<MsgOutput<T>>,
}

/// MsgOutput: output message from the worker
struct MsgOutput<T> {
    value: T,
    idx: usize,
}

#[allow(dead_code)]
pub fn sequential_multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T> + AddAssign,
{
    if a.cols != b.rows {
        return Err(anyhow!("Matrix multiplication: dimension mismatch"));
    }

    let mut data = vec![T::default(); a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            for k in 0..a.cols {
                data[i * b.cols + j] += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
        }
    }
    Ok(Matrix::new(a.rows, b.cols, data))
}

pub fn parallel_multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T> + AddAssign + Send + 'static,
{
    if a.cols != b.rows {
        return Err(anyhow!("Matrix multiplication: dimension mismatch"));
    }

    let senders = (0..NUM_PRODUCERS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<MsgInput<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.info.row, msg.info.col)?;
                    if let Err(e) = msg.tx.send(MsgOutput {
                        value,
                        idx: msg.info.idx,
                    }) {
                        eprintln!("Worker send error: {:?}", e);
                    };
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    let matrix_len = a.rows * b.cols;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    for i in 0..a.rows {
        for j in 0..b.cols {
            let row = Vector::new(&a.data[i * a.cols..(i + 1) * a.cols]);
            let col = Vector::new(&b.data[j..b.cols * (b.rows - 1) + j + 1]);
            let idx = i * b.cols + j;
            let info = Info { row, col, idx };
            let (tx, rx) = oneshot::channel();
            senders[idx % NUM_PRODUCERS]
                .send(MsgInput { info, tx })
                .map_err(|e| anyhow!("Worker send error: {:?}", e))?;
            receivers.push(rx);
        }
    }

    // TODO: handle all rx with async framework like tokio::join_all(receivers).await?
    for rx in receivers {
        let output = rx.recv()?;
        data[output.idx] = output.value;
    }

    Ok(Matrix::new(a.rows, b.cols, data))
}

impl<T> Mul<Matrix<T>> for Matrix<T>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T> + AddAssign + Send + 'static,
{
    type Output = Result<Matrix<T>>;

    fn mul(self, rhs: Matrix<T>) -> Self::Output {
        parallel_multiply(&self, &rhs)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use super::*;

    #[test]
    fn test_sequential_multiply() -> Result<()> {
        let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(3, 2, vec![1, 2, 3, 4, 5, 6]);
        let c = sequential_multiply(&a, &b)?;
        assert_eq!(format!("{}", c), "{22 28, 49 64}");
        Ok(())
    }

    #[test]
    fn test_demention_mismatch() {
        let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        assert!((a * b).is_err());
    }
}
