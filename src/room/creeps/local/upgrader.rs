use log::info;
use screeps::{Creep, StructureController, HasPosition, ResourceType, find, game};

use crate::{memory::CreepMemory, traits::{creep::CreepExtensions, room::RoomExtensions}, cache::ScreepsCache};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController, cache: &mut ScreepsCache) {
    let starting_cpu = game::cpu::get_used();
    let inventory = creep.store();
    if creepmem.s == "energy" {
        let closest_energy = cache.energy.get(&creep.room().unwrap().name_str()).unwrap().first();
        if let Some(energy_id) = closest_energy {
            let energy = energy_id.resolve().unwrap();
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
                info!("     Pickup time: {:?}", game::cpu::get_used() - starting_cpu);
            } else {
                creep.better_move_to(creepmem, cache, energy.pos(), 1);
                info!("     Move time energy: {:?}", game::cpu::get_used() - starting_cpu);
            }
        }
    } else if !creep.pos().in_range_to(controller.pos(), 2) {
        creep.better_move_to(creepmem, cache, controller.pos(), 2);
        info!("     Move time controller: {:?}", game::cpu::get_used() - starting_cpu);
    }
    if inventory.get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
    }
    if inventory.get_free_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "work".to_string();
    }
}
