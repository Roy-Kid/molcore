use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use super::dtype::{DType, HasDType};
use super::base::Array;

/// A minimal n-dimensional array with row-major storage and NumPy-like indexing helpers.
///
/// - Storage: contiguous `Vec<T>` in row-major order
/// - Shape: owned `Vec<usize>`; returned as slice for `Array` trait
/// - Indexing: `get(&[usize])`, `get_mut(&[usize])`, and `Index` for 1D/2D/3D via `[usize; N]`
///
/// # Examples
///
/// ```
/// use molcore::core::array::{NdArray, Array, DType};
///
/// // 2x3 array with row-major data
/// let arr = NdArray::from_vec(vec![2, 3], vec![1, 2, 3, 4, 5, 6]);
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
/// A simple contiguous N-dimensional array with owned or borrowed backing storage.
#[derive(Debug, Clone, PartialEq)]
pub enum Buffer<T> {
    Owned(Vec<T>),
    Borrowed(*const T, usize, PhantomData<*const T>), // ptr, len
}

/// Row-major N-dimensional array with shape and strides. Can wrap owned `Vec<T>` or a borrowed pointer.
#[derive(Debug, Clone, PartialEq)]
pub struct NdArray<T> {
    data: Buffer<T>,
    shape: Vec<usize>,
    strides: Vec<usize>, // row-major strides
}

impl<T> NdArray<T> {
    /// Creates a new array with owned row-major data from `shape` and `data`.
    ///
    /// Panics if `data.len()` does not match the product of `shape`.
    pub fn from_vec(shape: Vec<usize>, data: Vec<T>) -> Self {
        let expected = shape.iter().product::<usize>();
        assert!(expected == data.len(), "data length {} does not match shape product {}", data.len(), expected);
        let strides = compute_row_major_strides(&shape);
        Self { data: Buffer::Owned(data), shape, strides }
    }

    /// Temporary back-compat alias for `from_vec`.
    #[deprecated(note = "use NdArray::from_vec(shape, data) instead of new(shape, data)")]
    pub fn new(shape: Vec<usize>, data: Vec<T>) -> Self { Self::from_vec(shape, data) }

    /// Creates a borrowed view array from a raw pointer and `len` elements.
    ///
    /// Safety: caller must guarantee that `ptr..ptr+len` is valid for reads for the
    /// lifetime of the returned NdArray, and that it matches the given shape/strides.
    pub unsafe fn from_ptr(shape: Vec<usize>, ptr: *const T, len: usize, strides: Option<Vec<usize>>) -> Self {
        let expected = shape.iter().product::<usize>();
        assert!(expected == len, "ptr length {} does not match shape product {}", len, expected);
        let strides = strides.unwrap_or_else(|| compute_row_major_strides(&shape));
        Self { data: Buffer::Borrowed(ptr, len, PhantomData), shape, strides }
    }

    /// Returns a shared slice view to the underlying contiguous buffer
    #[inline]
    pub fn data(&self) -> &[T] {
        match &self.data {
            Buffer::Owned(v) => v.as_slice(),
            Buffer::Borrowed(ptr, len, _) => unsafe { core::slice::from_raw_parts(*ptr, *len) },
        }
    }

    /// Returns a mutable slice if the array owns its buffer; None for borrowed arrays
    #[inline]
    pub fn data_mut(&mut self) -> Option<&mut [T]> {
        match &mut self.data {
            Buffer::Owned(v) => Some(v.as_mut_slice()),
            Buffer::Borrowed(_, _, _) => None,
        }
    }

    /// Safe element access by n-dimensional index; returns `None` if out of bounds.
    #[inline]
    pub fn get(&self, idx: &[usize]) -> Option<&T> {
    self.offset_of(idx).map(|o| &self.data()[o])
    }

    /// Safe mutable element access by n-dimensional index; returns `None` if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, idx: &[usize]) -> Option<&mut T> {
        match self.offset_of(idx) {
            Some(o) => self.data_mut().map(|s| &mut s[o]),
            None => None,
        }
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

impl<T: HasDType + Send + Sync + 'static> Array for NdArray<T> {
    #[inline]
    fn dtype(&self) -> DType { T::dtype() }

