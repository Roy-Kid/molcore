//! Triclinic simulation box and periodic operations (array-based, no external deps).
//!
//! Conventions (fractional/cartesian):
//! - cart = origin + H * frac
//! - frac = H^{-1} * (cart - origin)
//! - Lattice vectors are the columns of H.

use super::region::{Point3f, Vector3f, F, PointsNx3f};
use crate::core::array::Mat3;
use crate::core::array::NdArray;
use crate::core::array::Array;

/// Result type for coordinate operations returning an N×3 array (x, y, z columns)
pub type CoordsNx3 = PointsNx3f;

/// Simulation box: triclinic cell with origin and per-axis PBC mask
#[derive(Debug, Clone)]
pub struct Box {
    /// Triclinic cell matrix H (columns are lattice vectors)
    pub h: Mat3<F>,
    /// Origin of the cell in Cartesian coordinates
    pub origin: Point3f,
    /// Per-axis periodic boundary condition flags (x, y, z)
    pub pbc: [bool; 3],
}

impl Box {
    /// Construct from triclinic cell matrix `H`, origin `O`, and per-axis PBC flags
    pub fn new(h: Mat3<F>, origin: Point3f, pbc: [bool; 3]) -> Self {
        Self { h, origin, pbc }
    }

    /// Factory: cubic box with edge length `a` and origin `O`
    pub fn cube(a: F, origin: Point3f, pbc: [bool; 3]) -> Self {
        let h = Mat3::new(a, 0.0, 0.0,
                          0.0, a, 0.0,
                          0.0, 0.0, a);
        Self::new(h, origin, pbc)
    }

    /// Factory: ortho box with lengths (ax, ay, az) and origin `O`
    pub fn ortho(lengths: Vector3f, origin: Point3f, pbc: [bool; 3]) -> Self {
        let h = Mat3::new(lengths.x, 0.0, 0.0,
                          0.0, lengths.y, 0.0,
                          0.0, 0.0, lengths.z);
        Self::new(h, origin, pbc)
    }

    /// Cell volume (|det(H)|)
    pub fn volume(&self) -> F { self.h.det().abs() }

    /// Return lattice vector by index (0,1,2) as a Vec3 (columns of H)
    pub fn lattice_vector(&self, index: usize) -> Vector3f {
        assert!(index < 3, "lattice_vector index must be 0..2");
        let m = self.h.as_array();
        // columns of H: take each row's selected column
        Vector3f::new(m[0][index], m[1][index], m[2][index])
    }

    /// Distance from origin to nearest plane for each axis (|a|/2, |b|/2, |c|/2)
    pub fn nearest_plane_distance(&self) -> Vector3f {
        let a = self.lattice_vector(0);
        let b = self.lattice_vector(1);
        let c = self.lattice_vector(2);
        Vector3f::new(0.5 * a.norm(), 0.5 * b.norm(), 0.5 * c.norm())
    }

    /// Convert a single Cartesian coordinate to fractional
    pub fn to_frac_single(&self, cart: Point3f) -> Vector3f {
        let inv = self.h.inv().expect("cell matrix is singular");
        let r = Vector3f::new(cart.x - self.origin.x, cart.y - self.origin.y, cart.z - self.origin.z);
        inv * r
    }

    /// Convert a single fractional coordinate to Cartesian
    pub fn to_cart_single(&self, frac: Vector3f) -> Point3f {
        // r = H * frac
        let r = self.h * frac;
        Point3f::new(self.origin.x + r.x, self.origin.y + r.y, self.origin.z + r.z)
    }

    /// Convert Cartesian coordinates (N×3) to scaled (fractional) coordinates (N×3).
    /// LAMMPS convention: x,y,z (Cartesian) -> xs,ys,zs (scaled/fractional)
    pub fn to_frac_points(&self, xyz: &PointsNx3f) -> CoordsNx3 {
        assert!(xyz.shape().len() == 2 && xyz.shape()[1] == 3, "xyz must have shape (N, 3)");
        let n = xyz.shape()[0];
        let inv = self.h.inv().expect("cell matrix is singular");
        let mut out = Vec::with_capacity(n * 3);
        for i in 0..n {
            let base = i * 3;
            let px = xyz.data()[base + 0];
            let py = xyz.data()[base + 1];
            let pz = xyz.data()[base + 2];
            let v = Vector3f::new(px - self.origin.x, py - self.origin.y, pz - self.origin.z);
            let f = inv * v;
            out.push(f.x);
            out.push(f.y);
            out.push(f.z);
        }
    NdArray::from_vec(vec![n, 3], out)
    }

