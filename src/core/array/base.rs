use super::dtype::DType;
use core::any::Any;

/// A trait for array-like types with dtype and shape information
pub trait Array: Send + Sync + Any {
    /// Returns the data type of the array
    fn dtype(&self) -> DType;
    /// Returns the shape of the array as a slice of dimensions
    fn shape(&self) -> &[usize];
    /// Returns a `&dyn Any` for downcasting to concrete array types
    fn as_any(&self) -> &dyn Any;
}

impl dyn Array {
    /// Try to downcast this trait object reference to a concrete type.
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}