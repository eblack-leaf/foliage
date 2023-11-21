use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bevy_ecs::prelude::Component;
/// order in the instance buffers for a specific entity
#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Index {
    pub value: u32,
}

impl Index {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl From<u32> for Index {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}
/// Manager for indexes that increments and holds the current count of instances
#[derive(Component)]
pub struct Indexer<Key: Eq + Hash + PartialEq + Copy + Clone + 'static> {
    pub indices: HashMap<Key, Index>,
    pub count: u32,
    pub max: u32,
    pub holes: HashSet<Index>,
}

impl<Key: Eq + Hash + PartialEq + Copy + Clone + 'static> Indexer<Key> {
    pub fn new(max: u32) -> Self {
        Self {
            indices: HashMap::new(),
            count: 0,
            max,
            holes: HashSet::new(),
        }
    }
    pub fn has_instances(&self) -> bool {
        self.count() > 0
    }
    /// The max the current buffers can hold
    pub fn max(&self) -> u32 {
        self.max
    }
    /// The current amount in the buffers
    pub fn count(&self) -> u32 {
        self.count
    }
    /// obtain the next index and associate it with a key for reference
    pub fn next(&mut self, key: Key) -> Index {
        let index = match self.holes.is_empty() {
            true => {
                self.count += 1;
                Index::new(self.count.checked_sub(1).unwrap_or(self.count))
            }
            false => {
                let index = *self.holes.iter().next().unwrap();
                self.holes.remove(&index);
                index
            }
        };
        self.indices.insert(key, index);
        index
    }
    /// free a key's index
    pub fn remove(&mut self, key: Key) -> Option<Index> {
        if let Some(index) = self.indices.remove(&key) {
            self.holes.insert(index);
            return Some(index);
        }
        None
    }
    /// get the index for an associated key
    pub fn get_index(&self, key: Key) -> Option<Index> {
        self.indices.get(&key).copied()
    }
    /// signals if the amount of requested indexes is greater than the current max
    pub fn should_grow(&mut self) -> bool {
        if self.count > self.max {
            while self.count > self.max {
                self.max += 1; // replace with growth factor
            }
            return true;
        }
        false
    }
}
