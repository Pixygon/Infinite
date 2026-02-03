use crate::world::World;

/// A system that operates on the world each tick.
pub trait System: Send + Sync {
    fn run(&mut self, world: &mut World);
}

/// Blanket implementation so closures can be used as systems.
impl<F: FnMut(&mut World) + Send + Sync> System for F {
    fn run(&mut self, world: &mut World) {
        (self)(world);
    }
}

/// An ordered list of systems to run each frame.
pub struct SystemSchedule {
    systems: Vec<Box<dyn System>>,
}

impl SystemSchedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system to the end of the schedule.
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Run all systems in order on the given world.
    pub fn run_all(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }

    /// Number of systems in the schedule.
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}

impl Default for SystemSchedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn closure_system() {
        let mut world = World::new();
        world.insert_resource(0u32);

        let mut system = |w: &mut World| {
            let count = w.resource_mut::<u32>().unwrap();
            *count += 1;
        };
        system.run(&mut world);
        assert_eq!(*world.resource::<u32>().unwrap(), 1);
    }

    #[test]
    fn schedule_ordering() {
        let mut world = World::new();
        let log = Arc::new(Mutex::new(Vec::<u32>::new()));

        let mut schedule = SystemSchedule::new();
        let log1 = log.clone();
        schedule.add_system(move |_: &mut World| log1.lock().unwrap().push(1));
        let log2 = log.clone();
        schedule.add_system(move |_: &mut World| log2.lock().unwrap().push(2));
        let log3 = log.clone();
        schedule.add_system(move |_: &mut World| log3.lock().unwrap().push(3));

        schedule.run_all(&mut world);
        assert_eq!(*log.lock().unwrap(), vec![1, 2, 3]);
    }
}
