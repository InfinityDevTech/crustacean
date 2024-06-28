use log::info;
use screeps::{game, Room};

use crate::{memory::{RoomMemory, ScreepsMemory}, room::cache::tick_cache::RoomCache, traits::room::RoomExtensions};

pub mod construction;
pub mod structure_visuals;
pub mod remotes;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn plan_room(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    if game::cpu::bucket() < 500 {
        info!("  [PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("  [PLANNER] Planning order recieved! Planning: {}", room.name_str());

    let room_memory = RoomMemory {
        name: room.name(),
        rcl: room.controller().unwrap().level(),
        planned: false,
        id: 0,
        creeps: Vec::new(),
        remotes: Vec::new(),
        hauler_count: 0,
    };

    memory.create_room(&room.name(), room_memory);

    info!("[PLANNER]  Inserted room into memory! Making cache!");

    cache.create_if_not_exists(room, memory, None);
    true
}