use ndarray::{array, Array1, Array2};
use crate::core::region::r#box::Box;

/// Half-stencil for 3D 26-neighborhood (including self cell), unique pairs
/// relative to a base cell. Shape: [14, 3].
#[inline]
fn half_stencil() -> Array2<i32> { array![
    [ 0,  0,  0],
    [ 0,  0,  1],
    [ 0,  1, -1],
    [ 0,  1,  0],
    [ 0,  1,  1],
    [ 1, -1, -1],
    [ 1, -1,  0],
    [ 1, -1,  1],
    [ 1,  0, -1],
    [ 1,  0,  0],
    [ 1,  0,  1],
    [ 1,  1, -1],
    [ 1,  1,  0],
    [ 1,  1,  1],
] }

#[derive(Debug, Clone)]
pub struct Grid {
    pub origin: Array1<f32>,
    pub lengths: Array1<f32>,
    pub dims: [usize; 3],
    pub cell: [f32; 3],
    pub strides: [usize; 3],
    pub nb_half: Array2<i32>,
}

impl Grid {
    /// Build a grid from a simulation box and cutoff. Assumes orthorhombic lengths
    /// taken from lattice vector norms; works reasonably for typical boxes.
    pub fn from_cutoff(bx: &Box, cutoff: f32) -> Self {
        assert!(cutoff > 0.0, "cutoff must be positive");
        // Get axis lengths as 2× nearest-plane distances (|a|,|b|,|c|)
        let half = bx.nearest_plane_distance();
        let lengths = array![2.0 * half[0], 2.0 * half[1], 2.0 * half[2]];

        let mut dims = [0usize; 3];
        let mut cell = [0f32; 3];
        for d in 0..3 {
            let n = (lengths[d] / cutoff).floor() as usize;
            dims[d] = n.max(1);
            cell[d] = lengths[d] / dims[d] as f32;
        }

        let strides = [1, dims[0], dims[0] * dims[1]];
        let nb_half = half_stencil();

        Self {
            origin: bx.origin.clone(),
            lengths,
            dims,
            cell,
            strides,
            nb_half,
        }
    }

    /// Compute the 3D integer cell index for a Cartesian coordinate r (clamped to domain).
    #[inline]
    pub fn index_of(&self, r: &[f32; 3]) -> [usize; 3] {
        let mut out = [0usize; 3];
        for d in 0..3 {
            let mut i = (((r[d] - self.origin[d]) / self.cell[d]).floor() as isize).max(0);
            let max_i = (self.dims[d] as isize) - 1;
            if i > max_i { i = max_i; }
            out[d] = i as usize;
        }
        out
    }

    /// Linearize a 3D cell index using row-major layout with Euclidean modulo per axis (PBC).
    #[inline]
    pub fn lin(&self, idx: &[usize; 3]) -> usize {
        let mut acc = 0usize;
        for d in 0..3 {
            let m = self.dims[d];
            let ik = idx[d] % m;
            acc += ik * self.strides[d];
        }
        acc
    }

    #[inline]
    pub fn n_cells(&self) -> usize { self.dims[0] * self.dims[1] * self.dims[2] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::region::r#box::Box;
    use ndarray::Array1;

    #[test]
    fn grid_from_cutoff_ortho() {
        let bx = Box::cube(10.0, Array1::zeros(3), [true, true, true]);
        let g = Grid::from_cutoff(&bx, 3.9);
        // 10/3.9 -> floor=2 per axis
        assert_eq!(g.dims, [2, 2, 2]);
        assert_eq!(g.n_cells(), 8);
        // origin at box origin
        assert!((g.origin[0] - 0.0).abs() < 1e-6);
    }
}
