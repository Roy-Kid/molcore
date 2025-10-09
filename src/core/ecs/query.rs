//! Query utilities to iterate components.
use super::entity::Entity;
use super::storage::{iter_storage, iter_storage_mut};
use std::marker::PhantomData;

/// Immutable query over a single component type.
pub struct Query<'w, T: 'static> {
	pub(crate) storages: &'w std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>,
	_p: PhantomData<T>,
}
impl<'w, T: 'static> Query<'w, T> {
	pub(crate) fn new(storages: &'w std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>) -> Self {
		Self { storages, _p: PhantomData }
	}
	/// Iterate over all `(Entity, &T)` pairs for this component type.
	pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
		iter_storage::<T>(self.storages)
	}
}

/// Mutable query over a single component type.
pub struct QueryMut<'w, T: 'static + Send + Sync> {
	pub(crate) storages: &'w mut std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>,
	_p: PhantomData<T>,
}
impl<'w, T: 'static + Send + Sync> QueryMut<'w, T> {
	pub(crate) fn new(storages: &'w mut std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>) -> Self {
		Self { storages, _p: PhantomData }
	}
	/// Iterate over all `(Entity, &mut T)` pairs for this component type.
	pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
		iter_storage_mut::<T>(self.storages)
	}
}

/// Convenience function to create an immutable query.
pub fn query<'w, T: 'static>(storages: &'w std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>) -> Query<'w, T> {
	Query::new(storages)
}
/// Convenience function to create a mutable query.
pub fn query_mut<'w, T: 'static + Send + Sync>(storages: &'w mut std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>) -> QueryMut<'w, T> {
	QueryMut::new(storages)
}

/// Iterate two component types immutably, joining on entity id.
pub fn query2<'w, A: 'static, B: 'static>(storages: &'w std::collections::HashMap<std::any::TypeId, Box<dyn super::storage::Storage>>)
	-> impl Iterator<Item = (Entity, (&'w A, &'w B))>
{
	super::storage::iter_join2::<A, B>(storages)
}
