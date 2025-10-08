//! Simple sequential scheduler that runs systems in insertion order.
use super::system::{into_system, System};
use super::world::World;

/// A schedule is an ordered list of systems to run each tick/frame.
pub struct Schedule {
	systems: Vec<System>,
}

impl Schedule {
	/// Create an empty schedule.
	pub fn new() -> Self { Self { systems: Vec::new() } }

	/// Add a system to this schedule.
	pub fn add_system<F>(&mut self, f: F) -> &mut Self
	where
		F: Fn(&mut World) + Send + Sync + 'static,
	{
		self.systems.push(into_system(f));
		self
	}

	/// Run all systems once with the provided world.
	pub fn run(&self, world: &mut World) {
		for sys in &self.systems {
			(sys)(world);
		}
	}
}
