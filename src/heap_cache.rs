#![allow(clippy::new_without_default)]

use std::{collections::HashMap, sync::Mutex};

use screeps::RoomName;
use serde::{Deserialize, Serialize};

use crate::{memory::{Role, ScreepsMemory}, room::cache::heap_cache::RoomHeapCache};


// This is the Top level heap, if its mutable, its a mutex.
// The room fetches itself at the beginning of its execution
pub struct GlobalHeapCache {
    pub rooms: Mutex<HashMap<String, RoomHeapCache>>,
    pub memory: Mutex<ScreepsMemory>,

    pub my_username: Mutex<String>,

    pub heap_lifetime: Mutex<u32>,
}

impl GlobalHeapCache {
    pub fn new() -> GlobalHeapCache {
        GlobalHeapCache {
            rooms: Mutex::new(HashMap::new()),
            memory: Mutex::new(ScreepsMemory::init_memory()),

            my_username: Mutex::new(String::new()),

            heap_lifetime: Mutex::new(0),
        }
    }
}
