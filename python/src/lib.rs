use molcore::core::array::{Array, Mat3, NdArray, Vec3 as Vector3f};
use molcore::core::region::r#box::Box as RustBox;
use numpy::{
    PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray1, PyReadonlyArray2, PyUntypedArrayMethods,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Helper: Borrow NumPy (N×3) row-major array as NdArray (N×3) of f32 without copying
#[inline]
fn numpy_to_ndarray3<'py>(arr: PyReadonlyArray2<'py, f32>) -> PyResult<NdArray<f32>> {
    let shape = arr.shape();
    if shape.len() != 2 || shape[1] != 3 {
        return Err(PyValueError::new_err("expected shape (N,3)"));
    }
    // This ensures C-contiguous; returns an error otherwise
    let slice = arr.as_slice()?;
    let n = shape[0];
    let len = slice.len();
    // Safety: underlying NumPy memory stays alive as long as the Python array lives; we'll use it immediately.
    let nd = unsafe { NdArray::from_ptr(vec![n, 3], slice.as_ptr(), len, None) };
    Ok(nd)
}

/// Helper: Convert NdArray (N×3) to NumPy (N×3) row-major array; fills existing array slice in-place
#[inline]
fn ndarray3_to_numpy(arr: &NdArray<f32>, out_slice: &mut [f32]) {
    let shape = arr.shape();
    assert!(shape.len() == 2 && shape[1] == 3, "expected (N,3)");
    let n = shape[0];
    for i in 0..n {
        let b = i * 3;
        out_slice[b + 0] = arr.data()[b + 0];
        out_slice[b + 1] = arr.data()[b + 1];
        out_slice[b + 2] = arr.data()[b + 2];
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
        if h_slice.len() != 9 {
            return Err(PyValueError::new_err("h must be 3x3"));
        }
        let h_matrix = Mat3::from_rows(
            [h_slice[0], h_slice[1], h_slice[2]],
            [h_slice[3], h_slice[4], h_slice[5]],
            [h_slice[6], h_slice[7], h_slice[8]],
        );

        let origin_vec = if let Some(o) = origin {
            let o_slice = o.as_slice()?;
            if o_slice.len() != 3 {
                return Err(PyValueError::new_err("origin must have 3 elements"));
            }
            Vector3f::new(o_slice[0], o_slice[1], o_slice[2])
        } else {
            Vector3f::origin()
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
            Vector3f::new(o_slice[0], o_slice[1], o_slice[2])
        } else {
            Vector3f::origin()
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
        let lengths_vec = Vector3f::new(lengths_slice[0], lengths_slice[1], lengths_slice[2]);

        let origin_vec = if let Some(o) = origin {
            let o_slice = o.as_slice()?;
            if o_slice.len() != 3 {
                return Err(PyValueError::new_err("origin must have 3 elements"));
            }
            Vector3f::new(o_slice[0], o_slice[1], o_slice[2])
        } else {
            Vector3f::origin()
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
    fn lattice_vector<'py>(
        &self,
        py: Python<'py>,
        index: usize,
    ) -> PyResult<Bound<'py, PyArray1<f32>>> {
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
    fn to_frac<'py>(
        &self,
        py: Python<'py>,
        xyz: PyReadonlyArray2<f32>,
    ) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = xyz.shape();
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyz must have shape (N, 3)"));
        }

        let n = shape[0];
        // Convert NumPy → NdArray (zero-copy borrowed view)
        let arr = numpy_to_ndarray3(xyz)?;
        let frac = self.inner.to_frac_points(&arr);
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let out = result.as_slice_mut()?;
            ndarray3_to_numpy(&frac, out);
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
    fn to_cart<'py>(
        &self,
        py: Python<'py>,
        xyzs: PyReadonlyArray2<f32>,
    ) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = xyzs.shape();
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyzs must have shape (N, 3)"));
        }

        let n = shape[0];
        // Convert NumPy → NdArray (zero-copy borrowed view)
        let arr = numpy_to_ndarray3(xyzs)?;
        let cart = self.inner.to_cart_points(&arr);
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let out = result.as_slice_mut()?;
            ndarray3_to_numpy(&cart, out);
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
    fn wrap<'py>(
        &self,
        py: Python<'py>,
        xyzu: PyReadonlyArray2<f32>,
    ) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = xyzu.shape();
        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyzu must have shape (N, 3)"));
        }

        let n = shape[0];
        // Convert NumPy → NdArray (zero-copy borrowed view)
        let arr = numpy_to_ndarray3(xyzu)?;
        let wrapped = self.inner.wrap_points(&arr);
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let out = result.as_slice_mut()?;
            ndarray3_to_numpy(&wrapped, out);
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
            return Err(PyValueError::new_err(
                "xyzu1 and xyzu2 must have the same shape",
            ));
        }

        if shape1[1] != 3 {
            return Err(PyValueError::new_err("arrays must have shape (N, 3)"));
        }

        let n = shape1[0];
        // Convert NumPy → NdArray (zero-copy borrowed view)
        let arr1 = numpy_to_ndarray3(xyzu1)?;
        let arr2 = numpy_to_ndarray3(xyzu2)?;
        let d = self.inner.delta_points(&arr1, &arr2, minimum_image);
        let result = PyArray2::zeros_bound(py, [n, 3], false);
        unsafe {
            let out = result.as_slice_mut()?;
            ndarray3_to_numpy(&d, out);
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
    fn isin<'py>(
        &self,
        py: Python<'py>,
        xyz: PyReadonlyArray2<f32>,
    ) -> PyResult<Bound<'py, PyArray1<bool>>> {
        let shape = xyz.shape();

        if shape[1] != 3 {
            return Err(PyValueError::new_err("xyz must have shape (N, 3)"));
        }

        let n = shape[0];
        // Convert NumPy → NdArray (zero-copy borrowed view)
        let arr = numpy_to_ndarray3(xyz)?;
        let inside = self.inner.isin_points(&arr);
        let mut result_vec = Vec::with_capacity(n);
        for i in 0..n {
            result_vec.push(inside[[i]]);
        }
        Ok(PyArray1::from_vec_bound(py, result_vec))
    }

    fn __repr__(&self) -> String {
        format!("Box(volume={:.2})", self.volume())
    }
}

/// Python module definition
#[pymodule]
fn molrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBox>()?;
    Ok(())
}
