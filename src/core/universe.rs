use super::region::r#box::Box;
use super::topology::Topology;
use super::ecs::World;

#[derive(Debug)]
pub struct Universe {
    pub simbox: Box,
    pub topology: Topology,
    pub ecs: World
}

impl Universe {
    pub fn new(simbox: Box, topology: Topology) -> Self {
        Self { simbox, topology, ecs: World::new() }
    }

    pub fn empty(simbox: Box) -> Self {
        Self { simbox, topology: Topology::new(), ecs: World::new() }
    }
}