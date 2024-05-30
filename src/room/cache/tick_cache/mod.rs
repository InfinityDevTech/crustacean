use std::collections::HashMap;

use screeps::Room;

use crate::memory::ScreepsMemory;

use self::{creeps::CreepCache, hauling::HaulingCache, resources::RoomResourceCache, structures::RoomStructureCache, traffic::TrafficCache};

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
}

impl RoomCache {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory) -> RoomCache {
        RoomCache {
            structures: RoomStructureCache::new_from_room(room, memory),
            creeps: CreepCache::new_from_room(room, memory),
            traffic: TrafficCache::new(),

            resources: RoomResourceCache::new_from_room(room, memory),

            hauling: HaulingCache::new(),
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