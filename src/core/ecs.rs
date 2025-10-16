//! Minimal, dependency-free ECS: Entities with generations, per-type sparse sets, and simple queries.
//! This is a compact implementation intended for small to medium workloads.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

/// Packed entity handle: 32-bit index + 32-bit generation encoded in a u64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u64);

impl Entity {
    #[inline]
    pub fn index(self) -> usize { (self.0 & 0xFFFF_FFFF) as usize }
    #[inline]
    pub fn generation(self) -> u32 { (self.0 >> 32) as u32 }
    #[inline]
    fn make(index: usize, generation: u32) -> Self { Entity(((generation as u64) << 32) | (index as u64)) }
}

/// Erased store API for despawn sweeping.
trait ErasedStore: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_entity(&mut self, e: Entity);
}

/// SparseSet storage for components of type T.
struct Store<T> {
    dense_e: Vec<Entity>,    // dense entities
    dense_v: Vec<T>,         // dense values
    sparse: Vec<Option<usize>>, // sparse index by entity.index -> dense idx
}

impl<T> Store<T> {
    fn new() -> Self { Self { dense_e: Vec::new(), dense_v: Vec::new(), sparse: Vec::new() } }

    #[inline]
    fn ensure_sparse(&mut self, idx: usize) {
        if self.sparse.len() <= idx { self.sparse.resize(idx + 1, None); }
    }

    fn add(&mut self, e: Entity, val: T) -> bool {
        let idx = e.index();
        self.ensure_sparse(idx);
        if self.sparse[idx].is_some() { return false; }
        let didx = self.dense_e.len();
        self.dense_e.push(e);
        self.dense_v.push(val);
        self.sparse[idx] = Some(didx);
        true
    }

    fn remove(&mut self, e: Entity) -> bool {
        let idx = e.index();
        if idx >= self.sparse.len() { return false; }
        let Some(di) = self.sparse[idx] else { return false };
        // swap-remove in dense
        let last_i = self.dense_e.len() - 1;
        self.dense_e.swap(di, last_i);
        self.dense_v.swap(di, last_i);
        let moved_e = self.dense_e[di];
        self.sparse[moved_e.index()] = Some(di);
        self.dense_e.pop();
        self.dense_v.pop();
        self.sparse[idx] = None;
        true
    }

    fn has(&self, e: Entity) -> bool {
        let idx = e.index();
        idx < self.sparse.len() && self.sparse[idx].is_some()
    }

    fn get(&self, e: Entity) -> Option<&T> {
        let idx = e.index();
        let di = *self.sparse.get(idx)?.as_ref()?;
        self.dense_v.get(di)
    }

    fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        let idx = e.index();
        let di = *self.sparse.get(idx)?.as_ref()?;
        self.dense_v.get_mut(di)
    }
}

impl<T: 'static> ErasedStore for Store<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn remove_entity(&mut self, e: Entity) { let _ = self.remove(e); }
}

/// The ECS world: manages entity lifetimes and per-type component stores.
pub struct World {
    generations: Vec<u32>,
    free: Vec<usize>,
    stores: HashMap<TypeId, Box<dyn ErasedStore>>, // per-type stores
}

impl World {
    pub fn new() -> Self {
        Self { generations: Vec::new(), free: Vec::new(), stores: HashMap::new() }
    }

    /// Spawn a new entity. Reuse a free slot when available.
    pub fn spawn(&mut self) -> Entity {
        if let Some(idx) = self.free.pop() {
            let generation = self.generations[idx];
            return Entity::make(idx, generation);
        }
        let idx = self.generations.len();
        self.generations.push(0);
        Entity::make(idx, 0)
    }

    /// Despawn an entity, sweeping all components and incrementing its generation.
    /// Returns false if entity is invalid or already stale.
    pub fn despawn(&mut self, e: Entity) -> bool {
        let idx = e.index();
        if idx >= self.generations.len() { return false; }
        if self.generations[idx] != e.generation() { return false; }
        // remove from all stores
        for store in self.stores.values_mut() {
            store.remove_entity(e);
        }
        // bump generation and recycle index
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        self.free.push(idx);
        true
    }

