//! Array module containing dtype, array trait, and vector implementations

mod dtype;
mod base;
mod vec3;
mod mat3;
mod ndarray;
#[macro_use]
mod macros;

pub use dtype::{DType, HasDType};
pub use base::Array;
pub use vec3::{Vec3, Vec3View};
pub use mat3::{Mat3, Mat3View};
pub use ndarray::NdArray;