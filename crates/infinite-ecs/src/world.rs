use std::any::TypeId;
use std::collections::HashMap;

use crate::component::{Component, ComponentStorage, SparseSet};
use crate::entity::{Entity, EntityAllocator};
use crate::query::{QueryIter, WorldQuery};
use crate::resource::Resources;

/// The central ECS container. Owns all entities, components, and resources.
pub struct World {
    pub(crate) entities: EntityAllocator,
    pub(crate) components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    resources: Resources,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            components: HashMap::new(),
            resources: Resources::new(),
        }
    }

    // ---- Entity management ----

    /// Spawn a new entity with no components.
    pub fn spawn(&mut self) -> Entity {
        self.entities.allocate()
    }

    /// Despawn an entity, removing all its components.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.deallocate(entity) {
            return false;
        }
        for storage in self.components.values_mut() {
            storage.remove(entity.index);
        }
        true
    }

    /// Check whether an entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Number of alive entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    // ---- Component management ----

    fn storage_mut<T: Component>(&mut self) -> &mut SparseSet<T> {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(SparseSet::<T>::new()))
            .as_any_mut()
            .downcast_mut::<SparseSet<T>>()
            .expect("component type mismatch")
    }

    fn storage<T: Component>(&self) -> Option<&SparseSet<T>> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|s| s.as_any().downcast_ref::<SparseSet<T>>())
    }

    /// Insert a component on an entity. Replaces any existing component of the same type.
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        assert!(
            self.entities.is_alive(entity),
            "cannot insert component on dead entity {entity:?}"
        );
        self.storage_mut::<T>().insert(entity.index, component);
    }

    /// Get an immutable reference to a component on an entity.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        self.storage::<T>()?.get(entity.index)
    }

    /// Get a mutable reference to a component on an entity.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        self.storage_mut::<T>().get_mut(entity.index)
    }

    /// Remove a component from an entity. Returns `true` if it was present.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }
        if let Some(storage) = self.components.get_mut(&TypeId::of::<T>()) {
            storage.remove(entity.index)
        } else {
            false
        }
    }

    /// Check whether an entity has a component of the given type.
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }
        self.storage::<T>()
            .map_or(false, |s| s.get(entity.index).is_some())
    }

    // ---- Queries ----

    /// Query entities that match the given component pattern.
    ///
    /// Returns an iterator of `(Entity, Q::Item)`.
    ///
    /// # Example
    /// ```ignore
    /// for (entity, (pos, vel)) in world.query::<(&Position, &Velocity)>() {
    ///     // ...
    /// }
    /// ```
    pub fn query<Q: WorldQuery>(&self) -> QueryIter<'_, Q> {
        let required = Q::required_type_ids();

        // Find the smallest required storage to use as iteration base.
        let candidate_indices = if required.is_empty() {
            // No required components — iterate all alive entities.
            (0..self.entities.generations.len() as u32)
                .filter(|&i| self.entities.alive.get(i as usize).copied().unwrap_or(false))
                .collect()
        } else {
            let mut best_candidates: Option<Vec<u32>> = None;
            for tid in &required {
                if let Some(storage) = self.components.get(tid) {
                    let indices: Vec<u32> = (0..self.entities.generations.len() as u32)
                        .filter(|&i| storage.has(i))
                        .collect();
                    if best_candidates.as_ref().map_or(true, |b| indices.len() < b.len()) {
                        best_candidates = Some(indices);
                    }
                } else {
                    // A required component type has no storage — no matches possible.
                    return QueryIter {
                        entities_generations: &self.entities.generations,
                        entities_alive: &self.entities.alive,
                        storages: &self.components,
                        candidate_indices: vec![],
                        position: 0,
                        _marker: std::marker::PhantomData,
                    };
                }
            }
            best_candidates.unwrap_or_default()
        };

        QueryIter {
            entities_generations: &self.entities.generations,
            entities_alive: &self.entities.alive,
            storages: &self.components,
            candidate_indices,
            position: 0,
            _marker: std::marker::PhantomData,
        }
    }

    // ---- Resources ----

    /// Insert a singleton resource.
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, value: T) {
        self.resources.insert(value);
    }

    /// Get an immutable reference to a resource.
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    /// Get a mutable reference to a resource.
    pub fn resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    /// Remove a resource.
    pub fn remove_resource<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.resources.remove::<T>()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Name(String);

    #[test]
    fn spawn_and_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        assert_eq!(world.entity_count(), 1);
        world.despawn(e);
        assert!(!world.is_alive(e));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn insert_get_remove_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 1.0, y: 2.0 });
        assert_eq!(
            world.get::<Position>(e),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert!(world.has::<Position>(e));
        assert!(world.remove::<Position>(e));
        assert!(!world.has::<Position>(e));
    }

    #[test]
    fn component_mutation() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.get_mut::<Position>(e).unwrap().x = 5.0;
        assert_eq!(world.get::<Position>(e).unwrap().x, 5.0);
    }

    #[test]
    fn query_single_component() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let results: Vec<_> = world.query::<(&Position,)>().collect();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_multi_component() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e1, Velocity { dx: 1.0, dy: 0.0 });
        world.insert(e2, Position { x: 2.0, y: 0.0 });
        // e2 has no velocity
        world.insert(e3, Velocity { dx: 3.0, dy: 0.0 });
        // e3 has no position

        let results: Vec<_> = world.query::<(&Position, &Velocity)>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, e1);
    }

    #[test]
    fn query_optional() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e1, Name("one".to_string()));
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let results: Vec<_> = world.query::<(&Position, Option<&Name>)>().collect();
        assert_eq!(results.len(), 2);

        let with_name: Vec<_> = results.iter().filter(|(_, (_, n))| n.is_some()).collect();
        assert_eq!(with_name.len(), 1);
    }

    #[test]
    fn despawn_removes_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 1.0, y: 0.0 });
        world.despawn(e);

        let results: Vec<_> = world.query::<(&Position,)>().collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn resource_insert_get() {
        let mut world = World::new();
        world.insert_resource(42u32);
        assert_eq!(world.resource::<u32>(), Some(&42));
        *world.resource_mut::<u32>().unwrap() = 100;
        assert_eq!(world.resource::<u32>(), Some(&100));
    }

    #[test]
    fn generation_reuse_isolation() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.despawn(e1);

        let e2 = world.spawn(); // reuses slot 0 with generation 1
        assert_ne!(e1, e2);
        assert_eq!(world.get::<Position>(e1), None);
        assert_eq!(world.get::<Position>(e2), None);
    }
}
