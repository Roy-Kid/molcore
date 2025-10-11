use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use numpy::{PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2, PyArrayMethods, PyUntypedArrayMethods};
use nalgebra as na;
use molcore::core::region::r#box::Box as RustBox;
use polars::prelude::*;

/// Helper: Convert NumPy (N×3) row-major array to 3 Polars Series (zero-copy where possible)
/// Returns (x, y, z) Series extracted from columns
#[inline]
fn numpy_to_series3(slice: &[f32], n: usize) -> (Series, Series, Series) {
    let mut x_vec = Vec::with_capacity(n);
    let mut y_vec = Vec::with_capacity(n);
    let mut z_vec = Vec::with_capacity(n);
    
    for i in 0..n {
        unsafe {
            // Use unsafe get for performance (bounds already checked)
            x_vec.push(*slice.get_unchecked(i * 3));
            y_vec.push(*slice.get_unchecked(i * 3 + 1));
            z_vec.push(*slice.get_unchecked(i * 3 + 2));
        }
    }
    
    (
        Series::new("x".into(), x_vec),
        Series::new("y".into(), y_vec),
        Series::new("z".into(), z_vec),
    )
}

/// Helper: Convert 3 Polars Series to NumPy (N×3) row-major array
/// Fills existing array slice in-place
#[inline]
fn series3_to_numpy(xs: &Series, ys: &Series, zs: &Series, out_slice: &mut [f32]) {
    let n = xs.len();
    let xs_ca = xs.f32().unwrap();
    let ys_ca = ys.f32().unwrap();
    let zs_ca = zs.f32().unwrap();
    
    for i in 0..n {
        unsafe {
            // Use unsafe get for performance
            *out_slice.get_unchecked_mut(i * 3) = xs_ca.get_unchecked(i).unwrap();
            *out_slice.get_unchecked_mut(i * 3 + 1) = ys_ca.get_unchecked(i).unwrap();
            *out_slice.get_unchecked_mut(i * 3 + 2) = zs_ca.get_unchecked(i).unwrap();
        }
    }
}

/// Python wrapper for the Rust Box struct
#[pyclass(name = "Box")]
#[derive(Clone)]
pub struct PyBox {
    inner: RustBox,
}

#[pymethods]
impl PyBox {
    /// Create a new triclinic box
    /// 
    /// Parameters
    /// ----------
    /// h : array_like, shape (3, 3), dtype=float32
    ///     Cell matrix with lattice vectors as columns
    /// origin : array_like, shape (3,), dtype=float32, optional
    ///     Origin of the box in Cartesian coordinates
    /// pbc : array_like, shape (3,), optional
    ///     Periodic boundary conditions for x, y, z axes
    #[new]
    #[pyo3(signature = (h, origin=None, pbc=None))]
    fn new(
        h: PyReadonlyArray2<f32>,
        origin: Option<PyReadonlyArray1<f32>>,
        pbc: Option<PyReadonlyArray1<bool>>,
    ) -> PyResult<Self> {
        let h_shape = h.shape();
        if h_shape[0] != 3 || h_shape[1] != 3 {
            return Err(PyValueError::new_err("h must be a 3x3 matrix"));
        }
        
        let h_slice = h.as_slice()?;
        let h_matrix = na::Matrix3::from_column_slice(h_slice);
        
        let origin_vec = if let Some(o) = origin {
            let o_slice = o.as_slice()?;
            if o_slice.len() != 3 {
                return Err(PyValueError::new_err("origin must have 3 elements"));
            }
            na::Point3::new(o_slice[0], o_slice[1], o_slice[2])
        } else {
            na::Point3::origin()
        };
        
        let pbc_array = if let Some(p) = pbc {
            let p_slice = p.as_slice()?;
            if p_slice.len() != 3 {
                return Err(PyValueError::new_err("pbc must have 3 elements"));
            }
            [p_slice[0], p_slice[1], p_slice[2]]
        } else {
            [true, true, true]
        };
        
        Ok(PyBox {
            inner: RustBox::new(h_matrix, origin_vec, pbc_array),
        })
    }
    
    /// Create a cubic box
    /// 
    /// Parameters
    /// ----------
    /// a : float
    ///     Edge length of the cube
    /// origin : array_like, shape (3,), dtype=float32, optional
    ///     Origin of the box
    /// pbc : array_like, shape (3,), optional
    ///     Periodic boundary conditions
    #[staticmethod]
    #[pyo3(signature = (a, origin=None, pbc=None))]
    fn cube(
        a: f32,
        origin: Option<PyReadonlyArray1<f32>>,
        pbc: Option<PyReadonlyArray1<bool>>,
    ) -> PyResult<Self> {
        let origin_vec = if let Some(o) = origin {
            let o_slice = o.as_slice()?;
            if o_slice.len() != 3 {
                return Err(PyValueError::new_err("origin must have 3 elements"));
            }
            na::Point3::new(o_slice[0], o_slice[1], o_slice[2])
        } else {
            na::Point3::origin()
        };
        
        let pbc_array = if let Some(p) = pbc {
            let p_slice = p.as_slice()?;
            if p_slice.len() != 3 {
                return Err(PyValueError::new_err("pbc must have 3 elements"));
            }
            [p_slice[0], p_slice[1], p_slice[2]]
        } else {
            [true, true, true]
        };
        
        Ok(PyBox {
            inner: RustBox::cube(a, origin_vec, pbc_array),
        })
    }
    
