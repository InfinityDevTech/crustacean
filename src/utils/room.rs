use log::info;
use screeps::{game, OwnedStructureProperties, Room};

use crate::memory::{Role, ScreepsMemory};

pub fn room_is_ours(room: &Room) -> bool {
    room.controller().map_or(false, |controller| {
        controller.my()
    })
}

pub fn room_get_miner_target(room: &Room, memory: &mut ScreepsMemory) -> u8 {
    let room_memory = memory.get_room_mut(&room.name());
    let source = room_memory.sources.iter().find(|source| {
        source.assigned_creeps < source.mining_spots
    });

    if source.is_none() {
        room_memory.sources[0].assigned_creeps += 1;
        0
    } else {
        let pos = room_memory.sources.iter().position(|s| s.id == source.unwrap().id).unwrap();
        room_memory.sources[pos].assigned_creeps += 1;
        pos as u8
    }


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