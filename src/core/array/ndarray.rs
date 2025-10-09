use core::ops::{Index, IndexMut};

use super::dtype::{DType, HasDType};
use super::trait_array::Array;

/// A minimal n-dimensional array with row-major storage and NumPy-like indexing helpers.
///
/// - Storage: contiguous `Vec<T>` in row-major order
/// - Shape: owned `Vec<usize>`; returned as slice for `Array` trait
/// - Indexing: `get(&[usize])`, `get_mut(&[usize])`, and `Index` for 1D/2D/3D via `[usize; N]`
///
/// # Examples
///
/// ```
/// use molomni::core::array::{NdArray, Array, DType};
///
/// // 2x3 array with row-major data
/// let arr = NdArray::new(vec![2, 3], vec![1, 2, 3, 4, 5, 6]);
/// assert_eq!(arr.dtype(), DType::Int32);
/// assert_eq!(arr.shape(), &[2, 3]);
///
/// // Safe indexing
/// assert_eq!(arr.get(&[0, 2]), Some(&3));
/// assert_eq!(arr.get(&[1, 0]), Some(&4));
/// assert!(arr.get(&[2, 0]).is_none());
///
/// // Index trait (panics on OOB), like NumPy IndexError
/// assert_eq!(arr[[1, 2]], 6);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NdArray<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>, // row-major strides
}

impl<T> NdArray<T> {
    /// Creates a new array from `shape` (row-major) and `data`.
    ///
    /// # Panics
    ///
    /// Panics if `data.len()` does not match the product of `shape`.
    pub fn new(shape: Vec<usize>, data: Vec<T>) -> Self {
        let expected = shape.iter().product::<usize>();
        assert!(expected == data.len(), "data length {} does not match shape product {}", data.len(), expected);
        let strides = compute_row_major_strides(&shape);
        Self { data, shape, strides }
    }

    /// Returns a reference to the underlying data buffer
    #[inline]
    pub fn data(&self) -> &Vec<T> { &self.data }

    /// Returns a mutable reference to the underlying data buffer
    #[inline]
    pub fn data_mut(&mut self) -> &mut Vec<T> { &mut self.data }

    /// Safe element access by n-dimensional index; returns `None` if out of bounds.
    #[inline]
    pub fn get(&self, idx: &[usize]) -> Option<&T> {
        self.offset_of(idx).map(|o| &self.data[o])
    }

    /// Safe mutable element access by n-dimensional index; returns `None` if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, idx: &[usize]) -> Option<&mut T> {
        self.offset_of(idx).map(|o| &mut self.data[o])
    }

    /// Computes the flat offset for an n-dimensional index if in bounds.
    #[inline]
    fn offset_of(&self, idx: &[usize]) -> Option<usize> {
        if idx.len() != self.shape.len() { return None; }
        let mut off = 0usize;
        for (&i, (&dim, &stride)) in idx.iter().zip(self.shape.iter().zip(self.strides.iter())) {
            if i >= dim { return None; }
            off += i * stride;
        }
        Some(off)
    }
}

impl<T: HasDType + Send + Sync> Array for NdArray<T> {
    #[inline]
    fn dtype(&self) -> DType { T::dtype() }

    #[inline]
    fn shape(&self) -> &[usize] { &self.shape }
}

impl<T> NdArray<T> {
    /// Convert the array to another element type using `From<T>` for each element.
    pub fn astype<U>(self) -> NdArray<U>
    where
        U: From<T>,
    {
        let data = self.data.into_iter().map(U::from).collect::<Vec<U>>();
        NdArray::new(self.shape, data)
    }

    /// Stack a list of equally-shaped arrays along a new leading axis.
    ///
    /// Given children each with shape `[d0, d1, ...]`, returns an array with shape
    /// `[n, d0, d1, ...]` where `n = children.len()`, with row-major concatenation order.
    /// Returns `Err` if shapes are ragged (not all equal).
    pub fn stack(children: Vec<NdArray<T>>) -> Result<NdArray<T>, &'static str> {
        // Handle empty children: produce a 1D empty array with shape [0]
        if children.is_empty() {
            return Ok(NdArray::new(vec![0], Vec::new()));
        }

