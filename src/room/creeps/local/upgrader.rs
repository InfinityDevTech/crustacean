use screeps::{Creep, StructureController, HasPosition, ResourceType, find};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController) {
    let inventory = creep.store();
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
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
    } else {
        match creep.upgrade_controller(&controller) {
            Ok(_) => {},
            Err(test) => {
                if let screeps::ErrorCode::NotInRange = test {
                    creep.better_move_to(creepmem, controller.pos(), 2);
                }
            },
        }
    }

    if inventory.get_free_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "work".to_string();
    }
}
