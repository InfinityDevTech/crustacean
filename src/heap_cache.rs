#![allow(clippy::new_without_default)]

use std::{collections::HashMap, sync::Mutex};

use crate::room::cache::heap_cache::RoomHeapCache;

// This is the Top level heap, if its mutable, its a mutex.
// The room fetches itself at the beginning of its execution
pub struct GlobalHeapCache {
    pub rooms: Mutex<HashMap<String, RoomHeapCache>>,

    pub heap_lifetime: Mutex<u32>,
}

impl GlobalHeapCache {
    pub fn new() -> GlobalHeapCache {
        GlobalHeapCache {
            rooms: Mutex::new(HashMap::new()),

            heap_lifetime: Mutex::new(0),
        }
    }
}
