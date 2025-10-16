use ndarray::{Array2, array};
use crate::core::region::r#box::Box;

use super::cell_list::CellList;
use super::grid::Grid;

#[derive(Debug, Clone)]
pub struct LinkedCell {
    pub grid: Grid,
    pub cells: CellList,
    pub bx: Box,
}

impl LinkedCell {
    pub fn build(points: &Array2<f32>, cutoff: f32, bx: &Box) -> Self {
        let grid = Grid::from_cutoff(bx, cutoff);
        let cells = CellList::build(points, &grid);
        Self { grid, cells, bx: bx.clone() }
    }

    /// Iterate unique pairs (i<j) within cutoff using minimum image convention.
    pub fn pairs(&self, points: &Array2<f32>, cutoff: f32) -> Vec<(usize, usize)> {
        assert_eq!(points.ncols(), 3, "points must have shape (N,3)");
        let cutoff2 = cutoff * cutoff;
        let dims = self.grid.dims;
        let strides = self.grid.strides;
        let num_cells = dims[0] * dims[1] * dims[2];
        let mut out = Vec::new();

        for c0 in 0..num_cells {
            // decode c0 -> (i,j,k) in row-major
            let k0 = c0 / strides[2];
            let rem2 = c0 % strides[2];
            let j0 = rem2 / strides[1];
            let i0 = (rem2 % strides[1]) / strides[0];
            let idx0 = [i0 as i32, j0 as i32, k0 as i32];

            for nb in 0..self.grid.nb_half.nrows() {
                let dx = self.grid.nb_half[[nb, 0]];
                let dy = self.grid.nb_half[[nb, 1]];
                let dz = self.grid.nb_half[[nb, 2]];
                let idx_nb = [idx0[0] + dx, idx0[1] + dy, idx0[2] + dz];

                // wrap neighbor index into 0..dims using Euclidean modulo then linearize
                let wrap = |i: i32, m: usize| -> usize {
                    let m_i = m as i32;
                    let mut v = i % m_i;
                    if v < 0 { v += m_i; }
                    v as usize
                };
                let cnb = self.grid.lin(&[
                    wrap(idx_nb[0], dims[0]),
                    wrap(idx_nb[1], dims[1]),
                    wrap(idx_nb[2], dims[2]),
                ]);

                let lo0 = self.cells.offsets[c0] as usize;
                let hi0 = self.cells.offsets[c0 + 1] as usize;
                let lon = self.cells.offsets[cnb] as usize;
                let hin = self.cells.offsets[cnb + 1] as usize;

                if dx == 0 && dy == 0 && dz == 0 {
                    // same cell: i<j pairs
                    for a in lo0..hi0 {
                        let ia = self.cells.indices[a] as usize;
                        for b in a + 1..hi0 {
                            let ib = self.cells.indices[b] as usize;
                            if self.within_cutoff(points, ia, ib, cutoff2) {
                                out.push((ia, ib));
                            }
                        }
                    }
                } else {
                    for a in lo0..hi0 {
                        let ia = self.cells.indices[a] as usize;
                        for b in lon..hin {
                            let ib = self.cells.indices[b] as usize;
                            if self.within_cutoff(points, ia, ib, cutoff2) {
                                let (i, j) = if ia < ib { (ia, ib) } else { (ib, ia) };
                                out.push((i, j));
                            }
                        }
                    }
                }
            }
        }

        out
    }

    #[inline]
    fn within_cutoff(&self, points: &Array2<f32>, i: usize, j: usize, cutoff2: f32) -> bool {
        let p1 = array![[points[[i,0]], points[[i,1]], points[[i,2]]]]; // 1x3
        let p2 = array![[points[[j,0]], points[[j,1]], points[[j,2]]]]; // 1x3
        let d = self.bx.delta(p1.view(), p2.view(), true);
        let dx = d[[0,0]]; let dy = d[[0,1]]; let dz = d[[0,2]];
        (dx*dx + dy*dy + dz*dz) <= cutoff2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    use crate::core::region::r#box::Box;

    #[test]
    fn linked_cell_pairs_match_bruteforce() {
        let bx = Box::cube(5.0, array![0.0,0.0,0.0], [true, true, true]);
        let pts = array![[0.1, 0.1, 0.1],
            [1.0, 1.0, 1.0],
            [2.5, 2.5, 2.5],
            [4.9, 4.9, 4.9],
            [0.2, 4.8, 0.2]];
        let cutoff = 1.5;
        let lc = LinkedCell::build(&pts, cutoff, &bx);
        let mut pairs = lc.pairs(&pts, cutoff);
        pairs.sort();

        // brute-force with MIC
        let mut bf = Vec::new();
        for i in 0..pts.nrows() {
            for j in i + 1..pts.nrows() {
                let p1 = array![[pts[[i,0]], pts[[i,1]], pts[[i,2]]]];
                let p2 = array![[pts[[j,0]], pts[[j,1]], pts[[j,2]]]];
                let d = bx.delta(p1.view(), p2.view(), true);
                let d2 = d[[0,0]]*d[[0,0]] + d[[0,1]]*d[[0,1]] + d[[0,2]]*d[[0,2]];
                if d2 <= cutoff*cutoff { bf.push((i, j)); }
            }
        }
        bf.sort();
        assert_eq!(pairs, bf);
    }
}
