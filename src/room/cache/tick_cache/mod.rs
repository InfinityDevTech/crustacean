use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use screeps::Room;

use crate::{heap_cache, memory::ScreepsMemory, traits::room::RoomExtensions, HEAP_CACHE};

use self::{creeps::CreepCache, hauling::HaulingCache, resources::RoomResourceCache, structures::RoomStructureCache, traffic::TrafficCache};

use super::heap_cache::RoomHeapCache;

pub mod structures;
pub mod creeps;
pub mod hauling;
pub mod resources;
pub mod traffic;

pub struct RoomCache {
    pub structures: RoomStructureCache,
    pub creeps: CreepCache,
    pub traffic: TrafficCache,

    pub resources: RoomResourceCache,

    //pub hauling: RefCell<HaulingCache>,
    pub hauling: HaulingCache,

    pub heap_cache: RefCell<RoomHeapCache>,
}

impl RoomCache {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory) -> RoomCache {
        let room_heap = heap_cache();

        let room_heap = room_heap.rooms.get(&room.name_str()).unwrap_or_else(|| {
            room_heap.rooms.insert(room.name_str(), RoomHeapCache::new(room));
            room_heap.rooms.get(&room.name_str()).unwrap()
        });

        RoomCache {
            structures: RoomStructureCache::new_from_room(room, memory),
            creeps: CreepCache::new_from_room(room, memory),
            traffic: TrafficCache::new(),
            resources: RoomResourceCache::new_from_room(room, memory),

            hauling: HaulingCache::new(),

            heap_cache: RefCell::new(room_heap),
            //hauling: RefCell::new(HaulingCache::new()),
        }
    }

    pub fn _refresh_cache(&mut self, room: &Room, _memory: &mut ScreepsMemory) {
        self.structures.refresh_structure_cache(room);
        self.structures.refresh_source_cache(room);
        self.structures.refresh_spawn_cache(room);

        self.creeps.refresh_creep_cache(room);

        self.traffic.move_targets = HashMap::new();
        self.traffic.move_requests = HashMap::new();
        self.traffic.movement_map = HashMap::new();
        self.traffic.visited_creeps = HashMap::new();
        self.traffic.cached_ops = HashMap::new();
        self.traffic.move_intents = 0;
    }
}