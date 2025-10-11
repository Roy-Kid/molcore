//! Triclinic simulation box and periodic operations (polars-based).
//!
//! Conventions (fractional/cartesian):
//! - cart = origin + H * frac
//! - frac = H^{-1} * (cart - origin)
//! - Lattice vectors are the columns of H.

use nalgebra::{Matrix3, Point3};
use polars::prelude::*;
use super::region::{Point3f, Vector3f, F};

/// Result type for coordinate operations returning (x, y, z) Series
pub type Coords3 = (Series, Series, Series);

/// Simulation box: triclinic cell with origin and per-axis PBC mask
#[derive(Debug, Clone)]
pub struct Box {
    /// Triclinic cell matrix H (columns are lattice vectors)
    pub h: Matrix3<F>,
    /// Origin of the cell in Cartesian coordinates
    pub origin: Point3f,
    /// Per-axis periodic boundary condition flags (x, y, z)
    pub pbc: [bool; 3],
}

impl Box {
    /// Construct from triclinic cell matrix `H`, origin `O`, and per-axis PBC flags
    pub fn new(h: Matrix3<F>, origin: Point3f, pbc: [bool; 3]) -> Self {
        Self { h, origin, pbc }
    }

    /// Factory: cubic box with edge length `a` and origin `O`
    pub fn cube(a: F, origin: Point3f, pbc: [bool; 3]) -> Self {
        let h = Matrix3::new(a, 0.0, 0.0,
                              0.0, a, 0.0,
                              0.0, 0.0, a);
        Self::new(h, origin, pbc)
    }

    /// Factory: ortho box with lengths (ax, ay, az) and origin `O`
    pub fn ortho(lengths: Vector3f, origin: Point3f, pbc: [bool; 3]) -> Self {
        let h = Matrix3::new(lengths.x, 0.0, 0.0,
                              0.0, lengths.y, 0.0,
                              0.0, 0.0, lengths.z);
        Self::new(h, origin, pbc)
    }

    /// Cell volume (|det(H)|)
    pub fn volume(&self) -> F { self.h.determinant().abs() }

    /// Return lattice vector by index (0,1,2) as a Vec3 (columns of H)
    pub fn lattice_vector(&self, index: usize) -> Vector3f {
        assert!(index < 3, "lattice_vector index must be 0..2");
        self.h.column(index).into()
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
        let inv = self.h.try_inverse().expect("cell matrix is singular");
        let r: Vector3f = cart - self.origin;
        inv * r
    }

    /// Convert a single fractional coordinate to Cartesian
    pub fn to_cart_single(&self, frac: Vector3f) -> Point3f {
        let r = self.h * frac;
        self.origin + r
    }

    /// Convert Cartesian coordinates to scaled (fractional) coordinates.
    /// LAMMPS convention: x,y,z (Cartesian) -> xs,ys,zs (scaled/fractional)
    /// 
    /// # Arguments
    /// * `x`, `y`, `z` - Polars Series containing Cartesian coordinates
    /// 
    /// # Returns
    /// Tuple of three Series containing scaled coordinates (xs, ys, zs)
    pub fn to_frac(&self, x: &Series, y: &Series, z: &Series) -> Coords3 {
        let n = x.len();
        assert_eq!(y.len(), n, "to_frac: y.len() must equal x.len()");
        assert_eq!(z.len(), n, "to_frac: z.len() must equal x.len()");
        
        let inv = self.h.try_inverse().expect("cell matrix is singular");
        
        let x_vals = x.f32().expect("x must be f32");
        let y_vals = y.f32().expect("y must be f32");
        let z_vals = z.f32().expect("z must be f32");
        
        let mut xs_vec = Vec::with_capacity(n);
        let mut ys_vec = Vec::with_capacity(n);
        let mut zs_vec = Vec::with_capacity(n);
        
        for i in 0..n {
            let px = x_vals.get(i).unwrap_or(0.0);
            let py = y_vals.get(i).unwrap_or(0.0);
            let pz = z_vals.get(i).unwrap_or(0.0);
            
            let v = Vector3f::new(px - self.origin.x, py - self.origin.y, pz - self.origin.z);
            let f = inv * v;
            
            xs_vec.push(f.x);
            ys_vec.push(f.y);
            zs_vec.push(f.z);
        }
        
        (
            Series::new("xs".into(), xs_vec),
            Series::new("ys".into(), ys_vec),
            Series::new("zs".into(), zs_vec),
        )
    }

