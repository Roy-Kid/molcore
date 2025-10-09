//! System abstraction
//!
//! A system is any function that takes `&mut World`. The `Schedule` executes
//! systems in order. Parameter extraction, event readers/writers, etc. can be
//! implemented inside the system using `World` APIs.

use super::world::World;

/// Boxed system type. Send + Sync to allow future parallel schedulers.
pub type System = Box<dyn Fn(&mut World) + Send + Sync + 'static>;

/// Create a boxed system from a function or closure.
pub fn into_system<F>(f: F) -> System
where
    F: Fn(&mut World) + Send + Sync + 'static,
{
    Box::new(f)
}
