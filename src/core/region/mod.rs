//! Region module exports
//!
//! This module collects geometric region data types and traits.

// Expose the ndarray-based Box merged into box.rs
pub mod r#box;
pub use r#box::Box;
