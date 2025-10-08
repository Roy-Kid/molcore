use molomni::core::ecs::World;

#[test]
fn events_roundtrip() {
    let mut world = World::new();
    world.events_mut::<i32>().send(7);
    world.events_mut::<i32>().send(3);
    let got: Vec<i32> = world.events_mut::<i32>().drain().collect();
    assert_eq!(got, vec![7, 3]);
    // After drain, should be empty
    let got2: Vec<i32> = world.events_mut::<i32>().drain().collect();
    assert!(got2.is_empty());
}
