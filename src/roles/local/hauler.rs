use std::cmp::min;

use screeps::{
    find, Creep, HasPosition, ResourceType, SharedCreepProperties, Structure, StructureObject, HasTypedId,
};

use crate::{memory::{CreepMemory, Task}, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, deposit: Structure) {
    if creepmem.s == "energy" {
        get_energy(creep, creepmem);
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            creepmem.s = "work".to_string();
            haul_energy(creep, creepmem, deposit);
        }
    } else if creepmem.s == "work" && rename(creep, creepmem) {
        haul_energy(creep, creepmem, deposit);
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            creepmem.s = "energy".to_string();
            get_energy(creep, creepmem);
        }
    }
}

pub fn get_energy(creep: &Creep, creepmem: &mut CreepMemory) {
    let closest_energy = creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES);
        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
            } else {
                creep.better_move_to(creepmem, energy.pos(), 1)
            }
        }
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            creepmem.s = "work".to_string();
        }
}

pub fn haul_energy(creep: &Creep, creepmem: &mut CreepMemory, deposit: Structure) {
    let structure_object = StructureObject::from(deposit);
        if let Some(structure) = structure_object.as_transferable() {
            if structure_object
                .as_has_store()
                .unwrap()
                .store()
                .get_free_capacity(Some(ResourceType::Energy))
                > 0
            {
                if creep.pos().is_near_to(structure.pos()) {
                    let _ = creep.transfer(
                        structure,
                        ResourceType::Energy,
                        Some(min(
                            creep.store().get_used_capacity(Some(ResourceType::Energy)),
                            structure_object
                                .as_has_store()
                                .unwrap()
                                .store()
                                .get_free_capacity(Some(ResourceType::Energy)) as u32,
                    )));
                } else {
                    creep.better_move_to(creepmem, structure.pos(), 1);
                }
            } else {
                let find_res = creep.room().unwrap().find(find::MY_STRUCTURES, None);
                let new_target = find_res.iter().filter(|s| s.as_transferable().is_some()).find(|s| s.as_has_store().unwrap().store().get_free_capacity(Some(ResourceType::Energy)) > 0);
                if let Some(new_target) = new_target {
                    creepmem.t = Some(Task::Hauler(new_target.as_structure().id()));
                }
            }
        }
}

pub fn rename(creep: &Creep, creepmem: &mut CreepMemory) -> bool {
    if let Some(sign) = creep.room().unwrap().controller().unwrap().sign() {
        if sign.text() != "Ferris FTW!" {
            let controller = creep.room().unwrap().controller().unwrap();
            if creep.pos().is_near_to(controller.pos()) {
                let _ = creep.sign_controller(&controller, "Ferris FTW!");
                return false;
            } else {
                creep.better_move_to(creepmem, controller.pos(), 1);
                return false;
            }
        }
        true
    } else {
        let controller = creep.room().unwrap().controller().unwrap();
        if creep.pos().is_near_to(controller.pos()) {
            let _ = creep.sign_controller(&controller, "Ferris FTW!");
            false
        } else {
            creep.better_move_to(creepmem, controller.pos(), 1);
            false
        }
    }
}