    /// Convert scaled (fractional) coordinates to Cartesian coordinates.
    /// LAMMPS convention: xs,ys,zs (scaled/fractional) -> x,y,z (Cartesian)
    /// 
    /// # Arguments
    /// * `xs`, `ys`, `zs` - Polars Series containing scaled coordinates
    /// 
    /// # Returns
    /// Tuple of three Series containing Cartesian coordinates (x, y, z)
    pub fn to_cart(&self, xs: &Series, ys: &Series, zs: &Series) -> Coords3 {
        let n = xs.len();
        assert_eq!(ys.len(), n, "to_cart: ys.len() must equal xs.len()");
        assert_eq!(zs.len(), n, "to_cart: zs.len() must equal xs.len()");
        
        let xs_vals = xs.f32().expect("xs must be f32");
        let ys_vals = ys.f32().expect("ys must be f32");
        let zs_vals = zs.f32().expect("zs must be f32");
        
        let mut x_vec = Vec::with_capacity(n);
        let mut y_vec = Vec::with_capacity(n);
        let mut z_vec = Vec::with_capacity(n);
        
        for i in 0..n {
            let xs_val = xs_vals.get(i).unwrap_or(0.0);
            let ys_val = ys_vals.get(i).unwrap_or(0.0);
            let zs_val = zs_vals.get(i).unwrap_or(0.0);
            
            let frac = Vector3f::new(xs_val, ys_val, zs_val);
            let cart = self.origin + self.h * frac;
            
            x_vec.push(cart.x);
            y_vec.push(cart.y);
            z_vec.push(cart.z);
        }
        
        (
            Series::new("x".into(), x_vec),
            Series::new("y".into(), y_vec),
            Series::new("z".into(), z_vec),
        )
    }

    /// Check whether points are inside primary cell (0 <= scaled < 1 per periodic axis).
    /// 
    /// # Arguments
    /// * `x`, `y`, `z` - Polars Series containing Cartesian coordinates
    /// 
    /// # Returns
    /// Boolean Series indicating which points are inside the cell
    pub fn isin(&self, x: &Series, y: &Series, z: &Series) -> Series {
        let (xs, ys, zs) = self.to_frac(x, y, z);
        let n = xs.len();
        
        let xs_vals = xs.f32().expect("xs must be f32");
        let ys_vals = ys.f32().expect("ys must be f32");
        let zs_vals = zs.f32().expect("zs must be f32");
        
        let mut result = Vec::with_capacity(n);
        
        for i in 0..n {
            let xs_val = xs_vals.get(i).unwrap_or(0.0);
            let ys_val = ys_vals.get(i).unwrap_or(0.0);
            let zs_val = zs_vals.get(i).unwrap_or(0.0);

            let okx = in01(self.pbc[0], xs_val);
            let oky = in01(self.pbc[1], ys_val);
            let okz = in01(self.pbc[2], zs_val);

            result.push(okx && oky && okz);
        }
        
        Series::new("isin".into(), result)
    }



    /// Displacement vector d = b - a in Cartesian, with optional minimum image in PBC.
    pub fn delta_vec(&self, a: Point3f, b: Point3f, minimum_image: bool) -> Vector3f {
        if !minimum_image {
            return b - a;
        }
        let inv = self.h.try_inverse().expect("cell matrix is singular");
        let fa = inv * (a - self.origin);
        let fb = inv * (b - self.origin);
        let mut df = fb - fa;
        if self.pbc[0] { df.x = wrap_mi(df.x); }
        if self.pbc[1] { df.y = wrap_mi(df.y); }
        if self.pbc[2] { df.z = wrap_mi(df.z); }
        self.h * df
    }

