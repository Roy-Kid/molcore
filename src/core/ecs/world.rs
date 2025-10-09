use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::entity::Entity;
use super::events::Events;
use super::query::{Query, QueryMut};
use super::storage::{get_component, get_component_mut, insert_component, Storage};

/// Central ECS state holding entities, components, and resources.
#[derive(Default)]
pub struct World {
    next: u32,
    storages: HashMap<TypeId, Box<dyn Storage>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    /// Create an empty `World`.
    pub fn new() -> Self { Self::default() }

    /// Spawn a new empty entity and return its handle.
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next);
        self.next += 1;
        entity
    }

    /// Insert or replace a component of type `T` for the given entity.
    pub fn insert_component<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) {
        insert_component::<T>(&mut self.storages, entity, component);
    }

    /// Get an immutable reference to a component `T` for the given entity.
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        get_component::<T>(&self.storages, entity)
    }

    /// Get a mutable reference to a component `T` for the given entity.
    pub fn get_component_mut<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        get_component_mut::<T>(&mut self.storages, entity)
    }

    /// Add or replace a resource value `T` stored globally in the `World`.
    pub fn insert_resource<T: 'static>(&mut self, res: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(res));
    }

    /// Get a shared reference to a resource.
    pub fn resource<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }

    /// Get an exclusive reference to a resource.
    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T>())
    }

    /// Create an immutable query over component type `T`.
    pub fn query<T: 'static>(&self) -> Query<'_, T> { Query::new(&self.storages) }
    /// Create a mutable query over component type `T`.
    pub fn query_mut<T: 'static + Send + Sync>(&mut self) -> QueryMut<'_, T> { QueryMut::new(&mut self.storages) }

    /// Create an immutable join query over two component types.
    pub fn query2<'a, A: 'static, B: 'static>(&'a self) -> impl Iterator<Item = (Entity, (&'a A, &'a B))> + 'a {
        super::storage::iter_join2::<A, B>(&self.storages)
    }

    /// Iterate only the entities that have component `T`.
    pub fn entities_with<T: 'static>(&self) -> impl Iterator<Item = Entity> + '_ {
        super::storage::iter_storage::<T>(&self.storages).map(|(e, _)| e)
    }

    /// Iterate only `&T` values (discard entity IDs).
    pub fn values<T: 'static>(&self) -> impl Iterator<Item = &T> + '_ {
        super::storage::iter_storage::<T>(&self.storages).map(|(_, v)| v)
    }

    /// Iterate pairs of `(&A, &B)` for entities that have both components (discard entity IDs).
    pub fn pairs<'a, A: 'static, B: 'static>(&'a self) -> impl Iterator<Item = (&'a A, &'a B)> + 'a {
        super::storage::iter_join2::<A, B>(&self.storages).map(|(_, (a, b))| (a, b))
    }

    /// Access the events resource of type `T` (creates it if missing).
    pub fn events_mut<T: 'static>(&mut self) -> &mut Events<T> {
        use std::any::TypeId;
        self.resources
            .entry(TypeId::of::<Events<T>>())
            .or_insert_with(|| Box::new(Events::<T>::new()))
            .downcast_mut::<Events<T>>()
            .expect("correct type")
    }
}