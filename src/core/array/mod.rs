//! Array module containing dtype, array trait, and vector implementations

mod dtype;
mod trait_array;
mod vec3;
mod mat3;
mod ndarray;
#[macro_use]
mod macros;

pub use dtype::{DType, HasDType};
pub use trait_array::Array;
pub use vec3::Vec3;
pub use mat3::Mat3;
pub use ndarray::NdArray;
