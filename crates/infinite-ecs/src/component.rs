use std::any::Any;

/// Marker trait for types that can be stored as ECS components.
pub trait Component: 'static + Send + Sync {}

/// Blanket implementation: any `'static + Send + Sync` type is a valid component.
impl<T: 'static + Send + Sync> Component for T {}

/// Type-erased component storage interface.
pub(crate) trait ComponentStorage: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, index: u32) -> bool;
    fn has(&self, index: u32) -> bool;
}

/// Sparse-set storage for a single component type. Provides O(1) insert/remove/lookup
/// and dense iteration.
pub(crate) struct SparseSet<T> {
    /// Maps entity index → dense index. `None` means the entity has no component.
    sparse: Vec<Option<usize>>,
    /// Packed component values.
    dense: Vec<T>,
    /// Entity indices corresponding to each dense slot (for iteration).
    entities: Vec<u32>,
}

impl<T: Component> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Insert or replace a component for the given entity index.
    pub fn insert(&mut self, index: u32, value: T) {
        let idx = index as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, None);
        }
        if let Some(dense_idx) = self.sparse[idx] {
            self.dense[dense_idx] = value;
        } else {
            let dense_idx = self.dense.len();
            self.sparse[idx] = Some(dense_idx);
            self.dense.push(value);
            self.entities.push(index);
        }
    }

    /// Get an immutable reference to the component for an entity.
    pub fn get(&self, index: u32) -> Option<&T> {
        let idx = index as usize;
        self.sparse
            .get(idx)
            .and_then(|s| s.map(|dense_idx| &self.dense[dense_idx]))
    }

    /// Get a mutable reference to the component for an entity.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        let idx = index as usize;
        self.sparse
            .get(idx)
            .and_then(|s| s.map(|dense_idx| &mut self.dense[dense_idx]))
    }

    /// Iterate over all (entity_index, &component) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.entities.iter().copied().zip(self.dense.iter())
    }

    /// Iterate over all (entity_index, &mut component) pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.entities.iter().copied().zip(self.dense.iter_mut())
    }

    /// The dense array of all entity indices that have this component.
    pub fn entity_indices(&self) -> &[u32] {
        &self.entities
    }

    /// Number of components stored.
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }
}

impl<T: Component> ComponentStorage for SparseSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        if idx >= self.sparse.len() {
            return false;
        }
        let Some(dense_idx) = self.sparse[idx] else {
            return false;
        };
        self.sparse[idx] = None;

        let last = self.dense.len() - 1;
        if dense_idx != last {
            // Swap-remove: move the last element into the removed slot.
            self.dense.swap(dense_idx, last);
            self.entities.swap(dense_idx, last);
            let moved_entity = self.entities[dense_idx];
            self.sparse[moved_entity as usize] = Some(dense_idx);
        }
        self.dense.pop();
        self.entities.pop();
        true
    }

    fn has(&self, index: u32) -> bool {
        let idx = index as usize;
        idx < self.sparse.len() && self.sparse[idx].is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut set = SparseSet::new();
        set.insert(5, 42i32);
        assert_eq!(set.get(5), Some(&42));
        assert_eq!(set.get(0), None);
    }

    #[test]
    fn overwrite() {
        let mut set = SparseSet::new();
        set.insert(0, 1i32);
        set.insert(0, 2);
        assert_eq!(set.get(0), Some(&2));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn remove_and_swap() {
        let mut set = SparseSet::new();
        set.insert(0, 'a');
        set.insert(1, 'b');
        set.insert(2, 'c');
        assert!(set.remove(0));
        assert_eq!(set.get(0), None);
        assert_eq!(set.get(1), Some(&'b'));
        assert_eq!(set.get(2), Some(&'c'));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn iteration() {
        let mut set = SparseSet::new();
        set.insert(10, 100i32);
        set.insert(20, 200);
        let mut items: Vec<_> = set.iter().collect();
        items.sort_by_key(|(idx, _)| *idx);
        assert_eq!(items, vec![(10, &100), (20, &200)]);
    }
}
