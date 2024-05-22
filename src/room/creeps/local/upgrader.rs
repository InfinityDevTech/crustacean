use screeps::{find, Creep, ErrorCode, HasPosition, ResourceType, SharedCreepProperties};

use crate::{memory::ScreepsMemory, room::object_cache::RoomStructureCache, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, _structure_cache: &RoomStructureCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    let needs_energy = creep_memory.needs_energy.unwrap_or(false);
    let controller = creep.room().unwrap().controller().unwrap();

    if needs_energy {
        let closest_energy = creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES);

        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.pos()) {
                let _ = creep.pickup(&energy);
                if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    creep_memory.needs_energy = None;
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
                creep_memory.needs_energy = Some(true);
            }
            _ => {}
        }
    } else {
        creep.better_move_to(creep_memory, controller.pos(), 2);
    }
}