    /// Create an orthorhombic box
    /// 
    /// Parameters
    /// ----------
    /// lengths : array_like, shape (3,), dtype=float32
    ///     Box lengths along x, y, z axes
    /// origin : array_like, shape (3,), dtype=float32, optional
    ///     Origin of the box
    /// pbc : array_like, shape (3,), optional
    ///     Periodic boundary conditions
    #[staticmethod]
    #[pyo3(signature = (lengths, origin=None, pbc=None))]
    fn ortho(
        lengths: PyReadonlyArray1<f32>,
        origin: Option<PyReadonlyArray1<f32>>,
        pbc: Option<PyReadonlyArray1<bool>>,
    ) -> PyResult<Self> {
        let lengths_slice = lengths.as_slice()?;
        if lengths_slice.len() != 3 {
            return Err(PyValueError::new_err("lengths must have 3 elements"));
        }
        let lengths_vec = na::Vector3::new(lengths_slice[0], lengths_slice[1], lengths_slice[2]);
        
        let origin_vec = if let Some(o) = origin {
            let o_slice = o.as_slice()?;
            if o_slice.len() != 3 {
                return Err(PyValueError::new_err("origin must have 3 elements"));
            }
            na::Point3::new(o_slice[0], o_slice[1], o_slice[2])
        } else {
            na::Point3::origin()
        };
        
        let pbc_array = if let Some(p) = pbc {
            let p_slice = p.as_slice()?;
            if p_slice.len() != 3 {
                return Err(PyValueError::new_err("pbc must have 3 elements"));
            }
            [p_slice[0], p_slice[1], p_slice[2]]
        } else {
            [true, true, true]
        };
        
        Ok(PyBox {
            inner: RustBox::ortho(lengths_vec, origin_vec, pbc_array),
        })
    }
    
    /// Get the volume of the box
    fn volume(&self) -> f32 {
        self.inner.volume()
    }
    
    /// Get a lattice vector
    /// 
    /// Parameters
    /// ----------
    /// index : int
    ///     Index of the lattice vector (0, 1, or 2)
    /// 
    /// Returns
    /// -------
    /// vector : ndarray, shape (3,), dtype=float32
    ///     Lattice vector
    fn lattice_vector<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyArray1<f32>>> {
        if index >= 3 {
            return Err(PyValueError::new_err("index must be 0, 1, or 2"));
        }
        let vec = self.inner.lattice_vector(index);
        Ok(PyArray1::from_vec_bound(py, vec![vec.x, vec.y, vec.z]))
    }
    
