use screeps::{find, memory, Creep, ErrorCode, HasPosition, ResourceType, SharedCreepProperties, StructureController};

use crate::{memory::{CreepMemory, ScreepsMemory}, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let creep_memory = memory.get_creep_mut(creep.name().as_str());

    let needs_energy = creep_memory.n_e.unwrap_or_else(|| false);
    let controller = creep.room().unwrap().controller().unwrap();

    if needs_energy{
        let closest_energy = creep
            .pos()
            .find_closest_by_path(find::DROPPED_RESOURCES, None);
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
    } else {
        match creep.upgrade_controller(&controller) {
            Ok(_) => {},
            Err(err) => {
                if let screeps::ErrorCode::NotInRange = err {
                    creep.better_move_to(creep_memory, controller.pos(), 2);
                } else if ErrorCode::NotEnough == err {
                    creep_memory.n_e = Some(true);
                }
            },
        }
    }
}
