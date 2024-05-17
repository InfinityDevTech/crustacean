use std::cmp::min;

use screeps::{
    find, game, Creep, HasPosition, ResourceType, SharedCreepProperties, Structure, StructureObject,
};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    traits::creep::CreepExtensions,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let creep_memory = memory.get_creep_mut(&creep.name());
    let needs_energy = creep_memory.n_e.unwrap_or_else(|| false);

    if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
        get_energy(creep, creep_memory);
    } else {
        haul_energy(creep, memory);
    }
}

pub fn get_energy(creep: &Creep, creep_memory: &mut CreepMemory) {
    let closest_energy = creep.pos().find_closest_by_range(find::DROPPED_RESOURCES);
    if let Some(energy) = closest_energy {
        if creep.pos().is_near_to(energy.clone().pos()) {
            let _ = creep.pickup(&energy);
        } else {
            creep.better_move_to(creep_memory, energy.pos(), 1)
        }
    }
}

pub fn haul_energy(creep: &Creep, screeps_memory: &mut ScreepsMemory) {
    let room_memory = screeps_memory.get_room(&creep.room().unwrap().name());
    let creep_memory = screeps_memory.get_creep_mut(&creep.name());

    //let task_id = &room_memory.haul_orders[creep_memory.t_id.unwrap() as usize];
    let task_id = &room_memory.haul_orders[0];
    let target_structure = game::get_object_by_id_typed(&task_id.target_id).unwrap();
    let structure_object = StructureObject::from(target_structure);

    if let Some(structure) = structure_object.as_transferable() {
        if structure_object
            .as_has_store()
            .unwrap()
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            > 0
        {
            if creep.pos().is_near_to(structure.pos()) {
                let _ = creep.transfer(
                    structure,
                    ResourceType::Energy,
                    Some(min(
                        creep.store().get_used_capacity(Some(ResourceType::Energy)),
                        structure_object
                            .as_has_store()
                            .unwrap()
                            .store()
                            .get_free_capacity(Some(ResourceType::Energy))
                            as u32,
                    )),
                );
            } else {
                creep.better_move_to(creep_memory, structure.pos(), 1);
            }
        }
    }
}
