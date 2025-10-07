use molcore::core::ecs::{World, Schedule};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Pos(f32);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vel(f32);

#[test]
fn schedule_runs_systems_in_order() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert_component(e, Pos(0.0));
    world.insert_component(e, Vel(0.5));

    let mut sched = Schedule::new();
    let target = e;
    sched.add_system(|w: &mut World| {
        let ids: Vec<_> = w.query::<Pos>().iter().map(|(e, _)| e).collect();
        for id in ids { let v = w.get_component::<Vel>(id).copied().unwrap_or(Vel(0.0)); if let Some(p) = w.get_component_mut::<Pos>(id) { p.0 += v.0; } }
    });
    sched.add_system(move |w: &mut World| {
        if let Some(p) = w.get_component_mut::<Pos>(target) { p.0 *= 2.0; }
    });

    sched.run(&mut world);

    assert_eq!(world.get_component::<Pos>(e), Some(&Pos(1.0)));
}
