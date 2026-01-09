use serde::{Deserialize, Serialize};
use crate::fishing::rod_data::bait::Bait;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaitBucket {
    pub baits: Vec<Bait>,
}

impl BaitBucket {
    pub fn new() -> Self {
        Self { baits: Vec::new() }
    }

    pub fn add(&mut self, bait: Bait) {
        self.baits.push(bait);
    }

    pub fn remove_index(&mut self, index: usize) -> Option<Bait> {
        if index < self.baits.len() {
            Some(self.baits.remove(index))
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&Bait> {
        self.baits.get(index)
    }

    pub fn len(&self) -> usize {
        self.baits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.baits.is_empty()
    }
}