    /// Check if an entity handle is currently alive (generation matches).
    pub fn is_alive(&self, e: Entity) -> bool {
        let idx = e.index();
        idx < self.generations.len() && self.generations[idx] == e.generation()
    }

    fn get_store_mut<T: 'static + Send + Sync>(&mut self) -> &mut Store<T> {
        let tid = TypeId::of::<T>();
        if !self.stores.contains_key(&tid) {
            self.stores.insert(tid, Box::new(Store::<T>::new()));
        }
        self.stores.get_mut(&tid).unwrap().as_any_mut().downcast_mut::<Store<T>>().unwrap()
    }

    fn get_store<T: 'static + Send + Sync>(&self) -> Option<&Store<T>> {
        let tid = TypeId::of::<T>();
        self.stores.get(&tid)?.as_any().downcast_ref::<Store<T>>()
    }

    /// Add a component to an entity. Returns false if entity invalid or component already present.
    pub fn add<T: 'static + Send + Sync>(&mut self, e: Entity, c: T) -> bool {
        if !self.is_alive(e) { return false; }
        self.get_store_mut::<T>().add(e, c)
    }

    /// Remove a component from an entity. Returns false if missing or entity invalid.
    pub fn remove<T: 'static + Send + Sync>(&mut self, e: Entity) -> bool {
        if !self.is_alive(e) { return false; }
        self.get_store_mut::<T>().remove(e)
    }

    /// Get an immutable reference to a component, if present and entity alive.
    pub fn get<T: 'static + Send + Sync>(&self, e: Entity) -> Option<&T> {
        if !self.is_alive(e) { return None; }
        self.get_store::<T>()?.get(e)
    }

    /// Get a mutable reference to a component, if present and entity alive.
    pub fn get_mut<T: 'static + Send + Sync>(&mut self, e: Entity) -> Option<&mut T> {
        if !self.is_alive(e) { return None; }
        self.get_store_mut::<T>().get_mut(e)
    }

    /// Check component existence.
    pub fn has<T: 'static + Send + Sync>(&self, e: Entity) -> bool {
        self.is_alive(e) && self.get_store::<T>().map_or(false, |s| s.has(e))
    }

    /// Minimal, read-only query: single component type.
    pub fn query<'w, T: 'static + Send + Sync>(&'w self) -> QueryIter1<'w, T> {
        QueryIter1 { e: 0, store: self.get_store::<T>() }
    }

    /// Minimal, read-only query for two components.
    pub fn query2<'w, A: 'static + Send + Sync, B: 'static + Send + Sync>(&'w self) -> QueryIter2<'w, A, B> {
        QueryIter2 { e: 0, a: self.get_store::<A>(), b: self.get_store::<B>() }
    }

    /// Minimal, mutable query for (&mut A, &B). Restrict to one mutable type.
    pub fn query_mut2<'w, A: 'static + Send + Sync, B: 'static + Send + Sync>(&'w mut self) -> QueryMutIter2<'w, A, B> {
        // split borrow: we only take &mut to A store, & to B store
        let a_ptr: *mut Store<A> = self.get_store_mut::<A>();
        let b_ref: Option<&Store<B>> = self.get_store::<B>();
        QueryMutIter2 { e: 0, a: unsafe { Some(&mut *a_ptr) }, b: b_ref }
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("entities", &self.generations.len())
            .field("free", &self.free.len())
            .field("component_types", &self.stores.len())
            .finish()
    }
}

