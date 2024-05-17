use std::str::FromStr;

use screeps::{game, Creep, HasPosition, Part, ResourceType, RoomName, SharedCreepProperties, Source};

use crate::{memory::{CreepMemory, ScreepsMemory}, traits::creep::CreepExtensions, utils::creep::creep_parts_of_type};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let CreepMemory {o_r, t_id, ..} = memory.get_creep(&creep.name());
    let room_memory = memory.get_room(&RoomName::from_str(&o_r).unwrap());

    let pointer_index = t_id.unwrap() as usize;
    let scouted_source = &room_memory.sources[pointer_index];
    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep.pos().is_near_to(source.pos()) {
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32 {
            let _ = creep.drop(ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));
        } else {
            creep.harvest(&source).unwrap_or(());
        }
    } else {
        creep.better_move_to(memory.get_creep_mut(creep.name().as_str()), source.pos(), 1)
    }
}