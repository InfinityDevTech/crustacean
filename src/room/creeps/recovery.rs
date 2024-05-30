use screeps::{game, SharedCreepProperties};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, utils::name_to_role};

pub fn recover_creeps(memory: &mut ScreepsMemory) {
    let creep_names = game::creeps().keys();
    for creep_name in creep_names {
        if memory.creeps.contains_key(&creep_name) {
            continue;
        }

        let mut split_name = creep_name.split('-');
        let role = name_to_role(split_name.next().unwrap());
        let time = split_name.next().unwrap();
        let room = split_name.next().unwrap();

        let creep = game::creeps().get(creep_name.clone()).unwrap();

        let Some(role) = role else {
            let _ = creep.suicide();
            continue;
        };

        match role {
            Role::Hauler => {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    owning_room: room.to_string(),
                    path: None,
                };

                memory.create_creep(room, &creep.name(), cmemory);
            }
            Role::Builder => {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    owning_room: room.to_string(),
                    path: None,
                };

                memory.create_creep(room, &creep.name(), cmemory);
            }
            _ => {
                let _ = creep.suicide();
            }
        }
    }
}