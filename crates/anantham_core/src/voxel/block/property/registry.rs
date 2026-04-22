use bevy::prelude::*;
use std::sync::{Arc, RwLock};

pub trait PropertyPadder: Send + Sync + 'static {
    fn pad_to(&self, new_size: usize);
}

pub(crate) struct PadderHook<T> {
    pub data_ref: Arc<RwLock<Vec<T>>>,
}

impl<T: Default + Clone + Send + Sync + 'static> PropertyPadder for PadderHook<T> {
    fn pad_to(&self, total_states: usize) {
        let mut array = self.data_ref.write().unwrap();
        if array.len() < total_states {
            array.resize(total_states, T::default());
        }
    }
}

/// The ultra-fast, thread-safe flat array accessed by systems (like Meshing) at runtime.
#[derive(Resource, Clone)]
pub struct PropertyRegistry<T> {
    pub data: Arc<RwLock<Vec<T>>>,
}

impl<T> Default for PropertyRegistry<T> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl<T: Default + Clone + Send + Sync + 'static> PropertyRegistry<T> {
    pub fn set(&self, state_id: u32, value: T) {
        let mut write_lock = self.data.write().unwrap();
        debug_assert!(
            (state_id as usize) < write_lock.len(),
            "PopertyRegistry::set() called with out-of-bounds ID: {}",
            state_id
        );
        write_lock[state_id as usize] = value;
    }

    #[inline(always)]
    pub fn get(&self, state_id: u32) -> T {
        let read_lock = self.data.read().unwrap();
        unsafe { read_lock.get_unchecked(state_id as usize).clone() }
    }
}
