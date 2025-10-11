use super::dtype::{DType, HasDType};
use super::base::Array;
use super::vec3::{Vec3, Vec3View};
use core::marker::PhantomData;
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

    /// Create a raw read-only view over the contiguous data.
    ///
    /// Safety: The returned view must not outlive `self` and no mutable aliasing must occur
    /// while the view is in use.
    #[inline]
    pub unsafe fn view(&self) -> Mat3View<T> {
        let ptr: *const T = &self.data[0][0];
        Mat3View { ptr, row_stride: 3, col_stride: 1, _marker: PhantomData }
    }
}

impl<T: Copy> Mat3<T> {
    /// Returns the i-th row as a Vec3 (0-based index)
    #[inline]
    pub fn row(&self, i: usize) -> Vec3<T> {
        assert!(i < 3, "row index must be 0..2");
        let r = self.data[i];
        Vec3::new(r[0], r[1], r[2])
    }

    /// Returns the i-th column as a Vec3 (0-based index)
    #[inline]
    pub fn col(&self, j: usize) -> Vec3<T> {
        assert!(j < 3, "col index must be 0..2");
        let m = &self.data;
        Vec3::new(m[0][j], m[1][j], m[2][j])
    }
}

impl<T: HasDType + Send + Sync + 'static> Array for Mat3<T> {
    #[inline]
    fn dtype(&self) -> DType { T::dtype() }

    #[inline]
    fn shape(&self) -> &[usize] { &[3, 3] }

    #[inline]
    fn as_any(&self) -> &dyn core::any::Any { self }
}

/// A read-only 3x3 matrix view with configurable element strides (in elements).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3View<T> {
    pub(super) ptr: *const T,
    pub(super) row_stride: usize,
    pub(super) col_stride: usize,
    pub(super) _marker: PhantomData<*const T>,
}

impl<T> Mat3View<T> {
    /// Create a view from a base pointer and strides in elements.
    ///
    /// Safety: caller must guarantee the pointer remains valid for reads of all 9 positions
    /// addressed by row/col and that aliasing rules are respected.
    #[inline]
    pub unsafe fn from_ptr(ptr: *const T, row_stride: usize, col_stride: usize) -> Self {
        Self { ptr, row_stride, col_stride, _marker: PhantomData }
    }

    #[inline]
    pub fn get(&self, r: usize, c: usize) -> &T {
        assert!(r < 3 && c < 3, "index out of bounds for Mat3View");
        let off = r * self.row_stride + c * self.col_stride;
        unsafe { &*self.ptr.add(off) }
    }

    /// i-th row as a Vec3View
    #[inline]
    pub fn row(&self, i: usize) -> Vec3View<T> {
        assert!(i < 3);
        let base = unsafe { self.ptr.add(i * self.row_stride) };
        unsafe { Vec3View::from_ptr(base, self.col_stride) }
    }

    /// j-th column as a Vec3View
    #[inline]
    pub fn col(&self, j: usize) -> Vec3View<T> {
        assert!(j < 3);
        let base = unsafe { self.ptr.add(j * self.col_stride) };
        unsafe { Vec3View::from_ptr(base, self.row_stride) }
    }
}

impl<T: HasDType + Send + Sync + 'static> Array for Mat3View<T> {
    #[inline]
    fn dtype(&self) -> DType { T::dtype() }
    #[inline]
    fn shape(&self) -> &[usize] {
        const SH: [usize; 2] = [3, 3];
        &SH
    }
    #[inline]
    fn as_any(&self) -> &dyn core::any::Any { self }
}

// Safety: read-only raw pointer based view; Send/Sync when T: Sync
unsafe impl<T: Sync> Send for Mat3View<T> {}
unsafe impl<T: Sync> Sync for Mat3View<T> {}

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

impl Mat3View<f32> {
    #[inline]
    pub fn det(&self) -> f32 {
        let m00 = *self.get(0,0); let m01 = *self.get(0,1); let m02 = *self.get(0,2);
        let m10 = *self.get(1,0); let m11 = *self.get(1,1); let m12 = *self.get(1,2);
        let m20 = *self.get(2,0); let m21 = *self.get(2,1); let m22 = *self.get(2,2);
        m00 * (m11 * m22 - m12 * m21)
        - m01 * (m10 * m22 - m12 * m20)
        + m02 * (m10 * m21 - m11 * m20)
    }

