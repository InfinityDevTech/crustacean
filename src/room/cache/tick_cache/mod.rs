use std::collections::HashMap;

use screeps::{game, Room, RoomName};
use stats::StatsCache;

use crate::{heap, memory::ScreepsMemory, traits::room::RoomExtensions};

use self::{creeps::CreepCache, hauling::HaulingCache, resources::RoomResourceCache, structures::RoomStructureCache, traffic::TrafficCache};

use super::heap_cache::RoomHeapCache;

pub mod structures;
pub mod creeps;
pub mod hauling;
pub mod resources;
pub mod traffic;
pub mod stats;

#[derive(Debug, Clone)]
pub struct RoomCache {
    pub rooms: HashMap<RoomName, CachedRoom>,

    pub my_rooms: Vec<RoomName>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomCache {
    pub fn new() -> RoomCache {
        RoomCache {
            rooms: HashMap::new(),

            my_rooms: Vec::new(),
        }
    }

    pub fn create_if_not_exists(&mut self, room: &Room, memory: &mut ScreepsMemory, remote_manager: Option<RoomName>) {
        self.rooms.entry(room.name()).or_insert_with(|| {
            let cached_room = CachedRoom::new_from_room(room, memory, remote_manager);

            cached_room
        });
    }
}

#[derive(Debug, Clone)]
pub struct CachedRoom {
    pub room_name: RoomName,
    pub manager: Option<RoomName>,

    pub remotes: Vec<RoomName>,

    pub structures: RoomStructureCache,
    pub creeps: CreepCache,
    pub traffic: TrafficCache,

    pub resources: RoomResourceCache,

    //pub hauling: RefCell<HaulingCache>,
    pub hauling: HaulingCache,

    pub heap_cache: RoomHeapCache,

    pub stats: StatsCache
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CachedRoom {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory, remote_manager: Option<RoomName>) -> CachedRoom {
        let pre_cache_cpu = game::cpu::get_used();

        let mut room_cache = heap().rooms.lock().unwrap();

        let mut room_heap = room_cache.remove(&room.name_str()).unwrap_or_else(|| {
            RoomHeapCache::new(room)
        });

        let mut resources = RoomResourceCache::new_from_room(room, memory, &mut room_heap);
        let structures = RoomStructureCache::new_from_room(room, &mut resources, memory, &mut room_heap);
        let mut stats =  StatsCache::default();
        stats.energy.spending_spawning = 0;

        let mut cached = CachedRoom {
            room_name: room.name(),
            manager: remote_manager,
            remotes: Vec::new(),

            structures,
            creeps: CreepCache::new_from_room(room, memory),
            traffic: TrafficCache::new(),
            resources,

            hauling: HaulingCache::new(),

            heap_cache: room_heap,
            stats,
            //hauling: RefCell::new(HaulingCache::new()),
        };

        if let Some(room_memory) = memory.rooms.get(&room.name()) {
            cached.remotes.clone_from(&room_memory.remotes);
        }

        cached.stats.cpu_cache += game::cpu::get_used() - pre_cache_cpu;

        cached
    }

    pub fn _refresh_cache(&mut self, room: &Room, memory: &mut ScreepsMemory) {
        self.resources.refresh_source_cache(room, &mut self.heap_cache);
        self.structures.refresh_structure_cache(&mut self.resources, room);

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