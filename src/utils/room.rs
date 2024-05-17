use screeps::{game, OwnedStructureProperties, Room};

use crate::memory::{Role, ScreepsMemory};

pub fn room_is_ours(room: &Room) -> bool {
    room.controller().map_or(false, |controller| {
        controller.my()
    })
}

pub fn room_get_creeps_of_role(room: &Room, memory: &mut ScreepsMemory, role: Role) -> Vec<String> {
    memory.clone().get_room(&room.name()).creeps.iter().filter_map(|creep_name| {
        let creep_memory = memory.get_creep(creep_name);
        if creep_memory.r == role {
            Some(creep_name.clone())
        } else {
            None
        }
    }).collect()
}