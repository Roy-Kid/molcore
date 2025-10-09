//! Core molecular modeling types and functionality.

pub mod element;
pub use element::Element;

/// Array trait and implementations
pub mod array;
pub use array::{Array, DType, HasDType, Vec3};

/// Block: dict-like keyed arrays with consistent axis-0 length
pub mod block;
pub use block::Block;

/// Frame: dictionary of Blocks
pub mod frame;
pub use frame::Frame;

/// Triclinic simulation box
pub mod simbox;
pub use simbox::SimBox;

/// Entity Component System (ECS) runtime
pub mod ecs;

/// ForceField definition layer using ECS
pub mod forcefield;
