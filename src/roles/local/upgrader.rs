use screeps::{Creep, StructureController, HasPosition, ResourceType, find};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn upgrade(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController) {
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
                creep.better_move_to(creepmem, energy.pos(), 1);
            }
        }
    } else if creep.pos().is_near_to(controller.pos()) {
        let _ = creep.upgrade_controller(&controller);
    } else {
        creep.better_move_to(creepmem, controller.pos(), 2)
    }
}