    /// Compute displacement vectors between two sets of points.
    /// LAMMPS convention: uses unwrapped coordinates (xu, yu, zu) for input
    /// 
    /// # Arguments
    /// * `xu1`, `yu1`, `zu1` - Polars Series containing first set of unwrapped Cartesian coordinates
    /// * `xu2`, `yu2`, `zu2` - Polars Series containing second set of unwrapped Cartesian coordinates
    /// * `minimum_image` - If true, apply minimum image convention for periodic boundaries
    /// 
    /// # Returns
    /// Tuple of three Series containing displacement vectors (dx, dy, dz) where d = point2 - point1
    pub fn delta(&self, xu1: &Series, yu1: &Series, zu1: &Series, 
                 xu2: &Series, yu2: &Series, zu2: &Series, 
                 minimum_image: bool) -> Coords3 {
        let n = xu1.len();
        assert_eq!(yu1.len(), n, "delta: yu1.len() must equal xu1.len()");
        assert_eq!(zu1.len(), n, "delta: zu1.len() must equal xu1.len()");
        assert_eq!(xu2.len(), n, "delta: xu2.len() must equal xu1.len()");
        assert_eq!(yu2.len(), n, "delta: yu2.len() must equal xu1.len()");
        assert_eq!(zu2.len(), n, "delta: zu2.len() must equal xu1.len()");
        
        let xu1_vals = xu1.f32().expect("xu1 must be f32");
        let yu1_vals = yu1.f32().expect("yu1 must be f32");
        let zu1_vals = zu1.f32().expect("zu1 must be f32");
        let xu2_vals = xu2.f32().expect("xu2 must be f32");
        let yu2_vals = yu2.f32().expect("yu2 must be f32");
        let zu2_vals = zu2.f32().expect("zu2 must be f32");
        
        let mut dx_vec = Vec::with_capacity(n);
        let mut dy_vec = Vec::with_capacity(n);
        let mut dz_vec = Vec::with_capacity(n);
        
        for i in 0..n {
            let p1 = Point3::new(
                xu1_vals.get(i).unwrap_or(0.0),
                yu1_vals.get(i).unwrap_or(0.0),
                zu1_vals.get(i).unwrap_or(0.0),
            );
            let p2 = Point3::new(
                xu2_vals.get(i).unwrap_or(0.0),
                yu2_vals.get(i).unwrap_or(0.0),
                zu2_vals.get(i).unwrap_or(0.0),
            );
            
            let d = self.delta_vec(p1, p2, minimum_image);
            dx_vec.push(d.x);
            dy_vec.push(d.y);
            dz_vec.push(d.z);
        }
        
        (
            Series::new("dx".into(), dx_vec),
            Series::new("dy".into(), dy_vec),
            Series::new("dz".into(), dz_vec),
        )
    }

