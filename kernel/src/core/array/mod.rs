//! Array module containing dtype, array trait, and vector implementations

mod dtype;
mod trait_array;
mod vec3;

pub use dtype::{DType, HasDType};
pub use trait_array::Array;
pub use vec3::Vec3;
