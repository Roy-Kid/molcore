//! Triclinic simulation box and periodic operations based on ndarray.
//!
//! Conventions (fractional/cartesian):
//! - cart = origin + H * frac
//! - frac = H^{-1} * (cart - origin)
//! - Lattice vectors are the columns of H.

use crate::math::{det3, inv3, matmul, norm3};
use ndarray::{array, Array1, Array2, ArrayView2, Axis, azip};

pub type F = f32;
pub type Vec3 = Array1<F>; // length-3
pub type Mat3 = Array2<F>; // shape (3,3)
pub type PointsNx3 = Array2<F>; // shape (N,3) owned

/// Simulation box: triclinic cell with origin and per-axis PBC mask
#[derive(Debug, Clone)]
pub struct Box {
    /// Triclinic cell matrix H (columns are lattice vectors)
    pub h: Mat3,
    /// Precomputed inverse of H
    pub inv: Mat3,
    /// Origin of the cell in Cartesian coordinates
    pub origin: Vec3,
    /// Per-axis periodic boundary condition flags (x, y, z)
    pub pbc: Array1<bool>,
}

// define box error
#[derive(Debug)]
pub enum BoxError {
    SingularCell,
}

impl Box {
    /// Construct from triclinic cell matrix `H`, origin `O`, and per-axis PBC flags
    pub fn new(h: Mat3, origin: Vec3, pbc: [bool; 3]) -> Self {
        let inv = inv3(&h).expect("cell matrix is singular");
        let pbc = array![pbc[0], pbc[1], pbc[2]];
        Self { h, inv, origin, pbc }
    }

    pub fn try_new(h: Mat3, origin: Vec3, pbc: [bool; 3]) -> Result<Self, BoxError> {
        if let Some(inv) = inv3(&h) {
            let pbc = array![pbc[0], pbc[1], pbc[2]];
            Ok(Self { h, inv, origin, pbc })
        } else {
            Err(BoxError::SingularCell)
        }
    }

    /// Factory: cubic box with edge length `a` and origin `O`
    pub fn cube(a: F, origin: Vec3, pbc: [bool; 3]) -> Self {
        let h = array![[a, 0.0, 0.0], [0.0, a, 0.0], [0.0, 0.0, a],];
        Self::new(h, origin, pbc)
    }

    /// Factory: ortho box with lengths (ax, ay, az) and origin `O`
    pub fn ortho(lengths: Vec3, origin: Vec3, pbc: [bool; 3]) -> Self {
        let h = array![
            [lengths[0], 0.0, 0.0],
            [0.0, lengths[1], 0.0],
            [0.0, 0.0, lengths[2]],
        ];
        Self::new(h, origin, pbc)
    }

    /// Cell volume (|det(H)|)
    pub fn volume(&self) -> F {
        det3(&self.h).abs()
    }

    pub fn tilts(&self) -> Vec3 {
        array![
            self.h[[0, 1]], // xy
            self.h[[0, 2]], // xz
            self.h[[1, 2]], // yz
        ]
    }

    pub fn lengths(&self) -> Vec3 {
        array![
            norm3(&self.lattice(0)), // |a|
            norm3(&self.lattice(1)), // |b|
            norm3(&self.lattice(2)), // |c|
        ]
    }

    /// Return lattice vector by index (0,1,2) as a length-3 ndarray (columns of H)
    pub fn lattice(&self, index: usize) -> Vec3 {
        assert!(index < 3, "lattice index must be 0..2");
        array![self.h[[0, index]], self.h[[1, index]], self.h[[2, index]]]
    }

    /// Distance from origin to nearest plane for each axis (|a|/2, |b|/2, |c|/2)
    pub fn nearest_plane_distance(&self) -> Vec3 {
        let a = self.lattice(0);
        let b = self.lattice(1);
        let c = self.lattice(2);
        array![0.5 * norm3(&a), 0.5 * norm3(&b), 0.5 * norm3(&c)]
    }

    /// Convert Cartesian points to fractional coordinates (N×3)
    pub fn to_frac(&self, xyz: ArrayView2<F>) -> Array2<F> {
        assert_eq!(xyz.ncols(), 3, "to_frac expects (N,3) points");
    let mut shifted = xyz.to_owned();
        for j in 0..3 {
            let o = self.origin[j];
            shifted
                .column_mut(j)
                .iter_mut()
                .for_each(|v| *v -= o);
        }
    let inv_t = self.inv.t().to_owned();
    matmul(shifted.view(), &inv_t)
    }

