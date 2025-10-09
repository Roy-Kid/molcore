use super::dtype::DType;

/// A trait for array-like types with dtype and shape information
pub trait Array: Send + Sync {
    /// Returns the data type of the array
    fn dtype(&self) -> DType;
    /// Returns the shape of the array as a slice of dimensions
    fn shape(&self) -> &[usize];
}
