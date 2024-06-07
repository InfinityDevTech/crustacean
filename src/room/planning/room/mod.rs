use log::info;
use screeps::{game, Room};

use crate::{memory::{RoomMemory, ScreepsMemory}, room::cache::tick_cache::RoomCache, traits::room::RoomExtensions};

pub mod construction;
pub mod structure_visuals;
pub mod remotes;

pub fn plan_room(room: &Room, memory: &mut ScreepsMemory) -> bool {
    if game::cpu::bucket() < 500 {
        info!("[PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("[PLANNER] Planning order recieved! Planning: {}", room.name_str());
    let cache = &mut RoomCache::new_from_room(room, memory, true);

    let remotes = remotes::fetch_possible_remotes(room, memory, cache);

    let room_memory = RoomMemory {
        name: room.name_str(),
        rcl: room.controller().unwrap().level(),
        planned: false,
        id: 0,
        creeps: Vec::new(),
        remotes,
    };

    memory.create_room(&room.name(), room_memory);
    true
}