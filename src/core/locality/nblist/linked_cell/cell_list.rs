use ndarray::{Array1, Array2};
use super::grid::Grid;

#[derive(Debug, Clone)]
pub struct CellList {
    pub offsets: Array1<u32>,
    pub indices: Array1<u32>,
}

impl CellList {
    /// Build a cell list for points (N×3, row-major) using the Grid.
    /// Offsets has length num_cells + 1, indices has length N.
    pub fn build(points: &Array2<f32>, grid: &Grid) -> Self {
        assert_eq!(points.ncols(), 3, "points must have shape (N,3)");
        let n = points.nrows();
        let num_cells = grid.n_cells();
        let mut counts = vec![0usize; num_cells];

        // First pass: counts
        for i in 0..n {
            let r = [points[[i,0]], points[[i,1]], points[[i,2]]];
            let idx = grid.index_of(&r);
            let c = grid.lin(&idx);
            counts[c] += 1;
        }

        // Prefix sum -> offsets
    let mut offsets_usize = vec![0usize; num_cells + 1];
    for c in 0..num_cells { offsets_usize[c + 1] = offsets_usize[c] + counts[c]; }

        // Second pass: fill indices using a write pointer per cell
    let mut write_ptrs = offsets_usize.clone();
    let mut indices_vec = vec![0u32; n];
        for i in 0..n {
            let r = [points[[i,0]], points[[i,1]], points[[i,2]]];
            let idx = grid.index_of(&r);
            let c = grid.lin(&idx);
            let w = write_ptrs[c];
            indices_vec[w] = i as u32;
            write_ptrs[c] += 1;
        }

        let offsets = Array1::from_vec(offsets_usize.into_iter().map(|x| x as u32).collect());
        let indices = Array1::from_vec(indices_vec);
        Self { offsets, indices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array1, array};
    use crate::core::region::r#box::Box;

    #[test]
    fn cell_list_basic() {
        let bx = Box::cube(4.0, Array1::zeros(3), [true, true, true]);
        let pts = array![[0.1, 0.2, 0.3],
            [1.9, 1.8, 1.7],
            [2.1, 2.2, 2.3],
            [3.9, 3.8, 3.7]];
        let g = Grid::from_cutoff(&bx, 2.0);
        let cl = CellList::build(&pts, &g);
        assert_eq!(cl.indices.len(), 4);
        assert_eq!(cl.offsets.len(), g.n_cells() + 1);
        let sum_counts: usize = (0..g.n_cells())
            .map(|c| (cl.offsets[c + 1] - cl.offsets[c]) as usize)
            .sum();
        assert_eq!(sum_counts, 4);
    }
}
