use log::info;
use screeps::{game, Room, SharedCreepProperties};

use crate::{
    memory::{Role, ScreepsMemory},
    room::cache::RoomCache,
    traits::creep::CreepExtensions,
};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    info!("  [CREEPS] Running creeps in room: {}", room.name());

    let creeps = memory.rooms.get(&room.name()).unwrap().creeps.clone();

    for creep_name in creeps {
        let creep = game::creeps().get(creep_name.to_string());

        if creep.is_none() {
            let creepmem = memory.creeps.get(&creep_name);

            if let Some(creepmem) = creepmem {
                if creepmem.role == Role::Miner {
                    let tid = creepmem.task_id.unwrap();

                    let source = memory
                        .rooms
                        .get_mut(&room.name())
                        .unwrap()
                        .sources
                        .get_mut(tid as usize)
                        .unwrap();
                    source.assigned_creeps -= 1;
                    source.work_parts -= 1;
                }
            }
            let _ = memory.creeps.remove(&creep_name);
            memory.rooms.get_mut(&room.name()).unwrap().creeps.retain(|x| x != &creep_name);
            continue;
        }

        let creep = creep.unwrap();

        let creep_memory = memory.creeps.get(&creep.name()).unwrap();

        if creep.spawning() || creep.tired() {
            let _ = creep.say("😴", false);
            continue;
        }

        match creep_memory.role {
            Role::Miner => local::source_miner::run_creep(&creep, memory, cache),
            Role::Hauler => {
                cache.hauling.haulers.push(creep.name());
            }
            Role::Upgrader => local::upgrader::run_creep(&creep, memory, cache),
            //Role::Builder => local::builder::run_creep(&creep, memory),
            _ => {}
        }
    }
}
