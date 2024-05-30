use log::info;
use screeps::{game, Room, SharedCreepProperties};

use crate::{
    memory::{Role, ScreepsMemory},
    room::cache::RoomCache,
    traits::creep::CreepExtensions, utils,
};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creeps = memory.rooms.get(&room.name()).unwrap().creeps.clone();
    let starting_cpu = game::cpu::get_used();

    info!("  [CREEPS] Running {} creeps", creeps.len());

    for creep_name in creeps {
        let creep = game::creeps().get(creep_name.to_string());

        if creep.is_none() {
            let _ = memory.creeps.remove(&creep_name);
            memory.rooms.get_mut(&room.name()).unwrap().creeps.retain(|x| x != &creep_name);
            continue;
        }

        let creep = creep.unwrap();

        if creep.spawning() { return; }

        let role = utils::name_to_role(&creep.name());

        if role.is_none() { return; }

        match role.unwrap() {
            Role::Miner => local::source_miner::run_creep(&creep, memory, cache),
            Role::Hauler => {
                cache.hauling.haulers.push(creep.name());
            }
            Role::Upgrader => local::upgrader::run_creep(&creep, memory, cache),
            Role::Builder => local::builder::run_creep(&creep, memory, cache),
            _ => {}
        }
    }

    let end_cpu = game::cpu::get_used();
    info!("  [CREEPS] Used {:.4} CPU to run creeps", end_cpu - starting_cpu);
}
