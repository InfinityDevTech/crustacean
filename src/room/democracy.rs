use log::info;
use screeps::Room;

use crate::{memory::ScreepsMemory, room::{creeps::organizer, structure_cache::RoomStructureCache}};

use super::{planning::creep::miner::formulate_miner, tower};

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    info!("[GOVERNMENT] Starting government for room: {}", room.name());
    let structure_cache = RoomStructureCache::new_from_room(&room);

    let spawn = structure_cache.spawns.iter().next();

    tower::run_towers(&room, &structure_cache);
    organizer::run_creeps(&room, memory, &structure_cache);

    let _ = formulate_miner(&room, memory, spawn.unwrap().1.clone());
}
