use log::info;
use screeps::{game, Room};

use crate::{memory::{RoomMemory, ScreepsMemory}, room::cache::tick_cache::RoomCache, traits::room::{RoomExtensions, RoomType}};

pub mod construction;
pub mod structure_visuals;
pub mod remotes;

pub fn plan_room(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    if game::cpu::bucket() < 500 {
        info!("[PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("[PLANNER] Planning order recieved! Planning: {}", room.name_str());
    cache.create_if_not_exists(room, memory, None);

    let remotes = remotes::fetch_possible_remotes(room, memory, cache.rooms.get_mut(&room.name()).unwrap());

    let room_memory = RoomMemory {
        name: room.name(),
        rcl: room.controller().unwrap().level(),
        planned: false,
        id: 0,
        creeps: Vec::new(),
        remotes,
    };

    memory.create_room(&room.name(), room_memory);
    true
}