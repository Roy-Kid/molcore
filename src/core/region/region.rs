//! Geometric regions and spatial predicates.
//!
//! This module provides a lightweight trait [`Region`] for geometric regions and a
//! concrete [`Sphere`] implementation. It also defines a few nalgebra-based type
//! aliases (points as an N×3 matrix, bounds as a 3×2 matrix, etc.).
//!
//! Type layout conventions:
//! - [`PointsNx3f`]: N×3 row-major matrix, each row is a point (x, y, z).
//! - [`Bounds3f`]: 3×2 matrix where column 0 is the min corner, column 1 is the max corner;
//!   rows correspond to x/y/z respectively.
//!
//! Example
//! -------
//! ```
//! use molcore::core::region::{Region, Sphere, Point3f, PointsNx3f};
//! use nalgebra as na;
//!
//! let sphere = Sphere::new(Point3f::new(1.0, 2.0, 3.0), 2.0);
//! let pts: PointsNx3f = na::MatrixXx3::from_rows(&[
//!     na::RowVector3::new(1.0, 2.0, 3.0), // center
//!     na::RowVector3::new(3.0, 4.0, 5.0), // on surface (distance = 2*sqrt(3) > 2?)
//!     na::RowVector3::new(-1.0, 0.0, 1.0),
//! ]);
//! let mask = sphere.contains(&pts);
//! assert_eq!(mask.len(), 3);
//! // First point is inside; the others depend on radius and center
//! assert!(mask[0]);
//! ```

use nalgebra as na;
use na::{DVector, Matrix3x2, MatrixXx3, Point3, Vector3};

/// Scalar used by region types.
pub type F = f32;

/// 3D vector of `F`.
pub type Vector3f = Vector3<F>;

/// 3D point of `F`.
pub type Point3f = Point3<F>;

/// Axis-aligned bounding box (AABB) as a 3×2 matrix.
///
/// Column 0 is the minimum corner, column 1 is the maximum corner.
/// Rows correspond to x, y, z respectively:
///
/// [ [min_x, max_x],
///   [min_y, max_y],
///   [min_z, max_z] ]
pub type Bounds3f = Matrix3x2<F>;

/// N×3 matrix of points; each row is a point (x, y, z).
pub type PointsNx3f = MatrixXx3<F>;

/// Region trait for geometric queries.
pub trait Region {
    /// Returns the axis-aligned bounding box of the region.
    ///
    /// Layout: rows = x/y/z; col 0 = min, col 1 = max.
    fn bounds(&self) -> Bounds3f;

    /// Batched containment test for a set of 3D points.
    ///
    /// Returns a boolean vector of length N where each entry indicates whether
    /// the corresponding row in [`PointsNx3f`] lies inside the region.
    ///
    /// Panics
    /// - If `points` does not have exactly 3 columns.
    fn contains(&self, points: &PointsNx3f) -> DVector<bool>;
}

/// A solid sphere region.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    /// Center of the sphere.
    pub center: Point3f,
    /// Radius of the sphere.
    pub radius: F,
}

impl Sphere {
    /// Creates a sphere with a given center and radius.
    pub fn new(center: Point3f, radius: F) -> Self {
        Self { center, radius }
    }

    /// Creates a sphere centered at the origin with the given radius.
    pub fn with_radius(radius: F) -> Self {
        Self {
            center: Point3f::origin(),
            radius,
        }
    }
}

impl Region for Sphere {
    fn bounds(&self) -> Bounds3f {
        let r = Vector3f::repeat(self.radius);
        let min = self.center.coords - r;
        let max = self.center.coords + r;
        Bounds3f::from_columns(&[min, max])
    }

    fn contains(&self, points: &PointsNx3f) -> DVector<bool> {
        assert!(
            points.ncols() == 3,
            "points must have shape (N, 3), got (N, {})",
            points.ncols()
        );

        let r2 = self.radius * self.radius;
        let n = points.nrows();
        let mut mask = DVector::from_element(n, false);

        // Iterate by row and compute squared distance to center.
        for i in 0..n {
            let p = points.row(i);
            let dx = p[0] - self.center.x;
            let dy = p[1] - self.center.y;
            let dz = p[2] - self.center.z;
            mask[i] = (dx * dx + dy * dy + dz * dz) <= r2;
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra as na;

    #[test]
    fn sphere_bounds_are_correct() {
        let s = Sphere::new(Point3f::new(1.0, 2.0, 3.0), 2.0);
        let b = s.bounds();
        // min column
        assert_eq!(b[(0, 0)], -1.0);
        assert_eq!(b[(1, 0)], 0.0);
        assert_eq!(b[(2, 0)], 1.0);
        // max column
        assert_eq!(b[(0, 1)], 3.0);
        assert_eq!(b[(1, 1)], 4.0);
        assert_eq!(b[(2, 1)], 5.0);
    }

    #[test]
    fn sphere_contains_points() {
        let s = Sphere::with_radius(2.0);
        let pts: PointsNx3f = na::MatrixXx3::from_rows(&[
            na::RowVector3::new(0.0, 0.0, 0.0), // inside (center)
            na::RowVector3::new(2.0, 0.0, 0.0), // on surface
            na::RowVector3::new(2.1, 0.0, 0.0), // outside
        ]);
        let mask = s.contains(&pts);
        assert_eq!(mask.len(), 3);
        assert_eq!(mask[0], true);
        assert_eq!(mask[1], true);
        assert_eq!(mask[2], false);
    }
}