    /// Wrap unwrapped coordinates into the primary cell (0 <= scaled < 1 for periodic axes).
    /// LAMMPS convention: xu,yu,zu (unwrapped) -> x,y,z (wrapped)
    /// 
    /// # Arguments
    /// * `xu`, `yu`, `zu` - Polars Series containing unwrapped Cartesian coordinates
    /// 
    /// # Returns
    /// Tuple of three Series containing wrapped Cartesian coordinates (x, y, z)
    pub fn wrap(&self, xu: &Series, yu: &Series, zu: &Series) -> Coords3 {
        let (xs, ys, zs) = self.to_frac(xu, yu, zu);
        let n = xs.len();
        
        let xs_vals = xs.f32().expect("xs must be f32");
        let ys_vals = ys.f32().expect("ys must be f32");
        let zs_vals = zs.f32().expect("zs must be f32");
        
        let mut xs_wrapped = Vec::with_capacity(n);
        let mut ys_wrapped = Vec::with_capacity(n);
        let mut zs_wrapped = Vec::with_capacity(n);
        
        for i in 0..n {
            let mut xs_val = xs_vals.get(i).unwrap_or(0.0);
            let mut ys_val = ys_vals.get(i).unwrap_or(0.0);
            let mut zs_val = zs_vals.get(i).unwrap_or(0.0);
            
            if self.pbc[0] { xs_val = xs_val - xs_val.floor(); }
            if self.pbc[1] { ys_val = ys_val - ys_val.floor(); }
            if self.pbc[2] { zs_val = zs_val - zs_val.floor(); }
            
            xs_wrapped.push(xs_val);
            ys_wrapped.push(ys_val);
            zs_wrapped.push(zs_val);
        }
        
        let xs_series = Series::new("xs".into(), xs_wrapped);
        let ys_series = Series::new("ys".into(), ys_wrapped);
        let zs_series = Series::new("zs".into(), zs_wrapped);
        
        self.to_cart(&xs_series, &ys_series, &zs_series)
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

    #[test]
    fn roundtrip_frac_cart() {
        let bx = Box::ortho(
            Vector3f::new(2.0, 3.0, 4.0), 
            Point3f::new(0.5, -1.0, 2.0), 
            [true, true, true]
        );
        
        let x = Series::new("x".into(), vec![0.5_f32, 2.5_f32]);
        let y = Series::new("y".into(), vec![-1.0_f32, 2.0_f32]);
        let z = Series::new("z".into(), vec![2.0_f32, 6.0_f32]);
        
        let (xs, ys, zs) = bx.to_frac(&x, &y, &z);
        let (x2, y2, z2) = bx.to_cart(&xs, &ys, &zs);
        
        let x_vals = x.f32().unwrap();
        let x2_vals = x2.f32().unwrap();
        let y_vals = y.f32().unwrap();
        let y2_vals = y2.f32().unwrap();
        let z_vals = z.f32().unwrap();
        let z2_vals = z2.f32().unwrap();
        
        for i in 0..x.len() {
            assert!((x_vals.get(i).unwrap() - x2_vals.get(i).unwrap()).abs() < 1e-5);
            assert!((y_vals.get(i).unwrap() - y2_vals.get(i).unwrap()).abs() < 1e-5);
            assert!((z_vals.get(i).unwrap() - z2_vals.get(i).unwrap()).abs() < 1e-5);
        }
    }

    #[test]
    fn wrap_into_cell() {
        let bx = Box::cube(2.0, Point3f::origin(), [true, true, true]);
        
        let x = Series::new("x".into(), vec![2.1_f32, -1.9_f32]);
        let y = Series::new("y".into(), vec![-0.1_f32, 4.2_f32]);
        let z = Series::new("z".into(), vec![3.9_f32, 0.0_f32]);
        
        let (wx, wy, wz) = bx.wrap(&x, &y, &z);
        let (xs, ys, zs) = bx.to_frac(&wx, &wy, &wz);
        
        let fx_vals = xs.f32().unwrap();
        let fy_vals = ys.f32().unwrap();
        let fz_vals = zs.f32().unwrap();
        
        for i in 0..xs.len() {
            let fx_val = fx_vals.get(i).unwrap();
            let fy_val = fy_vals.get(i).unwrap();
            let fz_val = fz_vals.get(i).unwrap();
            assert!(fx_val >= 0.0 && fx_val < 1.0, "xs[{}] = {} not in [0, 1)", i, fx_val);
            assert!(fy_val >= 0.0 && fy_val < 1.0, "ys[{}] = {} not in [0, 1)", i, fy_val);
            assert!(fz_val >= 0.0 && fz_val < 1.0, "zs[{}] = {} not in [0, 1)", i, fz_val);
        }
    }

    #[test]
    fn minimum_image_delta() {
        let my_box = Box::ortho(
            Vector3f::new(10.0, 10.0, 10.0), 
            Point3f::origin(), 
            [true, true, true]
        );
        
        let ax = Series::new("ax".into(), vec![9.0_f32]);
        let ay = Series::new("ay".into(), vec![9.0_f32]);
        let az = Series::new("az".into(), vec![9.0_f32]);
        let bx = Series::new("bx".into(), vec![1.0_f32]);
        let by = Series::new("by".into(), vec![1.0_f32]);
        let bz = Series::new("bz".into(), vec![1.0_f32]);
        
        let (dx, dy, dz) = my_box.delta(&ax, &ay, &az, &bx, &by, &bz, true);
        
        let dx_val = dx.f32().unwrap().get(0).unwrap();
        let dy_val = dy.f32().unwrap().get(0).unwrap();
        let dz_val = dz.f32().unwrap().get(0).unwrap();
        
        // Minimum image should take the shorter path: (-8,-8,-8) -> (+2,+2,+2)
        assert!((dx_val.abs() - 2.0).abs() < 1e-5, "dx = {}", dx_val);
        assert!((dy_val.abs() - 2.0).abs() < 1e-5, "dy = {}", dy_val);
        assert!((dz_val.abs() - 2.0).abs() < 1e-5, "dz = {}", dz_val);
    }
    
    #[test]
    fn is_in_test() {
        let bx = Box::cube(2.0, Point3f::origin(), [true, true, true]);
        
        let x = Series::new("x".into(), vec![1.0_f32, 2.5_f32, -0.5_f32]);
        let y = Series::new("y".into(), vec![1.0_f32, 1.0_f32, 1.0_f32]);
        let z = Series::new("z".into(), vec![1.0_f32, 1.0_f32, 1.0_f32]);
        
        let result = bx.isin(&x, &y, &z);
        let result_vals = result.bool().unwrap();
        
        assert_eq!(result_vals.get(0), Some(true));  // (1.0, 1.0, 1.0) is inside
        assert_eq!(result_vals.get(1), Some(false)); // (2.5, 1.0, 1.0) is outside
        assert_eq!(result_vals.get(2), Some(false)); // (-0.5, 1.0, 1.0) is outside
    }
}
