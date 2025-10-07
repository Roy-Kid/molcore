use molcore::core::ecs::World;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Pos(f32);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vel(f32);

#[test]
fn spawn_and_components() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert_component(e, Pos(1.0));
    world.insert_component(e, Vel(2.0));
    assert_eq!(world.get_component::<Pos>(e), Some(&Pos(1.0)));
    assert_eq!(world.get_component::<Vel>(e), Some(&Vel(2.0)));
}

#[test]
fn query_and_update_without_conflicts() {
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    world.insert_component(e1, Pos(0.0));
    world.insert_component(e1, Vel(1.0));
    world.insert_component(e2, Pos(10.0));

    let ids: Vec<_> = world.query::<Pos>().iter().map(|(e, _)| e).collect();
    for e in ids {
        let v = world.get_component::<Vel>(e).copied().unwrap_or(Vel(0.0));
        if let Some(p) = world.get_component_mut::<Pos>(e) { p.0 += v.0; }
    }

    assert_eq!(world.get_component::<Pos>(e1), Some(&Pos(1.0)));
    assert_eq!(world.get_component::<Pos>(e2), Some(&Pos(10.0)));
}
