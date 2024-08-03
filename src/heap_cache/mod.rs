#![allow(clippy::new_without_default)]

use std::{collections::HashMap, sync::Mutex};

use compressed_matrix::CompressedMatrix;
use hauling::HeapHaulingCache;
use heap_creep::HeapCreep;
use heap_room::HeapRoom;
use screeps::{game, Direction, LocalCostMatrix, Position, RoomName};

use crate::{constants::ROOM_AREA, memory::ScreepsMemory, movement::movement_utils::num_to_dir};

pub mod heap_creep;
pub mod hauling;
pub mod compressed_matrix;
pub mod heap_room;
// TODO:
// Fully flesh this out, here are my ideas for paths to cache.
// For owned rooms (resets every 500 ticks):
// - Storage
// - Controller storage (Link, container, or, default to controller)
// - Path to every container (max 5)
// For remotes (resets every 2000 ticks):
// - Owning room storage (Lazy FF)
// - Each source or its container, again, lazy FF
// - Controller (Possibly, doesnt really matter as much)
// Now, im considering doing it the way v1king did it, and add path entrances and exits
// but i think it would be better to just have a list of "cacheable positions", and then
// if that position is in the cache, use it, if not, use a pathfinder call to generate it
// I think this would be better, as we dont need to pre-gen it, plus, over time, CPU usage would go down

// TODO:
// Figure out how to lazily reset these, im thinking of:
// - Controller upgrade
// - Storage or container built / destroyed
// - Every 10k ticks, reset all paths (Just in case)
#[derive(Debug, Clone)]
pub struct RoomHeapFlowCache {
    pub storage: Option<CompressedMatrix>,
    pub controller: Option<CompressedMatrix>,
    pub paths: HashMap<Position, CompressedMatrix>,
}

impl RoomHeapFlowCache {
    pub fn new() -> RoomHeapFlowCache {
        RoomHeapFlowCache {
            storage: None,
            controller: None,
            paths: HashMap::new(),
        }
    }
}

// This is the Top level heap, if its mutable, its a mutex.
// The room fetches itself at the beginning of its execution
pub struct GlobalHeapCache {
    pub rooms: Mutex<HashMap<RoomName, HeapRoom>>,
    pub creeps: Mutex<HashMap<String, HeapCreep>>,
    pub hauling: Mutex<HeapHaulingCache>,
    pub memory: Mutex<ScreepsMemory>,

    pub my_username: Mutex<String>,

    pub per_tick_cost_matrixes: Mutex<HashMap<RoomName, LocalCostMatrix>>,
    pub per_tick_creep_says: Mutex<HashMap<String, (bool, String)>>,

    pub flow_cache: Mutex<HashMap<RoomName, RoomHeapFlowCache>>,
    pub cachable_positions: Mutex<HashMap<RoomName, Vec<Position>>>,
    pub needs_cachable_position_generation: Mutex<Vec<RoomName>>,

    pub creep_say: Mutex<bool>,
    pub heap_lifetime: Mutex<u32>,
    pub unique_id: Mutex<u128>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl GlobalHeapCache {
    pub fn new() -> GlobalHeapCache {
        GlobalHeapCache {
            rooms: Mutex::new(HashMap::new()),
            creeps: Mutex::new(HashMap::new()),
            memory: Mutex::new(ScreepsMemory::init_memory()),
            hauling: Mutex::new(HeapHaulingCache::default()),

            my_username: Mutex::new(String::new()),

            per_tick_cost_matrixes: Mutex::new(HashMap::new()),
            per_tick_creep_says: Mutex::new(HashMap::new()),

            flow_cache: Mutex::new(HashMap::new()),
            cachable_positions: Mutex::new(HashMap::new()),
            needs_cachable_position_generation: Mutex::new(Vec::new()),

            creep_say: Mutex::new(true),
            heap_lifetime: Mutex::new(0),
            unique_id: Mutex::new(game::time() as u128),
        }
    }
}
