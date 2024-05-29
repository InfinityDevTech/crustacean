use screeps::{game, SharedCreepProperties};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, utils::name_to_role};

pub fn recover_creeps(memory: &mut ScreepsMemory) {
    let creeps = game::creeps().keys();
    for creep in creeps {
        if memory.creeps.contains_key(&creep) {
            continue;
        }

        let mut split_name = creep.split('-');
        let role = name_to_role(split_name.next().unwrap());
        let time = split_name.next().unwrap();
        let room = split_name.next().unwrap();

        let creep = game::creeps().get(creep.clone()).unwrap();

        if let Some(role) = role {
            if role == crate::memory::Role::Hauler {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    owning_room: room.to_string(),
                    path: None,
                };

                memory.create_creep(room, &creep.name(), cmemory);
            } else if role == crate::memory::Role::Builder {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    owning_room: room.to_string(),
                    path: None,
                };

                memory.create_creep(room, &creep.name(), cmemory);
            } else {
                let _ = creep.suicide();
            }
        } else {
            let _ = creep.suicide();
        }
    }
}