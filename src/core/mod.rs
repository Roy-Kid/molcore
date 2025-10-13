//! Core molecular modeling types and functionality.

pub mod element;
pub use element::Element;

/// Lightweight array types (Vec3, Mat3, NdArray, dtypes)
pub mod array;

/// Geometric regions and predicates
pub mod region;

/// Heterogeneous, axis-0-consistent column store
pub mod block;

/// Top-level container mapping names to blocks, with frame-level metadata
pub mod frame;

#[cfg(feature = "polars")]
pub mod ext_polars;
