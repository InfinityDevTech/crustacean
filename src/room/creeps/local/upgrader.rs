use screeps::{find, Creep, ErrorCode, HasPosition, ResourceType, SharedCreepProperties};

use crate::{memory::ScreepsMemory, room::structure_cache::RoomStructureCache, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, _structure_cache: &RoomStructureCache) {
    let creep_memory = memory.get_creep_mut(creep.name().as_str());

    let needs_energy = creep_memory.n_e.unwrap_or(false);
    let controller = creep.room().unwrap().controller().unwrap();

    if needs_energy {
        let closest_energy = creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES);

        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
                if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    creep_memory.n_e = None;
                }
            } else {
                creep.better_move_to(creep_memory, energy.pos(), 1);
            }
        }
    } else if creep.pos().get_range_to(controller.pos()) <= 2 {
        match creep.upgrade_controller(&controller) {
            Ok(_) => {}
            Err(ErrorCode::NotInRange) => {
                creep.better_move_to(creep_memory, controller.pos(), 2);
            }
            Err(ErrorCode::NotEnough) => {
                creep_memory.n_e = Some(true);
            }
            _ => {}
        }
    } else {
        creep.better_move_to(creep_memory, controller.pos(), 2);
    }
}
