use super::entity::Entity;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Type-erased component storage.
pub trait Storage {
    /// Downcast helper (immutable).
    fn as_any(&self) -> &dyn Any;
    /// Downcast helper (mutable).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Sparse set storage for components of type `T`.
struct SparseSet<T> {
    sparse: HashMap<Entity, usize>,
    dense: Vec<(Entity, T)>,
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self { sparse: HashMap::new(), dense: Vec::new() }
    }
}

impl<T: 'static + Send + Sync> Storage for SparseSet<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl<T: 'static> SparseSet<T> {
    /// Insert or replace a component value for entity `e`.
    fn insert(&mut self, e: Entity, v: T) {
        if let Some(&i) = self.sparse.get(&e) {
            self.dense[i].1 = v;
        } else {
            let i = self.dense.len();
            self.sparse.insert(e, i);
            self.dense.push((e, v));
        }
    }
    /// Get an immutable reference to a component for `e`.
    fn get(&self, e: Entity) -> Option<&T> {
        self.sparse.get(&e).map(|&i| &self.dense[i].1)
    }
    /// Get a mutable reference to a component for `e`.
    fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        if let Some(&i) = self.sparse.get(&e) { Some(&mut self.dense[i].1) } else { None }
    }
    /// Remove a component for `e`, returning the value if present.
    fn remove(&mut self, e: Entity) -> Option<T> {
        if let Some(index) = self.sparse.remove(&e) {
            let (removed_e, removed_val) = self.dense.swap_remove(index);
            if let Some((moved_e, _)) = self.dense.get(index) {
                self.sparse.insert(*moved_e, index);
            }
            debug_assert_eq!(removed_e, e);
            Some(removed_val)
        } else {
            None
        }
    }
}

fn storage_ref<T: 'static>(map: &HashMap<TypeId, Box<dyn Storage>>) -> &SparseSet<T> {
    map.get(&TypeId::of::<T>())
        .and_then(|b| b.as_any().downcast_ref())
        .expect("component not registered yet")
}

fn storage_mut<T: 'static + Send + Sync>(map: &mut HashMap<TypeId, Box<dyn Storage>>) -> &mut SparseSet<T> {
    map.entry(TypeId::of::<T>())
        .or_insert_with(|| Box::new(SparseSet::<T>::default()))
        .as_any_mut()
        .downcast_mut()
        .unwrap()
}

/// Remove a component storage by `TypeId`. Used for entity despawn cleanup.
pub fn storage_remove_for_type(map: &mut HashMap<TypeId, Box<dyn Storage>>, ty: TypeId, e: Entity) {
    if let Some(boxed) = map.get_mut(&ty) {
        let _ = boxed.as_any_mut();
        let _ = e; // placeholder: generic removal requires knowing T
    }
}

/// Iterate all entities inside a storage of `T`, yielding pairs (Entity, &T).
/// Iterate immutably over all `(Entity, &T)` entries in the `T` storage.
pub fn iter_storage<T: 'static>(map: &HashMap<TypeId, Box<dyn Storage>>) -> impl Iterator<Item = (Entity, &T)> {
    // Return an iterator over entries without exposing internal fields
    struct Wrap<'a, T> { inner: std::slice::Iter<'a, (Entity, T)> }
    impl<'a, T> Iterator for Wrap<'a, T> {
        type Item = (Entity, &'a T);
        fn next(&mut self) -> Option<Self::Item> { self.inner.next().map(|(e, v)| (*e, v)) }
    }
    let dense_ref: &'_ Vec<(Entity, T)> = &storage_ref::<T>(map).dense;
    Wrap { inner: dense_ref.iter() }
}

/// Iterate all entities inside a storage of `T`, yielding pairs (Entity, &mut T).
/// Iterate mutably over all `(Entity, &mut T)` entries in the `T` storage.
pub fn iter_storage_mut<T: 'static + Send + Sync>(map: &mut HashMap<TypeId, Box<dyn Storage>>) -> impl Iterator<Item = (Entity, &mut T)> {
    let dense_ptr: *mut Vec<(Entity, T)> = &mut storage_mut::<T>(map).dense;
    // SAFETY: We hold exclusive &mut to the storage, so iter_mut is safe.
    unsafe { (&mut *dense_ptr).iter_mut().map(|(e, v)| (*e, v)) }
}

// High-level wrappers used by other ECS modules
/// Insert or replace component `T` of entity `e`.
pub fn insert_component<T: 'static + Send + Sync>(map: &mut HashMap<TypeId, Box<dyn Storage>>, e: Entity, v: T) {
    storage_mut::<T>(map).insert(e, v);
}

/// Get immutable reference to component `T` of entity `e`.
pub fn get_component<T: 'static>(map: &HashMap<TypeId, Box<dyn Storage>>, e: Entity) -> Option<&T> {
    storage_ref::<T>(map).get(e)
}

/// Get mutable reference to component `T` of entity `e`.
pub fn get_component_mut<T: 'static + Send + Sync>(map: &mut HashMap<TypeId, Box<dyn Storage>>, e: Entity) -> Option<&mut T> {
    storage_mut::<T>(map).get_mut(e)
}

/// Join two component types immutably, yielding pairs for entities that have both.
/// Join two immutable component storages `A` and `B` by entity id.
pub fn iter_join2<'a, A: 'static, B: 'static>(map: &'a HashMap<TypeId, Box<dyn Storage>>) -> impl Iterator<Item = (Entity, (&'a A, &'a B))> + 'a {
    iter_storage::<A>(map).filter_map(move |(e, a)| { get_component::<B>(map, e).map(|b| (e, (a, b))) })
}
