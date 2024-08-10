use std::collections::HashMap;

use screeps::{game, OwnedStructureProperties, ResourceType, Room, RoomName, RoomXY};
use stats::StatsCache;
use terminals::TerminalCache;

use crate::{heap, heap_cache::heap_room::HeapRoom, memory::ScreepsMemory, room::spawning::spawn_manager::SpawnManager};

use self::{creeps::CreepCache, hauling::HaulingCache, resources::RoomResourceCache, structures::RoomStructureCache, traffic::TrafficCache};

use super::planning::room::economy;

pub mod structures;
pub mod creeps;
pub mod hauling;
pub mod resources;
pub mod traffic;
pub mod terminals;
pub mod stats;

pub struct RoomCache {
    pub rooms: HashMap<RoomName, CachedRoom>,
    pub my_rooms: Vec<RoomName>,

    pub spawning: SpawnManager,
    pub terminals: TerminalCache,

    pub creeps_moving_stuff: HashMap<String, bool>,
}

#[derive(Debug, Clone)]
pub struct StorageStatus {
    pub has_storage: bool,
    pub stored_energy: u32,
    pub wanted_energy: u32,

    pub energy_needed: u32,
    pub needs_energy: bool,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomCache {
    pub fn new(spawn_manager: SpawnManager) -> RoomCache {
        RoomCache {
            rooms: HashMap::new(),
            my_rooms: Vec::new(),

            terminals: TerminalCache::new(),

            spawning: spawn_manager,
            creeps_moving_stuff: HashMap::new(),
        }
    }

    pub fn create_if_not_exists(&mut self, room: &Room, memory: &mut ScreepsMemory, remote_manager: Option<RoomName>) {
        self.rooms.entry(room.name()).or_insert_with(|| {
            CachedRoom::new_from_room(room, memory, remote_manager)
        });
    }
}

#[derive(Debug, Clone)]
pub struct CachedRoom {
    pub room: Room,
    pub current_holder: Option<String>,
    pub rcl: u8,
    pub reservation: u32,

    pub idle_haulers: u32,
    pub manager: Option<RoomName>,

    pub remotes_with_harvester: Vec<RoomName>,

    pub remotes: Vec<RoomName>,
    pub spawn_center: Option<RoomXY>,
    pub storage_center: Option<RoomXY>,


    pub storage_status: StorageStatus,
    pub structures: RoomStructureCache,
    pub creeps: CreepCache,
    pub traffic: TrafficCache,
    pub resources: RoomResourceCache,
    //pub hauling: RefCell<HaulingCache>,
    pub hauling: HaulingCache,
    pub room_heap_cache: HeapRoom,
    pub stats: StatsCache,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CachedRoom {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory, owning_room: Option<RoomName>) -> CachedRoom {
        let pre_cache_cpu = game::cpu::get_used();

        let mut room_cache = heap().rooms.lock().unwrap();

        let mut room_heap = room_cache.remove(&room.name()).unwrap_or_default();

        let mut resources = RoomResourceCache::new_from_room(room, memory, &mut room_heap);
        let mut structures = RoomStructureCache::new_from_room(room, &mut resources, memory, &mut room_heap);
        let creeps = CreepCache::new_from_room(room, memory, &structures, owning_room);

        let storage_status = storage_status(room, &mut structures);
        let mut stats =  StatsCache::default();
        stats.energy.spending_spawning = 0;

        let mut sp_center = None;
        let mut st_center = None;

        if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
            sp_center = Some(room_memory.spawn_center);
            st_center = Some(room_memory.storage_center);
        }

        let mut cached = CachedRoom {
            room: room.clone(),
            current_holder: None,
            rcl: 0,
            reservation: 0,

            idle_haulers: 0,
            manager: owning_room,
            remotes: Vec::new(),

            remotes_with_harvester: Vec::new(),

            spawn_center: sp_center,
            storage_center: st_center,

            structures,
            creeps,
            traffic: TrafficCache::new(),
            resources,
            hauling: HaulingCache::new(),
            room_heap_cache: room_heap,
            stats,

            storage_status,
        };

        if let Some(room_memory) = memory.rooms.get(&room.name()) {
            cached.remotes.clone_from(&room_memory.remotes);
        }

        if let Some(ref controller) = cached.structures.controller {
            cached.rcl = controller.level();
            cached.reservation = controller.reservation().map_or(0, |r| r.ticks_to_end());

            if let Some(ref reservation) = controller.reservation() {
                cached.current_holder = Some(reservation.username().to_string());
            }

            if let Some(ref owner) = controller.owner() {
                cached.current_holder = Some(owner.username().to_string());
            }
        }

        cached.stats.cpu_cache += game::cpu::get_used() - pre_cache_cpu;

        cached
    }

    pub fn _refresh_cache(&mut self, room: &Room, memory: &mut ScreepsMemory, owning_room: Option<RoomName>) {
        self.resources.refresh_source_cache(room, &mut self.room_heap_cache);
        self.structures.refresh_structure_cache(&mut self.resources, memory);

        self.creeps.refresh_creep_cache(memory, room, &self.structures, owning_room);

        self.traffic.intended_move = HashMap::new();
        self.traffic.movement_map = HashMap::new();
        self.traffic.cached_ops = HashMap::new();
        self.traffic.move_intents = 0;
    }


    pub fn write_cache_to_heap(&self, room: &Room) {
        let mut heap_cache = heap().rooms.lock().unwrap();

        heap_cache.insert(room.name(), self.room_heap_cache.clone());
    }
}

// TODO:
// For lower RCL's, make this take into account fast filler containers,
// Those actually count as a form of storage.
fn storage_status(room: &Room, structures: &mut RoomStructureCache) -> StorageStatus{
    let mut needs_energy = false;
    let mut stored_energy = 0;
    let mut needed_energy = 0;

    if let Some(storage) = &structures.storage {
        stored_energy = storage.store().get_used_capacity(Some(ResourceType::Energy));

        let needed = economy::get_required_energy_storage(room);

        if needed >= stored_energy {
            needed_energy = needed - stored_energy;
            needs_energy = true;
        }

        StorageStatus {
            has_storage: true,
            stored_energy,
            wanted_energy: needed,

            energy_needed: needed_energy,
            needs_energy,
        }
    } else {
        StorageStatus {
            has_storage: false,
            stored_energy,
            wanted_energy: 0,

            energy_needed: needed_energy,
            needs_energy,
        }
    }
}