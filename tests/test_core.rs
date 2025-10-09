// Integration tests for core module

mod utils;
mod test_core {
    mod test_element;
    mod test_array;
    mod test_ecs {
        mod test_entity;
        mod test_world;
        mod test_scheduler;
        mod test_events;
    }
    mod test_forcefield;
}
