use log::info;
use screeps::{creep, game, Room};

use crate::{memory::{Role, ScreepsMemory}, traits::room::RoomExtensions};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory) {
    info!("[CREEPS] Running creeps in room: {}", room.name());
    let creeps = memory.get_room(&room.name()).creeps.clone();

    for creep_name in creeps {
        let creep = game::creeps().get(creep_name.clone());

        if creep.is_none() {
            memory.creeps.remove(&creep_name);
            memory
                .get_room(&room.name())
                .creeps
                .retain(|x| x != &creep_name);
            continue;
        }
        let creep = creep.unwrap();
        let creep_memory = memory.get_creep(&creep_name);

        if creep.spawning() {
            continue;
        }

        match creep_memory.r {
            Role::Miner => {
                local::source_miner::run_creep(&creep, memory)
            }
            Role::Hauler => {
                local::hauler::run_creep(&creep, memory)
            }
            Role::Upgrader => {
                local::upgrader::run_creep(&creep, memory)
            }
            Role::Builder => local::builder::run_creep(&creep, memory),
            _ => {}
        }
    }
}