    /// Convert fractional coordinates to Cartesian points (N×3)
    pub fn to_cart(&self, xyzs: ArrayView2<F>) -> Array2<F> {
        assert_eq!(xyzs.ncols(), 3, "to_cart expects (N,3) points");
    let h_t = self.h.t().to_owned();
    let mut out = matmul(xyzs, &h_t);
        for j in 0..3 {
            let o = self.origin[j];
            out.column_mut(j).iter_mut().for_each(|v| *v += o);
        }
        out
    }

    /// Check if points lie within [0,1) in fractional space for enabled PBC axes.
    pub fn isin(&self, xyz: ArrayView2<F>) -> Array1<bool> {
        let frac = self.to_frac(xyz);
        let n = frac.nrows();
        let mut mask = Array1::from_elem(n, true);

        if self.pbc[0] {
            let fx = frac.index_axis(Axis(1), 0);
            azip!((m in &mut mask, &v in fx) { *m &= v >= 0.0 && v < 1.0; });
        }
        if self.pbc[1] {
            let fy = frac.index_axis(Axis(1), 1);
            azip!((m in &mut mask, &v in fy) { *m &= v >= 0.0 && v < 1.0; });
        }
        if self.pbc[2] {
            let fz = frac.index_axis(Axis(1), 2);
            azip!((m in &mut mask, &v in fz) { *m &= v >= 0.0 && v < 1.0; });
        }
        mask
    }

    /// Batched displacement vectors row-wise (N×3)
    pub fn delta(
        &self,
        xyzu1: ArrayView2<F>,
        xyzu2: ArrayView2<F>,
        minimum_image: bool,
    ) -> Array2<F> {
        assert_eq!(xyzu1.ncols(), 3);
        assert_eq!(xyzu2.ncols(), 3);
        assert_eq!(xyzu1.nrows(), xyzu2.nrows());
        if !minimum_image {
            return &xyzu2.to_owned() - &xyzu1.to_owned();
        }
        // MIC path in fractional space
        let n = xyzu1.nrows();
        let mut s1 = xyzu1.to_owned();
        let mut s2 = xyzu2.to_owned();
        for j in 0..3 {
            let o = self.origin[j];
            s1.column_mut(j).iter_mut().for_each(|v| *v -= o);
            s2.column_mut(j).iter_mut().for_each(|v| *v -= o);
        }
    let inv_t = self.inv.t().to_owned();
        let f1 = matmul(s1.view(), &inv_t);
        let f2 = matmul(s2.view(), &inv_t);
        let df_raw = &f2 - &f1; // raw fractional displacement
        let df_wrapped = df_raw.mapv(wrap_mi);
        // Build float masks from boolean PBC flags
        let mask: Vec3 = self.pbc.mapv(|b| if b { 1.0 } else { 0.0 });
        let inv_mask: Vec3 = mask.mapv(|m| 1.0 - m);
        let mask_bc = mask.broadcast((n, 3)).expect("broadcast mask");
        let inv_mask_bc = inv_mask.broadcast((n, 3)).expect("broadcast inv mask");
        // df = wrapped*mask + raw*(1-mask)
        let df = df_wrapped * &mask_bc + df_raw * &inv_mask_bc;
        // back to cart
    let h_t = self.h.t().to_owned();
    matmul(df.view(), &h_t)
    }

    /// Wrap Cartesian points into the unit cell according to PBC
    pub fn wrap(&self, xyzu: ArrayView2<F>) -> Array2<F> {
        assert_eq!(xyzu.ncols(), 3, "wrap expects (N,3) points");
        let frac_orig = self.to_frac(xyzu);
        let n = frac_orig.nrows();
        let frac_wrapped = frac_orig.mapv(|x| x - x.floor());
        let mask: Vec3 = self.pbc.mapv(|b| if b { 1.0 } else { 0.0 });
        let inv_mask: Vec3 = mask.mapv(|m| 1.0 - m);
        let mask_bc = mask.broadcast((n, 3)).expect("broadcast mask");
        let inv_mask_bc = inv_mask.broadcast((n, 3)).expect("broadcast inv mask");
        let frac = frac_wrapped * &mask_bc + frac_orig * &inv_mask_bc;
        self.to_cart(frac.view())
    }
}