    #[inline]
    fn shape(&self) -> &[usize] { &self.shape }

    #[inline]
    fn as_any(&self) -> &dyn core::any::Any { self }
}

// Public inherent helpers so callers don't need to import the `Array` trait.
impl<T: HasDType> NdArray<T> {
    /// Public accessor for the dtype of the array's element type.
    #[inline]
    pub fn dtype(&self) -> DType {
        T::dtype()
    }

    /// Public accessor for the array shape (slice of dimensions).
    #[inline]
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
}

impl<T> NdArray<T> {
    /// Convert the array to another element type using `From<T>` for each element.
    pub fn astype<U>(self) -> NdArray<U>
    where
        T: Clone,
        U: From<T>,
    {
        let data: Vec<U> = match self.data {
            Buffer::Owned(v) => v.into_iter().map(U::from).collect(),
            Buffer::Borrowed(ptr, len, _) => {
                let s = unsafe { core::slice::from_raw_parts(ptr, len) };
                s.iter().map(|x| U::from(x.clone())).collect()
            }
        };
        NdArray::from_vec(self.shape, data)
    }

    /// Stack a list of equally-shaped arrays along a new leading axis.
    ///
    /// Given children each with shape `[d0, d1, ...]`, returns an array with shape
    /// `[n, d0, d1, ...]` where `n = children.len()`, with row-major concatenation order.
    /// Returns `Err` if shapes are ragged (not all equal).
    pub fn stack(children: Vec<NdArray<T>>) -> Result<NdArray<T>, &'static str>
    where
        T: Clone,
    {
        // Handle empty children: produce a 1D empty array with shape [0]
        if children.is_empty() {
            return Ok(NdArray::from_vec(vec![0], Vec::new()));
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
        for c in children.into_iter() {
            let s = c.data();
            debug_assert_eq!(c.shape.iter().product::<usize>(), s.len());
            data.extend(s.iter().cloned());
        }

        // New shape is [n, ..first_shape]
        let mut new_shape = Vec::with_capacity(1 + first_shape.len());
        new_shape.push(n);
        new_shape.extend(first_shape);

        Ok(NdArray::from_vec(new_shape, data))
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
    NdArray::from_vec(vec![data.len()], data)
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
    NdArray::from_vec(vec![data.len()], data)
    }
}

// Index trait implementations (panic on OOB), mimicking NumPy IndexError behavior
impl<T> Index<[usize; 1]> for NdArray<T> {
    type Output = T;
    fn index(&self, index: [usize; 1]) -> &Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;1]");
        &self.data()[o]
    }
}

impl<T> IndexMut<[usize; 1]> for NdArray<T> {
    fn index_mut(&mut self, index: [usize; 1]) -> &mut Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;1]");
        self.data_mut().and_then(|s| s.get_mut(o)).expect("borrowed array is not mutable or index OOB")
}
}

impl<T> Index<[usize; 2]> for NdArray<T> {
    type Output = T;
    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;2]");
        &self.data()[o]
    }
}

impl<T> IndexMut<[usize; 2]> for NdArray<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;2]");
        self.data_mut().and_then(|s| s.get_mut(o)).expect("borrowed array is not mutable or index OOB")
    }
}

impl<T> Index<[usize; 3]> for NdArray<T> {
    type Output = T;
    fn index(&self, index: [usize; 3]) -> &Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;3]");
        &self.data()[o]
    }
}

impl<T> IndexMut<[usize; 3]> for NdArray<T> {
    fn index_mut(&mut self, index: [usize; 3]) -> &mut Self::Output {
        let o = self.offset_of(&index).expect("index out of bounds or wrong rank for [usize;3]");
        self.data_mut().and_then(|s| s.get_mut(o)).expect("borrowed array is not mutable or index OOB")
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

// Safety: NdArray only provides shared access for borrowed buffers and optional mutable
// access only when it owns the buffer. The raw pointer in Borrowed variant is read-only
// and used to create shared slices. We assert it's Send + Sync when T is Send + Sync.
unsafe impl<T: Send + Sync> Send for NdArray<T> {}
unsafe impl<T: Send + Sync> Sync for NdArray<T> {}