        let n = children.len();

        // Borrow first shape as reference, verify all equal and compute total elements
        let first_shape = children[0].shape.clone();
        for c in children.iter().skip(1) {
            if c.shape != first_shape {
                return Err("ragged shape in stack");
            }
        }

        let inner_elems: usize = first_shape.iter().product::<usize>();
        let total = n * inner_elems;

        // Move data out in order
        let mut data = Vec::with_capacity(total);
        for mut c in children.into_iter() {
            // Ensure strides don't affect layout; data is already contiguous row-major
            debug_assert_eq!(c.shape.iter().product::<usize>(), c.data.len());
            data.append(&mut c.data);
        }

        // New shape is [n, ..first_shape]
        let mut new_shape = Vec::with_capacity(1 + first_shape.len());
        new_shape.push(n);
        new_shape.extend(first_shape);

        Ok(NdArray::new(new_shape, data))
    }
}

impl NdArray<i32> {
    /// Creates a 1D array with values in the half-open interval [start, stop) with a given step.
    /// Panics if `step == 0` or the number of elements would overflow usize.
    pub fn range(start: i32, stop: i32, step: i32) -> Self {
        assert!(step != 0, "step must not be zero");
        let mut data = Vec::new();
        let mut v = start;
        if step > 0 {
            while v < stop { data.push(v); v = v.saturating_add(step); }
        } else {
            while v > stop { data.push(v); v = v.saturating_add(step); }
        }
        NdArray::new(vec![data.len()], data)
    }

    /// NumPy-like arange that defaults to integer array
    pub fn arange(start: i32, stop: i32, step: i32) -> Self {
        Self::range(start, stop, step)
    }
}

impl NdArray<f64> {
    /// Creates a 1D array with values in [start, stop) using a floating step.
    /// Note: This is a simple implementation susceptible to accumulation error.
    pub fn range(start: f64, stop: f64, step: f64) -> Self {
        assert!(step != 0.0, "step must not be zero");
        let mut data = Vec::new();
        let mut v = start;
        if step > 0.0 {
            while v < stop { data.push(v); v += step; }
        } else {
            while v > stop { data.push(v); v += step; }
        }
        NdArray::new(vec![data.len()], data)
    }
}

// Index trait implementations (panic on OOB), mimicking NumPy IndexError behavior
impl<T> Index<[usize; 1]> for NdArray<T> {
    type Output = T;
    fn index(&self, index: [usize; 1]) -> &Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;1]");
        &self.data[o]
    }
}

impl<T> IndexMut<[usize; 1]> for NdArray<T> {
    fn index_mut(&mut self, index: [usize; 1]) -> &mut Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;1]");
        &mut self.data[o]
    }
}

impl<T> Index<[usize; 2]> for NdArray<T> {
    type Output = T;
    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;2]");
        &self.data[o]
    }
}

impl<T> IndexMut<[usize; 2]> for NdArray<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;2]");
        &mut self.data[o]
    }
}

impl<T> Index<[usize; 3]> for NdArray<T> {
    type Output = T;
    fn index(&self, index: [usize; 3]) -> &Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;3]");
        &self.data[o]
    }
}

impl<T> IndexMut<[usize; 3]> for NdArray<T> {
    fn index_mut(&mut self, index: [usize; 3]) -> &mut Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;3]");
        &mut self.data[o]
    }
}

#[inline]
fn compute_row_major_strides(shape: &[usize]) -> Vec<usize> {
    let n = shape.len();
    if n == 0 { return vec![]; }
    let mut strides = vec![0usize; n];
    let mut acc = 1usize;
    for (i, &dim) in shape.iter().enumerate().rev() {
        strides[i] = acc;
        acc = acc.saturating_mul(dim.max(1));
    }
    strides
}
