//! Triclinic simulation box with general coordinate transforms and PBC handling.
//!
//! The box is represented by a 3x3 cell matrix `H` (Mat3<f32>) and an origin `O` (Vec3<f32>).
//! We treat all boxes as triclinic; orthorhombic and cubic are special cases of `H`.
//!
//! Conventions
//! - Cartesian: `cart = O + H * frac`
//! - Fractional: `frac = H^{-1} * (cart - O)`
//! - Lattice vectors are the columns of `H`.

use super::array::{Mat3, Vec3, NdArray, Array};

/// Simulation box: triclinic cell with origin and per-axis PBC mask
#[derive(Debug, Clone)]
pub struct SimBox {
    /// Triclinic cell matrix H (columns are lattice vectors)
    pub h: Mat3<f32>,
    /// Origin of the cell in Cartesian coordinates
    pub origin: Vec3<f32>,
    /// Per-axis periodic boundary condition flags (x, y, z)
    pub pbc: Vec3<bool>,
}

impl SimBox {
    /// Construct from triclinic cell matrix `H`, origin `O`, and per-axis PBC flags
    pub fn new(h: Mat3<f32>, origin: Vec3<f32>, pbc: Vec3<bool>) -> Self {
        Self { h, origin, pbc }
    }

    /// Factory: cubic box with edge length `a` and origin `O`
    pub fn cube(a: f32, origin: Vec3<f32>, pbc: Vec3<bool>) -> Self {
        let h = Mat3::from_rows([a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a]);
        Self::new(h, origin, pbc)
    }

    /// Factory: orthorhombic box with lengths (ax, ay, az) and origin `O`
    pub fn orthorhombic(lengths: Vec3<f32>, origin: Vec3<f32>, pbc: Vec3<bool>) -> Self {
        let h = Mat3::from_rows([lengths.x, 0.0, 0.0], [0.0, lengths.y, 0.0], [0.0, 0.0, lengths.z]);
        Self::new(h, origin, pbc)
    }

    /// Cell volume (|det(H)|)
    pub fn volume(&self) -> f64 { self.h.det().abs() as f64 }

    /// Return lattice vector by index (0,1,2) as a Vec3 (columns of H)
    pub fn lattice_vector(&self, index: usize) -> Vec3<f32> {
        assert!(index < 3, "lattice_vector index must be 0..2");
        let m = self.h.data;
        // columns: (m[0][j], m[1][j], m[2][j])
        match index {
            0 => Vec3::new(m[0][0], m[1][0], m[2][0]),
            1 => Vec3::new(m[0][1], m[1][1], m[2][1]),
            _ => Vec3::new(m[0][2], m[1][2], m[2][2]),
        }
    }

    /// Distance from origin to nearest plane for each axis (|a|/2, |b|/2, |c|/2)
    pub fn nearest_plane_distance(&self) -> Vec3<f32> {
        let a = self.lattice_vector(0);
        let b = self.lattice_vector(1);
        let c = self.lattice_vector(2);
        Vec3::new(0.5 * a.norm(), 0.5 * b.norm(), 0.5 * c.norm())
    }

    /// Convert a single Cartesian coordinate to fractional
    pub fn to_frac_vec(&self, cart: Vec3<f32>) -> Vec3<f32> {
        let inv = self.h.inv().expect("cell matrix is singular");
        let r = cart - self.origin;
        inv * r
    }

    /// Convert a single fractional coordinate to Cartesian
    pub fn to_cart_vec(&self, frac: Vec3<f32>) -> Vec3<f32> {
        let r = self.h * frac;
        self.origin + r
    }

    /// Convert Cartesian array (shape [3] or [N,3]) to fractional
    pub fn to_frac(&self, cart: &NdArray<f32>) -> NdArray<f32> {
        let inv = self.h.inv().expect("cell matrix is singular");
        match cart.shape() {
            [3] => {
                let v = Vec3::new(cart[[0]], cart[[1]], cart[[2]]);
                let f = &inv * (v - self.origin);
                NdArray::new(vec![3], vec![f.x, f.y, f.z])
            }
            [n, 3] => {
                let n = *n;
                let mut out = Vec::with_capacity(3 * n);
                for i in 0..n {
                    let v = Vec3::new(cart[[i, 0]], cart[[i, 1]], cart[[i, 2]]);
                    let f = &inv * (v - self.origin);
                    out.extend_from_slice(&[f.x, f.y, f.z]);
                }
                NdArray::new(vec![n, 3], out)
            }
            shp => panic!("to_frac expects shape [3] or [N,3], got {:?}", shp),
        }
    }

    /// Convert fractional array (shape [3] or [N,3]) to Cartesian
    pub fn to_cart(&self, frac: &NdArray<f32>) -> NdArray<f32> {
        match frac.shape() {
            [3] => {
                let v = Vec3::new(frac[[0]], frac[[1]], frac[[2]]);
                let c = self.to_cart_vec(v);
                NdArray::new(vec![3], vec![c.x, c.y, c.z])
            }
            [n, 3] => {
                let n = *n;
                let mut out = Vec::with_capacity(3 * n);
                for i in 0..n {
                    let v = Vec3::new(frac[[i, 0]], frac[[i, 1]], frac[[i, 2]]);
                    let c = self.to_cart_vec(v);
                    out.extend_from_slice(&[c.x, c.y, c.z]);
                }
                NdArray::new(vec![n, 3], out)
            }
            shp => panic!("to_cart expects shape [3] or [N,3], got {:?}", shp),
        }
    }

