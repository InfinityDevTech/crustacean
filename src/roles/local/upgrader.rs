use screeps::{Creep, SharedCreepProperties, StructureController, HasPosition, ResourceType, find};

use crate::{memory::CreepMemory, movement};

pub fn upgrade(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController) {
    let name = creep.name();
    let inventory = creep.store();
    if inventory.get_free_capacity(None)
        > inventory.get_used_capacity(Some(ResourceType::Energy)) as i32
    {
        let closest_energy = creep
            .pos()
            .find_closest_by_path(find::DROPPED_RESOURCES, None);
        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
            } else {
                movement::creep::move_to(&name, creepmem, energy.pos())
            }
        }
    } else if creep.pos().is_near_to(controller.pos()) {
        let _ = creep.upgrade_controller(&controller);
    } else {
        movement::creep::move_to(&name, creepmem, controller.pos())
    }
}
