use log::info;
use screeps::{Creep, StructureController, HasPosition, ResourceType, find, game, ErrorCode};

use crate::{memory::CreepMemory, traits::{creep::CreepExtensions, room::RoomExtensions}, cache::ScreepsCache};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController, cache: &mut ScreepsCache) {
    let starting_cpu = game::cpu::get_used();
    let inventory = creep.store();
    if creepmem.s == "energy" {
        get_energy(creep, creepmem, cache);
    } else if creepmem.s == "work" {
        info!("     Move time controller: {:?}", game::cpu::get_used() - starting_cpu);
        upgrade(creep, creepmem, cache);
        info!("     Upgrade time {}", game::cpu::get_used() - starting_cpu);
    }
    if inventory.get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
    }
    if inventory.get_free_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "work".to_string();
    }
}

pub fn upgrade(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    info!("     Upgrading");
    let starting_cpu = game::cpu::get_used();
    let controller = creep.room().unwrap().controller().unwrap();
    if creep.better_is_near(controller.pos()) <= 3 {
        info!("Time to calculate upgrade {}", game::cpu::get_used() - starting_cpu);
        let _ = creep.upgrade_controller(&controller);
        info!("Cost: {}", game::cpu::get_used() - starting_cpu);
    } else {
        info!("     Caloling upfrade move");
        creep.better_move_to(creepmem, cache, controller.pos(), 2);
        info!("     Move time controller: {:?}", game::cpu::get_used() - starting_cpu);
    }
    info!("     Upgrade time: {:?}", game::cpu::get_used() - starting_cpu);
}

pub fn get_energy(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    info!("     Energy!");
    let starting_cpu = game::cpu::get_used();
    let closest_energy = cache.room_specific.get(&creep.room().unwrap().name_str()).unwrap().energy.first();
        if let Some(energy_id) = closest_energy {
            let energy = energy_id.resolve().unwrap();
            if creep.better_is_near(energy.pos()) <= 1 {
                let _ = creep.pickup(&energy);
                info!("     Pickup time: {:?}", game::cpu::get_used() - starting_cpu);
            } else {
                info!("     Before Move time energy: {:?}", game::cpu::get_used() - starting_cpu);
                info!("     Attempting energy at: x: {}, y: {}, room: {}", energy.pos().x(), energy.pos().y(), energy.pos().room_name());
                creep.better_move_to(creepmem, cache, energy.pos(), 1);
                info!("     Move time energy: {:?}", game::cpu::get_used() - starting_cpu);
            }
        }
}