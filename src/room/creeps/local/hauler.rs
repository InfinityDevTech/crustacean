use std::cmp::min;

use log::info;
use screeps::{
    find, game, Creep, HasPosition, HasTypedId, ResourceType, SharedCreepProperties, Structure,
    StructureObject,
};

use crate::{
    cache::ScreepsCache,
    memory::{CreepMemory, Task},
    traits::{creep::CreepExtensions, room::RoomExtensions},
};

pub fn run_creep(
    creep: &Creep,
    creepmem: &mut CreepMemory,
    deposit: Structure,
    cache: &mut ScreepsCache,
) {
    if creepmem.s == "energy" {
        get_energy(creep, creepmem, cache);
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            creepmem.s = "work".to_string();
            haul_energy(creep, creepmem, deposit, cache);
        }
    } else if creepmem.s == "work" && rename(creep, creepmem, cache) {
        haul_energy(creep, creepmem, deposit, cache);
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            creepmem.s = "energy".to_string();
            get_energy(creep, creepmem, cache);
        }
    }
}

pub fn get_energy(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    let starting_cpu = game::cpu::get_used();
    let closest_energy = cache.room_specific.get(&creep.room().unwrap().name_str()).unwrap().energy.first();
    if let Some(energy_id) = closest_energy {
        let energy = energy_id.resolve().unwrap();
        info!("     Find time: {:?}", game::cpu::get_used() - starting_cpu);
        if creep.better_is_near(energy.clone().pos()) <= 1 {
            let _ = creep.pickup(&energy);
            info!(
                "     Pickup time: {:?}",
                game::cpu::get_used() - starting_cpu
            );
        } else {
            creep.better_move_to(creepmem, cache, energy.pos(), 1);
            info!("     Move time: {:?}", game::cpu::get_used() - starting_cpu);
        }
    }
    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "work".to_string();
    }
    info!(
        "     Get energy CPU: {}",
        game::cpu::get_used() - starting_cpu
    );
}

pub fn haul_energy(
    creep: &Creep,
    creepmem: &mut CreepMemory,
    deposit: Structure,
    cache: &mut ScreepsCache,
) {
    let starting_cpu = game::cpu::get_used();
    let structure_object = StructureObject::from(deposit);
    if let Some(structure) = structure_object.as_transferable() {
        if structure_object
            .as_has_store()
            .unwrap()
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            > 0
        {
            info!("    Got structure {}", game::cpu::get_used() - starting_cpu);
            if creep.better_is_near(structure.pos()) <= 1 {
                let _ = creep.transfer(
                    structure,
                    ResourceType::Energy,
                    Some(min(
                        creep.store().get_used_capacity(Some(ResourceType::Energy)),
                        structure_object
                            .as_has_store()
                            .unwrap()
                            .store()
                            .get_free_capacity(Some(ResourceType::Energy))
                            as u32,
                    )),
                );
                info!("    Transfered {}", game::cpu::get_used() - starting_cpu);
            } else {
                info!("    Before move {}", game::cpu::get_used() - starting_cpu);
                creep.better_move_to(creepmem, cache, structure.pos(), 1);
                info!("    Moved {}", game::cpu::get_used() - starting_cpu);
            }
        } else {
            //let structures = cache
            //    .structures
            //    .values()
            //    .flatten()
            //    .filter(|s| {
            //        s.resolve().unwrap().room().unwrap().name() == creep.room().unwrap().name()
            //    })
            //    .find(|s| {
            //        StructureObject::from(s.resolve().unwrap())
            //            .as_transferable()
            //            .is_some()
            //    });
            //if let Some(transferrable) = structures {
            //    creepmem.t = Some(Task::Hauler(*transferrable));
            //}
            let find_res = creep.room().unwrap().find(find::MY_STRUCTURES, None);
                let new_target = find_res.iter().filter(|s| s.as_transferable().is_some()).find(|s| s.as_has_store().unwrap().store().get_free_capacity(Some(ResourceType::Energy)) > 0);
                if let Some(new_target) = new_target {
                    creepmem.t = Some(Task::Hauler(new_target.as_structure().id()));
                }
        }
        info!(
            "     Haul energy CPU: {}",
            game::cpu::get_used() - starting_cpu
        );
    }
}

pub fn rename(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) -> bool {
    if let Some(sign) = creep.room().unwrap().controller().unwrap().sign() {
        if sign.text() != "Ferris FTW!" {
            let controller = creep.room().unwrap().controller().unwrap();
            if creep.better_is_near(controller.pos()) <= 1 {
                let _ = creep.sign_controller(&controller, "Ferris FTW!");
                return false;
            } else {
                creep.better_move_to(creepmem, cache, controller.pos(), 1);
                return false;
            }
        }
        true
    } else {
        let controller = creep.room().unwrap().controller().unwrap();
        if creep.better_is_near(controller.pos()) <= 1 {
            let _ = creep.sign_controller(&controller, "Ferris FTW!");
            false
        } else {
            creep.better_move_to(creepmem, cache, controller.pos(), 1);
            false
        }
    }
}