    #[inline]
    pub fn inv(&self) -> Option<Mat3<f32>> {
        let m = |r,c| *self.get(r,c);
        let c00 =  m(1,1) * m(2,2) - m(1,2) * m(2,1);
        let c01 = -(m(1,0) * m(2,2) - m(1,2) * m(2,0));
        let c02 =  m(1,0) * m(2,1) - m(1,1) * m(2,0);

        let c10 = -(m(0,1) * m(2,2) - m(0,2) * m(2,1));
        let c11 =  m(0,0) * m(2,2) - m(0,2) * m(2,0);
        let c12 = -(m(0,0) * m(2,1) - m(0,1) * m(2,0));

        let c20 =  m(0,1) * m(1,2) - m(0,2) * m(1,1);
        let c21 = -(m(0,0) * m(1,2) - m(0,2) * m(1,0));
        let c22 =  m(0,0) * m(1,1) - m(0,1) * m(1,0);

        let det = m(0,0) * c00 + m(0,1) * c01 + m(0,2) * c02;
        let eps = 1e-8_f32;
        if det.abs() <= eps { return None; }
        let inv_det = 1.0 / det;
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

impl Mul<Vec3<f32>> for Mat3View<f32> {
    type Output = Vec3<f32>;
    #[inline]
    fn mul(self, v: Vec3<f32>) -> Self::Output {
        Vec3::new(
            *self.get(0,0) * v.x + *self.get(0,1) * v.y + *self.get(0,2) * v.z,
            *self.get(1,0) * v.x + *self.get(1,1) * v.y + *self.get(1,2) * v.z,
            *self.get(2,0) * v.x + *self.get(2,1) * v.y + *self.get(2,2) * v.z,
        )
    }
}

impl<'a> Mul<Vec3<f32>> for &'a Mat3View<f32> {
    type Output = Vec3<f32>;
    #[inline]
    fn mul(self, v: Vec3<f32>) -> Self::Output {
        Vec3::new(
            *self.get(0,0) * v.x + *self.get(0,1) * v.y + *self.get(0,2) * v.z,
            *self.get(1,0) * v.x + *self.get(1,1) * v.y + *self.get(1,2) * v.z,
            *self.get(2,0) * v.x + *self.get(2,1) * v.y + *self.get(2,2) * v.z,
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

impl Mat3View<f64> {
    #[inline]
    pub fn det(&self) -> f64 {
        let m00 = *self.get(0,0); let m01 = *self.get(0,1); let m02 = *self.get(0,2);
        let m10 = *self.get(1,0); let m11 = *self.get(1,1); let m12 = *self.get(1,2);
        let m20 = *self.get(2,0); let m21 = *self.get(2,1); let m22 = *self.get(2,2);
        m00 * (m11 * m22 - m12 * m21)
        - m01 * (m10 * m22 - m12 * m20)
        + m02 * (m10 * m21 - m11 * m20)
    }

    #[inline]
    pub fn inv(&self) -> Option<Mat3<f64>> {
        let m = |r,c| *self.get(r,c);
        let c00 =  m(1,1) * m(2,2) - m(1,2) * m(2,1);
        let c01 = -(m(1,0) * m(2,2) - m(1,2) * m(2,0));
        let c02 =  m(1,0) * m(2,1) - m(1,1) * m(2,0);

        let c10 = -(m(0,1) * m(2,2) - m(0,2) * m(2,1));
        let c11 =  m(0,0) * m(2,2) - m(0,2) * m(2,0);
        let c12 = -(m(0,0) * m(2,1) - m(0,1) * m(2,0));

        let c20 =  m(0,1) * m(1,2) - m(0,2) * m(1,1);
        let c21 = -(m(0,0) * m(1,2) - m(0,2) * m(1,0));
        let c22 =  m(0,0) * m(1,1) - m(0,1) * m(1,0);

        let det = m(0,0) * c00 + m(0,1) * c01 + m(0,2) * c02;
        let eps = 1e-12_f64;
        if det.abs() <= eps { return None; }
        let inv_det = 1.0 / det;
        Some(Mat3::from_rows(
            [c00 * inv_det, c10 * inv_det, c20 * inv_det],
            [c01 * inv_det, c11 * inv_det, c21 * inv_det],
            [c02 * inv_det, c12 * inv_det, c22 * inv_det],
        ))
    }
}

impl Mul<Vec3<f64>> for Mat3View<f64> {
    type Output = Vec3<f64>;
    #[inline]
    fn mul(self, v: Vec3<f64>) -> Self::Output {
        Vec3::new(
            *self.get(0,0) * v.x + *self.get(0,1) * v.y + *self.get(0,2) * v.z,
            *self.get(1,0) * v.x + *self.get(1,1) * v.y + *self.get(1,2) * v.z,
            *self.get(2,0) * v.x + *self.get(2,1) * v.y + *self.get(2,2) * v.z,
        )
    }
}

impl<'a> Mul<Vec3<f64>> for &'a Mat3View<f64> {
    type Output = Vec3<f64>;
    #[inline]
    fn mul(self, v: Vec3<f64>) -> Self::Output {
        Vec3::new(
            *self.get(0,0) * v.x + *self.get(0,1) * v.y + *self.get(0,2) * v.z,
            *self.get(1,0) * v.x + *self.get(1,1) * v.y + *self.get(1,2) * v.z,
            *self.get(2,0) * v.x + *self.get(2,1) * v.y + *self.get(2,2) * v.z,
        )
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