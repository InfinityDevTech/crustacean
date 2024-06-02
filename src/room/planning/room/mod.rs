use log::info;
use screeps::{game, Room};

use crate::{memory::{RoomMemory, ScreepsMemory}, traits::room::RoomExtensions};

pub mod construction;
pub mod structure_visuals;

pub fn plan_room(room: &Room, memory: &mut ScreepsMemory) -> bool {
    if game::cpu::bucket() < 500 {
        info!("[PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("[PLANNER] Planning order recieved! Planning: {}", room.name_str());

    let room_memory = RoomMemory {
        name: room.name_str(),
        rcl: room.controller().unwrap().level(),
        planned: false,
        id: 0,
        creeps: Vec::new(),
    };

    memory.create_room(&room.name(), room_memory);
    true
}