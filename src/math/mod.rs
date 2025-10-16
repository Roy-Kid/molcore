//! Small linear algebra helpers with optional BLAS-backed implementation.
//!
//! Default: hand-written 3x3 routines (WASM-friendly, zero external deps).
//! Feature `blas`: use ndarray-linalg for determinant and inverse (requires LAPACK backend).

use ndarray::{array, Array1, Array2, ArrayView2};

pub type F = f32;
pub type Vec3 = Array1<F>;
pub type Mat3 = Array2<F>; // shape (3,3)

#[inline]
pub fn norm3(v: &Vec3) -> F {
    // Use ndarray dot for clarity; equivalent to sqrt(v·v)
    v.dot(v).sqrt()
}

#[inline]
pub fn mat3_mul_vec(m: &Mat3, v: &Vec3) -> Vec3 {
    m.dot(v)
}

#[cfg(not(feature = "blas"))]
pub fn det3(m: &Mat3) -> F {
    let m = |r: usize, c: usize| m[[r, c]];
    m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1))
        - m(0, 1) * (m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0))
        + m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0))
}

#[cfg(not(feature = "blas"))]
pub fn inv3(m: &Mat3) -> Option<Mat3> {
    let m = |r: usize, c: usize| m[[r, c]];
    let c00 = m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1);
    let c01 = -(m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0));
    let c02 = m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0);

    let c10 = -(m(0, 1) * m(2, 2) - m(0, 2) * m(2, 1));
    let c11 = m(0, 0) * m(2, 2) - m(0, 2) * m(2, 0);
    let c12 = -(m(0, 0) * m(2, 1) - m(0, 1) * m(2, 0));

    let c20 = m(0, 1) * m(1, 2) - m(0, 2) * m(1, 1);
    let c21 = -(m(0, 0) * m(1, 2) - m(0, 2) * m(1, 0));
    let c22 = m(0, 0) * m(1, 1) - m(0, 1) * m(1, 0);

    let det = m(0, 0) * c00 + m(0, 1) * c01 + m(0, 2) * c02;
    let eps: F = 1e-8;
    if det.abs() <= eps { return None; }
    let inv_det = 1.0 / det;
    Some(array![
        [c00 * inv_det, c10 * inv_det, c20 * inv_det],
        [c01 * inv_det, c11 * inv_det, c21 * inv_det],
        [c02 * inv_det, c12 * inv_det, c22 * inv_det],
    ])
}

#[cfg(feature = "blas")]
pub fn det3(m: &Mat3) -> F {
    use ndarray_linalg::Determinant;
    // Safe unwrap if upstream guarantees non-singularity for typical boxes; otherwise map to 0.0
    m.clone().det().unwrap_or(F::NAN)
}

#[cfg(feature = "blas")]
pub fn inv3(m: &Mat3) -> Option<Mat3> {
    use ndarray_linalg::Inverse;
    ndarray_linalg::Inverse::inv(m.clone()).ok()
}

/// General matrix multiplication: C = A x B
/// - A: (m x k) view
/// - B: (k x n) owned or borrowable
/// - Returns C: (m x n) owned
///
/// Path selection:
/// - With feature `blas` enabled: use ndarray's optimized dot (BLAS/LAPACK backend if linked)
/// - Else if feature `rayon` enabled: parallel row-by-row multiply
/// - Else: serial multiply
pub fn matmul(a: ArrayView2<F>, b: &Array2<F>) -> Array2<F> {
    let (m, k_a) = a.dim();
    let (k_b, n) = b.dim();
    assert_eq!(k_a, k_b, "matmul: inner dims must match: got {} vs {}", k_a, k_b);

    #[cfg(feature = "blas")]
    {
        // ndarray's .dot is backed by optimized kernels; with proper backend it will leverage BLAS
        return a.dot(b);
    }

    #[cfg(all(not(feature = "blas"), feature = "rayon"))]
    {
        use rayon::prelude::*;
        let mut c = Array2::<F>::zeros((m, n));
        c.axis_iter_mut(ndarray::Axis(0))
            .into_par_iter()
            .enumerate()
            .for_each(|(i, mut row)| {
                for j in 0..n {
                    let mut sum: F = 0.0;
                    for k in 0..k_a {
                        sum += a[[i, k]] * b[[k, j]];
                    }
                    row[j] = sum;
                }
            });
        return c;
    }

    #[cfg(all(not(feature = "blas"), not(feature = "rayon")))]
    {
        let mut c = Array2::<F>::zeros((m, n));
        for i in 0..m {
            for j in 0..n {
                let mut sum: F = 0.0;
                for k in 0..k_a {
                    sum += a[[i, k]] * b[[k, j]];
                }
                c[[i, j]] = sum;
            }
        }
        return c;
    }
}