/// Iterator over (&T) results.
pub struct QueryIter1<'w, T: 'static> { e: usize, store: Option<&'w Store<T>> }
impl<'w, T: 'static + Send + Sync> Iterator for QueryIter1<'w, T> {
    type Item = (Entity, &'w T);
    fn next(&mut self) -> Option<Self::Item> {
        let store = self.store?;
        while self.e < store.dense_e.len() {
            let idx = self.e; self.e += 1;
            let e = store.dense_e[idx];
            let t = &store.dense_v[idx];
            return Some((e, t));
        }
        None
    }
}

/// Iterator over (&A, &B) for entities that have both.
pub struct QueryIter2<'w, A: 'static, B: 'static> { e: usize, a: Option<&'w Store<A>>, b: Option<&'w Store<B>> }
impl<'w, A: 'static + Send + Sync, B: 'static + Send + Sync> Iterator for QueryIter2<'w, A, B> {
    type Item = (Entity, &'w A, &'w B);
    fn next(&mut self) -> Option<Self::Item> {
        let (a, b) = (self.a?, self.b?);
        while self.e < a.dense_e.len() {
            let idx = self.e; self.e += 1;
            let e = a.dense_e[idx];
            if let Some(di_b) = b.sparse.get(e.index()).and_then(|o| *o) {
                return Some((e, &a.dense_v[idx], &b.dense_v[di_b]));
            }
        }
        None
    }
}

/// Mutable A + immutable B iterator.
pub struct QueryMutIter2<'w, A: 'static, B: 'static> { e: usize, a: Option<&'w mut Store<A>>, b: Option<&'w Store<B>> }
impl<'w, A: 'static + Send + Sync, B: 'static + Send + Sync> Iterator for QueryMutIter2<'w, A, B> {
    type Item = (Entity, &'w mut A, &'w B);
    fn next(&mut self) -> Option<Self::Item> {
        let (a_store, b_store) = (self.a.as_mut()?, self.b?);
        while self.e < a_store.dense_e.len() {
            let idx = self.e; self.e += 1;
            let e = a_store.dense_e[idx];
            if let Some(di_b) = b_store.sparse.get(e.index()).and_then(|o| *o) {
                // SAFETY: We return a unique &mut to A for one entity at a time
                let a_ptr: *mut A = &mut a_store.dense_v[idx];
                let a_ref: &'w mut A = unsafe { &mut *a_ptr };
                return Some((e, a_ref, &b_store.dense_v[di_b]));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct A(i32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct B(i32);

    #[test]
    fn spawn_despawn_generation() {
        let mut w = World::new();
        let e1 = w.spawn();
        let e2 = w.spawn();
        assert!(w.is_alive(e1) && w.is_alive(e2));
        assert_eq!(e1.index(), 0);
        assert_eq!(e2.index(), 1);

        assert!(w.despawn(e1));
        assert!(!w.is_alive(e1));

        let e3 = w.spawn();
        assert_eq!(e3.index(), 0); // reused slot
        assert!(e3.generation() > e1.generation());
    }

    #[test]
    fn component_add_remove_get() {
        let mut w = World::new();
        let e = w.spawn();
        assert!(w.add(e, A(1)));
        assert!(w.has::<A>(e));
        assert_eq!(w.get::<A>(e), Some(&A(1)));
        assert!(w.remove::<A>(e));
        assert!(!w.has::<A>(e));
        assert!(w.get::<A>(e).is_none());
    }

    #[test]
    fn generation_safety() {
        let mut w = World::new();
        let e = w.spawn();
        assert!(w.despawn(e));
        // Old handle should be stale
        assert!(!w.add(e, A(1)));
        assert!(w.get::<A>(e).is_none());
    }

    #[test]
    fn query_read_only_and_mut() {
        let mut w = World::new();
        let e1 = w.spawn();
        let e2 = w.spawn();
        w.add(e1, A(10)); w.add(e1, B(1));
        w.add(e2, A(20));

        // (&A, &B)
        let mut seen = Vec::new();
        for (e, a, b) in w.query2::<A, B>() { seen.push((e.index(), a.0, b.0)); }
        assert_eq!(seen, vec![(e1.index(), 10, 1)]);

        // (&mut A, &B)
        for (_e, a, b) in w.query_mut2::<A, B>() { a.0 += b.0; }
        assert_eq!(w.get::<A>(e1), Some(&A(11)));
    }

    #[test]
    fn despawn_cleans_components() {
        let mut w = World::new();
        let e1 = w.spawn();
        w.add(e1, A(5)); w.add(e1, B(7));
        assert!(w.despawn(e1));
        assert!(w.query2::<A, B>().next().is_none());
        assert!(!w.has::<A>(e1));
        assert!(!w.has::<B>(e1));
    }
}
