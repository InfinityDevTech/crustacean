use std::{collections::HashMap, str::FromStr};

use log::info;
use screeps::{
    find, game, look::{self, LookResult}, ErrorCode, HasId, HasPosition, ObjectId, Part, Room, Terrain
};

use crate::{memory::{ScoutedSource, ScreepsMemory}, room::structure_cache::RoomStructureCache, traits::room::RoomExtensions};

use super::{creeps, planning::creep::miner::formulate_miner, tower};

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    info!("[GOVERNMENT] Starting government for room: {}", room.name());
    let structure_cache = RoomStructureCache::new_from_room(&room);

    let spawn = structure_cache.spawns.iter().next();

    if formulate_miner(&room, memory, spawn.unwrap().1.clone()).is_ok() {
        let room_memory = memory.get_room_mut(&room.name());
        room_memory.creeps_manufactured += 1
    }

    tower::run_towers(&room);
    creeps::organizer::run_creeps(&room, memory);
}
