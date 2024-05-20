use log::info;
use screeps::{game, Room};

use crate::{memory::{RoomMemory, ScreepsMemory}, traits::room::RoomExtensions};

pub mod resources;

pub fn plan_room(room: &Room, memory: &mut ScreepsMemory) -> bool {
    if game::cpu::bucket() < 100 {
        info!("[PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("[PLANNER] Planning order recieved! Planning: {}", room.name_str());

    let mut room_memory = RoomMemory {
        name: room.name_str(),
        creeps: Vec::new(),
        sources: Vec::new(),
        haul_orders: Vec::new(),
        links: None,
        creeps_manufactured: 0
    };

    let sources = resources::find_sources(room);

    room_memory.sources = sources;

    memory.create_room(&room.name(), &room_memory);
    return true;
}