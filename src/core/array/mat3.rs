use super::dtype::{DType, HasDType};
use super::trait_array::Array;
use super::vec3::Vec3;
use core::ops::Mul;

/// A 3x3 matrix with elements of type `T` (row-major storage)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3<T> {
    /// Row-major data storage: data[row][col]
    pub data: [[T; 3]; 3],
}

impl<T> Mat3<T> {
    /// Creates a new 3x3 matrix from row-major elements
    #[inline]
    pub fn new(
        m00: T, m01: T, m02: T,
        m10: T, m11: T, m12: T,
        m20: T, m21: T, m22: T,
    ) -> Self {
        Mat3 {
            data: [
                [m00, m01, m02],
                [m10, m11, m12],
                [m20, m21, m22],
            ],
        }
    }

    /// Construct from rows
    #[inline]
    pub fn from_rows(r0: [T; 3], r1: [T; 3], r2: [T; 3]) -> Self {
        Mat3 { data: [r0, r1, r2] }
    }

    /// Returns a reference to the underlying row-major array
    #[inline]
    pub fn as_array(&self) -> &[[T; 3]; 3] { &self.data }
}

impl<T: HasDType + Send + Sync> Array for Mat3<T> {
    #[inline]
    fn dtype(&self) -> DType { T::dtype() }

    #[inline]
    fn shape(&self) -> &[usize] { &[3, 3] }
}

impl Mat3<f32> {
    /// Determinant of the matrix
    #[inline]
    pub fn det(&self) -> f32 {
        let m = &self.data;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// Returns the inverse of the matrix if it exists.
    ///
    /// Uses a small epsilon to guard against near-singular matrices.
    #[inline]
    pub fn inv(&self) -> Option<Self> {
        let m = &self.data;
        let c00 =  m[1][1] * m[2][2] - m[1][2] * m[2][1];
        let c01 = -(m[1][0] * m[2][2] - m[1][2] * m[2][0]);
        let c02 =  m[1][0] * m[2][1] - m[1][1] * m[2][0];

        let c10 = -(m[0][1] * m[2][2] - m[0][2] * m[2][1]);
        let c11 =  m[0][0] * m[2][2] - m[0][2] * m[2][0];
        let c12 = -(m[0][0] * m[2][1] - m[0][1] * m[2][0]);

        let c20 =  m[0][1] * m[1][2] - m[0][2] * m[1][1];
        let c21 = -(m[0][0] * m[1][2] - m[0][2] * m[1][0]);
        let c22 =  m[0][0] * m[1][1] - m[0][1] * m[1][0];

        let det = m[0][0] * c00 + m[0][1] * c01 + m[0][2] * c02;
        let eps = 1e-8_f32;
        if det.abs() <= eps {
            return None;
        }
        let inv_det = 1.0 / det;

        // adjugate transposes the cofactor matrix
        Some(Mat3::from_rows(
            [c00 * inv_det, c10 * inv_det, c20 * inv_det],
            [c01 * inv_det, c11 * inv_det, c21 * inv_det],
            [c02 * inv_det, c12 * inv_det, c22 * inv_det],
        ))
    }
}

impl Mul<Vec3<f32>> for Mat3<f32> {
    type Output = Vec3<f32>;
    #[inline]
    fn mul(self, v: Vec3<f32>) -> Self::Output {
        let a = self.data;
        Vec3::new(
            a[0][0] * v.x + a[0][1] * v.y + a[0][2] * v.z,
            a[1][0] * v.x + a[1][1] * v.y + a[1][2] * v.z,
            a[2][0] * v.x + a[2][1] * v.y + a[2][2] * v.z,
        )
    }
}

impl<'a> Mul<Vec3<f32>> for &'a Mat3<f32> {
    type Output = Vec3<f32>;
    #[inline]
    fn mul(self, v: Vec3<f32>) -> Self::Output {
        let a = self.data;
        Vec3::new(
            a[0][0] * v.x + a[0][1] * v.y + a[0][2] * v.z,
            a[1][0] * v.x + a[1][1] * v.y + a[1][2] * v.z,
            a[2][0] * v.x + a[2][1] * v.y + a[2][2] * v.z,
        )
    }
}

impl Mat3<f64> {
    /// Determinant of the matrix
    #[inline]
    pub fn det(&self) -> f64 {
        let m = &self.data;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// Returns the inverse of the matrix if it exists.
    #[inline]
    pub fn inv(&self) -> Option<Self> {
        let m = &self.data;
        let c00 =  m[1][1] * m[2][2] - m[1][2] * m[2][1];
        let c01 = -(m[1][0] * m[2][2] - m[1][2] * m[2][0]);
        let c02 =  m[1][0] * m[2][1] - m[1][1] * m[2][0];

        let c10 = -(m[0][1] * m[2][2] - m[0][2] * m[2][1]);
        let c11 =  m[0][0] * m[2][2] - m[0][2] * m[2][0];
        let c12 = -(m[0][0] * m[2][1] - m[0][1] * m[2][0]);

        let c20 =  m[0][1] * m[1][2] - m[0][2] * m[1][1];
        let c21 = -(m[0][0] * m[1][2] - m[0][2] * m[1][0]);
        let c22 =  m[0][0] * m[1][1] - m[0][1] * m[1][0];

        let det = m[0][0] * c00 + m[0][1] * c01 + m[0][2] * c02;
        let eps = 1e-12_f64;
        if det.abs() <= eps {
            return None;
        }
        let inv_det = 1.0 / det;

        Some(Mat3::from_rows(
            [c00 * inv_det, c10 * inv_det, c20 * inv_det],
            [c01 * inv_det, c11 * inv_det, c21 * inv_det],
            [c02 * inv_det, c12 * inv_det, c22 * inv_det],
        ))
    }
}

impl Mul<Vec3<f64>> for Mat3<f64> {
    type Output = Vec3<f64>;
    #[inline]
    fn mul(self, v: Vec3<f64>) -> Self::Output {
        let a = self.data;
        Vec3::new(
            a[0][0] * v.x + a[0][1] * v.y + a[0][2] * v.z,
            a[1][0] * v.x + a[1][1] * v.y + a[1][2] * v.z,
            a[2][0] * v.x + a[2][1] * v.y + a[2][2] * v.z,
        )
    }
}

impl<'a> Mul<Vec3<f64>> for &'a Mat3<f64> {
    type Output = Vec3<f64>;
    #[inline]
    fn mul(self, v: Vec3<f64>) -> Self::Output {
        let a = self.data;
        Vec3::new(
            a[0][0] * v.x + a[0][1] * v.y + a[0][2] * v.z,
            a[1][0] * v.x + a[1][1] * v.y + a[1][2] * v.z,
            a[2][0] * v.x + a[2][1] * v.y + a[2][2] * v.z,
        )
    }
}
