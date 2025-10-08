use molomni::core::ecs::{World, Schedule, Entity};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Pos(f32);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vel(f32);

#[test]
fn spawn_and_insert_components() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert_component(e, Pos(1.0));
    world.insert_component(e, Vel(2.0));

    assert_eq!(world.get_component::<Pos>(e), Some(&Pos(1.0)));
    assert_eq!(world.get_component::<Vel>(e), Some(&Vel(2.0)));
}

#[test]
fn query_and_update() {
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    world.insert_component(e1, Pos(0.0));
    world.insert_component(e1, Vel(1.0));
    world.insert_component(e2, Pos(10.0));

    // system: pos += vel
    // To avoid conflicting borrows, collect ids first, then update via world
    let ids: Vec<_> = world.query::<Pos>().iter().map(|(e, _)| e).collect();
    for e in ids {
        let vel = world.get_component::<Vel>(e).copied().unwrap_or(Vel(0.0));
        if let Some(p) = world.get_component_mut::<Pos>(e) { p.0 += vel.0; }
    }

    assert_eq!(world.get_component::<Pos>(e1), Some(&Pos(1.0)));
    assert_eq!(world.get_component::<Pos>(e2), Some(&Pos(10.0)));
}

#[test]
fn schedule_runs_systems() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert_component(e, Pos(0.0));
    world.insert_component(e, Vel(0.5));

    let mut sched = Schedule::new();
    sched.add_system(|w: &mut World| {
        let ids: Vec<_> = w.query::<Pos>().iter().map(|(e, _)| e).collect();
        for e in ids { let v = w.get_component::<Vel>(e).copied().unwrap_or(Vel(0.0)); if let Some(p) = w.get_component_mut::<Pos>(e) { p.0 += v.0; } }
    });

    sched.run(&mut world);
    sched.run(&mut world);

    assert_eq!(world.get_component::<Pos>(e), Some(&Pos(1.0)));
}
