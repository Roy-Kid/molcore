//! Geometric regions and spatial predicates.
//!
//! This module provides a lightweight trait [`Region`] for geometric regions and a
//! concrete [`Sphere`] implementation. It also defines array-based type aliases
//! (points as an N×3 NdArray, bounds as a 3×2 NdArray, etc.).
//!
//! Type layout conventions:
//! - [`PointsNx3f`]: N×3 row-major array, each row is a point (x, y, z).
//! - [`Bounds3f`]: 3×2 array where column 0 is the min corner, column 1 is the max corner;
//!   rows correspond to x/y/z respectively.
//!
//! Example
//! -------
//! ```
//! use molcore::core::region::{Region, Sphere, Point3f, PointsNx3f};
//! use molcore::core::array::NdArray;
//!
//! let sphere = Sphere::new(Point3f::new(1.0, 2.0, 3.0), 2.0);
//! let pts: PointsNx3f = NdArray::from_vec(
//!     vec![3, 3],
//!     vec![
//!         1.0, 2.0, 3.0, // center
//!         3.0, 4.0, 5.0,
//!        -1.0, 0.0, 1.0,
//!     ],
//! );
//! let mask = sphere.contains(&pts);
//! assert_eq!(mask.data().len(), 3);
//! // First point is inside; the others depend on radius and center
//! assert!(mask[[0]]);
//! ```

use crate::core::array::{NdArray, Vec3, Array};

/// Scalar used by region types.
pub type F = f32;

/// 3D vector of `F`.
pub type Vector3f = Vec3<F>;

/// 3D point of `F`.
pub type Point3f = Vec3<F>;

/// Axis-aligned bounding box (AABB) as a 3×2 matrix.
///
/// Column 0 is the minimum corner, column 1 is the maximum corner.
/// Rows correspond to x, y, z respectively:
///
/// [ [min_x, max_x],
///   [min_y, max_y],
///   [min_z, max_z] ]
/// Bounds as a 3×2 row-major array: rows=x/y/z; cols=(min, max)
pub type Bounds3f = NdArray<F>;

/// N×3 matrix of points; each row is a point (x, y, z).
/// Points as an N×3 row-major array (each row is x,y,z)
pub type PointsNx3f = NdArray<F>;

/// Region trait for geometric queries.
pub trait Region {
    /// Returns the axis-aligned bounding box of the region.
    ///
    /// Layout: rows = x/y/z; col 0 = min, col 1 = max.
    fn bounds(&self) -> Bounds3f;

    /// Batched containment test for a set of 3D points.
    ///
    /// Returns a boolean NdArray of shape [N] where each entry indicates whether
    /// the corresponding row in [`PointsNx3f`] lies inside the region.
    ///
    /// Panics
    /// - If `points` does not have exactly 3 columns.
    fn contains(&self, points: &PointsNx3f) -> NdArray<bool>;
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
        let r = self.radius;
        let min = Vector3f::new(self.center.x - r, self.center.y - r, self.center.z - r);
        let max = Vector3f::new(self.center.x + r, self.center.y + r, self.center.z + r);
        // Row-major [ [min_x,max_x], [min_y,max_y], [min_z,max_z] ]
    NdArray::from_vec(
            vec![3, 2],
            vec![min.x, max.x, min.y, max.y, min.z, max.z],
        )
    }

    fn contains(&self, points: &PointsNx3f) -> NdArray<bool> {
    assert!(points.shape().len() == 2 && points.shape()[1] == 3, "points must have shape (N, 3)");

        let r2 = self.radius * self.radius;
        let n = points.shape()[0];
        let mut data = Vec::with_capacity(n);

        for i in 0..n {
            // row-major access
            let base = i * 3;
            let px = points.data()[base + 0];
            let py = points.data()[base + 1];
            let pz = points.data()[base + 2];
            let dx = px - self.center.x;
            let dy = py - self.center.y;
            let dz = pz - self.center.z;
            data.push((dx * dx + dy * dy + dz * dz) <= r2);
        }

    NdArray::from_vec(vec![n], data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::array::NdArray;

    #[test]
    fn sphere_bounds_are_correct() {
        let s = Sphere::new(Point3f::new(1.0, 2.0, 3.0), 2.0);
        let b = s.bounds();
        // Row-major [ [min_x,max_x], [min_y,max_y], [min_z,max_z] ]
        assert_eq!(b[[0, 0]], -1.0);
        assert_eq!(b[[1, 0]], 0.0);
        assert_eq!(b[[2, 0]], 1.0);
        assert_eq!(b[[0, 1]], 3.0);
        assert_eq!(b[[1, 1]], 4.0);
        assert_eq!(b[[2, 1]], 5.0);
    }

    #[test]
    fn sphere_contains_points() {
        let s = Sphere::with_radius(2.0);
    let pts: PointsNx3f = NdArray::from_vec(
            vec![3, 3],
            vec![
                0.0, 0.0, 0.0, // inside (center)
                2.0, 0.0, 0.0, // on surface
                2.1, 0.0, 0.0, // outside
            ],
        );
        let mask = s.contains(&pts);
        assert_eq!(mask.shape(), &[3]);
        assert_eq!(mask[[0]], true);
        assert_eq!(mask[[1]], true);
        assert_eq!(mask[[2]], false);
    }
}
