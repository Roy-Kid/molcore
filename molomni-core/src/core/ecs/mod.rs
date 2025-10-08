//! Minimal Entity Component System (ECS)
//!
//! This module provides a lightweight ECS tailored for building external
//! applications (e.g. forcefield, molecule) without imposing an "app"
//! framework. You construct a `World`, register resources, spawn entities,
//! insert components, and run a `Schedule` of systems.

pub mod entity;
pub mod storage;
pub mod world;
pub mod query;
pub mod system;
pub mod scheduler;
pub mod events;

// Re-exports for ergonomic external use
pub use entity::Entity;
pub use events::Events;
pub use query::{query, query_mut, query2};
pub use scheduler::Schedule;
pub use world::World;