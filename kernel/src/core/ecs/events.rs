//! Simple event bus for ECS.
//!
//! Events are just resources stored in the `World`. To use events, insert
//! `Events<T>` into the world, then write events via `send`, and read them via
//! `iter` or `drain`.

/// Event container for type `T`.
#[derive(Default)]
pub struct Events<T> {
    queue: Vec<T>,
}

impl<T> Events<T> {
    /// Create an empty event queue.
    pub fn new() -> Self { Self { queue: Vec::new() } }

    /// Send an event by pushing it into the queue.
    pub fn send(&mut self, e: T) { self.queue.push(e); }

    /// Iterate over current events by reference.
    pub fn iter(&self) -> impl Iterator<Item = &T> { self.queue.iter() }

    /// Drain all events, returning an iterator that yields owned values.
    pub fn drain(&mut self) -> std::vec::IntoIter<T> { std::mem::take(&mut self.queue).into_iter() }

    /// Clear all events without reading.
    pub fn clear(&mut self) { self.queue.clear(); }

    /// True if there are events available.
    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
    /// Number of pending events.
    pub fn len(&self) -> usize { self.queue.len() }
}
//
