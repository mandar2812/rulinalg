//! The norm module
//!
//! This module contains implementations of various linear algebra norms.
//! The implementations are contained within the `VectorNorm` and 
//! `MatrixNorm` traits. This module also contains `VectorMetric` and
//! `MatrixMetric` traits which are used to compute the metric distance.
//!
//! These traits can be used directly by importing implementors from
//! this module. In most cases it will be easier to use the `norm` and
//! `metric` functions which exist for both vectors and matrices. These
//! functions take generic arguments for the norm to be used.
//!
//! In general you should use the least generic norm that fits your purpose.
//! For example you would choose to use a `Euclidean` norm instead of an
//! `Lp(2.0)` norm - despite them being mathematically equivalent. 
//!
//! # Defining your own norm
//!
//! Note that these traits enforce no requirements on the norm. It is up
//! to the user to ensure that they define a norm correctly.
//!
//! To define your own norm you need to implement the `MatrixNorm`
//! and/or the `VectorNorm` on your own struct. When you have defined
//! a norm you get the _induced metric_ for free.

use matrix::BaseMatrix;
use vector::Vector;
use utils;

use std::ops::Sub;
use libnum::Float;

/// Trait for vector norms
pub trait VectorNorm<T> {
    /// Computes the vector norm.
    fn norm(&self, v: &Vector<T>) -> T;
}

/// Trait for vector metrics.
pub trait VectorMetric<T> {
    /// Computes the metric distance between two vectors.
    fn metric(&self, v1: &Vector<T>, v2: &Vector<T>) -> T;
}

/// Trait for matrix norms.
pub trait MatrixNorm<T, M: BaseMatrix<T>> {
    /// Computes the matrix norm.
    fn norm(&self, m: &M) -> T;
}

/// Trait for matrix metrics.
pub trait MatrixMetric<'a, 'b, T, M1: 'a + BaseMatrix<T>, M2: 'b + BaseMatrix<T>> {
    /// Computes the metric distance between two matrices.
    fn metric(&self, m1: &'a M1, m2: &'b M2) -> T;
}

/// The induced vector metric
///
/// Given a norm `N`, the induced vector metric `M` computes
/// the metric distance, `d`, between two vectors `v1` and `v2`
/// as follows:
///
/// `d = M(v1, v2) = N(v1 - v2)`
impl<U, T> VectorMetric<T> for U
    where U: VectorNorm<T>, T: Copy + Sub<T, Output=T> {
    fn metric(&self, v1: &Vector<T>, v2: &Vector<T>) -> T {
        self.norm(&(v1 - v2))
    }
}

/// The induced matrix metric
///
/// Given a norm `N`, the induced matrix metric `M` computes
/// the metric distance, `d`, between two matrices `m1` and `m2`
/// as follows:
///
/// `d = M(m1, m2) = N(m1 - m2)`
impl<'a, 'b, U, T, M1, M2> MatrixMetric<'a, 'b, T, M1, M2> for U
    where U: MatrixNorm<T, M1>,
    M1: 'a + BaseMatrix<T>,
    M2: 'b + BaseMatrix<T>,
    &'a M1: Sub<&'b M2, Output=M1> {

    fn metric(&self, m1: &'a M1, m2: &'b M2) -> T {
        self.norm(&(m1 - m2))
    }
}

/// The Euclidean norm
///
/// The Euclidean norm computes the square-root
/// of the sum of squares.
///
/// `||v|| = SQRT(SUM(v_i * v_i))`
#[derive(Debug)]
pub struct Euclidean;

impl<T: Float> VectorNorm<T> for Euclidean {
    fn norm(&self, v: &Vector<T>) -> T {
        utils::dot(v.data(), v.data()).sqrt()
    }
}

impl<T: Float, M: BaseMatrix<T>> MatrixNorm<T, M> for Euclidean {
    fn norm(&self, m: &M) -> T {
        let mut s = T::zero();

        for row in m.iter_rows() {
            s = s + utils::dot(row.raw_slice(), row.raw_slice());
        }

        s.sqrt()
    }
}

