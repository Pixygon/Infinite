use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Type-map storage for singleton resources.
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a resource, replacing any previous value of the same type.
    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get an immutable reference to a resource.
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref())
    }

    /// Get a mutable reference to a resource.
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut())
    }

    /// Remove a resource, returning it if it existed.
    pub fn remove<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast().ok())
            .map(|b| *b)
    }

    /// Check whether a resource of this type exists.
    pub fn contains<T: 'static + Send + Sync>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut res = Resources::new();
        res.insert(42u32);
        res.insert("hello".to_string());
        assert_eq!(res.get::<u32>(), Some(&42));
        assert_eq!(res.get::<String>(), Some(&"hello".to_string()));
    }

    #[test]
    fn replace() {
        let mut res = Resources::new();
        res.insert(1u32);
        res.insert(2u32);
        assert_eq!(res.get::<u32>(), Some(&2));
    }

    #[test]
    fn mutate() {
        let mut res = Resources::new();
        res.insert(vec![1, 2, 3]);
        res.get_mut::<Vec<i32>>().unwrap().push(4);
        assert_eq!(res.get::<Vec<i32>>().unwrap().len(), 4);
    }

    #[test]
    fn remove_resource() {
        let mut res = Resources::new();
        res.insert(99u32);
        assert_eq!(res.remove::<u32>(), Some(99));
        assert!(!res.contains::<u32>());
    }
}
