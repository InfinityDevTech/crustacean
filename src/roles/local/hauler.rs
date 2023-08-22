use screeps::{Creep, SharedCreepProperties, HasPosition, find, ResourceType, StructureSpawn};

use crate::{memory::CreepMemory, movement};

pub fn haul(creep: &Creep, creepmem: &mut CreepMemory, deposit: StructureSpawn) {
    let name = creep.name();
    let inventory = creep.store();
    if inventory.get_free_capacity(None) > inventory.get_used_capacity(Some(ResourceType::Energy)) as i32 {
        let closest_energy = creep.pos().find_closest_by_path(find::DROPPED_RESOURCES, None);
        if let Some(energy) = closest_energy {
                if creep.pos().is_near_to(energy.clone().pos()) {
                    let _ = creep.pickup(&energy);
                } else {
                    movement::creep::move_to(&name, creepmem, energy.pos())
                }
            }
    } else if creep.pos().is_near_to(deposit.pos()) {
        let _ = creep.transfer(&deposit, ResourceType::Energy, Some(inventory.get_used_capacity(Some(ResourceType::Energy))));
    }
}