/// The Lp norm
///
/// The Lp norm computes the `p`th root
/// of the sum of elements to the `p`th power.
///
/// The Lp norm requires `p` to be greater than
/// or equal `1`.
///
/// # p = infinity
///
/// In the special case where `p` is positive infinity,
/// the Lp norm becomes a supremum over the absolute values.
#[derive(Debug)]
pub struct Lp<T: Float>(T);

impl<T: Float> VectorNorm<T> for Lp<T> {
    fn norm(&self, v: &Vector<T>) -> T {
        if self.0 < T::one() {
            panic!("p value in Lp norm must >= 1")
        } else if self.0.is_infinite() {
            // Compute supremum
            let mut abs_sup = T::zero();
            for d in v {
                if d.abs() > abs_sup {
                    abs_sup = *d;
                }
            }
            abs_sup
        } else {
            // Compute standard lp norm
            let mut s = T::zero();
            for x in v {
                s = s + x.abs().powf(self.0);
            }
            s.powf(self.0.recip())
        }
    }
}

impl<T: Float, M: BaseMatrix<T>> MatrixNorm<T, M> for Lp<T> {
    fn norm(&self, m: &M) -> T {
        if self.0 < T::one() {
            panic!("p value in Lp norm must >= 1")
        } else if self.0.is_infinite() {
            // Compute supremum
            let mut abs_sup = T::zero();
            for d in m.iter() {
                if d.abs() > abs_sup {
                    abs_sup = *d;
                }
            }
            abs_sup
        } else {
            // Compute standard lp norm
            let mut s = T::zero();
            for x in m.iter() {
                s = s + x.abs().powf(self.0);
            }
            s.powf(self.0.recip())
        }
    }
}

#[cfg(test)]
mod tests {
    use libnum::Float;
    use super::*;
    use vector::Vector;
    use matrix::{Matrix, MatrixSlice};

    #[test]
    fn test_euclidean_vector_norm() {
        let v = Vector::new(vec![3.0, 4.0]);
        assert!((VectorNorm::norm(&Euclidean, &v) - 5.0) < 1e-30);
    }

    #[test]
    fn test_euclidean_matrix_norm() {
        let m = matrix![3.0, 4.0;
                        1.0, 3.0];
        assert!((MatrixNorm::norm(&Euclidean, &m) - 35.0.sqrt()) < 1e-30);

        let slice = MatrixSlice::from_matrix(&m, [0,0], 1, 2);
        assert!((MatrixNorm::norm(&Euclidean, &slice) - 5.0) < 1e-30);
    }

    #[test]
    fn test_euclidean_vector_metric() {
        let v = Vector::new(vec![3.0, 4.0]);
        assert!((VectorMetric::metric(&Euclidean, &v, &v)) < 1e-30);

        let v1 = Vector::new(vec![0.0, 0.0]);
        assert!((VectorMetric::metric(&Euclidean, &v, &v1) - 5.0) < 1e-30);

        let v2 = Vector::new(vec![4.0, 3.0]);
        assert!((VectorMetric::metric(&Euclidean, &v, &v2) - 2.0.sqrt()) < 1e-30);
    }

    #[test]
    #[should_panic]
    fn test_euclidean_vector_metric_bad_dim() {
        let v = Vector::new(vec![3.0, 4.0]);
        let v2 = Vector::new(vec![1.0, 2.0, 3.0]);

        VectorMetric::metric(&Euclidean, &v, &v2);
    }

    #[test]
    fn test_euclidean_matrix_metric() {
        let m = matrix![3.0, 4.0;
                        1.0, 3.0];
        assert!((MatrixMetric::metric(&Euclidean, &m, &m)) < 1e-30);

        let m1 = Matrix::zeros(2, 2);
        assert!((MatrixMetric::metric(&Euclidean, &m, &m1) - 35.0.sqrt()) < 1e-30);

        let m2 = matrix![2.0, 3.0;
                         2.0, 4.0];
        assert!((MatrixMetric::metric(&Euclidean, &m, &m2) - 2.0) < 1e-30);
    }

    #[test]
    #[should_panic]
    fn test_euclidean_matrix_metric_bad_dim() {
        let m = matrix![3.0, 4.0];
        let m2 = matrix![1.0, 2.0, 3.0];

        MatrixMetric::metric(&Euclidean, &m, &m2);
    }
}