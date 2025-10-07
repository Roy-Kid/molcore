use molcore::core::ecs::World;

#[test]
fn entity_is_copy_and_eq_hash() {
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = e1;
    assert_eq!(e1, e2);
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(e1);
    assert!(set.contains(&e2));
}
