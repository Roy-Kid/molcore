use super::dtype::{DType, HasDType};
use super::trait_array::Array;
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
}

impl Vec3<f64> {
    /// Euclidean norm (length) of the vector
    #[inline]
    pub fn norm(&self) -> f64 { (self.x * self.x + self.y * self.y + self.z * self.z).sqrt() }
}

impl<T: HasDType + Send + Sync> Array for Vec3<T> {
    fn dtype(&self) -> DType {
        T::dtype()
    }

    fn shape(&self) -> &[usize] {
        &[3]
    }
}
