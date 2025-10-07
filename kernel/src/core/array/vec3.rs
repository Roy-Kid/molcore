use super::dtype::{DType, HasDType};
use super::trait_array::Array;

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

impl<T: HasDType + Send + Sync> Array for Vec3<T> {
    fn dtype(&self) -> DType {
        T::dtype()
    }

    fn shape(&self) -> &[usize] {
        &[3]
    }
}