    /// Convert scaled (fractional) coordinates (N×3) to Cartesian coordinates (N×3).
    /// LAMMPS convention: xs,ys,zs (scaled/fractional) -> x,y,z (Cartesian)
    pub fn to_cart_points(&self, xyzs: &PointsNx3f) -> CoordsNx3 {
        assert!(xyzs.shape().len() == 2 && xyzs.shape()[1] == 3, "xyzs must have shape (N, 3)");
        let n = xyzs.shape()[0];
        let mut out = Vec::with_capacity(n * 3);
        for i in 0..n {
            let base = i * 3;
            let fx = xyzs.data()[base + 0];
            let fy = xyzs.data()[base + 1];
            let fz = xyzs.data()[base + 2];
            let frac = Vector3f::new(fx, fy, fz);
            let r = self.h * frac;
            let cart = Point3f::new(self.origin.x + r.x, self.origin.y + r.y, self.origin.z + r.z);
            out.push(cart.x);
            out.push(cart.y);
            out.push(cart.z);
        }
    NdArray::from_vec(vec![n, 3], out)
    }

    /// Check whether points are inside primary cell (0 <= scaled < 1 per periodic axis).
    /// Returns a boolean array of shape [N].
    pub fn isin_points(&self, xyz: &PointsNx3f) -> NdArray<bool> {
        let frac = self.to_frac_points(xyz);
        let n = frac.shape()[0];
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let base = i * 3;
            let xs = frac.data()[base + 0];
            let ys = frac.data()[base + 1];
            let zs = frac.data()[base + 2];
            let okx = in01(self.pbc[0], xs);
            let oky = in01(self.pbc[1], ys);
            let okz = in01(self.pbc[2], zs);
            out.push(okx && oky && okz);
        }
    NdArray::from_vec(vec![n], out)
    }



    /// Displacement vector d = b - a in Cartesian, with optional minimum image in PBC.
    pub fn delta_vec(&self, a: Point3f, b: Point3f, minimum_image: bool) -> Vector3f {
        if !minimum_image {
            return b - a;
        }
        let inv = self.h.inv().expect("cell matrix is singular");
        let ra = Vector3f::new(a.x - self.origin.x, a.y - self.origin.y, a.z - self.origin.z);
        let rb = Vector3f::new(b.x - self.origin.x, b.y - self.origin.y, b.z - self.origin.z);
        let fa = inv * ra;
        let fb = inv * rb;
        let mut df = Vector3f::new(fb.x - fa.x, fb.y - fa.y, fb.z - fa.z);
        if self.pbc[0] { df.x = wrap_mi(df.x); }
        if self.pbc[1] { df.y = wrap_mi(df.y); }
        if self.pbc[2] { df.z = wrap_mi(df.z); }
        self.h * df
    }

    /// Compute displacement vectors between two sets of points (both N×3).
    /// LAMMPS convention: uses unwrapped coordinates for input. Returns N×3 array of d = p2 - p1.
    pub fn delta_points(&self, xyzu1: &PointsNx3f, xyzu2: &PointsNx3f, minimum_image: bool) -> CoordsNx3 {
        assert!(xyzu1.shape().len() == 2 && xyzu1.shape()[1] == 3, "xyzu1 must have shape (N, 3)");
        assert!(xyzu2.shape().len() == 2 && xyzu2.shape()[1] == 3, "xyzu2 must have shape (N, 3)");
        assert_eq!(xyzu1.shape()[0], xyzu2.shape()[0], "xyzu1/xyzu2 must have same number of rows");
        let n = xyzu1.shape()[0];
        let mut out = Vec::with_capacity(n * 3);
        for i in 0..n {
            let b = i * 3;
            let p1 = Point3f::new(xyzu1.data()[b + 0], xyzu1.data()[b + 1], xyzu1.data()[b + 2]);
            let p2 = Point3f::new(xyzu2.data()[b + 0], xyzu2.data()[b + 1], xyzu2.data()[b + 2]);
            let d = self.delta_vec(p1, p2, minimum_image);
            out.push(d.x);
            out.push(d.y);
            out.push(d.z);
        }
    NdArray::from_vec(vec![n, 3], out)
    }

    /// Wrap unwrapped coordinates (N×3) into the primary cell (0 <= scaled < 1 for periodic axes).
    /// LAMMPS convention: xu,yu,zu (unwrapped) -> x,y,z (wrapped)
    pub fn wrap_points(&self, xyzu: &PointsNx3f) -> CoordsNx3 {
    let frac = self.to_frac_points(xyzu);
    let n = frac.shape()[0];
    let mut data: Vec<F> = frac.data().to_vec();
        for i in 0..n {
            let b = i * 3;
            if self.pbc[0] { data[b + 0] = data[b + 0] - data[b + 0].floor(); }
            if self.pbc[1] { data[b + 1] = data[b + 1] - data[b + 1].floor(); }
            if self.pbc[2] { data[b + 2] = data[b + 2] - data[b + 2].floor(); }
        }
    let wrapped_frac = NdArray::from_vec(vec![n, 3], data);
        self.to_cart_points(&wrapped_frac)
    }
}

