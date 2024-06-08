use std::collections::HashMap;

use screeps::{Room, RoomName};

use crate::{heap, memory::ScreepsMemory, traits::room::RoomExtensions};

use self::{creeps::CreepCache, hauling::HaulingCache, resources::RoomResourceCache, structures::RoomStructureCache, traffic::TrafficCache};

use super::heap_cache::RoomHeapCache;

pub mod structures;
pub mod creeps;
pub mod hauling;
pub mod resources;
pub mod traffic;

#[derive(Debug, Clone)]
pub struct RoomCache {
    pub rooms: HashMap<RoomName, CachedRoom>,
}

impl RoomCache {
    pub fn new() -> RoomCache {
        RoomCache {
            rooms: HashMap::new()
        }
    }

    pub fn create_if_not_exists(&mut self, room: &Room, memory: &mut ScreepsMemory) {
        if !self.rooms.contains_key(&room.name()) {
            let cached_room = CachedRoom::new_from_room(room, memory, room.my());

            self.rooms.insert(room.name(), cached_room);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedRoom {
    pub my_room: bool,
    pub room_name: RoomName,

    pub structures: RoomStructureCache,
    pub creeps: CreepCache,
    pub traffic: TrafficCache,

    pub resources: RoomResourceCache,

    //pub hauling: RefCell<HaulingCache>,
    pub hauling: HaulingCache,

    pub heap_cache: RoomHeapCache,
}

impl CachedRoom {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory, my: bool) -> CachedRoom {
        let mut room_cache = heap().rooms.lock().unwrap();

        let mut room_heap = room_cache.remove(&room.name_str()).unwrap_or_else(|| {
            RoomHeapCache::new(room)
        });

        CachedRoom {
            my_room: my,
            room_name: room.name(),

            structures: RoomStructureCache::new_from_room(room, memory, &mut room_heap),
            creeps: CreepCache::new_from_room(room, memory),
            traffic: TrafficCache::new(),
            resources: RoomResourceCache::new_from_room(room, memory, &mut room_heap),

            hauling: HaulingCache::new(),

            heap_cache: room_heap,
            //hauling: RefCell::new(HaulingCache::new()),
        }
    }

    pub fn _refresh_cache(&mut self, room: &Room, memory: &mut ScreepsMemory) {
        self.structures.refresh_structure_cache(room);
        self.structures.refresh_spawn_cache(room);

        self.resources.refresh_source_cache(room, &mut self.heap_cache);

        self.creeps.refresh_creep_cache(memory, room);

        self.traffic.intended_move = HashMap::new();
        self.traffic.movement_map = HashMap::new();
        self.traffic.cached_ops = HashMap::new();
        self.traffic.move_intents = 0;
    }

    pub fn write_cache_to_heap(&self, room: &Room) {
        let mut heap_cache = heap().rooms.lock().unwrap();

        heap_cache.insert(room.name_str(), self.heap_cache.clone());
    }
}