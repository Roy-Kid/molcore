//! Core molecular modeling types and functionality.

pub mod element;
pub use element::Element;

/// Array trait and implementations
pub mod array;
pub use array::{Array, DType, HasDType, Vec3};
