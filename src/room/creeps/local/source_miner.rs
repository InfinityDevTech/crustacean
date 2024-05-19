use std::str::FromStr;

use log::info;
use screeps::{game, Creep, HasPosition, Part, ResourceType, RoomName, SharedCreepProperties, Source};

use crate::{memory::{CreepMemory, ScreepsMemory}, traits::creep::CreepExtensions, utils::{creep::creep_parts_of_type, room::room_get_miner_target}};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let cloned_memory = memory.clone();
    let CreepMemory {o_r, mut t_id, ..} = memory.get_creep(&creep.name());
    let room_memory = cloned_memory.get_room(&RoomName::from_str(&o_r).unwrap());

    if t_id.is_none() {
        let room = game::rooms().get(RoomName::from_str(&o_r).unwrap()).unwrap();
        let target = room_get_miner_target(&room, memory);

        memory.get_creep_mut(&creep.name()).t_id = Some(target);
        t_id = Some(target);
    }

    let pointer_index = t_id.unwrap() as usize;
    let scouted_source = &room_memory.sources[pointer_index];
    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep.pos().is_near_to(source.pos()) {
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) < creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32 {
            let _ = creep.drop(ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));
            info!("Dropping??");
        } else {
            creep.harvest(&source).unwrap_or(());
        }
    } else {
        creep.better_move_to(memory.get_creep_mut(creep.name().as_str()), source.pos(), 1)
    }
}