#![allow(private_interfaces)]

use std::any::TypeId;
use std::collections::HashMap;

use crate::component::{ComponentStorage, SparseSet};
use crate::entity::Entity;

/// Trait implemented for query parameter types (`&T`, `&mut T`, `Option<&T>`, etc.).
///
/// # Safety
/// Implementors must correctly report the component TypeId they access.
pub unsafe trait WorldQuery {
    type Item<'w>;

    /// The TypeIds of components this query requires (must be present on the entity).
    fn required_type_ids() -> Vec<TypeId>;

    /// The TypeIds of components this query optionally reads (may be absent).
    fn optional_type_ids() -> Vec<TypeId>;

    /// Fetch the item for a given entity index from the storages map.
    ///
    /// # Safety
    /// The caller must ensure the entity has all required components and the
    /// pointer aliasing rules for `&` vs `&mut` are upheld.
    unsafe fn fetch<'w>(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        index: u32,
    ) -> Option<Self::Item<'w>>;
}

// --- Implementations for &T (immutable borrow) ---

unsafe impl<T: 'static + Send + Sync> WorldQuery for &T {
    type Item<'w> = &'w T;

    fn required_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn optional_type_ids() -> Vec<TypeId> {
        vec![]
    }

    unsafe fn fetch<'w>(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        index: u32,
    ) -> Option<Self::Item<'w>> {
        let storage = storages.get(&TypeId::of::<T>())?;
        let sparse = storage.as_any().downcast_ref::<SparseSet<T>>()?;
        sparse.get(index)
    }
}

// --- Implementations for &mut T (mutable borrow) ---

unsafe impl<T: 'static + Send + Sync> WorldQuery for &mut T {
    type Item<'w> = &'w mut T;

    fn required_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn optional_type_ids() -> Vec<TypeId> {
        vec![]
    }

    unsafe fn fetch<'w>(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        index: u32,
    ) -> Option<Self::Item<'w>> {
        let storage = storages.get(&TypeId::of::<T>())?;
        // We need a mutable reference. The caller guarantees aliasing safety.
        let storage_ptr = storage.as_ref() as *const dyn ComponentStorage as *mut dyn ComponentStorage;
        let sparse = (*storage_ptr)
            .as_any_mut()
            .downcast_mut::<SparseSet<T>>()?;
        sparse.get_mut(index)
    }
}

// --- Implementation for Option<&T> (optional immutable borrow) ---

unsafe impl<T: 'static + Send + Sync> WorldQuery for Option<&T> {
    type Item<'w> = Option<&'w T>;

    fn required_type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn optional_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    unsafe fn fetch<'w>(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        index: u32,
    ) -> Option<Self::Item<'w>> {
        let result = storages
            .get(&TypeId::of::<T>())
            .and_then(|s| s.as_any().downcast_ref::<SparseSet<T>>())
            .and_then(|sparse| sparse.get(index));
        Some(result)
    }
}

// --- Tuple implementations ---

macro_rules! impl_world_query_tuple {
    ($($name:ident),+) => {
        #[allow(non_snake_case)]
        unsafe impl<$($name: WorldQuery),+> WorldQuery for ($($name,)+) {
            type Item<'w> = ($($name::Item<'w>,)+);

            fn required_type_ids() -> Vec<TypeId> {
                let mut ids = Vec::new();
                $(ids.extend($name::required_type_ids());)+
                ids
            }

            fn optional_type_ids() -> Vec<TypeId> {
                let mut ids = Vec::new();
                $(ids.extend($name::optional_type_ids());)+
                ids
            }

            unsafe fn fetch<'w>(
                storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
                index: u32,
            ) -> Option<Self::Item<'w>> {
                Some(($($name::fetch(storages, index)?,)+))
            }
        }
    };
}

impl_world_query_tuple!(A);
impl_world_query_tuple!(A, B);
impl_world_query_tuple!(A, B, C);
impl_world_query_tuple!(A, B, C, D);
impl_world_query_tuple!(A, B, C, D, E);
impl_world_query_tuple!(A, B, C, D, E, F);
impl_world_query_tuple!(A, B, C, D, E, F, G);
impl_world_query_tuple!(A, B, C, D, E, F, G, H);

/// Iterator returned by `World::query`. Yields `(Entity, Q::Item)` for each matching entity.
pub struct QueryIter<'w, Q: WorldQuery> {
    pub(crate) entities_generations: &'w [u32],
    pub(crate) entities_alive: &'w [bool],
    pub(crate) storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
    pub(crate) candidate_indices: Vec<u32>,
    pub(crate) position: usize,
    pub(crate) _marker: std::marker::PhantomData<Q>,
}

impl<'w, Q: WorldQuery> Iterator for QueryIter<'w, Q> {
    type Item = (Entity, Q::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.position >= self.candidate_indices.len() {
                return None;
            }
            let index = self.candidate_indices[self.position];
            self.position += 1;

            let idx = index as usize;
            if idx >= self.entities_alive.len() || !self.entities_alive[idx] {
                continue;
            }

            // Safety: we iterate each entity at most once so aliasing is safe.
            if let Some(item) = unsafe { Q::fetch(self.storages, index) } {
                let entity = Entity {
                    index,
                    generation: self.entities_generations[idx],
                };
                return Some((entity, item));
            }
        }
    }
}
