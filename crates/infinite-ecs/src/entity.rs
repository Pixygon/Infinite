use std::fmt;

/// A generational entity handle. Uses compact u32 index + generation for cache performance.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl Entity {
    /// Create an entity from raw parts (mainly for testing).
    pub fn from_raw(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// The slot index of this entity.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// The generation of this entity (incremented on reuse).
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.index, self.generation)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.index, self.generation)
    }
}

/// Allocates and recycles entity slots with generational tracking.
pub struct EntityAllocator {
    pub(crate) generations: Vec<u32>,
    pub(crate) alive: Vec<bool>,
    free_list: Vec<u32>,
    len: usize,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            alive: Vec::new(),
            free_list: Vec::new(),
            len: 0,
        }
    }

    /// Allocate a new entity, reusing a freed slot if available.
    pub fn allocate(&mut self) -> Entity {
        self.len += 1;
        if let Some(index) = self.free_list.pop() {
            self.alive[index as usize] = true;
            Entity {
                index,
                generation: self.generations[index as usize],
            }
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            self.alive.push(true);
            Entity {
                index,
                generation: 0,
            }
        }
    }

    /// Deallocate an entity. Returns `true` if it was alive.
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        if idx >= self.alive.len() {
            return false;
        }
        if !self.alive[idx] || self.generations[idx] != entity.generation {
            return false;
        }
        self.alive[idx] = false;
        self.generations[idx] += 1;
        self.free_list.push(entity.index);
        self.len -= 1;
        true
    }

    /// Check if an entity is currently alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        idx < self.alive.len() && self.alive[idx] && self.generations[idx] == entity.generation
    }

    /// Number of currently alive entities.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether there are no alive entities.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_sequential() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate();
        let e1 = alloc.allocate();
        assert_eq!(e0.index, 0);
        assert_eq!(e1.index, 1);
        assert_eq!(e0.generation, 0);
        assert_eq!(alloc.len(), 2);
    }

    #[test]
    fn deallocate_and_reuse() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate();
        assert!(alloc.deallocate(e0));
        let e0_reused = alloc.allocate();
        assert_eq!(e0_reused.index, 0);
        assert_eq!(e0_reused.generation, 1);
        assert_ne!(e0, e0_reused);
    }

    #[test]
    fn double_deallocate_fails() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.allocate();
        assert!(alloc.deallocate(e));
        assert!(!alloc.deallocate(e));
    }

    #[test]
    fn stale_entity_not_alive() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate();
        alloc.deallocate(e0);
        assert!(!alloc.is_alive(e0));
        let e0_new = alloc.allocate();
        assert!(alloc.is_alive(e0_new));
    }
}
