//! Core molecular modeling types and functionality.

pub mod element;
pub use element::Element;

// Temporarily disable the legacy custom array module during ndarray migration
// mod array;

/// Geometric regions and predicates
pub mod region;
pub mod types;
pub mod block;
pub mod frame;
pub mod locality;
pub mod ecs;
pub mod topology;
pub mod universe;

// Optional: external integrations can go behind features