    /// Convert Cartesian coordinates to fractional coordinates
    /// LAMMPS convention: x,y,z -> xs,ys,zs
    /// 
    /// Parameters
    /// ----------
    /// xyz : array_like, shape (N, 3), dtype=float32
    ///     Cartesian coordinates (x, y, z columns)
    /// 
    /// Returns
    /// -------
    /// xyzs : ndarray, shape (N, 3), dtype=float32
    ///     Fractional coordinates (xs, ys, zs columns)
    fn to_frac<'py>(&self, py: Python<'py>, xyz: PyReadonlyArray2<f32>) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = xyz.shape();
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyz must have shape (N, 3)"));
        }
        
        let n = shape[0];
        let xyz_slice = xyz.as_slice()?;
        
        // Convert NumPy → Polars (optimized with helper)
        let (x, y, z) = numpy_to_series3(xyz_slice, n);
        
        // Call Rust method
        let (xs, ys, zs) = self.inner.to_frac(&x, &y, &z);
        
        // Convert Polars → NumPy (optimized with helper)
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let result_slice = result.as_slice_mut()?;
            series3_to_numpy(&xs, &ys, &zs, result_slice);
        }
        Ok(result)
    }
    
    /// Convert scaled (fractional) coordinates to Cartesian
    /// LAMMPS convention: xs,ys,zs -> x,y,z
    /// 
    /// Parameters
    /// ----------
    /// xyzs : array_like, shape (N, 3), dtype=float32
    ///     Scaled/fractional coordinates (xs, ys, zs columns)
    /// 
    /// Returns
    /// -------
    /// xyz : ndarray, shape (N, 3), dtype=float32
    ///     Cartesian coordinates (x, y, z columns)
    fn to_cart<'py>(&self, py: Python<'py>, xyzs: PyReadonlyArray2<f32>) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = xyzs.shape();
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyzs must have shape (N, 3)"));
        }
        
        let n = shape[0];
        let xyzs_slice = xyzs.as_slice()?;
        
        // Convert NumPy → Polars (optimized)
        let (xs, ys, zs) = numpy_to_series3(xyzs_slice, n);
        
        // Call Rust method
        let (x, y, z) = self.inner.to_cart(&xs, &ys, &zs);
        
        // Convert Polars → NumPy (optimized)
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let result_slice = result.as_slice_mut()?;
            series3_to_numpy(&x, &y, &z, result_slice);
        }
        Ok(result)
    }
    
    /// Wrap unwrapped coordinates into the primary cell
    /// LAMMPS convention: xu,yu,zu -> x,y,z
    /// 
    /// Parameters
    /// ----------
    /// xyzu : array_like, shape (N, 3), dtype=float32
    ///     Unwrapped Cartesian coordinates (xu, yu, zu columns)
    /// 
    /// Returns
    /// -------
    /// xyz : ndarray, shape (N, 3), dtype=float32
    ///     Wrapped Cartesian coordinates (x, y, z columns)
    fn wrap<'py>(&self, py: Python<'py>, xyzu: PyReadonlyArray2<f32>) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = xyzu.shape();
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyzu must have shape (N, 3)"));
        }
        
        let n = shape[0];
        let xyzu_slice = xyzu.as_slice()?;
        
        // Convert NumPy → Polars (optimized)
        let (xu, yu, zu) = numpy_to_series3(xyzu_slice, n);
        
        // Call Rust method
        let (x, y, z) = self.inner.wrap(&xu, &yu, &zu);
        
        // Convert Polars → NumPy (optimized)
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let result_slice = result.as_slice_mut()?;
            series3_to_numpy(&x, &y, &z, result_slice);
        }
        Ok(result)
    }
    
    /// Calculate displacement vectors with optional minimum image convention
    /// LAMMPS convention: uses unwrapped coordinates (xu, yu, zu)
    /// 
    /// Parameters
    /// ----------
    /// xyzu1 : array_like, shape (N, 3)
    ///     Starting unwrapped points (xu1, yu1, zu1 columns)
    /// xyzu2 : array_like, shape (N, 3)
    ///     Ending unwrapped points (xu2, yu2, zu2 columns)
    /// minimum_image : bool, optional
    ///     Whether to use minimum image convention (default: False)
    /// 
    /// Returns
    /// -------
    /// dxyz : ndarray, shape (N, 3)
    ///     Displacement vectors (dx, dy, dz columns)
    #[pyo3(signature = (xyzu1, xyzu2, minimum_image=false))]
    fn delta<'py>(
        &self,
        py: Python<'py>,
        xyzu1: PyReadonlyArray2<f32>,
        xyzu2: PyReadonlyArray2<f32>,
        minimum_image: bool,
    ) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape1 = xyzu1.shape();
        let shape2 = xyzu2.shape();
        
        if shape1 != shape2 {
            return Err(PyValueError::new_err("xyzu1 and xyzu2 must have the same shape"));
        }
        
        if shape1[1] != 3 {
            return Err(PyValueError::new_err("arrays must have shape (N, 3)"));
        }
        
        let n = shape1[0];
        let xyzu1_slice = xyzu1.as_slice()?;
        let xyzu2_slice = xyzu2.as_slice()?;
        
        // Convert NumPy → Polars (optimized)
        let (xu1, yu1, zu1) = numpy_to_series3(xyzu1_slice, n);
        let (xu2, yu2, zu2) = numpy_to_series3(xyzu2_slice, n);
        
        // Call Rust method
        let (dx, dy, dz) = self.inner.delta(&xu1, &yu1, &zu1, &xu2, &yu2, &zu2, minimum_image);
        
        // Convert Polars → NumPy (optimized)
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let result_slice = result.as_slice_mut()?;
            series3_to_numpy(&dx, &dy, &dz, result_slice);
        }
        Ok(result)
    }
    
    /// Check if points are inside the primary cell
    /// 
    /// Parameters
    /// ----------
    /// xyz : array_like, shape (N, 3), dtype=float32
    ///     Points in Cartesian coordinates (x, y, z columns)
    /// 
    /// Returns
    /// -------
    /// inside : ndarray, shape (N,), dtype=bool
    ///     Boolean array indicating if each point is inside
    fn isin<'py>(&self, py: Python<'py>, xyz: PyReadonlyArray2<f32>) -> PyResult<Bound<'py, PyArray1<bool>>> {
        let shape = xyz.shape();
        
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyz must have shape (N, 3)"));
        }
        
        let n = shape[0];
        let xyz_slice = xyz.as_slice()?;
        
        // Convert NumPy → Polars (optimized)
        let (x, y, z) = numpy_to_series3(xyz_slice, n);
        
        // Call Rust method (returns bool Series)
        let result_series = self.inner.isin(&x, &y, &z);
        let result_bools = result_series.bool().unwrap();
        
        // Convert to NumPy bool array (optimized)
        let mut result_vec = Vec::with_capacity(n);
        unsafe {
            for i in 0..n {
                result_vec.push(result_bools.get_unchecked(i).unwrap());
            }
        }
        
        Ok(PyArray1::from_vec_bound(py, result_vec))
    }
    
    fn __repr__(&self) -> String {
        format!("Box(volume={:.2})", self.volume())
    }
}

/// Python module definition
#[pymodule]
fn _molrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBox>()?;
    Ok(())
}
