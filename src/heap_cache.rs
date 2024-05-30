use std::collections::HashMap;

use screeps::Room;

use crate::{room::cache::heap_cache::RoomHeapCache, traits::room::RoomExtensions};

pub struct GlobalHeapCache {
    pub rooms: HashMap<String, RoomHeapCache>,
}

impl GlobalHeapCache {
    pub fn new() -> GlobalHeapCache {
        GlobalHeapCache {
            rooms: HashMap::new(),
        }
    }
}
