use log::info;
use screeps::{game, Room, SharedCreepProperties};

use crate::{memory::{Role, ScreepsMemory}, room::object_cache::RoomStructureCache, traits::creep::CreepExtensions};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, structure_cache: &RoomStructureCache) {
    info!("  [CREEPS] Running creeps in room: {}", room.name());

    let creeps = memory.rooms.get(&room.name()).unwrap().creeps.clone();

    for creep_name in creeps {
        let creep = game::creeps().get(creep_name.to_string()).unwrap();
        let creep_memory = memory.creeps.get(&creep.name()).unwrap();

        if creep.spawning() || creep.tired() {
            let _ = creep.say("😴", false);
            continue;
        }

        match creep_memory.role {
            Role::Miner => local::source_miner::run_creep(&creep, memory, structure_cache),
            Role::Hauler => local::hauler::run_creep(&creep, memory),
            Role::Upgrader => local::upgrader::run_creep(&creep, memory, structure_cache),
            Role::Builder => local::builder::run_creep(&creep, memory),
            _ => {}
        }
    }
}