    /// Check whether points (Cartesian, shape [3] or [N,3]) are inside primary cell.
    /// For periodic axes, check 0 <= frac < 1; for non-PBC axes, check the same bounds by projection.
    pub fn is_in(&self, points: &NdArray<f32>) -> NdArray<bool> {
        let frac = self.to_frac(points);
        match frac.shape() {
            [3] => {
                let x = frac[[0]]; let y = frac[[1]]; let z = frac[[2]];
                let okx = in01(self.pbc.x, x);
                let oky = in01(self.pbc.y, y);
                let okz = in01(self.pbc.z, z);
                NdArray::new(vec![1], vec![okx && oky && okz])
            }
            [n, 3] => {
                let n = *n;
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let x = frac[[i, 0]]; let y = frac[[i, 1]]; let z = frac[[i, 2]];
                    out.push(in01(self.pbc.x, x) && in01(self.pbc.y, y) && in01(self.pbc.z, z));
                }
                NdArray::new(vec![n], out)
            }
            _ => unreachable!(),
        }
    }

    /// Wrap Cartesian points into the primary cell (vectorized)
    pub fn wrap(&self, points: &NdArray<f32>) -> NdArray<f32> {
        let frac = self.to_frac(points);
        let wrapped = match frac.shape() {
            [3] => {
                let mut fx = frac[[0]]; let mut fy = frac[[1]]; let mut fz = frac[[2]];
                if self.pbc.x { fx = fx - fx.floor(); }
                if self.pbc.y { fy = fy - fy.floor(); }
                if self.pbc.z { fz = fz - fz.floor(); }
                NdArray::new(vec![3], vec![fx, fy, fz])
            }
            [n, 3] => {
                let n = *n;
                let mut out = Vec::with_capacity(3 * n);
                for i in 0..n {
                    let mut fx = frac[[i, 0]]; let mut fy = frac[[i, 1]]; let mut fz = frac[[i, 2]];
                    if self.pbc.x { fx = fx - fx.floor(); }
                    if self.pbc.y { fy = fy - fy.floor(); }
                    if self.pbc.z { fz = fz - fz.floor(); }
                    out.extend_from_slice(&[fx, fy, fz]);
                }
                NdArray::new(vec![n, 3], out)
            }
            _ => unreachable!(),
        };
        self.to_cart(&wrapped)
    }

    /// Displacement vector d = b - a in Cartesian, with optional minimum image in PBC.
    pub fn delta_vec(&self, a: Vec3<f32>, b: Vec3<f32>, minimum_image: bool) -> Vec3<f32> {
        if !minimum_image {
            return b - a;
        }
        let inv = self.h.inv().expect("cell matrix is singular");
        let fa = &inv * (a - self.origin);
        let fb = &inv * (b - self.origin);
        let mut df = fb - fa;
        if self.pbc.x { df.x = wrap_mi(df.x); }
        if self.pbc.y { df.y = wrap_mi(df.y); }
        if self.pbc.z { df.z = wrap_mi(df.z); }
        self.h * df
    }

    /// Displacements for arrays (Cartesian, shape [3] or [N,3])
    pub fn delta(&self, a: &NdArray<f32>, b: &NdArray<f32>, minimum_image: bool) -> NdArray<f32> {
        match (a.shape(), b.shape()) {
            ([3], [3]) => {
                let va = Vec3::new(a[[0]], a[[1]], a[[2]]);
                let vb = Vec3::new(b[[0]], b[[1]], b[[2]]);
                let d = self.delta_vec(va, vb, minimum_image);
                NdArray::new(vec![3], vec![d.x, d.y, d.z])
            }
            ([n, 3], [m, 3]) if n == m => {
                let n = *n;
                let mut out = Vec::with_capacity(3 * n);
                for i in 0..n {
                    let va = Vec3::new(a[[i, 0]], a[[i, 1]], a[[i, 2]]);
                    let vb = Vec3::new(b[[i, 0]], b[[i, 1]], b[[i, 2]]);
                    let d = self.delta_vec(va, vb, minimum_image);
                    out.extend_from_slice(&[d.x, d.y, d.z]);
                }
                NdArray::new(vec![n, 3], out)
            }
            (sa, sb) => panic!("delta expects shapes both [3] or both [N,3], got {:?} and {:?}", sa, sb),
        }
    }
}

#[inline]
fn in01(pbc: bool, x: f32) -> bool {
    if pbc { x >= 0.0 && x < 1.0 } else { x >= 0.0 && x < 1.0 }
}

#[inline]
fn wrap_mi(x: f32) -> f32 { x - x.round() }