#[inline]
fn wrap_mi(x: F) -> F {
    x - x.round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_frac_cart() {
        let bx = Box::ortho(
            array![2.0, 3.0, 4.0],
            array![0.5, -1.0, 2.0],
            [true, true, true],
        );
        let pts = array![[0.5, -1.0, 2.0], [2.5, 2.0, 6.0]];
        let frac = bx.to_frac(pts.view());
        let cart = bx.to_cart(frac.view());
        assert!((&pts - &cart).iter().all(|v| v.abs() < 1e-5));
    }

    #[test]
    fn wrap_into_cell() {
        let bx = Box::cube(2.0, array![0.0, 0.0, 0.0], [true, true, true]);
        let pts = array![[2.1, -0.1, 3.9], [-1.9, 4.2, 0.0]];
        let wrapped = bx.wrap(pts.view());
        let frac = bx.to_frac(wrapped.view());
        for i in 0..wrapped.nrows() {
            let fx = frac[[i, 0]];
            let fy = frac[[i, 1]];
            let fz = frac[[i, 2]];
            assert!(fx >= 0.0 && fx < 1.0);
            assert!(fy >= 0.0 && fy < 1.0);
            assert!(fz >= 0.0 && fz < 1.0);
        }
    }

    #[test]
    fn minimum_image_delta() {
        let bx = Box::ortho(
            array![10.0, 10.0, 10.0],
            array![0.0, 0.0, 0.0],
            [true, true, true],
        );
        let p1 = array![[9.0, 9.0, 9.0]]; // 1x3
        let p2 = array![[1.0, 1.0, 1.0]];
        let d = bx.delta(p1.view(), p2.view(), true);
        let dx = d[[0, 0]];
        let dy = d[[0, 1]];
        let dz = d[[0, 2]];
        assert!((dx.abs() - 2.0).abs() < 1e-5);
        assert!((dy.abs() - 2.0).abs() < 1e-5);
        assert!((dz.abs() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_construct() {

        // singular box should fail
        let b = Box::try_new(
            array![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            array![0.0, 0.0, 0.0],
            [true, true, true],
        );
        assert!(b.is_err());

        let b = Box::try_new(
            array![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            array![0.0, 0.0, 0.0],
            [true, true, true],
        );
        assert!(b.is_ok());
    }

    #[test]
    fn test_get_length() {
        let b = Box::new(array![
            [2.0, 0.0, 0.0],
            [0.0, 3.0, 0.0],
            [0.0, 0.0, 4.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let lengths = b.lengths();
        assert_eq!(lengths, array![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_get_tilt_factor() {
        let b = Box::new(array![
            [2.0, 1.0, 2.0],
            [0.0, 3.0, 3.0],
            [0.0, 0.0, 4.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let tilt = b.tilts();
        assert_eq!(tilt, array![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_volume() {
        let b = Box::new(array![
            [2.0, 1.0, 2.0],
            [0.0, 3.0, 3.0],
            [0.0, 0.0, 4.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let vol = b.volume();
        assert!((vol - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_wrap_single_particle_through_batch() {
        let b = Box::new(array![
            [2.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 2.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let pts = array![[0.0, -1.0, -1.0]];
        let wrapped = b.wrap(pts.view());
        let expected = array![[1.0, 1.0, 1.0]];
        for j in 0..3 {
            assert!((wrapped[[0, j]] - expected[[0, j]]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_wrap_multiple_particles() {
        let b = Box::new(array![
            [2.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 2.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let pts = array![[0.0, -1.0, -1.0], [0.0, 0.5, 0.0]];
        let wrapped = b.wrap(pts.view());
        let expected = array![[1.0, 1.0, 1.0], [2.0, 0.5, 0.0]];
        for i in 0..wrapped.nrows() {
            for j in 0..3 {
                assert!((wrapped[[i, j]] - expected[[i, j]]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_wrap_multiple_images() {
        let b = Box::new(array![
            [2.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 2.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let pts = array![[10.0, -5.0, -5.0], [0.0, 0.5, 0.0]];
        let wrapped = b.wrap(pts.view());
        let expected = array![[1.0, 1.0, 1.0], [2.0, 0.5, 0.0]];
        for i in 0..wrapped.nrows() {
            for j in 0..3 {
                assert!((wrapped[[i, j]] - expected[[i, j]]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_wrap() {
        let b = Box::new(array![
            [2.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 2.0],
        ], array![0.0, 0.0, 0.0], [true, true, true]);
        let pts = array![[10.0, -5.0, -5.0], [0.0, 0.5, 0.0]];
        let wrapped = b.wrap(pts.view());
        let expected = array![[1.0, 1.0, 1.0], [2.0, 0.5, 0.0]];
        for i in 0..wrapped.nrows() {
            for j in 0..3 {
                assert!((wrapped[[i, j]] - expected[[i, j]]).abs() < 1e-6);
            }
        }
    }

}
