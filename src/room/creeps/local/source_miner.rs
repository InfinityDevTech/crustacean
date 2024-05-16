use screeps::{Source, HasPosition, Creep, ResourceType, Part, SharedCreepProperties};

use crate::{memory::ScreepsMemory, traits::creep::CreepExtensions, utils::creep::creep_parts_of_type};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let owning_room = memory.get_creep(&creep.name()).o_r.clone();
    if creep.pos().is_near_to(source.pos()) {

        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32 {

            let _ = creep.drop(ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));
        } else {

        creep.harvest(&source).unwrap_or(());
            let energy_harvested = std::cmp::min(creep_parts_of_type(&creep, Part::Work) * 2, source.energy()) as u64;
        }
    } else {

        creep.better_move_to(memory.get_creep(&creep.name()), source.pos(), 1)

    }
}