#[inline]
fn in01(pbc: bool, x: F) -> bool {
    if pbc { x >= 0.0 && x < 1.0 } else { x >= 0.0 && x < 1.0 }
}

#[inline]
fn wrap_mi(x: F) -> F { x - x.round() }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::array::NdArray;

    #[test]
    fn roundtrip_frac_cart() {
        let bx = Box::ortho(
            Vector3f::new(2.0, 3.0, 4.0), 
            Point3f::new(0.5, -1.0, 2.0), 
            [true, true, true]
        );
    let pts = NdArray::from_vec(vec![2,3], vec![0.5, -1.0, 2.0, 2.5, 2.0, 6.0]);
        let frac = bx.to_frac_points(&pts);
        let cart = bx.to_cart_points(&frac);
        for i in 0..2 {
            let b = i*3;
            assert!((pts.data()[b+0] - cart.data()[b+0]).abs() < 1e-5);
            assert!((pts.data()[b+1] - cart.data()[b+1]).abs() < 1e-5);
            assert!((pts.data()[b+2] - cart.data()[b+2]).abs() < 1e-5);
        }
    }

    #[test]
    fn wrap_into_cell() {
        let bx = Box::cube(2.0, Point3f::origin(), [true, true, true]);
    let pts = NdArray::from_vec(vec![2,3], vec![2.1, -0.1, 3.9, -1.9, 4.2, 0.0]);
        let wrapped = bx.wrap_points(&pts);
        let frac = bx.to_frac_points(&wrapped);
        for i in 0..2 { 
            let b = i*3;
            let fx = frac.data()[b+0];
            let fy = frac.data()[b+1];
            let fz = frac.data()[b+2];
            assert!(fx >= 0.0 && fx < 1.0, "xs[{}] = {} not in [0, 1)", i, fx);
            assert!(fy >= 0.0 && fy < 1.0, "ys[{}] = {} not in [0, 1)", i, fy);
            assert!(fz >= 0.0 && fz < 1.0, "zs[{}] = {} not in [0, 1)", i, fz);
        }
    }

    #[test]
    fn minimum_image_delta() {
        let my_box = Box::ortho(
            Vector3f::new(10.0, 10.0, 10.0), 
            Point3f::origin(), 
            [true, true, true]
        );
    let p1 = NdArray::from_vec(vec![1,3], vec![9.0, 9.0, 9.0]);
    let p2 = NdArray::from_vec(vec![1,3], vec![1.0, 1.0, 1.0]);
        let d = my_box.delta_points(&p1, &p2, true);
        let dx = d.data()[0];
        let dy = d.data()[1];
        let dz = d.data()[2];
        assert!((dx.abs() - 2.0).abs() < 1e-5, "dx = {}", dx);
        assert!((dy.abs() - 2.0).abs() < 1e-5, "dy = {}", dy);
        assert!((dz.abs() - 2.0).abs() < 1e-5, "dz = {}", dz);
    }
    
    #[test]
    fn is_in_test() {
        let bx = Box::cube(2.0, Point3f::origin(), [true, true, true]);
    let pts = NdArray::from_vec(vec![3,3], vec![1.0,1.0,1.0, 2.5,1.0,1.0, -0.5,1.0,1.0]);
        let mask = bx.isin_points(&pts);
        assert_eq!(mask[[0]], true);
        assert_eq!(mask[[1]], false);
        assert_eq!(mask[[2]], false);
    }
}
