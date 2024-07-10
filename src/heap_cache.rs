#![allow(clippy::new_without_default)]

use std::{collections::HashMap, sync::Mutex};

use screeps::{game, LocalCostMatrix, RoomName};

use crate::{memory::ScreepsMemory, room::cache::heap_cache::{hauling::HeapHaulingCache, RoomHeapCache}};


// This is the Top level heap, if its mutable, its a mutex.
// The room fetches itself at the beginning of its execution
pub struct GlobalHeapCache {
    pub rooms: Mutex<HashMap<RoomName, RoomHeapCache>>,
    pub hauling: Mutex<HeapHaulingCache>,
    pub memory: Mutex<ScreepsMemory>,

    pub my_username: Mutex<String>,

    pub per_tick_cost_matrixes: Mutex<HashMap<RoomName, LocalCostMatrix>>,

    pub creep_say: Mutex<bool>,
    pub heap_lifetime: Mutex<u32>,
    pub unique_id: Mutex<u128>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl GlobalHeapCache {
    pub fn new() -> GlobalHeapCache {
        GlobalHeapCache {
            rooms: Mutex::new(HashMap::new()),
            memory: Mutex::new(ScreepsMemory::init_memory()),
            hauling: Mutex::new(HeapHaulingCache::default()),

            my_username: Mutex::new(String::new()),

            per_tick_cost_matrixes: Mutex::new(HashMap::new()),

            creep_say: Mutex::new(true),
            heap_lifetime: Mutex::new(0),
            unique_id: Mutex::new(game::time() as u128),
        }
    }
}
