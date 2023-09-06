use screeps::{Creep, StructureController, HasPosition, ResourceType, find};

use crate::{memory::CreepMemory, traits::{creep::CreepExtensions, room::RoomExtensions}, cache::ScreepsCache};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController, cache: &mut ScreepsCache) {
    let inventory = creep.store();
    if creepmem.s == "energy" {
        let closest_energy = cache.energy.get(&creep.room().unwrap().name_str()).unwrap().first();
        if let Some(energy_id) = closest_energy {
            let energy = energy_id.resolve().unwrap();
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
            } else {
                creep.better_move_to(creepmem, cache, energy.pos(), 1);
            }
        }
    } else {
        match creep.upgrade_controller(&controller) {
            Ok(_) => {},
            Err(test) => {
                if let screeps::ErrorCode::NotInRange = test {
                    creep.better_move_to(creepmem, cache, controller.pos(), 2);
                }
            },
        }
    }
    if inventory.get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
    }
    if inventory.get_free_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "work".to_string();
    }
}
