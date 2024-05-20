use log::info;
use screeps::{Part, 
    Room}
;

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, room::{creeps::organizer, structure_cache::RoomStructureCache}, traits::room::RoomExtensions, utils::role_to_name};

use super::{creeps, planning::creep::miner::formulate_miner, tower};

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    info!("[GOVERNMENT] Starting government for room: {}", room.name());
    let structure_cache = RoomStructureCache::new_from_room(&room);

    let spawn = structure_cache.spawns.iter().next();

    tower::run_towers(&room, &structure_cache);
    organizer::run_creeps(&room, memory, &structure_cache);

    let _ = formulate_miner(&room, memory, spawn.unwrap().1.clone());
}
