//! Entity handle type.
/// A unique handle for an entity in the `World`.
///
/// Entities are lightweight copyable IDs. Components are attached to entities
/// via the `World` APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) u32);