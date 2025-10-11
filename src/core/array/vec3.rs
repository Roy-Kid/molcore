use super::dtype::{DType, HasDType};
use super::base::Array;
use core::marker::PhantomData;
use core::ops::{Add, Sub};

/// A 3-dimensional vector with generic type T
#[derive(Debug, Clone, PartialEq)]
pub struct Vec3<T> {
    /// The x component
    pub x: T,
    /// The y component
    pub y: T,
    /// The z component
    pub z: T,
}

impl<T> Vec3<T> {
    /// Creates a new Vec3 with the given x, y, z components
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3 { x, y, z }
    }
}

// Allow Vec3 to be Copy when its components are Copy
impl<T: Copy> Copy for Vec3<T> {}

// Basic vector arithmetic for compatible scalar types
impl<T> Add for Vec3<T>
where
    T: Add<Output = T>,
{
    type Output = Vec3<T>;
    #[inline]
    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T> Sub for Vec3<T>
where
    T: Sub<Output = T>,
{
    type Output = Vec3<T>;
    #[inline]
    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Vec3<f32> {
    /// Euclidean norm (length) of the vector
    #[inline]
    pub fn norm(&self) -> f32 { (self.x * self.x + self.y * self.y + self.z * self.z).sqrt() }

    /// Origin (0,0,0)
    #[inline]
    pub fn origin() -> Self { Vec3::new(0.0, 0.0, 0.0) }
}

impl Vec3<f64> {
    /// Euclidean norm (length) of the vector
    #[inline]
    pub fn norm(&self) -> f64 { (self.x * self.x + self.y * self.y + self.z * self.z).sqrt() }
}

impl<T: HasDType + Send + Sync + 'static> Array for Vec3<T> {
    fn dtype(&self) -> DType {
        T::dtype()
    }

    fn shape(&self) -> &[usize] {
        &[3]
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// A read-only view into a 3-element vector with optional stride.
///
/// This mirrors NdArray's borrowed storage pattern using a raw pointer so that the view
/// is `'static` at the type level. Safety is enforced by constructors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3View<T> {
    ptr: *const T,
    stride: usize, // in elements
    _marker: PhantomData<*const T>,
}

impl<T> Vec3View<T> {
    /// Create a view over a contiguous slice of length at least 3
    #[inline]
    pub fn from_slice(slice: &[T]) -> Self {
        assert!(slice.len() >= 3, "slice must have at least 3 elements");
        Self { ptr: slice.as_ptr(), stride: 1, _marker: PhantomData }
    }

    /// Create a view from a raw pointer with an element stride.
    ///
    /// Safety: caller must ensure that `ptr` is valid to read 3 elements spaced by `stride`.
    #[inline]
    pub unsafe fn from_ptr(ptr: *const T, stride: usize) -> Self {
        Self { ptr, stride, _marker: PhantomData }
    }

    /// Get reference to i-th component (0..=2)
    #[inline]
    pub fn get(&self, i: usize) -> &T {
        assert!(i < 3, "index out of bounds for Vec3View");
        unsafe { &*self.ptr.add(i * self.stride) }
    }

    #[inline] pub fn x(&self) -> &T { self.get(0) }
    #[inline] pub fn y(&self) -> &T { self.get(1) }
    #[inline] pub fn z(&self) -> &T { self.get(2) }

    /// Materialize into an owning Vec3 by copying values
    #[inline]
    pub fn to_owned(&self) -> Vec3<T>
    where T: Copy {
        Vec3::new(*self.x(), *self.y(), *self.z())
    }
}

impl<T: HasDType + Send + Sync + 'static> Array for Vec3View<T> {
    #[inline]
    fn dtype(&self) -> DType { T::dtype() }
    #[inline]
    fn shape(&self) -> &[usize] {
        const SHAPE: [usize; 1] = [3];
        &SHAPE
    }
    #[inline]
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// Safety: Vec3View only provides shared access via raw pointer; it is Send/Sync
// when the underlying element type is Sync (read-only sharable).
unsafe impl<T: Sync> Send for Vec3View<T> {}
unsafe impl<T: Sync> Sync for Vec3View<T> {}